//! # Módulo `trafego` — blindagem contra análise de tráfego (ARQ4, ANO1–ANO3)
//!
//! Cifrar esconde o conteúdo, mas **quando** e **quanto** alguém envia ainda conta uma história:
//! quem observa a rede deduz com quem se fala, a que horas e o tamanho do que foi enviado.
//! Duas defesas, calculadas no aparelho:
//!
//! 1. **Tamanho padronizado** — todo envio vira uma faixa fixa (o `envelope` já preenche).
//! 2. **Batida constante** — o app envia um pacote a cada intervalo fixo; se não houver mensagem real,
//!    envia um pacote de enchimento, indistinguível para quem observa a rede. Assim, silêncio e
//!    conversa parecem iguais de fora.
//!
//! O servidor descarta os pacotes de enchimento sem registrar nada.

/// Intervalo padrão da batida, em milissegundos.
pub const INTERVALO_MS: u64 = 500;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Pacote { Real, Enchimento }

/// Fila de envio com batida constante.
#[derive(Debug)]
pub struct Batida { intervalo_ms: u64, proximo_envio_ms: u64, pendentes: usize }

impl Batida {
    pub fn nova(inicio_ms: u64) -> Self {
        Batida { intervalo_ms: INTERVALO_MS, proximo_envio_ms: inicio_ms, pendentes: 0 }
    }

    pub fn com_intervalo(inicio_ms: u64, intervalo_ms: u64) -> Self {
        Batida { intervalo_ms: intervalo_ms.max(1), proximo_envio_ms: inicio_ms, pendentes: 0 }
    }

    /// O app enfileira uma mensagem real; ela só sai na próxima batida.
    pub fn enfileirar(&mut self) { self.pendentes += 1; }

    pub fn pendentes(&self) -> usize { self.pendentes }

    /// Chamado pelo laço de rede. Devolve o pacote a enviar, ou `None` se ainda não é hora.
    pub fn na_hora(&mut self, agora_ms: u64) -> Option<Pacote> {
        if agora_ms < self.proximo_envio_ms { return None; }
        self.proximo_envio_ms += self.intervalo_ms;
        if self.pendentes > 0 { self.pendentes -= 1; Some(Pacote::Real) } else { Some(Pacote::Enchimento) }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn silencio_tambem_envia_pacote() {
        let mut b = Batida::nova(0);
        assert_eq!(b.na_hora(0), Some(Pacote::Enchimento));
        assert_eq!(b.na_hora(INTERVALO_MS), Some(Pacote::Enchimento));
    }

    #[test]
    fn mensagem_real_so_sai_na_batida() {
        let mut b = Batida::nova(0);
        let _ = b.na_hora(0);
        b.enfileirar();
        assert_eq!(b.na_hora(1), None, "não antecipa o envio");
        assert_eq!(b.na_hora(INTERVALO_MS), Some(Pacote::Real));
    }

    #[test]
    fn ritmo_e_identico_com_ou_sem_conversa() {
        let (mut calado, mut falante) = (Batida::nova(0), Batida::nova(0));
        for _ in 0..5 { falante.enfileirar(); }
        let mut t = 0;
        for _ in 0..5 {
            assert!(calado.na_hora(t).is_some());
            assert!(falante.na_hora(t).is_some());
            t += INTERVALO_MS;
        }
    }

    #[test]
    fn rajada_e_espalhada_pelas_batidas() {
        let mut b = Batida::nova(0);
        for _ in 0..3 { b.enfileirar(); }
        let mut t = 0;
        for _ in 0..3 { assert_eq!(b.na_hora(t), Some(Pacote::Real)); t += INTERVALO_MS; }
        assert_eq!(b.na_hora(t), Some(Pacote::Enchimento));
        assert_eq!(b.pendentes(), 0);
    }

    #[test]
    fn intervalo_nunca_e_zero() {
        let mut b = Batida::com_intervalo(0, 0);
        let _ = b.na_hora(0);
        assert_eq!(b.na_hora(0), None, "sem intervalo zero, não há laço infinito");
    }
}
