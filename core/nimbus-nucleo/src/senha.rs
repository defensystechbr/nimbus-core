//! # Módulo `senha` — Argon2id para tudo o que depende de algo que uma pessoa digita (SEG5, ENV3, BA3)
//!
//! Senhas e PINs têm pouca aleatoriedade: se vazar um verificador ou um bloco protegido por senha, o
//! atacante tenta bilhões de combinações no próprio computador. O Argon2id (IETF RFC 9106) torna cada
//! tentativa cara em memória e tempo, o que inviabiliza placas de vídeo e chips dedicados.
//!
//! Parâmetros: segunda recomendação da RFC 9106 — 64 MiB de memória, 3 passagens, 4 faixas.
use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

/// Memória em KiB (64 MiB), passagens e faixas: RFC 9106, seção 4, segunda opção recomendada.
pub const MEMORIA_KIB: u32 = 64 * 1024;
pub const PASSAGENS: u32 = 3;
pub const FAIXAS: u32 = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeSenha { SalCurto, Falhou }

/// Deriva `N` bytes da senha com o sal dado (mínimo 16 bytes de sal).
pub fn derivar<const N: usize>(senha: &[u8], sal: &[u8]) -> Result<Zeroizing<[u8; N]>, ErroDeSenha> {
    if sal.len() < 16 { return Err(ErroDeSenha::SalCurto); }
    let params = Params::new(MEMORIA_KIB, PASSAGENS, FAIXAS, Some(N)).map_err(|_| ErroDeSenha::Falhou)?;
    let a = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut saida = Zeroizing::new([0u8; N]);
    a.hash_password_into(senha, sal, &mut saida[..]).map_err(|_| ErroDeSenha::Falhou)?;
    Ok(saida)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn mesma_senha_e_sal_dao_o_mesmo_resultado() {
        assert_eq!(*derivar::<32>(b"minha senha", &[1; 16]).unwrap(), *derivar::<32>(b"minha senha", &[1; 16]).unwrap());
    }

    #[test]
    fn sal_ou_senha_diferente_muda_tudo() {
        let a = derivar::<32>(b"minha senha", &[1; 16]).unwrap();
        assert_ne!(*a, *derivar::<32>(b"minha senha", &[2; 16]).unwrap());
        assert_ne!(*a, *derivar::<32>(b"minha senhA", &[1; 16]).unwrap());
    }

    #[test]
    fn sal_curto_e_recusado() {
        assert_eq!(derivar::<32>(b"x", &[1; 8]).unwrap_err(), ErroDeSenha::SalCurto);
    }

    #[test]
    fn parametros_sao_os_da_rfc_9106() {
        assert_eq!((MEMORIA_KIB, PASSAGENS, FAIXAS), (65_536, 3, 4));
    }
}
