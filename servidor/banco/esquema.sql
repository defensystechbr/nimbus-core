-- Esquema do banco do servidor Nimbus — PostgreSQL 16
-- Versão 37 · gerado e testado com teste_do_banco.sh
--
-- O banco guarda só o que o servidor precisa para entregar: caixas (endereço + chave pública da caixa) e
-- blocos cifrados com prazo de até 48 h. Nenhuma coluna tem texto de mensagem, nome, telefone, e-mail ou IP
-- do aparelho. Mesmo que alguém leve o banco inteiro, leva só bloco cifrado com prazo.
--
-- Defesa em camadas:
-- 1. O app conecta com o papel nimbus_app: não é dono das tabelas, não é superusuário, não ignora RLS.
-- 2. RLS ligada e FORÇADA em todas as tabelas: sem a caixa autorizada pelo porteiro (módulo autenticacao),
--    nenhuma linha aparece. O padrão é negar.
-- 3. Restrições no próprio banco: prazo máximo de 48 h, tamanhos fixos, nenhum UPDATE em bloco.
-- 4. Registros de acesso (Marco Civil, art. 15) ficam cifrados para uma chave que não está no servidor;
--    o app só escreve, nunca lê.

BEGIN;

-- Papéis (sem senha aqui: a conexão do app usa certificado, configurado na infraestrutura).
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'nimbus_dono') THEN CREATE ROLE nimbus_dono NOLOGIN; END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'nimbus_app') THEN
    CREATE ROLE nimbus_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS NOREPLICATION; END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'nimbus_faxina') THEN
    CREATE ROLE nimbus_faxina LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS NOREPLICATION; END IF;
END $$;

REVOKE ALL ON SCHEMA public FROM PUBLIC;
CREATE SCHEMA nimbus AUTHORIZATION nimbus_dono;
REVOKE ALL ON SCHEMA nimbus FROM PUBLIC;
GRANT USAGE ON SCHEMA nimbus TO nimbus_app, nimbus_faxina;
SET ROLE nimbus_dono;

-- Caixa autorizada pelo porteiro nesta transação: o servidor chama SELECT set_config('nimbus.caixa', $1, true)
-- com o hexadecimal como PARÂMETRO (nunca SET LOCAL montado com texto: SET não aceita parâmetro).
-- Atenção: a coluna "endereco" guarda o ÍNDICE CEGO do endereço (metadados::ChavesDeIndice), nunca o
-- endereço em si; a chave do índice vive fora do banco e é destruída 48 h depois do fim da época.
-- Sem o ajuste, devolve NULL, e NULL nunca é igual a nada: nenhuma linha passa.
CREATE FUNCTION nimbus.caixa_autorizada() RETURNS bytea LANGUAGE sql STABLE AS $$
  SELECT CASE WHEN current_setting('nimbus.caixa', true) ~ '^[0-9a-f]{64}$'
              THEN decode(current_setting('nimbus.caixa', true), 'hex') END
$$;
-- Depósito autorizado (ficha anônima conferida pelo servidor nesta transação).
CREATE FUNCTION nimbus.deposito_autorizado() RETURNS boolean LANGUAGE sql STABLE AS $$
  SELECT coalesce(current_setting('nimbus.deposito', true), '') = 'sim'
$$;

