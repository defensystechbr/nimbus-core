//! # Módulo `assinatura` — assinatura híbrida pós-quântica (PQS1–PQS6)
//!
//! Toda assinatura do NimbusOne é **dupla**: Ed25519 (clássica, madura) **e** ML-DSA-87 (pós-quântica,
//! NIST FIPS 204, nível 5 — a exigida pela suíte CNSA 2.0 da NSA para sistemas de segurança nacional).
//! Para aceitar, as **duas** precisam conferir. Um atacante teria de quebrar as duas ao mesmo tempo:
//! se um computador quântico quebrar a clássica, a pós-quântica segura; se aparecer uma falha na
//! pós-quântica (mais nova), a clássica segura.
//!
//! Cada uso tem um **contexto** próprio (separação de domínio): uma assinatura feita para aprovar um
//! pagamento nunca vale como assinatura de prazo de mensagem, e vice-versa.
use ed25519_dalek::{Signature as AssinaturaEd, Signer, SigningKey as ChaveEd, Verifier, VerifyingKey as VerificacaoEd};
use ml_dsa::{EncodedSignature, EncodedVerifyingKey, Keypair, MlDsa87, Signature as AssinaturaPq, SigningKey as ChavePq, VerifyingKey as VerificacaoPq};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha512};
use zeroize::Zeroizing;

/// Tamanhos fixos (FIPS 204, ML-DSA-87).
pub const TAM_VERIFICACAO_PQ: usize = 2592;
pub const TAM_ASSINATURA_PQ: usize = 4627;

/// Chave de assinatura (fica no aparelho; no app real, protegida pelo chip de segurança).
pub struct ChaveDeAssinatura { ed: ChaveEd, pq: ChavePq<MlDsa87> }

impl core::fmt::Debug for ChaveDeAssinatura {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ChaveDeAssinatura(Ed25519 + ML-DSA-87, <oculta>)") }
}

/// Chave de verificação (pública).
#[derive(Clone)]
pub struct ChaveDeVerificacao { ed: VerificacaoEd, pq: VerificacaoPq<MlDsa87> }

impl core::fmt::Debug for ChaveDeVerificacao {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ChaveDeVerificacao({:02x?}…)", &self.impressao_digital()[..4]) }
}

impl PartialEq for ChaveDeVerificacao {
    fn eq(&self, outro: &Self) -> bool { self.para_bytes() == outro.para_bytes() }
}
impl Eq for ChaveDeVerificacao {}

/// Assinatura dupla.
#[derive(Clone)]
pub struct AssinaturaHibrida { ed: AssinaturaEd, pq: Vec<u8> }

impl core::fmt::Debug for AssinaturaHibrida {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "AssinaturaHibrida({} bytes)", 64 + self.pq.len()) }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeAssinatura { ContextoGrandeDemais, ChaveMalformada }

pub fn gerar() -> ChaveDeAssinatura {
    let mut semente = Zeroizing::new([0u8; 32]);
    OsRng.fill_bytes(&mut semente[..]);
    let pq = ChavePq::<MlDsa87>::from_seed(&(*semente).into());
    ChaveDeAssinatura { ed: ChaveEd::generate(&mut OsRng), pq }
}

/// Recria a chave a partir de 64 bytes de semente (32 do Ed25519 + 32 do ML-DSA-87). Só para os assinantes
/// do selo de liberação, cuja semente vive em cofre offline (nunca em repositório, servidor ou variável de ambiente).
pub fn de_semente(semente: &[u8; 64]) -> ChaveDeAssinatura {
    let mut ed = Zeroizing::new([0u8; 32]);
    ed.copy_from_slice(&semente[..32]);
    let mut pq = Zeroizing::new([0u8; 32]);
    pq.copy_from_slice(&semente[32..]);
    ChaveDeAssinatura { ed: ChaveEd::from_bytes(&ed), pq: ChavePq::<MlDsa87>::from_seed(&(*pq).into()) }
}

impl ChaveDeAssinatura {
    pub fn verificacao(&self) -> ChaveDeVerificacao {
        ChaveDeVerificacao { ed: self.ed.verifying_key(), pq: self.pq.verifying_key() }
    }

