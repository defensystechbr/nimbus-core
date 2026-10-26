# Checklist de segurança do back-end — Nimbus

**Versão 37 · Confidencial**

Este documento responde, uma a uma, às cinco perguntas clássicas de segurança de back-end (rotas autenticadas, senhas no banco, limite de tentativas, RLS e dados criptografados) e vai além delas, com a pergunta que mais importa: **e se a porta abrir, o que o atacante leva?**

## 1. As cinco perguntas, respondidas para o Nimbus

| Pergunta | Resposta | Onde está e como se prova |
|---|---|---|
| **Protegeu as rotas da API com autenticação?** | Sim, e de um jeito mais forte que login: **negar por padrão** (só existem as rotas do inventário, comparadas por igualdade exata — `..`, `//`, `%2e`, maiúscula e barra no fim não existem); **prova por recurso, não por pessoa** (a caixa de mensagens é aberta com a chave daquela caixa, e o servidor nunca sabe quem é a pessoa nem liga duas caixas entre si); a assinatura cobre método, caminho, corpo, horário e um número de uso único; janela de 60 s; pedido repetido recusado; revogação imediata; chave de caixa não abre rota de aparelho | `autenticacao::rota`, `autenticacao::Porteiro::autorizar`; prova C104 |
| **Criptografou as senhas dos usuários no banco?** | Correção importante: senha **não se criptografa** (criptografia se desfaz com a chave); guarda-se **resumo lento com sal**. No Nimbus, porém, **não existe senha no servidor**: a conta é um par de chaves no chip do aparelho. O PIN nunca sai do aparelho e é conferido com Argon2id (64 MiB, 3 passagens), em tempo constante. A senha opcional do link do Envio também é endurecida com Argon2id, no navegador. Não há o que vazar | `senha::derivar`, `coacao`; prova C80 |
| **Validou a quantidade de tentativas de login?** | Não há login com senha para adivinhar. Mesmo assim: no aparelho, o PIN tem erros livres, espera que dobra (medida em tempo real, que não acelera mudando o relógio) e apagamento no limite; no servidor, toda rota que dá para martelar tem **cota por janela, castigo que dobra a cada falha até 24 h e memória com teto** (falha fechada). Convites e links têm 256 bits: adivinhar é inviável, e ainda assim há limite | `coacao::Tentativas`, `coacao::RelogioDeEspera`, `limites::Limitador`; provas C82, C105 |
| **Criou as políticas de RLS nas tabelas?** | Sim, em PostgreSQL 16, **ligadas e forçadas em todas as tabelas**: sem a caixa autorizada pelo porteiro na transação, nenhuma linha aparece; depósito e criação de caixa só com ficha; prazo máximo de 48 h garantido pelo **próprio banco**; nenhuma alteração de bloco; registros de acesso só de escrita; o app não é dono das tabelas e não ignora RLS. **Testado num banco real** e com **12 defeitos plantados no esquema, todos pegos** | `servidor/banco/esquema.sql`, `teste_do_banco.sh`, `mutacao_do_banco.sh`; prova C106 |
| **Salvou os dados do usuário com criptografia?** | Sim, e antes de saírem do aparelho: tudo é cifrado ponta a ponta (troca de chaves híbrida pós-quântica, XChaCha20-Poly1305), com tamanho padronizado. O servidor nunca tem chave de conteúdo. Além disso, o disco do banco e as cópias de segurança são cifrados (camada extra), e os registros de acesso exigidos pelo Marco Civil são cifrados para uma chave que **não fica no servidor** | `envelope`, `prechaves`, `cofre`; provas C1, C102 |

## 2. E se a porta abrir?

Proteger não basta: cada camada parte do princípio de que a anterior pode falhar.

| Se falhar... | A próxima camada segura | O que o atacante leva no fim |
|---|---|---|
| A autenticação das rotas (erro no servidor) | RLS no banco: sem caixa autorizada na transação, nenhuma linha | Nada |
| A RLS (erro na política ou credencial do dono vazada) | Tudo no banco é bloco cifrado de ponta a ponta | Blocos cifrados, de tamanho padronizado, com prazo de até 48 h |
| O banco inteiro é copiado | Sem chave de conteúdo no servidor; chaves de recebimento destruídas depois do prazo | O mesmo: blocos cifrados |
| O servidor inteiro é tomado | O servidor nunca teve chave de conteúdo; o IP do aparelho para no retransmissor | Metadados mínimos daquela janela (que caixa recebeu um bloco, quando) |
| A chave de algum aparelho é roubada | Revogação imediata; registro público com prova de consistência pega chave plantada | O que aquele aparelho ainda tinha, dentro do prazo |
| Um computador quântico aparece no futuro | Troca de chaves e assinatura híbridas pós-quânticas | Nada do tráfego gravado |

