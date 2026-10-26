-- Testes do esquema: cada bloco precisa terminar sem erro; qualquer ASSERT falso derruba o teste.
\set ON_ERROR_STOP on
SET client_min_messages = warning;

-- Dados de teste (como dono, com ficha e caixa ajustadas).
BEGIN;
SET LOCAL ROLE nimbus_app;
SET LOCAL nimbus.deposito = 'sim';
INSERT INTO nimbus.caixas (endereco, chave_publica) VALUES (decode(repeat('aa',32),'hex'), decode(repeat('01',64),'hex'));
INSERT INTO nimbus.caixas (endereco, chave_publica) VALUES (decode(repeat('bb',32),'hex'), decode(repeat('02',64),'hex'));
INSERT INTO nimbus.blocos (id, endereco, bloco, expira_em) VALUES (decode(repeat('01',16),'hex'), decode(repeat('aa',32),'hex'), decode(repeat('ff',256),'hex'), now() + interval '1 hour');
INSERT INTO nimbus.blocos (id, endereco, bloco, expira_em) VALUES (decode(repeat('02',16),'hex'), decode(repeat('bb',32),'hex'), decode(repeat('ee',256),'hex'), now() + interval '1 hour');
COMMIT;

-- 1. Sem caixa autorizada, nada aparece (negar por padrão).
BEGIN; SET LOCAL ROLE nimbus_app;
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos) = 0, 'sem caixa autorizada nada pode aparecer';
            ASSERT (SELECT count(*) FROM nimbus.caixas) = 0, 'sem caixa autorizada nenhuma caixa aparece'; END $$;
COMMIT;

-- 2. Com a caixa A autorizada, só os blocos de A.
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.caixa = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos) = 1, 'só o bloco da própria caixa';
            ASSERT (SELECT count(*) FROM nimbus.blocos WHERE endereco = decode(repeat('bb',32),'hex')) = 0, 'bloco de outra caixa nunca aparece'; END $$;
-- 2b. Apagar bloco de outra caixa não apaga nada.
DELETE FROM nimbus.blocos WHERE id = decode(repeat('02',16),'hex');
COMMIT;
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.caixa = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos) = 1, 'o bloco de B continua lá'; END $$;
COMMIT;

-- 2c. DELETE sem WHERE (sem leitura de colunas, só a política de DELETE vale): apaga só da própria caixa.
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.caixa = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
DELETE FROM nimbus.blocos;
RESET ROLE;
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos WHERE endereco = decode(repeat('bb',32),'hex')) = 1, 'DELETE sem WHERE apagou bloco de outra caixa'; END $$;
ROLLBACK;

-- 2d. O servidor ajusta a caixa com set_config e parâmetro (nunca concatenando texto em SQL).
BEGIN; SET LOCAL ROLE nimbus_app;
SELECT set_config('nimbus.caixa', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', true);
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos) = 1, 'set_config com a caixa autorizada funciona'; END $$;
SELECT set_config('nimbus.caixa', $q$aa'; RESET ROLE; --$q$, true);
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos) = 0, 'valor com tentativa de injeção vira só texto e não abre nada'; END $$;
COMMIT;

-- 3. Valor torto no ajuste da caixa (injeção, tamanho errado, maiúsculas) não abre nada.
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.caixa = 'aa'' OR true --';
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos) = 0, 'ajuste torto não abre nada'; END $$;
COMMIT;
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.caixa = 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos) = 0, 'maiúsculas não abrem'; END $$;
COMMIT;

-- 4. Sem ficha, não deposita nem cria caixa.
BEGIN; SET LOCAL ROLE nimbus_app;
DO $$ BEGIN
  BEGIN INSERT INTO nimbus.blocos (id, endereco, bloco, expira_em) VALUES (decode(repeat('03',16),'hex'), decode(repeat('aa',32),'hex'), decode(repeat('ff',256),'hex'), now() + interval '1 hour');
        RAISE EXCEPTION 'depositou sem ficha';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  BEGIN INSERT INTO nimbus.caixas (endereco, chave_publica) VALUES (decode(repeat('cc',32),'hex'), decode(repeat('03',64),'hex'));
        RAISE EXCEPTION 'criou caixa sem ficha';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
