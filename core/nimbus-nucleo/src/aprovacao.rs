//! # Módulo `aprovacao` — assinaturas e aprovações de alto valor (ASS1–ASS12, BL4b)
//!
//! Uma engrenagem só, para três usos:
//! 1. **Documento** assinado por executivos (contrato, ata, procuração).
//! 2. **Pagamento** aprovado por mais de uma pessoa (regra dos quatro olhos, ou mais).
//! 3. **Publicação** de nova versão do app ou da imagem do Nimbus OS — nenhum programador sozinho
//!    consegue publicar: exige k de n assinaturas de pessoas diferentes, inclusive de fora da equipe.
//!
//! Proteções:
//! * **Vinculação dinâmica**: a assinatura cobre o hash exato do documento e, no pagamento, o valor e
//!   o beneficiário. Trocar uma vírgula, o valor ou a conta invalida todas as aprovações (mesmo princípio
//!   da autenticação forte com vinculação dinâmica do Regulamento Delegado (UE) 2018/389, art. 5 — validar).
//! * **k de n**: exige um número mínimo de aprovadores **diferentes** e **autorizados**; assinatura
//!   repetida da mesma pessoa conta uma vez; chave fora da lista é ignorada.
//! * **Prazo**: pedido vencido não pode mais ser aprovado.
//! * **Carência**: beneficiário novo ou valor acima do limite exige espera antes de liberar, com janela
//!   para cancelar — a defesa clássica contra golpe do falso executivo e troca de dados bancários.
//!
//! No app real, cada chave de aprovação fica no chip de segurança do aparelho do executivo e só assina
//! depois da biometria do próprio sistema (Face ID / biometria forte do Android). O NimbusOne nunca
//! recebe nem guarda rosto ou digital.
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Contexto próprio: uma assinatura de aprovação não vale em nenhum outro uso.
const CONTEXTO: &[u8] = b"NimbusOne/v1/aprovacao";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoDePedido { Documento, Pagamento, Publicacao, Administracao }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pedido {
    pub tipo: TipoDePedido,
    pub id: [u8; 16],
    /// Hash SHA-256 do documento ou da versão (para pagamento: da ordem completa).
    pub hash_do_conteudo: [u8; 32],
    pub valor_centavos: Option<u64>,
    pub beneficiario: Option<String>,
    pub criado_em: u64,
    pub expira_em: u64,
    pub minimo_de_aprovacoes: u8,
}

pub fn hash_do_conteudo(conteudo: &[u8]) -> [u8; 32] { Sha256::digest(conteudo).into() }

fn bytes_para_assinar(p: &Pedido) -> Vec<u8> {
    let mut v = b"NimbusOne/v1/aprovacao".to_vec();
    v.push(match p.tipo { TipoDePedido::Documento => 1, TipoDePedido::Pagamento => 2, TipoDePedido::Publicacao => 3, TipoDePedido::Administracao => 4 });
    v.extend_from_slice(&p.id);
    v.extend_from_slice(&p.hash_do_conteudo);
    match p.valor_centavos { Some(x) => { v.push(1); v.extend_from_slice(&x.to_be_bytes()); } None => v.push(0) }
    match &p.beneficiario { Some(b) => { v.push(1); v.extend_from_slice(&(b.len() as u32).to_be_bytes()); v.extend_from_slice(b.as_bytes()); } None => v.push(0) }
    v.extend_from_slice(&p.criado_em.to_be_bytes());
    v.extend_from_slice(&p.expira_em.to_be_bytes());
    v.push(p.minimo_de_aprovacoes);
    v
}

/// Uma aprovação: impressão digital da chave do aprovador + assinatura híbrida (Ed25519 + ML-DSA-87).
#[derive(Clone, Debug)]
pub struct Aprovacao { pub aprovador: [u8; 32], pub assinatura: AssinaturaHibrida }

