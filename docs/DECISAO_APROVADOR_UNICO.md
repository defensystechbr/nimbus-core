# Decisão — aprovador único no repositório

**26/10/2026 · Wellber Santos (dono do repositório e do produto) · substitui a regra de duas aprovações na T0**

## A decisão

O repositório `defensystechbr/nimbus-core` passa a operar com **aprovador único**: o dono revisa e aprova pelo
merge. A branch `principal` continua exigindo pedido de mudança, commit assinado, histórico linear, proibição de
força e de apagamento, verificações obrigatórias e nenhum passe livre. O que saiu foi a exigência de uma **segunda
pessoa** aprovar.

O piso de aprovações passou a viver em `core/nimbus-nucleo/CATRACA.json` (`aprovacoes_minimas: 0`), não espalhado
em número solto dentro do conferidor. Alterá-lo é uma edição visível num arquivo que a catraca vigia.

## Por que

1. **O autor não pode aprovar o próprio pedido.** Com um só revisor humano, o piso 2 não é "mais seguro": é um
   pedido que nunca fecha. Era isso que travava o desenvolvimento, não a falta de revisão.
2. **Ninguém estava revisando.** O código foi importado num único pedido por um único autor. Manter o piso 2 só
   produzia vermelho e treinava todo mundo a ignorar o vermelho, que é pior do que um piso menor declarado.
3. **A proteção que resta é a que funciona sem gente:** commit assinado (quem escreveu é quem assinou), catraca
   (nenhuma proteção a menos), varredura de segredos, histórico linear e sem força. Nenhuma delas depende de haver
   um segundo humano acordado.

## O que isso custa, dito sem enfeite

- **Some a revisão por outra pessoa antes do merge.** Um erro de projeto entra sem segunda leitura. O selo de
  liberação passa a ter uma aprovação humana, a do dono.
- **O piso 2 do manual continua sendo a meta** para quando houver time: `docs/MANUAL_NimbusChat.md` (GOV1, GOV6) e
  `core/nimbus-nucleo/GUIA_DO_TIME.md` descrevem o regime alvo, não o atual. A diferença está declarada aqui, não
  escondida.
- **A administração da organização continua incompleta** (sem segundo fator obrigatório, um administrador só, sem
  equipe de segurança com duas pessoas). Isso não é decisão de engenharia: depende de quem é dono da organização.
  O portão `administracao_dupla` **continua existindo e continua sendo exigido no selo de liberação** (`--estrito`);
  o que mudou é que ele não segura o merge de desenvolvimento, porque não há o que o desenvolvedor faça sobre ele.

## Como voltar atrás

1. `CATRACA.json`: `aprovacoes_minimas` de `0` para `2`.
2. `.github/regras/protecao-principal.json`: `required_approving_review_count` de `0` para `2`.
3. `sh .github/regras/aplicar_protecao.sh defensystechbr/nimbus-core`.

A catraca reprova se os dois números divergirem, então não dá para "voltar pela metade".

## O que continua proibido, com ou sem segunda pessoa

- Merge em `principal` sem pedido de mudança.
- Commit sem assinatura verificada.
- `push --force`, apagar a branch, reescrever histórico.
- Liberar versão sem o selo, e o selo sem os portões (inclusive o de administração da organização).
