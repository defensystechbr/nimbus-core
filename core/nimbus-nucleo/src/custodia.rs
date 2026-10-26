//! # Módulo `custodia` — chave mestra dividida entre guardiões, k de n (GUA1–GUA8)
//!
//! A chave mestra da empresa nunca existe inteira em lugar nenhum: nem no servidor, nem com a
//! NimbusOne, nem com um administrador. Ela é dividida entre `n` guardiões (ex.: diretor jurídico,
//! diretor de segurança, presidente), e só `k` deles juntos conseguem reconstruí-la.
//!
//! Método: partilha replicada por XOR (Ito–Saito–Nishizeki). Para cada grupo de `k` guardiões, a chave
//! é dividida em `k` peças aleatórias cuja soma XOR é a chave; cada membro do grupo guarda uma peça.
//! Qualquer grupo com menos de `k` guardiões não completa nenhum grupo e não obtém nada — as peças que
//! tem são ruído aleatório (segurança de teoria da informação, sem aritmética criptográfica própria).
//! Limite prático: até 7 guardiões.
//!
//! Um compromisso (SHA-512 da chave) é publicado junto com a política: ao reconstruir, qualquer peça
//! adulterada ou misturada de outra divisão é detectada.
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha512};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

pub const MAX_GUARDIOES: u8 = 7;

/// Uma peça de um grupo.
pub struct Peca { pub grupo: u16, pub dados: Zeroizing<Vec<u8>> }

impl core::fmt::Debug for Peca {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "Peca(grupo {}, <oculta>)", self.grupo) }
}

/// Tudo o que um guardião guarda (no app real, no chip de segurança do aparelho dele ou em chave física).
pub struct PartesDoGuardiao { pub guardiao: u8, pub n: u8, pub k: u8, pub pecas: Vec<Peca>, pub compromisso: [u8; 64] }