    /// Assina `mensagem` no `contexto` dado (no máximo 255 bytes).
    pub fn assinar(&self, contexto: &[u8], mensagem: &[u8]) -> Result<AssinaturaHibrida, ErroDeAssinatura> {
        if contexto.len() > 255 { return Err(ErroDeAssinatura::ContextoGrandeDemais); }
        let ed = self.ed.sign(&mensagem_ed(contexto, mensagem));
        // Variante "hedged" da FIPS 204 (Algoritmo 2 com 32 bytes aleatórios), que a norma recomenda como
        // padrão: resiste melhor a injeção de falha e a canais laterais do que a variante determinística.
        // M' = 0 ‖ |ctx| ‖ ctx ‖ M, exatamente como a verificação com contexto reconstrói.
        let mut rnd = ml_dsa::B32::default();
        OsRng.fill_bytes(&mut rnd);
        let prefixo = [0u8, contexto.len() as u8];
        let pq = self.pq.expanded_key().sign_internal(&[&prefixo, contexto, mensagem], &rnd);
        Ok(AssinaturaHibrida { ed, pq: pq.encode().to_vec() })
    }
}

/// A parte clássica assina contexto + mensagem, com o tamanho do contexto: sem ambiguidade entre usos.
fn mensagem_ed(contexto: &[u8], mensagem: &[u8]) -> Vec<u8> {
    let mut v = b"NimbusOne/v1/assinatura-hibrida".to_vec();
    v.push(contexto.len() as u8);
    v.extend_from_slice(contexto);
    v.extend_from_slice(mensagem);
    v
}

impl ChaveDeVerificacao {
    /// Confere as DUAS assinaturas. Basta uma falhar para recusar.
    pub fn verificar(&self, contexto: &[u8], mensagem: &[u8], a: &AssinaturaHibrida) -> bool {
        if contexto.len() > 255 { return false; }
        let ok_ed = self.ed.verify(&mensagem_ed(contexto, mensagem), &a.ed).is_ok();
        let ok_pq = match EncodedSignature::<MlDsa87>::try_from(a.pq.as_slice()).ok().and_then(|e| AssinaturaPq::<MlDsa87>::decode(&e)) {
            Some(s) => self.pq.verify_with_context(mensagem, contexto, &s),
            None => false,
        };
        ok_ed && ok_pq
    }

    /// 32 bytes Ed25519 + 2592 bytes ML-DSA-87.
    pub fn para_bytes(&self) -> Vec<u8> {
        let mut v = self.ed.to_bytes().to_vec();
        v.extend_from_slice(self.pq.encode().as_slice());
        v
    }

    pub fn de_bytes(b: &[u8]) -> Result<Self, ErroDeAssinatura> {
        if b.len() != 32 + TAM_VERIFICACAO_PQ { return Err(ErroDeAssinatura::ChaveMalformada); }
        let ed_bytes: [u8; 32] = b[..32].try_into().map_err(|_| ErroDeAssinatura::ChaveMalformada)?;
        let ed = VerificacaoEd::from_bytes(&ed_bytes).map_err(|_| ErroDeAssinatura::ChaveMalformada)?;
        let enc = EncodedVerifyingKey::<MlDsa87>::try_from(&b[32..]).map_err(|_| ErroDeAssinatura::ChaveMalformada)?;
        Ok(ChaveDeVerificacao { ed, pq: VerificacaoPq::<MlDsa87>::decode(&enc) })
    }

    /// Identificador curto e estável da chave (SHA-512 truncado): usado em listas de autorizados.
    pub fn impressao_digital(&self) -> [u8; 32] {
        let h = Sha512::digest(self.para_bytes());
        let mut id = [0u8; 32];
        id.copy_from_slice(&h[..32]);
        id
    }
}

impl AssinaturaHibrida {
    /// Para testes de adulteração e para serializar: (clássica, pós-quântica).
    pub fn partes(&self) -> ([u8; 64], &[u8]) { (self.ed.to_bytes(), &self.pq) }
    pub fn de_partes(ed: [u8; 64], pq: Vec<u8>) -> Self { AssinaturaHibrida { ed: AssinaturaEd::from_bytes(&ed), pq } }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn assina_e_confere() {
        let k = gerar();
        let a = k.assinar(b"pagamento", b"ordem 881").unwrap();
        assert!(k.verificacao().verificar(b"pagamento", b"ordem 881", &a));
    }

