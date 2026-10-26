//! # Módulo `rede` — Tor quando importa, velocidade no resto (NET1–NET14)
//!
//! Três escolhas para o usuário, num só controle:
//!
//! * **Automático (recomendado, padrão)** — o dia a dia vai pela rota rápida em duas etapas; o app liga
//!   o Tor sozinho nos **momentos que revelam identidade e relações** (criar ou recuperar conta,
//!   adicionar contato, cadastrar aparelho, aprovar, publicar chaves), e quando a rede **bloqueia** a rota
//!   rápida (aí usa Tor com ponte, que parece tráfego comum).
//! * **Sempre Tor (máximo)** — tudo pelo Tor, exceto chamadas; mais lento, avisado ao usuário.
//! * **Rápido (sem Tor)** — nunca usa Tor. Existe porque, em alguns países, usar Tor chama atenção ou é
//!   bloqueado; a rota em duas etapas parece uma conexão comum com um grande provedor.
//!
//! **v37, quinta rodada — Conversa Blindada deixou de forçar Tor.** Antes desta rodada, toda conversa
//! marcada como Blindada forçava Tor no modo Automático, mesmo fora dos momentos de identidade. Decisão
//! do dono: a rota "Rápido (sem Tor)" (`Rota::DuasEtapas`) já é, por desenho, uma rota de duplo salto —
//! Oblivious HTTP (RFC 9458) por retransmissor de terceiro + ECH (RFC 9849) + DNS cifrado — em que o
//! destino nunca vê o IP real de quem envia, e nenhum único operador vê IP de origem e conteúdo ao mesmo
//! tempo (`sortear_primeira_etapa` nunca deixa um operador fixo). Ela não é idêntica ao Tor (o conjunto de
//! anonimato é dois operadores vetados, não milhares de nós voluntários globais — ressalva honesta, não
//! escondida), mas não entrega IP nem conteúdo a ninguém de fora, que era a condição posta pelo dono para
//! não exigir Tor. Por isso: Conversa Blindada (`politica::Regras.conversa_blindada`) continua **sempre
//! ligada**, sem opção de desligar — isso governa só o teto de exportação de arquivo (`liberacao`), que é
//! incondicional e não depende deste módulo. A ROTA de rede (Tor vs. duas etapas) volta a ser inteiramente
//! a escolha do usuário via `Preferencia`, nos três modos — nenhum deles abre mão de IP ou conteúdo para
//! o destino ou para a rede, só muda o tamanho do conjunto de anonimato.
//!
//! Regras que nunca mudam:
//! * Chamadas sempre por retransmissor independente (o Tor não transporta tempo real).
//! * Nunca rebaixar em silêncio: se o Tor for exigido e falhar, tenta ponte; se falhar de novo, avisa.
//! * Subir para o Tor é permitido (mais proteção), exceto se o usuário escolheu "Rápido (sem Tor)".
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::Sha512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preferencia { Automatico, SempreTor, Rapido }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operacao {
    Mensagem, Arquivo, Notificacao, Chamada,
    CriacaoDeConta, RecuperacaoDeConta, NovoContato, NovoAparelho, Aprovacao, PublicacaoDeChaves,
}

