//! # Módulo `metadados` — o mínimo que o servidor vê, cada vez menor (MET1–MET5)
//!
//! O servidor precisa de um endereço para entregar um bloco. Isso é inevitável enquanto ele estiver
//! entregando. O que este módulo garante é que esse pouco **não se acumula nem se reaproveita**:
//!
//! 1. **Caixa que muda todo dia** (`caixa_da_epoca`): o endereço de cada relação é recalculado por época
//!    (um dia) a partir do segredo da relação. Quem observa o servidor não liga a caixa de hoje à de ontem.
//! 2. **Índice cego no disco** (`ChavesDeIndice`): o banco guarda o bloco sob um índice derivado do endereço
//!    com uma chave do dia, que é **destruída** 48 h depois. Quem levar o disco depois disso não consegue
//!    nem conferir se um endereço conhecido recebeu algo.
//! 3. **Registros exigidos por lei, selados** (`selar_registro`): data, hora e IP do retransmissor
//!    (Marco Civil, art. 15) são cifrados para uma chave jurídica guardada fora do servidor, com tamanho
//!    fixo. Quem opera o servidor escreve, mas não lê; abrir exige a chave jurídica, que fica dividida entre
//!    custodiantes (módulo `custodia`) e só é usada com ordem judicial.
//!
//! O que NÃO some, dito com verdade: enquanto entrega, o servidor (a memória dele) vê que algum endereço do
//! dia recebeu um bloco de certa classe de tamanho, numa certa hora — nunca quem enviou, nunca o IP do
//! aparelho, nunca o conteúdo. Diminuir isso além daqui exige computação confidencial com atestação
//! remota, recuperação privada de informação ou rede de mistura (ver o documento de metadados).

use crate::envelope::{self, ChavePublica, ChaveSecreta};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::Sha512;
use zeroize::Zeroizing;

/// Duração de uma época: um dia.
pub const SEGUNDOS_POR_EPOCA: u64 = 24 * 3600;
/// A chave de índice de uma época vive até 48 h depois do fim dela (prazo máximo de um bloco).
pub const VIDA_DA_CHAVE_DE_INDICE: u64 = 48 * 3600;
/// Tamanho fixo do registro legal antes de selar: nada no tamanho revela o IP.
pub const TAM_DO_REGISTRO: usize = 64;

/// Época de um instante. `agora` vem de `expiracao::hora_confiavel` no aparelho e do relógio do servidor no servidor.
pub fn epoca_de(agora: u64) -> u64 { agora / SEGUNDOS_POR_EPOCA }

/// Endereço da caixa de uma relação numa época. Os dois lados calculam o mesmo; épocas diferentes dão
/// endereços sem relação aparente.
pub fn caixa_da_epoca(segredo_da_relacao: &[u8; 32], epoca: u64) -> [u8; 32] {
    let h = Hkdf::<Sha512>::new(Some(b"NimbusOne/v1/caixa-da-epoca"), segredo_da_relacao);
    let mut e = [0u8; 32];
    h.expand(&epoca.to_be_bytes(), &mut e).expect("tamanho válido");
    e
}

/// Chaves de índice do servidor, uma por época, destruídas depois do prazo. Vivem só em memória (ou em
/// módulo de hardware) e nunca vão para o disco junto com o banco.
#[derive(Default)]
pub struct ChavesDeIndice { chaves: Vec<(u64, Zeroizing<[u8; 32]>)> }

impl core::fmt::Debug for ChavesDeIndice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ChavesDeIndice({} épocas, <ocultas>)", self.chaves.len()) }
}

impl ChavesDeIndice {
    /// Índice cego do endereço na época; cria a chave da época se ainda não existir.
    pub fn indice(&mut self, epoca: u64, endereco: &[u8; 32]) -> [u8; 32] {
        if !self.chaves.iter().any(|(e, _)| *e == epoca) {
            let mut k = Zeroizing::new([0u8; 32]);
            OsRng.fill_bytes(&mut k[..]);
            self.chaves.push((epoca, k));
        }
        let (_, k) = self.chaves.iter().find(|(e, _)| *e == epoca).expect("acabou de ser criada");
        derivar_indice(k, endereco)
    }

    /// Índice de uma época já existente, sem criar chave nova (para leitura). `None` se a chave já foi destruída.
    pub fn indice_existente(&self, epoca: u64, endereco: &[u8; 32]) -> Option<[u8; 32]> {
        self.chaves.iter().find(|(e, _)| *e == epoca).map(|(_, k)| derivar_indice(k, endereco))
    }

