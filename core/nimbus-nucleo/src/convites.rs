//! # Módulo `convites` — sem telefone, mas sem spam (SPAM1–SPAM6)
//!
//! Conta sem telefone é ótima para privacidade e perigosa para spam: robôs criariam contas sem fim.
//! Defesa: **ninguém fala com você sem convite**. Não existe busca por nome nem por número. Para iniciar
//! uma conversa é preciso um convite de uso único, com prazo, que você entrega pessoalmente (QR) ou por
//! um canal de sua escolha. O servidor só guarda o **hash** do convite, nunca o convite em si.
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use zeroize::Zeroizing;

pub type Convite = Zeroizing<[u8; 32]>;

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeConvite { Inexistente, Expirado, JaUsado }

#[derive(Debug)]
struct Registro { expira_em: u64, usado: bool }

/// Lado do servidor: guarda só o hash de cada convite pendente.
#[derive(Default, Debug)]
pub struct Convites { pendentes: HashMap<[u8; 32], Registro> }

fn hash(c: &[u8; 32]) -> [u8; 32] { Sha256::digest(c).into() }

impl Convites {
    /// Aparelho do dono gera o convite e registra apenas o hash no servidor.
    pub fn criar(&mut self, agora: u64, validade_segundos: u64) -> Convite {
        let mut c = Zeroizing::new([0u8; 32]);
        OsRng.fill_bytes(&mut c[..]);
        let _ = self.pendentes.insert(hash(&c), Registro { expira_em: agora.saturating_add(validade_segundos), usado: false });
        c
    }

    /// Quem recebeu o convite o apresenta uma única vez.
    pub fn resgatar(&mut self, convite: &[u8; 32], agora: u64) -> Result<(), RecusaDeConvite> {
        let r = self.pendentes.get_mut(&hash(convite)).ok_or(RecusaDeConvite::Inexistente)?;
        if r.usado { return Err(RecusaDeConvite::JaUsado); }
        if agora >= r.expira_em { return Err(RecusaDeConvite::Expirado); }
        r.usado = true;
        Ok(())
    }

    /// Faxina: remove convites usados ou vencidos (nada fica guardado além do necessário).
    pub fn limpar(&mut self, agora: u64) { self.pendentes.retain(|_, r| !r.usado && agora < r.expira_em); }

    pub fn quantidade(&self) -> usize { self.pendentes.len() }
}

/// Lista de 256 palavras simples, sem acento, fáceis de falar e de distinguir.
pub const PALAVRAS: [&str; 256] = [
    "abacaxi", "abelha", "acorde", "agulha", "aguia", "alface", "algodao", "alicate", "ameixa", "amigo", "ancora", "anel",
    "anjo", "antena", "apito", "arara", "areia", "arvore", "atleta", "aurora", "azeite", "bacia", "baleia", "balde",
    "bambu", "banana", "bandeira", "barco", "batata", "bengala", "bicho", "bigode", "boia", "bola", "bolo", "bombom",
    "bosque", "bota", "brasa", "brisa", "broto", "bule", "burro", "cabide", "cacau", "cadeira", "caderno", "cafe",
    "caju", "calda", "cama", "camelo", "campo", "caneca", "canoa", "capim", "carta", "casa", "castelo", "cavalo",
    "cebola", "cedro", "cenoura", "cereja", "chave", "chuva", "cinema", "cipo", "circo", "cobra", "coelho", "cofre",
    "coruja", "cravo", "cristal", "cubo", "cuia", "dado", "dedal", "delta", "diamante", "disco", "doce", "dragao",
    "duna", "eclipse", "elefante", "escada", "escudo", "esfera", "espelho", "estrela", "farol", "faca", "fada", "feijao",
    "ferro", "figo", "flauta", "flor", "foca", "fogo", "folha", "fonte", "formiga", "forno", "foguete", "fruta",
    "fumaca", "funil", "galho", "garfo", "gato", "gaveta", "gelo", "girafa", "globo", "gota", "grilo", "guitarra",
    "harpa", "helice", "hortela", "iate", "ilha", "ima", "iogurte", "jabuti", "janela", "jarra", "jasmim", "joia",
    "judo", "lago", "lampada", "lapis", "laranja", "leao", "leite", "lenco", "limao", "lince", "lira", "lobo",
    "lontra", "lua", "maca", "macaco", "madeira", "manga", "mapa", "mar", "martelo", "mel", "melancia", "mesa",
    "milho", "moeda", "montanha", "morango", "mosca", "motor", "mula", "muro", "navio", "neve", "ninho", "noz",
    "nuvem", "oceano", "olho", "onda", "orquidea", "osso", "ostra", "ouro", "ovelha", "palha", "panela", "papel",
    "pato", "pavao", "peixe", "pena", "pera", "perola", "peteca", "piano", "pimenta", "pinha", "pipa", "pirata",
    "planeta", "pneu", "polvo", "ponte", "porco", "porta", "pote", "praia", "prato", "pulga", "quadro", "queijo",
    "quiabo", "radio", "raio", "raposa", "rede", "relogio", "remo", "rio", "robo", "roda", "rosa", "sabao",
    "sapato", "sapo", "seda", "selo", "serra", "sino", "sofa", "sol", "sorvete", "tambor", "tatu", "teia",
    "telha", "tigre", "tijolo", "tinta", "tomate", "touro", "trem", "trevo", "trigo", "tubarao", "tucano", "uva",
    "vaca", "vagao", "vela", "veleiro", "vento", "vidro", "vinho", "violao", "vulcao", "xadrez", "xale", "zebra",
    "zinco", "abobora", "acai", "agenda",
];

