# Nimbus — comece aqui (técnicos e engenheiros)

**Versão 37 · revisada em 03/10/2026 · Confidencial**

> **Regras de trabalho e de entrega:** `REGRAS_DE_TRABALHO_E_ENTREGA.md`. **Prova final no CI:** `COMO_RODAR_A_PROVA_FINAL_NO_CI.md`. **O que está coberto e o que falta:** `MATRIZ_DE_ATAQUES_E_BARREIRAS.md`. A entrega só é declarada pelo `entregar.py`, depois de o pacote final ser extraído e verificado.
>
> **Antes de qualquer tarefa: a Tarefa T0.** Nenhum trabalho novo começa antes de a proteção da branch, a administração dupla e o responsável de segurança estarem ativos (`GUIA_DE_LIBERACAO.md` §2). Todo o código que já existe foi escrito sem essa proteção: ele entra no repositório protegido por um único pedido de importação, revisado integralmente por duas pessoas, com o termo assinado (`REVISAO_INTEGRAL_T0.md`). A T1 só começa depois.

Este é o ponto de partida do time. Em uma página: o que é o projeto, o que já está provado em código, o que vocês constroem, em que ordem, com quais regras, e como saber que uma tarefa está pronta. Tudo o que está escrito aqui foi conferido por máquina no pacote; o que não foi, está marcado.

## 1. O que é

Dois aplicativos, uma conta (um par de chaves no chip do aparelho):

- **NimbusChat** — comunicação: conversas, grupos, chamadas, envio de arquivos.
- **NimbusOne** — cofre pessoal, celular virtual, Sala Blindada, assinaturas e aprovações, console corporativo.

Princípio inviolável: **acesso zero**. O servidor guarda só blocos cifrados e nunca tem as chaves. O conteúdo só existe aberto no aparelho de quem tem direito, enquanto aparece na tela.

Superfícies: **servidor, celular (iOS e Android) e computador (macOS e Windows)**. O computador é aparelho vinculado ao celular e nunca passa do nível amarelo (manual, A6X).

## 2. O que já existe e está provado

| Peça | Situação | Como conferir |
|---|---|---|
| Núcleo de segurança em Rust (`nimbus-nucleo`, 58 módulos — era 56; +`cobranca` [cobrança RFC 9474] e +`codec_palavras` [BIP-39, regra 47 da ponte], módulos de rodadas posteriores a esta nunca antes somados aqui; reconferido na Rodada 26, 12/10/2026, direto contra `grep "^pub mod " src/lib.rs`, 58 de 58 batendo nos dois sentidos) | Escrito e testado; **ainda não auditado** | `./verificar.sh` na raiz de cada kit |
| 725 testes rápidos (era 678, nesta linha; medido na Rodada 26, 12/10/2026, com `cargo +stable test --locked`, TODOS os binários, 0 falharam, 6 ignorados — reconfirmado uma segunda vez dentro de uma cópia extraída isolada de um dos kits, mesmo resultado) | Passam | `cargo test --locked` |
| 353 defeitos plantados (mutação) (valor já correto nesta linha desde a Rodada 20; confirmado de novo na Rodada 26, 12/10/2026, cruzado contra `CATRACA.json` e `NUMEROS.json`, sem campanha de mutação nova rodada nesta revisão) | Todos pegos (353 de 353, campanha completa, 11/10/2026) | `python3 verificacao/mutantes.py` (≈ 30 min; aceita faixas: `152 163`) |
| 428 funções públicas do núcleo (era 407, nesta linha; regerado na Rodada 26, 12/10/2026, com `gerar_api.py` — as +21 vêm de `cobranca` e `codec_palavras`, os mesmos dois módulos novos da linha acima; **histórico da própria linha: já chegou a citar "311", "348", "394" e "407" — nenhum dos quatro valores antigos deve ser citado de novo**) | `API_DO_NUCLEO.md` regerado | `python3 gerar_api.py nimbus-nucleo_src/nimbus-nucleo API_DO_NUCLEO.md` |
| Fuzzing, tempo constante, varredura (47 regras — reconferido na Rodada 26, 12/10/2026, contra `nimbus-regras.yml`; valor já estava correto), dependências (84 versões de 71 bibliotecas — **ainda NÃO reconferido nesta rodada nem nas anteriores desde 10/10/2026**, deixado como estava), defeitos plantados contra a catraca, o auditor e o próprio verificador | Aprovados (na última vez em que foram medidos) | `./verificar.sh` (completo, falha fechada) |
| Protótipo navegável (66 telas — era 54; os 3 addenda da sessão foram fundidos de verdade no protótipo mestre em 10/10/2026, ver `PENDENCIAS_E_DECISOES.md` seção 32; **+4 telas novas desenhadas em 11/10/2026, décima nona rodada, ainda como arquivos HTML autônomos, não fundidas aqui — 66 continua correto até a fusão, ver `PENDENCIAS_E_DECISOES.md`, Rodada 19**) e guias visuais (regerados em 10/10/2026, cobrindo as 66 telas; +`Guia_Visual_Rodada19_Recuperacao_Liberacao_Retirada_Disfarce.pdf`, 11/10/2026, 17 páginas, cobrindo as 4 telas novas em addendum separado — mesmo padrão dos addenda 15/16, ver `LEIA-ME_Indice.md`) | Prontos (as 66 telas fundidas); 4 adicionais em revisão, com guia em PDF próprio | `docs/PROTOTIPO.html` e `docs/Guia_Visual_*.pdf` |
| Manual Técnico v37 (598 requisitos, 119 provas) | Pronto | `docs/referencia/MANUAL_TECNICO_COMPLETO.docx` |

