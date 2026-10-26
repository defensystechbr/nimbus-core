//! # Módulo `sessoes` — sessão única e anti-clonagem (ID2, CG1) — lógica do servidor
//!
//! Distinção essencial para não virar arma contra o próprio dono:
//! * Pedido **sem credencial válida** (quem só sabe o identificador da conta) é simplesmente **recusado**
//!   e contado — **não derruba ninguém**. Senão, qualquer pessoa derrubaria o dono sem parar.
//! * Pedido **com credencial válida** vindo de aparelho não registrado, de app sem atestação ou em
//!   paralelo a uma sessão ativa é sinal de **clone**: derruba **todas** as sessões, inclusive a legítima
//!   (o servidor não sabe qual é a verdadeira), exige reautenticação, gera código de incidente para o
//!   dono e só uma contagem anônima para o painel.
//!
//! **Expiração automática da sessão do computador (v37, quinta rodada).** O celular e um computador
//! vinculado continuam coexistindo de propósito (PC1 — ver o campo `ativas` abaixo); o que esta rodada
//! adiciona é um TETO de duração para a sessão do COMPUTADOR especificamente (nunca do celular): no
//! máximo 12 horas, com opções menores escolhidas pelo dono (ex.: 1, 2, 4, 6, 8, 12h). Isto é aditivo e
//! separado da detecção de clone — não mexe em `conectar`, `Resultado` nem nos gatilhos existentes, só
//! acrescenta um relógio por sessão de computador que a ponte consulta e aplica (`sessao_de_computador_
//! expirou` + `expirar_computadores_vencidos`). Por que só o computador: é o mesmo raciocínio do PC1 —
//! o computador é sempre o aparelho de confiança mais baixa (nunca passa do nível amarelo em
//! `integridade`), e uma sessão aberta esquecida num computador compartilhado ou de escritório é o caso
//! de uso real que este teto evita; o celular, que já exige biometria para abrir o app, não precisa disso.
use rand_core::{OsRng, RngCore};
use std::collections::{HashMap, HashSet};

pub type IdConta = [u8; 32];
pub type IdAparelho = [u8; 32];

/// Teto de duração da sessão do computador: 1 a 12 horas, escolhido pelo dono (ex.: 1, 2, 4, 6, 8, 12h
/// na tela; valores fora da faixa são grampeados, nunca rejeitados — mesmo padrão de `coacao::TetoDiurno`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TetoDeSessaoDoComputador(u8);

pub const TETO_SESSAO_COMPUTADOR_MINIMO_HORAS: u8 = 1;
pub const TETO_SESSAO_COMPUTADOR_MAXIMO_HORAS: u8 = 12;

impl TetoDeSessaoDoComputador {
    pub fn de_horas(horas: u8) -> Self {
        TetoDeSessaoDoComputador(horas.clamp(TETO_SESSAO_COMPUTADOR_MINIMO_HORAS, TETO_SESSAO_COMPUTADOR_MAXIMO_HORAS))
    }
    pub fn em_segundos(self) -> u64 { self.0 as u64 * 3_600 }
}

/// Teto padrão quando o dono não escolheu outro: o máximo permitido (12h), nunca "sem teto" — uma sessão
/// de computador sem teto nenhum seria voltar ao comportamento de antes desta rodada.
impl Default for TetoDeSessaoDoComputador {
    fn default() -> Self { TetoDeSessaoDoComputador(TETO_SESSAO_COMPUTADOR_MAXIMO_HORAS) }
}

/// **Tempo de arrependimento ao vincular um aparelho novo (Rodada 20, 11/10/2026, pedido do dono: "nao
/// seria legal ter um tempo de arrependimento quando da autorização para outro app, aparelho ou
/// programa?").** Janela curta, em minutos, logo depois que `pode_vincular` (abaixo) aceita um vínculo —
/// não é um atraso que impede o aparelho novo de funcionar (ele já funciona normalmente desde o primeiro
/// instante, mesmo espírito de "nunca trancar o usuário para fora" que o dono pediu nesta mesma rodada
/// para a frase de recuperação) — é só o quanto, DEPOIS do fato, qualquer um dos aparelhos JÁ ativos
/// pode olhar "um aparelho novo foi vinculado" e desfazer com um toque, sem precisar explicar por quê,
/// antes de a própria pessoa ganhar confiança de que foi ela mesma. **O mecanismo de desfazer já existia
/// antes desta rodada e não precisou de nenhuma chave ou objeto novo: `Registro::revogar_aparelho` já
/// revoga qualquer aparelho, a qualquer momento, dentro ou fora desta janela** (ver o teste
/// `revogar_aparelho_funciona_dentro_e_fora_da_janela_de_arrependimento`, abaixo) — o que faltava, e esta
/// rodada acrescenta, é só SABER quando um aparelho foi vinculado (`vinculados_em`, abaixo) para a ponte
/// decidir quando vale a pena destacar esse botão com mais urgência na tela (ex.: notificação imediata
/// aos outros aparelhos, badge "vinculado há 3 min"). Grampeada entre 1 e 60 minutos, nunca "zero" (sem
/// janela nenhuma seria voltar a não ter nada) nem um valor longo demais (o objetivo é uma janela CURTA de
/// atenção, não um segundo período de carência como o da recuperação de conta — que já existe, é de 72h,
/// e cobre um risco diferente: recuperar a conta do ZERO, não vincular mais um aparelho a uma conta já
/// acessível; ver `crate::recuperacao::CARENCIA_SEGUNDOS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodoDeArrependimento(u8);

