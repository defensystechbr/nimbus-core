//! # Módulo `selo` — sem checklist completo, nada roda (SLO1–SLO7)
//!
//! O **selo de liberação** é o documento que diz: "este artefato exato (resumo SHA-512) saiu deste commit e
//! passou por TODOS os portões de segurança". Ele precisa da assinatura de **três chaves diferentes**,
//! fixadas no próprio código: a do CI (que só assina se todos os portões passaram), a do responsável de
//! segurança e a do auditor independente, de fora da empresa.
//!
//! O servidor e os apps conferem o selo **na partida**, pela sentinela. Sem selo válido para o artefato que
//! está rodando — portão faltando, portão reprovado, artefato diferente, assinaturas insuficientes, versão
//! mais antiga que a mínima — **não funcionam**. Por isso, se os engenheiros não ligarem a proteção do
//! repositório, a revisão de duas pessoas ou qualquer outro portão, não sai selo, e o sistema não anda.
//!
//! O que isto NÃO faz, dito com verdade: se as três pessoas donas das chaves combinarem entre si (uma delas
//! de fora da empresa), elas conseguem assinar um selo para um artefato ruim. A defesa contra isso é a
//! independência delas, o registro público de cada selo e a auditoria.

use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use sha2::{Digest, Sha512};

const CONTEXTO: &[u8] = b"NimbusOne/v1/selo-de-liberacao";

/// Portões que todo selo precisa ter, todos aprovados. Tirar um daqui exige mudar o código (duas aprovações)
/// e a catraca confere a lista.
pub const PORTOES_OBRIGATORIOS: &[&str] = &[
    "testes", "mutacao", "fuzzing", "tempo_constante", "catraca", "varredura", "segredos", "dependencias",
    "banco_rls", "banco_mutacao", "sentinela", "protecao_do_repositorio", "revisao_de_duas_pessoas", "administracao_dupla",
    "commits_assinados", "procedencia", "consistencia_dos_documentos",
];