O que **não** existe ainda: os apps (iOS, Android, computador), a ponte nativa, o servidor em produção, a infraestrutura, o pentest humano, a auditoria criptográfica e o teste de usabilidade. É isso que vocês constroem.

## 3. Os arquivos que importam, nesta ordem

1. **`COMECE_AQUI_TECNICOS.md`** — este arquivo.
2. **`PENDENCIAS_E_DECISOES.md`** — o que está aberto, o que depende de terceiros e os limites honestos. Leia antes de prometer prazo.
3. **`CONTRATO_DA_PONTE.md`** — a Tarefa T1: o que a ponte Swift/Kotlin precisa expor e as 49 regras que ela não pode quebrar (era 45 nesta linha; +4 da Rodada 21 — regras 46 a 49; antes disso era citado "30"/"41"/"43", já desatualizados; contagem direta das 49 linhas numeradas da tabela, reconferida 1–49 sem buraco na Rodada 26, 12/10/2026).
4. **`API_DO_NUCLEO.md`** — inventário gerado por máquina de tudo o que o núcleo expõe (428 funções públicas, regerado na Rodada 26, 12/10/2026 — era 407 nesta linha; ver nota na tabela da seção 2 sobre os valores antigos errados).
5. **`MATRIZ_DE_RASTREABILIDADE.csv`** (planilha) e **`.md`** — os 598 requisitos, com o módulo responsável, as funções, as provas e a evidência encontrada no código. Mostra exatamente o que já está no núcleo e o que é trabalho de app, servidor ou infraestrutura.
6. Dentro de cada kit: `CLAUDE.md` (regras), `docs/PLANO_DE_TAREFAS.md` (ordem), `docs/TELAS.md` (cada tela e a função que ela chama), `docs/MANUAL_*.md`.
7. **`NimbusOne_Engenharia_Social_e_Seguranca_Operacional.md`** — os 27 golpes contra pessoas e processos que o produto e a operação precisam impedir. Leia antes de desenhar qualquer tela que peça uma decisão.
8. **`NimbusOne_Ataques_com_IA_e_Defesa_em_Profundidade.md`** — o que um atacante com IA tentaria, o que ele levaria em cada camada se passasse e o que ainda falta fortalecer.
9. **`NimbusOne_Checklist_de_Seguranca_do_Backend.md`** — rotas negadas por padrão, limites de tentativa, banco com RLS testado num PostgreSQL real, OWASP API Top 10 item por item e o que sai se uma porta abrir. Os arquivos do banco estão em `servidor/banco/` de cada kit.
10. **`REGISTRO_E_POLITICA_PERMANENTE_DE_SEGURANCA.md`** — a política que vale para tudo (segredos, entradas, IA, senhas, limites, manutenção) e o registro de cada pedido de segurança do fundador, com o que foi feito e onde está.
11. **`ACESSO_SEM_AUTENTICACAO.md`**, **`PERGUNTAS_DE_TESTE_E_VERIFICACAO.md`** e **`MAPA_ASVS_5.0.csv`** — o que se acessa sem prova (gerado do código), 149 perguntas de teste com situação, e os 345 requisitos do OWASP ASVS 5.0.0 com situação proposta.
12. **`GUIA_DE_LIBERACAO.md`** — leia ANTES de tudo o que envolve GitHub: sem os 17 portões e o selo assinado por três chaves (CI, responsável de segurança e auditor independente), o servidor e os apps não funcionam.
13. **`NimbusOne_Metadados_O_Que_Sobra.md`** — cada metadado, quem vê, o que seria preciso para ligá-lo a alguém, e os próximos passos com custo.
14. **`CHECKLIST_DE_ENTRADA_EM_PRODUCAO.md`** — nada vai para produção com uma caixa vazia; os itens ⚙ a máquina confere, os ✍ precisam de registro assinado.
15. **`01_PRD_Nimbus.md` a `06_Plano_de_Implementacao_Nimbus.md`** — produto, requisitos técnicos, fluxo do app, UI/UX, esquema do back-end e plano de implementação; e **`NimbusOne_Revisao_de_Fechamento.md`** — ferramentas obrigatórias e limites.
16. Para a segurança: `NimbusOne_Plano_de_Pentest.md` (PT-01 a PT-48 e o critério de lançamento), `NimbusOne_CI_de_Seguranca.md`, `NimbusOne_Fluxos_de_Dados.md`, `NimbusOne_Modelo_Ameaca_STRIDE_LINDDUN.md`.
17. **`juridico/`** (dentro de cada kit) — `Politica_de_Privacidade.md`, `Termos_de_Uso.md` e, nova nesta rodada (27, 12/10/2026), **`NimbusOne_DPIA_LGPD.md`** (RIPD/LGPD; **esboço para revisão jurídica, ainda não é peça final**). Mesmo em esboço, já nomeia duas lacunas de que quem constrói precisa saber, não só o jurídico: nenhum **mecanismo de verificação de idade** foi encontrado (hoje é só política declarada "18+", sem checagem técnica — ver ECA Digital, Lei nº 15.211/2025) e não existe **procedimento de comunicação formal de incidente** (quem decide notificar, em que prazo, com qual texto — art. 48 da LGPD). As decisões que faltam (quem é o controlador, quem é o encarregado, qual hipótese do Art. 33 cobre a transferência internacional) são do fundador e do advogado, não de código — lista completa em `PENDENCIAS_E_DECISOES.md` §4 e no checklist de 9 itens da seção 11 do próprio DPIA.

