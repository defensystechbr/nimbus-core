//! # Módulo `envelope` — envelope híbrido pós-quântico
//!
//! Envelope híbrido pós-quântico do NimbusOne (requisitos CRY1, CRY3, ARQ4 do Blueprint).
//!
//! * Troca de chaves: X25519 **e** ML-KEM-1024 (NIST FIPS 203). O atacante precisa quebrar as duas.
//! * Combinação: HKDF-SHA-512 sobre os dois segredos, amarrada às chaves públicas e ao texto cifrado do KEM.
//! * Cifra: XChaCha20-Poly1305 (chave de 256 bits, nonce aleatório de 192 bits).
//! * Preenchimento: o tamanho do texto é arredondado para faixas fixas (potências de 2, mínimo 256 bytes).
//! * Segredos são zerados da memória ao sair de escopo (`zeroize`).
//!
//! **Protótipo para auditoria — não usar em produção antes da prova C1 (auditoria independente).**

use aes_gcm::Aes256Gcm;
use chacha20poly1305::{aead::{Aead, KeyInit, Payload}, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use ml_kem::{
    kem::{Decapsulate, Encapsulate},
    Ciphertext, EncodedSizeUser, KemCore, MlKem1024,
};
use rand_core::{OsRng, RngCore};
use sha2::Sha512;
use x25519_dalek::{PublicKey as XPublic, StaticSecret as XSecret};
use zeroize::{Zeroize, Zeroizing};

type ChaveDecap = <MlKem1024 as KemCore>::DecapsulationKey;
type ChaveEncap = <MlKem1024 as KemCore>::EncapsulationKey;

/// Versão do formato do envelope.
pub const VERSAO: u8 = 1;
const DOMINIO: &[u8] = b"NimbusOne/v1/envelope-hibrido/X25519+ML-KEM-1024";
pub const TAM_X25519: usize = 32;
const TAM_CT_KEM: usize = 1568; // FIPS 203, ML-KEM-1024
const TAM_CHAVE_KEM: usize = 1568;
const TAM_NONCE: usize = 24;
const TAM_CABECALHO: usize = 2 + TAM_X25519 + TAM_CT_KEM + TAM_NONCE;
const FAIXA_MINIMA: usize = 256;

#[derive(Debug, PartialEq, Eq)]
pub enum Erro {
    MalFormado,
    FalhaAoDecifrar,
    VersaoInvalida,
    /// Conteúdo acima do limite do envelope; arquivos grandes vão pelo módulo `envio`, em blocos.
    GrandeDemais,
    /// Chave pública X25519 de ordem baixa (degenerada): recusada para não zerar metade do segredo.
    ChaveInvalida,
}

/// Limite do envelope: 64 MiB. Acima disso, usar a cifra em blocos do módulo `envio`.
pub const TAM_MAXIMO: usize = 64 * 1024 * 1024;

/// Confere o limite antes de alocar qualquer coisa.
pub fn tamanho_permitido(n: usize) -> bool { n <= TAM_MAXIMO }

/// Suíte de cifra do conteúdo.
/// * `XChaCha20Poly1305` — padrão da edição consumidor.
/// * `Aes256Gcm` — edição FIPS (bancos, governo, empresas). Em produção, a edição FIPS deve usar um
///   módulo criptográfico com validação FIPS 140-3; esta implementação (RustCrypto) serve para testes.
///
/// A chave é nova a cada envelope (derivada de KEM efêmero), então o nonce nunca se repete sob a mesma chave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suite { XChaCha20Poly1305 = 1, Aes256Gcm = 2 }

impl Suite {
    fn de_byte(b: u8) -> Result<Self, Erro> {
        match b { 1 => Ok(Suite::XChaCha20Poly1305), 2 => Ok(Suite::Aes256Gcm), _ => Err(Erro::VersaoInvalida) }
    }
}

fn cifrar(suite: Suite, chave: &[u8; 32], nonce: &[u8; TAM_NONCE], msg: &[u8], aad: &[u8]) -> Vec<u8> {
    match suite {
        Suite::XChaCha20Poly1305 => XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(chave))
            .encrypt(XNonce::from_slice(nonce), Payload { msg, aad }).expect("cifra"),
        Suite::Aes256Gcm => Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(chave))
            .encrypt(aes_gcm::Nonce::from_slice(&nonce[..12]), Payload { msg, aad }).expect("cifra"),
    }
}

fn decifrar(suite: Suite, chave: &[u8; 32], nonce: &[u8], ct: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {
    match suite {
        Suite::XChaCha20Poly1305 => XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(chave))
            .decrypt(XNonce::from_slice(nonce), Payload { msg: ct, aad }),
        Suite::Aes256Gcm => Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(chave))
            .decrypt(aes_gcm::Nonce::from_slice(&nonce[..12]), Payload { msg: ct, aad }),
    }.map_err(|_| Erro::FalhaAoDecifrar)
}