    /// Segunda camada no disco ("apagamento criptográfico"): o bloco, que já é cifrado de ponta a ponta,
    /// é cifrado de novo com uma chave derivada da chave do dia, amarrada ao índice. Disco de estado sólido
    /// não garante apagar de verdade (cópias internas ficam); destruída a chave do dia, nem as sobras servem.
    pub fn selar_no_disco(&mut self, epoca: u64, endereco: &[u8; 32], bloco: &[u8]) -> Vec<u8> {
        let indice = self.indice(epoca, endereco);
        let (_, k) = self.chaves.iter().find(|(e, _)| *e == epoca).expect("criada em indice()");
        let chave = derivar_chave_de_disco(k);
        let mut nonce = [0u8; 24];
        OsRng.fill_bytes(&mut nonce);
        let c = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&chave[..]));
        let mut saida = nonce.to_vec();
        saida.extend(c.encrypt(XNonce::from_slice(&nonce), Payload { msg: bloco, aad: &indice }).expect("cifra de tamanho limitado"));
        saida
    }

    /// Abre o que foi selado no disco. `None` se a chave do dia já foi destruída ou se algo foi trocado.
    pub fn abrir_do_disco(&self, epoca: u64, endereco: &[u8; 32], selado: &[u8]) -> Option<Zeroizing<Vec<u8>>> {
        let (_, k) = self.chaves.iter().find(|(e, _)| *e == epoca)?;
        let indice = derivar_indice(k, endereco);
        if selado.len() < 24 { return None; }
        let chave = derivar_chave_de_disco(k);
        let c = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&chave[..]));
        c.decrypt(XNonce::from_slice(&selado[..24]), Payload { msg: &selado[24..], aad: &indice }).ok().map(Zeroizing::new)
    }

    /// Destrói as chaves de épocas encerradas há mais de 48 h. Devolve quantas foram destruídas.
    pub fn destruir_vencidas(&mut self, agora: u64) -> usize {
        let antes = self.chaves.len();
        self.chaves.retain(|(e, _)| agora < e.saturating_add(1).saturating_mul(SEGUNDOS_POR_EPOCA).saturating_add(VIDA_DA_CHAVE_DE_INDICE));
        antes - self.chaves.len()
    }
}

fn derivar_chave_de_disco(chave: &[u8; 32]) -> Zeroizing<[u8; 32]> {
    let h = Hkdf::<Sha512>::new(Some(b"NimbusOne/v1/disco"), chave);
    let mut k = Zeroizing::new([0u8; 32]);
    h.expand(b"chave-do-disco", &mut k[..]).expect("tamanho válido");
    k
}

