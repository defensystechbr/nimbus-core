//! # Módulo `direto` — arquivo de Nimbus para Nimbus, fora da conversa (DIR1–DIR7)
//!
//! Para mandar um arquivo sem abrir conversa e sem deixar marca: o arquivo é inspecionado (`inspecao`), limpo
//! de metadados (`higiene`), cifrado para a chave de quem recebe com o nome por dentro (`envelope`) e entregue
//! UMA vez pelos retransmissores — nunca direto entre os aparelhos, para nenhum dos dois ver o IP do outro. O
//! servidor apaga ao entregar ou no prazo (no máximo 48 horas). Quem recebe inspeciona de novo: nunca confia no
//! aparelho de quem enviou.
//!
//! **NimbusDrop lacrado (ND2–ND5).** Por proximidade, o arquivo pode chegar sem internet, mas chega lacrado:
//! duas camadas — a de fora só a chave do destinatário abre (quem captar o Bluetooth não lê nada); a de dentro
//! usa uma chave de uso único que segue pelo canal normal do NimbusChat, cifrado de ponta a ponta. Sem essa
//! chave, que só chega pelo servidor, o arquivo não abre. Ao liberar, valem a hora confirmada pelo servidor
//! (`expiracao::hora_confiavel`), o prazo e o nível do aparelho: só o verde recebe por proximidade.
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use crate::{assinatura, higiene, inspecao, integridade};
use sha2::{Digest, Sha512};
use chacha20poly1305::{aead::{Aead, KeyInit, Payload}, XChaCha20Poly1305, XNonce};
use rand_core::{OsRng, RngCore};
use std::collections::HashMap;
use zeroize::Zeroizing;

const CONTEXTO: &[u8] = b"NimbusOne/v1/direto";
/// Prazo máximo de uma remessa esperando no servidor.
pub const PRAZO_MAXIMO: u64 = 48 * 3_600;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDireto { Recusado(inspecao::Motivo), NomeGrandeDemais, Cifra, Expirada, NaoEncontrada, Ilegivel,
    /// A assinatura de quem enviou não confere: o arquivo não abre (DIR8).
    RemetenteNaoConfere }

/// O que vai para o servidor: bloco ilegível, com prazo.
#[derive(Debug, Clone)]
pub struct Remessa { pub id: [u8; 16], pub bloco: Vec<u8>, pub valida_ate: u64 }

/// No aparelho de quem envia.
/// Inspeciona, limpa os metadados e monta nome + conteúdo. Usado pela remessa e pelo NimbusDrop lacrado.
fn conferir_e_montar(nome: &str, dados: &[u8]) -> Result<Zeroizing<Vec<u8>>, ErroDireto> {
    if let inspecao::Decisao::Bloquear(m) = inspecao::inspecionar(nome, dados).decisao { return Err(ErroDireto::Recusado(m)); }
    if nome.len() > u16::MAX as usize { return Err(ErroDireto::NomeGrandeDemais); }
    let limpo = higiene::limpar(dados).unwrap_or_else(|_| dados.to_vec());
    let mut claro = Zeroizing::new((nome.len() as u16).to_be_bytes().to_vec());
    claro.extend_from_slice(nome.as_bytes());
    claro.extend_from_slice(&limpo);
    Ok(claro)
}

/// O identificador e o prazo da remessa vão no dado autenticado: esticar o prazo invalida a remessa (v37, segunda rodada).
fn aad_da_remessa(id: &[u8; 16], valida_ate: u64) -> Vec<u8> { aad_com(CONTEXTO, id, valida_ate) }

fn aad_com(contexto: &[u8], id: &[u8; 16], valida_ate: u64) -> Vec<u8> {
    let mut aad = contexto.to_vec();
    aad.extend_from_slice(id);
    aad.extend_from_slice(&valida_ate.to_be_bytes());
    aad
}

pub fn preparar(destino: &ChavePublica, id: [u8; 16], nome: &str, dados: &[u8], agora: u64, prazo: u64) -> Result<Remessa, ErroDireto> {
    let claro = conferir_e_montar(nome, dados)?;
    let valida_ate = agora.saturating_add(prazo.min(PRAZO_MAXIMO));
    let bloco = envelope::selar(destino, &claro, &aad_da_remessa(&id, valida_ate)).map_err(|_| ErroDireto::Cifra)?;
    Ok(Remessa { id, bloco, valida_ate })
}

