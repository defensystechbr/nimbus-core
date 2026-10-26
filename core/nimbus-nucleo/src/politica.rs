//! # Módulo `politica` — regras por conversa e por mensagem (CH2, CH3, AR2)
//!
//! Cada conversa tem uma **política padrão** e cada mensagem pode trazer uma **política própria**. A
//! política efetiva é sempre a **mais restritiva** das duas: uma mensagem nunca afrouxa a regra da conversa.
//!
//! Cobre: prazos (depois de enviada, depois de aberta, para ler), visualização única para todo tipo de
//! conteúdo, responder citando e Conversa Blindada.
//!
//! O que pode SAIR da conversa (guardar, encaminhar, levar para fora) não é decidido aqui: fica só no
//! módulo `liberacao`. Mensagem nunca sai; arquivo só com autorização assinada de quem enviou. Copiar para
//! a área de transferência, print e gravação de tela não existem no NimbusChat, em nenhum caso.
use crate::expiracao::Politica as PoliticaDeExpiracao;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoDeConteudo {
    Texto, Audio, Foto, Video, VideoCircular, Arquivo, Figurinha,
    /// Localização de um instante só (um endereço, um ponto no mapa) — pode virar cartão encaminhável,
    /// como um texto (`liberacao::pode_ser_cartao`).
    Localizacao,
    /// Localização AO VIVO (RV7): uma série de atualizações de posição dentro de uma
    /// `SessaoDeLocalizacaoAoVivo` com prazo definido. Deliberadamente um tipo DIFERENTE de `Localizacao`:
    /// nunca vira cartão encaminhável nem vai para a Minha Pasta (`liberacao::pode_ser_cartao` não inclui
    /// este tipo) — permitir reencaminhar um rastreamento ao vivo seria deixar outra pessoa acompanhar
    /// alguém sem que essa pessoa tenha autorizado isso, o que o cartão de localização pontual não causa.
    LocalizacaoAoVivo,
}

/// Prazo de visibilidade escolhido por quem envia (RV1–RV3): por quanto tempo o nome da conversa fica na
/// lista do destinatário, ou o registro fica na lista de ligações do Nimbus — nunca o conteúdo da
/// mensagem em si, que continua regido só por `expiracao` (teto de 48 h). Esta lista é só do Nimbus: nunca
/// se relaciona com a lista de chamadas nativa do aparelho (iOS ou Android) — ver `chamada`, nota de
/// isolamento.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrazoDeVisibilidade {
    UmaHora, DuasHoras, TresHoras, SeisHoras, DozeHoras, VinteQuatroHoras,
    SeteDias, QuinzeDias, TrintaDias, SessentaDias, TresMeses,
    /// "Tempo indeterminado": sem o remetente escolher um prazo menor que o pedido explicitamente. Mesmo
    /// assim, nunca passa de 3 meses — nenhuma opção desta lista supera o teto declarado ao usuário.
    Indeterminado,
}

/// 3 meses = 90 dias. Também o teto de "tempo indeterminado": nada nesta lista supera isto.
pub const TRES_MESES_EM_SEGUNDOS: u64 = 90 * 24 * 3_600;

impl PrazoDeVisibilidade {
    pub fn em_segundos(self) -> u64 {
        use PrazoDeVisibilidade::*;
        match self {
            UmaHora => 3_600,
            DuasHoras => 2 * 3_600,
            TresHoras => 3 * 3_600,
            SeisHoras => 6 * 3_600,
            DozeHoras => 12 * 3_600,
            VinteQuatroHoras => 24 * 3_600,
            SeteDias => 7 * 24 * 3_600,
            QuinzeDias => 15 * 24 * 3_600,
            TrintaDias => 30 * 24 * 3_600,
            SessentaDias => 60 * 24 * 3_600,
            TresMeses | Indeterminado => TRES_MESES_EM_SEGUNDOS,
        }
    }
}

#[cfg(test)]
mod testes_prazo_de_visibilidade {
    use super::PrazoDeVisibilidade::*;
    use super::TRES_MESES_EM_SEGUNDOS;