/// Chave pública de um destinatário (pode ser publicada).
#[derive(Clone)]
pub struct ChavePublica { x: XPublic, kem: ChaveEncap }

impl core::fmt::Debug for ChavePublica {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ChavePublica(X25519 + ML-KEM-1024)") }
}

/// Chave secreta do destinatário. Zerada da memória ao ser descartada (features `zeroize` dos crates).
/// No app real, a chave de longo prazo fica protegida por hardware seguro (Secure Enclave/StrongBox).
pub struct ChaveSecreta { x: XSecret, kem: ChaveDecap }

impl core::fmt::Debug for ChaveSecreta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ChaveSecreta(<oculta>)") }
}

/// Tamanho da chave secreta serializada: X25519 (32) + ML-KEM-1024 (3168).
pub const TAM_CHAVE_SECRETA: usize = 32 + 3168;

impl ChaveSecreta {
    /// Serializa a chave secreta — usado só para dividir a chave mestra entre guardiões (módulo custodia).
    /// O resultado se zera sozinho ao sair de escopo.
    pub fn para_bytes(&self) -> Zeroizing<Vec<u8>> {
        let mut v = Zeroizing::new(Vec::with_capacity(TAM_CHAVE_SECRETA));
        v.extend_from_slice(&self.x.to_bytes());
        v.extend_from_slice(self.kem.as_bytes().as_slice());
        v
    }

    /// Reconstrói a chave secreta a partir dos bytes (depois de reunir as partes dos guardiões).
    pub fn de_bytes(b: &[u8]) -> Result<Self, Erro> {
        if b.len() != TAM_CHAVE_SECRETA { return Err(Erro::MalFormado); }
        let mut xb = Zeroizing::new([0u8; 32]);
        xb.copy_from_slice(&b[..32]);
        let enc = b[32..].try_into().map_err(|_| Erro::MalFormado)?;
        Ok(ChaveSecreta { x: XSecret::from(*xb), kem: ChaveDecap::from_bytes(enc) })
    }
}

/// Gera um par de chaves híbrido.
pub fn gerar_par_de_chaves() -> (ChavePublica, ChaveSecreta) {
    let x = XSecret::random_from_rng(OsRng);
    let (secreta_kem, publica_kem) = MlKem1024::generate(&mut OsRng);
    (ChavePublica { x: XPublic::from(&x), kem: publica_kem }, ChaveSecreta { x, kem: secreta_kem })
}

impl ChavePublica {
    /// Serializa: 32 bytes X25519 || 1568 bytes ML-KEM-1024.
    pub fn para_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(TAM_X25519 + TAM_CHAVE_KEM);
        v.extend_from_slice(self.x.as_bytes());
        v.extend_from_slice(self.kem.as_bytes().as_slice());
        v
    }
    /// Lê uma chave pública recebida de fora. Recusa chave X25519 de ordem baixa (degenerada): toda
    /// `ChavePublica` existente no programa é, por construção, válida.
    pub fn de_bytes(b: &[u8]) -> Result<Self, Erro> {
        if b.len() != TAM_X25519 + TAM_CHAVE_KEM { return Err(Erro::MalFormado); }
        let mut xb = [0u8; TAM_X25519];
        xb.copy_from_slice(&b[..TAM_X25519]);
        let x = XPublic::from(xb);
        // Um ponto de ordem baixa multiplicado por qualquer escalar ajustado dá o elemento neutro.
        if !XSecret::from([7u8; 32]).diffie_hellman(&x).was_contributory() { return Err(Erro::ChaveInvalida); }
        let arr_kem = b[TAM_X25519..].try_into().map_err(|_| Erro::MalFormado)?;
        let kem = ChaveEncap::from_bytes(arr_kem);
        // FIPS 203, 7.2 (verificação da chave de encapsulamento): decodificar e codificar de novo tem de
        // devolver os mesmos bytes. Coeficientes fora do intervalo (>= q) seriam reduzidos em silêncio e a
        // chave usada não seria a publicada.
        if kem.as_bytes().as_slice() != &b[TAM_X25519..] { return Err(Erro::ChaveInvalida); }
        Ok(ChavePublica { x, kem })
    }

}

