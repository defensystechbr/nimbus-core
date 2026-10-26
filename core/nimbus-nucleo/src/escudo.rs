//! # Módulo `escudo` — Escudo de Conversa (requisitos EC1–EC9)
//!
//! Enquanto uma conversa está aberta, cada aparelho participante envia aos demais, pelo canal cifrado,
//! um **estado de segurança assinado** pela chave do aparelho: nível (verde/amarelo/vermelho) e se há
//! captura de tela ativa. Cada cliente decidir localmente o que mostrar:
//!
//! * todos verdes e sem captura → conteúdo visível;
//! * alguém capturando, amarelo/vermelho, estado vencido, assinatura inválida ou repetição → **borrar**
//!   a conversa em todos os aparelhos e avisar quem causou (princípio "na dúvida, fecha").
//!
//! Limites (declarados no Manual): não impede câmera externa; um cliente adulterado pode mentir — por isso
//! a conexão exige atestação (App Attest / Play Integrity) e o servidor recusa clientes não atestados.
use crate::integridade::Nivel;
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};

const CONTEXTO: &[u8] = b"NimbusOne/v1/escudo";
use std::collections::HashMap;

/// Validade máxima de um estado. Sem novo estado nesse prazo, o participante é tratado como inseguro.
pub const VALIDADE_SEGUNDOS: u64 = 10;
/// Diferença máxima aceita entre o relógio do outro aparelho e o deste (ambos sincronizados com o servidor).
pub const TOLERANCIA_DE_RELOGIO: u64 = 30;

#[derive(Clone, Debug)]
pub struct EstadoDoParticipante {
    pub participante: [u8; 32],
    pub conversa: [u8; 16],
    pub sequencia: u64,
    pub carimbo: u64,
    pub nivel: Nivel,
    pub captura_ativa: bool,
    pub assinatura: AssinaturaHibrida,
}

fn codificar(participante: &[u8; 32], conversa: &[u8; 16], sequencia: u64, carimbo_visto: u64, nivel: Nivel, captura: bool) -> Vec<u8> {
    let mut v = b"NimbusOne/v1/shield".to_vec();
    v.extend_from_slice(participante);
    v.extend_from_slice(conversa);
    v.extend_from_slice(&sequencia.to_be_bytes());
    v.extend_from_slice(&carimbo_visto.to_be_bytes());
    v.push(nivel as u8);
    v.push(captura as u8);
    v
}

