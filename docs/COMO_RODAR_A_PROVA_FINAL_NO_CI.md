# Como rodar a prova final no CI — para os técnicos

**Versão 37 · 05/10/2026 · Confidencial**

## 1. A verdade sobre o que já foi medido

- O `./verificar.sh` completo **nunca rodou inteiro, num comando só**, na máquina que preparou este pacote: ela tinha 1 núcleo, pouco disco e reiniciava. Cada prova longa foi medida separadamente sobre o mesmo código, e cada registro traz dentro dele a identidade do código medido (`PROVAS_DA_RODADA.md`).
- O CI do projeto **ainda não existe**: ele nasce na Tarefa T0. Nenhuma prova foi rodada em CI até hoje.
- O escudo nativo (Kotlin e Swift) **nunca foi compilado**. Só passou por análise sintática, que não prova nomes de API, tipos nem comportamento.
- Por isso, este pacote é uma **pré-conferência**: tudo foi verificado, mas a entrega só é **declarada** pelo `entregar.py` rodando no CI, em máquina adequada.

## 2. Ordem obrigatória

1. **T0 primeiro** (`REVISAO_INTEGRAL_T0.md`): proteção da branch, administração dupla e responsável de segurança ativos; importação do código revisada inteira por duas pessoas, com termo assinado. Nada roda antes.
2. **CI ativo**: os fluxos dos kits (`nimbus-kit-ci.yml`, `sentinela-diaria.yml`, `vigia-da-protecao.yml`) e o fluxo de entrega (`ci/entrega.yml`, copiado para `.github/workflows/` na raiz do repositório da transferência).
3. **Primeira prova de liberação**: o fluxo `entrega`, disparado à mão, num executor com pelo menos **4 núcleos, 8 GB de memória e 20 GB livres**. O portão recusa máquina menor, execução fora do GitHub Actions e árvore git com mudança não registrada.
4. Guardar `ENTREGA_DECLARADA.txt` e os logs do CI junto do selo. Toda versão seguinte sai do mesmo jeito.

## 3. O que o portão roda (e por quê)

| Etapa | O que prova | Por que existe |
|---|---|---|
| Ataque a si mesmo | O portão recusa os defeitos plantados contra ele | Conferidor não atacado não vale como prova |
| `medir_provas.py` | Todas as provas rápidas desta execução, registros longos com identidade do código, faixas de mutação sem buraco | Número velho voltava a ser citado |
| gitleaks atacado | Pega segredos plantados; depois varre fontes e saídas | Ferramenta sem ataque prévio pode estar cega |
| Empacotamento | Conferidores atacados antes do uso; 19 conferências byte a byte | Cópias velhas sobreviviam dentro dos kits |
| Kits extraídos | Identidade do código igual à das provas; `./verificar.sh` **completo** dentro de cada kit | A falha do LIB1 só apareceu no pacote extraído |
| Partes extraídas de novo | Manifestos, pacote igual ao verificado, números, registros e identidade | Entrega diferente do que foi provado |

## 4. Medições sem armadilha

- **Cópia e pasta de compilação próprias**: `CARGO_TARGET_DIR` definido reprova. Pasta compartilhada já fez o Cargo aproveitar o binário de um defeito plantado de outra cópia.
- **Identidade do código no começo e no fim**: se o código mudar durante a verificação, nada vale.
- **Tempo constante e tempo de recusa sozinhos na máquina**: são sensíveis a disputa de processador.
- **Disco**: o portão exige espaço antes de começar e libera a compilação de cada kit depois de verificá-lo; já houve reprovação por disco cheio no meio.

## 5. O que ainda não está ligado

- `cargo clippy --all-targets --locked -- -D warnings`: **não roda** no CI nem no `verificar.sh`. Ligar na T1 e corrigir os avisos.
- Escudo nativo: compilar com o Android SDK e o Xcode na T2 e aprovar os testes VSE1–VSE8 e RSD1–RSD12 em aparelho real.
- 47 identificadores de requisito citados pelo núcleo ainda não definidos no Manual (`REQUISITOS_CITADOS_SEM_DEFINICAO.json`; a lista só pode diminuir).

## 6. Verificar antes de usar

1. Que o executor do CI tem os recursos exigidos (o portão confere e recusa se não tiver).
2. Que as versões fixadas (Rust 1.91.1, semgrep 1.179.0, tree-sitter-language-pack 1.21.0, gitleaks 8.30.1) continuam as aprovadas.

## 7. O que esta rodada assistida por IA (09/10/2026) conseguiu verificar neste sandbox isolado, e o que não — sem arredondar para cima

Esta seção existe porque "rodei os testes" não é uma frase precisa o bastante para você confiar — aqui está exatamente o que rodou, em qual ambiente, e o que não rodou e por quê. Nenhum destes limites é novo por causa do código desta rodada (a aba trancada/custódia mútua); são limites do AMBIENTE (um contêiner isolado, sem rede liberada para a maioria dos destinos, sem o repositório completo do projeto), os mesmos que já valiam antes desta sessão começar.