pub const ARREPENDIMENTO_MINIMO_MINUTOS: u8 = 1;
pub const ARREPENDIMENTO_MAXIMO_MINUTOS: u8 = 60;

impl PeriodoDeArrependimento {
    pub fn de_minutos(minutos: u8) -> Self {
        PeriodoDeArrependimento(minutos.clamp(ARREPENDIMENTO_MINIMO_MINUTOS, ARREPENDIMENTO_MAXIMO_MINUTOS))
    }
    pub fn em_segundos(self) -> u64 { self.0 as u64 * 60 }
}

/// Padrão sugerido quando o dono não escolheu outro: 10 minutos — tempo real para uma notificação chegar
/// e alguém reagir, sem virar um segundo período de espera longo (esse já existe, é a carência de
/// recuperação). **Marcado como recomendação desta rodada, não como um número medido ou pedido
/// explicitamente pelo dono** — different de `TetoDeSessaoDoComputador::default`, que reusa um valor
/// (12h) que já vinha de uma decisão anterior; aqui não havia nenhum número anterior para reaproveitar.
/// Ver "Verificar antes de usar" desta rodada.
impl Default for PeriodoDeArrependimento {
    fn default() -> Self { PeriodoDeArrependimento(10) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gatilho { SessaoParalela, AparelhoNaoRegistrado, AtestacaoFalhou, SemCredencial }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resultado {
    Aceita,
    /// Tentativa sem credencial válida: recusada, sem afetar o dono.
    Recusada,
    /// Sinal de clone: todas as sessões derrubadas. O `codigo_incidente` vai cifrado só para o dono.
    TodasDerrubadas { gatilho: Gatilho, codigo_incidente: [u8; 8] },
}

#[derive(Default, Debug)]
pub struct Registro {
    aparelhos: HashMap<IdConta, HashSet<IdAparelho>>,
    /// Uma sessão por aparelho registrado (celular e computador vinculado convivem, PC1). Clone é o mesmo
    /// aparelho conectado duas vezes, ou um aparelho que não foi registrado.
    ativas: HashMap<IdConta, HashSet<IdAparelho>>,
    /// Painel da operação: só contagens por tipo. Sem conta, sem aparelho, sem IP.
    pub contagens_anonimas: HashMap<Gatilho, u64>,
    /// Quantas conexões ACEITAS a conta teve HOJE (dia corrente, identificado por um índice de dia — ex.:
    /// dias desde a época — que o próprio app decide e passa). Só o dia de hoje: vira 1 no primeiro acesso
    /// do dia seguinte, nunca acumula um histórico de dias passados — mantém a mesma regra de "sem
    /// histórico" do resto deste módulo (nada gravado em disco, e agora também nada que sobreviva ao dia).
    /// É um sinal para o próprio dono perceber uso fora do padrão (ex.: muito mais conexões que o normal no
    /// mesmo dia) — nunca uma prova de invasão: reconexões legítimas (perda de rede, trocar de Wi-Fi para
    /// dados) também contam aqui.
    conexoes_do_dia: HashMap<IdConta, (u64, u32)>,
    /// Sessões de COMPUTADOR com teto de duração (v37, quinta rodada): `(conectado_em, teto_segundos)`.
    /// Só aparelhos registrados aqui através de `registrar_conexao_de_computador` têm teto — o celular
    /// nunca entra neste mapa, e por isso nunca expira por tempo.
    sessoes_de_computador: HashMap<(IdConta, IdAparelho), (u64, u64)>,
    /// Quando cada aparelho foi vinculado (Rodada 20) — só quem entra por `registrar_vinculo` (abaixo)
    /// tem uma entrada aqui; quem entra pelo `registrar_aparelho` "cru" (ex.: o primeiro aparelho, criado
    /// junto com a própria conta, onde não existe "antes" para se arrepender) nunca aparece neste mapa, e
    /// por isso `dentro_do_periodo_de_arrependimento` sempre devolve `false` para ele — tratado como já
    /// fora de qualquer janela, nunca como "suspeito por não ter data".
    vinculados_em: HashMap<(IdConta, IdAparelho), u64>,
}

impl Registro {
    pub fn registrar_aparelho(&mut self, conta: IdConta, aparelho: IdAparelho) {
        let _ = self.aparelhos.entry(conta).or_default().insert(aparelho);
    }

    pub fn revogar_aparelho(&mut self, conta: IdConta, aparelho: IdAparelho) {
        if let Some(s) = self.aparelhos.get_mut(&conta) { let _ = s.remove(&aparelho); }
        if let Some(a) = self.ativas.get_mut(&conta) { let _ = a.remove(&aparelho); }
        let _ = self.vinculados_em.remove(&(conta, aparelho));
    }

    /// Registra um aparelho E a data do vínculo (Rodada 20) — a ponte chama isto, em vez do
    /// `registrar_aparelho` "cru", sempre que o vínculo vem da cerimônia de verdade (`pode_vincular`,
    /// abaixo): é essa data que alimenta `dentro_do_periodo_de_arrependimento`. `registrar_aparelho`
    /// continua existindo sem mudança (usado por quem já tinha testes ou chamadas escritas contra ele, e
    /// pelo primeiro aparelho da conta, onde não há "arrependimento" possível) — esta função não o
    /// substitui, só acrescenta a data para quem precisa dela.
    pub fn registrar_vinculo(&mut self, conta: IdConta, aparelho: IdAparelho, vinculado_em: u64) {
        self.registrar_aparelho(conta, aparelho);
        let _ = self.vinculados_em.insert((conta, aparelho), vinculado_em);
    }

    /// Este aparelho ainda está dentro do tempo de arrependimento do próprio vínculo? `false` tanto para
    /// "já passou a janela" quanto para "nunca foi registrado com `registrar_vinculo`" (ver a nota no
    /// campo `vinculados_em`, acima) — os dois casos têm a mesma resposta prática: nenhum motivo para a
    /// tela destacar o botão de desfazer com urgência extra. Não decide nada por si: `revogar_aparelho`
    /// já funciona a qualquer momento, dentro ou fora desta janela — esta função só informa a ponte
    /// sobre QUANDO vale a pena chamar mais atenção para essa opção já existente.
    pub fn dentro_do_periodo_de_arrependimento(&self, conta: &IdConta, aparelho: &IdAparelho, agora: u64, janela: PeriodoDeArrependimento) -> bool {
        self.vinculados_em.get(&(*conta, *aparelho))
            .is_some_and(|vinculado_em| agora < vinculado_em.saturating_add(janela.em_segundos()))
    }

    /// Revoga TODOS os aparelhos hoje registrados para a conta — nunca uma lista parcial escolhida
    /// por quem chama. Usar no MESMO passo em que uma recuperação por guardiões conclui
    /// (`recuperacao::pode_concluir` devolveu `Ok`), antes de registrar o aparelho novo — nunca
    /// depois, nunca como etapa separada que a ponte possa esquecer.
    ///
    /// Achado desta rodada, checado contra o código, não presumido: sem esta chamada, a sessão do
    /// aparelho perdido SOBREVIVE indefinidamente depois de uma recuperação concluída. `conectar`
    /// só deriva `Gatilho::SessaoParalela` para o MESMO `IdAparelho` tentando conectar de novo
    /// enquanto já ativo, ou `AparelhoNaoRegistrado` para um aparelho nunca registrado — o aparelho
    /// novo da recuperação, registrado como parte do próprio fluxo de recuperação, nunca aciona
    /// nenhum dos dois, então nada no caminho normal de `conectar` derruba o aparelho antigo por
    /// conta própria.
    ///
    /// Por que TODOS, não só "o que parece comprometido": quem precisou de recuperação por
    /// guardiões, por definição, não tem mais controle confiável sobre nenhum aparelho de antes — se
    /// tivesse, usaria `pode_vincular` (regra 16/35), não este caminho. Revogar parcialmente exigiria
    /// a ponte adivinhar qual aparelho é seguro manter, um julgamento que este módulo não tem como
    /// fazer com segurança. Quem ainda tiver um aparelho legítimo de antes em mãos simplesmente vincula
    /// de novo, pela cerimônia normal, depois da recuperação — sem perda de segurança, só um passo
    /// extra deliberado.
    pub fn revogar_todos_os_aparelhos(&mut self, conta: IdConta) {
        let _ = self.aparelhos.remove(&conta);
        let _ = self.ativas.remove(&conta);
        self.sessoes_de_computador.retain(|(c, _), _| *c != conta);
        self.vinculados_em.retain(|(c, _), _| *c != conta);
    }

    fn contar(&mut self, g: Gatilho) { *self.contagens_anonimas.entry(g).or_default() += 1; }

    fn incidente(&mut self, conta: IdConta, gatilho: Gatilho) -> Resultado {
        let _ = self.ativas.remove(&conta);
        self.contar(gatilho);
        let mut codigo_incidente = [0u8; 8];
        OsRng.fill_bytes(&mut codigo_incidente);
        Resultado::TodasDerrubadas { gatilho, codigo_incidente }
    }

    /// `credencial_valida`: o pedido provou posse da credencial da conta (ex.: assinatura com a chave da
    /// conta). `atestado`: o app passou na atestação da Apple ou do Google. `dia`: índice do dia corrente
    /// (ex.: dias desde a época), só para o contador de conexões do dia — nunca usado para decidir nada de
    /// segurança, só para o dono acompanhar o próprio padrão de uso.
    pub fn conectar(&mut self, conta: IdConta, aparelho: IdAparelho, credencial_valida: bool, atestado: bool, dia: u64) -> Resultado {
        if !credencial_valida { self.contar(Gatilho::SemCredencial); return Resultado::Recusada; }
        if !atestado { return self.incidente(conta, Gatilho::AtestacaoFalhou); }
        if !self.aparelhos.get(&conta).is_some_and(|s| s.contains(&aparelho)) {
            return self.incidente(conta, Gatilho::AparelhoNaoRegistrado);
        }
        if self.ativas.get(&conta).is_some_and(|a| a.contains(&aparelho)) { return self.incidente(conta, Gatilho::SessaoParalela); }
        let _ = self.ativas.entry(conta).or_default().insert(aparelho);
        let entrada = self.conexoes_do_dia.entry(conta).or_insert((dia, 0));
        if entrada.0 != dia { *entrada = (dia, 0); }
        entrada.1 = entrada.1.saturating_add(1);
        Resultado::Aceita
    }

    pub fn desconectar(&mut self, conta: IdConta, aparelho: IdAparelho) {
        if let Some(a) = self.ativas.get_mut(&conta) { let _ = a.remove(&aparelho); }
    }

    pub fn tem_sessao_ativa(&self, conta: &IdConta) -> bool { self.ativas.get(conta).is_some_and(|a| !a.is_empty()) }

    /// Os aparelhos ativos AGORA, para a tela "aparelhos ativos" (nunca um histórico — só o instante atual).
    /// O nome de cada aparelho é dado pelo próprio dono no app e vive só localmente; aqui é só o
    /// identificador. Ordem não é significativa (vem de um `HashSet`).
    pub fn aparelhos_ativos_agora(&self, conta: &IdConta) -> Vec<IdAparelho> {
        self.ativas.get(conta).map(|a| a.iter().copied().collect()).unwrap_or_default()
    }

    /// Quantas conexões aceitas a conta teve no dia informado. Só responde para o dia de HOJE (o `dia` que
    /// a chamadora passa): se o último dia registrado for diferente, devolve 0 — nunca mistura ou soma
    /// dias diferentes, e nunca revela a contagem de um dia passado.
    pub fn conexoes_hoje(&self, conta: &IdConta, dia_de_hoje: u64) -> u32 {
        self.conexoes_do_dia.get(conta).filter(|(dia, _)| *dia == dia_de_hoje).map(|(_, n)| *n).unwrap_or(0)
    }

    /// Registra um teto de duração para a sessão de COMPUTADOR que acabou de ser aceita por `conectar`.
    /// A ponte chama isto logo depois de um `conectar` bem-sucedido, só quando a plataforma do aparelho é
    /// computador (nunca para celular). `agora`: instante da conexão, vindo de hora confiável do servidor
    /// (nunca do relógio do próprio aparelho — mesma exigência de sempre para qualquer prazo neste núcleo).
    pub fn registrar_conexao_de_computador(&mut self, conta: IdConta, aparelho: IdAparelho, agora: u64, teto: TetoDeSessaoDoComputador) {
        self.sessoes_de_computador.insert((conta, aparelho), (agora, teto.em_segundos()));
    }

    /// Esta sessão de computador específica já passou do próprio teto? Para um aparelho que nunca foi
    /// registrado como computador (ou é o celular), sempre `false` — o teto só vale para quem foi
    /// explicitamente marcado como computador.
    pub fn sessao_de_computador_expirou(&self, conta: &IdConta, aparelho: &IdAparelho, agora: u64) -> bool {
        self.sessoes_de_computador.get(&(*conta, *aparelho))
            .is_some_and(|(conectado_em, teto_segundos)| agora >= conectado_em.saturating_add(*teto_segundos))
    }

    /// Derruba (só) as sessões de computador já vencidas, em todas as contas, e devolve quais foram
    /// derrubadas — para a ponte avisar "sessão do computador encerrada por tempo" a quem pediu. Não
    /// afeta sessão de celular nem conta como incidente de clone: é um encerramento normal por prazo, não
    /// um sinal de ataque.
    pub fn expirar_computadores_vencidos(&mut self, agora: u64) -> Vec<(IdConta, IdAparelho)> {
        let vencidos: Vec<(IdConta, IdAparelho)> = self.sessoes_de_computador.iter()
            .filter(|(_, (conectado_em, teto_segundos))| agora >= conectado_em.saturating_add(*teto_segundos))
            .map(|(chave, _)| *chave).collect();
        for (conta, aparelho) in &vencidos {
            self.desconectar(*conta, *aparelho);
            self.sessoes_de_computador.remove(&(*conta, *aparelho));
        }
        vencidos
    }
}

// ---------------------------------------------------------------------------------------------------------
// Vínculo de aparelho novo (computador ou celular adicional) à prova de golpe (EO2). O golpe conhecido nos
// mensageiros: alguém manda um QR "para verificar sua conta" ou "entrar no grupo"; a vítima escaneia e vincula,
// sem saber, o computador do golpista. Por isso: o vínculo só começa na tela "Vincular aparelho" do próprio
// app (nunca por link, câmera do sistema ou QR recebido), exige o celular no nível verde e biometria, e as
// palavras de confirmação têm de aparecer iguais nos dois aparelhos — lado a lado, nas mãos do dono.
// ---------------------------------------------------------------------------------------------------------

/// De onde partiu o pedido de vínculo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrigemDoVinculo {
    /// O dono abriu "Vincular aparelho" no próprio app e leu o QR mostrado no aparelho novo.
    TelaVincularDoApp,
    /// Link recebido, câmera do sistema ou QR que chegou por mensagem: recusado sempre.
    LinkOuCodigoRecebido,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeVinculo { OrigemNaoPermitida, CelularNaoVerde, SemBiometria, PalavrasNaoConferidas }

/// Palavras que os dois aparelhos mostram durante o vínculo: as mesmas, se ninguém estiver no meio.
pub fn palavras_do_vinculo(chave_do_celular: &[u8], chave_do_aparelho_novo: &[u8]) -> [&'static str; 6] {
    crate::convites::palavras_com_dominio(b"NimbusOne/v1/vinculo", chave_do_celular, chave_do_aparelho_novo)
}

/// Decide se o vínculo pode ser concluído. Todas as condições, sem exceção.
pub fn pode_vincular(origem: OrigemDoVinculo, nivel_do_celular: crate::integridade::Nivel, biometria_ok: bool,
                     palavras_conferidas: bool) -> Result<(), RecusaDeVinculo> {
    if origem != OrigemDoVinculo::TelaVincularDoApp { return Err(RecusaDeVinculo::OrigemNaoPermitida); }
    if nivel_do_celular != crate::integridade::Nivel::Verde { return Err(RecusaDeVinculo::CelularNaoVerde); }
    if !biometria_ok { return Err(RecusaDeVinculo::SemBiometria); }
    if !palavras_conferidas { return Err(RecusaDeVinculo::PalavrasNaoConferidas); }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    const CONTA: IdConta = [1; 32];
    const APARELHO: IdAparelho = [2; 32];
    const CLONE: IdAparelho = [3; 32];

    #[test]
    fn sessao_unica_normal() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        assert_eq!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::Aceita);
    }

