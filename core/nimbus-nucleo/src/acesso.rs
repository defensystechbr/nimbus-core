//! # Módulo `acesso` — pedidos ao servidor que não se ligam entre si (ANO9–ANO14)
//!
//! Problema: mesmo sem ler nada, o servidor poderia perceber **quais blocos são pedidos juntos** e
//! deduzir que pertencem à mesma pessoa. Solução em camadas — o servidor recebe pedidos que não
//! consegue ligar uns aos outros nem ao dono:
//!
//! 1. **Identificadores derivados**: o endereço de cada bloco é calculado pelo próprio dono a partir de
//!    uma semente secreta (HKDF). Para o servidor, cada endereço é um número aleatório sem relação com
//!    os outros. Bônus: o aparelho não precisa guardar a lista do que existe — ele recalcula.
//! 2. **Lotes de tamanho fixo com cobertura**: cada ida ao servidor pede sempre o mesmo número de
//!    blocos, misturando os reais com endereços falsos, em ordem embaralhada.
//! 3. **Sem login no pedido**: cada lote é pago com uma ficha anônima de uso único (Privacy Pass),
//!    não com a identidade da conta.
//! 4. **Cada lote por um circuito Tor diferente** (isolamento de fluxo): o servidor não vê IP nem
//!    consegue agrupar pedidos pela conexão.
//!
//! Itens 3 e 4 são da camada de rede (Manual, ANO12–ANO14); este módulo entrega 1 e 2, testados.
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::Sha512;
use zeroize::Zeroizing;

pub type Endereco = [u8; 32];

/// Semente secreta do dono (no app real, protegida pelo chip do aparelho e sincronizada cifrada).
pub struct Semente(Zeroizing<[u8; 32]>);

impl core::fmt::Debug for Semente {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "Semente(<oculta>)") }
}

impl Semente {
    pub fn nova() -> Self { let mut s = [0u8; 32]; OsRng.fill_bytes(&mut s); Semente(Zeroizing::new(s)) }
    pub fn de_bytes(b: [u8; 32]) -> Self { Semente(Zeroizing::new(b)) }
}

/// Endereço do bloco `indice` no contexto (ex.: "historico", "agenda", "album").
pub fn endereco(semente: &Semente, contexto: &str, indice: u64) -> Endereco {
    let h = Hkdf::<Sha512>::new(Some(b"NimbusOne/v1/enderecos"), &semente.0[..]);
    let mut info = contexto.as_bytes().to_vec();
    info.push(0);
    info.extend_from_slice(&indice.to_be_bytes());
    let mut e = [0u8; 32];
    h.expand(&info, &mut e).expect("tamanho válido");
    e
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeLote { TamanhoInvalido }

/// Um lote: sempre `tamanho` endereços, reais e falsos misturados, em ordem aleatória.
#[derive(Debug)]
pub struct Lote { pub enderecos: Vec<Endereco> }

fn aleatorio_ate(n: usize) -> usize {
    // Amostragem sem viés (rejeição).
    let limite = u32::MAX - (u32::MAX % n as u32);
    loop { let x = OsRng.next_u32(); if x < limite { return (x % n as u32) as usize; } }
}

/// Divide os pedidos reais em lotes de tamanho fixo, completando com cobertura e embaralhando.
pub fn montar_lotes(reais: &[Endereco], tamanho: usize) -> Result<Vec<Lote>, ErroDeLote> {
    if tamanho < 2 { return Err(ErroDeLote::TamanhoInvalido); }
    let mut lotes = vec![];
    // Até metade de cada lote é real; o restante é sempre cobertura.
    let reais_por_lote = tamanho / 2;
    let grupos: Vec<&[Endereco]> = if reais.is_empty() { vec![&[]] } else { reais.chunks(reais_por_lote).collect() };
    for grupo in grupos {
        let mut v: Vec<Endereco> = grupo.to_vec();
        while v.len() < tamanho { let mut e = [0u8; 32]; OsRng.fill_bytes(&mut e); v.push(e); }
        for i in (1..v.len()).rev() { let j = aleatorio_ate(i + 1); v.swap(i, j); }
        lotes.push(Lote { enderecos: v });
    }
    Ok(lotes)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_dono_recalcula_o_mesmo_endereco() {
        let s = Semente::de_bytes([7; 32]);
        assert_eq!(endereco(&s, "agenda", 5), endereco(&s, "agenda", 5));
    }

    #[test]
    fn enderecos_nao_se_parecem_entre_si() {
        let s = Semente::de_bytes([7; 32]);
        let a = endereco(&s, "agenda", 5);
        let b = endereco(&s, "agenda", 6);
        let c = endereco(&s, "album", 5);
        assert_ne!(a, b); assert_ne!(a, c);
        let bits_diferentes: u32 = a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum();
        assert!(bits_diferentes > 80 && bits_diferentes < 176, "parecem independentes ({bits_diferentes} de 256 bits)");
    }

    #[test]
    fn outra_semente_gera_enderecos_sem_relacao() {
        assert_ne!(endereco(&Semente::de_bytes([1; 32]), "agenda", 0), endereco(&Semente::de_bytes([2; 32]), "agenda", 0));
    }

    #[test]
    fn todo_lote_tem_o_mesmo_tamanho_e_contem_os_reais() {
        let s = Semente::nova();
        let reais: Vec<Endereco> = (0..11).map(|i| endereco(&s, "historico", i)).collect();
        let lotes = montar_lotes(&reais, 16).unwrap();
        assert!(lotes.iter().all(|l| l.enderecos.len() == 16));
        for r in &reais { assert!(lotes.iter().any(|l| l.enderecos.contains(r))); }
    }

    #[test]
    fn sem_pedido_real_ainda_sai_um_lote_so_de_cobertura() {
        let lotes = montar_lotes(&[], 16).unwrap();
        assert_eq!(lotes.len(), 1);
        assert_eq!(lotes[0].enderecos.len(), 16);
    }

    #[test]
    fn posicao_dos_reais_e_embaralhada() {
        let s = Semente::nova();
        let real = endereco(&s, "historico", 0);
        let posicoes: Vec<usize> = (0..50).map(|_| {
            let l = &montar_lotes(&[real], 16).unwrap()[0];
            l.enderecos.iter().position(|e| *e == real).unwrap()
        }).collect();
        assert!(posicoes.iter().any(|p| *p != posicoes[0]), "a posição varia entre lotes");
    }

    #[test]
    fn lote_pequeno_demais_e_recusado() {
        assert_eq!(montar_lotes(&[], 1).unwrap_err(), ErroDeLote::TamanhoInvalido);
    }
}