**Rodou, de ponta a ponta, e pode ser conferido pelo hash abaixo:**
- `cargo test --locked` completo (todos os binários, não só `--lib`): **571 de 571 passaram, 0 falhas, 6 ignorados de propósito**.
- `python3 verificacao/mutantes.py` — rodada COMPLETA, defeito por defeito (em faixas, por causa do limite de tempo por comando deste sandbox, nunca pulando nenhum): **321 de 321 defeitos plantados pegos, zero sobrevivente**.
- As três provas pesadas marcadas `--ignored` (normalmente só rodadas à parte): fuzzing de 50.000 entradas por alvo, tempo constante (PIN real/coação/errado) e tempo de recusa — **as três passaram** nesta rodada.
- Identidade do código (`verificacao/rodar_prova.py::identidade`) calculada ANTES e DEPOIS de toda a bateria acima, e de novo numa cópia extraída do `.zip` final: as três batem — `c61ea5cc8ec108ed71e59b7b7776acc5409bfe9e64b95aa8a5f944dfeb5d1986`.
- Toolchain usado: `stable` (1.97.0) instalado neste contêiner — **não** o `1.91.1` fixado em `rust-toolchain.toml`; a rede deste sandbox não libera o `rustup` baixando esse toolchain específico. Revalidar com 1.91.1 antes de qualquer promessa externa, exatamente como a seção 6 já pede.

**Tentou e não coube no limite de tempo de um comando deste sandbox (não é falha do código, é duração real):**
- Fuzzing profundo de 2.000.000 de entradas por alvo: tentado, ultrapassou o teto de ~10 minutos por comando deste ambiente (a própria documentação do `verificar.sh` já avisa que esse passo é bem mais longo numa máquina de poucos núcleos — este sandbox é exatamente esse caso). Nenhum dos arquivos tocados nesta rodada (`custodia_mutua.rs`) é exercitado por esses alvos de fuzzing (que testam leitores de formato: `inspecao`, `registro`, `envelope`, etc.) — ou seja, a mudança desta rodada não é candidata a motivo de regressão aqui, mas o passo precisa rodar inteiro, sem pular, antes de qualquer selo de liberação de verdade. Comando exato para o CI real: `FUZZ_RODADAS=2000000 cargo test --profile verificacao --test fuzzing --locked -- --ignored`.

**Não rodou neste sandbox — ferramenta ou rede ausente, não escolha:**
- `semgrep` — não está instalado neste contêiner (`command -v semgrep` não encontra nada), e a varredura das 47 regras próprias (`nimbus-regras.yml`) é portão obrigatório (seção 1 de `NimbusOne_CI_de_Seguranca.md`). Comando exato para o CI real: `semgrep --config nimbus-nucleo_src/nimbus-nucleo/nimbus-regras.yml . --metrics=off --error`.
- `verificacao/auditar_dependencias.py` (auditoria de dependências contra a base RustSec) — precisa de acesso de rede a uma base de avisos atualizada, que este sandbox não libera para esse destino. Comando exato para o CI real, dentro de `nimbus-nucleo_src/nimbus-nucleo`: `python3 verificacao/auditar_dependencias.py`.
- `./verificar.sh` (o script completo, por si só) — exige `semgrep`, `git` com acesso à base de avisos, e o toolchain 1.91.1 instalado via `rustup`; nenhum dos três está disponível aqui sem rede liberada para esses destinos. O que esta rodada fez em vez disso foi rodar manualmente, um por um, cada passo do script que NÃO depende de rede (testes, mutação completa, as três provas `--ignored`, identidade do código) — o que falta é só o que depende de rede/ferramenta ausente, listado acima.
- **`empacotar_e_verificar.sh` / `verificar_pacote.py` (o empacotamento completo multi-produto: núcleo avulso + kit NimbusChat + kit NimbusOne + comparação byte a byte entre eles)** — este script espera uma estrutura de diretórios (`handoff/`, `kits/nimbuschat`, `kits/nimbusone`, `kits_comum/nativo/`) e um Manual Técnico `.docx` externo em `/mnt/user-data/outputs/`, nenhum dos quais existe neste sandbox isolado (aqui só existe o núcleo avulso e os documentos técnicos deste pacote — `handoff_2026-10-08/`). Isto não é algo que faltou RODAR; é algo que falta **existir neste ambiente** para rodar. No repositório real do projeto (onde `handoff/` e `kits/` existem), rodar `./empacotar_e_verificar.sh` é o comando certo e deve ser a próxima verificação antes de tratar este pacote como o pacote comercial final.

**Checklist direto para o seu time técnico rodar no ambiente real, nesta ordem:**
1. `rustup toolchain install` (lê `rust-toolchain.toml`, instala o 1.91.1 fixado) e repetir a bateria de testes/mutação com ele, não com `stable` — confirmar que a identidade do código não muda.
2. `semgrep --config nimbus-regras.yml . --metrics=off --error` a partir de `nimbus-nucleo_src/nimbus-nucleo`.
3. `python3 verificacao/auditar_dependencias.py` com acesso real à base RustSec.
4. `FUZZ_RODADAS=2000000 cargo test --profile verificacao --test fuzzing --locked -- --ignored` até o fim, sem interromper.
5. `./verificar.sh` completo, um comando só, numa máquina com os recursos da seção 2.3 acima — isto sozinho já repete 1–4, então rodar 1–4 separado só serve para isolar qual etapa falhar, se alguma falhar.
6. Só então `./empacotar_e_verificar.sh` no repositório completo (com `handoff/` e `kits/`), para o empacotamento multi-produto e a comparação byte a byte entre o núcleo avulso e os dois kits.

Nenhuma dessas seis etapas pode ser substituída por esta sessão de IA dizendo "deveria passar" — são exatamente o que a seção 1 deste documento já chama de diferença entre "pré-conferência" e "declarada".