    #[test]
    fn opcoes_oferecidas_na_tela_batem_com_o_pedido() {
        let segundos: Vec<u64> = [UmaHora, DuasHoras, TresHoras, SeisHoras, DozeHoras, VinteQuatroHoras,
                                   SeteDias, QuinzeDias, TrintaDias, SessentaDias, TresMeses]
            .iter().map(|p| p.em_segundos()).collect();
        assert_eq!(segundos, vec![3_600, 7_200, 10_800, 21_600, 43_200, 86_400, 604_800, 1_296_000, 2_592_000, 5_184_000, 7_776_000]);
        assert!(segundos.windows(2).all(|w| w[0] < w[1]), "cada opção é estritamente maior que a anterior");
    }

    #[test]
    fn nada_passa_de_tres_meses_nem_mesmo_indeterminado() {
        for p in [UmaHora, DuasHoras, TresHoras, SeisHoras, DozeHoras, VinteQuatroHoras, SeteDias, QuinzeDias, TrintaDias, SessentaDias, TresMeses, Indeterminado] {
            assert!(p.em_segundos() <= TRES_MESES_EM_SEGUNDOS, "{p:?} passou de 3 meses");
        }
        assert_eq!(Indeterminado.em_segundos(), TresMeses.em_segundos(), "indeterminado é o mesmo teto que 3 meses, nunca mais");
    }
}

/// RV7 — localização ao vivo: duração ESCOLHIDA no início, nunca "para sempre". Mesmo a opção mais
/// longa (`UmDia`) tem teto: ao fim, a sessão termina sozinha e precisa ser reaberta por uma NOVA
/// autorização explícita de quem compartilha — nunca renova sozinha, nunca fica ligada por padrão.
/// Compartilhar a qualquer momento é opt-in; parar de compartilhar é sempre possível, sem esperar o prazo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DuracaoDeLocalizacaoAoVivo { QuinzeMinutos, UmaHora, OitoHoras, UmDia }

/// Teto absoluto de qualquer sessão de localização ao vivo: 24 horas. Não existe opção "indeterminado"
/// aqui (ao contrário de `PrazoDeVisibilidade`) — localização ao vivo sem prazo nenhum não é oferecida.
pub const TETO_DE_LOCALIZACAO_AO_VIVO_SEGUNDOS: u64 = 24 * 3_600;

impl DuracaoDeLocalizacaoAoVivo {
    pub fn em_segundos(self) -> u64 {
        use DuracaoDeLocalizacaoAoVivo::*;
        match self {
            QuinzeMinutos => 15 * 60,
            UmaHora => 3_600,
            OitoHoras => 8 * 3_600,
            UmDia => 24 * 3_600,
        }
    }
}

/// Uma sessão de localização ao vivo entre o relógio de início e a duração escolhida. As atualizações de
/// posição dentro da sessão são mensagens comuns (`TipoDeConteudo::LocalizacaoAoVivo`), seladas por
/// `envelope::selar` e enviadas na MESMA cadência fixa de `trafego::Batida` de qualquer outra mensagem —
/// nenhum canal novo, nenhum padrão novo observável no tráfego de rede (ver Contrato da Ponte, regra 27).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessaoDeLocalizacaoAoVivo {
    pub iniciada_em: u64,
    pub duracao: DuracaoDeLocalizacaoAoVivo,
}

impl SessaoDeLocalizacaoAoVivo {
    /// Começa agora, pela duração escolhida por quem compartilha.
    pub fn iniciar(agora: u64, duracao: DuracaoDeLocalizacaoAoVivo) -> Self {
        SessaoDeLocalizacaoAoVivo { iniciada_em: agora, duracao }
    }

    fn termina_em(&self) -> u64 { self.iniciada_em.saturating_add(self.duracao.em_segundos()) }