END $$;
COMMIT;

-- 5. Prazo acima de 48 h é recusado pelo próprio banco, mesmo com ficha.
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.deposito = 'sim';
DO $$ BEGIN
  BEGIN INSERT INTO nimbus.blocos (id, endereco, bloco, expira_em) VALUES (decode(repeat('04',16),'hex'), decode(repeat('aa',32),'hex'), decode(repeat('ff',256),'hex'), now() + interval '49 hours');
        RAISE EXCEPTION 'aceitou prazo acima de 48 h';
  EXCEPTION WHEN check_violation OR insufficient_privilege THEN NULL; END;
  BEGIN INSERT INTO nimbus.blocos (id, endereco, bloco, expira_em) VALUES (decode(repeat('05',16),'hex'), decode(repeat('aa',32),'hex'), decode(repeat('ff',10),'hex'), now() + interval '1 hour');
        RAISE EXCEPTION 'aceitou bloco menor que o preenchimento mínimo';
  EXCEPTION WHEN check_violation THEN NULL; END;
END $$;
COMMIT;

-- 5b. A restrição de 48 h vale no próprio banco, mesmo para quem não passa pela RLS (administrador).
DO $$ BEGIN
  BEGIN INSERT INTO nimbus.blocos (id, endereco, bloco, expira_em) VALUES (decode(repeat('08',16),'hex'), decode(repeat('aa',32),'hex'), decode(repeat('ff',256),'hex'), now() + interval '49 hours');
        RAISE EXCEPTION 'restrição de 48 h não existe no banco';
  EXCEPTION WHEN check_violation THEN NULL; END;
END $$;

-- 6. Bloco não se altera; registros de acesso não se leem; o app não cria tabela nem mexe em papel.
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.caixa = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
DO $$ BEGIN
  BEGIN UPDATE nimbus.blocos SET expira_em = now() + interval '40 hours'; RAISE EXCEPTION 'alterou bloco';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  BEGIN PERFORM count(*) FROM nimbus.registros_de_acesso; RAISE EXCEPTION 'leu registros de acesso';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  BEGIN EXECUTE 'CREATE TABLE nimbus.x (a int)'; RAISE EXCEPTION 'criou tabela';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  BEGIN EXECUTE 'ALTER TABLE nimbus.blocos DISABLE ROW LEVEL SECURITY'; RAISE EXCEPTION 'desligou RLS';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
  BEGIN PERFORM nimbus.apagar_vencidos(); RAISE EXCEPTION 'app rodou a faxina';
  EXCEPTION WHEN insufficient_privilege THEN NULL; END;
END $$;
INSERT INTO nimbus.registros_de_acesso (cifrado, guardar_ate) VALUES (decode(repeat('09',64),'hex'), now() + interval '6 months');
COMMIT;

-- 7. Bloco vencido some da leitura na hora e é apagado pela faxina.
-- Simula um bloco vencido (inserido pelo administrador do banco, fora do app).
INSERT INTO nimbus.blocos (id, endereco, bloco, criado_em, expira_em) VALUES (decode(repeat('06',16),'hex'), decode(repeat('aa',32),'hex'), decode(repeat('ff',256),'hex'), now() - interval '3 hours', now() - interval '1 hour');
BEGIN; SET LOCAL ROLE nimbus_app; SET LOCAL nimbus.caixa = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos WHERE id = decode(repeat('06',16),'hex')) = 0, 'vencido não aparece'; END $$;
COMMIT;
BEGIN; SET LOCAL ROLE nimbus_faxina;
DO $$ BEGIN ASSERT nimbus.apagar_vencidos() >= 1, 'faxina apagou o vencido'; END $$;
COMMIT;
DO $$ BEGIN ASSERT (SELECT count(*) FROM nimbus.blocos WHERE id = decode(repeat('06',16),'hex')) = 0, 'vencido foi apagado de verdade';
            ASSERT (SELECT count(*) FROM nimbus.blocos) = 2, 'a faxina não apagou o que ainda vale'; END $$;

SELECT 'BANCO: todos os testes passaram' AS resultado;