fn derivar_chave(suite: Suite, segredo_kem: &[u8], segredo_x: &[u8], ct_kem: &[u8], x_efemera: &[u8; 32], destinatario: &ChavePublica) -> Zeroizing<[u8; 32]> {
    let mut entrada = Zeroizing::new(Vec::with_capacity(64));
    entrada.extend_from_slice(segredo_kem);
    entrada.extend_from_slice(segredo_x);
    let mut info = Vec::with_capacity(1 + TAM_CT_KEM + 2 * TAM_X25519 + TAM_CHAVE_KEM);
    info.push(suite as u8);
    info.extend_from_slice(ct_kem);
    info.extend_from_slice(x_efemera);
    info.extend_from_slice(&destinatario.para_bytes());
    let hkdf = Hkdf::<Sha512>::new(Some(DOMINIO), &entrada);
    let mut material = Zeroizing::new([0u8; 32]);
    hkdf.expand(&info, material.as_mut()).expect("32 bytes é um tamanho válido para HKDF-SHA-512");
    material
}

/// Tamanho preenchido: próxima potência de 2, mínimo 256 bytes (requisito ARQ4).
pub fn tamanho_preenchido(n: usize) -> usize { (n + 4).max(FAIXA_MINIMA).next_power_of_two() }

fn preencher(msg: &[u8]) -> Zeroizing<Vec<u8>> {
    let total = tamanho_preenchido(msg.len());
    let mut v = Zeroizing::new(Vec::with_capacity(total));
    v.extend_from_slice(&(msg.len() as u32).to_be_bytes());
    v.extend_from_slice(msg);
    v.resize(total, 0);
    v
}

fn remover_preenchimento(buf: &[u8]) -> Result<Vec<u8>, Erro> {
    if buf.len() < 4 { return Err(Erro::MalFormado); }
    let mut t = [0u8; 4];
    t.copy_from_slice(&buf[..4]);
    let n = u32::from_be_bytes(t) as usize;
    if n > buf.len() - 4 { return Err(Erro::MalFormado); }
    Ok(buf[4..4 + n].to_vec())
}

/// Cifra `texto_claro` para `destinatario` com a suíte padrão (XChaCha20-Poly1305).
/// `aad` = dados associados autenticados (ex.: id da conversa, prazo de expiração).
pub fn selar(destinatario: &ChavePublica, texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {
    selar_com(Suite::XChaCha20Poly1305, destinatario, texto_claro, aad)
}

/// Cifra com a suíte escolhida.
pub fn selar_com(suite: Suite, destinatario: &ChavePublica, texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {
    if !tamanho_permitido(texto_claro.len()) { return Err(Erro::GrandeDemais); }
    let efemera = XSecret::random_from_rng(OsRng);
    let eph_pub = XPublic::from(&efemera);
    let segredo_x = efemera.diffie_hellman(&destinatario.x);
    if !segredo_x.was_contributory() { return Err(Erro::ChaveInvalida); }
    let (ct_kem, segredo_kem) = destinatario.kem.encapsulate(&mut OsRng).expect("encapsulamento ML-KEM");
    let chave = derivar_chave(suite, segredo_kem.as_slice(), segredo_x.as_bytes(), ct_kem.as_slice(), eph_pub.as_bytes(), destinatario);
    let mut nonce = [0u8; TAM_NONCE];
    OsRng.fill_bytes(&mut nonce);
    let preenchido = preencher(texto_claro);
    let mut cabecalho = Vec::with_capacity(TAM_CABECALHO);
    cabecalho.push(VERSAO);
    cabecalho.push(suite as u8);
    cabecalho.extend_from_slice(eph_pub.as_bytes());
    cabecalho.extend_from_slice(ct_kem.as_slice());
    cabecalho.extend_from_slice(&nonce);
    let mut aad_completo = cabecalho.clone();
    aad_completo.extend_from_slice(aad);
    let ct = cifrar(suite, &chave, &nonce, &preenchido, &aad_completo);
    let mut saida = cabecalho;
    saida.extend_from_slice(&ct);
    Ok(saida)
}

/// Tamanho da chave simétrica usada por `selar_com_chave_simetrica`/`abrir_com_chave_simetrica`.
pub const TAM_CHAVE_SIMETRICA: usize = 32;
const TAM_CABECALHO_SIMETRICO: usize = 2 + TAM_NONCE;

/// Cifra com uma chave simétrica que as duas pontas já têm por outro canal — ao contrário de `selar`, que
/// cifra PARA uma `ChavePublica` (troca de chaves X25519+ML-KEM nova a cada chamada). Usada quando não há
/// par de chaves assimétrico envolvido, ex.: a frase de recuperação pessoal (`recuperacao::preparar_frase_pessoal`),
/// onde a "chave" É o segredo que a pessoa guarda — não algo que se troca com ninguém. Mesma cifra
/// (XChaCha20-Poly1305) e o mesmo preenchimento de `selar`; só a etapa de acordo de chave (KEM/X25519) não
/// existe aqui, porque não há destinatário remoto — as duas ferramentas resolvem problemas diferentes.
pub fn selar_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {
    if !tamanho_permitido(texto_claro.len()) { return Err(Erro::GrandeDemais); }
    let mut nonce = [0u8; TAM_NONCE];
    OsRng.fill_bytes(&mut nonce);
    let preenchido = preencher(texto_claro);
    let mut cabecalho = Vec::with_capacity(TAM_CABECALHO_SIMETRICO);
    cabecalho.push(VERSAO);
    cabecalho.push(Suite::XChaCha20Poly1305 as u8);
    cabecalho.extend_from_slice(&nonce);
    let mut aad_completo = cabecalho.clone();
    aad_completo.extend_from_slice(aad);
    let ct = cifrar(Suite::XChaCha20Poly1305, chave, &nonce, &preenchido, &aad_completo);
    let mut saida = cabecalho;
    saida.extend_from_slice(&ct);
    Ok(saida)
}

/// Decifra o que `selar_com_chave_simetrica` produziu. Qualquer byte alterado (ou `aad` diferente) falha.
/// **Achado da Rodada 22 (releitura byte a byte contra a regra 2 do CONTRATO_DA_PONTE.md):** até esta
/// rodada, devolvia `Vec<u8>` comum — o buffer de preenchimento interno já era zerado (`preenchido.zeroize()`
/// abaixo), mas o texto claro de verdade, que SAI desta função e chega a quem chamou, não tinha zeragem
/// nenhuma garantida pelo tipo; dependia de cada chamador lembrar de envolver o retorno em `Zeroizing`
/// (a maioria já fazia isso manualmente — ver `recuperacao.rs`, `direto.rs`, `contatos.rs`, `pacote.rs`,
/// `metadados.rs` — mas pelo menos `historico::ler` e `custodia_mutua::abrir_pacote_de_transferencia` só
/// alcançavam o mesmo resultado por fazer a mesma coisa manualmente, e nada no tipo impedia um futuro
/// chamador de esquecer). Agora o próprio tipo de retorno garante a zeragem automática ao sair de
/// escopo, para todo chamador, sem exceção.
pub fn abrir_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], envelope: &[u8], aad: &[u8]) -> Result<Zeroizing<Vec<u8>>, Erro> {
    if envelope.len() < TAM_CABECALHO_SIMETRICO + 16 { return Err(Erro::MalFormado); }
    if envelope[0] != VERSAO { return Err(Erro::VersaoInvalida); }
    let suite = Suite::de_byte(envelope[1])?;
    let nonce = &envelope[2..TAM_CABECALHO_SIMETRICO];
    let aad_completo = {
        let mut v = envelope[..TAM_CABECALHO_SIMETRICO].to_vec();
        v.extend_from_slice(aad);
        v
    };
    let mut preenchido = decifrar(suite, chave, nonce, &envelope[TAM_CABECALHO_SIMETRICO..], &aad_completo)?;
    let r = remover_preenchimento(&preenchido).map(Zeroizing::new);
    preenchido.zeroize();
    r
}