/// Arquivo recebido, já decifrado e inspecionado de novo.
#[derive(Debug)]
pub struct Recebido { pub nome: String, pub dados: Zeroizing<Vec<u8>>, pub laudo: inspecao::Laudo }

/// No aparelho de quem recebe.
pub fn abrir(sec: &ChaveSecreta, publ: &ChavePublica, r: &Remessa, agora: u64) -> Result<Recebido, ErroDireto> {
    if agora >= r.valida_ate { return Err(ErroDireto::Expirada); }
    let claro = envelope::abrir(sec, publ, &r.bloco, &aad_da_remessa(&r.id, r.valida_ate)).map_err(|_| ErroDireto::Ilegivel)?;
    if claro.len() < 2 { return Err(ErroDireto::Ilegivel); }
    let n = u16::from_be_bytes([claro[0], claro[1]]) as usize;
    if claro.len() < 2 + n { return Err(ErroDireto::Ilegivel); }
    let nome = core::str::from_utf8(&claro[2..2 + n]).map_err(|_| ErroDireto::Ilegivel)?.to_string();
    let dados = Zeroizing::new(claro[2 + n..].to_vec());
    let laudo = inspecao::inspecionar(&nome, &dados);
    if let inspecao::Decisao::Bloquear(m) = laudo.decisao { return Err(ErroDireto::Recusado(m)); }
    Ok(Recebido { nome, dados, laudo })
}

/// Servidor: entrega uma vez só e apaga.
#[derive(Debug, Default)]
pub struct Caixa { remessas: HashMap<[u8; 16], Remessa> }

impl Caixa {
    pub fn depositar(&mut self, r: Remessa) { let _ = self.remessas.insert(r.id, r); }
    pub fn retirar(&mut self, id: &[u8; 16], agora: u64) -> Result<Remessa, ErroDireto> {
        let r = self.remessas.remove(id).ok_or(ErroDireto::NaoEncontrada)?;
        if agora >= r.valida_ate { return Err(ErroDireto::Expirada); }
        Ok(r)
    }
    pub fn limpar(&mut self, agora: u64) { self.remessas.retain(|_, r| agora < r.valida_ate); }
    pub fn quantidade(&self) -> usize { self.remessas.len() }
}

// ---------------------------------------------------------------------------------------------------------
// Remessa assinada pelo remetente (DIR8). O envelope é anônimo por construção: qualquer pessoa que tenha a
// chave pública de quem recebe consegue montar um. Por isso o app só mostra "enviado por" e só abre o
// arquivo quando a assinatura de quem enviou confere — e a assinatura cobre o destinatário, para ninguém
// reaproveitar um arquivo assinado mandando-o a outra pessoa.
// ---------------------------------------------------------------------------------------------------------

const CONTEXTO_REMETENTE: &[u8] = b"NimbusOne/v1/direto-remetente";
const CONTEXTO_ASSINADA: &[u8] = b"NimbusOne/v1/direto-assinada";
const MAGICO_ASSINADA: &[u8; 4] = b"NDA1";

fn aad_assinada(id: &[u8; 16], valida_ate: u64) -> Vec<u8> { aad_com(CONTEXTO_ASSINADA, id, valida_ate) }

fn mensagem_do_remetente(id: &[u8; 16], valida_ate: u64, destino: &ChavePublica, claro: &[u8]) -> Vec<u8> {
    let mut m = id.to_vec();
    m.extend_from_slice(&valida_ate.to_be_bytes());
    m.extend_from_slice(&Sha512::digest(destino.para_bytes()));
    m.extend_from_slice(&Sha512::digest(claro));
    m
}

