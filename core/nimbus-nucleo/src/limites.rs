//! # Módulo `limites` — limite de tentativas e de uso no servidor (SRV7–SRV9) — lógica do servidor
//!
//! O Nimbus não tem login com senha: não há senha para adivinhar. Mesmo assim, toda rota que alguém poderia
//! martelar ganha limite — resgate de convite, retirada de caixa, publicação de chave, registro de aparelho:
//! * **Cota por janela:** no máximo `maximo` pedidos por `janela` segundos, por chave de limite.
//! * **Castigo que dobra:** cada falha (prova inválida, convite inexistente) dobra a espera, até 24 h.
//! * **Memória limitada:** o limitador nunca cresce sem fim; cheio, recusa chaves novas (falha fechada).
//!
//! A chave de limite nunca é a pessoa: é o titular da prova (uma caixa), a ficha anônima ou o retransmissor
//! de onde o pedido chegou. O servidor não vê o IP do aparelho.

use std::collections::HashMap;

/// Espera máxima de castigo: 24 horas.
pub const CASTIGO_MAXIMO: u64 = 24 * 3600;

#[derive(Debug, PartialEq, Eq)]
pub enum Limite { EsperarAte(u64), Cheio }

#[derive(Debug, Clone, Copy)]
struct Balde { inicio_da_janela: u64, usados: u32, falhas: u32, bloqueado_ate: u64 }

#[derive(Debug)]
pub struct Limitador { maximo: u32, janela: u64, castigo_inicial: u64, capacidade: usize, baldes: HashMap<Vec<u8>, Balde> }

impl Limitador {
    /// `maximo` pedidos por `janela` segundos; primeira falha castiga `castigo_inicial` segundos; no máximo
    /// `capacidade` chaves acompanhadas ao mesmo tempo.
    pub fn novo(maximo: u32, janela: u64, castigo_inicial: u64, capacidade: usize) -> Self {
        Limitador { maximo: maximo.max(1), janela: janela.max(1), castigo_inicial: castigo_inicial.max(1), capacidade: capacidade.max(1), baldes: HashMap::new() }
    }

    fn faxina(&mut self, agora: u64) {
        let janela = self.janela;
        self.baldes.retain(|_, b| agora < b.bloqueado_ate || agora < b.inicio_da_janela.saturating_add(janela) || b.falhas > 0 && agora < b.bloqueado_ate.saturating_add(CASTIGO_MAXIMO));
    }

    /// Pode atender agora? Conta o pedido se puder.
    pub fn permitir(&mut self, chave: &[u8], agora: u64) -> Result<(), Limite> {
        if !self.baldes.contains_key(chave) {
            if self.baldes.len() >= self.capacidade { self.faxina(agora); }
            if self.baldes.len() >= self.capacidade { return Err(Limite::Cheio); }
            let _ = self.baldes.insert(chave.to_vec(), Balde { inicio_da_janela: agora, usados: 0, falhas: 0, bloqueado_ate: 0 });
        }
        let janela = self.janela;
        let maximo = self.maximo;
        let b = self.baldes.get_mut(chave).ok_or(Limite::Cheio)?;
        if agora < b.bloqueado_ate { return Err(Limite::EsperarAte(b.bloqueado_ate)); }
        if agora >= b.inicio_da_janela.saturating_add(janela) { b.inicio_da_janela = agora; b.usados = 0; }
        if b.usados >= maximo { return Err(Limite::EsperarAte(b.inicio_da_janela.saturating_add(janela))); }
        b.usados += 1;
        Ok(())
    }

    /// Falhou (prova inválida, convite inexistente…): a espera dobra a cada falha seguida.
    pub fn registrar_falha(&mut self, chave: &[u8], agora: u64) -> u64 {
        if !self.baldes.contains_key(chave) {
            if self.baldes.len() >= self.capacidade { self.faxina(agora); }
            // Cheio: não acompanha a chave nova (a memória nunca passa do teto); `permitir` já recusa chave nova.
            if self.baldes.len() >= self.capacidade { return agora; }
        }
        let inicial = self.castigo_inicial;
        let b = self.baldes.entry(chave.to_vec()).or_insert(Balde { inicio_da_janela: agora, usados: 0, falhas: 0, bloqueado_ate: 0 });
        b.falhas = b.falhas.saturating_add(1);
        let passos = (b.falhas - 1).min(20);
        let espera = inicial.saturating_mul(1u64 << passos).min(CASTIGO_MAXIMO);
        b.bloqueado_ate = agora.saturating_add(espera);
        b.bloqueado_ate
    }

    /// Acertou: zera as falhas (a cota da janela continua valendo).
    pub fn registrar_sucesso(&mut self, chave: &[u8]) {
        if let Some(b) = self.baldes.get_mut(chave) { b.falhas = 0; b.bloqueado_ate = 0; }
    }

    pub fn acompanhadas(&self) -> usize { self.baldes.len() }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn cota_por_janela() {
        let mut l = Limitador::novo(3, 60, 10, 100);
        for _ in 0..3 { assert!(l.permitir(b"caixa-a", 1_000).is_ok()); }
        assert_eq!(l.permitir(b"caixa-a", 1_010), Err(Limite::EsperarAte(1_060)));
        assert!(l.permitir(b"caixa-b", 1_010).is_ok(), "outra chave não é afetada");
        assert!(l.permitir(b"caixa-a", 1_060).is_ok(), "janela nova");
    }

    #[test]
    fn castigo_dobra_ate_o_teto() {
        let mut l = Limitador::novo(100, 60, 10, 100);
        assert_eq!(l.registrar_falha(b"r", 1_000), 1_010);
        assert_eq!(l.registrar_falha(b"r", 1_010), 1_030);
        assert_eq!(l.registrar_falha(b"r", 1_030), 1_070);
        assert_eq!(l.permitir(b"r", 1_069), Err(Limite::EsperarAte(1_070)));
        for _ in 0..40 { let _ = l.registrar_falha(b"r", 2_000); }
        assert_eq!(l.registrar_falha(b"r", 2_000), 2_000 + CASTIGO_MAXIMO, "nunca passa de 24 h");
        l.registrar_sucesso(b"r");
        assert!(l.permitir(b"r", 2_001).is_ok());
    }

    #[test]
    fn memoria_limitada_e_falha_fechada() {
        let mut l = Limitador::novo(1, 60, 10, 2);
        assert!(l.permitir(b"a", 1_000).is_ok());
        assert!(l.permitir(b"b", 1_000).is_ok());
        assert_eq!(l.permitir(b"c", 1_000), Err(Limite::Cheio), "cheio: recusa chave nova");
        assert!(l.permitir(b"c", 1_061).is_ok(), "passada a janela, a faxina libera espaço");
        assert!(l.acompanhadas() <= 2);
        for i in 0..1_000u32 { let _ = l.registrar_falha(&i.to_be_bytes(), 1_061); }
        assert!(l.acompanhadas() <= 2, "falhas com chaves inventadas não enchem a memória");
    }
}