impl Operacao {
    /// Momentos que revelam quem é a pessoa ou com quem ela se relaciona.
    pub fn e_momento_de_identidade(self) -> bool {
        matches!(self, Operacao::CriacaoDeConta | Operacao::RecuperacaoDeConta | Operacao::NovoContato
            | Operacao::NovoAparelho | Operacao::Aprovacao | Operacao::PublicacaoDeChaves)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EstadoDaRede { Normal, RotaRapidaBloqueada, TorBloqueado }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rota {
    /// Oblivious HTTP (RFC 9458) por retransmissor de terceiro + ECH (RFC 9849) + DNS cifrado.
    DuasEtapas,
    /// Tor com circuito isolado por lote.
    Tor,
    /// Tor por ponte disfarçada (quando a rede bloqueia o Tor comum).
    TorComPonte,
    /// Retransmissor de mídia de operador independente (chamadas).
    RetransmissorIndependente,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aviso {
    /// Usando Tor: pode ficar mais lento.
    MaisLento,
    /// Momento de identidade sem Tor, por escolha do usuário.
    SemTorPorEscolhaDoUsuario,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decisao { pub rota: Rota, pub aviso: Option<Aviso> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Falha { PerguntarAoUsuario }

fn tor_conforme(rede: EstadoDaRede) -> Rota {
    if rede == EstadoDaRede::TorBloqueado { Rota::TorComPonte } else { Rota::Tor }
}

/// Decide a rota de uma operação. A partir da quinta rodada, Conversa Blindada não entra mais aqui —
/// ela não força Tor (ver o porquê na doc do módulo); a rota é só `pref` + o momento + o estado da rede.
pub fn decidir(pref: Preferencia, op: Operacao, rede: EstadoDaRede) -> Result<Decisao, Falha> {
    if op == Operacao::Chamada {
        return Ok(Decisao { rota: Rota::RetransmissorIndependente, aviso: None });
    }
    match pref {
        Preferencia::SempreTor => Ok(Decisao { rota: tor_conforme(rede), aviso: Some(Aviso::MaisLento) }),
        Preferencia::Rapido => {
            if rede == EstadoDaRede::RotaRapidaBloqueada { return Err(Falha::PerguntarAoUsuario); }
            let aviso = if op.e_momento_de_identidade() { Some(Aviso::SemTorPorEscolhaDoUsuario) } else { None };
            Ok(Decisao { rota: Rota::DuasEtapas, aviso })
        }
        Preferencia::Automatico => {
            let precisa_tor = op.e_momento_de_identidade() || rede == EstadoDaRede::RotaRapidaBloqueada;
            if precisa_tor {
                let rota = if rede == EstadoDaRede::RotaRapidaBloqueada { Rota::TorComPonte } else { tor_conforme(rede) };
                Ok(Decisao { rota, aviso: Some(Aviso::MaisLento) })
            } else {
                Ok(Decisao { rota: Rota::DuasEtapas, aviso: None })
            }
        }
    }
}

/// Se a rota falhar: Tor falhou → tenta ponte; ponte falhou → pergunta. Rota rápida falhou → no
/// Automático sobe para Tor com ponte; no Rápido pergunta. Nunca desce de Tor para a rota rápida sozinho.
pub fn ao_falhar(pref: Preferencia, rota_que_falhou: Rota) -> Result<Decisao, Falha> {
    match (pref, rota_que_falhou) {
        (_, Rota::Tor) => Ok(Decisao { rota: Rota::TorComPonte, aviso: Some(Aviso::MaisLento) }),
        (_, Rota::TorComPonte) => Err(Falha::PerguntarAoUsuario),
        (Preferencia::Automatico, Rota::DuasEtapas) => Ok(Decisao { rota: Rota::TorComPonte, aviso: Some(Aviso::MaisLento) }),
        (_, Rota::DuasEtapas) => Err(Falha::PerguntarAoUsuario),
        (_, Rota::RetransmissorIndependente) => Ok(Decisao { rota: Rota::RetransmissorIndependente, aviso: None }),
    }
}

/// Sorteia, por sessão, qual operador independente faz a 1ª etapa da rota rápida.
/// Vários operadores, de empresas e países diferentes, tornam um conluio muito mais difícil.
/// Sem nenhum operador configurado, devolve `None` (a rota rápida não pode ser usada).
pub fn sortear_primeira_etapa(quantidade_de_operadores: usize) -> Option<usize> {
    if quantidade_de_operadores == 0 || quantidade_de_operadores > u32::MAX as usize { return None; }
    let n = quantidade_de_operadores as u32;
    let limite = u32::MAX - (u32::MAX % n);
    loop { let x = OsRng.next_u32(); if x < limite { return Some((x % n) as usize); } }
}

/// Cabeçalhos iguais para todos os usuários: sem idioma, fuso, modelo do aparelho ou versão do sistema.
pub fn cabecalhos_uniformes() -> Vec<(&'static str, &'static str)> {
    vec![("user-agent", "NimbusOne"), ("accept", "application/octet-stream"), ("accept-encoding", "identity")]
}

/// Identificador de Bluetooth da transferência presencial: derivado do segredo do QR e da janela de tempo.
pub fn identificador_bluetooth(segredo_do_qr: &[u8; 32], janela: u64) -> [u8; 16] {
    let h = Hkdf::<Sha512>::new(Some(b"NimbusOne/v1/ble"), segredo_do_qr);
    let mut id = [0u8; 16];
    h.expand(&janela.to_be_bytes(), &mut id).expect("tamanho válido");
    id
}

#[cfg(test)]
mod testes {
    use super::*;
    use EstadoDaRede::*;
    use Operacao::*;
    use Preferencia::*;

    #[test]
    fn automatico_dia_a_dia_e_rapido() {
        for op in [Mensagem, Arquivo, Notificacao] {
            assert_eq!(decidir(Automatico, op, Normal).unwrap(), Decisao { rota: Rota::DuasEtapas, aviso: None });
        }
    }

    #[test]
    fn automatico_liga_tor_nos_momentos_de_identidade() {
        for op in [CriacaoDeConta, RecuperacaoDeConta, NovoContato, NovoAparelho, Aprovacao, PublicacaoDeChaves] {
            assert_eq!(decidir(Automatico, op, Normal).unwrap().rota, Rota::Tor, "{op:?}");
        }
    }

    #[test]
    fn automatico_nao_forca_tor_so_pela_conversa_blindada() {
        // v37, quinta rodada: Conversa Blindada não entra mais na assinatura de `decidir` — ela não
        // governa mais a rota de rede (ver a doc do módulo). Fora dos momentos de identidade, uma
        // mensagem comum no modo Automático continua pela rota rápida de duas etapas.
        assert_eq!(decidir(Automatico, Mensagem, Normal).unwrap().rota, Rota::DuasEtapas);
    }

    #[test]
    fn automatico_usa_ponte_se_a_rede_bloquear() {
        assert_eq!(decidir(Automatico, Mensagem, RotaRapidaBloqueada).unwrap().rota, Rota::TorComPonte);
        assert_eq!(decidir(Automatico, NovoContato, TorBloqueado).unwrap().rota, Rota::TorComPonte);
    }

    #[test]
    fn sempre_tor_usa_tor_em_tudo_menos_chamada() {
        for op in [Mensagem, Arquivo, NovoContato] { assert_eq!(decidir(SempreTor, op, Normal).unwrap().rota, Rota::Tor); }
        assert_eq!(decidir(SempreTor, Chamada, Normal).unwrap().rota, Rota::RetransmissorIndependente);
    }

    #[test]
    fn rapido_nunca_usa_tor_e_avisa_nos_momentos_de_identidade() {
        let d = decidir(Rapido, NovoContato, Normal).unwrap();
        assert_eq!(d, Decisao { rota: Rota::DuasEtapas, aviso: Some(Aviso::SemTorPorEscolhaDoUsuario) });
        assert_eq!(decidir(Rapido, Mensagem, Normal).unwrap().rota, Rota::DuasEtapas, "Rápido nunca usa Tor, por escolha do usuário");
        assert_eq!(decidir(Rapido, Mensagem, RotaRapidaBloqueada), Err(Falha::PerguntarAoUsuario));
    }

    #[test]
    fn chamadas_nunca_pelo_tor() {
        for p in [Automatico, SempreTor, Rapido] {
            assert_eq!(decidir(p, Chamada, Normal).unwrap().rota, Rota::RetransmissorIndependente);
        }
    }

    #[test]
    fn nunca_desce_do_tor_para_a_rota_rapida_sozinho() {
        for p in [Automatico, SempreTor, Rapido] {
            assert_eq!(ao_falhar(p, Rota::Tor).unwrap().rota, Rota::TorComPonte);
            assert_eq!(ao_falhar(p, Rota::TorComPonte), Err(Falha::PerguntarAoUsuario));
        }
    }

    #[test]
    fn rota_rapida_falhou_automatico_sobe_rapido_pergunta() {
        assert_eq!(ao_falhar(Automatico, Rota::DuasEtapas).unwrap().rota, Rota::TorComPonte);
        assert_eq!(ao_falhar(Rapido, Rota::DuasEtapas), Err(Falha::PerguntarAoUsuario));
    }

    #[test]
    fn sorteio_de_operador_varia_e_fica_no_intervalo() {
        assert_eq!(sortear_primeira_etapa(0), None);
        let v: Vec<usize> = (0..200).map(|_| sortear_primeira_etapa(3).unwrap()).collect();
        assert!(v.iter().all(|x| *x < 3));
        for i in 0..3 { assert!(v.contains(&i), "todos os operadores são sorteados"); }
    }

    #[test]
    fn cabecalhos_nao_revelam_idioma_fuso_nem_aparelho() {
        let c = cabecalhos_uniformes();
        let nomes: Vec<&str> = c.iter().map(|(n, _)| *n).collect();
        for proibido in ["accept-language", "time-zone", "x-device", "x-os-version"] { assert!(!nomes.contains(&proibido)); }
    }

    #[test]
    fn bluetooth_muda_a_cada_janela_e_bate_nos_dois_aparelhos() {
        let s = [9u8; 32];
        assert_eq!(identificador_bluetooth(&s, 7), identificador_bluetooth(&s, 7));
        assert_ne!(identificador_bluetooth(&s, 7), identificador_bluetooth(&s, 8));
    }
}
