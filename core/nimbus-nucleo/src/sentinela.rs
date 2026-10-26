//! # Módulo `sentinela` — o sistema se confere, trava e diz onde está o problema (SEN1–SEN5)
//!
//! O servidor e os apps chamam `autoverificar()` **na partida e periodicamente** (no servidor, a cada hora;
//! nos apps, a cada abertura). Se qualquer conferência falhar, `Relatorio::pode_funcionar()` é falso e quem
//! chamou **não atende ninguém** (falha fechada): o servidor sai do ar e alerta; o app mostra "manutenção de
//! segurança" e não abre conteúdo. Cada achado diz **onde** está o problema e **como consertar**.
//!
//! Por que não "consertar sozinho": um sistema que se reconfigura automaticamente em produção é, ele mesmo,
//! uma porta — quem descobrir o gatilho escolhe a configuração. Aqui o conserto é sempre feito por gente,
//! com teste e duas aprovações; a sentinela trava, relata e diz o que fazer.
//!
//! O que ela confere: invariantes de segurança (rotas, limites, prazos, esperas, parâmetros de senha) e
//! autotestes criptográficos (cifra e assinatura funcionam e recusam adulteração; chave malformada é recusada;
//! o gerador de números aleatórios não está travado).

use crate::{assinatura, autenticacao, corporativo, direto, envelope, expiracao, prechaves, recuperacao, selo, senha};
use rand_core::{OsRng, RngCore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achado { pub codigo: &'static str, pub onde: &'static str, pub problema: String, pub como_corrigir: &'static str }

#[derive(Debug, Default)]
pub struct Relatorio { pub achados: Vec<Achado> }

impl Relatorio {
    /// Só funciona sem nenhum achado. Não existe "achado leve": qualquer um trava.
    pub fn pode_funcionar(&self) -> bool { self.achados.is_empty() }

    /// Texto do relatório para o alerta (sem conteúdo de usuário, sem segredo: só código, lugar e conserto).
    pub fn texto(&self) -> String {
        if self.achados.is_empty() { return "Sentinela: tudo conferido; pode funcionar.".to_string(); }
        let mut t = format!("Sentinela: TRAVADO por {} achado(s).\n", self.achados.len());
        for a in &self.achados {
            t.push_str(&format!("[{}] {} — {}\n    Como corrigir: {}\n", a.codigo, a.onde, a.problema, a.como_corrigir));
        }
        t
    }

    fn acusar(&mut self, codigo: &'static str, onde: &'static str, problema: String, como_corrigir: &'static str) {
        self.achados.push(Achado { codigo, onde, problema, como_corrigir });
    }
}

/// Valores que a sentinela confere. Em produção vêm das constantes do núcleo (`Valores::atuais`).
#[derive(Debug, Clone, Copy)]
pub struct Valores {
    pub rotas: &'static [autenticacao::Rota], pub teto_por_minuto: u32, pub janela: u64,
    pub vida_maxima: u64, pub prazo_direto: u64, pub limite_para_ler: u64, pub limite_sem_hora: u64,
    pub prechave_validade: u64, pub prechave_sobrevida: u64, pub carencia_recuperacao: u64, pub carencia_corporativa: u64,
    pub senha_memoria_kib: u32, pub senha_passagens: u32, pub senha_faixas: u32,
}

impl Valores {
    pub fn atuais() -> Self {
        Valores { rotas: autenticacao::ROTAS, teto_por_minuto: autenticacao::LIMITE_MAXIMO_POR_MINUTO, janela: autenticacao::JANELA_SEGUNDOS,
                  vida_maxima: expiracao::VIDA_MAXIMA, prazo_direto: direto::PRAZO_MAXIMO, limite_para_ler: expiracao::LIMITE_PARA_LER,
                  limite_sem_hora: expiracao::LIMITE_SEM_HORA_CONFIRMADA, prechave_validade: prechaves::VALIDADE_MAXIMA,
                  prechave_sobrevida: prechaves::SOBREVIDA_DA_SECRETA, carencia_recuperacao: recuperacao::CARENCIA_SEGUNDOS,
                  carencia_corporativa: corporativo::CARENCIA_DA_RECUPERACAO, senha_memoria_kib: senha::MEMORIA_KIB,
                  senha_passagens: senha::PASSAGENS, senha_faixas: senha::FAIXAS }
    }
}

/// Confere as invariantes do núcleo (rápido; sem autotestes criptográficos).
pub fn conferir_invariantes() -> Relatorio { conferir_valores(&Valores::atuais()) }

/// Confere um conjunto de valores. Separado para poder ser testado com valores errados.
pub fn conferir_valores(v: &Valores) -> Relatorio {
    let mut r = Relatorio::default();
    let rotas = v.rotas;
    if rotas.is_empty() { r.acusar("SEN-ROTA-0", "autenticacao::ROTAS", "nenhuma rota declarada".into(), "Restaurar o inventário de rotas a partir do repositório."); }
    for (i, a) in rotas.iter().enumerate() {
        if a.por_minuto == 0 || a.por_minuto > v.teto_por_minuto || v.teto_por_minuto > 600 {
            r.acusar("SEN-ROTA-1", "autenticacao::ROTAS", format!("rota {} sem limite válido ({} por minuto)", a.caminho, a.por_minuto),
                     "Dar à rota um limite entre 1 e LIMITE_MAXIMO_POR_MINUTO (no máximo 600); rodar a catraca.");
        }
        if !a.caminho.starts_with("/v1/") || a.caminho.ends_with('/') || a.caminho.contains("..") || a.caminho.contains("//") || a.caminho.chars().any(|c| c.is_ascii_uppercase() || c == '%') {
            r.acusar("SEN-ROTA-2", "autenticacao::ROTAS", format!("caminho fora do padrão: {}", a.caminho), "Usar só /v1/ em minúsculas, sem barra no fim, sem .., // ou %.");
        }
        if rotas[i + 1..].iter().any(|b| b.metodo == a.metodo && b.caminho == a.caminho) {
            r.acusar("SEN-ROTA-3", "autenticacao::ROTAS", format!("rota repetida: {} {}", a.metodo, a.caminho), "Remover a duplicata.");
        }
    }
    if v.janela > 60 { r.acusar("SEN-JAN", "autenticacao::JANELA_SEGUNDOS", format!("janela de {} s", v.janela), "Voltar a janela para no máximo 60 s."); }
    if v.vida_maxima > 48 * 3600 || v.prazo_direto > 48 * 3600 || v.limite_para_ler > 48 * 3600 {
        r.acusar("SEN-PRAZO", "expiracao / direto", "prazo máximo acima de 48 h".into(), "Restaurar 48 h (decisão do fundador; nada dura mais de 48 h).");
    }
    if v.limite_sem_hora > 15 * 60 { r.acusar("SEN-HORA", "expiracao::LIMITE_SEM_HORA_CONFIRMADA", "acima de 15 min".into(), "Restaurar 15 min (OFF2)."); }
    if v.prechave_validade > 7 * 24 * 3600 || v.prechave_sobrevida > 48 * 3600 {
        r.acusar("SEN-FS", "prechaves", "chaves de recebimento vivem mais que o permitido".into(), "Restaurar 7 dias de validade e 48 h de sobrevida (FS1–FS4).");
    }
    if v.carencia_recuperacao < 72 * 3600 { r.acusar("SEN-REC", "recuperacao::CARENCIA_SEGUNDOS", "espera da recuperação abaixo de 3 dias".into(), "Restaurar 3 dias (EO4)."); }
    if v.carencia_corporativa < 24 * 3600 { r.acusar("SEN-CORP", "corporativo::CARENCIA_DA_RECUPERACAO", "espera abaixo de 24 h".into(), "Restaurar 24 h (EO5)."); }
    if v.senha_memoria_kib < 19 * 1024 || v.senha_passagens < 2 || v.senha_faixas < 1 {
        let (memoria_kib, passagens) = (v.senha_memoria_kib, v.senha_passagens);
        let descricao = format!("Argon2id {memoria_kib} KiB, {passagens} passagens");
        r.acusar("SEN-SENHA", "senha", descricao, "Restaurar 64 MiB e 3 passagens (mínimo absoluto OWASP: 19 MiB e 2).");
    }
    r
}

/// Invariantes + autotestes criptográficos. Chamar na partida e a cada hora (servidor) ou a cada abertura (app).
pub fn autoverificar() -> Relatorio {
    let mut r = conferir_invariantes();
    // Gerador de números aleatórios: dois sorteios não podem ser iguais nem nulos.
    let (mut a, mut b) = ([0u8; 32], [0u8; 32]);
    OsRng.fill_bytes(&mut a); OsRng.fill_bytes(&mut b);
    if a == b || a == [0u8; 32] { r.acusar("SEN-RNG", "gerador do sistema (OsRng)", "sorteios repetidos ou nulos".into(), "Não atender: investigar o sistema operacional e a máquina antes de voltar."); }
    // Cifra: abre o que selou; recusa adulteração e dado autenticado trocado.
    let (publica, secreta) = envelope::gerar_par_de_chaves();
    match envelope::selar(&publica, b"autoteste da sentinela", b"sentinela") {
        Ok(env) => {
            if envelope::abrir(&secreta, &publica, &env, b"sentinela").ok().as_deref().map(|v| v.as_slice()) != Some(&b"autoteste da sentinela"[..]) {
                r.acusar("SEN-CIFRA-1", "envelope::abrir", "não abriu o que acabou de selar".into(), "Não atender: biblioteca de criptografia corrompida ou adulterada; reinstalar a partir do build verificado.");
            }
            let mut adulterado = env.clone();
            if let Some(u) = adulterado.last_mut() { *u ^= 1; }
            if envelope::abrir(&secreta, &publica, &adulterado, b"sentinela").is_ok() || envelope::abrir(&secreta, &publica, &env, b"outro").is_ok() {
                r.acusar("SEN-CIFRA-2", "envelope::abrir", "aceitou envelope adulterado ou dado autenticado trocado".into(), "Não atender: verificação de integridade quebrada; reinstalar do build verificado e acionar a auditoria.");
            }
        }
        Err(_) => r.acusar("SEN-CIFRA-0", "envelope::selar", "não conseguiu selar".into(), "Não atender: reinstalar do build verificado."),
    }
    // Chave ML-KEM malformada tem de ser recusada (FIPS 203, 7.2).
    let mut b_pub = publica.para_bytes();
    if b_pub.len() > envelope::TAM_X25519 + 3 { for byte in &mut b_pub[envelope::TAM_X25519..envelope::TAM_X25519 + 3] { *byte = 0xFF; } }
    if envelope::ChavePublica::de_bytes(&b_pub).is_ok() {
        r.acusar("SEN-KEM", "envelope::ChavePublica::de_bytes", "aceitou chave ML-KEM fora do intervalo".into(), "Restaurar a verificação da FIPS 203 (CRY9) e rodar os testes.");
    }
    // Assinatura: confere a própria, recusa outra mensagem e outro contexto; variante hedged.
    let k = assinatura::gerar();
    match (k.assinar(b"sentinela", b"autoteste"), k.assinar(b"sentinela", b"autoteste")) {
        (Ok(s1), Ok(s2)) => {
            let v = k.verificacao();
            if !v.verificar(b"sentinela", b"autoteste", &s1) || v.verificar(b"sentinela", b"outra", &s1) || v.verificar(b"outro", b"autoteste", &s1) {
                r.acusar("SEN-ASS-1", "assinatura", "verificação aceita ou recusa errado".into(), "Não atender: reinstalar do build verificado e acionar a auditoria.");
            }
            if s1.partes().1 == s2.partes().1 { r.acusar("SEN-ASS-2", "assinatura::ChaveDeAssinatura::assinar", "ML-DSA determinístico (sem hedge)".into(), "Restaurar a variante hedged (PQS7)."); }
        }
        _ => r.acusar("SEN-ASS-0", "assinatura", "não conseguiu assinar".into(), "Não atender: reinstalar do build verificado."),
    }
    r
}

/// Partida do servidor e dos apps em produção: tudo de `autoverificar` + o selo de liberação do artefato que
/// está rodando. Sem selo válido (portão faltando ou reprovado, artefato diferente, menos de três chaves,
/// versão antiga), não funciona.
pub fn autoverificar_com_selo(s: &selo::Selo, assinaturas: &[([u8; 32], assinatura::AssinaturaHibrida)], artefato_rodando: &[u8; 64],
                              chaves_fixadas: &[assinatura::ChaveDeVerificacao], versao_minima: u64, agora: u64) -> Relatorio {
    let mut r = autoverificar();
    if let Err(e) = selo::conferir(s, assinaturas, artefato_rodando, chaves_fixadas, versao_minima, agora) {
        r.acusar("SEN-SELO", "selo de liberação", format!("{e:?}"),
                 "Não rodar este artefato: gerá-lo pelo CI com todos os portões aprovados e o selo assinado por três chaves diferentes (ver GUIA_DE_LIBERACAO.md).");
    }
    r
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn sistema_atual_passa_em_tudo() {
        let r = autoverificar();
        assert!(r.pode_funcionar(), "{}", r.texto());
        assert!(r.texto().contains("tudo conferido"));
    }

    #[test]
    fn relatorio_trava_com_qualquer_achado_e_diz_onde_e_como() {
        let mut r = Relatorio::default();
        r.acusar("SEN-X", "modulo::funcao", "problema de exemplo".into(), "faça isto");
        assert!(!r.pode_funcionar());
        let t = r.texto();
        assert!(t.contains("TRAVADO") && t.contains("SEN-X") && t.contains("modulo::funcao") && t.contains("Como corrigir: faça isto"));
    }

    #[test]
    fn invariantes_atuais_conferem() {
        assert!(conferir_invariantes().pode_funcionar(), "{}", conferir_invariantes().texto());
    }

    #[test]
    fn cada_invariante_violada_trava() {
        use crate::autenticacao::{Exigencia, Rota};
        static SEM_LIMITE: [Rota; 1] = [Rota { metodo: "POST", caminho: "/v1/x", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 0 }];
        static TORTA: [Rota; 1] = [Rota { metodo: "POST", caminho: "/v1/X/", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 }];
        static MAIUSCULA: [Rota; 1] = [Rota { metodo: "POST", caminho: "/v1/Caixa", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 }];
        static PERCENT: [Rota; 1] = [Rota { metodo: "POST", caminho: "/v1/%61", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 }];
        static PONTOS: [Rota; 1] = [Rota { metodo: "POST", caminho: "/v1/a/../b", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 }];
        static DUPLA: [Rota; 1] = [Rota { metodo: "POST", caminho: "/v1//a", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 }];
        static SEM_V1: [Rota; 1] = [Rota { metodo: "POST", caminho: "/admin", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 }];
        static REPETIDA: [Rota; 2] = [Rota { metodo: "POST", caminho: "/v1/a", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 },
                                      Rota { metodo: "POST", caminho: "/v1/a", exigencia: Exigencia::Publica, corpo_maximo: 1, por_minuto: 1 }];
        let base = Valores::atuais();
        let casos: Vec<(&str, Valores)> = vec![
            ("SEN-ROTA-0", Valores { rotas: &[], ..base }),
            ("SEN-ROTA-1", Valores { rotas: &SEM_LIMITE, ..base }),
            ("SEN-ROTA-1", Valores { teto_por_minuto: 10_000, ..base }),
            ("SEN-ROTA-2", Valores { rotas: &TORTA, ..base }),
            ("SEN-ROTA-2", Valores { rotas: &MAIUSCULA, ..base }),
            ("SEN-ROTA-2", Valores { rotas: &PERCENT, ..base }),
            ("SEN-ROTA-2", Valores { rotas: &PONTOS, ..base }),
            ("SEN-ROTA-2", Valores { rotas: &DUPLA, ..base }),
            ("SEN-ROTA-2", Valores { rotas: &SEM_V1, ..base }),
            ("SEN-ROTA-3", Valores { rotas: &REPETIDA, ..base }),
            ("SEN-JAN", Valores { janela: 61, ..base }),
            ("SEN-PRAZO", Valores { vida_maxima: 48 * 3600 + 1, ..base }),
            ("SEN-PRAZO", Valores { prazo_direto: 49 * 3600, ..base }),
            ("SEN-PRAZO", Valores { limite_para_ler: 49 * 3600, ..base }),
            ("SEN-HORA", Valores { limite_sem_hora: 15 * 60 + 1, ..base }),
            ("SEN-FS", Valores { prechave_validade: 8 * 24 * 3600, ..base }),
            ("SEN-FS", Valores { prechave_sobrevida: 49 * 3600, ..base }),
            ("SEN-REC", Valores { carencia_recuperacao: 72 * 3600 - 1, ..base }),
            ("SEN-CORP", Valores { carencia_corporativa: 24 * 3600 - 1, ..base }),
            ("SEN-SENHA", Valores { senha_memoria_kib: 19 * 1024 - 1, ..base }),
            ("SEN-SENHA", Valores { senha_passagens: 1, ..base }),
            ("SEN-SENHA", Valores { senha_faixas: 0, ..base }),
        ];
        for (codigo, v) in casos {
            let r = conferir_valores(&v);
            assert!(!r.pode_funcionar() && r.achados.iter().any(|a| a.codigo == codigo), "{codigo} não travou");
        }
        assert!(conferir_valores(&base).pode_funcionar());
    }


    #[test]
    fn sem_selo_valido_o_sistema_nao_anda() {
        let (ci, seg, aud) = (assinatura::gerar(), assinatura::gerar(), assinatura::gerar());
        let fix = [ci.verificacao(), seg.verificacao(), aud.verificacao()];
        let mut s = selo::Selo { versao: 1, commit: "abc".into(), resumo_do_artefato: selo::resumir(b"servidor"), emitido_em: 10,
                                 portoes: selo::PORTOES_OBRIGATORIOS.iter().map(|p| ((*p).to_string(), true)).collect() };
        let ass = vec![selo::assinar(&ci, &s).unwrap(), selo::assinar(&seg, &s).unwrap(), selo::assinar(&aud, &s).unwrap()];
        assert!(autoverificar_com_selo(&s, &ass, &selo::resumir(b"servidor"), &fix, 1, 10).pode_funcionar());
        s.portoes.retain(|(n, _)| n != "protecao_do_repositorio");
        let r = autoverificar_com_selo(&s, &ass, &selo::resumir(b"servidor"), &fix, 1, 10);
        assert!(!r.pode_funcionar() && r.texto().contains("SEN-SELO"));
    }

}