## 4. Primeiro dia

```bash
unzip NimbusChat_Pacote.zip && unzip NimbusChat_Kit_ClaudeCode.zip
cd nimbuschat
./verificar.sh            # verificação completa, falha fechada: tem de terminar com "Tudo certo: N etapas obrigatórias rodaram e passaram."
```

Pré-requisitos (a falta de qualquer um reprova): Rust **1.91.1** (fixado em `rust-toolchain.toml`), Python 3, git, `semgrep==1.179.0` e a base de avisos RustSec do dia: `git clone --depth 1 https://github.com/RustSec/advisory-db /tmp/advisory-db` (com mais de 7 dias, a auditoria reprova).

Depois, abra a pasta no Claude Code (ele lê `CLAUDE.md` sozinho) e cole `prompts/00-comece-aqui.md`. A partir daí, uma tarefa por vez com `prompts/01-proxima-tarefa.md`.

## 5. Ordem de construção

| Etapa | O que | Pronto quando |
|---|---|---|
| T0 | Proteção da branch, administração dupla e responsável de segurança ativos; importação do código atual revisada integralmente por duas pessoas — ver `REVISAO_INTEGRAL_T0.md` | Portões `protecao_do_repositorio`, `administracao_dupla` e `revisao_de_duas_pessoas` aprovados e termo de revisão assinado |
| T1 | Ponte nativa (UniFFI) — ver `CONTRATO_DA_PONTE.md` | Os testes de aceite da ponte passam nas duas plataformas e o PT-16 não acha garantia desfeita |
| T2–T3 | Esqueleto dos apps sem SDK de terceiros; servidor que só guarda blocos cifrados | Print sai preto; varredura limpa; banco sem conteúdo legível |
| T4–T11 | Protocolo (libsignal e MLS), conta, conversas, chamadas, contatos, ajustes, envio | Provas citadas em cada tarefa do `PLANO_DE_TAREFAS.md` |
| T11B | App de computador (macOS e Windows), teto amarelo | C89 e C90 |
| T12–T13 | Infraestrutura; provas com auditor; lojas | Critério de "pronto para lançar" do Plano de Pentest (§7) |

