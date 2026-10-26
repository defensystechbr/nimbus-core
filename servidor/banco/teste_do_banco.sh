#!/bin/sh
# Sobe um PostgreSQL temporário, aplica o esquema e roda os testes de RLS. Sai !=0 se qualquer teste falhar.
# Uso: ./teste_do_banco.sh   (precisa de PostgreSQL 16 instalado: initdb, pg_ctl, psql)
set -eu
DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="${PG_BIN:-$(ls -d /usr/lib/postgresql/*/bin 2>/dev/null | sort -V | tail -1)}"
TMP="$(mktemp -d)"
trap '"$BIN/pg_ctl" -D "$TMP/dados" -m immediate stop >/dev/null 2>&1 || true; rm -rf "$TMP"' EXIT
"$BIN/initdb" -D "$TMP/dados" -U postgres -A trust >/dev/null
"$BIN/pg_ctl" -D "$TMP/dados" -o "-k $TMP -p 55432 -c listen_addresses=''" -l "$TMP/log" -w start >/dev/null
"$BIN/psql" -h "$TMP" -p 55432 -U postgres -d postgres -v ON_ERROR_STOP=1 -q -f "$DIR/esquema.sql"
"$BIN/psql" -h "$TMP" -p 55432 -U postgres -d postgres -v ON_ERROR_STOP=1 -q -t -f "$DIR/teste_do_banco.sql"
"$BIN/psql" -h "$TMP" -p 55432 -U postgres -d postgres -v ON_ERROR_STOP=1 -q -t -f "$DIR/verificar_banco_na_partida.sql"
"$BIN/psql" -h "$TMP" -p 55432 -U nimbus_app -d postgres -v ON_ERROR_STOP=1 -q -t -f "$DIR/teste_papel_do_app.sql"