impl core::fmt::Debug for PartesDoGuardiao {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PartesDoGuardiao(guardião {}, {} de {}, <peças ocultas>)", self.guardiao, self.k, self.n)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeCustodia { ParametrosInvalidos, Insuficiente, Incompativel, PecaAdulterada }

fn compromisso_de(segredo: &[u8]) -> [u8; 64] {
    let mut h = Sha512::new();
    h.update(b"NimbusOne/v1/custodia");
    h.update(segredo);
    h.finalize().into()
}

/// Todos os grupos de `k` guardiões entre `0..n`, em ordem lexicográfica (o índice é o id do grupo).
fn grupos(n: u8, k: u8) -> Vec<Vec<u8>> {
    fn rec(inicio: u8, n: u8, k: u8, atual: &mut Vec<u8>, saida: &mut Vec<Vec<u8>>) {
        if atual.len() == k as usize { saida.push(atual.clone()); return; }
        for i in inicio..n { atual.push(i); rec(i + 1, n, k, atual, saida); let _ = atual.pop(); }
    }
    let mut saida = vec![];
    rec(0, n, k, &mut vec![], &mut saida);
    saida
}

/// Divide o segredo entre `n` guardiões, exigindo `k` para reconstruir. Devolve as partes e o compromisso.
pub fn dividir(segredo: &[u8], n: u8, k: u8) -> Result<(Vec<PartesDoGuardiao>, [u8; 64]), ErroDeCustodia> {
    if segredo.is_empty() || k < 2 || k > n || n > MAX_GUARDIOES { return Err(ErroDeCustodia::ParametrosInvalidos); }
    let compromisso = compromisso_de(segredo);
    let mut partes: Vec<PartesDoGuardiao> = (0..n).map(|g| PartesDoGuardiao { guardiao: g, n, k, pecas: vec![], compromisso }).collect();
    for (id, grupo) in grupos(n, k).into_iter().enumerate() {
        // k-1 peças aleatórias; a última completa o XOR até dar o segredo.
        let mut ultima = Zeroizing::new(segredo.to_vec());
        for membro in &grupo[..grupo.len() - 1] {
            let mut peca = Zeroizing::new(vec![0u8; segredo.len()]);
            OsRng.fill_bytes(&mut peca);
            for (u, p) in ultima.iter_mut().zip(peca.iter()) { *u ^= p; }
            partes[*membro as usize].pecas.push(Peca { grupo: id as u16, dados: peca });
        }
        partes[grupo[grupo.len() - 1] as usize].pecas.push(Peca { grupo: id as u16, dados: ultima });
    }
    Ok((partes, compromisso))
}

/// Reúne as partes de pelo menos `k` guardiões distintos e reconstrói o segredo, conferindo o compromisso.
pub fn reconstruir(participantes: &[&PartesDoGuardiao]) -> Result<Zeroizing<Vec<u8>>, ErroDeCustodia> {
    let primeiro = participantes.first().ok_or(ErroDeCustodia::Insuficiente)?;
    let (n, k, compromisso) = (primeiro.n, primeiro.k, primeiro.compromisso);
    if participantes.iter().any(|p| p.n != n || p.k != k || p.compromisso != compromisso) { return Err(ErroDeCustodia::Incompativel); }
    let mut ids: Vec<u8> = participantes.iter().map(|p| p.guardiao).filter(|g| *g < n).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() < k as usize { return Err(ErroDeCustodia::Insuficiente); }
    let escolhidos: Vec<u8> = ids[..k as usize].to_vec();
    let id_grupo = grupos(n, k).iter().position(|g| *g == escolhidos).ok_or(ErroDeCustodia::Incompativel)? as u16;
    let mut segredo: Option<Zeroizing<Vec<u8>>> = None;
    for g in &escolhidos {
        let dono = participantes.iter().find(|p| p.guardiao == *g).ok_or(ErroDeCustodia::Insuficiente)?;
        let peca = dono.pecas.iter().find(|p| p.grupo == id_grupo).ok_or(ErroDeCustodia::PecaAdulterada)?;
        match &mut segredo {
            None => segredo = Some(Zeroizing::new(peca.dados.to_vec())),
            Some(s) => {
                if s.len() != peca.dados.len() { return Err(ErroDeCustodia::PecaAdulterada); }
                for (a, b) in s.iter_mut().zip(peca.dados.iter()) { *a ^= b; }
            }
        }
    }
    let segredo = segredo.ok_or(ErroDeCustodia::Insuficiente)?;
    if !bool::from(compromisso_de(&segredo).ct_eq(&compromisso)) { return Err(ErroDeCustodia::PecaAdulterada); }
    Ok(segredo)
}

impl PartesDoGuardiao {
    /// Bytes para cifrar e entregar a um guardião (módulo `recuperacao`).
    pub fn para_bytes(&self) -> Zeroizing<Vec<u8>> {
        let mut v = Zeroizing::new(vec![self.guardiao, self.n, self.k]);
        v.extend_from_slice(&self.compromisso);
        v.extend_from_slice(&(self.pecas.len() as u16).to_be_bytes());
        for p in &self.pecas {
            v.extend_from_slice(&p.grupo.to_be_bytes());
            v.extend_from_slice(&(p.dados.len() as u32).to_be_bytes());
            v.extend_from_slice(&p.dados);
        }
        v
    }