/// Palavras de confirmação: depois de aceitar um convite, os dois leem estas seis palavras em voz alta (numa
/// chamada Nimbus ou pessoalmente). Se baterem, ninguém se colocou no meio — mesmo que o código do convite
/// tenha passado por outro aplicativo. A ordem das chaves não importa: os dois veem as mesmas palavras.
pub fn palavras_de_confirmacao(chave_a: &[u8], chave_b: &[u8]) -> [&'static str; 6] {
    palavras_com_dominio(b"NimbusOne/v1/confirmacao", chave_a, chave_b)
}

/// Seis palavras derivadas das duas chaves, na mesma ordem nos dois aparelhos, separadas por finalidade
/// (convite, vínculo de aparelho, recuperação): palavras de uma não servem para a outra.
pub fn palavras_com_dominio(dominio: &[u8], chave_a: &[u8], chave_b: &[u8]) -> [&'static str; 6] {
    let (x, y) = if chave_a <= chave_b { (chave_a, chave_b) } else { (chave_b, chave_a) };
    let mut h = Sha256::new();
    h.update(dominio);
    h.update((x.len() as u64).to_be_bytes());
    h.update(x);
    h.update(y);
    let d: [u8; 32] = h.finalize().into();
    [PALAVRAS[d[0] as usize], PALAVRAS[d[1] as usize], PALAVRAS[d[2] as usize], PALAVRAS[d[3] as usize], PALAVRAS[d[4] as usize], PALAVRAS[d[5] as usize]]
}

/// Confirmação que não dá para pular (EO1). O golpista diz "é só tocar em iguais"; por isso o app não tem
/// botão "são iguais": mostra quatro palavras e pede que a pessoa toque na que o outro LEU em voz alta, na
/// posição sorteada. Devolve as quatro opções (a certa misturada a três outras da lista).
pub fn opcoes_para_confirmar(palavras: &[&'static str; 6], posicao: usize) -> [&'static str; 4] {
    let certa = palavras[posicao % 6];
    let mut opcoes = [certa; 4];
    let mut preenchidas = 1;
    while preenchidas < 4 {
        let mut b = [0u8; 1];
        OsRng.fill_bytes(&mut b);
        let candidata = PALAVRAS[b[0] as usize];
        if !opcoes[..preenchidas].contains(&candidata) { opcoes[preenchidas] = candidata; preenchidas += 1; }
    }
    let mut b = [0u8; 1];
    OsRng.fill_bytes(&mut b);
    opcoes.swap(0, (b[0] % 4) as usize);
    opcoes
}

/// A palavra tocada é a da posição pedida?
pub fn confirmacao_correta(palavras: &[&'static str; 6], posicao: usize, escolhida: &str) -> bool {
    palavras[posicao % 6] == escolhida
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn palavras_sao_256_diferentes_e_sem_acento() {
        let mut v = PALAVRAS.to_vec();
        v.sort_unstable();
        v.dedup();
        assert_eq!(v.len(), 256);
        assert!(PALAVRAS.iter().all(|p| p.is_ascii() && !p.is_empty()));
    }

    #[test]
    fn os_dois_veem_as_mesmas_palavras_e_chaves_diferentes_dao_outras() {
        let (a, b, c) = ([1u8; 32], [2u8; 32], [3u8; 32]);
        assert_eq!(palavras_de_confirmacao(&a, &b), palavras_de_confirmacao(&b, &a));
        assert_ne!(palavras_de_confirmacao(&a, &b), palavras_de_confirmacao(&a, &c), "quem se põe no meio muda as palavras");
    }

    #[test]
    fn convite_vale_uma_vez() {
        let mut s = Convites::default();
        let c = s.criar(0, 86_400);
        assert_eq!(s.resgatar(&c, 10), Ok(()));
        assert_eq!(s.resgatar(&c, 11), Err(RecusaDeConvite::JaUsado));
    }

    #[test]
    fn convite_vencido_e_recusado() {
        let mut s = Convites::default();
        let c = s.criar(0, 60);
        assert_eq!(s.resgatar(&c, 60), Err(RecusaDeConvite::Expirado));
    }

    #[test]
    fn convite_inventado_e_recusado() {
        let mut s = Convites::default();
        let _ = s.criar(0, 60);
        assert_eq!(s.resgatar(&[7u8; 32], 1), Err(RecusaDeConvite::Inexistente));
    }

    #[test]
    fn servidor_guarda_so_o_hash() {
        let mut s = Convites::default();
        let c = s.criar(0, 60);
        assert!(!s.pendentes.contains_key(&*c), "o convite em si não está guardado");
        assert!(s.pendentes.contains_key(&hash(&c)));
    }

    #[test]
    fn faxina_remove_usados_e_vencidos() {
        let mut s = Convites::default();
        let a = s.criar(0, 1_000);
        let _b = s.criar(0, 10);
        let _c = s.criar(0, 1_000);
        s.resgatar(&a, 5).unwrap();
        s.limpar(20);
        assert_eq!(s.quantidade(), 1, "sobra só o convite válido e não usado");
    }

    #[test]
    fn confirmacao_forcada_nao_se_pula() {
        let p = palavras_de_confirmacao(b"chave-a", b"chave-b");
        for pos in 0..6 {
            let o = opcoes_para_confirmar(&p, pos);
            assert!(o.contains(&p[pos]), "a certa está entre as opções");
            let mut u = o.to_vec(); u.sort_unstable(); u.dedup(); assert_eq!(u.len(), 4, "quatro opções diferentes");
            assert!(confirmacao_correta(&p, pos, p[pos]));
            for e in o.iter().filter(|e| **e != p[pos]) { assert!(!confirmacao_correta(&p, pos, e)); }
        }
    }

}