## 3. OWASP API Security Top 10 (2023), item por item

| Risco | Situação no Nimbus |
|---|---|
| API1 — Autorização quebrada por objeto (acessar o objeto de outro) | Prova por recurso (`autenticacao`) **e** RLS por caixa no banco: duas camadas independentes |
| API2 — Autenticação quebrada | Sem senha; assinatura híbrida por pedido; janela curta; uso único; revogação imediata |
| API3 — Autorização quebrada por propriedade (ler ou mudar campo que não devia) | Não há campos editáveis: bloco não se altera; respostas sem campos além do bloco |
| API4 — Consumo de recursos sem limite | Limite de corpo por rota; cota e castigo (`limites`); memória com teto; envelope até 64 MiB |
| API5 — Autorização quebrada por função (rota de administrador exposta) | Não existe rota de administração na API pública; rotas de aparelho exigem chave de aparelho |
| API6 — Fluxo sensível sem proteção (automação abusiva) | Depósito só com ficha de licença; convite de uso único; recuperação com espera de 3 dias |
| API7 — Servidor busca endereço vindo do usuário (SSRF) | O servidor nunca busca URL recebida de fora; nenhuma rota aceita endereço como parâmetro |
| API8 — Configuração insegura | Rotas de diagnóstico proibidas (lição do caso TeleMessage); regras de varredura; cabeçalhos fixos; sem mensagens de erro detalhadas |
| API9 — Inventário mal gerido (rota esquecida) | Inventário único e testado em `autenticacao::ROTAS`; o que não está lá não existe |
| API10 — Consumo inseguro de APIs de terceiros | O servidor não consome API de terceiros com dado de usuário; fichas de pagamento conferidas por assinatura |

## 4. O que vai além do checklist

1. **Transporte:** TLS 1.3 entre retransmissor e servidor; o aparelho fala com o servidor por retransmissores independentes (OHTTP), e o servidor nunca vê o IP do aparelho.
2. **Conexão com o banco:** por certificado, em rede privada; o papel do app não é dono, não é superusuário e não ignora RLS; migrações aplicadas por papel separado, com revisão de duas pessoas.
3. **Segredos do servidor** (chave que assina a hora, chave de TLS): em módulo de hardware ou cofre da nuvem; nunca em arquivo, variável de ambiente ou repositório. **Vale para o núcleo e o servidor Nimbus descritos neste documento; ainda não vale para `pagamento_web/servidor-cobranca`** — servidor de referência separado (cobrança via Stripe, não embarcado nos kits de entrega) que hoje lê duas chaves por variável de ambiente simples; achado, documentado e com recomendação de correção em `PENDENCIAS_E_DECISOES.md` §51.
4. **Erros:** respostas genéricas, sem rastro de pilha nem "quase acertou"; registro interno sem conteúdo, nome, chave ou endereço.
5. **Entrada:** formatos binários de tamanho fixo ou com limite declarado; leitores em Rust que recusam entrada malformada sem cair (fuzzing de 19 leitores).
6. **Faxina automática:** o que venceu é apagado do banco e da memória (`nimbus.apagar_vencidos`, papel próprio).
7. **Cópias de segurança:** cifradas, com restauração testada; e, ainda assim, só com bloco cifrado com prazo.
8. **Monitoramento:** só contagens anônimas (tentativas recusadas por tipo); alerta quando sobem.
9. **Resposta a incidente:** procedimento em `NimbusOne_Runbooks_Operacionais.md`.

## 5. Verificar antes de usar

1. O texto oficial da OWASP API Security Top 10 de 2023, para conferir os títulos dos itens.
2. O comportamento da RLS do PostgreSQL usado em produção (a versão testada aqui é a 16); rodar `teste_do_banco.sh` e `mutacao_do_banco.sh` a cada mudança de esquema e de versão do banco.
3. Os limites de cota e castigo de cada rota precisam ser calibrados com o uso real do piloto.
