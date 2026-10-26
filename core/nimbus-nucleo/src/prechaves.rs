//! # Módulo `prechaves` — chaves de recebimento que se destroem (sigilo futuro fora da libsignal) (FS1–FS5)
//!
//! As conversas usam a libsignal, que já troca de chave a cada mensagem. Mas o envio direto, o NimbusDrop e
//! outros canais selam com `envelope`, para a chave pública do destinatário. Se essa chave fosse de longo
//! prazo, quem gravasse os envelopes hoje e roubasse o aparelho amanhã leria tudo de volta.
//!
//! Aqui, cada aparelho publica **chaves de recebimento** de vida curta (no máximo 7 dias), assinadas pela
//! identidade. Quem envia sela para a mais nova; quem recebe guarda a parte secreta só até o prazo e a
//! **destrói** em seguida (`Chaveiro::apagar_vencidas`). Depois disso, nem o próprio dono — nem quem tomar o
//! aparelho, nem quem tiver gravado o tráfego — consegue abrir os envelopes antigos.
//!
//! Regra prática: chave nova todo dia; a secreta vive até o fim da validade dela mais 48 h (o prazo máximo
//! de qualquer mensagem). Passado isso, some.

use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use zeroize::Zeroizing;

const CONTEXTO: &[u8] = b"NimbusOne/v1/prechave";
const CONTEXTO_AAD: &[u8] = b"NimbusOne/v1/prechave/aad";

/// Validade máxima de uma chave de recebimento publicada: 7 dias.
pub const VALIDADE_MAXIMA: u64 = 7 * 24 * 3600;
/// Depois do fim da validade, a secreta ainda vive 48 h (prazo máximo de uma mensagem) e então é destruída.
pub const SOBREVIDA_DA_SECRETA: u64 = 48 * 3600;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDePrechave {
    /// Assinatura da identidade não confere: chave plantada por outra pessoa (ou pelo servidor).
    AssinaturaInvalida,
    /// Vencida ou com validade acima do máximo.
    ForaDoPrazo,
    /// Chave pública malformada.
    Malformada,
    /// A secreta correspondente já foi destruída (ou nunca existiu neste aparelho).
    ChaveDestruida,
    /// O envelope não abre.
    Ilegivel,
}

/// O que vai para o servidor: a parte pública, o prazo e a assinatura da identidade.
#[derive(Debug, Clone)]
pub struct PrechavePublicada { pub id: [u8; 16], pub publica: Vec<u8>, pub valida_ate: u64, pub assinatura: AssinaturaHibrida }

/// O que fica no aparelho até ser destruído.
pub struct PrechaveSecreta { id: [u8; 16], publica: ChavePublica, secreta: ChaveSecreta, valida_ate: u64 }

impl core::fmt::Debug for PrechaveSecreta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "PrechaveSecreta(<oculta>, até {})", self.valida_ate) }
}

fn bytes_para_assinar(id: &[u8; 16], publica: &[u8], valida_ate: u64) -> Vec<u8> {
    let mut v = id.to_vec();
    v.extend_from_slice(&valida_ate.to_be_bytes());
    v.extend_from_slice(publica);
    v
}

fn aad(id: &[u8; 16], valida_ate: u64, aad_do_uso: &[u8]) -> Vec<u8> {
    let mut v = CONTEXTO_AAD.to_vec();
    v.extend_from_slice(id);
    v.extend_from_slice(&valida_ate.to_be_bytes());
    v.extend_from_slice(aad_do_uso);
    v
}

/// No aparelho: gera uma chave de recebimento, assinada pela identidade.
pub fn gerar(identidade: &ChaveDeAssinatura, id: [u8; 16], agora: u64, validade: u64) -> (PrechavePublicada, PrechaveSecreta) {
    let valida_ate = agora.saturating_add(validade.min(VALIDADE_MAXIMA));
    let (publica, secreta) = envelope::gerar_par_de_chaves();
    let bytes = publica.para_bytes();
    let assinatura = identidade.assinar(CONTEXTO, &bytes_para_assinar(&id, &bytes, valida_ate)).expect("contexto fixo e curto");
    (PrechavePublicada { id, publica: bytes, valida_ate, assinatura }, PrechaveSecreta { id, publica, secreta, valida_ate })
}

/// Quem envia: confere a assinatura da identidade do destinatário e o prazo antes de usar a chave.
/// Assim o servidor não consegue trocar a chave de recebimento por uma dele.
pub fn conferir(identidade: &ChaveDeVerificacao, p: &PrechavePublicada, agora: u64) -> Result<ChavePublica, ErroDePrechave> {
    if !identidade.verificar(CONTEXTO, &bytes_para_assinar(&p.id, &p.publica, p.valida_ate), &p.assinatura) {
        return Err(ErroDePrechave::AssinaturaInvalida);
    }
    if agora >= p.valida_ate || p.valida_ate > agora.saturating_add(VALIDADE_MAXIMA) { return Err(ErroDePrechave::ForaDoPrazo); }
    ChavePublica::de_bytes(&p.publica).map_err(|_| ErroDePrechave::Malformada)
}

/// Quem envia: sela para a chave de recebimento já conferida. O id e o prazo vão no dado autenticado.
pub fn selar(p: &PrechavePublicada, conferida: &ChavePublica, texto: &[u8], aad_do_uso: &[u8]) -> Result<Vec<u8>, ErroDePrechave> {
    envelope::selar(conferida, texto, &aad(&p.id, p.valida_ate, aad_do_uso)).map_err(|_| ErroDePrechave::Ilegivel)
}

/// As chaves secretas de recebimento deste aparelho. Destruir = largar a chave (ela se zera sozinha).
#[derive(Debug, Default)]
pub struct Chaveiro { chaves: Vec<PrechaveSecreta> }

