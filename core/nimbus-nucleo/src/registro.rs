//! # Módulo `registro` — registro público só-de-acréscimo (árvore de Merkle) (BL1, CO7, BC2)
//!
//! É a parte **útil** da tecnologia de blockchain, sem a parte que nos prejudicaria.
//! * Útil: um registro em que ninguém — nem nós — consegue apagar ou trocar uma entrada sem ser pego.
//!   Usamos para as chaves dos aparelhos (aparelho fantasma é denunciado) e para as imagens do Nimbus OS
//!   e as versões do app (versão adulterada é denunciada).
//! * Dispensada: gravar dados de usuário em cadeia pública e permanente. Isso impediria apagar
//!   (quebra o apagamento criptográfico) e exporia metadados para sempre.
//!
//! Estrutura conforme a Transparência de Certificados (IETF RFC 9162, seção 2.1): folha = SHA-256(0x00‖dado),
//! nó = SHA-256(0x01‖esquerda‖direita). Qualquer pessoa confere se uma entrada está no registro com uma
//! prova curta (log₂ n hashes), sem baixar o registro inteiro.
//! Prova de consistência entre duas versões do registro (RFC 9162, 2.1.4): sem ela, um servidor desonesto
//! poderia mostrar registros diferentes a pessoas diferentes ou reescrever o passado sem ser pego.
use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];

pub fn hash_de_folha(dado: &[u8]) -> Hash {
    let mut h = Sha256::new(); h.update([0x00]); h.update(dado); h.finalize().into()
}
fn hash_de_no(e: &Hash, d: &Hash) -> Hash {
    let mut h = Sha256::new(); h.update([0x01]); h.update(e); h.update(d); h.finalize().into()
}
fn maior_potencia_de_2_menor_que(n: usize) -> usize { let mut k = 1; while k << 1 < n { k <<= 1; } k }

fn raiz_de(folhas: &[Hash]) -> Hash {
    match folhas.len() {
        0 => Sha256::digest([]).into(),
        1 => folhas[0],
        n => { let k = maior_potencia_de_2_menor_que(n); hash_de_no(&raiz_de(&folhas[..k]), &raiz_de(&folhas[k..])) }
    }
}

fn caminho(m: usize, folhas: &[Hash]) -> Vec<Hash> {
    let n = folhas.len();
    if n <= 1 { return vec![]; }
    let k = maior_potencia_de_2_menor_que(n);
    if m < k { let mut p = caminho(m, &folhas[..k]); p.push(raiz_de(&folhas[k..])); p }
    else { let mut p = caminho(m - k, &folhas[k..]); p.push(raiz_de(&folhas[..k])); p }
}

/// Registro só-de-acréscimo. Não existe função de apagar nem de alterar.
#[derive(Default, Debug)]
pub struct Registro { folhas: Vec<Hash> }

impl Registro {
    pub fn acrescentar(&mut self, dado: &[u8]) -> usize { self.folhas.push(hash_de_folha(dado)); self.folhas.len() - 1 }
    pub fn tamanho(&self) -> usize { self.folhas.len() }
    pub fn raiz(&self) -> Hash { raiz_de(&self.folhas) }
    pub fn prova_de_inclusao(&self, indice: usize) -> Option<Vec<Hash>> {
        if indice >= self.folhas.len() { return None; }
        Some(caminho(indice, &self.folhas))
    }
}

/// Qualquer um verifica, sem confiar em nós: a entrada `dado` está na posição `indice` do registro de
/// tamanho `tamanho` cuja raiz é `raiz`. Algoritmo da RFC 9162, 2.1.3.2.
pub fn verificar_inclusao(dado: &[u8], indice: usize, tamanho: usize, prova: &[Hash], raiz: &Hash) -> bool {
    if indice >= tamanho { return false; }
    let (mut fn_, mut sn) = (indice, tamanho - 1);
    let mut r = hash_de_folha(dado);
    for p in prova {
        if sn == 0 { return false; }
        if fn_ & 1 == 1 || fn_ == sn {
            r = hash_de_no(p, &r);
            if fn_ & 1 == 0 { while fn_ & 1 == 0 && fn_ != 0 { fn_ >>= 1; sn >>= 1; } }
        } else {
            r = hash_de_no(&r, p);
        }
        fn_ >>= 1; sn >>= 1;
    }
    sn == 0 && &r == raiz
}

