# Provas da rodada — data, comando e resultado de cada medição

**Gerado por `medir_provas.py` em 2026-10-06 18:23 UTC. Código medido (SHA-256 de src, tests, examples, Cargo.toml e Cargo.lock): `31b0cc409002f4a01673eda207f5a625079ae94e7e428ea77e678762d185c802`.** Número de prova sem data e comando de uma execução desta rodada não vale; documento que cite número sem apontar para cá deve ser tratado como não verificado.

## Provas rápidas (medidas nesta execução)

| Prova | Resultado | Data (UTC) | Comando | SHA-256 da saída (16) |
|---|---|---|---|---|
| Catraca | aprovado — Catraca: nenhuma proteção a menos. | 2026-10-06 18:20 UTC | `python3 verificacao/catraca.py` (em `nimbus-nucleo`) | c325144fa4350fda |
| Catraca atacada (afrouxamentos plantados) | aprovado — Catraca: 31 de 31 afrouxamentos pegos. | 2026-10-06 18:20 UTC | `python3 verificacao/testar_catraca.py` (em `nimbus-nucleo`) | ece38a980a00de50 |
| Auditor de dependências atacado | aprovado — Auditor de dependências: 7 de 7 casos corretos. | 2026-10-06 18:20 UTC | `python3 verificacao/testar_auditoria.py` (em `nimbus-nucleo`) | 6125b693ce42f5ca |
| verificar.sh atacado (falha fechada) | aprovado — verificar.sh: 9 de 9 situações tratadas em falha fechada. | 2026-10-06 18:20 UTC | `python3 verificacao/testar_verificar.py` (em `nimbus-nucleo`) | f8c0755ddcb49241 |
| Defeitos plantados apontam para código que existe | aprovado — Defeitos plantados: os 288 trechos existem exatamente uma vez no código. | 2026-10-06 18:21 UTC | `python3 verificacao/mutantes.py --conferir` (em `nimbus-nucleo`) | 3b10a55faa775943 |
| Auditoria de dependências (base RustSec do dia) | aprovado — Auditoria de dependências: 84 versões (71 bibliotecas), 17 avisos conferidos, base de 3 dia(s); nenhuma vulnerabilidade conhecida em aberto. | 2026-10-06 18:21 UTC | `python3 verificacao/auditar_dependencias.py` (em `nimbus-nucleo`) | f86af5d13b695d91 |
| Portões de governança atacados | aprovado — Portões de governança: 20 de 20 casos corretos. | 2026-10-06 18:21 UTC | `python3 kits_comum/portoes_de_governanca.py --teste` (em `.`) | d53e6a7084543b68 |
| Catraca dentro do kit NimbusOne (inclui escudo nativo e regras do repositório do kit) | aprovado — Catraca: nenhuma proteção a menos. | 2026-10-06 18:21 UTC | `python3 core/nimbus-nucleo/verificacao/catraca.py servidor/banco` (em `kits/nimbusone`) | c325144fa4350fda |
| Catraca dentro do kit NimbusChat (inclui escudo nativo e regras do repositório do kit) | aprovado — Catraca: nenhuma proteção a menos. | 2026-10-06 18:21 UTC | `python3 core/nimbus-nucleo/verificacao/catraca.py servidor/banco` (em `kits/nimbuschat`) | c325144fa4350fda |
| Consistência do kit NimbusOne | aprovado — Consistência: tudo certo. | 2026-10-06 18:21 UTC | `python3 verificacao/consistencia.py` (em `kits/nimbusone`) | 7e5ac642e0298630 |
| Consistência do kit NimbusChat | aprovado — Consistência: tudo certo. | 2026-10-06 18:21 UTC | `python3 verificacao/consistencia.py` (em `kits/nimbuschat`) | 7e5ac642e0298630 |
| Conferidores da raiz atacados | aprovado — Conferidores atacados: 25 de 25 resultados corretos. | 2026-10-06 18:21 UTC | `python3 testar_conferidores.py` (em `.`) | c4aab885d9f193b3 |
| Portão de entrega atacado | aprovado — Portão de entrega atacado: 16 de 16 defeitos plantados recusados ou casos certos aceitos. | 2026-10-06 18:22 UTC | `python3 entregar.py --testar` (em `.`) | 99a448cd68b9abf8 |
| Números e fatos em todos os materiais | aprovado — ✓ números e fatos conferem em 183 materiais; identificadores únicos; documentos gerados e cópias dos kits em dia | 2026-10-06 18:22 UTC | `python3 verificar_numeros.py` (em `.`) | 9d2375d533449d21 |
| Texto invisível em todos os materiais (inclusive dentro de .docx, .pptx e .zip) | aprovado — Varredura de texto invisível: nenhuma ocorrência em fontes, kits e saídas (inclusive dentro de .docx, .pptx e .zip); ataques plantados pegos | 2026-10-06 18:23 UTC | `python3 varrer_invisiveis.py` (em `.`) | f501eec0fe5d1f1b |
| Sintaxe do escudo nativo (não é compilação) | aprovado — Sintaxe do escudo nativo: aprovada (erro plantado pego; isto não é compilação) | 2026-10-06 18:23 UTC | `python3 conferir_sintaxe_nativa.py` (em `.`) | 38b2f7e73f77696b |

