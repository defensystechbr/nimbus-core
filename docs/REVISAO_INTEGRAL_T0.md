# Revisão integral do código atual — Tarefa T0

**Versão 37 · 03/10/2026 · Confidencial · Pré-condição de toda tarefa (T1 em diante)**

> **Atualização de 26/10/2026 — leia antes do resto.** A T0 na parte que dependia do repositório está **concluída**:
> a proteção da `principal` está ativa e aplicada a partir de `.github/regras/protecao-principal.json`, o código
> atual entrou por um único pedido de importação submetido a ela, e o CI do kit roda verde. A **revisão por duas
> pessoas** descrita abaixo foi substituída por **aprovador único, o dono** (`docs/DECISAO_APROVADOR_UNICO.md`),
> porque o autor não pode aprovar o próprio pedido e não havia segunda pessoa. As seções 2 (passos 2 e 4), 3 e 5
> descrevem portanto o regime **alvo**, não o vigente. O que continua valendo sem exceção: proteção ativa antes do
> primeiro commit de trabalho, `./verificar.sh` completo e verde em todo pedido, e nenhum passe livre na branch.

## 1. Por que existe

Todo o código entregue até aqui (núcleo, verificações, banco, kits e documentos) foi escrito **sem** a proteção da branch ativa: nenhum commit passou pela regra de duas aprovações, pelo commit assinado nem pelos portões de governança. A regra do projeto é que nenhuma mudança entra por uma pessoa só. Por isso o código atual não é tratado como aprovado: ele entra no repositório protegido como **um único pedido de importação**, revisado inteiro por duas pessoas, e só depois disso qualquer tarefa nova começa.

## 2. Ordem obrigatória

1. Um administrador aplica a proteção da branch a partir de `.github/regras/protecao-principal.json` e liga a administração dupla (`GUIA_DE_LIBERACAO.md` §2). O repositório começa vazio, só com o README.
2. O vigia da proteção roda e aprova: portões `protecao_do_repositorio` e `administracao_dupla` (`python3 verificacao/portoes_de_governanca.py`).
3. Uma pessoa abre o pedido de importação com o conteúdo exato do pacote entregue. O SHA-256 de cada ZIP é conferido contra o manifesto da transferência antes do commit.
4. Duas pessoas, nenhuma delas autora do pedido, revisam **todos** os arquivos da tabela abaixo e marcam cada linha. A IA não conta como revisora.
5. `./verificar.sh` roda no CI do pedido e termina com "Tudo certo" (verificação completa, falha fechada). Etapa pulada reprova.
6. As duas assinam o termo (§5). Só então o pedido é aceito e a T1 pode ser aberta.

Commit de trabalho novo feito antes do passo 6 é descartado e refeito.

## 3. O que cada revisor confere em cada arquivo

- Faz o que o requisito citado diz, e nada além (sem rede, log, arquivo ou chamada escondidos).
- Nenhum segredo, chave, endereço ou dado pessoal escrito no código.
- Nenhuma criptografia fora das bibliotecas aprovadas; nenhum `unsafe` fora de `src/memoria.rs`.
- Erros fecham (falha fechada); nenhum caminho aceita por padrão.
- Cada teste testa de verdade (não passa com o código apagado); o defeito plantado correspondente existe em `verificacao/mutantes.py`.
- Comentários e documentos não prometem mais do que o código faz.

## 4. Lista de revisão

### 4.1 Módulos do núcleo (45)