/// Aparelho: assina o próprio estado.
pub fn assinar_estado(chave: &ChaveDeAssinatura, participante: [u8; 32], conversa: [u8; 16], sequencia: u64, carimbo: u64, nivel: Nivel, captura_ativa: bool) -> EstadoDoParticipante {
    let assinatura = chave.assinar(CONTEXTO, &codificar(&participante, &conversa, sequencia, carimbo, nivel, captura_ativa)).expect("contexto fixo e curto");
    EstadoDoParticipante { participante, conversa, sequencia, carimbo, nivel, captura_ativa, assinatura }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Motivo { CapturaAtiva, NaoEstaVerde, EstadoVencido, AssinaturaInvalida, RepeticaoDeEstado, SemEstado }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Acao {
    Mostrar,
    /// Borrar a conversa e avisar: quem e por quê.
    Borrar(Vec<([u8; 32], Motivo)>),
}

/// Estado da conversa num aparelho.
#[derive(Debug)]
pub struct Escudo {
    conversa: [u8; 16],
    /// Participante → chave pública do aparelho (vinda do registro público de chaves).
    chaves: HashMap<[u8; 32], ChaveDeVerificacao>,
    /// sequência, momento em que **este aparelho** recebeu o estado, nível, captura.
    ultimo: HashMap<[u8; 32], (u64, u64, Nivel, bool)>,
    problemas: HashMap<[u8; 32], Motivo>,
}

impl Escudo {
    pub fn novo(conversa: [u8; 16], participantes: HashMap<[u8; 32], ChaveDeVerificacao>) -> Self {
        Escudo { conversa, chaves: participantes, ultimo: HashMap::new(), problemas: HashMap::new() }
    }

    /// Recebe o estado de um participante. `agora` é o relógio deste aparelho, sincronizado com o servidor.
    ///
    /// A validade é contada a partir do **recebimento aqui**, não do carimbo enviado pelo outro lado:
    /// assim, um carimbo adiantado não mantém um estado “fresco” para sempre. Carimbo mais de
    /// `TOLERANCIA_DE_RELOGIO` segundos no futuro é tratado como estado inválido.
    pub fn receber(&mut self, s: &EstadoDoParticipante, agora: u64) {
        let Some(chave_publica) = self.chaves.get(&s.participante) else { return };
        if s.conversa != self.conversa { return; }
        let msg = codificar(&s.participante, &s.conversa, s.sequencia, s.carimbo, s.nivel, s.captura_ativa);
        if !chave_publica.verificar(CONTEXTO, &msg, &s.assinatura) {
            let _ = self.problemas.insert(s.participante, Motivo::AssinaturaInvalida);
            return;
        }
        if s.carimbo > agora.saturating_add(TOLERANCIA_DE_RELOGIO) {
            let _ = self.problemas.insert(s.participante, Motivo::AssinaturaInvalida);
            return;
        }
        if let Some((sequencia, _, _, _)) = self.ultimo.get(&s.participante) {
            if s.sequencia <= *sequencia { let _ = self.problemas.insert(s.participante, Motivo::RepeticaoDeEstado); return; }
        }
        let _ = self.problemas.remove(&s.participante);
        let _ = self.ultimo.insert(s.participante, (s.sequencia, agora, s.nivel, s.captura_ativa));
    }

    /// Decide o que mostrar agora (`agora` = relógio sincronizado com o servidor).
    pub fn decidir(&self, agora: u64) -> Acao {
        let mut saida = vec![];
        let mut ids: Vec<_> = self.chaves.keys().copied().collect();
        ids.sort();
        for p in ids {
            if let Some(r) = self.problemas.get(&p) { saida.push((p, r.clone())); continue; }
            match self.ultimo.get(&p) {
                None => saida.push((p, Motivo::SemEstado)),
                Some((_, recebido_em, nivel, captura)) => {
                    if agora.saturating_sub(*recebido_em) > VALIDADE_SEGUNDOS { saida.push((p, Motivo::EstadoVencido)); }
                    else if *captura { saida.push((p, Motivo::CapturaAtiva)); }
                    else if *nivel != Nivel::Verde { saida.push((p, Motivo::NaoEstaVerde)); }
                }
            }
        }
        if saida.is_empty() { Acao::Mostrar } else { Acao::Borrar(saida) }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;
    const CONV: [u8; 16] = [9; 16];
    const ANA: [u8; 32] = [1; 32];
    const BIA: [u8; 32] = [2; 32];

    fn preparar() -> (ChaveDeAssinatura, ChaveDeAssinatura, Escudo) {
        let ka = assinatura::gerar();
        let kb = assinatura::gerar();
        let mut m = HashMap::new();
        let _ = m.insert(ANA, ka.verificacao());
        let _ = m.insert(BIA, kb.verificacao());
        (ka, kb, Escudo::novo(CONV, m))
    }

    #[test]
    fn todos_verdes_mostra() {
        let (ka, kb, mut sh) = preparar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 100, Nivel::Verde, false), 100);
        assert_eq!(sh.decidir(105), Acao::Mostrar);
    }

    #[test]
    fn captura_no_outro_aparelho_borra_e_diz_quem() {
        let (ka, kb, mut sh) = preparar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 100, Nivel::Verde, true), 100);
        assert_eq!(sh.decidir(101), Acao::Borrar(vec![(BIA, Motivo::CapturaAtiva)]));
    }

    #[test]
    fn captura_encerrada_volta_a_mostrar() {
        let (ka, kb, mut sh) = preparar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 100, Nivel::Verde, true), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 2, 103, Nivel::Verde, false), 103);
        assert_eq!(sh.decidir(104), Acao::Mostrar);
    }

    #[test]
    fn silencio_vencido_borra() {
        let (ka, kb, mut sh) = preparar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 100, Nivel::Verde, false), 100);
        assert_eq!(sh.decidir(100 + VALIDADE_SEGUNDOS + 1), Acao::Borrar(vec![(ANA, Motivo::EstadoVencido), (BIA, Motivo::EstadoVencido)]));
    }

    #[test]
    fn estado_falsificado_borra() {
        let (ka, _kb, mut sh) = preparar();
        let intruso = assinatura::gerar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&intruso, BIA, CONV, 1, 100, Nivel::Verde, false), 100);
        assert_eq!(sh.decidir(101), Acao::Borrar(vec![(BIA, Motivo::AssinaturaInvalida)]));
    }

    #[test]
    fn repeticao_de_estado_antigo_borra() {
        let (ka, kb, mut sh) = preparar();
        let velho_verde = assinar_estado(&kb, BIA, CONV, 1, 100, Nivel::Verde, false);
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&velho_verde, 101);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 2, 101, Nivel::Verde, true), 101);
        sh.receber(&velho_verde, 101);
        assert!(matches!(sh.decidir(102), Acao::Borrar(v) if v.contains(&(BIA, Motivo::RepeticaoDeEstado))));
    }

    #[test]
    fn repeticao_com_a_mesma_sequencia_tambem_borra() {
        let (ka, kb, mut sh) = preparar();
        let estado = assinar_estado(&kb, BIA, CONV, 5, 100, Nivel::Verde, false);
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&estado, 100);
        sh.receber(&estado, 101);
        assert_eq!(sh.decidir(102), Acao::Borrar(vec![(BIA, Motivo::RepeticaoDeEstado)]));
    }

    #[test]
    fn participante_amarelo_borra() {
        let (ka, kb, mut sh) = preparar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 100, Nivel::Amarelo, false), 100);
        assert_eq!(sh.decidir(101), Acao::Borrar(vec![(BIA, Motivo::NaoEstaVerde)]));
    }

    #[test]
    fn carimbo_no_futuro_nao_segura_estado_fresco() {
        let (ka, kb, mut sh) = preparar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        // Bia manda um carimbo 1 hora à frente: recusado.
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 3_700, Nivel::Verde, false), 100);
        assert_eq!(sh.decidir(101), Acao::Borrar(vec![(BIA, Motivo::AssinaturaInvalida)]));
    }

    #[test]
    fn validade_conta_da_chegada_e_nao_do_carimbo_enviado() {
        let (ka, kb, mut sh) = preparar();
        // Carimbos dentro da tolerância, mas a validade vale a partir da chegada (100).
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 120, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 125, Nivel::Verde, false), 100);
        assert_eq!(sh.decidir(105), Acao::Mostrar);
        assert!(matches!(sh.decidir(100 + VALIDADE_SEGUNDOS + 1), Acao::Borrar(_)));
    }

    #[test]
    fn sem_estado_ainda_borra() {
        let (ka, _kb, mut sh) = preparar();
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        assert_eq!(sh.decidir(101), Acao::Borrar(vec![(BIA, Motivo::SemEstado)]));
    }

    // ---- Prontidão para grupo (mais de 2 participantes): o Escudo já é genérico por construção
    // (`chaves`/`ultimo`/`problemas` são HashMaps por participante, não um par fixo) — este teste prova
    // isso com 3 participantes em vez de supor que generaliza. ----

    #[test]
    fn grupo_de_tres_so_mostra_se_todos_estiverem_verdes() {
        let ka = assinatura::gerar();
        let kb = assinatura::gerar();
        let kc = assinatura::gerar();
        const CARLOS: [u8; 32] = [9; 32];
        let mut m = HashMap::new();
        let _ = m.insert(ANA, ka.verificacao());
        let _ = m.insert(BIA, kb.verificacao());
        let _ = m.insert(CARLOS, kc.verificacao());
        let mut sh = Escudo::novo(CONV, m);
        sh.receber(&assinar_estado(&ka, ANA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kb, BIA, CONV, 1, 100, Nivel::Verde, false), 100);
        sh.receber(&assinar_estado(&kc, CARLOS, CONV, 1, 100, Nivel::Verde, false), 100);
        assert_eq!(sh.decidir(101), Acao::Mostrar, "os 3 verdes: mostra");
        // Um terceiro participante capturando a tela borra para o GRUPO TODO, não só para ele.
        sh.receber(&assinar_estado(&kc, CARLOS, CONV, 2, 101, Nivel::Verde, true), 101);
        assert_eq!(sh.decidir(102), Acao::Borrar(vec![(CARLOS, Motivo::CapturaAtiva)]), "um só capturando já borra para todos, mesmo em grupo");
    }
}
