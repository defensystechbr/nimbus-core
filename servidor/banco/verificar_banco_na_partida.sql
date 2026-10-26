-- Sentinela do banco: o servidor roda isto NA PARTIDA e a cada hora, com o papel nimbus_app.
-- Se qualquer conferência falhar, levanta erro com o lugar e o conserto, e o servidor NÃO atende (falha fechada).
\set ON_ERROR_STOP on
DO $$
DECLARE r record; achados text := '';
BEGIN
  -- 1. Toda tabela do esquema com RLS ligada E forçada.
  FOR r IN SELECT c.relname, c.relrowsecurity, c.relforcerowsecurity FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
           WHERE n.nspname = 'nimbus' AND c.relkind = 'r' LOOP
    IF NOT (r.relrowsecurity AND r.relforcerowsecurity) THEN
      achados := achados || format(E'[BANCO-RLS] tabela nimbus.%s sem RLS ligada e forçada — rodar: ALTER TABLE nimbus.%s ENABLE ROW LEVEL SECURITY; ALTER TABLE nimbus.%s FORCE ROW LEVEL SECURITY;\n', r.relname, r.relname, r.relname);
    END IF;
  END LOOP;
  -- 2. O papel do app nunca é superusuário, nunca ignora RLS e não é dono de tabela.
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'nimbus_app' AND (rolsuper OR rolbypassrls OR rolcreaterole OR rolcreatedb)) THEN
    achados := achados || E'[BANCO-PAPEL] nimbus_app com poder demais — rodar: ALTER ROLE nimbus_app NOSUPERUSER NOBYPASSRLS NOCREATEROLE NOCREATEDB;\n';
  END IF;
  IF EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace JOIN pg_roles o ON o.oid = c.relowner
             WHERE n.nspname = 'nimbus' AND o.rolname = 'nimbus_app') THEN
    achados := achados || E'[BANCO-DONO] nimbus_app é dono de tabela — transferir para nimbus_dono (ALTER TABLE ... OWNER TO nimbus_dono).\n';
  END IF;
  IF pg_has_role('nimbus_app', 'nimbus_dono', 'MEMBER') OR pg_has_role('nimbus_app', 'nimbus_faxina', 'MEMBER') THEN
    achados := achados || E'[BANCO-HERANCA] nimbus_app é membro de outro papel — rodar: REVOKE nimbus_dono, nimbus_faxina FROM nimbus_app;\n';
  END IF;
  -- 3. O app nunca altera bloco nem lê registro de acesso.
  IF has_table_privilege('nimbus_app', 'nimbus.blocos', 'UPDATE') THEN
    achados := achados || E'[BANCO-UPDATE] nimbus_app pode alterar blocos — rodar: REVOKE UPDATE ON nimbus.blocos FROM nimbus_app;\n';
  END IF;
  IF has_table_privilege('nimbus_app', 'nimbus.registros_de_acesso', 'SELECT') THEN
    achados := achados || E'[BANCO-REGISTRO] nimbus_app pode ler registros de acesso — rodar: REVOKE SELECT ON nimbus.registros_de_acesso FROM nimbus_app;\n';
  END IF;
  -- 4. Nenhuma política aberta (USING true / WITH CHECK true) para o app.
  FOR r IN SELECT tablename, policyname FROM pg_policies WHERE schemaname = 'nimbus' AND 'nimbus_app' = ANY(roles)
           AND (coalesce(qual, '') IN ('true', '(true)') OR coalesce(with_check, '') IN ('true', '(true)')) LOOP
    achados := achados || format(E'[BANCO-POLITICA] política aberta %s em nimbus.%s — restaurar a do esquema.sql\n', r.policyname, r.tablename);
  END LOOP;
  -- 5. A restrição de 48 h existe no próprio banco.
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'prazo_maximo_48h') THEN
    achados := achados || E'[BANCO-PRAZO] restrição prazo_maximo_48h ausente — restaurar do esquema.sql\n';
  END IF;
  IF achados <> '' THEN
    RAISE EXCEPTION E'SENTINELA DO BANCO: TRAVADO. O servidor não pode atender.\n%', achados;
  END IF;
END $$;
SELECT 'SENTINELA DO BANCO: tudo conferido' AS resultado;