    #[test]
    fn quem_so_sabe_o_identificador_nao_derruba_o_dono() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        let _ = r.conectar(CONTA, APARELHO, true, true, 1);
        for _ in 0..100 { assert_eq!(r.conectar(CONTA, CLONE, false, true, 1), Resultado::Recusada); }
        assert!(r.tem_sessao_ativa(&CONTA), "o dono continua conectado");
        assert_eq!(r.contagens_anonimas.get(&Gatilho::SemCredencial), Some(&100));
    }

    #[test]
    fn aparelhos_ativos_agora_mostra_so_quem_esta_conectado_neste_instante() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        assert!(r.aparelhos_ativos_agora(&CONTA).is_empty(), "ninguém conectado ainda");
        let _ = r.conectar(CONTA, APARELHO, true, true, 1);
        assert_eq!(r.aparelhos_ativos_agora(&CONTA), vec![APARELHO]);
        r.desconectar(CONTA, APARELHO);
        assert!(r.aparelhos_ativos_agora(&CONTA).is_empty(), "desconectou: some da lista, não vira histórico");
    }

    #[test]
    fn conexoes_hoje_conta_so_o_dia_de_hoje_e_zera_sozinho_no_dia_seguinte() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        assert_eq!(r.conexoes_hoje(&CONTA, 10), 0, "nada ainda");
        let _ = r.conectar(CONTA, APARELHO, true, true, 10);
        r.desconectar(CONTA, APARELHO);
        let _ = r.conectar(CONTA, APARELHO, true, true, 10);
        r.desconectar(CONTA, APARELHO);
        let _ = r.conectar(CONTA, APARELHO, true, true, 10);
        assert_eq!(r.conexoes_hoje(&CONTA, 10), 3, "3 conexões aceitas no dia 10");
        assert_eq!(r.conexoes_hoje(&CONTA, 9), 0, "nunca mistura com outro dia, nem o de ontem");
        r.desconectar(CONTA, APARELHO);
        let _ = r.conectar(CONTA, APARELHO, true, true, 11);
        assert_eq!(r.conexoes_hoje(&CONTA, 11), 1, "virou o dia: contador reinicia sozinho");
        assert_eq!(r.conexoes_hoje(&CONTA, 10), 0, "o dia anterior não fica acessível depois que o dia vira: sem histórico");
    }

    #[test]
    fn conexao_recusada_ou_que_derruba_tudo_nao_conta_como_conexao_aceita() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        let _ = r.conectar(CONTA, CLONE, false, true, 5); // recusada: sem credencial
        assert_eq!(r.conexoes_hoje(&CONTA, 5), 0);
        let _ = r.conectar(CONTA, CLONE, true, true, 5); // derruba tudo: aparelho não registrado
        assert_eq!(r.conexoes_hoje(&CONTA, 5), 0, "só conexão ACEITA conta");
    }

    #[test]
    fn teto_de_sessao_do_computador_e_grampeado_entre_1_e_12_horas() {
        assert_eq!(TetoDeSessaoDoComputador::de_horas(0).em_segundos(), 3_600, "menos de 1h vira 1h");
        assert_eq!(TetoDeSessaoDoComputador::de_horas(1).em_segundos(), 3_600);
        assert_eq!(TetoDeSessaoDoComputador::de_horas(12).em_segundos(), 12 * 3_600);
        assert_eq!(TetoDeSessaoDoComputador::de_horas(200).em_segundos(), 12 * 3_600, "nunca mais de 12h, por mais que peçam");
        assert_eq!(TetoDeSessaoDoComputador::default().em_segundos(), 12 * 3_600, "padrão é o teto máximo permitido, nunca 'sem teto'");
    }

    #[test]
    fn sessao_de_computador_expira_sozinha_no_teto_mas_celular_nunca_expira_por_tempo() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO); // o "computador" nesta simulação
        r.registrar_aparelho(CONTA, CLONE); // o "celular", nunca registrado como computador
        assert_eq!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::Aceita);
        assert_eq!(r.conectar(CONTA, CLONE, true, true, 1), Resultado::Aceita);
        r.registrar_conexao_de_computador(CONTA, APARELHO, 1_000, TetoDeSessaoDoComputador::de_horas(2));
        assert!(!r.sessao_de_computador_expirou(&CONTA, &APARELHO, 1_000 + 2 * 3_600 - 1), "ainda dentro do teto de 2h");
        assert!(r.sessao_de_computador_expirou(&CONTA, &APARELHO, 1_000 + 2 * 3_600), "no limite exato já expirou");
        assert!(!r.sessao_de_computador_expirou(&CONTA, &CLONE, 1_000 + 100 * 3_600), "celular nunca foi registrado com teto: nunca expira por tempo");
        let derrubados = r.expirar_computadores_vencidos(1_000 + 2 * 3_600);
        assert_eq!(derrubados, vec![(CONTA, APARELHO)], "só o computador vencido cai, ninguém mais");
        assert!(r.aparelhos_ativos_agora(&CONTA).contains(&CLONE), "o celular continua ativo");
        assert!(!r.aparelhos_ativos_agora(&CONTA).contains(&APARELHO), "o computador caiu por tempo");
        assert!(!r.sessao_de_computador_expirou(&CONTA, &APARELHO, 1_000 + 2 * 3_600), "já foi derrubado, não fica 'expirando' para sempre");
    }

    #[test]
    fn sessao_paralela_com_credencial_derruba_todas() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        let _ = r.conectar(CONTA, APARELHO, true, true, 1);
        assert!(matches!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::SessaoParalela, .. }));
        assert!(!r.tem_sessao_ativa(&CONTA), "a sessão legítima também cai");
    }

    #[test]
    fn clone_com_credencial_em_aparelho_nao_registrado_derruba_todas() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        let _ = r.conectar(CONTA, APARELHO, true, true, 1);
        assert!(matches!(r.conectar(CONTA, CLONE, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::AparelhoNaoRegistrado, .. }));
        assert!(!r.tem_sessao_ativa(&CONTA));
    }

    #[test]
    fn app_adulterado_com_credencial_derruba_todas() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        assert!(matches!(r.conectar(CONTA, APARELHO, true, false, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::AtestacaoFalhou, .. }));
    }

    #[test]
    fn aparelho_roubado_revogado_nao_entra_mais() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        r.revogar_aparelho(CONTA, APARELHO);
        assert!(matches!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::AparelhoNaoRegistrado, .. }));
    }

    #[test]
    fn celular_e_computador_vinculado_convivem() {
        const COMPUTADOR: IdAparelho = [4; 32];
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        r.registrar_aparelho(CONTA, COMPUTADOR);
        assert_eq!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::Aceita);
        assert_eq!(r.conectar(CONTA, COMPUTADOR, true, true, 1), Resultado::Aceita, "computador vinculado não é clone");
        // O mesmo computador conectado de novo em paralelo é clone: tudo cai.
        assert!(matches!(r.conectar(CONTA, COMPUTADOR, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::SessaoParalela, .. }));
        assert!(!r.tem_sessao_ativa(&CONTA));
        // Desconectar um não derruba o outro.
        assert_eq!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::Aceita);
        assert_eq!(r.conectar(CONTA, COMPUTADOR, true, true, 1), Resultado::Aceita);
        r.desconectar(CONTA, COMPUTADOR);
        assert!(r.tem_sessao_ativa(&CONTA));
        r.desconectar(CONTA, APARELHO);
        assert!(!r.tem_sessao_ativa(&CONTA));
    }

    #[test]
    fn vinculo_por_qr_recebido_e_recusado() {
        use crate::integridade::Nivel;
        assert_eq!(pode_vincular(OrigemDoVinculo::LinkOuCodigoRecebido, Nivel::Verde, true, true), Err(RecusaDeVinculo::OrigemNaoPermitida));
        assert_eq!(pode_vincular(OrigemDoVinculo::TelaVincularDoApp, Nivel::Amarelo, true, true), Err(RecusaDeVinculo::CelularNaoVerde));
        assert_eq!(pode_vincular(OrigemDoVinculo::TelaVincularDoApp, Nivel::Verde, false, true), Err(RecusaDeVinculo::SemBiometria));
        assert_eq!(pode_vincular(OrigemDoVinculo::TelaVincularDoApp, Nivel::Verde, true, false), Err(RecusaDeVinculo::PalavrasNaoConferidas));
        assert_eq!(pode_vincular(OrigemDoVinculo::TelaVincularDoApp, Nivel::Verde, true, true), Ok(()));
        let a = palavras_do_vinculo(b"celular", b"computador");
        assert_eq!(a, palavras_do_vinculo(b"computador", b"celular"), "iguais nos dois aparelhos");
        assert_ne!(a, palavras_do_vinculo(b"celular", b"computador-do-golpista"));
        assert_ne!(a, crate::convites::palavras_de_confirmacao(b"celular", b"computador"), "domínio próprio: não reaproveita palavras de convite");
    }

    #[test]
    fn painel_so_tem_contagens() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        let _ = r.conectar(CONTA, CLONE, true, true, 1);
        assert_eq!(r.contagens_anonimas.get(&Gatilho::AparelhoNaoRegistrado), Some(&1));
    }

    // --- revogar todos os aparelhos ao concluir recuperação (achado desta rodada) ---

    #[test]
    fn revogar_todos_os_aparelhos_tira_o_antigo_mesmo_sem_estar_conectado_no_momento() {
        const COMPUTADOR: IdAparelho = [4; 32];
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        r.registrar_aparelho(CONTA, COMPUTADOR);
        assert_eq!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::Aceita);
        r.desconectar(CONTA, APARELHO); // aparelho perdido: nem precisa estar "ativo" agora para ser revogado
        r.revogar_todos_os_aparelhos(CONTA);
        assert!(matches!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::AparelhoNaoRegistrado, .. }), "o antigo nunca mais entra, mesmo desconectado no instante da revogação");
        assert!(matches!(r.conectar(CONTA, COMPUTADOR, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::AparelhoNaoRegistrado, .. }), "TODOS os aparelhos antigos caem, não só o que estava ativo");
    }

    #[test]
    fn aparelho_novo_da_recuperacao_conecta_normalmente_depois_da_revogacao() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO); // aparelho antigo, perdido
        let _ = r.conectar(CONTA, APARELHO, true, true, 1);
        r.revogar_todos_os_aparelhos(CONTA); // passo que a ponte faz ao concluir a recuperação
        const APARELHO_NOVO: IdAparelho = [9; 32];
        r.registrar_aparelho(CONTA, APARELHO_NOVO); // a ponte registra o aparelho novo DEPOIS de revogar
        assert_eq!(r.conectar(CONTA, APARELHO_NOVO, true, true, 1), Resultado::Aceita, "o aparelho novo da recuperação conecta sem problema");
        assert_eq!(r.aparelhos_ativos_agora(&CONTA), vec![APARELHO_NOVO], "só o novo está ativo — o antigo já não existe mais no registro");
    }

    #[test]
    fn revogar_todos_os_aparelhos_tambem_limpa_o_teto_de_sessao_do_computador() {
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        let _ = r.conectar(CONTA, APARELHO, true, true, 1);
        r.registrar_conexao_de_computador(CONTA, APARELHO, 1_000, TetoDeSessaoDoComputador::de_horas(2));
        r.revogar_todos_os_aparelhos(CONTA);
        assert!(!r.sessao_de_computador_expirou(&CONTA, &APARELHO, 1_000 + 2 * 3_600), "não sobra estado de teto de computador para uma conta que já foi totalmente revogada");
        assert_eq!(r.expirar_computadores_vencidos(1_000 + 2 * 3_600), Vec::new(), "nada para expirar: já foi limpo por completo");
    }

    #[test]
    fn revogar_todos_os_aparelhos_nao_afeta_outra_conta() {
        const OUTRA_CONTA: IdConta = [5; 32];
        const APARELHO_DA_OUTRA: IdAparelho = [6; 32];
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        r.registrar_aparelho(OUTRA_CONTA, APARELHO_DA_OUTRA);
        let _ = r.conectar(OUTRA_CONTA, APARELHO_DA_OUTRA, true, true, 1);
        r.revogar_todos_os_aparelhos(CONTA);
        assert!(r.aparelhos_ativos_agora(&OUTRA_CONTA).contains(&APARELHO_DA_OUTRA), "a outra conta nunca foi tocada — isto não é uma varredura global");
        // Reconectar o MESMO aparelho, já ativo, continua dando SessaoParalela (não AparelhoNaoRegistrado):
        // prova que ele continua REGISTRADO, não que foi apagado por engano junto com a conta revogada.
        assert!(matches!(r.conectar(OUTRA_CONTA, APARELHO_DA_OUTRA, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::SessaoParalela, .. }));
    }

    // --- tempo de arrependimento ao vincular aparelho novo (Rodada 20, 11/10/2026) ---

    #[test]
    fn periodo_de_arrependimento_e_grampeado_entre_1_e_60_minutos() {
        assert_eq!(PeriodoDeArrependimento::de_minutos(0).em_segundos(), 60, "menos de 1 min vira 1 min");
        assert_eq!(PeriodoDeArrependimento::de_minutos(10).em_segundos(), 600);
        assert_eq!(PeriodoDeArrependimento::de_minutos(60).em_segundos(), 3_600);
        assert_eq!(PeriodoDeArrependimento::de_minutos(200).em_segundos(), 3_600, "nunca mais de 60 min, por mais que peçam");
        assert_eq!(PeriodoDeArrependimento::default().em_segundos(), 600, "padrão recomendado desta rodada: 10 minutos");
    }

    #[test]
    fn aparelho_recem_vinculado_comeca_dentro_do_periodo_de_arrependimento_e_depois_sai_sozinho() {
        let mut r = Registro::default();
        let janela = PeriodoDeArrependimento::de_minutos(10);
        r.registrar_vinculo(CONTA, APARELHO, 1_000);
        assert!(r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000, janela), "acabou de vincular");
        assert!(r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000 + 600 - 1, janela), "um segundo antes do fim da janela");
        assert!(!r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000 + 600, janela), "no instante exato do fim, já passou");
        assert!(!r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000 + 3_600, janela), "bem depois da janela");
    }

    #[test]
    fn aparelho_registrado_sem_data_de_vinculo_nunca_aparece_dentro_da_janela() {
        // registrar_aparelho "cru" (sem passar por registrar_vinculo) é o caminho do primeiro aparelho da
        // conta — não há "antes" para se arrepender, e não deve ser tratado como suspeito por não ter
        // data: a resposta certa é `false` (sem urgência extra), nunca um erro ou um `true` por padrão.
        let mut r = Registro::default();
        r.registrar_aparelho(CONTA, APARELHO);
        let janela = PeriodoDeArrependimento::de_minutos(10);
        assert!(!r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000, janela));
        assert!(!r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 0, janela), "mesmo em agora=0, sem data não é 'recém-vinculado'");
    }

    #[test]
    fn revogar_aparelho_funciona_dentro_e_fora_da_janela_de_arrependimento() {
        // O "desfazer" nunca precisou de nenhum mecanismo novo: revogar_aparelho já funciona a qualquer
        // momento. O que esta rodada acrescenta (saber quando destacar o botão na tela) não é uma
        // condição para o desfazer em si funcionar — provado aqui dentro E fora da janela, com dois
        // aparelhos independentes para não misturar o efeito de um revogar no estado do outro.
        const DEPOIS_DA_JANELA: IdAparelho = [9; 32];
        let mut r = Registro::default();
        let janela = PeriodoDeArrependimento::de_minutos(10);

        r.registrar_vinculo(CONTA, APARELHO, 1_000);
        assert_eq!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::Aceita, "o aparelho novo já funciona, sem esperar a janela passar");
        r.revogar_aparelho(CONTA, APARELHO); // ainda dentro da janela (agora=1_000, vinculado em 1_000)
        assert!(matches!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::AparelhoNaoRegistrado, .. }), "revogado dentro da janela: não entra mais");
        assert!(!r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000, janela), "revogado: não sobra data de vínculo para confundir uma futura reativação");

        r.registrar_vinculo(CONTA, DEPOIS_DA_JANELA, 2_000);
        assert!(!r.dentro_do_periodo_de_arrependimento(&CONTA, &DEPOIS_DA_JANELA, 2_000 + 3_600, janela), "bem fora da janela");
        r.revogar_aparelho(CONTA, DEPOIS_DA_JANELA); // fora da janela, continua funcionando
        assert!(matches!(r.conectar(CONTA, DEPOIS_DA_JANELA, true, true, 1), Resultado::TodasDerrubadas { gatilho: Gatilho::AparelhoNaoRegistrado, .. }), "revogado fora da janela: também não entra mais");
    }

    #[test]
    fn revogar_todos_os_aparelhos_tambem_limpa_as_datas_de_vinculo() {
        let mut r = Registro::default();
        r.registrar_vinculo(CONTA, APARELHO, 1_000);
        let janela = PeriodoDeArrependimento::de_minutos(10);
        assert!(r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000, janela));
        r.revogar_todos_os_aparelhos(CONTA);
        assert!(!r.dentro_do_periodo_de_arrependimento(&CONTA, &APARELHO, 1_000, janela), "conta totalmente revogada: nenhuma data de vínculo sobrevive");
    }

    #[test]
    fn registrar_vinculo_registra_o_aparelho_como_registrar_aparelho_tambem_faz() {
        // registrar_vinculo não é um caminho paralelo que esquece de registrar o aparelho de verdade —
        // é registrar_aparelho MAIS a data, nunca um substituto incompleto.
        let mut r = Registro::default();
        r.registrar_vinculo(CONTA, APARELHO, 1_000);
        assert_eq!(r.conectar(CONTA, APARELHO, true, true, 1), Resultado::Aceita);
    }
}
