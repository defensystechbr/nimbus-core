# 05 · Esquema Backend — Servidor Nimbus

**Versão 37 · Código pronto e testado: módulos `autenticacao`, `limites`, `sentinela`, `selo`, `metadados`, `chaves_do_servidor`, `prechaves`, `registro`; banco em `servidor/banco/`**

## 1. Caminho de um pedido (cada camada supõe que a anterior falhou)

1. **Retransmissor (OHTTP)** — tira o IP do aparelho.
2. **Origem** — `autenticacao::origem_permitida`: app nativo não manda Origin; navegador só na rota do Envio e só da origem oficial. Sem CORS.
3. **Rota exata** — `autenticacao::rota`: o que não está no inventário não existe; parâmetro na URL faz a rota não existir.
4. **Tamanho** — limite de corpo por rota.
5. **Limite de uso** — antes de qualquer conta cara; castigo que dobra até 24 h.
6. **Prova** — assinatura híbrida que cobre método, caminho, corpo, titular, hora e número de uso único; janela de 60 s; repetição recusada.
7. **Resposta genérica** — `autenticacao::resposta_publica`: toda falha de autorização vira "401 nao autorizado"; nada revela se a caixa existe.
8. **Banco** — transação com `SELECT set_config('nimbus.caixa', $1, true)`, o índice cego da caixa autorizada passado **como parâmetro** (ou `nimbus.deposito`); nunca `SET LOCAL` montado com texto; RLS forçada como segunda barreira contra acesso a objeto de outro (IDOR).
9. **Disco** — bloco selado de novo com a chave do dia (`metadados::ChavesDeIndice::selar_no_disco`).
10. **Cabeçalhos** — `autenticacao::cabecalhos_da_api`: sem cache, sem moldura, sem farejamento de tipo, HSTS.

## 2. Rotas (11; inventário completo em `ACESSO_SEM_AUTENTICACAO.md`, gerado do código)

| Rota | Exige | Limite/min |
|---|---|---|
| POST /v1/hora | Nada (resposta assinada) | 120 |
| GET /v1/prechave | Nada (dado assinado na origem) | 120 |
| POST /v1/convite/resgatar | Código de uso único | 10 |
| POST /v1/caixa/criar | Ficha anônima | 10 |
| POST /v1/caixa/depositar | Ficha anônima | 120 |
| POST /v1/caixa/retirar | Chave da caixa | 120 |
| POST /v1/caixa/apagar | Chave da caixa | 120 |
| POST /v1/prechave/publicar | Chave do aparelho | 10 |
| POST /v1/aparelho/registrar | Chave do aparelho | 5 |
| POST /v1/aparelho/revogar | Chave do aparelho | 10 |

## 3. Banco (PostgreSQL 16)

| Tabela | Colunas | Observação |
|---|---|---|
| `nimbus.caixas` | `endereco` (índice cego, 32 B), `chave_publica`, `criada_em` | Ler só a própria; criar só com ficha |
| `nimbus.blocos` | `id`, `endereco`, `bloco` (256 B a 64 MiB), `criado_em`, `expira_em` | Prazo máximo de 48 h no próprio banco; sem UPDATE |
| `nimbus.registros_de_acesso` | `cifrado`, `guardar_ate` | Só escrita; selado para a chave jurídica |

Papéis: `nimbus_dono` (migrações), `nimbus_app` (sem superusuário, sem ignorar RLS, não é dono), `nimbus_faxina` (só apaga vencidos). Testes num PostgreSQL real e 15 de 15 defeitos plantados no esquema pegos.

## 4. O que o servidor nunca guarda

Conteúdo legível, chave de conteúdo, IP do aparelho, nome, telefone, e-mail, documento, lista de contatos, endereço de caixa em claro (só o índice cego), registro legal legível.

## 5. Os testes clássicos de API, um a um

| Teste | Situação no Nimbus | Prova |
|---|---|---|
| Rate limit ausente | Toda rota tem limite obrigatório (a catraca reprova rota sem limite); por origem, titular ou ficha — o servidor não vê o IP do aparelho, por desenho | `autenticacao`, `limites`; C105, C107 |
| CORS refletindo a origem | Não existe CORS; origem refletida, curinga e "null" recusados | `origem_permitida`; teste |
| Dado demais na resposta / hash de senha | Não existe senha nem dado pessoal no servidor; respostas só com o bloco ou o código de estado | `resposta_publica` |
| Token na URL / token vivo depois de sair | Não existe token de sessão: cada pedido é assinado e vale 60 s uma vez; parâmetro na URL faz a rota não existir; revogação imediata do aparelho | C104; teste da URL |
| Enumeração de usuário | Não existe usuário consultável; toda falha de autorização dá a mesma resposta | `resposta_publica`; teste |
| Clickjacking | `X-Frame-Options: DENY` e `frame-ancestors 'none'` na API; não existe página web | `cabecalhos_da_api` |
| Injeção de SQL | Regra de varredura proíbe SQL montado com texto; RLS como segunda barreira | `nimbus-sql-montado-com-texto`; C106 |
| Acesso a objeto de outro (IDOR) | Prova por recurso + RLS por caixa | C104, C106 |

## 6. Operação

Partida: `sentinela::autoverificar_com_selo` + `verificar_banco_na_partida.sql`; sem achado nenhum, atende. A cada hora: a mesma verificação, `ChavesDeIndice::destruir_vencidas`, `prechaves` e certificado do dia. Memória de troca e despejo de falha desligados. Segredos só no cofre.