fn derivar_indice(chave: &[u8; 32], endereco: &[u8; 32]) -> [u8; 32] {
    let h = Hkdf::<Sha512>::new(Some(b"NimbusOne/v1/indice-cego"), chave);
    let mut i = [0u8; 32];
    h.expand(endereco, &mut i).expect("tamanho válido");
    i
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeRegistro { GrandeDemais, Cifra, Ilegivel }

/// Sela um registro de acesso (Marco Civil, art. 15) para a chave jurídica, que não fica no servidor.
pub fn selar_registro(chave_juridica: &ChavePublica, carimbo: u64, ip_do_retransmissor: &str) -> Result<Vec<u8>, ErroDeRegistro> {
    let ip = ip_do_retransmissor.as_bytes();
    if ip.len() > TAM_DO_REGISTRO - 9 { return Err(ErroDeRegistro::GrandeDemais); }
    let mut claro = Zeroizing::new([0u8; TAM_DO_REGISTRO]);
    claro[..8].copy_from_slice(&carimbo.to_be_bytes());
    claro[8] = ip.len() as u8;
    claro[9..9 + ip.len()].copy_from_slice(ip);
    envelope::selar(chave_juridica, &claro[..], b"NimbusOne/v1/registro-legal").map_err(|_| ErroDeRegistro::Cifra)
}

/// Abre um registro selado — só com a chave jurídica (reunida pelos custodiantes, com ordem judicial).
pub fn abrir_registro(sec: &ChaveSecreta, publ: &ChavePublica, selado: &[u8]) -> Result<(u64, String), ErroDeRegistro> {
    let claro = envelope::abrir(sec, publ, selado, b"NimbusOne/v1/registro-legal").map_err(|_| ErroDeRegistro::Ilegivel)?;
    if claro.len() != TAM_DO_REGISTRO { return Err(ErroDeRegistro::Ilegivel); }
    let mut c = [0u8; 8];
    c.copy_from_slice(&claro[..8]);
    let n = claro[8] as usize;
    if 9 + n > TAM_DO_REGISTRO { return Err(ErroDeRegistro::Ilegivel); }
    let ip = core::str::from_utf8(&claro[9..9 + n]).map_err(|_| ErroDeRegistro::Ilegivel)?.to_string();
    Ok((u64::from_be_bytes(c), ip))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn caixa_muda_todo_dia_e_os_dois_lados_concordam() {
        let segredo = [7u8; 32];
        let hoje = epoca_de(1_000_000_000);
        assert_eq!(caixa_da_epoca(&segredo, hoje), caixa_da_epoca(&segredo, hoje));
        assert_ne!(caixa_da_epoca(&segredo, hoje), caixa_da_epoca(&segredo, hoje + 1), "amanhã é outra caixa");
        assert_ne!(caixa_da_epoca(&segredo, hoje), caixa_da_epoca(&[8u8; 32], hoje), "outra relação, outra caixa");
        assert_eq!(epoca_de(SEGUNDOS_POR_EPOCA - 1), 0);
        assert_eq!(epoca_de(SEGUNDOS_POR_EPOCA), 1);
    }

    #[test]
    fn disco_levado_depois_nao_confere_endereco() {
        let mut k = ChavesDeIndice::default();
        let end = [1u8; 32];
        let i = k.indice(10, &end);
        assert_eq!(k.indice_existente(10, &end), Some(i), "durante o prazo, o servidor encontra o bloco");
        assert_ne!(i, end, "o índice não é o endereço");
        assert_ne!(k.indice(11, &end), i, "outra época, outro índice");
        let fim_da_epoca_10 = 11 * SEGUNDOS_POR_EPOCA;
        assert_eq!(k.destruir_vencidas(fim_da_epoca_10 + VIDA_DA_CHAVE_DE_INDICE - 1), 0);
        assert_eq!(k.destruir_vencidas(fim_da_epoca_10 + VIDA_DA_CHAVE_DE_INDICE), 1);
        assert_eq!(k.indice_existente(10, &end), None, "chave destruída: ninguém confere mais o endereço no disco antigo");
        assert!(format!("{k:?}").contains("ocultas"));
    }

    #[test]
    fn registro_legal_so_abre_com_a_chave_juridica_e_tem_tamanho_fixo() {
        let (pj, sj) = envelope::gerar_par_de_chaves();
        let (pa, sa) = envelope::gerar_par_de_chaves();
        let r1 = selar_registro(&pj, 1_700_000_000, "203.0.113.7").unwrap();
        let r2 = selar_registro(&pj, 1_700_000_001, "2001:db8:85a3::8a2e:370:7334").unwrap();
        assert_eq!(r1.len(), r2.len(), "o tamanho não revela o IP");
        assert_eq!(abrir_registro(&sj, &pj, &r1), Ok((1_700_000_000, "203.0.113.7".to_string())));
        assert_eq!(abrir_registro(&sa, &pa, &r1), Err(ErroDeRegistro::Ilegivel), "quem opera o servidor não lê");
        assert_eq!(selar_registro(&pj, 1, &"x".repeat(56)), Err(ErroDeRegistro::GrandeDemais));
        assert!(selar_registro(&pj, 1, &"x".repeat(55)).is_ok());
    }

    #[test]
    fn sobras_do_disco_viram_lixo_quando_a_chave_do_dia_morre() {
        let mut k = ChavesDeIndice::default();
        let (end, outro) = ([1u8; 32], [2u8; 32]);
        let selado = k.selar_no_disco(5, &end, b"bloco ja cifrado de ponta a ponta");
        assert_eq!(k.abrir_do_disco(5, &end, &selado).as_deref().map(|v| &v[..]), Some(&b"bloco ja cifrado de ponta a ponta"[..]));
        assert!(k.abrir_do_disco(5, &outro, &selado).is_none(), "amarrado ao índice: não serve para outra caixa");
        let mut adulterado = selado.clone(); let n = adulterado.len(); adulterado[n - 1] ^= 1;
        assert!(k.abrir_do_disco(5, &end, &adulterado).is_none());
        assert!(k.abrir_do_disco(5, &end, &selado[..10]).is_none());
        let _ = k.destruir_vencidas(6 * SEGUNDOS_POR_EPOCA + VIDA_DA_CHAVE_DE_INDICE);
        assert!(k.abrir_do_disco(5, &end, &selado).is_none(), "chave destruída: a sobra no disco não abre mais");
    }

}