/// No aparelho de quem envia: inspeciona, limpa, assina (cobrindo destinatário, id e prazo) e cifra.
pub fn preparar_assinado(remetente: &assinatura::ChaveDeAssinatura, destino: &ChavePublica, id: [u8; 16], nome: &str,
                         dados: &[u8], agora: u64, prazo: u64) -> Result<Remessa, ErroDireto> {
    let claro = conferir_e_montar(nome, dados)?;
    let valida_ate = agora.saturating_add(prazo.min(PRAZO_MAXIMO));
    let assinatura = remetente.assinar(CONTEXTO_REMETENTE, &mensagem_do_remetente(&id, valida_ate, destino, &claro))
        .map_err(|_| ErroDireto::Cifra)?;
    let vk = remetente.verificacao().para_bytes();
    let (ed, pq) = assinatura.partes();
    if vk.len() > u16::MAX as usize || pq.len() > u16::MAX as usize { return Err(ErroDireto::Cifra); }
    let mut interno = Zeroizing::new(MAGICO_ASSINADA.to_vec());
    interno.extend_from_slice(&(vk.len() as u16).to_be_bytes());
    interno.extend_from_slice(&vk);
    interno.extend_from_slice(&ed);
    interno.extend_from_slice(&(pq.len() as u16).to_be_bytes());
    interno.extend_from_slice(pq);
    interno.extend_from_slice(&claro);
    let bloco = envelope::selar(destino, &interno, &aad_assinada(&id, valida_ate)).map_err(|_| ErroDireto::Cifra)?;
    Ok(Remessa { id, bloco, valida_ate })
}

/// No aparelho de quem recebe: só abre se a assinatura de quem enviou conferir para ESTE destinatário.
/// Devolve também a chave de verificação de quem enviou: o app confere a impressão digital contra os
/// contatos; se não for um contato, mostra "remetente desconhecido" e não abre.
pub fn abrir_assinado(sec: &ChaveSecreta, publ: &ChavePublica, r: &Remessa, agora: u64)
                      -> Result<(Recebido, assinatura::ChaveDeVerificacao), ErroDireto> {
    if agora >= r.valida_ate { return Err(ErroDireto::Expirada); }
    let interno = envelope::abrir(sec, publ, &r.bloco, &aad_assinada(&r.id, r.valida_ate)).map_err(|_| ErroDireto::Ilegivel)?;
    let mut i = 0usize;
    let pegar = |i: &mut usize, n: usize| -> Result<&[u8], ErroDireto> {
        let fim = i.checked_add(n).ok_or(ErroDireto::Ilegivel)?;
        let s = interno.get(*i..fim).ok_or(ErroDireto::Ilegivel)?;
        *i = fim;
        Ok(s)
    };
    if pegar(&mut i, 4)? != MAGICO_ASSINADA { return Err(ErroDireto::Ilegivel); }
    let nvk = pegar(&mut i, 2)?; let nvk = u16::from_be_bytes([nvk[0], nvk[1]]) as usize;
    let vk = assinatura::ChaveDeVerificacao::de_bytes(pegar(&mut i, nvk)?).map_err(|_| ErroDireto::RemetenteNaoConfere)?;
    let mut ed = [0u8; 64]; ed.copy_from_slice(pegar(&mut i, 64)?);
    let npq = pegar(&mut i, 2)?; let npq = u16::from_be_bytes([npq[0], npq[1]]) as usize;
    let pq = pegar(&mut i, npq)?.to_vec();
    let claro = interno.get(i..).ok_or(ErroDireto::Ilegivel)?;
    let a = assinatura::AssinaturaHibrida::de_partes(ed, pq);
    if !vk.verificar(CONTEXTO_REMETENTE, &mensagem_do_remetente(&r.id, r.valida_ate, publ, claro), &a) {
        return Err(ErroDireto::RemetenteNaoConfere);
    }
    if claro.len() < 2 { return Err(ErroDireto::Ilegivel); }
    let n = u16::from_be_bytes([claro[0], claro[1]]) as usize;
    if claro.len() < 2 + n { return Err(ErroDireto::Ilegivel); }
    let nome = core::str::from_utf8(&claro[2..2 + n]).map_err(|_| ErroDireto::Ilegivel)?.to_string();
    let dados = Zeroizing::new(claro[2 + n..].to_vec());
    let laudo = inspecao::inspecionar(&nome, &dados);
    if let inspecao::Decisao::Bloquear(m) = laudo.decisao { return Err(ErroDireto::Recusado(m)); }
    Ok((Recebido { nome, dados, laudo }, vk))
}

