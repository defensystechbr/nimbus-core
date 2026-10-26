#!/bin/sh
# Aplica a proteção da branch principal como código (ruleset do GitHub). Rodar UMA vez, por um administrador,
# com o GitHub CLI autenticado:  sh .github/regras/aplicar_protecao.sh dono/repositorio
# Depois disso, o fluxo "vigia-da-protecao" confere todo dia que ela continua ativa e reprova se alguém a afrouxar.
set -eu
REPO="${1:?informe dono/repositorio}"
DIR="$(cd "$(dirname "$0")" && pwd)"
# Aplica TODAS as regras da pasta: branch principal, todas as branches (commits assinados) e tags (sem apagar,
# mover ou reescrever). Cada arquivo tem o próprio nome; regra existente é atualizada, nova é criada.
for ARQ in "$DIR"/*.json; do
  NOME=$(python3 -c "import json,sys;print(json.load(open(sys.argv[1]))['name'])" "$ARQ")
  EXISTENTE=$(gh api "repos/$REPO/rulesets" --paginate --jq ".[] | select(.name==\"$NOME\") | .id" || true)
  if [ -n "$EXISTENTE" ]; then
    gh api --method PUT "repos/$REPO/rulesets/$EXISTENTE" --input "$ARQ" >/dev/null
  else
    gh api --method POST "repos/$REPO/rulesets" --input "$ARQ" >/dev/null
  fi
  echo "  regra aplicada: $NOME"
done
echo "Proteção aplicada em $REPO: 2 aprovações (incluindo dono do código e último envio aprovado por outra pessoa), commits assinados em todas as branches, histórico linear, sem força nem apagamento, 6 verificações obrigatórias, tags imutáveis, ninguém com passe livre."
