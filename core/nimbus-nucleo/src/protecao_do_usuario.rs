//! # Módulo `protecao_do_usuario` — bloquear, denunciar e apagar a conta sem quebrar o sigilo
//! (BLQ1–BLQ4, DEN1–DEN5, APG1–APG4)
//!
//! A App Store exige, em app com conteúdo de usuários, bloquear quem abusa e denunciar conteúdo ofensivo
//! (regra 1.2), e exige apagar a conta dentro do app (regra 5.1.1(v)). O Nimbus faz os três sem que o servidor
//! passe a ver nada que não via:
//!
//! * **Bloquear (BLQ):** a lista fica só no aparelho, cifrada junto do resto. Mensagem, chamada, arquivo ou
//!   convite de quem está bloqueado é descartado em silêncio, antes de qualquer resposta: quem foi bloqueado
//!   não recebe confirmação de leitura, nem "digitando", nem toque, e não consegue saber que foi bloqueado.
//! * **Denunciar (DEN):** denuncia a CONTA, nunca o conteúdo. O NimbusChat não deixa copiar, encaminhar nem
//!   exportar mensagens, e a denúncia não abre exceção: o pacote leva só a impressão digital de quem é
//!   denunciado, o motivo e nada mais — nenhuma mensagem, nenhum texto livre, nenhum dado de quem denuncia.
//!   Só por iniciativa da pessoa, com consentimento explícito; cifrado para a chave da moderação (vinda da
//!   configuração assinada do app) e de tamanho fixo. Denunciar também bloqueia. Nada é automático.
//! * **Apagar a conta (APG):** pedido assinado pela chave da conta, válido por 5 minutos e de uso único; o
//!   servidor apaga caixa, pré-chaves e avisos de entrega. Exige o PIN real: sob o PIN de coação o pedido não
//!   é emitido, e o app mostra a mesma falha genérica de rede (nada revela o modo de proteção).
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use crate::envelope::{self, ChavePublica};
use std::collections::BTreeSet;

/// Impressão digital da chave de identidade de uma pessoa (32 bytes).
pub type Digital = [u8; 32];

// ---------------------------------------------------------------- bloquear