| # | Arquivo | Requisitos | Revisor 1 (iniciais) | Revisor 2 (iniciais) | Achados |
|---|---|---|---|---|---|
| 1 | `src/acesso.rs` | ANO9–ANO14 | | | |
| 2 | `src/album.rs` | ALB1–ALB8 | | | |
| 3 | `src/ambiente.rs` | AMB1–AMB4 | | | |
| 4 | `src/apresentacao.rs` | APR1–APR8 | | | |
| 5 | `src/aprovacao.rs` | ASS1–ASS12, BL4b | | | |
| 6 | `src/assinatura.rs` | PQS1–PQS6 | | | |
| 7 | `src/autenticacao.rs` | SRV1–SRV6 | | | |
| 8 | `src/chamada.rs` | C88 | | | |
| 9 | `src/chaves_do_servidor.rs` | ROT1–ROT4 | | | |
| 10 | `src/coacao.rs` | AL2, AR6 | | | |
| 11 | `src/cofre.rs` | COF1–COF8 | | | |
| 12 | `src/contatos.rs` | CTT1–CTT8 | | | |
| 13 | `src/conversa.rs` | HIS1–HIS16 | | | |
| 14 | `src/convites.rs` | SPAM1–SPAM6 | | | |
| 15 | `src/corporativo.rs` | COR1–COR24 | | | |
| 16 | `src/custodia.rs` | GUA1–GUA8 | | | |
| 17 | `src/direto.rs` | DIR1–DIR8, ND2–ND5 | | | |
| 18 | `src/envelope.rs` | CRY1, CRY3, CRY9, ARQ4, FP1, FP2 | | | |
| 19 | `src/envio.rs` | ENV1–ENV9 | | | |
| 20 | `src/escudo.rs` | EC1–EC9 | | | |
| 21 | `src/expiracao.rs` | CH2, OFF2 | | | |
| 22 | `src/higiene.rs` | HIG1–HIG6 | | | |
| 23 | `src/historico.rs` | EF5, EF6, CH4 | | | |
| 24 | `src/inspecao.rs` | INS1–INS9 | | | |
| 25 | `src/integridade.rs` | DV2, DV3, PC1–PC5 | | | |
| 26 | `src/liberacao.rs` | LIB1–LIB16 | | | |
| 27 | `src/limites.rs` | SRV7–SRV9 | | | |
| 28 | `src/memoria.rs` | RZ3, RZ11, RZ12 | | | |
| 29 | `src/metadados.rs` | MET1–MET5 | | | |
| 30 | `src/modos.rs` | MOD1–MOD8 | | | |
| 31 | `src/notificacao.rs` | NOT1–NOT7 | | | |
| 32 | `src/pacote.rs` | MTA1–MTA3 | | | |
| 33 | `src/panico.rs` | PAN1–PAN9 | | | |
| 34 | `src/politica.rs` | CH2, CH3, AR2 | | | |
| 35 | `src/prechaves.rs` | FS1–FS5 | | | |
| 36 | `src/recuperacao.rs` | REC1–REC6 | | | |
| 37 | `src/rede.rs` | NET1–NET14 | | | |
| 38 | `src/registro.rs` | BL1, BL6, CO7, BC2 | | | |
| 39 | `src/selo.rs` | SLO1–SLO7 | | | |
| 40 | `src/senha.rs` | SEN1–SEN4 | | | |
| 41 | `src/sentinela.rs` | SEN1–SEN5 | | | |
| 42 | `src/sessoes.rs` | ID2, CG1 | | | |
| 43 | `src/trafego.rs` | ANO1–ANO3 | | | |
| 44 | `src/validade.rs` | VAL1–VAL8 | | | |
| 45 | `src/voz.rs` | VZ4, VC4, VN1, VN2 | | | |

### 4.2 Testes, verificações e configuração do núcleo

| Arquivo ou pasta | Revisor 1 | Revisor 2 | Achados |
|---|---|---|---|
| `tests/adversario_ativo.rs` | | | |
| `tests/aparelho_real.rs` | | | |
| `tests/ataque_vermelho.rs` | | | |
| `tests/criptografia.rs` | | | |
| `tests/fuzzing.rs` | | | |
| `tests/propriedades.rs` | | | |
| `tests/robustez.rs` | | | |
| `tests/servidor_atestacao.rs` | | | |
| `tests/tempo_de_recusa.rs` | | | |
| `tests/timing.rs` | | | |
| `src/lib.rs` (proibição de `unsafe`, lista de módulos) | | | |
| `verificacao/catraca.py`, `CATRACA.json`, `verificacao/testar_catraca.py` | | | |
| `verificacao/auditar_dependencias.py`, `verificacao/testar_auditoria.py` | | | |
| `verificar.sh`, `verificacao/testar_verificar.py`, `verificacao/mutantes.py` | | | |
| `nimbus-regras.yml`, `.semgrepignore`, `.gitleaks.toml`, `deny.toml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | | | |
| `.github/workflows/*`, `.github/regras/*`, `CODEOWNERS` | | | |
| `examples/*` | | | |

### 4.3 Kits e servidor

| Arquivo ou pasta | Revisor 1 | Revisor 2 | Achados |
|---|---|---|---|
| `servidor/banco/*` (esquema, RLS, testes e defeitos plantados do banco) | | | |
| `verificacao/portoes_de_governanca.py`, `verificacao/consistencia.py`, `verificacao/verificar_kit.py` | | | |
| `.github/workflows/*` dos kits (CI, sentinela diária, liberação) | | | |
| `CLAUDE.md`, `.claude/settings.json`, `prompts/*` | | | |
| `docs/*` (telas, plano de tarefas, manual por produto) | | | |

## 5. Termo de revisão

Declaramos que revisamos, cada um por conta própria, todos os arquivos listados acima, no commit `__________________` (SHA completo), que conferimos o SHA-256 dos pacotes contra o manifesto da transferência, e que todo achado está registrado na coluna "Achados" com o pedido de correção correspondente.

| | Revisor 1 | Revisor 2 |
|---|---|---|
| Nome | | |
| Função | | |
| Data | | |
| Assinatura | | |

Este termo é guardado junto do registro de liberação (`GUIA_DE_LIBERACAO.md`) e é exigido pelo item 0 do `CHECKLIST_DE_ENTRADA_EM_PRODUCAO.md`.

## 6. Verificar antes de usar

1. Que a regra aplicada no GitHub é igual à do arquivo `.github/regras/protecao-principal.json` (o vigia compara sozinho; conferir o resultado dele).
2. Que nenhum dos dois revisores é autor do pedido de importação nem tem o papel de quem pode aprovar sozinho na organização.