    #[test]
    fn mensagem_ou_contexto_diferente_recusa() {
        let k = gerar();
        let a = k.assinar(b"pagamento", b"ordem 881").unwrap();
        assert!(!k.verificacao().verificar(b"pagamento", b"ordem 882", &a));
        assert!(!k.verificacao().verificar(b"prazo", b"ordem 881", &a), "assinatura de um uso não vale em outro");
    }

    #[test]
    fn quebrar_so_a_classica_nao_basta() {
        let k = gerar();
        let intruso = gerar();
        let boa = k.assinar(b"c", b"m").unwrap();
        let falsa = intruso.assinar(b"c", b"m").unwrap();
        // Pós-quântica verdadeira + clássica falsificada: recusada.
        let (ed_falsa, _) = falsa.partes();
        let mista = AssinaturaHibrida::de_partes(ed_falsa, boa.partes().1.to_vec());
        assert!(!k.verificacao().verificar(b"c", b"m", &mista));
    }

    #[test]
    fn quebrar_so_a_pos_quantica_nao_basta() {
        let k = gerar();
        let intruso = gerar();
        let boa = k.assinar(b"c", b"m").unwrap();
        let falsa = intruso.assinar(b"c", b"m").unwrap();
        let mista = AssinaturaHibrida::de_partes(boa.partes().0, falsa.partes().1.to_vec());
        assert!(!k.verificacao().verificar(b"c", b"m", &mista));
    }

    #[test]
    fn partes_de_contextos_diferentes_nao_se_misturam() {
        let k = gerar();
        // Contextos do mesmo tamanho: a diferença está só no conteúdo do contexto.
        let a = k.assinar(b"AAAA", b"m").unwrap();
        let b = k.assinar(b"BBBB", b"m").unwrap();
        let mista = AssinaturaHibrida::de_partes(a.partes().0, b.partes().1.to_vec());
        assert!(!k.verificacao().verificar(b"BBBB", b"m", &mista), "a parte clássica também é amarrada ao contexto");
    }

    #[test]
    fn assinatura_pos_quantica_corrompida_e_recusada() {
        let k = gerar();
        let boa = k.assinar(b"c", b"m").unwrap();
        let mut pq = boa.partes().1.to_vec();
        pq[100] ^= 1;
        assert!(!k.verificacao().verificar(b"c", b"m", &AssinaturaHibrida::de_partes(boa.partes().0, pq.clone())));
        pq.truncate(10);
        assert!(!k.verificacao().verificar(b"c", b"m", &AssinaturaHibrida::de_partes(boa.partes().0, pq)));
    }

    #[test]
    fn chave_publica_serializa_e_tem_tamanhos_do_padrao() {
        let k = gerar();
        let v = k.verificacao();
        let b = v.para_bytes();
        assert_eq!(b.len(), 32 + TAM_VERIFICACAO_PQ);
        let v2 = ChaveDeVerificacao::de_bytes(&b).unwrap();
        assert_eq!(v, v2);
        let a = k.assinar(b"c", b"m").unwrap();
        assert_eq!(a.partes().1.len(), TAM_ASSINATURA_PQ);
        assert!(v2.verificar(b"c", b"m", &a));
        assert_eq!(ChaveDeVerificacao::de_bytes(&b[..100]).unwrap_err(), ErroDeAssinatura::ChaveMalformada);
    }

    #[test]
    fn contexto_grande_demais_e_recusado() {
        assert_eq!(gerar().assinar(&[0u8; 256], b"m").unwrap_err(), ErroDeAssinatura::ContextoGrandeDemais);
    }

    #[test]
    fn debug_nunca_mostra_a_chave_secreta() {
        assert!(format!("{:?}", gerar()).contains("oculta"));
    }

    #[test]
    fn assinatura_pos_quantica_e_hedged_e_confere() {
        let k = gerar();
        let a = k.assinar(b"ctx", b"mesma mensagem").unwrap();
        let b = k.assinar(b"ctx", b"mesma mensagem").unwrap();
        assert_ne!(a.partes().1, b.partes().1, "variante hedged: duas assinaturas da mesma mensagem diferem");
        assert!(k.verificacao().verificar(b"ctx", b"mesma mensagem", &a) && k.verificacao().verificar(b"ctx", b"mesma mensagem", &b));
        assert!(!k.verificacao().verificar(b"outro", b"mesma mensagem", &a), "contexto continua amarrado");
    }

}
