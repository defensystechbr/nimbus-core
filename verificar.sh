#!/bin/sh
# Verifica, de uma vez, tudo o que já foi feito. FALHA FECHADA: toda etapa da lista é obrigatória;
# ferramenta ausente, etapa que não rodou ou etapa pulada REPROVA. "Tudo certo." só aparece se TODAS rodaram e passaram.
#
# Uso: ./verificar.sh             verificação completa (testes, catraca e seus defeitos plantados, exemplo,
#                                 fuzzing, varredura, auditoria de dependências e seus defeitos plantados,
#                                 consistência no kit, mutação completa, tempo constante, tempo de recusa e
#                                 fuzzing profundo de 2 milhões). Cerca de 25 min numa máquina de 8 núcleos;
#                                 bem mais numa de 1 núcleo.
#      ./verificar.sh --completo  o mesmo (mantido por compatibilidade com o CI e com o sentinela diário).
#      ./verificar.sh --amostra   tudo do completo, mas a mutação roda uma amostra determinística de 40 dos
#                                 356 defeitos (semente = GITHUB_SHA ou o hash do commit) e o fuzzing profundo
#                                 roda 200 mil entradas em vez de 2 milhões. É a verificação do fluxo de PR:
#                                 não espera 1h a 2h, e o catálogo inteiro e as 2 milhões continuam no diário.
#      ./verificar.sh --rapido    só as etapas rápidas, para o dia a dia. Termina com código 3 e a palavra
#                                 PARCIAL: nunca vale como verificação, nem no CI, nem no selo.
# Variáveis opcionais: CARGO=cargo-1.91  ADVISORY_DB=/caminho/advisory-db  MUTANTES_AMOSTRA=40  MUTANTES_SEMENTE=<sha>
set -eu
CARGO="${CARGO:-cargo}"
AMOSTRA="${MUTANTES_AMOSTRA:-40}"
# Fuzzing profundo: 2 milhões na campanha integral (sentinela diário), 200 mil no fluxo de PR. A catraca
# vigia o valor integral: baixá-lo é edição visível em arquivo protegido, como o piso dos testes.
FUZZ_INTEGRAL=2000000
FUZZ_FLUXO=200000
MODO=completo
case "${1:-}" in
  ""|--completo) MODO=completo ;;
  --amostra) MODO=amostra ;;
  --rapido) MODO=rapido ;;
  *) echo "Opção desconhecida: $1 (use --rapido, --amostra ou nada)"; exit 1 ;;