/// Máximo de pessoas bloqueadas (a lista é guardada com tamanho fixo para não revelar quantas são).
pub const MAXIMO_DE_BLOQUEADOS: usize = 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListaDeBloqueio { bloqueados: BTreeSet<Digital> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entrada { Mensagem, Chamada, Arquivo, Convite, AtualizacaoDePerfil }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destino {
    Entregar,
    /// Descartar sem responder nada: nem recibo, nem toque, nem erro para quem enviou (BLQ2, BLQ3).
    DescartarEmSilencio,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeBloqueio { ListaCheia }

impl ListaDeBloqueio {
    /// BLQ1: bloquear. Idempotente.
    pub fn bloquear(&mut self, d: Digital) -> Result<(), ErroDeBloqueio> {
        if !self.bloqueados.contains(&d) && self.bloqueados.len() >= MAXIMO_DE_BLOQUEADOS { return Err(ErroDeBloqueio::ListaCheia); }
        self.bloqueados.insert(d);
        Ok(())
    }
    pub fn desbloquear(&mut self, d: &Digital) { self.bloqueados.remove(d); }
    pub fn esta_bloqueado(&self, d: &Digital) -> bool { self.bloqueados.contains(d) }
    pub fn quantos(&self) -> usize { self.bloqueados.len() }

    /// BLQ2–BLQ3: decide o destino de tudo o que chega, antes de qualquer resposta ao remetente.
    pub fn decidir(&self, remetente: &Digital, _tipo: Entrada) -> Destino {
        if self.esta_bloqueado(remetente) { Destino::DescartarEmSilencio } else { Destino::Entregar }
    }

    /// BLQ4: guardada com tamanho fixo (as vagas vazias viram zeros), para o tamanho não revelar quantas são.
    pub fn para_bytes_de_tamanho_fixo(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(4 + MAXIMO_DE_BLOQUEADOS * 32);
        v.extend_from_slice(&(self.bloqueados.len() as u32).to_be_bytes());
        for d in &self.bloqueados { v.extend_from_slice(d); }
        v.resize(4 + MAXIMO_DE_BLOQUEADOS * 32, 0);
        v
    }

    pub fn de_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != 4 + MAXIMO_DE_BLOQUEADOS * 32 { return None; }
        let n = u32::from_be_bytes(b[..4].try_into().ok()?) as usize;
        if n > MAXIMO_DE_BLOQUEADOS { return None; }
        let mut l = ListaDeBloqueio::default();
        for i in 0..n {
            let mut d = [0u8; 32];
            d.copy_from_slice(&b[4 + i * 32..4 + (i + 1) * 32]);
            if !l.bloqueados.insert(d) { return None; } // repetido = adulterado
        }
        if b[4 + n * 32..].iter().any(|&x| x != 0) { return None; } // sobra que não é zero = adulterado
        Some(l)
    }
}

// ---------------------------------------------------------------- denunciar

/// Tamanho fixo da denúncia antes de cifrar.
pub const TAMANHO_FIXO_DA_DENUNCIA: usize = 256;
const CONTEXTO_DA_DENUNCIA: &[u8] = b"NimbusOne/v1/denuncia";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motivo { Assedio, Golpe, ConteudoIlegal, Spam, Outro }

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeDenuncia { SemConsentimento, FalhaAoCifrar }

/// DEN1–DEN5: denúncia da CONTA, sem nenhum conteúdo. A assinatura da função não aceita mensagem nem texto:
/// não existe caminho, dentro do app, para uma conversa sair por aqui. Quem denuncia passa a ter o denunciado
/// bloqueado (o app chama `ListaDeBloqueio::bloquear` junto).
pub fn preparar_denuncia(
    moderacao: &ChavePublica,
    denunciado: &Digital,
    motivo: Motivo,
    consentimento_explicito: bool,
) -> Result<Vec<u8>, ErroDeDenuncia> {
    if !consentimento_explicito { return Err(ErroDeDenuncia::SemConsentimento); }
    let mut claro = Vec::with_capacity(TAMANHO_FIXO_DA_DENUNCIA);
    claro.extend_from_slice(denunciado);
    claro.push(motivo as u8);
    claro.resize(TAMANHO_FIXO_DA_DENUNCIA, 0);
    envelope::selar(moderacao, &claro, CONTEXTO_DA_DENUNCIA).map_err(|_| ErroDeDenuncia::FalhaAoCifrar)
}

// ---------------------------------------------------------------- apagar a conta

pub const VALIDADE_DO_PEDIDO_DE_APAGAR: u64 = 300;
const CONTEXTO_DE_APAGAR: &[u8] = b"NimbusOne/v1/apagar-conta";

#[derive(Clone, Debug)]
pub struct PedidoDeApagar { pub conta: Digital, pub emitido_em: u64, pub nonce: [u8; 16], pub assinatura: AssinaturaHibrida }

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeApagar { PinNaoConfirmado, FalhaDeRede, AssinaturaInvalida, Vencido, DoFuturo, Repetido }

fn mensagem_de_apagar(conta: &Digital, emitido_em: u64, nonce: &[u8; 16]) -> Vec<u8> {
    let mut m = Vec::with_capacity(56);
    m.extend_from_slice(conta);
    m.extend_from_slice(&emitido_em.to_be_bytes());
    m.extend_from_slice(nonce);
    m
}

/// APG1–APG2: no aparelho. Só com o PIN real confirmado; sob o PIN de coação, a mesma falha genérica de rede.
pub fn pedir_apagar_conta(
    chave_da_conta: &ChaveDeAssinatura,
    conta: Digital,
    agora: u64,
    nonce: [u8; 16],
    pin_real_confirmado: bool,
    em_coacao: bool,
) -> Result<PedidoDeApagar, RecusaDeApagar> {
    if em_coacao { return Err(RecusaDeApagar::FalhaDeRede); }
    if !pin_real_confirmado { return Err(RecusaDeApagar::PinNaoConfirmado); }
    let assinatura = chave_da_conta
        .assinar(CONTEXTO_DE_APAGAR, &mensagem_de_apagar(&conta, agora, &nonce))
        .map_err(|_| RecusaDeApagar::AssinaturaInvalida)?;
    Ok(PedidoDeApagar { conta, emitido_em: agora, nonce, assinatura })
}

/// APG3–APG4: no servidor. Assinatura da própria conta, dentro do prazo, nunca do futuro, nunca repetido.
pub fn conferir_pedido_de_apagar(
    verificacao_da_conta: &ChaveDeVerificacao,
    p: &PedidoDeApagar,
    agora: u64,
    nonces_ja_usados: &mut BTreeSet<[u8; 16]>,
) -> Result<(), RecusaDeApagar> {
    if !verificacao_da_conta.verificar(CONTEXTO_DE_APAGAR, &mensagem_de_apagar(&p.conta, p.emitido_em, &p.nonce), &p.assinatura) {
        return Err(RecusaDeApagar::AssinaturaInvalida);
    }
    if p.emitido_em > agora.saturating_add(30) { return Err(RecusaDeApagar::DoFuturo); }
    if agora > p.emitido_em.saturating_add(VALIDADE_DO_PEDIDO_DE_APAGAR) { return Err(RecusaDeApagar::Vencido); }
    if !nonces_ja_usados.insert(p.nonce) { return Err(RecusaDeApagar::Repetido); }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    #[test]
    fn bloqueado_e_descartado_em_silencio_em_todo_tipo() {
        let mut l = ListaDeBloqueio::default();
        let ruim = [7u8; 32]; let bom = [8u8; 32];
        l.bloquear(ruim).unwrap();
        for t in [Entrada::Mensagem, Entrada::Chamada, Entrada::Arquivo, Entrada::Convite, Entrada::AtualizacaoDePerfil] {
            assert_eq!(l.decidir(&ruim, t), Destino::DescartarEmSilencio, "{t:?}");
            assert_eq!(l.decidir(&bom, t), Destino::Entregar, "{t:?}");
        }
        l.desbloquear(&ruim);
        assert_eq!(l.decidir(&ruim, Entrada::Mensagem), Destino::Entregar);
    }

    #[test]
    fn lista_tem_tamanho_fixo_e_recusa_adulteracao() {
        let mut l = ListaDeBloqueio::default();
        assert_eq!(l.para_bytes_de_tamanho_fixo().len(), ListaDeBloqueio { bloqueados: [[1u8; 32]].into() }.para_bytes_de_tamanho_fixo().len());
        l.bloquear([1u8; 32]).unwrap(); l.bloquear([2u8; 32]).unwrap(); l.bloquear([1u8; 32]).unwrap();
        assert_eq!(l.quantos(), 2, "bloquear de novo não duplica");
        let b = l.para_bytes_de_tamanho_fixo();
        assert_eq!(ListaDeBloqueio::de_bytes(&b), Some(l.clone()));
        let mut sobra = b.clone(); *sobra.last_mut().unwrap() = 1;
        assert_eq!(ListaDeBloqueio::de_bytes(&sobra), None, "lixo na sobra");
        let mut repetido = b.clone(); repetido[36..68].copy_from_slice(&[1u8; 32]);
        assert_eq!(ListaDeBloqueio::de_bytes(&repetido), None, "repetido");
        let mut contagem = b.clone(); contagem[..4].copy_from_slice(&(MAXIMO_DE_BLOQUEADOS as u32 + 1).to_be_bytes());
        assert_eq!(ListaDeBloqueio::de_bytes(&contagem), None, "contagem impossível");
        assert_eq!(ListaDeBloqueio::de_bytes(&b[1..]), None, "tamanho errado");
    }

    #[test]
    fn contagem_maior_que_o_espaco_e_recusada_sem_quebrar() {
        // Lista cheia, toda distinta: com a contagem adulterada para uma a mais, a leitura passaria do fim
        // do arquivo. Tem de recusar, nunca entrar em pânico (defeito plantado nº 275 achou este caso).
        let mut l = ListaDeBloqueio::default();
        for i in 0..MAXIMO_DE_BLOQUEADOS { let mut d = [0u8; 32]; d[..8].copy_from_slice(&(i as u64 + 1).to_be_bytes()); l.bloquear(d).unwrap(); }
        let mut b = l.para_bytes_de_tamanho_fixo();
        assert_eq!(ListaDeBloqueio::de_bytes(&b), Some(l));
        b[..4].copy_from_slice(&(MAXIMO_DE_BLOQUEADOS as u32 + 1).to_be_bytes());
        assert_eq!(ListaDeBloqueio::de_bytes(&b), None);
        b[..4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(ListaDeBloqueio::de_bytes(&b), None);
    }

    #[test]
    fn lista_cheia_recusa_novo_mas_aceita_quem_ja_esta() {
        let mut l = ListaDeBloqueio::default();
        for i in 0..MAXIMO_DE_BLOQUEADOS { let mut d = [0u8; 32]; d[..8].copy_from_slice(&(i as u64).to_be_bytes()); l.bloquear(d).unwrap(); }
        assert_eq!(l.bloquear([0xFFu8; 32]), Err(ErroDeBloqueio::ListaCheia));
        let mut ja = [0u8; 32]; ja[..8].copy_from_slice(&0u64.to_be_bytes());
        assert_eq!(l.bloquear(ja), Ok(()));
    }

    #[test]
    fn denuncia_so_com_consentimento_e_sem_nenhum_conteudo() {
        let (moderacao, sec) = envelope::gerar_par_de_chaves();
        let d = [3u8; 32];
        assert_eq!(preparar_denuncia(&moderacao, &d, Motivo::Golpe, false), Err(ErroDeDenuncia::SemConsentimento));
        let a = preparar_denuncia(&moderacao, &d, Motivo::Assedio, true).unwrap();
        let b = preparar_denuncia(&moderacao, &[4u8; 32], Motivo::Spam, true).unwrap();
        assert_eq!(a.len(), b.len(), "sempre o mesmo tamanho");
        let claro = envelope::abrir(&sec, &moderacao, &a, CONTEXTO_DA_DENUNCIA).unwrap();
        assert_eq!(claro.len(), TAMANHO_FIXO_DA_DENUNCIA);
        assert_eq!(&claro[..32], &d);
        assert_eq!(claro[32], Motivo::Assedio as u8);
        assert!(claro[33..].iter().all(|&x| x == 0), "nada além da conta e do motivo: nenhum conteúdo");
        let (outra, outra_sec) = envelope::gerar_par_de_chaves();
        assert!(envelope::abrir(&outra_sec, &outra, &a, CONTEXTO_DA_DENUNCIA).is_err(), "só a moderação abre");
    }

    #[test]
    fn apagar_conta_exige_pin_real_e_coacao_parece_falha_de_rede() {
        let k = assinatura::gerar();
        let conta = [9u8; 32];
        assert_eq!(pedir_apagar_conta(&k, conta, 100, [1u8; 16], false, false).unwrap_err(), RecusaDeApagar::PinNaoConfirmado);
        assert_eq!(pedir_apagar_conta(&k, conta, 100, [1u8; 16], true, true).unwrap_err(), RecusaDeApagar::FalhaDeRede);
        assert!(pedir_apagar_conta(&k, conta, 100, [1u8; 16], true, false).is_ok());
    }

    #[test]
    fn servidor_so_aceita_pedido_da_propria_conta_no_prazo_e_uma_vez() {
        let k = assinatura::gerar(); let v = k.verificacao();
        let outra = assinatura::gerar().verificacao();
        let conta = [9u8; 32];
        let p = pedir_apagar_conta(&k, conta, 1_000, [5u8; 16], true, false).unwrap();
        let mut usados = BTreeSet::new();
        assert_eq!(conferir_pedido_de_apagar(&outra, &p, 1_000, &mut usados), Err(RecusaDeApagar::AssinaturaInvalida));
        assert_eq!(conferir_pedido_de_apagar(&v, &p, 1_000 + VALIDADE_DO_PEDIDO_DE_APAGAR + 1, &mut usados), Err(RecusaDeApagar::Vencido));
        assert_eq!(conferir_pedido_de_apagar(&v, &p, 900, &mut usados), Err(RecusaDeApagar::DoFuturo));
        assert_eq!(conferir_pedido_de_apagar(&v, &p, 1_000 + VALIDADE_DO_PEDIDO_DE_APAGAR, &mut usados), Ok(()));
        assert_eq!(conferir_pedido_de_apagar(&v, &p, 1_001, &mut usados), Err(RecusaDeApagar::Repetido));
        let mut adulterado = p.clone(); adulterado.conta = [8u8; 32];
        assert_eq!(conferir_pedido_de_apagar(&v, &adulterado, 1_000, &mut BTreeSet::new()), Err(RecusaDeApagar::AssinaturaInvalida));
        let mut outro_horario = p.clone(); outro_horario.emitido_em = 1_100;
        assert_eq!(conferir_pedido_de_apagar(&v, &outro_horario, 1_100, &mut BTreeSet::new()), Err(RecusaDeApagar::AssinaturaInvalida));
    }
}