fn subprova(m: usize, folhas: &[Hash], completo: bool) -> Vec<Hash> {
    let n = folhas.len();
    if m == n { return if completo { vec![] } else { vec![raiz_de(folhas)] }; }
    let k = maior_potencia_de_2_menor_que(n);
    if m <= k { let mut p = subprova(m, &folhas[..k], completo); p.push(raiz_de(&folhas[k..])); p }
    else { let mut p = subprova(m - k, &folhas[k..], false); p.push(raiz_de(&folhas[..k])); p }
}

impl Registro {
    /// Raiz do registro quando ele tinha só as primeiras `tamanho` entradas.
    pub fn raiz_em(&self, tamanho: usize) -> Option<Hash> {
        if tamanho > self.folhas.len() { return None; }
        Some(raiz_de(&self.folhas[..tamanho]))
    }
    /// Prova de que a versão com `anterior` entradas é o começo exato da versão com `atual` entradas
    /// (RFC 9162, 2.1.4.1). Exige 0 < anterior < atual <= tamanho.
    pub fn prova_de_consistencia(&self, anterior: usize, atual: usize) -> Option<Vec<Hash>> {
        if anterior == 0 || anterior >= atual || atual > self.folhas.len() { return None; }
        Some(subprova(anterior, &self.folhas[..atual], true))
    }
}