    pub fn de_bytes(b: &[u8]) -> Result<Self, ErroDeCustodia> {
        let erro = ErroDeCustodia::PecaAdulterada;
        if b.len() < 3 + 64 + 2 { return Err(erro); }
        let (guardiao, n, k) = (b[0], b[1], b[2]);
        let mut compromisso = [0u8; 64];
        compromisso.copy_from_slice(&b[3..67]);
        let qtd = u16::from_be_bytes([b[67], b[68]]) as usize;
        let mut i = 69;
        let mut pecas = Vec::with_capacity(qtd.min(64));
        for _ in 0..qtd {
            if b.len() < i + 6 { return Err(ErroDeCustodia::PecaAdulterada); }
            let grupo = u16::from_be_bytes([b[i], b[i + 1]]);
            let tam = u32::from_be_bytes([b[i + 2], b[i + 3], b[i + 4], b[i + 5]]) as usize;
            i += 6;
            if b.len() < i + tam { return Err(ErroDeCustodia::PecaAdulterada); }
            pecas.push(Peca { grupo, dados: Zeroizing::new(b[i..i + tam].to_vec()) });
            i += tam;
        }
        if i != b.len() { return Err(ErroDeCustodia::PecaAdulterada); }
        Ok(PartesDoGuardiao { guardiao, n, k, pecas, compromisso })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn segredo() -> Vec<u8> { let mut s = vec![0u8; 3200]; OsRng.fill_bytes(&mut s); s }
    fn combinacoes(n: u8, tamanho: u8) -> Vec<Vec<u8>> { grupos(n, tamanho) }

    #[test]
    fn todo_grupo_de_k_guardioes_reconstroi() {
        for (n, k) in [(2, 2), (3, 2), (5, 3), (5, 2), (7, 4), (7, 7)] {
            let s = segredo();
            let (partes, _) = dividir(&s, n, k).unwrap();
            for grupo in combinacoes(n, k) {
                let escolhidos: Vec<&PartesDoGuardiao> = grupo.iter().map(|g| &partes[*g as usize]).collect();
                assert_eq!(&reconstruir(&escolhidos).unwrap()[..], &s[..], "n={n} k={k} grupo={grupo:?}");
            }
        }
    }

    #[test]
    fn menos_de_k_guardioes_nunca_reconstroi() {
        for (n, k) in [(3, 2), (5, 3), (7, 4)] {
            let (partes, _) = dividir(&segredo(), n, k).unwrap();
            for grupo in combinacoes(n, k - 1) {
                let escolhidos: Vec<&PartesDoGuardiao> = grupo.iter().map(|g| &partes[*g as usize]).collect();
                assert_eq!(reconstruir(&escolhidos).unwrap_err(), ErroDeCustodia::Insuficiente);
            }
        }
    }

    #[test]
    fn o_mesmo_guardiao_duas_vezes_conta_uma() {
        let (partes, _) = dividir(&segredo(), 3, 2).unwrap();
        assert_eq!(reconstruir(&[&partes[0], &partes[0]]).unwrap_err(), ErroDeCustodia::Insuficiente);
    }

    #[test]
    fn peca_adulterada_e_detectada() {
        let (mut partes, _) = dividir(&segredo(), 3, 2).unwrap();
        partes[1].pecas[0].dados[10] ^= 1;
        let resultado = reconstruir(&[&partes[0], &partes[1]]);
        assert_eq!(resultado.unwrap_err(), ErroDeCustodia::PecaAdulterada);
    }

    #[test]
    fn partes_de_divisoes_diferentes_nao_se_misturam() {
        let s = segredo();
        let (a, _) = dividir(&s, 3, 2).unwrap();
        let (b, _) = dividir(&s, 3, 2).unwrap();
        // Mesmo segredo, divisões diferentes: peças não combinam, e o compromisso pega.
        assert_eq!(reconstruir(&[&a[0], &b[1]]).unwrap_err(), ErroDeCustodia::PecaAdulterada);
    }

    #[test]
    fn nenhuma_peca_sozinha_e_a_chave() {
        let s = segredo();
        let (partes, _) = dividir(&s, 5, 3).unwrap();
        for p in &partes { for peca in &p.pecas { assert_ne!(&peca.dados[..], &s[..]); } }
    }

    #[test]
    fn parametros_invalidos_sao_recusados() {
        assert_eq!(dividir(&[1], 3, 1).unwrap_err(), ErroDeCustodia::ParametrosInvalidos);
        assert_eq!(dividir(&[1], 3, 4).unwrap_err(), ErroDeCustodia::ParametrosInvalidos);
        assert_eq!(dividir(&[1], 8, 2).unwrap_err(), ErroDeCustodia::ParametrosInvalidos);
        assert_eq!(dividir(&[], 3, 2).unwrap_err(), ErroDeCustodia::ParametrosInvalidos);
    }

    #[test]
    fn partes_serializadas_voltam_iguais_e_bytes_quebrados_sao_recusados() {
        let s = segredo();
        let (partes, _) = dividir(&s, 3, 2).unwrap();
        let a = PartesDoGuardiao::de_bytes(&partes[0].para_bytes()).unwrap();
        let b = PartesDoGuardiao::de_bytes(&partes[2].para_bytes()).unwrap();
        assert_eq!(&reconstruir(&[&a, &b]).unwrap()[..], &s[..]);
        let bytes = partes[1].para_bytes();
        assert!(PartesDoGuardiao::de_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(PartesDoGuardiao::de_bytes(&[1, 2, 3]).is_err());
        let mut sobra = partes[1].para_bytes().to_vec();
        sobra.push(0);
        assert!(PartesDoGuardiao::de_bytes(&sobra).is_err(), "bytes a mais no fim também são recusados");
    }

    #[test]
    fn debug_nunca_mostra_as_pecas() {
        let (partes, _) = dividir(&segredo(), 3, 2).unwrap();
        assert!(format!("{:?}", partes[0]).contains("ocultas"));
    }
}
