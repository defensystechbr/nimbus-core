//! # Módulo `modos` — seu padrão de envio e a escolha na hora (MOD1–MOD8)
//!
//! Três camadas, simples de entender:
//! 1. **Regras da conversa** (combinadas entre os participantes, módulo `conversa`): o teto que ninguém passa.
//! 2. **Seu padrão de envio**, por tipo de conteúdo — texto, foto e vídeo, voz, arquivo —, aplicado sozinho.
//! 3. **A escolha na hora**: o botão acima do teclado já vem com o seu padrão; se quiser, você muda só para
//!    esta mensagem, antes de enviar.
//!
//! A mensagem sai com a combinação mais restritiva entre a conversa e a sua escolha. Nenhuma passa de 48 horas.
use crate::expiracao::VIDA_MAXIMA;
use crate::politica::{self, Regras, TipoDeConteudo};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    /// Some 48 horas depois de enviada (o máximo).
    Normal,
    /// Some 1 hora depois de aberta; se não for lida, some em 24 horas.
    Discreto,
    /// Abre uma vez e some.
    VisualizacaoUnica,
    /// Some este número de segundos depois de aberta (valores de `expiracao::TEMPORIZADORES`).
    Temporizador(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Categoria { Texto, FotoEVideo, Voz, Arquivo }

pub fn categoria(t: TipoDeConteudo) -> Categoria {
    match t {
        TipoDeConteudo::Texto | TipoDeConteudo::Figurinha | TipoDeConteudo::Localizacao | TipoDeConteudo::LocalizacaoAoVivo => Categoria::Texto,
        TipoDeConteudo::Foto | TipoDeConteudo::Video | TipoDeConteudo::VideoCircular => Categoria::FotoEVideo,
        TipoDeConteudo::Audio => Categoria::Voz,
        TipoDeConteudo::Arquivo => Categoria::Arquivo,
    }
}

/// Seu padrão de envio, um modo por categoria.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Padroes { pub texto: Modo, pub foto_e_video: Modo, pub voz: Modo, pub arquivo: Modo }

impl Padroes {
    /// Padrão de fábrica: foto e vídeo em visualização única (o que mais vaza por print), voz discreta (a voz
    /// identifica a pessoa), texto e arquivo normais.
    pub fn recomendados() -> Self {
        Padroes { texto: Modo::Normal, foto_e_video: Modo::VisualizacaoUnica, voz: Modo::Discreto, arquivo: Modo::Normal }
    }
    pub fn para(&self, t: TipoDeConteudo) -> Modo {
        match categoria(t) {
            Categoria::Texto => self.texto,
            Categoria::FotoEVideo => self.foto_e_video,
            Categoria::Voz => self.voz,
            Categoria::Arquivo => self.arquivo,
        }
    }
}

/// Regra de expiração de cada modo.
pub fn regra_do_modo(m: Modo) -> Regras {
    let mut r = Regras::padrao_nimbus();
    match m {
        Modo::Normal => r.expiracao.segundos_apos_enviar = Some(VIDA_MAXIMA),
        Modo::Discreto => { r.expiracao.segundos_apos_abrir = Some(3_600); r.expiracao.segundos_para_ler = Some(24 * 3_600); }
        Modo::VisualizacaoUnica => r.expiracao.visualizacao_unica = true,
        Modo::Temporizador(s) => r.expiracao.segundos_apos_abrir = Some(s.clamp(1, VIDA_MAXIMA)),
    }
    r
}

/// A regra com que a mensagem sai: a conversa combinada com a escolha na hora (ou com o seu padrão).
pub fn regra_da_mensagem(conversa: &Regras, padroes: &Padroes, tipo: TipoDeConteudo, escolha_na_hora: Option<Modo>) -> Regras {
    let modo = escolha_na_hora.unwrap_or_else(|| padroes.para(tipo));
    politica::efetiva(conversa, Some(&regra_do_modo(modo)))
}

/// Texto do botão acima do teclado.
pub fn rotulo(m: Modo) -> String {
    match m {
        Modo::Normal => "Normal · 48 h".into(),
        Modo::Discreto => "Discreto · 1 h".into(),
        Modo::VisualizacaoUnica => "Visualização única".into(),
        Modo::Temporizador(s) if s < 3_600 => format!("Some em {} min", s.div_ceil(60)),
        Modo::Temporizador(s) => format!("Some em {} h", s.div_ceil(3_600)),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use TipoDeConteudo::*;

    fn conversa() -> Regras { Regras::padrao_nimbus() }

    #[test]
    fn padrao_de_fabrica_por_tipo() {
        let p = Padroes::recomendados();
        assert_eq!((p.para(Texto), p.para(Foto), p.para(Video), p.para(Audio), p.para(Arquivo)),
                   (Modo::Normal, Modo::VisualizacaoUnica, Modo::VisualizacaoUnica, Modo::Discreto, Modo::Normal));
        assert_eq!(p.para(Localizacao), Modo::Normal);
    }

    #[test]
    fn sem_escolha_vale_o_seu_padrao() {
        let r = regra_da_mensagem(&conversa(), &Padroes::recomendados(), Foto, None);
        assert!(r.expiracao.visualizacao_unica, "foto sai em visualização única sem você fazer nada");
    }

    #[test]
    fn escolha_na_hora_vale_so_para_esta_mensagem() {
        let p = Padroes::recomendados();
        let r = regra_da_mensagem(&conversa(), &p, Foto, Some(Modo::Temporizador(300)));
        assert!(!r.expiracao.visualizacao_unica);
        assert_eq!(r.expiracao.segundos_apos_abrir, Some(300));
        assert!(regra_da_mensagem(&conversa(), &p, Foto, None).expiracao.visualizacao_unica, "a próxima volta ao padrão");
    }

    #[test]
    fn a_conversa_continua_sendo_o_teto() {
        let mut c = conversa();
        c.expiracao.segundos_apos_abrir = Some(600);
        let r = regra_da_mensagem(&c, &Padroes::recomendados(), Texto, Some(Modo::Temporizador(3_600)));
        assert_eq!(r.expiracao.segundos_apos_abrir, Some(600), "a conversa pede 10 min: a mensagem não passa disso");
        let mut unica = conversa();
        unica.expiracao.visualizacao_unica = true;
        assert!(regra_da_mensagem(&unica, &Padroes::recomendados(), Texto, Some(Modo::Normal)).expiracao.visualizacao_unica);
    }

    #[test]
    fn modos_nunca_passam_de_48_horas() {
        assert_eq!(regra_do_modo(Modo::Normal).expiracao.segundos_apos_enviar, Some(VIDA_MAXIMA));
        assert_eq!(regra_do_modo(Modo::Temporizador(30 * 86_400)).expiracao.segundos_apos_abrir, Some(VIDA_MAXIMA));
        let d = regra_do_modo(Modo::Discreto).expiracao;
        assert_eq!((d.segundos_apos_abrir, d.segundos_para_ler), (Some(3_600), Some(24 * 3_600)));
    }

    #[test]
    fn rotulos_do_botao() {
        assert_eq!(rotulo(Modo::Normal), "Normal · 48 h");
        assert_eq!(rotulo(Modo::Temporizador(300)), "Some em 5 min");
        assert_eq!(rotulo(Modo::Temporizador(7_200)), "Some em 2 h");
        assert_eq!(rotulo(Modo::VisualizacaoUnica), "Visualização única");
    }
}