/// Qualquer um verifica, sem confiar em nós, que o registro só cresceu: a versão de tamanho `anterior` e
/// raiz `raiz_anterior` está contida, sem alteração, na versão de tamanho `atual` e raiz `raiz_atual`.
/// Algoritmo da RFC 9162, 2.1.4.2. Tamanhos iguais exigem raízes iguais e prova vazia.
pub fn verificar_consistencia(anterior: usize, atual: usize, prova: &[Hash], raiz_anterior: &Hash, raiz_atual: &Hash) -> bool {
    if anterior == atual { return prova.is_empty() && raiz_anterior == raiz_atual; }
    if anterior == 0 || anterior > atual || prova.is_empty() { return false; }
    let mut caminho: Vec<Hash> = Vec::with_capacity(prova.len() + 1);
    if anterior.is_power_of_two() { caminho.push(*raiz_anterior); }
    caminho.extend_from_slice(prova);
    let (mut fn_, mut sn) = (anterior - 1, atual - 1);
    while fn_ & 1 == 1 { fn_ >>= 1; sn >>= 1; }
    let (mut fr, mut sr) = (caminho[0], caminho[0]);
    for c in &caminho[1..] {
        if sn == 0 { return false; }
        if fn_ & 1 == 1 || fn_ == sn {
            fr = hash_de_no(c, &fr);
            sr = hash_de_no(c, &sr);
            if fn_ & 1 == 0 { while fn_ & 1 == 0 && fn_ != 0 { fn_ >>= 1; sn >>= 1; } }
        } else {
            sr = hash_de_no(&sr, c);
        }
        fn_ >>= 1; sn >>= 1;
    }
    &fr == raiz_anterior && &sr == raiz_atual && sn == 0
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn consistencia_vale_para_todos_os_tamanhos_ate_40() {
        let r = registro_com(40);
        for atual in 1..=40usize {
            let raiz_atual = r.raiz_em(atual).unwrap();
            for anterior in 1..atual {
                let raiz_ant = r.raiz_em(anterior).unwrap();
                let p = r.prova_de_consistencia(anterior, atual).unwrap();
                assert!(verificar_consistencia(anterior, atual, &p, &raiz_ant, &raiz_atual), "{anterior}->{atual}");
                // Raiz anterior trocada (história reescrita), raiz atual trocada ou prova adulterada: recusa.
                let mut outra = raiz_ant; outra[0] ^= 1;
                assert!(!verificar_consistencia(anterior, atual, &p, &outra, &raiz_atual));
                let mut outra2 = raiz_atual; outra2[31] ^= 1;
                assert!(!verificar_consistencia(anterior, atual, &p, &raiz_ant, &outra2));
                for j in 0..p.len() {
                    let mut q = p.clone(); q[j][5] ^= 0x80;
                    assert!(!verificar_consistencia(anterior, atual, &q, &raiz_ant, &raiz_atual), "{anterior}->{atual} item {j}");
                }
                // Prova de outro par de tamanhos não serve.
                if anterior + 1 < atual {
                    assert!(!verificar_consistencia(anterior + 1, atual, &p, &r.raiz_em(anterior + 1).unwrap(), &raiz_atual) || p == r.prova_de_consistencia(anterior + 1, atual).unwrap());
                }
            }
        }
    }

    #[test]
    fn historia_reescrita_e_pega() {
        // O servidor mostra a raiz de 10 entradas; depois troca a entrada 3 e acrescenta outras.
        let honesto = registro_com(10);
        let mut desonesto = Registro::default();
        for i in 0..20u32 { let d = if i == 3 { b"aparelho-fantasma".to_vec() } else { i.to_be_bytes().to_vec() }; let _ = desonesto.acrescentar(&d); }
        let p = desonesto.prova_de_consistencia(10, 20).unwrap();
        assert!(!verificar_consistencia(10, 20, &p, &honesto.raiz(), &desonesto.raiz()));
    }

    #[test]
    fn tamanhos_invalidos_sao_recusados() {
        let r = registro_com(8);
        assert!(r.prova_de_consistencia(0, 8).is_none() && r.prova_de_consistencia(8, 8).is_none() && r.prova_de_consistencia(3, 9).is_none());
        assert!(!verificar_consistencia(0, 8, &[], &r.raiz(), &r.raiz()));
        assert!(!verificar_consistencia(5, 8, &[], &r.raiz_em(5).unwrap(), &r.raiz()));
        assert!(verificar_consistencia(8, 8, &[], &r.raiz(), &r.raiz()));
        assert!(!verificar_consistencia(8, 8, &[], &r.raiz_em(5).unwrap(), &r.raiz()), "mesmo tamanho, raízes diferentes: dois registros");
        assert!(!verificar_consistencia(9, 8, &[r.raiz()], &r.raiz(), &r.raiz()));
    }

    fn registro_com(n: usize) -> Registro {
        let mut r = Registro::default();
        for i in 0..n { let _ = r.acrescentar(format!("chave-do-aparelho-{i}").as_bytes()); }
        r
    }

    #[test]
    fn toda_entrada_de_todo_tamanho_tem_prova_valida() {
        for n in 1..=33 {
            let r = registro_com(n);
            let raiz = r.raiz();
            for i in 0..n {
                let p = r.prova_de_inclusao(i).unwrap();
                assert!(verificar_inclusao(format!("chave-do-aparelho-{i}").as_bytes(), i, n, &p, &raiz), "n={n} i={i}");
            }
        }
    }

    #[test]
    fn entrada_trocada_nao_passa() {
        let r = registro_com(10);
        let p = r.prova_de_inclusao(4).unwrap();
        assert!(!verificar_inclusao(b"chave-do-aparelho-FANTASMA", 4, 10, &p, &r.raiz()));
    }

    #[test]
    fn posicao_errada_nao_passa() {
        let r = registro_com(10);
        let p = r.prova_de_inclusao(4).unwrap();
        assert!(!verificar_inclusao(b"chave-do-aparelho-4", 5, 10, &p, &r.raiz()));
    }

    #[test]
    fn qualquer_acrescimo_muda_a_raiz() {
        let mut r = registro_com(8);
        let antes = r.raiz();
        let _ = r.acrescentar(b"nova entrada");
        assert_ne!(antes, r.raiz());
    }

    #[test]
    fn prova_e_curta() {
        let r = registro_com(1000);
        assert!(r.prova_de_inclusao(500).unwrap().len() <= 10, "log2(1000) ≈ 10 hashes");
    }

    #[test]
    fn raiz_bate_com_a_definicao_da_rfc_para_duas_folhas() {
        let r = registro_com(2);
        let esperado = hash_de_no(&hash_de_folha(b"chave-do-aparelho-0"), &hash_de_folha(b"chave-do-aparelho-1"));
        assert_eq!(r.raiz(), esperado);
    }
}
