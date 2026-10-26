//! # Módulo `chamada` — sinalização de chamada com atendimento humano obrigatório (CH1–CH5)
//!
//! Fecha a porta do ataque de "chamada forçada a atender" (o CVE-2019-17191 do Signal, em que uma mensagem
//! de conexão forjada fazia o áudio ligar sem o usuário tocar em nada). Aqui, o áudio SÓ passa a fluir
//! depois que o dono do aparelho aceita explicitamente. Nenhuma mensagem vinda da rede consegue pular esse
//! passo: a máquina de estados só sai de `Tocando` para `Ativa` pela ação local `aceitar()`.
//!
//! Este módulo é a regra que a ponte nativa (Swift/Kotlin) tem de obedecer: o microfone só é ligado quando
//! o estado é `Ativa`, e o estado só chega a `Ativa` por toque do usuário.

/// Estado de uma chamada, do ponto de vista do aparelho que recebe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Estado {
    /// Convite recebido; tocando. O microfone está DESLIGADO.
    Tocando,
    /// O dono aceitou (ação local). Só agora o áudio flui.
    Ativa,
    /// Recusada, não atendida a tempo, ou encerrada. Microfone desligado.
    Encerrada(Fim),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fim { Recusada, NaoAtendida, EncerradaPorMim, EncerradaPeloOutro }

/// O que pode acontecer com uma chamada. `Aceitar` é a única ação que vem do usuário local.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evento {
    /// Ação LOCAL do dono: tocar em "atender".
    Aceitar,
    /// Ação LOCAL do dono: tocar em "recusar".
    Recusar,
    /// Passou o tempo de toque sem o dono atender.
    Expirou,
    /// A outra ponta desligou.
    OutroDesligou,
    /// Ação LOCAL do dono: encerrar uma chamada ativa.
    EuDesligo,
    /// QUALQUER mensagem vinda da rede pedindo para conectar/ativar. Nunca liga o áudio sozinha.
    MensagemDaRede,
}

/// Tempo máximo de toque antes de virar "não atendida" (segundos).
pub const SEGUNDOS_DE_TOQUE: u64 = 45;

#[derive(Debug, PartialEq, Eq)]
pub struct Chamada { estado: Estado }

impl Chamada {
    /// Uma chamada recebida começa tocando, com o microfone desligado.
    pub fn recebida() -> Self { Chamada { estado: Estado::Tocando } }

    pub fn estado(&self) -> Estado { self.estado }

    /// O microfone só pode estar ligado quando a chamada está Ativa. A ponte nativa consulta isto.
    pub fn microfone_ligado(&self) -> bool { self.estado == Estado::Ativa }

    /// Aplica um evento e devolve o novo estado. A regra central: nada vindo da rede leva a `Ativa`.
    pub fn aplicar(&mut self, evento: Evento) -> Estado {
        self.estado = match (self.estado, evento) {
            // Só a ação local Aceitar, e só enquanto toca, ativa a chamada.
            (Estado::Tocando, Evento::Aceitar) => Estado::Ativa,
            (Estado::Tocando, Evento::Recusar) => Estado::Encerrada(Fim::Recusada),
            (Estado::Tocando, Evento::Expirou) => Estado::Encerrada(Fim::NaoAtendida),
            (Estado::Tocando, Evento::OutroDesligou) => Estado::Encerrada(Fim::EncerradaPeloOutro),
            // Uma mensagem da rede enquanto toca NÃO atende: a chamada continua apenas tocando.
            (Estado::Tocando, Evento::MensagemDaRede) => Estado::Tocando,
            // Já ativa: só encerra.
            (Estado::Ativa, Evento::EuDesligo) => Estado::Encerrada(Fim::EncerradaPorMim),
            (Estado::Ativa, Evento::OutroDesligou) => Estado::Encerrada(Fim::EncerradaPeloOutro),
            // Uma mensagem da rede jamais reativa nem altera uma chamada ativa ou encerrada.
            (estado, _) => estado,
        };
        self.estado
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn nenhuma_mensagem_da_rede_atende_a_chamada() {
        let mut c = Chamada::recebida();
        assert_eq!(c.estado(), Estado::Tocando);
        assert!(!c.microfone_ligado());
        // Por mais mensagens da rede que cheguem, o microfone continua desligado.
        for _ in 0..1000 {
            c.aplicar(Evento::MensagemDaRede);
            assert_eq!(c.estado(), Estado::Tocando);
            assert!(!c.microfone_ligado(), "nenhuma mensagem da rede pode ligar o microfone");
        }
    }

    #[test]
    fn so_a_acao_local_de_aceitar_liga_o_microfone() {
        let mut c = Chamada::recebida();
        assert_eq!(c.aplicar(Evento::Aceitar), Estado::Ativa);
        assert!(c.microfone_ligado(), "depois do toque humano em atender, o áudio flui");
    }

    #[test]
    fn recusar_e_expirar_encerram_sem_audio() {
        let mut c = Chamada::recebida();
        assert_eq!(c.aplicar(Evento::Recusar), Estado::Encerrada(Fim::Recusada));
        assert!(!c.microfone_ligado());
        let mut d = Chamada::recebida();
        assert_eq!(d.aplicar(Evento::Expirou), Estado::Encerrada(Fim::NaoAtendida));
        assert!(!d.microfone_ligado());
    }

    #[test]
    fn mensagem_da_rede_nao_reativa_chamada_encerrada() {
        let mut c = Chamada::recebida();
        c.aplicar(Evento::Recusar);
        c.aplicar(Evento::MensagemDaRede);
        c.aplicar(Evento::Aceitar); // nem um Aceitar tardio reabre uma chamada já encerrada
        assert_eq!(c.estado(), Estado::Encerrada(Fim::Recusada));
        assert!(!c.microfone_ligado());
    }

    #[test]
    fn chamada_ativa_so_encerra_nao_volta_a_tocar() {
        let mut c = Chamada::recebida();
        c.aplicar(Evento::Aceitar);
        c.aplicar(Evento::MensagemDaRede); // não muda nada
        assert_eq!(c.estado(), Estado::Ativa);
        assert_eq!(c.aplicar(Evento::EuDesligo), Estado::Encerrada(Fim::EncerradaPorMim));
    }
}
