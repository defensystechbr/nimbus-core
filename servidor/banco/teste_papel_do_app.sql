-- Conectado COMO nimbus_app (como o servidor real): o papel do app não vira outro papel
-- nem por SET ROLE, nem por set_config, nem por SET SESSION AUTHORIZATION.
\set ON_ERROR_STOP on
DO $$ BEGIN
  BEGIN EXECUTE 'SET ROLE nimbus_dono'; RAISE EXCEPTION 'nimbus_app virou dono';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  BEGIN PERFORM set_config('role', 'nimbus_faxina', true); RAISE EXCEPTION 'nimbus_app virou faxina';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  BEGIN EXECUTE 'SET SESSION AUTHORIZATION postgres'; RAISE EXCEPTION 'nimbus_app virou superusuário';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  ASSERT (SELECT count(*) FROM nimbus.blocos) = 0, 'sem caixa autorizada, nada aparece também na conexão real do app';
END $$;
SELECT 'PAPEL DO APP: não troca de papel' AS resultado;