/// Mínimo de assinaturas de chaves diferentes num selo: CI, responsável de segurança e auditor independente
/// (de fora da empresa). Assim, ninguém de dentro, nem dois de dentro combinados, libera sozinho.
pub const MINIMO_DE_ASSINATURAS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selo {
    pub versao: u64,
    pub commit: String,
    pub resumo_do_artefato: [u8; 64],
    pub emitido_em: u64,
    /// (nome do portão, aprovado?)
    pub portoes: Vec<(String, bool)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDoSelo {
    ArtefatoDiferente,
    PortaoAusente(&'static str),
    PortaoReprovado(String),
    PortaoDesconhecido(String),
    AssinaturasInsuficientes { validas: usize },
    VersaoAntiga,
    EmitidoNoFuturo,
    Malformado,
}

/// Resumo SHA-512 do artefato (binário do servidor, pacote do app).
pub fn resumir(artefato: &[u8]) -> [u8; 64] { Sha512::digest(artefato).into() }

/// Forma canônica do selo, a que é assinada. Portões em ordem alfabética, sem repetição.
pub fn canonico(s: &Selo) -> Result<Vec<u8>, RecusaDoSelo> {
    let mut portoes = s.portoes.clone();
    portoes.sort();
    for par in portoes.windows(2) { if par[0].0 == par[1].0 { return Err(RecusaDoSelo::Malformado); } }
    let mut v = Vec::new();
    v.extend_from_slice(&s.versao.to_be_bytes());
    v.extend_from_slice(&(s.commit.len() as u32).to_be_bytes());
    v.extend_from_slice(s.commit.as_bytes());
    v.extend_from_slice(&s.resumo_do_artefato);
    v.extend_from_slice(&s.emitido_em.to_be_bytes());
    v.extend_from_slice(&(portoes.len() as u32).to_be_bytes());
    for (nome, ok) in &portoes {
        if nome.is_empty() || nome.len() > 64 || !nome.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') { return Err(RecusaDoSelo::Malformado); }
        v.push(nome.len() as u8);
        v.extend_from_slice(nome.as_bytes());
        v.push(u8::from(*ok));
    }
    Ok(v)
}

/// Quem assina (CI, responsável de segurança, auditor) assina a forma canônica.
pub fn assinar(chave: &ChaveDeAssinatura, s: &Selo) -> Result<([u8; 32], AssinaturaHibrida), RecusaDoSelo> {
    let m = canonico(s)?;
    let a = chave.assinar(CONTEXTO, &m).map_err(|_| RecusaDoSelo::Malformado)?;
    Ok((chave.verificacao().impressao_digital(), a))
}

/// Confere o selo do artefato que está rodando. `chaves_fixadas` vêm do próprio código (não de arquivo
/// externo); `versao_minima` impede voltar para uma versão antiga com falha conhecida.
pub fn conferir(s: &Selo, assinaturas: &[([u8; 32], AssinaturaHibrida)], artefato_rodando: &[u8; 64],
                chaves_fixadas: &[ChaveDeVerificacao], versao_minima: u64, agora: u64) -> Result<(), RecusaDoSelo> {
    if &s.resumo_do_artefato != artefato_rodando { return Err(RecusaDoSelo::ArtefatoDiferente); }
    if s.versao < versao_minima { return Err(RecusaDoSelo::VersaoAntiga); }
    if s.emitido_em > agora.saturating_add(300) { return Err(RecusaDoSelo::EmitidoNoFuturo); }
    for obrig in PORTOES_OBRIGATORIOS {
        match s.portoes.iter().find(|(n, _)| n == obrig) {
            None => return Err(RecusaDoSelo::PortaoAusente(obrig)),
            Some((_, false)) => return Err(RecusaDoSelo::PortaoReprovado((*obrig).to_string())),
            Some((_, true)) => {}
        }
    }
    if let Some((n, _)) = s.portoes.iter().find(|(n, _)| !PORTOES_OBRIGATORIOS.contains(&n.as_str())) {
        return Err(RecusaDoSelo::PortaoDesconhecido(n.clone()));
    }
    let m = canonico(s)?;
    let mut validas: Vec<[u8; 32]> = Vec::new();
    for (fp, a) in assinaturas {
        if validas.contains(fp) { continue; }
        if let Some(k) = chaves_fixadas.iter().find(|k| &k.impressao_digital() == fp) {
            if k.verificar(CONTEXTO, &m, a) { validas.push(*fp); }
        }
    }
    if validas.len() < MINIMO_DE_ASSINATURAS { return Err(RecusaDoSelo::AssinaturasInsuficientes { validas: validas.len() }); }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    fn selo_completo(artefato: &[u8]) -> Selo {
        Selo { versao: 7, commit: "a1b2c3".into(), resumo_do_artefato: resumir(artefato), emitido_em: 1_000,
               portoes: PORTOES_OBRIGATORIOS.iter().map(|p| ((*p).to_string(), true)).collect() }
    }

    #[test]
    fn selo_com_tudo_e_duas_chaves_passa() {
        let (ci, seg, aud) = (assinatura::gerar(), assinatura::gerar(), assinatura::gerar());
        let s = selo_completo(b"binario");
        let ass = vec![assinar(&ci, &s).unwrap(), assinar(&seg, &s).unwrap(), assinar(&aud, &s).unwrap()];
        let fix = [ci.verificacao(), seg.verificacao(), aud.verificacao()];
        assert_eq!(conferir(&s, &ass, &resumir(b"binario"), &fix, 7, 1_000), Ok(()));
        assert_eq!(MINIMO_DE_ASSINATURAS, 3, "CI, segurança e auditor de fora");
        // Duas de três não bastam: nenhum par de dentro libera sozinho.
        assert_eq!(conferir(&s, &ass[..2], &resumir(b"binario"), &fix, 7, 1_000), Err(RecusaDoSelo::AssinaturasInsuficientes { validas: 2 }));
    }

    #[test]
    fn sem_checklist_completo_nada_roda() {
        let (ci, seg, aud) = (assinatura::gerar(), assinatura::gerar(), assinatura::gerar());
        let fix = [ci.verificacao(), seg.verificacao(), aud.verificacao()];
        let r = resumir(b"binario");
        let assina = |s: &Selo| vec![assinar(&ci, s).unwrap(), assinar(&seg, s).unwrap(), assinar(&aud, s).unwrap()];
        // Portão ausente (ex.: ninguém ligou a proteção do repositório).
        let mut s = selo_completo(b"binario"); s.portoes.retain(|(n, _)| n != "protecao_do_repositorio");
        assert_eq!(conferir(&s, &assina(&s), &r, &fix, 7, 1_000), Err(RecusaDoSelo::PortaoAusente("protecao_do_repositorio")));
        // Portão reprovado.
        let mut s = selo_completo(b"binario"); s.portoes.iter_mut().find(|(n, _)| n == "revisao_de_duas_pessoas").unwrap().1 = false;
        assert_eq!(conferir(&s, &assina(&s), &r, &fix, 7, 1_000), Err(RecusaDoSelo::PortaoReprovado("revisao_de_duas_pessoas".into())));
        // Portão inventado para confundir.
        let mut s = selo_completo(b"binario"); s.portoes.push(("ignorar_tudo".into(), true));
        assert_eq!(conferir(&s, &assina(&s), &r, &fix, 7, 1_000), Err(RecusaDoSelo::PortaoDesconhecido("ignorar_tudo".into())));
        // Artefato trocado depois do selo.
        let s = selo_completo(b"binario");
        assert_eq!(conferir(&s, &assina(&s), &resumir(b"binario-com-porta"), &fix, 7, 1_000), Err(RecusaDoSelo::ArtefatoDiferente));
        // Volta para versão antiga.
        assert_eq!(conferir(&s, &assina(&s), &r, &fix, 8, 1_000), Err(RecusaDoSelo::VersaoAntiga));
        // Data no futuro.
        assert_eq!(conferir(&s, &assina(&s), &r, &fix, 7, 600), Err(RecusaDoSelo::EmitidoNoFuturo));
    }

    #[test]
    fn uma_pessoa_sozinha_nao_libera() {
        let (ci, seg, aud, intruso) = (assinatura::gerar(), assinatura::gerar(), assinatura::gerar(), assinatura::gerar());
        let fix = [ci.verificacao(), seg.verificacao(), aud.verificacao()];
        let s = selo_completo(b"binario");
        let r = resumir(b"binario");
        // Só uma assinatura.
        assert_eq!(conferir(&s, &[assinar(&ci, &s).unwrap()], &r, &fix, 7, 1_000), Err(RecusaDoSelo::AssinaturasInsuficientes { validas: 1 }));
        // A mesma chave duas vezes.
        let a = assinar(&ci, &s).unwrap();
        assert_eq!(conferir(&s, &[a.clone(), a], &r, &fix, 7, 1_000), Err(RecusaDoSelo::AssinaturasInsuficientes { validas: 1 }));
        // Chave fora das fixadas.
        assert_eq!(conferir(&s, &[assinar(&ci, &s).unwrap(), assinar(&seg, &s).unwrap(), assinar(&intruso, &s).unwrap()], &r, &fix, 7, 1_000), Err(RecusaDoSelo::AssinaturasInsuficientes { validas: 2 }));
        // Assinatura de outro selo reaproveitada.
        let mut outro = selo_completo(b"binario"); outro.versao = 9;
        assert_eq!(conferir(&s, &[assinar(&ci, &s).unwrap(), assinar(&seg, &s).unwrap(), assinar(&aud, &outro).unwrap()], &r, &fix, 7, 1_000), Err(RecusaDoSelo::AssinaturasInsuficientes { validas: 2 }));
        // Três assinaturas, mas duas da mesma chave.
        assert_eq!(conferir(&s, &[assinar(&ci, &s).unwrap(), assinar(&ci, &s).unwrap(), assinar(&seg, &s).unwrap()], &r, &fix, 7, 1_000), Err(RecusaDoSelo::AssinaturasInsuficientes { validas: 2 }));
    }

    #[test]
    fn selo_malformado_e_recusado() {
        let mut s = selo_completo(b"x"); s.portoes.push(("testes".into(), true));
        assert_eq!(canonico(&s), Err(RecusaDoSelo::Malformado), "portão repetido");
        let mut s = selo_completo(b"x"); s.portoes.push(("Testes Maiusculo".into(), true));
        assert_eq!(canonico(&s), Err(RecusaDoSelo::Malformado));
        let k = assinatura::de_semente(&[7u8; 64]);
        assert_eq!(k.verificacao().impressao_digital(), assinatura::de_semente(&[7u8; 64]).verificacao().impressao_digital(), "semente recria a mesma chave");
    }
}