// ---------------------------------------------------------------------------------------------------------
// NimbusDrop lacrado: chega por proximidade, abre só depois de o servidor entregar a chave (ND2–ND5).
// ---------------------------------------------------------------------------------------------------------

const CONTEXTO_LACRE: &[u8] = b"NimbusOne/v1/nimbusdrop-lacre";

/// Chave de uso único da camada de dentro. Viaja só pelo canal cifrado ponta a ponta, via servidor.
pub struct ChaveDoLacre(Zeroizing<[u8; 32]>);
impl ChaveDoLacre {
    pub fn de_bytes(b: [u8; 32]) -> Self { ChaveDoLacre(Zeroizing::new(b)) }
    /// Para colocar dentro da mensagem cifrada ponta a ponta. Nunca vai pela proximidade.
    pub fn exportar(&self) -> Zeroizing<[u8; 32]> { Zeroizing::new(*self.0) }
}
impl core::fmt::Debug for ChaveDoLacre {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { f.write_str("ChaveDoLacre(32 bytes)") }
}

/// O que passa por proximidade: ilegível sem a chave do destinatário E sem a chave do lacre.
#[derive(Debug, Clone)]
pub struct Lacrado { pub id: [u8; 16], pub bloco: Vec<u8>, pub valida_ate: u64 }

/// Por que o arquivo recebido por proximidade ainda não abre.
#[derive(Debug, PartialEq, Eq)]
pub enum MotivoDoLacre {
    /// Aparelho amarelo ou vermelho: NimbusDrop só no verde.
    AparelhoNaoVerde,
    /// Sem hora confirmada pelo servidor: "Recebido. Abre quando houver conexão."
    SemHoraConfirmada,
    /// A chave do lacre ainda não chegou pelo servidor.
    ChaveAindaNaoChegou,
    /// Prazo vencido pela hora do servidor: não abre nunca mais.
    Vencido,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDoLacre { Lacre(MotivoDoLacre), Direto(ErroDireto) }

/// O identificador e o prazo vão no dado autenticado das duas camadas: alterar qualquer um invalida o lacre.
fn aad_do_lacre(id: &[u8; 16], valida_ate: u64) -> Vec<u8> {
    let mut a = CONTEXTO_LACRE.to_vec();
    a.extend_from_slice(id);
    a.extend_from_slice(&valida_ate.to_be_bytes());
    a
}

/// No aparelho de quem envia: inspeciona, limpa, lacra. Devolve o que vai por proximidade e a chave que vai
/// pelo canal cifrado.
pub fn lacrar(destino: &ChavePublica, id: [u8; 16], nome: &str, dados: &[u8], agora: u64, prazo: u64) -> Result<(Lacrado, ChaveDoLacre), ErroDireto> {
    let claro = conferir_e_montar(nome, dados)?;
    let mut k = [0u8; 32];
    OsRng.fill_bytes(&mut k);
    let chave = ChaveDoLacre::de_bytes(k);
    k.fill(0);
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let valida_ate = agora.saturating_add(prazo.min(PRAZO_MAXIMO));
    let aad = aad_do_lacre(&id, valida_ate);
    let c = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&chave.0[..]));
    let mut interno = Zeroizing::new(nonce.to_vec());
    interno.extend(c.encrypt(XNonce::from_slice(&nonce), Payload { msg: &claro, aad: &aad }).map_err(|_| ErroDireto::Cifra)?);
    let bloco = envelope::selar(destino, &interno, &aad).map_err(|_| ErroDireto::Cifra)?;
    Ok((Lacrado { id, bloco, valida_ate }, chave))
}