/// Decifra um envelope. Qualquer alteração em qualquer byte (ou `aad` diferente) retorna `Erro::FalhaAoDecifrar`.
/// **Achado da Rodada 22 — mesma correção de `abrir_com_chave_simetrica` acima, ver o comentário ali para
/// o porquê:** o texto claro devolvido agora é `Zeroizing<Vec<u8>>`, zerado automaticamente ao sair de
/// escopo, em vez de depender de cada chamador envolver o retorno por conta própria.
pub fn abrir(chave_secreta: &ChaveSecreta, chave_publica: &ChavePublica, envelope: &[u8], aad: &[u8]) -> Result<Zeroizing<Vec<u8>>, Erro> {
    if envelope.len() < TAM_CABECALHO + 16 { return Err(Erro::MalFormado); }
    if envelope[0] != VERSAO { return Err(Erro::VersaoInvalida); }
    let suite = Suite::de_byte(envelope[1])?;
    let mut efemera = [0u8; TAM_X25519];
    efemera.copy_from_slice(&envelope[2..2 + TAM_X25519]);
    let ct_kem_bytes = &envelope[2 + TAM_X25519..2 + TAM_X25519 + TAM_CT_KEM];
    let nonce = &envelope[2 + TAM_X25519 + TAM_CT_KEM..TAM_CABECALHO];
    let ct_kem: Ciphertext<MlKem1024> = ct_kem_bytes.try_into().map_err(|_| Erro::MalFormado)?;
    let segredo_kem = chave_secreta.kem.decapsulate(&ct_kem).map_err(|_| Erro::FalhaAoDecifrar)?;
    let segredo_x = chave_secreta.x.diffie_hellman(&XPublic::from(efemera));
    if !segredo_x.was_contributory() { return Err(Erro::ChaveInvalida); }
    let chave = derivar_chave(suite, segredo_kem.as_slice(), segredo_x.as_bytes(), ct_kem_bytes, &efemera, chave_publica);
    let mut aad_completo = envelope[..TAM_CABECALHO].to_vec();
    aad_completo.extend_from_slice(aad);
    let mut preenchido = decifrar(suite, &chave, nonce, &envelope[TAM_CABECALHO..], &aad_completo)?;
    let r = remover_preenchimento(&preenchido).map(Zeroizing::new);
    preenchido.zeroize();
    r
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn chave_ml_kem_fora_do_intervalo_e_recusada() {
        let (p, _s) = gerar_par_de_chaves();
        let mut b = p.para_bytes();
        // 12 bits por coeficiente: 0xFF 0xFF 0xFF codifica dois coeficientes 4095, acima de q = 3329.
        b[TAM_X25519] = 0xFF; b[TAM_X25519 + 1] = 0xFF; b[TAM_X25519 + 2] = 0xFF;
        assert_eq!(ChavePublica::de_bytes(&b).unwrap_err(), Erro::ChaveInvalida);
        assert!(ChavePublica::de_bytes(&p.para_bytes()).is_ok(), "a chave legítima continua aceita");
    }

    #[test]
    fn ida_e_volta() {
        let (chave_publica, chave_secreta) = gerar_par_de_chaves();
        let env = selar(&chave_publica, b"reuniao amanha 9h", b"conversa-42").unwrap();
        assert_eq!(abrir(&chave_secreta, &chave_publica, &env, b"conversa-42").unwrap().as_slice(), b"reuniao amanha 9h");
    }

    #[test]
    fn chave_errada_falha() {
        let (chave_publica, _) = gerar_par_de_chaves();
        let (pk2, sk2) = gerar_par_de_chaves();
        let env = selar(&chave_publica, b"segredo", b"").unwrap();
        assert_eq!(abrir(&sk2, &pk2, &env, b""), Err(Erro::FalhaAoDecifrar));
    }

    #[test]
    fn qualquer_byte_alterado_falha() {
        let (chave_publica, chave_secreta) = gerar_par_de_chaves();
        let env = selar(&chave_publica, b"segredo", b"").unwrap();
        for i in [0usize, 1, 40, 1600, TAM_CABECALHO - 1, TAM_CABECALHO, env.len() - 1] {
            let mut t = env.clone();
            t[i] ^= 0x01;
            assert!(abrir(&chave_secreta, &chave_publica, &t, b"").is_err(), "byte {i} alterado deveria falhar");
        }
    }

    #[test]
    fn aad_diferente_falha() {
        let (chave_publica, chave_secreta) = gerar_par_de_chaves();
        let env = selar(&chave_publica, b"segredo", b"expira:2026-09-30").unwrap();
        assert_eq!(abrir(&chave_secreta, &chave_publica, &env, b"expira:2099-01-01"), Err(Erro::FalhaAoDecifrar));
    }

    #[test]
    fn tamanho_nao_revela_conteudo() {
        let (chave_publica, _) = gerar_par_de_chaves();
        let a = selar(&chave_publica, b"ok", b"").unwrap();
        let b = selar(&chave_publica, &[7u8; 200], b"").unwrap();
        assert_eq!(a.len(), b.len(), "mensagens curtas caem na mesma faixa");
        assert_eq!(tamanho_preenchido(300), 512);
    }

    #[test]
    fn mesma_mensagem_gera_envelopes_diferentes() {
        let (chave_publica, _) = gerar_par_de_chaves();
        assert_ne!(selar(&chave_publica, b"x", b"").unwrap(), selar(&chave_publica, b"x", b"").unwrap());
    }

    #[test]
    fn suite_fips_aes256gcm_ida_e_volta() {
        let (chave_publica, chave_secreta) = gerar_par_de_chaves();
        let env = selar_com(Suite::Aes256Gcm, &chave_publica, b"edicao FIPS", b"banco").unwrap();
        assert_eq!(env[1], Suite::Aes256Gcm as u8);
        assert_eq!(abrir(&chave_secreta, &chave_publica, &env, b"banco").unwrap().as_slice(), b"edicao FIPS");
    }

    #[test]
    fn trocar_a_suite_no_cabecalho_falha() {
        let (chave_publica, chave_secreta) = gerar_par_de_chaves();
        let mut env = selar(&chave_publica, b"segredo", b"").unwrap();
        env[1] = Suite::Aes256Gcm as u8;
        assert!(abrir(&chave_secreta, &chave_publica, &env, b"").is_err());
    }

    #[test]
    fn chave_degenerada_e_recusada_ao_selar_e_ao_abrir() {
        let (p, s) = gerar_par_de_chaves();
        // Ponto zero do X25519 (ordem baixa) no lugar da chave do destinatário.
        let mut bytes = p.para_bytes();
        for b in bytes.iter_mut().take(TAM_X25519) { *b = 0; }
        assert_eq!(ChavePublica::de_bytes(&bytes).unwrap_err(), Erro::ChaveInvalida, "chave degenerada nem chega a existir");
        // Envelope com chave efêmera zerada.
        let mut env = selar(&p, b"x", b"").unwrap();
        for b in env.iter_mut().skip(2).take(TAM_X25519) { *b = 0; }
        assert_eq!(abrir(&s, &p, &env, b"").unwrap_err(), Erro::ChaveInvalida);
    }

    #[test]
    fn selar_recusa_conteudo_acima_do_limite() {
        let (p, _) = gerar_par_de_chaves();
        let grande = vec![0u8; TAM_MAXIMO + 1];
        assert_eq!(selar(&p, &grande, b"").unwrap_err(), Erro::GrandeDemais);
    }

    #[test]
    fn suite_entra_na_derivacao_da_chave() {
        // Separação de domínio: a mesma troca de chaves gera chaves diferentes para suítes diferentes.
        let (p, _) = gerar_par_de_chaves();
        let a = derivar_chave(Suite::XChaCha20Poly1305, &[1; 32], &[2; 32], &[3; 16], &[4; 32], &p);
        let b = derivar_chave(Suite::Aes256Gcm, &[1; 32], &[2; 32], &[3; 16], &[4; 32], &p);
        assert_ne!(*a, *b);
    }

    #[test]
    fn limite_de_tamanho_e_conferido_antes_de_alocar() {
        // Achado real do cargo-mutants (Rodada 23, 11/10/2026): as duas linhas abaixo só conferem
        // `tamanho_permitido` CONTRA O PRÓPRIO `TAM_MAXIMO` — se a conta `64 * 1024 * 1024` for mutada
        // (por exemplo para `64 * 1024 + 1024`), as duas continuam passando, porque passam a comparar
        // contra o novo valor errado, não contra 64 MiB de verdade. Por isso a linha extra abaixo, que
        // trava o valor FINAL (vindo de conta feita por fora: `python3 -c "64*1024*1024"`), não a fórmula.
        assert_eq!(TAM_MAXIMO, 67_108_864, "TAM_MAXIMO deveria ser exatamente 64 MiB");
        assert!(tamanho_permitido(TAM_MAXIMO));
        assert!(!tamanho_permitido(TAM_MAXIMO + 1));
    }

    /// Um remetente com app adulterado pode cifrar qualquer coisa, sem o preenchimento padrão.
    fn selar_sem_preenchimento(destinatario: &ChavePublica, claro: &[u8]) -> Vec<u8> {
        let suite = Suite::XChaCha20Poly1305;
        let efemera = XSecret::random_from_rng(OsRng);
        let eph_pub = XPublic::from(&efemera);
        let segredo_x = efemera.diffie_hellman(&destinatario.x);
        let (ct_kem, segredo_kem) = destinatario.kem.encapsulate(&mut OsRng).unwrap();
        let chave = derivar_chave(suite, segredo_kem.as_slice(), segredo_x.as_bytes(), ct_kem.as_slice(), eph_pub.as_bytes(), destinatario);
        let mut nonce = [0u8; TAM_NONCE];
        OsRng.fill_bytes(&mut nonce);
        let mut cab = vec![VERSAO, suite as u8];
        cab.extend_from_slice(eph_pub.as_bytes());
        cab.extend_from_slice(ct_kem.as_slice());
        cab.extend_from_slice(&nonce);
        let ct = cifrar(suite, &chave, &nonce, claro, &cab);
        let mut v = cab;
        v.extend_from_slice(&ct);
        v
    }

    #[test]
    fn remetente_malicioso_com_conteudo_fora_do_padrao_nao_derruba_o_app() {
        let (p, s) = gerar_par_de_chaves();
        // `[0, 0, 0, 5, 1, 2, 3, 4]` (achado do cargo-mutants, Rodada 23, 11/10/2026): prefixo diz "5 bytes
        // de conteúdo", mas só 4 bytes sobram depois do prefixo (buf tem 8 bytes no total). Nenhum dos
        // outros casos abaixo cobria esta folga específica (declarado só 1 a mais que o disponível) — e
        // sem cobertura, trocar o `-` por `+` na conta do limite sobrevivia: em vez de recusar com
        // `MalFormado`, o código tentaria ler `buf[4..9]` de um buffer de 8 bytes e ENTRARIA EM PÂNICO
        // (índice fora dos limites) — um envelope adulterado derrubando o processo que o abre, não só
        // sendo recusado de forma limpa.
        for claro in [vec![], vec![1u8], vec![0, 0, 0], vec![0xFF, 0xFF, 0xFF, 0xFF, 1, 2], vec![0, 0, 0, 9], vec![0, 0, 0, 5, 1, 2, 3, 4]] {
            assert_eq!(abrir(&s, &p, &selar_sem_preenchimento(&p, &claro), b"").unwrap_err(), Erro::MalFormado, "{claro:?}");
        }
        // E o caminho feliz com o mesmo construtor, para provar que o teste é realista.
        let mut ok = vec![0, 0, 0, 2, 7, 8];
        ok.resize(256, 0);
        assert_eq!(abrir(&s, &p, &selar_sem_preenchimento(&p, &ok), b"").unwrap().as_slice(), vec![7, 8]);
        // Borda exata (achado do cargo-mutants, mesma rodada): conteúdo vazio com o buffer no tamanho
        // MÍNIMO absoluto (so o prefixo de 4 bytes, dizendo "0 bytes depois dele") precisa ser ACEITO,
        // não recusado — até esta rodada nada testava esta borda, só carimbos de tamanho maior.
        assert!(abrir(&s, &p, &selar_sem_preenchimento(&p, &[0, 0, 0, 0]), b"").unwrap().is_empty());
    }

    #[test]
    fn chave_secreta_serializada_volta_igual_e_abre() {
        let (p, s) = gerar_par_de_chaves();
        let bytes = s.para_bytes();
        assert_eq!(bytes.len(), TAM_CHAVE_SECRETA);
        let s2 = ChaveSecreta::de_bytes(&bytes).unwrap();
        let env = selar(&p, b"segredo da empresa", b"").unwrap();
        assert_eq!(abrir(&s2, &p, &env, b"").unwrap().as_slice(), b"segredo da empresa");
        assert_eq!(ChaveSecreta::de_bytes(&bytes[..100]).unwrap_err(), Erro::MalFormado);
    }

    #[test]
    fn debug_nunca_mostra_a_chave_secreta() {
        let (_, s) = gerar_par_de_chaves();
        assert_eq!(format!("{s:?}"), "ChaveSecreta(<oculta>)");
    }

    #[test]
    fn debug_da_chave_publica_nao_despeja_bytes_crus() {
        // Achado do cargo-mutants (Rodada 23, 11/10/2026): só o `Debug` de `ChaveSecreta` tinha teste;
        // o de `ChavePublica` (linha 93) não tinha nenhum, e sobrevivia até trocar a função inteira por
        // um no-op. A chave pública não é segredo (pode ser publicada), então isto não é uma fuga de
        // segredo — mas é a mesma classe de função, e "sem teste" aqui só foi por falta de atenção, não
        // por decisão. Trava o texto exato para não despejar os bytes crus da chave em log/depuração.
        let (p, _) = gerar_par_de_chaves();
        assert_eq!(format!("{p:?}"), "ChavePublica(X25519 + ML-KEM-1024)");
    }

    #[test]
    fn abrir_confere_o_tamanho_minimo_exato() {
        // Achado da reconciliação pós-Rodada 23 (11/10/2026, ao investigar uma corrida de
        // cargo-mutants que tinha tirado o retrato do código ANTES das correções desta mesma
        // rodada — ver PENDENCIAS_E_DECISOES.md): `abrir_com_chave_simetrica_confere_o_tamanho_
        // minimo_exato` (acima) fechou a borda exata de `envelope.len() < TAM_CABECALHO_SIMETRICO
        // + 16` para a versão SIMÉTRICA, mas `abrir` (assimétrico) tem a MESMA forma de
        // conferência, na mesma linha de código (`envelope.len() < TAM_CABECALHO + 16`), e não
        // tinha o equivalente — sobrevivia a `<` por `==`, `<` por `<=` e `+` por `-` sem nenhum
        // teste reprovar. Não é a mesma linha de código (são funções diferentes), então o teste
        // acima não a cobria por acaso nem por construção; só por falta de atenção.
        let (p, s) = gerar_par_de_chaves();
        let env = selar(&p, b"x", b"").unwrap();
        let minimo = TAM_CABECALHO + 16;
        // 1 byte abaixo do mínimo: sempre malformado — a conferência de TAMANHO recusa antes de
        // qualquer tentativa de decifrar.
        assert_eq!(abrir(&s, &p, &env[..minimo - 1], b""), Err(Erro::MalFormado));
        // Exatamente no mínimo: passa a conferência de tamanho (não é `MalFormado` por isto);
        // ainda falha, mas por autenticação (o corpo cifrado real foi truncado), nunca por
        // tamanho — a distinção entre os dois motivos é exatamente o que a linha 288 decide.
        assert_eq!(abrir(&s, &p, &env[..minimo], b""), Err(Erro::FalhaAoDecifrar));
    }

    #[test]
    fn chave_publica_serializa() {
        let (chave_publica, chave_secreta) = gerar_par_de_chaves();
        let pk2 = ChavePublica::de_bytes(&chave_publica.para_bytes()).unwrap();
        let env = selar(&pk2, b"ola", b"").unwrap();
        assert_eq!(abrir(&chave_secreta, &chave_publica, &env, b"").unwrap().as_slice(), b"ola");
    }

    // ---- chave simétrica (sem par assimétrico) — ver `recuperacao::preparar_frase_pessoal` ----

    #[test]
    fn chave_simetrica_ida_e_volta() {
        let chave = [9u8; TAM_CHAVE_SIMETRICA];
        let env = selar_com_chave_simetrica(&chave, b"segredo da conta", b"ctx").unwrap();
        assert_eq!(abrir_com_chave_simetrica(&chave, &env, b"ctx").unwrap().as_slice(), b"segredo da conta");
    }

    #[test]
    fn chave_simetrica_errada_falha() {
        let chave = [9u8; TAM_CHAVE_SIMETRICA];
        let outra = [8u8; TAM_CHAVE_SIMETRICA];
        let env = selar_com_chave_simetrica(&chave, b"segredo", b"").unwrap();
        assert_eq!(abrir_com_chave_simetrica(&outra, &env, b""), Err(Erro::FalhaAoDecifrar));
    }

    #[test]
    fn chave_simetrica_aad_diferente_falha_e_qualquer_byte_alterado_falha() {
        let chave = [1u8; TAM_CHAVE_SIMETRICA];
        let env = selar_com_chave_simetrica(&chave, b"segredo", b"aad-a").unwrap();
        assert_eq!(abrir_com_chave_simetrica(&chave, &env, b"aad-b"), Err(Erro::FalhaAoDecifrar));
        for i in [0usize, 1, 10, TAM_CABECALHO_SIMETRICO - 1, TAM_CABECALHO_SIMETRICO, env.len() - 1] {
            let mut t = env.clone();
            t[i] ^= 0x01;
            assert!(abrir_com_chave_simetrica(&chave, &t, b"aad-a").is_err(), "byte {i} alterado deveria falhar");
        }
    }

    #[test]
    fn abrir_com_chave_simetrica_confere_o_tamanho_minimo_exato() {
        // Achado do cargo-mutants (Rodada 23, 11/10/2026): nada testava a borda exata de
        // `TAM_CABECALHO_SIMETRICO + 16` (cabeçalho + tag de autenticação, 16 bytes) — só a
        // fórmula podia mudar (`<` para `<=`, `+` para `-`) sem nenhum teste reprovar.
        let chave = [1u8; TAM_CHAVE_SIMETRICA];
        let env = selar_com_chave_simetrica(&chave, b"x", b"").unwrap();
        let minimo = TAM_CABECALHO_SIMETRICO + 16;
        // 1 byte abaixo do mínimo: sempre malformado — a conferência de TAMANHO recusa antes
        // de qualquer tentativa de decifrar.
        assert_eq!(abrir_com_chave_simetrica(&chave, &env[..minimo - 1], b""), Err(Erro::MalFormado));
        // Exatamente no mínimo: passa a conferência de tamanho (não é `MalFormado` por isto);
        // ainda falha, mas por autenticação (o corpo cifrado real foi truncado), nunca por
        // tamanho — a distinção entre os dois motivos é exatamente o que esta linha decide.
        assert_eq!(abrir_com_chave_simetrica(&chave, &env[..minimo], b""), Err(Erro::FalhaAoDecifrar));
    }

    #[test]
    fn chave_simetrica_mesma_mensagem_gera_envelopes_diferentes_e_cabecalho_bem_mais_curto_que_o_assimetrico() {
        let chave = [3u8; TAM_CHAVE_SIMETRICA];
        let a = selar_com_chave_simetrica(&chave, b"x", b"").unwrap();
        let b = selar_com_chave_simetrica(&chave, b"x", b"").unwrap();
        assert_ne!(a, b, "nonce aleatório a cada chamada");
        assert!(a.len() < TAM_CABECALHO, "sem KEM nem X25519: muito mais curto que o envelope assimétrico — é o que torna viável escrever a chave como frase");
    }

    #[test]
    fn chave_simetrica_recusa_conteudo_acima_do_limite_e_envelope_malformado() {
        let chave = [4u8; TAM_CHAVE_SIMETRICA];
        assert_eq!(selar_com_chave_simetrica(&chave, &vec![0u8; TAM_MAXIMO + 1], b"").unwrap_err(), Erro::GrandeDemais);
        assert_eq!(abrir_com_chave_simetrica(&chave, &[1, 2, 3], b""), Err(Erro::MalFormado));
    }
}