/// No aparelho do executivo, depois da biometria: assina exatamente este pedido (assinatura híbrida).
pub fn aprovar(chave: &ChaveDeAssinatura, p: &Pedido) -> Aprovacao {
    let assinatura = chave.assinar(CONTEXTO, &bytes_para_assinar(p)).expect("contexto fixo e curto");
    Aprovacao { aprovador: chave.verificacao().impressao_digital(), assinatura }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Resultado { Aprovado { aprovadores: usize }, Pendente { faltam: usize }, Expirado, RegraInvalida }

/// Avalia as aprovações recebidas. Só conta aprovadores autorizados, distintos e com assinatura válida
/// sobre o pedido **exato**.
pub fn avaliar(p: &Pedido, aprovacoes: &[Aprovacao], autorizados: &[ChaveDeVerificacao], agora: u64) -> Resultado {
    let minimo = p.minimo_de_aprovacoes as usize;
    // A regra vale sobre pessoas distintas: a mesma chave repetida na lista conta uma vez.
    let lista: HashMap<[u8; 32], &ChaveDeVerificacao> = autorizados.iter().map(|k| (k.impressao_digital(), k)).collect();
    if minimo == 0 || minimo > lista.len() { return Resultado::RegraInvalida; }
    if agora >= p.expira_em { return Resultado::Expirado; }
    let msg = bytes_para_assinar(p);
    let mut validos: HashSet<[u8; 32]> = HashSet::new();
    for a in aprovacoes {
        let Some(chave) = lista.get(&a.aprovador) else { continue };
        if chave.verificar(CONTEXTO, &msg, &a.assinatura) { let _ = validos.insert(a.aprovador); }
    }
    if validos.len() >= minimo { Resultado::Aprovado { aprovadores: validos.len() } }
    else { Resultado::Pendente { faltam: minimo - validos.len() } }
}

/// Carência antes de liberar um pagamento: beneficiário novo ou valor acima do limite → espera.
pub fn carencia_em_segundos(beneficiario_ja_usado: bool, valor_centavos: u64, limite_centavos: u64, espera: u64) -> u64 {
    if !beneficiario_ja_usado || valor_centavos > limite_centavos { espera } else { 0 }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    fn executivos(n: usize) -> Vec<ChaveDeAssinatura> { (0..n).map(|_| assinatura::gerar()).collect() }
    fn publicas(k: &[ChaveDeAssinatura]) -> Vec<ChaveDeVerificacao> { k.iter().map(|x| x.verificacao()).collect() }
    fn pagamento(valor: u64, conta: &str) -> Pedido {
        Pedido { tipo: TipoDePedido::Pagamento, id: [1; 16], hash_do_conteudo: hash_do_conteudo(b"ordem 4471"), valor_centavos: Some(valor),
                 beneficiario: Some(conta.into()), criado_em: 100, expira_em: 10_000, minimo_de_aprovacoes: 2 }
    }

    #[test]
    fn dois_de_tres_aprovam() {
        let ex = executivos(3);
        let p = pagamento(5_000_000, "Banco 341 ag 0001 cc 12345-6");
        let ap = vec![aprovar(&ex[0], &p), aprovar(&ex[2], &p)];
        assert_eq!(avaliar(&p, &ap, &publicas(&ex), 200), Resultado::Aprovado { aprovadores: 2 });
    }

    #[test]
    fn a_mesma_pessoa_duas_vezes_conta_uma() {
        let ex = executivos(3);
        let p = pagamento(5_000_000, "conta X");
        let ap = vec![aprovar(&ex[0], &p), aprovar(&ex[0], &p)];
        assert_eq!(avaliar(&p, &ap, &publicas(&ex), 200), Resultado::Pendente { faltam: 1 });
    }

    #[test]
    fn chave_fora_da_lista_e_ignorada() {
        let ex = executivos(3);
        let intruso = assinatura::gerar();
        let p = pagamento(5_000_000, "conta X");
        let ap = vec![aprovar(&ex[0], &p), aprovar(&intruso, &p)];
        assert_eq!(avaliar(&p, &ap, &publicas(&ex), 200), Resultado::Pendente { faltam: 1 });
    }

    #[test]
    fn trocar_valor_ou_conta_depois_de_aprovado_invalida_tudo() {
        let ex = executivos(3);
        let original = pagamento(5_000_000, "conta legitima");
        let ap = vec![aprovar(&ex[0], &original), aprovar(&ex[1], &original)];
        let valor_trocado = Pedido { valor_centavos: Some(50_000_000), ..original.clone() };
        let conta_trocada = Pedido { beneficiario: Some("conta do golpista".into()), ..original.clone() };
        assert_eq!(avaliar(&valor_trocado, &ap, &publicas(&ex), 200), Resultado::Pendente { faltam: 2 });
        assert_eq!(avaliar(&conta_trocada, &ap, &publicas(&ex), 200), Resultado::Pendente { faltam: 2 });
    }

    #[test]
    fn documento_alterado_invalida_a_assinatura() {
        let ex = executivos(2);
        let mut p = Pedido { tipo: TipoDePedido::Documento, id: [2; 16], hash_do_conteudo: hash_do_conteudo(b"Contrato v7: preco 10 milhoes"),
            valor_centavos: None, beneficiario: None, criado_em: 0, expira_em: 999, minimo_de_aprovacoes: 2 };
        let ap = vec![aprovar(&ex[0], &p), aprovar(&ex[1], &p)];
        assert!(matches!(avaliar(&p, &ap, &publicas(&ex), 1), Resultado::Aprovado { .. }));
        p.hash_do_conteudo = hash_do_conteudo(b"Contrato v7: preco 1 milhao");
        assert_eq!(avaliar(&p, &ap, &publicas(&ex), 1), Resultado::Pendente { faltam: 2 });
    }

    #[test]
    fn pedido_vencido_nao_aprova() {
        let ex = executivos(2);
        let p = Pedido { minimo_de_aprovacoes: 1, ..pagamento(100, "c") };
        assert_eq!(avaliar(&p, &[aprovar(&ex[0], &p)], &publicas(&ex), 10_000), Resultado::Expirado);
    }

    #[test]
    fn regra_impossivel_e_recusada() {
        let ex = executivos(2);
        let p = Pedido { minimo_de_aprovacoes: 3, ..pagamento(100, "c") };
        assert_eq!(avaliar(&p, &[], &publicas(&ex), 1), Resultado::RegraInvalida);
    }

    #[test]
    fn nenhum_programador_publica_sozinho() {
        // Versão do app: exige 3 de 5 (por exemplo, 2 da equipe + 1 auditor externo + 2 diretores).
        let chaves = executivos(5);
        let versao = Pedido { tipo: TipoDePedido::Publicacao, id: [3; 16], hash_do_conteudo: hash_do_conteudo(b"nimbuschat-2.4.1.ipa"),
            valor_centavos: None, beneficiario: None, criado_em: 0, expira_em: 999, minimo_de_aprovacoes: 3 };
        let so_um_programador = vec![aprovar(&chaves[0], &versao)];
        assert_eq!(avaliar(&versao, &so_um_programador, &publicas(&chaves), 1), Resultado::Pendente { faltam: 2 });
    }

    #[test]
    fn lista_com_a_mesma_chave_repetida_nao_forja_quorum() {
        let ex = executivos(1);
        let repetida = vec![ex[0].verificacao(), ex[0].verificacao()];
        let p = pagamento(100, "c");
        assert_eq!(avaliar(&p, &[aprovar(&ex[0], &p)], &repetida, 1), Resultado::RegraInvalida);
    }

    #[test]
    fn carencia_para_beneficiario_novo_ou_valor_alto() {
        assert_eq!(carencia_em_segundos(false, 1_000, 100_000, 86_400), 86_400);
        assert_eq!(carencia_em_segundos(true, 500_000, 100_000, 86_400), 86_400);
        assert_eq!(carencia_em_segundos(true, 50_000, 100_000, 86_400), 0);
    }
}
