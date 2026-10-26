//! # Módulo `chaves_do_servidor` — a chave de assinatura do servidor muda todo dia (ROT1–ROT4)
//!
//! O servidor assina a hora (`expiracao::emitir_hora`) e outras respostas. Se essa chave vazasse e valesse
//! para sempre, o vazamento valeria para sempre. Aqui:
//! * cada dia tem a sua chave de assinatura, gerada e guardada em módulo de hardware;
//! * cada chave do dia vem com um **certificado do dia**, assinado por pelo menos **2 de 3 chaves-raiz**
//!   guardadas offline por pessoas diferentes (cerimônia mensal que certifica os dias do mês seguinte);
//! * o app só aceita a chave do dia corrente (com 10 minutos de folga na virada); a de ontem já não vale.
//!
//! Uma chave do dia roubada perde o valor em no máximo um dia; as raízes nunca tocam a internet.

use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};

const CONTEXTO: &[u8] = b"NimbusOne/v1/certificado-do-dia";
/// Segundos por dia.
pub const DIA: u64 = 24 * 3600;
/// Folga na virada do dia, para relógios levemente diferentes.
pub const FOLGA: u64 = 600;
/// Mínimo de chaves-raiz diferentes que precisam assinar cada certificado.
pub const MINIMO_DE_RAIZES: usize = 2;

#[derive(Debug, Clone)]
pub struct CertificadoDoDia { pub dia: u64, pub chave_do_dia: Vec<u8>, pub assinaturas: Vec<([u8; 32], AssinaturaHibrida)> }

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDoCertificado { ForaDoDia, RaizesInsuficientes { validas: usize }, ChaveMalformada }

fn mensagem(dia: u64, chave_do_dia: &[u8]) -> Vec<u8> {
    let mut m = dia.to_be_bytes().to_vec();
    m.extend_from_slice(chave_do_dia);
    m
}

/// Cerimônia offline: uma chave-raiz assina o certificado de um dia.
pub fn certificar(raiz: &ChaveDeAssinatura, dia: u64, chave_do_dia: &ChaveDeVerificacao) -> ([u8; 32], AssinaturaHibrida) {
    let a = raiz.assinar(CONTEXTO, &mensagem(dia, &chave_do_dia.para_bytes())).expect("contexto fixo e curto");
    (raiz.verificacao().impressao_digital(), a)
}

/// No app: aceita a chave do dia só se o certificado é de hoje (com folga) e tem 2 de 3 raízes fixadas no app.
/// `agora` vem de `expiracao::hora_confiavel`.
pub fn conferir(c: &CertificadoDoDia, raizes_fixadas: &[ChaveDeVerificacao], agora: u64) -> Result<ChaveDeVerificacao, RecusaDoCertificado> {
    let inicio = c.dia.saturating_mul(DIA);
    if agora.saturating_add(FOLGA) < inicio || agora >= inicio.saturating_add(DIA).saturating_add(FOLGA) {
        return Err(RecusaDoCertificado::ForaDoDia);
    }
    let m = mensagem(c.dia, &c.chave_do_dia);
    let mut validas: Vec<[u8; 32]> = Vec::new();
    for (fp, a) in &c.assinaturas {
        if validas.contains(fp) { continue; }
        if raizes_fixadas.iter().any(|r| &r.impressao_digital() == fp && r.verificar(CONTEXTO, &m, a)) { validas.push(*fp); }
    }
    if validas.len() < MINIMO_DE_RAIZES { return Err(RecusaDoCertificado::RaizesInsuficientes { validas: validas.len() }); }
    ChaveDeVerificacao::de_bytes(&c.chave_do_dia).map_err(|_| RecusaDoCertificado::ChaveMalformada)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    #[test]
    fn chave_do_dia_so_vale_no_seu_dia_e_com_duas_raizes() {
        let (r1, r2, r3) = (assinatura::gerar(), assinatura::gerar(), assinatura::gerar());
        let fix = [r1.verificacao(), r2.verificacao(), r3.verificacao()];
        let hoje = assinatura::gerar();
        let c = CertificadoDoDia { dia: 100, chave_do_dia: hoje.verificacao().para_bytes(),
                                   assinaturas: vec![certificar(&r1, 100, &hoje.verificacao()), certificar(&r3, 100, &hoje.verificacao())] };
        let meio_dia = 100 * DIA + DIA / 2;
        assert_eq!(conferir(&c, &fix, meio_dia).map(|k| k.impressao_digital()), Ok(hoje.verificacao().impressao_digital()));
        assert!(conferir(&c, &fix, 100 * DIA - FOLGA).is_ok(), "folga antes da virada");
        assert!(conferir(&c, &fix, 101 * DIA + FOLGA - 1).is_ok(), "folga depois da virada");
        assert_eq!(conferir(&c, &fix, 101 * DIA + FOLGA).unwrap_err(), RecusaDoCertificado::ForaDoDia, "a chave de ontem já não vale");
        assert_eq!(conferir(&c, &fix, 100 * DIA - FOLGA - 1).unwrap_err(), RecusaDoCertificado::ForaDoDia);
    }

    #[test]
    fn uma_raiz_so_ou_raiz_estranha_nao_certifica() {
        let (r1, r2, r3, intrusa) = (assinatura::gerar(), assinatura::gerar(), assinatura::gerar(), assinatura::gerar());
        let fix = [r1.verificacao(), r2.verificacao(), r3.verificacao()];
        let k = assinatura::gerar();
        let base = |ass: Vec<([u8; 32], AssinaturaHibrida)>| CertificadoDoDia { dia: 7, chave_do_dia: k.verificacao().para_bytes(), assinaturas: ass };
        let agora = 7 * DIA + 10;
        assert_eq!(conferir(&base(vec![certificar(&r1, 7, &k.verificacao())]), &fix, agora).unwrap_err(), RecusaDoCertificado::RaizesInsuficientes { validas: 1 });
        let a = certificar(&r1, 7, &k.verificacao());
        assert_eq!(conferir(&base(vec![a.clone(), a]), &fix, agora).unwrap_err(), RecusaDoCertificado::RaizesInsuficientes { validas: 1 }, "mesma raiz duas vezes");
        assert_eq!(conferir(&base(vec![certificar(&r1, 7, &k.verificacao()), certificar(&intrusa, 7, &k.verificacao())]), &fix, agora).unwrap_err(), RecusaDoCertificado::RaizesInsuficientes { validas: 1 });
        // Certificado de outro dia reaproveitado: a assinatura cobre o dia.
        let mut c = base(vec![certificar(&r1, 6, &k.verificacao()), certificar(&r2, 6, &k.verificacao())]);
        c.dia = 7;
        assert_eq!(conferir(&c, &fix, agora).unwrap_err(), RecusaDoCertificado::RaizesInsuficientes { validas: 0 });
        // Chave trocada mantendo as assinaturas.
        let mut c = base(vec![certificar(&r1, 7, &k.verificacao()), certificar(&r2, 7, &k.verificacao())]);
        c.chave_do_dia = assinatura::gerar().verificacao().para_bytes();
        assert_eq!(conferir(&c, &fix, agora).unwrap_err(), RecusaDoCertificado::RaizesInsuficientes { validas: 0 });
    }
}