CREATE TABLE nimbus.caixas (
  endereco      bytea PRIMARY KEY CHECK (octet_length(endereco) = 32),
  chave_publica bytea NOT NULL CHECK (octet_length(chave_publica) BETWEEN 32 AND 4096),
  criada_em     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE nimbus.blocos (
  id        bytea PRIMARY KEY CHECK (octet_length(id) = 16),
  endereco  bytea NOT NULL REFERENCES nimbus.caixas (endereco) ON DELETE CASCADE,
  bloco     bytea NOT NULL CHECK (octet_length(bloco) BETWEEN 256 AND 67117056),
  criado_em timestamptz NOT NULL DEFAULT now(),
  expira_em timestamptz NOT NULL,
  CONSTRAINT prazo_maximo_48h CHECK (expira_em > criado_em AND expira_em <= criado_em + interval '48 hours')
);
CREATE INDEX blocos_por_caixa ON nimbus.blocos (endereco);
CREATE INDEX blocos_por_prazo ON nimbus.blocos (expira_em);

CREATE TABLE nimbus.registros_de_acesso (
  id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  cifrado     bytea NOT NULL CHECK (octet_length(cifrado) BETWEEN 64 AND 8192),
  guardar_ate timestamptz NOT NULL
);

-- RLS ligada e forçada (vale até para o dono).
ALTER TABLE nimbus.caixas ENABLE ROW LEVEL SECURITY;              ALTER TABLE nimbus.caixas FORCE ROW LEVEL SECURITY;
ALTER TABLE nimbus.blocos ENABLE ROW LEVEL SECURITY;              ALTER TABLE nimbus.blocos FORCE ROW LEVEL SECURITY;
ALTER TABLE nimbus.registros_de_acesso ENABLE ROW LEVEL SECURITY; ALTER TABLE nimbus.registros_de_acesso FORCE ROW LEVEL SECURITY;

-- Caixas: ler só a própria; criar só com ficha.
CREATE POLICY caixa_ler_a_propria ON nimbus.caixas FOR SELECT TO nimbus_app USING (endereco = nimbus.caixa_autorizada());
CREATE POLICY caixa_criar_com_ficha ON nimbus.caixas FOR INSERT TO nimbus_app WITH CHECK (nimbus.deposito_autorizado());

-- Blocos: retirar e apagar só da própria caixa e só o que não venceu; depositar só com ficha e prazo de até 48 h.
CREATE POLICY bloco_ler_da_propria ON nimbus.blocos FOR SELECT TO nimbus_app
  USING (endereco = nimbus.caixa_autorizada() AND expira_em > now());
CREATE POLICY bloco_apagar_da_propria ON nimbus.blocos FOR DELETE TO nimbus_app
  USING (endereco = nimbus.caixa_autorizada());
CREATE POLICY bloco_depositar_com_ficha ON nimbus.blocos FOR INSERT TO nimbus_app
  WITH CHECK (nimbus.deposito_autorizado() AND expira_em <= now() + interval '48 hours');
-- Nenhuma política de UPDATE: bloco não se altera.

-- Registros de acesso: o app só escreve; ninguém lê por este papel.
CREATE POLICY registro_so_escrever ON nimbus.registros_de_acesso FOR INSERT TO nimbus_app
  WITH CHECK (guardar_ate <= now() + interval '6 months 1 day');

-- Faxina: apaga o que venceu. Roda com privilégio do dono, mas só faz isso.
CREATE FUNCTION nimbus.apagar_vencidos() RETURNS bigint LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog, nimbus AS $$
DECLARE n bigint; m bigint;
BEGIN
  DELETE FROM nimbus.blocos WHERE expira_em <= now(); GET DIAGNOSTICS n = ROW_COUNT;
  DELETE FROM nimbus.registros_de_acesso WHERE guardar_ate <= now(); GET DIAGNOSTICS m = ROW_COUNT;
  RETURN n + m;
END $$;
-- Para a faxina funcionar com RLS forçada, o dono precisa de política própria de DELETE.
-- (O DELETE com WHERE também passa pelas políticas de SELECT: o dono só "vê" o que já venceu.)
CREATE POLICY faxina_ver_blocos_vencidos ON nimbus.blocos FOR SELECT TO nimbus_dono USING (expira_em <= now());
CREATE POLICY faxina_blocos ON nimbus.blocos FOR DELETE TO nimbus_dono USING (expira_em <= now());
CREATE POLICY faxina_ver_registros_vencidos ON nimbus.registros_de_acesso FOR SELECT TO nimbus_dono USING (guardar_ate <= now());
CREATE POLICY faxina_registros ON nimbus.registros_de_acesso FOR DELETE TO nimbus_dono USING (guardar_ate <= now());

RESET ROLE;
REVOKE ALL ON ALL TABLES IN SCHEMA nimbus FROM PUBLIC;
REVOKE ALL ON FUNCTION nimbus.apagar_vencidos() FROM PUBLIC;
GRANT SELECT, INSERT ON nimbus.caixas TO nimbus_app;
GRANT SELECT, INSERT, DELETE ON nimbus.blocos TO nimbus_app;
GRANT INSERT ON nimbus.registros_de_acesso TO nimbus_app;
GRANT EXECUTE ON FUNCTION nimbus.caixa_autorizada(), nimbus.deposito_autorizado() TO nimbus_app;
GRANT EXECUTE ON FUNCTION nimbus.apagar_vencidos() TO nimbus_faxina;

COMMIT;