/// Pode abrir? Decide sem tocar na cifra. A ordem importa: aparelho, hora, chave, prazo.
pub fn pode_abrir_lacrado(l: &Lacrado, chave_chegou: bool, hora_confiavel: Option<u64>, nivel: integridade::Nivel) -> Result<(), MotivoDoLacre> {
    if !integridade::permissoes(nivel).receber_por_proximidade { return Err(MotivoDoLacre::AparelhoNaoVerde); }
    let hora = hora_confiavel.ok_or(MotivoDoLacre::SemHoraConfirmada)?;
    if !chave_chegou { return Err(MotivoDoLacre::ChaveAindaNaoChegou); }
    if hora >= l.valida_ate { return Err(MotivoDoLacre::Vencido); }
    Ok(())
}

/// No aparelho de quem recebe: só abre com as duas chaves, hora confirmada, dentro do prazo e no verde.
pub fn abrir_lacrado(sec: &ChaveSecreta, publ: &ChavePublica, l: &Lacrado, chave: Option<&ChaveDoLacre>,
                     hora_confiavel: Option<u64>, nivel: integridade::Nivel) -> Result<Recebido, ErroDoLacre> {
    pode_abrir_lacrado(l, chave.is_some(), hora_confiavel, nivel).map_err(ErroDoLacre::Lacre)?;
    let chave = chave.ok_or(ErroDoLacre::Lacre(MotivoDoLacre::ChaveAindaNaoChegou))?;
    let aad = aad_do_lacre(&l.id, l.valida_ate);
    let interno = envelope::abrir(sec, publ, &l.bloco, &aad).map_err(|_| ErroDoLacre::Direto(ErroDireto::Ilegivel))?;
    if interno.len() < 24 + 16 { return Err(ErroDoLacre::Direto(ErroDireto::Ilegivel)); }
    let c = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&chave.0[..]));
    let claro = Zeroizing::new(c.decrypt(XNonce::from_slice(&interno[..24]), Payload { msg: &interno[24..], aad: &aad })
        .map_err(|_| ErroDoLacre::Direto(ErroDireto::Ilegivel))?);
    if claro.len() < 2 { return Err(ErroDoLacre::Direto(ErroDireto::Ilegivel)); }
    let n = u16::from_be_bytes([claro[0], claro[1]]) as usize;
    if claro.len() < 2 + n { return Err(ErroDoLacre::Direto(ErroDireto::Ilegivel)); }
    let nome = core::str::from_utf8(&claro[2..2 + n]).map_err(|_| ErroDoLacre::Direto(ErroDireto::Ilegivel))?.to_string();
    let dados = Zeroizing::new(claro[2 + n..].to_vec());
    let laudo = inspecao::inspecionar(&nome, &dados);
    if let inspecao::Decisao::Bloquear(m) = laudo.decisao { return Err(ErroDoLacre::Direto(ErroDireto::Recusado(m))); }
    Ok(Recebido { nome, dados, laudo })
}

#[cfg(test)]
mod testes {
    use super::*;