    /// Verdadeiro só dentro do prazo escolhido no início. Depois disso, a sessão acabou: não dá para
    /// reativar sem uma `iniciar()` nova (nova autorização explícita), mesmo que o `agora` volte a cair
    /// dentro de uma janela antiga por manipulação de relógio — quem decide isso é o relógio confiável do
    /// servidor (`expiracao::hora_confiavel`), igual às outras regras de prazo deste módulo.
    pub fn ativa(&self, agora: u64) -> bool { agora < self.termina_em() }

    /// Quanto falta, em segundos. Satura em 0 depois do fim (nunca "estoura" para um número gigante).
    pub fn segundos_restantes(&self, agora: u64) -> u64 { self.termina_em().saturating_sub(agora) }
}

/// Só é possível mandar uma atualização de posição enquanto a sessão está ativa. Parar manualmente
/// ("parar de compartilhar") é uma ação da UI que descarta a sessão inteira — está sempre disponível e
/// não depende desta função.
pub fn pode_enviar_atualizacao_de_localizacao(sessao: &SessaoDeLocalizacaoAoVivo, agora: u64) -> bool {
    sessao.ativa(agora)
}

#[cfg(test)]
mod testes_localizacao_ao_vivo {
    use super::*;

    #[test]
    fn nenhuma_duracao_passa_do_teto_de_um_dia() {
        use DuracaoDeLocalizacaoAoVivo::*;
        for d in [QuinzeMinutos, UmaHora, OitoHoras, UmDia] {
            assert!(d.em_segundos() <= TETO_DE_LOCALIZACAO_AO_VIVO_SEGUNDOS, "{d:?} passou do teto de 24h");
        }
        assert_eq!(UmDia.em_segundos(), TETO_DE_LOCALIZACAO_AO_VIVO_SEGUNDOS, "a opção mais longa é exatamente o teto, nunca mais");
    }