## 6. Regras que não se negociam (resumo do `CLAUDE.md`)

1. Nenhuma criptografia fora do núcleo. Nunca.
2. O núcleo só muda com teste novo que falha antes e passa depois, mutação da faixa afetada e revisão de uma segunda pessoa.
3. Nenhuma mudança entra no código principal nem é publicada por uma pessoa só (dupla aprovação; a IA não conta como aprovador).
4. Nada de conteúdo, nome, chave ou endereço em log, erro ou telemetria. Nenhum SDK de terceiros.
5. Sem rede mesh e sem cache offline no NimbusChat (OFF1, OFF3). Todo prazo usa a hora confirmada pelo servidor (OFF2).
6. Nunca apague nem desative um teste. `MINIMO_DE_TESTES` só sobe.
7. Cada tela mostra a "verdade" indicada em `docs/TELAS.md`: nunca prometer além do que o código faz.

## 7. Definição de pronto (vale para toda tarefa)

- `./verificar.sh` termina com "Tudo certo.";
- varredura `semgrep --config core/nimbus-nucleo/nimbus-regras.yml .` sem ocorrências;
- `python3 verificacao/consistencia.py` aprovado (telas × protótipo, funções citadas × núcleo);
- as provas da tarefa demonstradas e registradas;
- revisão de uma segunda pessoa;
- se o núcleo mudou: testes novos, mutação da faixa, `gerar_api.py` e `gerar_matriz.py` rodados de novo.

## 7b. Proteções de repositório já configuradas

`.gitignore` (segredos e chaves nunca entram), `.gitleaks.toml` + `.pre-commit-config.yaml` (varredura de segredos antes de cada commit), `SECURITY.md`, `.github/CODEOWNERS` e `.github/workflows/` (verificação, segredos, banco e catraca, com ações fixadas pelo hash). Proteção da branch como código em `.github/regras/`: um administrador roda `sh .github/regras/aplicar_protecao.sh dono/repositorio` uma vez; depois, o fluxo `vigia-da-protecao` confere de hora em hora (e a cada envio) que nenhuma das três regras (principal, todas as branches e tags) foi afrouxada (precisa do segredo `PROTECAO_LEITURA`, token só de leitura).

## 8. Ferramentas de verificação do pacote

| Script | O que faz |
|---|---|
| `empacotar_e_verificar.sh` | Gera todos os ZIPs e confere tudo byte a byte |
| `verificar_pacote.py` | 19 conferências de integridade do pacote, byte a byte |
| `verificar_numeros.py` | Confere os números de todos os materiais contra `NUMEROS.json` |
| `gerar_guia_visual.py` | Refaz os guias visuais a partir do protótipo |
| `gerar_matriz.py` | Refaz a matriz de rastreabilidade a partir do manual e do código |
| `gerar_api.py` | Refaz o inventário da API pública do núcleo |
| `core/nimbus-nucleo/verificacao/catraca.py` | Reprova qualquer proteção a menos que antes |