    fn foto() -> Vec<u8> { let mut j = vec![0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x0A]; j.extend_from_slice(b"GPS-SEGR"); j.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x04, 0x01, 0x00, 0x11, 0xFF, 0xD9]); j }

    #[test]
    fn envio_direto_chega_uma_vez_so_sem_metadados() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let mut caixa = Caixa::default();
        caixa.depositar(preparar(&p, [1; 16], "praia.jpg", &foto(), 100, 3_600).unwrap());
        let r = caixa.retirar(&[1; 16], 200).unwrap();
        let recebido = abrir(&s, &p, &r, 200).unwrap();
        assert_eq!(recebido.nome, "praia.jpg");
        assert!(!recebido.dados.windows(3).any(|w| w == b"GPS"), "sai sem GPS");
        assert_eq!(caixa.retirar(&[1; 16], 201).unwrap_err(), ErroDireto::NaoEncontrada, "segunda retirada: não existe mais");
    }

    #[test]
    fn programa_nao_sai_nem_entra() {
        let (p, _) = envelope::gerar_par_de_chaves();
        assert_eq!(preparar(&p, [2; 16], "foto.jpg", b"MZ\x90", 0, 60).unwrap_err(), ErroDireto::Recusado(inspecao::Motivo::Programa));
    }

    #[test]
    fn prazo_nunca_passa_de_48_horas_e_vencida_nao_abre() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let r = preparar(&p, [3; 16], "nota.txt", b"ok", 0, 30 * 86_400).unwrap();
        assert_eq!(r.valida_ate, PRAZO_MAXIMO);
        assert_eq!(abrir(&s, &p, &r, PRAZO_MAXIMO).unwrap_err(), ErroDireto::Expirada);
        let mut caixa = Caixa::default();
        caixa.depositar(r);
        caixa.limpar(PRAZO_MAXIMO);
        assert_eq!(caixa.quantidade(), 0);
    }

    #[test]
    fn remessa_para_outra_pessoa_ou_trocada_nao_abre() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let (p2, s2) = envelope::gerar_par_de_chaves();
        let r = preparar(&p, [4; 16], "nota.txt", b"segredo", 0, 60).unwrap();
        assert_eq!(abrir(&s2, &p2, &r, 1).unwrap_err(), ErroDireto::Ilegivel);
        let mut trocada = preparar(&p2, [5; 16], "nota.txt", b"x", 0, 60).unwrap();
        trocada.id = [6; 16];
        assert_eq!(abrir(&s2, &p2, &trocada, 1).unwrap_err(), ErroDireto::Ilegivel, "id trocado no caminho: recusado");
    }

    #[test]
    fn esticar_o_prazo_da_remessa_invalida() {
        let (p, s_) = envelope::gerar_par_de_chaves();
        let mut r = preparar(&p, [21; 16], "a.txt", b"conteudo de teste simples", 1_000, 60).unwrap();
        r.valida_ate += 100_000;
        assert_eq!(abrir(&s_, &p, &r, 1_070).unwrap_err(), ErroDireto::Ilegivel);
    }

    mod assinada {
        use super::super::*;
        use crate::{assinatura, envelope};
        const T: &[u8] = b"relatorio trimestral em texto simples";

        #[test]
        fn remetente_confere_e_e_devolvido() {
            let (p, s_) = envelope::gerar_par_de_chaves();
            let ana = assinatura::gerar();
            let r = preparar_assinado(&ana, &p, [1; 16], "a.txt", T, 1_000, 60).unwrap();
            let (rec, vk) = abrir_assinado(&s_, &p, &r, 1_010).unwrap();
            assert_eq!(&rec.dados[..], T);
            assert_eq!(vk.impressao_digital(), ana.verificacao().impressao_digital());
        }

        #[test]
        fn quem_so_tem_a_chave_publica_nao_se_passa_por_contato() {
            // Um estranho monta uma remessa sem assinatura válida: o envelope abre, a assinatura não.
            let (p, s_) = envelope::gerar_par_de_chaves();
            let sem = preparar(&p, [2; 16], "a.txt", T, 1_000, 60).unwrap();
            assert!(abrir_assinado(&s_, &p, &sem, 1_010).is_err(), "remessa sem assinatura não abre pelo caminho assinado");
        }

        #[test]
        fn arquivo_assinado_para_um_nao_serve_para_outro() {
            let (p1, _s1) = envelope::gerar_par_de_chaves();
            let (p2, s2) = envelope::gerar_par_de_chaves();
            let ana = assinatura::gerar();
            let r1 = preparar_assinado(&ana, &p1, [3; 16], "a.txt", T, 1_000, 60).unwrap();
            // Quem recebeu reabre e recifra o mesmo conteúdo assinado para outra pessoa: a assinatura cobre o destinatário.
            let interno = {
                // reconstrói o interno que o primeiro destinatário teria visto
                let s1 = _s1; envelope::abrir(&s1, &p1, &r1.bloco, &aad_assinada(&r1.id, r1.valida_ate)).unwrap()
            };
            let reenviada = Remessa { id: r1.id, valida_ate: r1.valida_ate, bloco: envelope::selar(&p2, &interno, &aad_assinada(&r1.id, r1.valida_ate)).unwrap() };
            assert_eq!(abrir_assinado(&s2, &p2, &reenviada, 1_010).unwrap_err(), ErroDireto::RemetenteNaoConfere);
        }

        #[test]
        fn esticar_prazo_ou_trocar_assinatura_nao_abre() {
            let (p, s_) = envelope::gerar_par_de_chaves();
            let ana = assinatura::gerar();
            let mut r = preparar_assinado(&ana, &p, [4; 16], "a.txt", T, 1_000, 60).unwrap();
            assert!(abrir_assinado(&s_, &p, &r, 1_060).is_err(), "vencida");
            r.valida_ate += 1_000;
            assert_eq!(abrir_assinado(&s_, &p, &r, 1_010).unwrap_err(), ErroDireto::Ilegivel);
        }

        #[test]
        fn formato_sem_a_marca_de_assinada_e_recusado() {
            let (p, s_) = envelope::gerar_par_de_chaves();
            let ana = assinatura::gerar();
            let r = preparar_assinado(&ana, &p, [6; 16], "a.txt", T, 1_000, 60).unwrap();
            let mut interno = envelope::abrir(&s_, &p, &r.bloco, &aad_assinada(&r.id, r.valida_ate)).unwrap();
            interno[..4].copy_from_slice(b"XXXX");
            let r2 = Remessa { id: r.id, valida_ate: r.valida_ate, bloco: envelope::selar(&p, &interno, &aad_assinada(&r.id, r.valida_ate)).unwrap() };
            assert_eq!(abrir_assinado(&s_, &p, &r2, 1_010).unwrap_err(), ErroDireto::Ilegivel);
        }

        #[test]
        fn interno_malformado_nao_derruba() {
            let (p, s_) = envelope::gerar_par_de_chaves();
            for lixo in [vec![], b"NDA1".to_vec(), b"NDA1\xff\xff".to_vec(), b"XXXXabc".to_vec()] {
                let r = Remessa { id: [5; 16], valida_ate: 2_000, bloco: envelope::selar(&p, &lixo, &aad_assinada(&[5; 16], 2_000)).unwrap() };
                assert!(abrir_assinado(&s_, &p, &r, 1_000).is_err());
            }
        }
    }

    mod lacre {
        use super::super::*;
        use crate::envelope;
        use crate::expiracao::hora_confiavel;
        use crate::integridade::Nivel;

        fn par() -> (ChaveSecreta, ChavePublica) { let (p, s) = envelope::gerar_par_de_chaves(); (s, p) }
        const TEXTO: &[u8] = b"relatorio trimestral em texto simples";

        #[test]
        fn abre_com_as_duas_chaves_hora_confirmada_e_verde() {
            let (sec, publ) = par();
            let (l, k) = lacrar(&publ, [1; 16], "rel.txt", TEXTO, 1_000, 3_600).unwrap();
            let r = abrir_lacrado(&sec, &publ, &l, Some(&k), hora_confiavel(1_010, 1_000, Some(10)), Nivel::Verde).unwrap();
            assert_eq!(&r.dados[..], TEXTO);
            assert_eq!(r.nome, "rel.txt");
        }

        #[test]
        fn sem_internet_nao_abre() {
            let (sec, publ) = par();
            let (l, k) = lacrar(&publ, [2; 16], "rel.txt", TEXTO, 1_000, 3_600).unwrap();
            // Com a chave em mãos, mas sem hora confirmada: continua lacrado.
            assert_eq!(abrir_lacrado(&sec, &publ, &l, Some(&k), None, Nivel::Verde).unwrap_err(), ErroDoLacre::Lacre(MotivoDoLacre::SemHoraConfirmada));
            // Com hora, mas a chave ainda não chegou pelo servidor.
            assert_eq!(abrir_lacrado(&sec, &publ, &l, None, Some(1_010), Nivel::Verde).unwrap_err(), ErroDoLacre::Lacre(MotivoDoLacre::ChaveAindaNaoChegou));
            assert_eq!(pode_abrir_lacrado(&l, false, Some(1_010), Nivel::Verde), Err(MotivoDoLacre::ChaveAindaNaoChegou));
            assert_eq!(pode_abrir_lacrado(&l, true, None, Nivel::Verde), Err(MotivoDoLacre::SemHoraConfirmada));
        }

        #[test]
        fn mudar_a_hora_do_celular_nao_abre_vencido() {
            let (sec, publ) = par();
            let (l, k) = lacrar(&publ, [3; 16], "rel.txt", TEXTO, 1_000, 60).unwrap();
            // Servidor diz 1_100 (vencido às 1_060); o celular foi atrasado para 1_000.
            let h = hora_confiavel(1_000, 1_100, Some(0));
            assert_eq!(abrir_lacrado(&sec, &publ, &l, Some(&k), h, Nivel::Verde).unwrap_err(), ErroDoLacre::Lacre(MotivoDoLacre::Vencido));
            assert_eq!(pode_abrir_lacrado(&l, true, Some(1_059), Nivel::Verde), Ok(()));
            assert_eq!(pode_abrir_lacrado(&l, true, Some(1_060), Nivel::Verde), Err(MotivoDoLacre::Vencido));
        }

        #[test]
        fn so_o_verde_recebe_por_proximidade() {
            let (sec, publ) = par();
            let (l, k) = lacrar(&publ, [4; 16], "rel.txt", TEXTO, 1_000, 3_600).unwrap();
            for n in [Nivel::Amarelo, Nivel::Vermelho] {
                assert_eq!(abrir_lacrado(&sec, &publ, &l, Some(&k), Some(1_010), n).unwrap_err(), ErroDoLacre::Lacre(MotivoDoLacre::AparelhoNaoVerde));
            }
        }

        #[test]
        fn quem_capta_o_bluetooth_nao_le_e_chave_errada_nao_abre() {
            let (sec, publ) = par();
            let (outro_sec, outro_publ) = par();
            let (l, k) = lacrar(&publ, [5; 16], "rel.txt", TEXTO, 1_000, 3_600).unwrap();
            assert!(!l.bloco.windows(TEXTO.len()).any(|w| w == TEXTO), "nada em claro no bloco");
            assert_eq!(abrir_lacrado(&outro_sec, &outro_publ, &l, Some(&k), Some(1_010), Nivel::Verde).unwrap_err(), ErroDoLacre::Direto(ErroDireto::Ilegivel));
            let errada = ChaveDoLacre::de_bytes([7; 32]);
            assert_eq!(abrir_lacrado(&sec, &publ, &l, Some(&errada), Some(1_010), Nivel::Verde).unwrap_err(), ErroDoLacre::Direto(ErroDireto::Ilegivel));
        }

        #[test]
        fn esticar_o_prazo_do_lacre_invalida_o_arquivo() {
            // Quem recebe (ou quem está no caminho) altera o prazo que viaja junto: o lacre deixa de abrir.
            let (sec, publ) = par();
            let (mut l, k) = lacrar(&publ, [11; 16], "rel.txt", TEXTO, 1_000, 60).unwrap();
            l.valida_ate += 10_000;
            assert_eq!(abrir_lacrado(&sec, &publ, &l, Some(&k), Some(1_070), Nivel::Verde).unwrap_err(), ErroDoLacre::Direto(ErroDireto::Ilegivel));
        }

        #[test]
        fn trocar_o_id_invalida_o_lacre() {
            let (sec, publ) = par();
            let (mut l, k) = lacrar(&publ, [6; 16], "rel.txt", TEXTO, 1_000, 3_600).unwrap();
            l.id = [9; 16];
            assert!(abrir_lacrado(&sec, &publ, &l, Some(&k), Some(1_010), Nivel::Verde).is_err());
        }

        #[test]
        fn programa_disfarcado_e_recusado_antes_de_lacrar() {
            let (_sec, publ) = par();
            assert!(matches!(lacrar(&publ, [8; 16], "foto.jpg", b"MZ\x90\x00programa", 1_000, 60), Err(ErroDireto::Recusado(_))));
        }

        #[test]
        fn prazo_nunca_passa_de_48_horas_e_chave_nao_aparece_na_depuracao() {
            let (_sec, publ) = par();
            let (l, k) = lacrar(&publ, [10; 16], "rel.txt", TEXTO, 1_000, 999_999).unwrap();
            assert_eq!(l.valida_ate, 1_000 + PRAZO_MAXIMO);
            assert_eq!(format!("{k:?}"), "ChaveDoLacre(32 bytes)");
        }
    }
}