    #[test]
    fn opcoes_sao_estritamente_crescentes() {
        use DuracaoDeLocalizacaoAoVivo::*;
        let segundos: Vec<u64> = [QuinzeMinutos, UmaHora, OitoHoras, UmDia].iter().map(|d| d.em_segundos()).collect();
        assert!(segundos.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn sessao_fica_ativa_so_dentro_do_prazo_escolhido() {
        let s = SessaoDeLocalizacaoAoVivo::iniciar(1_000, DuracaoDeLocalizacaoAoVivo::UmaHora);
        assert!(s.ativa(1_000));
        assert!(s.ativa(1_000 + 3_599));
        assert!(!s.ativa(1_000 + 3_600), "no segundo exato do prazo, já acabou");
        assert!(!s.ativa(1_000 + 10_000), "muito depois, continua acabada");
    }

    #[test]
    fn segundos_restantes_nunca_fica_negativo() {
        let s = SessaoDeLocalizacaoAoVivo::iniciar(1_000, DuracaoDeLocalizacaoAoVivo::QuinzeMinutos);
        assert_eq!(s.segundos_restantes(1_000), 900);
        assert_eq!(s.segundos_restantes(1_000 + 900), 0);
        assert_eq!(s.segundos_restantes(1_000 + 999_999), 0, "satura em 0, nunca estoura");
    }

    #[test]
    fn atualizacao_so_pode_ser_enviada_com_a_sessao_ativa() {
        let s = SessaoDeLocalizacaoAoVivo::iniciar(0, DuracaoDeLocalizacaoAoVivo::QuinzeMinutos);
        assert!(pode_enviar_atualizacao_de_localizacao(&s, 500));
        assert!(!pode_enviar_atualizacao_de_localizacao(&s, 901), "prazo acabou, nenhuma atualização nova sai");
    }

    #[test]
    fn sessao_vencida_precisa_de_nova_autorizacao_nao_renova_sozinha() {
        // Uma sessão vencida simplesmente continua vencida para sempre a partir dali: não existe operação
        // de "renovar" — só `iniciar()` de novo, que é uma NOVA autorização explícita.
        let s = SessaoDeLocalizacaoAoVivo::iniciar(0, DuracaoDeLocalizacaoAoVivo::QuinzeMinutos);
        assert!(!s.ativa(1_000_000));
        let nova = SessaoDeLocalizacaoAoVivo::iniciar(1_000_000, DuracaoDeLocalizacaoAoVivo::QuinzeMinutos);
        assert!(nova.ativa(1_000_000));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Regras {
    pub expiracao: PoliticaDeExpiracao,
    /// Responder citando, dentro da mesma conversa. A citação é uma referência ao original: some quando
    /// ele some, nunca é uma cópia.
    pub permitir_citar: bool,
    /// Conversa Blindada (v37, terceira rodada: deixou de ser opcional — SEMPRE `true` a partir de
    /// `padrao_nimbus()`; não existe mais "conversa não blindada"). Uma consequência real, incondicional,
    /// em `liberacao::teto_da_conversa`: nenhum arquivo pode ser levado para fora do Nimbus, em nenhuma
    /// conversa — essa trava nem lê este campo, é sempre aplicada. **v37, quinta rodada:** este campo
    /// NÃO força mais Tor em `rede::decidir` — essa ligação foi desfeita por decisão do dono: a rota
    /// "Rápido (sem Tor)" já é, por desenho, uma rota de duplo salto que não entrega IP nem conteúdo a
    /// ninguém de fora (ver a doc de `rede`), então a escolha de rota (Tor ou não) volta a ser inteira do
    /// usuário via `rede::Preferencia`, sem depender de Blindada. Este campo passa a ser, na prática, só
    /// o rótulo "esta conversa é Blindada" combinado entre participantes (`efetiva` continua OR, então
    /// nada dentro deste núcleo consegue produzir `false` a partir de `padrao_nimbus()`) — a ponte usa
    /// isso só para mostrar o indicador visual, não para decidir rota.
    pub conversa_blindada: bool,
}

impl Regras {
    /// Padrão do NimbusChat: sem citação; prazos e visualização única vêm das regras da conversa;
    /// Conversa Blindada sempre ligada (v37, terceira rodada — ver o campo acima).
    pub fn padrao_nimbus() -> Self {
        Regras {
            expiracao: PoliticaDeExpiracao { segundos_apos_abrir: None, segundos_apos_enviar: None, visualizacao_unica: false, segundos_para_ler: None },
            permitir_citar: false,
            conversa_blindada: true,
        }
    }
}

fn menor(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) { (Some(x), Some(y)) => Some(x.min(y)), (x, None) => x, (None, y) => y }
}

/// Combina a regra da conversa com a da mensagem: vence sempre a mais restritiva.
pub fn efetiva(conversa: &Regras, mensagem: Option<&Regras>) -> Regras {
    let Some(m) = mensagem else { return *conversa };
    Regras {
        expiracao: PoliticaDeExpiracao {
            segundos_apos_abrir: menor(conversa.expiracao.segundos_apos_abrir, m.expiracao.segundos_apos_abrir),
            segundos_apos_enviar: menor(conversa.expiracao.segundos_apos_enviar, m.expiracao.segundos_apos_enviar),
            visualizacao_unica: conversa.expiracao.visualizacao_unica || m.expiracao.visualizacao_unica,
            segundos_para_ler: menor(conversa.expiracao.segundos_para_ler, m.expiracao.segundos_para_ler),
        },
        permitir_citar: conversa.permitir_citar && m.permitir_citar,
        conversa_blindada: conversa.conversa_blindada || m.conversa_blindada,
    }
}

/// Ações que acontecem DENTRO da conversa. Tudo o que sai dela é do módulo `liberacao`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acao { Citar, AbrirNovamente }

/// O app consulta antes de cada ação. `ja_aberta` indica se a mensagem já foi vista.
pub fn pode(r: &Regras, acao: Acao, ja_aberta: bool) -> bool {
    let unica = r.expiracao.visualizacao_unica;
    match acao {
        // Visualização única nunca é citada.
        Acao::Citar => r.permitir_citar && !unica,
        Acao::AbrirNovamente => !(unica && ja_aberta),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn com_prazo(abrir: Option<u64>, enviar: Option<u64>, unica: bool) -> Regras {
        Regras { expiracao: PoliticaDeExpiracao { segundos_apos_abrir: abrir, segundos_apos_enviar: enviar, visualizacao_unica: unica, segundos_para_ler: None }, ..Regras::padrao_nimbus() }
    }

    #[test]
    fn sem_regra_propria_vale_a_da_conversa() {
        let c = com_prazo(None, Some(3_600), false);
        assert_eq!(efetiva(&c, None), c);
    }

    #[test]
    fn mensagem_pode_encurtar_o_prazo_da_conversa() {
        let e = efetiva(&com_prazo(Some(600), Some(3_600), false), Some(&com_prazo(Some(60), Some(300), false)));
        assert_eq!((e.expiracao.segundos_apos_abrir, e.expiracao.segundos_apos_enviar), (Some(60), Some(300)));
    }

    #[test]
    fn mensagem_nunca_alonga_o_prazo_da_conversa() {
        let e = efetiva(&com_prazo(Some(60), Some(300), false), Some(&com_prazo(Some(600), None, false)));
        assert_eq!((e.expiracao.segundos_apos_abrir, e.expiracao.segundos_apos_enviar), (Some(60), Some(300)));
    }

    #[test]
    fn visualizacao_unica_da_conversa_vale_mesmo_se_a_mensagem_nao_pedir() {
        assert!(efetiva(&com_prazo(None, None, true), Some(&com_prazo(None, None, false))).expiracao.visualizacao_unica);
        assert!(efetiva(&com_prazo(None, None, false), Some(&com_prazo(None, None, true))).expiracao.visualizacao_unica);
    }

    #[test]
    fn prazo_para_ler_da_mensagem_so_encurta() {
        let mut c = Regras::padrao_nimbus();
        c.expiracao.segundos_para_ler = Some(24 * 3_600);
        let mut m = Regras::padrao_nimbus();
        m.expiracao.segundos_para_ler = Some(2 * 3_600);
        assert_eq!(efetiva(&c, Some(&m)).expiracao.segundos_para_ler, Some(2 * 3_600));
        m.expiracao.segundos_para_ler = Some(40 * 3_600);
        assert_eq!(efetiva(&c, Some(&m)).expiracao.segundos_para_ler, Some(24 * 3_600), "a mensagem não alonga o prazo da conversa");
    }

    #[test]
    fn citar_vem_desligado_e_a_mensagem_nao_libera() {
        assert!(!pode(&Regras::padrao_nimbus(), Acao::Citar, false), "padrão: sem citação");
        let conversa_sem = Regras::padrao_nimbus();
        let mensagem_com = Regras { permitir_citar: true, ..Regras::padrao_nimbus() };
        assert!(!pode(&efetiva(&conversa_sem, Some(&mensagem_com)), Acao::Citar, false));
        let conversa_com = Regras { permitir_citar: true, ..Regras::padrao_nimbus() };
        assert!(pode(&efetiva(&conversa_com, Some(&mensagem_com)), Acao::Citar, false));
    }

    #[test]
    fn visualizacao_unica_nunca_e_citada_nem_reaberta() {
        let r = Regras { permitir_citar: true, ..com_prazo(None, None, true) };
        assert!(!pode(&r, Acao::Citar, false));
        assert!(pode(&r, Acao::AbrirNovamente, false), "abre uma vez");
        assert!(!pode(&r, Acao::AbrirNovamente, true), "não abre de novo");
        assert!(pode(&com_prazo(None, None, false), Acao::AbrirNovamente, true), "mensagem comum reabre");
    }

    #[test]
    fn conversa_blindada_prevalece() {
        let c = Regras { conversa_blindada: true, ..Regras::padrao_nimbus() };
        assert!(efetiva(&c, Some(&Regras::padrao_nimbus())).conversa_blindada);
        assert!(efetiva(&Regras::padrao_nimbus(), Some(&c)).conversa_blindada);
    }
}