## Provas longas (registros em `provas_logs/`)

Cada registro longo traz ao lado (`.codigo`) a identidade do código em que foi medido; registro de outro código ou sem identificação aparece como NÃO VALE e reprova este medidor. A mutação rodou em faixas porque a máquina de medição reinicia; os nº 205 e 215 foram corrigidos e rodados à parte.

| Prova | Resultado | Data do registro (UTC) | Comando | SHA-256 do registro (16) |
|---|---|---|---|---|
| Fuzzing profundo (2 milhões por alvo) — `fuzz2m.log` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 630.96s | 2026-10-06 16:10 UTC | `env FUZZ_RODADAS=2000000 cargo-1.91 test --profile verificacao --test fuzzing --locked -- --ignored` | e74e970d2368d7f5 |
| Mutação (faixa) — `mut_001a036.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 1 a 36 de 288). | 2026-10-06 16:18 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 1 36` | 85223101710d86df |
| Mutação (faixa) — `mut_037a072.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 37 a 72 de 288). | 2026-10-06 16:39 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 37 72` | a94b321206e04afb |
| Mutação (faixa) — `mut_073a108.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 73 a 108 de 288). | 2026-10-06 16:46 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 73 108` | 2c41607826af3100 |
| Mutação (faixa) — `mut_109a144.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 109 a 144 de 288). | 2026-10-06 16:53 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 109 144` | 2934570c1a0ee01e |
| Mutação (faixa) — `mut_145a180.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 145 a 180 de 288). | 2026-10-06 17:00 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 145 180` | 52dc3ac20e7787d1 |
| Mutação (faixa) — `mut_181a216.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 181 a 216 de 288). | 2026-10-06 17:07 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 181 216` | 287f0916fc58f129 |
| Mutação (faixa) — `mut_217a252.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 217 a 252 de 288). | 2026-10-06 17:30 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 217 252` | 6e14813005708ea3 |
| Mutação (faixa) — `mut_253a288.log` | Resultado: 36 de 36 defeitos plantados foram pegos pelos testes (mutantes 253 a 288 de 288). | 2026-10-06 17:37 UTC | `env CARGO=cargo-1.91 python3 -u verificacao/mutantes.py 253 288` | c0162a9770279535 |
| Tempo de recusa — `recusa.log` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s | 2026-10-06 15:55 UTC | `cargo-1.91 test --release --test tempo_de_recusa --locked -- --ignored` | 6f541db08ad84f47 |
| suite_completa.log — `suite_completa.log` | test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s | 2026-10-06 16:00 UTC | `cargo-1.91 test --locked --no-fail-fast` | e1cd675a3421968a |
| Tempo constante (C80) — `timing.log` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.12s | 2026-10-06 15:53 UTC | `cargo-1.91 test --profile verificacao --test timing --locked -- --ignored` | 68ec0d8d5316f6d8 |

## Verificar antes de usar

1. A prova que vale para liberar é o `./verificar.sh` completo rodando inteiro no CI, num comando só; este registro é a medição desta rodada, não substitui o selo.
2. As datas das provas longas vêm da data de modificação do arquivo de registro; conferir pelo conteúdo e pelo hash.
