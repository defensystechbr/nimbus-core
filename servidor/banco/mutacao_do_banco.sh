#!/bin/sh
# Planta defeitos no esquema, um por vez, e confere que os testes do banco pegam cada um.
# Defeitos que só aparecem quando duas camadas falham juntas são plantados juntos: com uma camada só,
# a outra segura (é a defesa em profundidade funcionando, não lacuna de teste).
# Uso: ./mutacao_do_banco.sh   (mesmos requisitos do teste_do_banco.sh)
set -u
DIR="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
pegos=0; total=0
planta() { # $1 descrição; depois pares (texto original, texto defeituoso)
  total=$((total+1)); cp "$DIR/teste_do_banco.sql" "$DIR/teste_do_banco.sh" "$DIR/verificar_banco_na_partida.sql" "$DIR/teste_papel_do_app.sql" "$TMP/"
  desc="$1"; shift
  python3 - "$DIR/esquema.sql" "$TMP/esquema.sql" "$@" <<'PY'
import sys; s=open(sys.argv[1],encoding='utf-8').read(); pares=sys.argv[3:]
for o,m in zip(pares[0::2], pares[1::2]):
    assert s.count(o)==1, o; s=s.replace(o,m)
open(sys.argv[2],'w',encoding='utf-8').write(s)
PY
  set -- "$desc"
  if sh "$TMP/teste_do_banco.sh" >/dev/null 2>&1; then echo "[VIVO] $desc"; else echo "[pego] $desc"; pegos=$((pegos+1)); fi
}
planta "RLS desligada em blocos" "ALTER TABLE nimbus.blocos ENABLE ROW LEVEL SECURITY;" "-- sem RLS"
planta "leitura de qualquer caixa" "USING (endereco = nimbus.caixa_autorizada() AND expira_em > now());" "USING (true);"
planta "vencido continua legível" "USING (endereco = nimbus.caixa_autorizada() AND expira_em > now());" "USING (endereco = nimbus.caixa_autorizada());"
planta "apagar de qualquer caixa" "FOR DELETE TO nimbus_app
  USING (endereco = nimbus.caixa_autorizada());" "FOR DELETE TO nimbus_app
  USING (true);"
planta "depósito sem ficha" "WITH CHECK (nimbus.deposito_autorizado() AND expira_em <= now() + interval '48 hours');" "WITH CHECK (expira_em <= now() + interval '48 hours');"
planta "caixa criada sem ficha" "caixa_criar_com_ficha ON nimbus.caixas FOR INSERT TO nimbus_app WITH CHECK (nimbus.deposito_autorizado());" "caixa_criar_com_ficha ON nimbus.caixas FOR INSERT TO nimbus_app WITH CHECK (true);"
planta "prazo de 49 horas aceito" "expira_em <= criado_em + interval '48 hours')" "expira_em <= criado_em + interval '49 days')"
planta "ajuste torto aceito" "THEN decode(current_setting('nimbus.caixa', true), 'hex') END" "THEN decode(current_setting('nimbus.caixa', true), 'hex') ELSE decode(repeat('aa',32),'hex') END"
planta "app pode alterar bloco" "GRANT SELECT, INSERT, DELETE ON nimbus.blocos TO nimbus_app;" "GRANT SELECT, INSERT, UPDATE, DELETE ON nimbus.blocos TO nimbus_app; CREATE POLICY x ON nimbus.blocos FOR UPDATE TO nimbus_app USING (true);"
planta "app lê registros de acesso" "GRANT INSERT ON nimbus.registros_de_acesso TO nimbus_app;" "GRANT SELECT, INSERT ON nimbus.registros_de_acesso TO nimbus_app;"
planta "faxina apaga o que ainda vale (função e políticas erradas)" "DELETE FROM nimbus.blocos WHERE expira_em <= now();" "DELETE FROM nimbus.blocos WHERE true;" "CREATE POLICY faxina_blocos ON nimbus.blocos FOR DELETE TO nimbus_dono USING (expira_em <= now());" "CREATE POLICY faxina_blocos ON nimbus.blocos FOR DELETE TO nimbus_dono USING (true);" "CREATE POLICY faxina_ver_blocos_vencidos ON nimbus.blocos FOR SELECT TO nimbus_dono USING (expira_em <= now());" "CREATE POLICY faxina_ver_blocos_vencidos ON nimbus.blocos FOR SELECT TO nimbus_dono USING (true);"
planta "app executa a faxina" "REVOKE ALL ON FUNCTION nimbus.apagar_vencidos() FROM PUBLIC;" "GRANT EXECUTE ON FUNCTION nimbus.apagar_vencidos() TO nimbus_app;"
planta "tabela nova sem RLS forçada" "ALTER TABLE nimbus.caixas FORCE ROW LEVEL SECURITY;" "-- sem FORCE"
planta "app dono de tabela" "CREATE TABLE nimbus.registros_de_acesso (" "CREATE TABLE nimbus.x_sem_dono (a int); ALTER TABLE nimbus.x_sem_dono ENABLE ROW LEVEL SECURITY; ALTER TABLE nimbus.x_sem_dono FORCE ROW LEVEL SECURITY; RESET ROLE; ALTER TABLE nimbus.x_sem_dono OWNER TO nimbus_app; SET ROLE nimbus_dono;
CREATE TABLE nimbus.registros_de_acesso ("
planta "app pode virar dono" "GRANT USAGE ON SCHEMA nimbus TO nimbus_app, nimbus_faxina;" "GRANT USAGE ON SCHEMA nimbus TO nimbus_app, nimbus_faxina; GRANT nimbus_dono TO nimbus_app;"
echo "Banco: $pegos de $total defeitos plantados pegos."
[ "$pegos" -eq "$total" ]