esac
RAIZ="$(cd "$(dirname "$0")" && pwd)"
if [ -d "$RAIZ/core/nimbus-nucleo" ]; then NUCLEO="$RAIZ/core/nimbus-nucleo"; NO_KIT=1; else NUCLEO="$RAIZ"; NO_KIT=0; fi
TMP="$(mktemp -d "${TMPDIR:-/tmp}/nimbus-verificar.XXXXXX")"
ETAPAS=0
ok() { ETAPAS=$((ETAPAS + 1)); printf '  ✔ %s\n' "$1"; }
falha() { printf '  ✘ %s\n' "$1"; printf 'REPROVADO: a verificação parou na primeira etapa que falhou ou não rodou.\n'; exit 1; }
precisa() { command -v "$1" >/dev/null 2>&1 || falha "$2"; }
echo "Verificação ($MODO) — falha fechada: toda etapa ausente reprova"
# Pasta de compilação compartilhada entre cópias do núcleo faz o Cargo reaproveitar o binário de OUTRA cópia
# (inclusive de um defeito plantado) e aprovar ou reprovar o código errado. Achado em 04/10/2026. Reprova.
[ -z "${CARGO_TARGET_DIR:-}" ] || falha "CARGO_TARGET_DIR definido ($CARGO_TARGET_DIR): o resultado poderia vir de outra cópia do código. Rode sem ele."
precisa "$CARGO" "Rust não encontrado — instale pelo rustup (https://rustup.rs); a versão 1.91.1 vem do rust-toolchain.toml"
precisa python3 "python3 não encontrado"
precisa semgrep "semgrep não encontrado: a varredura é obrigatória (pip install semgrep)"
precisa git "git não encontrado: a auditoria de dependências precisa saber a data da base de avisos"
if command -v rustup >/dev/null 2>&1; then (cd "$NUCLEO" && rustup toolchain install >/dev/null 2>&1) || true; fi
# O rust-toolchain.toml mora dentro do núcleo: a versão só é a fixada quando o cwd é o núcleo.
# Conferir antes do cd lia o Rust default do runner (1.99) e reprovava sem motivo.
cd "$NUCLEO"
VERSAO="$("$CARGO" --version)"; echo "  $VERSAO"
case "$VERSAO" in *" 1.91."*) ;; *) falha "versão do Rust diferente da fixada (1.91.x): $VERSAO" ;; esac
# Uma mutação interrompida pode deixar o código com um defeito plantado. Restaura antes de qualquer teste.
for ORIGINAL in src/*.original; do
  [ -f "$ORIGINAL" ] || continue
  mv "$ORIGINAL" "${ORIGINAL%.original}" && echo "  • restaurado ${ORIGINAL%.original}, deixado por uma mutação interrompida"
done
# Identidade do código verificado (a mesma do medidor de provas e do portão de entrega). Conferida de novo no fim:
# se o código mudar durante a verificação, nada do que foi medido vale.
identidade() { python3 -c "import sys; sys.path.insert(0, 'verificacao'); import rodar_prova as r; print(r.identidade('.'))"; }
IDENTIDADE=$(identidade) || falha "não consegui calcular a identidade do código"
echo "  código verificado: $IDENTIDADE"
confere_identidade() { [ "$(identidade)" = "$IDENTIDADE" ] || falha "o código mudou durante a verificação: nada do que foi medido vale"; }
"$CARGO" test --locked -q >"$TMP/testes.log" 2>&1 || { tail -40 "$TMP/testes.log"; falha "testes do núcleo"; }
TOTAL=$(grep -o '[0-9]* passed' "$TMP/testes.log" | awk '{s+=$1} END {print s+0}')
# Piso de testes: se algum teste for apagado ou deixar de ser encontrado, a verificação reprova.
# Ao acrescentar testes, suba este número; nunca o abaixe.
MINIMO_DE_TESTES=725
[ "$TOTAL" -ge "$MINIMO_DE_TESTES" ] || falha "só $TOTAL testes rodaram (mínimo $MINIMO_DE_TESTES): algum teste foi apagado ou deixou de ser encontrado"
ok "testes do núcleo: $TOTAL aprovados"
BANCO_ARG=""; [ -d "$RAIZ/servidor/banco" ] && BANCO_ARG="$RAIZ/servidor/banco"
python3 verificacao/catraca.py $BANCO_ARG >"$TMP/catraca.log" 2>&1 && ok "catraca: nenhuma proteção a menos" || { cat "$TMP/catraca.log"; falha "catraca"; }
# Achado do dono em 09/10/2026: o contrato cita nomes de função como prova de regra implementada, mas
# nada conferia se esses nomes continuavam existindo no código (a regra 35 desta mesma rodada descrevia
# como pronta uma função que ainda não tinha sido escrita). Conferido aqui, em toda rodada, não só na
# embalagem do kit — reprova se algum nome `modulo::item` ou `modulo::Tipo::membro` citado no contrato
# não existir mais; identificador solto sem módulo e referência a outro documento ficam visíveis mas não
# reprovam por si só (ver docstring do script para o porquê).
python3 verificacao/conferir_funcoes_citadas.py >"$TMP/citacoes.log" 2>&1 && ok "$(tail -1 "$TMP/citacoes.log")" || { cat "$TMP/citacoes.log"; falha "funções citadas no CONTRATO_DA_PONTE.md não encontradas no código"; }
python3 verificacao/testar_catraca.py >"$TMP/testar-catraca.log" 2>&1 && ok "$(tail -1 "$TMP/testar-catraca.log")" || { cat "$TMP/testar-catraca.log"; falha "a catraca deixou passar um afrouxamento plantado"; }
python3 verificacao/mutantes.py --conferir >"$TMP/mut-conferir.log" 2>&1 && ok "$(tail -1 "$TMP/mut-conferir.log")" || { cat "$TMP/mut-conferir.log"; falha "defeitos plantados velhos: o código mudou e a mutação deixaria de testar"; }
"$CARGO" run --example fluxo_completo --locked -q >"$TMP/exemplo.log" 2>&1 || { tail -20 "$TMP/exemplo.log"; falha "exemplo de ponta a ponta"; }
PASSOS=$(grep -c '^\[' "$TMP/exemplo.log" || true)
[ "$PASSOS" -ge 33 ] || falha "exemplo de ponta a ponta com só $PASSOS passos (mínimo 33)"
ok "exemplo de ponta a ponta: $PASSOS passos"
FUZZ_RODADAS=50000 "$CARGO" test --profile verificacao --test fuzzing --locked -q -- --ignored >"$TMP/fuzz.log" 2>&1 || { tail -25 "$TMP/fuzz.log"; falha "fuzzing dos leitores (algum leitor entrou em pânico)"; }
grep -q "1 passed" "$TMP/fuzz.log" || falha "o fuzzing não rodou (nenhum teste executado)"
ok "fuzzing: 50000 entradas maliciosas por alvo, nenhum pânico"
semgrep --config "$NUCLEO/nimbus-regras.yml" "$RAIZ" --metrics=off --error -q >"$TMP/semgrep.log" 2>&1 && ok "varredura de segurança ($(grep -c '^  - id:' "$NUCLEO/nimbus-regras.yml") regras): nenhuma ocorrência" || { tail -30 "$TMP/semgrep.log"; falha "varredura de segurança"; }
python3 verificacao/testar_auditoria.py >"$TMP/testar-auditoria.log" 2>&1 && ok "$(tail -1 "$TMP/testar-auditoria.log")" || { cat "$TMP/testar-auditoria.log"; falha "o auditor de dependências deixou passar um defeito plantado"; }
# O teste do verificar.sh roda cópias deste mesmo script; dentro dessas cópias ele não se chama de novo
# (senão seria recursão sem fim). A variável só é ligada pelo próprio testar_verificar.py.
if [ "${NIMBUS_DENTRO_DO_TESTE_DO_VERIFICADOR:-}" = 1 ]; then
  ok "teste do próprio verificar.sh: esta é uma das cópias sob teste (roda no nível de cima)"
else
  timeout 1800 python3 verificacao/testar_verificar.py >"$TMP/testar-verificar.log" 2>&1 && ok "$(tail -1 "$TMP/testar-verificar.log")" || { cat "$TMP/testar-verificar.log"; falha "o próprio verificar.sh deixou passar uma etapa ausente (ou o teste não terminou em 30 min)"; }
fi
if AUDITORIA=$(python3 verificacao/auditar_dependencias.py 2>&1); then
  ok "$AUDITORIA"
else
  printf '%s\n' "$AUDITORIA"; falha "auditoria de dependências (vulnerável ou não rodou)"
fi
if [ "$NO_KIT" = 1 ]; then
  [ -f "$RAIZ/verificacao/consistencia.py" ] || falha "consistência dos documentos: verificacao/consistencia.py ausente no kit"
  (cd "$RAIZ" && python3 verificacao/verificar_kit.py >"$TMP/kit.log" 2>&1) && ok "kit íntegro e documentos consistentes com o protótipo e o núcleo" || { cat "$TMP/kit.log"; falha "integridade e consistência do kit"; }
fi
confere_identidade
if [ "$MODO" = rapido ]; then
  echo "PARCIAL: $ETAPAS etapas rápidas aprovadas. NÃO rodaram: mutação completa, tempo constante, tempo de recusa e fuzzing profundo."
  echo "Isto não é uma verificação. Para verificar: ./verificar.sh"
  exit 3
fi
LIVRE_KB=$(df -Pk "$NUCLEO" | awk 'NR==2 {print $4}')
if [ "${LIVRE_KB:-0}" -lt 3000000 ]; then
  echo "  • atenção: menos de 3 GB livres; a mutação pode encher o disco. Se algo falhar sem motivo: rm -rf target e rode de novo."
fi
MINIMO_MUT=$(python3 -c "import json;print(json.load(open('CATRACA.json'))['defeitos_plantados_minimos'])")
# O catálogo não pode encolher por causa da amostra: quem confere o total é este número, não a quantidade rodada.
CATALOGO=$(python3 -c "import sys; sys.path.insert(0, 'verificacao'); import mutantes; print(len(mutantes.MUTANTES))")
[ "$CATALOGO" -ge "$MINIMO_MUT" ] || falha "catálogo de defeitos plantados encolheu: $CATALOGO < $MINIMO_MUT"
if [ "$MODO" = amostra ]; then
  SEMENTE="${MUTANTES_SEMENTE:-${GITHUB_SHA:-$(git rev-parse HEAD 2>/dev/null || echo local)}}"
  MUTANTES_AMOSTRA="$AMOSTRA" MUTANTES_SEMENTE="$SEMENTE" CARGO="$CARGO" python3 verificacao/mutantes.py >"$TMP/mutacao.log" 2>&1 || { grep -E 'VIVO|ERRO|Resultado|precisam' "$TMP/mutacao.log"; falha "mutação por amostra: algum defeito plantado não foi pego ou não compilou"; }
  PEGOS=$(sed -n 's/^Resultado: \([0-9]*\) de \([0-9]*\) .*/\1 \2/p' "$TMP/mutacao.log")
  set -- $PEGOS
  [ "${1:-0}" = "${2:-0}" ] && [ "${2:-0}" -eq "$AMOSTRA" ] || falha "amostra de mutação incompleta: ${1:-0} de ${2:-0} (esperado $AMOSTRA)"
  ok "mutação por amostra: $1 de $2 de $CATALOGO defeitos plantados pegos (semente ${SEMENTE%"${SEMENTE#????????????}"})"
else
  CARGO="$CARGO" python3 verificacao/mutantes.py >"$TMP/mutacao.log" 2>&1 || { grep -E 'VIVO|ERRO|Resultado|precisam' "$TMP/mutacao.log"; falha "mutação: algum defeito plantado não foi pego ou não compilou"; }
  PEGOS=$(sed -n 's/^Resultado: \([0-9]*\) de \([0-9]*\) .*/\1 \2/p' "$TMP/mutacao.log")
  set -- $PEGOS
  [ "${1:-0}" = "${2:-x}" ] && [ "${2:-0}" -ge "$MINIMO_MUT" ] || falha "mutação incompleta: ${1:-0} de ${2:-0} (mínimo $MINIMO_MUT)"
  ok "mutação completa: $1 de $2 defeitos plantados pegos"
fi
"$CARGO" test --profile verificacao --test timing --locked -q -- --ignored >"$TMP/timing.log" 2>&1 && grep -q "1 passed" "$TMP/timing.log" && ok "tempo constante: PIN real, de coação e errado levam o mesmo tempo" || { tail -15 "$TMP/timing.log"; falha "tempo constante"; }
"$CARGO" test --release --test tempo_de_recusa --locked -q -- --ignored >"$TMP/recusa.log" 2>&1 && grep -q "1 passed" "$TMP/recusa.log" && ok "tempo de recusa: não revela se uma caixa ou aparelho existe" || { tail -15 "$TMP/recusa.log"; falha "tempo de recusa"; }
if [ "$MODO" = amostra ]; then FUZZ_PROFUNDO="$FUZZ_FLUXO"; else FUZZ_PROFUNDO="$FUZZ_INTEGRAL"; fi
FUZZ_RODADAS="$FUZZ_PROFUNDO" "$CARGO" test --profile verificacao --test fuzzing --locked -q -- --ignored >"$TMP/fuzz2.log" 2>&1 && grep -q "1 passed" "$TMP/fuzz2.log" && ok "fuzzing profundo: $FUZZ_PROFUNDO entradas por alvo, nenhum pânico" || { tail -20 "$TMP/fuzz2.log"; falha "fuzzing profundo"; }
echo "  • fora deste comando (exigem aparelho físico): cd core/nimbus-nucleo && cargo test --test aparelho_real -- --ignored"
confere_identidade
if [ "$MODO" = amostra ]; then
  echo "Tudo certo (AMOSTRA): $ETAPAS etapas rodaram e passaram, com $AMOSTRA de $CATALOGO defeitos plantados (semente $SEMENTE) e $FUZZ_FLUXO entradas de fuzzing por alvo."
  echo "A campanha integral dos $CATALOGO e as $FUZZ_INTEGRAL entradas rodam no sentinela diário; a semente muda a cada commit e cobre o resto ao longo do tempo."
else
  echo "Tudo certo: $ETAPAS etapas obrigatórias rodaram e passaram (código $IDENTIDADE)."
fi