impl Chaveiro {
    pub fn guardar(&mut self, s: PrechaveSecreta) { self.chaves.push(s); }
    pub fn quantidade(&self) -> usize { self.chaves.len() }

    /// Abre um envelope selado para a chave `id`. Se a chave já foi destruída, não há como abrir.
    pub fn abrir(&self, id: &[u8; 16], envelope: &[u8], aad_do_uso: &[u8]) -> Result<Zeroizing<Vec<u8>>, ErroDePrechave> {
        let k = self.chaves.iter().find(|k| &k.id == id).ok_or(ErroDePrechave::ChaveDestruida)?;
        envelope::abrir(&k.secreta, &k.publica, envelope, &aad(&k.id, k.valida_ate, aad_do_uso))
            .map_err(|_| ErroDePrechave::Ilegivel)
    }

    /// Destrói as chaves vencidas há mais de 48 h. `agora` vem de `expiracao::hora_confiavel`.
    /// Devolve quantas foram destruídas.
    pub fn apagar_vencidas(&mut self, agora: u64) -> usize {
        let antes = self.chaves.len();
        self.chaves.retain(|k| agora < k.valida_ate.saturating_add(SOBREVIDA_DA_SECRETA));
        antes - self.chaves.len()
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    #[test]
    fn envelope_antigo_fica_ilegivel_para_sempre_depois_da_destruicao() {
        let bruno = assinatura::gerar();
        let mut chaveiro = Chaveiro::default();
        let (publicada, secreta) = gerar(&bruno, [1; 16], 1_000, 24 * 3600);
        chaveiro.guardar(secreta);
        // Ana confere e sela.
        let k = conferir(&bruno.verificacao(), &publicada, 1_100).unwrap();
        let env = selar(&publicada, &k, b"conteudo sigiloso", b"direto").unwrap();
        assert_eq!(&chaveiro.abrir(&[1; 16], &env, b"direto").unwrap()[..], b"conteudo sigiloso");
        // Passou o prazo + 48 h: a secreta é destruída. Quem tomar o aparelho depois não abre o que gravou antes.
        assert_eq!(chaveiro.apagar_vencidas(1_000 + 24 * 3600 + SOBREVIDA_DA_SECRETA - 1), 0);
        assert_eq!(chaveiro.apagar_vencidas(1_000 + 24 * 3600 + SOBREVIDA_DA_SECRETA), 1);
        assert_eq!(chaveiro.abrir(&[1; 16], &env, b"direto").unwrap_err(), ErroDePrechave::ChaveDestruida);
    }

    #[test]
    fn servidor_nao_troca_a_chave_de_recebimento() {
        let bruno = assinatura::gerar();
        let servidor = assinatura::gerar();
        let (publicada, _s) = gerar(&bruno, [2; 16], 1_000, 3600);
        // O servidor publica uma chave dele, assinada por ele: recusada.
        let (falsa, _) = gerar(&servidor, [2; 16], 1_000, 3600);
        assert_eq!(conferir(&bruno.verificacao(), &falsa, 1_100).unwrap_err(), ErroDePrechave::AssinaturaInvalida);
        // Troca a parte pública mantendo a assinatura: recusada.
        let mut trocada = publicada.clone();
        trocada.publica = envelope::gerar_par_de_chaves().0.para_bytes();
        assert_eq!(conferir(&bruno.verificacao(), &trocada, 1_100).unwrap_err(), ErroDePrechave::AssinaturaInvalida);
        // Estica o prazo: recusada.
        let mut esticada = publicada.clone();
        esticada.valida_ate += 10;
        assert_eq!(conferir(&bruno.verificacao(), &esticada, 1_100).unwrap_err(), ErroDePrechave::AssinaturaInvalida);
    }

    #[test]
    fn prazos_sao_respeitados() {
        let bruno = assinatura::gerar();
        let (p, _s) = gerar(&bruno, [3; 16], 1_000, 999 * 24 * 3600);
        assert_eq!(p.valida_ate, 1_000 + VALIDADE_MAXIMA, "nunca mais de 7 dias");
        assert!(conferir(&bruno.verificacao(), &p, 1_000 + VALIDADE_MAXIMA - 1).is_ok());
        assert_eq!(conferir(&bruno.verificacao(), &p, 1_000 + VALIDADE_MAXIMA).unwrap_err(), ErroDePrechave::ForaDoPrazo);
        // Publicada com validade acima do máximo em relação a agora (relógio do remetente muito atrasado): recusada.
        assert_eq!(conferir(&bruno.verificacao(), &p, 999).unwrap_err(), ErroDePrechave::ForaDoPrazo);
    }

    #[test]
    fn id_ou_uso_trocados_nao_abrem() {
        let bruno = assinatura::gerar();
        let mut chaveiro = Chaveiro::default();
        let (p1, s1) = gerar(&bruno, [4; 16], 1_000, 3600);
        let (p2, s2) = gerar(&bruno, [5; 16], 1_000, 3600);
        chaveiro.guardar(s1); chaveiro.guardar(s2);
        let env = selar(&p1, &conferir(&bruno.verificacao(), &p1, 1_001).unwrap(), b"x", b"direto").unwrap();
        assert_eq!(chaveiro.abrir(&[5; 16], &env, b"direto").unwrap_err(), ErroDePrechave::Ilegivel);
        assert_eq!(chaveiro.abrir(&[4; 16], &env, b"lacre").unwrap_err(), ErroDePrechave::Ilegivel, "uso diferente no AAD");
        assert_eq!(p2.id, [5; 16]);
        assert!(format!("{:?}", chaveiro).contains("oculta"));
    }
}
