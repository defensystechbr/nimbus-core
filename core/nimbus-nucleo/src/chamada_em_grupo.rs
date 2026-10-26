//! # Módulo `chamada_em_grupo` — chamada de áudio em grupo como MALHA de chamadas 1:1 (v37, terceira rodada)
//!
//! **A decisão de desenho, e por que é a de menor superfície de ataque possível**: em vez de inventar um
//! protocolo novo de sinalização multi-participante (o caminho normal seria um SFU — um servidor de mídia
//! que recebe o áudio de todos e reencaminha — algo com desenho e implementação próprios, logo uma
//! superfície de ataque nova, e que colocaria um servidor vendo metadados de QUEM fala com QUEM dentro do
//! grupo inteiro de uma vez), este módulo trata uma chamada em grupo como **N chamadas 1:1 independentes**
//! (uma "perna" por participante), cada uma usando `chamada::Chamada` **exatamente como já existe, sem
//! nenhuma mudança** — a mesma máquina de estados já testada e endurecida contra o CVE-2019-17191
//! (atendimento forçado pela rede). Nenhuma perna consegue ativar nenhuma outra: cada pessoa continua
//! precisando tocar em "atender" para a SUA própria perna, e uma perna recusada ou não atendida não afeta
//! as demais.
//!
//! **O limite honesto desta escolha**: malha não escala. Com N participantes, existem N(N-1)/2 pernas, e
//! cada aparelho precisa enviar e receber áudio de TODOS os outros ao mesmo tempo (banda e processamento
//! de O(N) por aparelho). Por isso há um teto (`MAX_PARTICIPANTES`, o mesmo de `grupo::MAX_MEMBROS` — não
//! por acaso: os dois tetos existem pelo mesmo motivo prático). Acima disso, a arquitetura certa é um SFU
//! — mas isso é um projeto novo e maior, com o próprio modelo de ameaça (o SFU vê quem fala e quando,
//! mesmo sem decifrar o conteúdo), e NÃO foi tentado nesta rodada: seria exatamente o tipo de coisa
//! apressada que aumenta risco em vez de reduzir. Para "pelo menos 3 pessoas, agora", a malha é honesta,
//! seminova, e reaproveita 100% do que já foi testado — para grupos maiores, é um projeto futuro distinto.
use crate::chamada::{Chamada, Estado, Evento};
use std::collections::HashMap;

pub type IdParticipante = [u8; 32];

/// Mesmo teto de `grupo::MAX_MEMBROS`, pelo motivo de banda explicado acima.
pub const MAX_PARTICIPANTES: usize = crate::grupo::MAX_MEMBROS;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeChamadaEmGrupo { GrupoCheio, JaNaChamada, NaoNaChamada }

/// Uma chamada em grupo: um mapa de participante → a PRÓPRIA perna 1:1 dele, sem nenhum estado
/// compartilhado entre pernas além deste agrupamento.
#[derive(Debug, Default)]
pub struct ChamadaEmGrupo { pernas: HashMap<IdParticipante, Chamada> }

impl ChamadaEmGrupo {
    pub fn nova() -> Self { ChamadaEmGrupo::default() }

    /// Convida um novo participante: cria a perna dele, começando a tocar (microfone desligado), como
    /// qualquer chamada 1:1 recebida. Não afeta nenhuma perna já existente.
    pub fn convidar(&mut self, quem: IdParticipante) -> Result<(), ErroDeChamadaEmGrupo> {
        if self.pernas.len() >= MAX_PARTICIPANTES { return Err(ErroDeChamadaEmGrupo::GrupoCheio); }
        if self.pernas.contains_key(&quem) { return Err(ErroDeChamadaEmGrupo::JaNaChamada); }
        self.pernas.insert(quem, Chamada::recebida());
        Ok(())
    }

    /// Aplica um evento só na perna DAQUELE participante — nunca nas outras. É a mesma garantia de
    /// `chamada::Chamada::aplicar`, só roteada por participante.
    pub fn aplicar(&mut self, quem: &IdParticipante, evento: Evento) -> Result<Estado, ErroDeChamadaEmGrupo> {
        match self.pernas.get_mut(quem) {
            Some(perna) => Ok(perna.aplicar(evento)),
            None => Err(ErroDeChamadaEmGrupo::NaoNaChamada),
        }
    }

    /// Quem está com áudio ligado agora: só quem tem a PRÓPRIA perna em `Estado::Ativa`. É exatamente
    /// essa lista que a ponte nativa usa para decidir de quem tocar o áudio recebido — nunca uma lista
    /// "do grupo todo", sempre derivada perna a perna.
    pub fn participantes_com_audio_ligado(&self) -> Vec<IdParticipante> {
        self.pernas.iter().filter(|(_, c)| c.microfone_ligado()).map(|(id, _)| *id).collect()
    }

    /// A chamada em grupo como um todo está "ativa" se pelo menos uma perna está ativa (ninguém sozinho
    /// "encerra a chamada para todo mundo" — cada um só encerra a própria perna, como numa ligação normal).
    pub fn alguma_perna_ativa(&self) -> bool { self.pernas.values().any(|c| c.estado() == Estado::Ativa) }

    pub fn tamanho(&self) -> usize { self.pernas.len() }

    /// Remove a perna de quem saiu (equivalente a desligar aquela chamada 1:1 especificamente).
    pub fn remover(&mut self, quem: &IdParticipante) -> Result<(), ErroDeChamadaEmGrupo> {
        self.pernas.remove(quem).map(|_| ()).ok_or(ErroDeChamadaEmGrupo::NaoNaChamada)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    const A: IdParticipante = [1; 32];
    const B: IdParticipante = [2; 32];
    const C: IdParticipante = [3; 32];

    #[test]
    fn convida_tres_e_cada_perna_comeca_tocando_com_microfone_desligado() {
        let mut g = ChamadaEmGrupo::nova();
        for p in [A, B, C] { g.convidar(p).unwrap(); }
        assert_eq!(g.tamanho(), 3);
        assert!(g.participantes_com_audio_ligado().is_empty(), "ninguém atendeu ainda");
    }

    #[test]
    fn so_quem_atende_a_propria_perna_fica_com_audio_ligado_os_outros_nao() {
        let mut g = ChamadaEmGrupo::nova();
        for p in [A, B, C] { g.convidar(p).unwrap(); }
        g.aplicar(&A, Evento::Aceitar).unwrap();
        let ligados = g.participantes_com_audio_ligado();
        assert_eq!(ligados, vec![A], "só A atendeu; B e C continuam tocando, sem áudio");
        assert!(g.alguma_perna_ativa());
    }

    #[test]
    fn mensagem_da_rede_nunca_atende_nenhuma_perna_de_ninguem() {
        // A mesma garantia do CVE-2019-17191 que já existe em chamada::Chamada, agora checada perna a
        // perna: nem uma avalanche de "mensagens da rede" em todas as pernas liga qualquer microfone.
        let mut g = ChamadaEmGrupo::nova();
        for p in [A, B, C] { g.convidar(p).unwrap(); }
        for p in [A, B, C] { for _ in 0..100 { g.aplicar(&p, Evento::MensagemDaRede).unwrap(); } }
        assert!(g.participantes_com_audio_ligado().is_empty());
        assert!(!g.alguma_perna_ativa());
    }

    #[test]
    fn recusar_ou_sair_so_afeta_a_propria_perna() {
        let mut g = ChamadaEmGrupo::nova();
        for p in [A, B, C] { g.convidar(p).unwrap(); }
        g.aplicar(&A, Evento::Aceitar).unwrap();
        g.aplicar(&B, Evento::Recusar).unwrap();
        assert_eq!(g.participantes_com_audio_ligado(), vec![A], "A continua com áudio mesmo com B recusando");
        g.remover(&B).unwrap();
        assert_eq!(g.tamanho(), 2);
        assert!(g.alguma_perna_ativa(), "A ainda ativa, mesmo depois de B sair");
    }

    #[test]
    fn nao_convida_o_mesmo_duas_vezes_nem_alem_do_teto() {
        let mut g = ChamadaEmGrupo::nova();
        g.convidar(A).unwrap();
        assert_eq!(g.convidar(A), Err(ErroDeChamadaEmGrupo::JaNaChamada));
        for i in 1..MAX_PARTICIPANTES as u8 { g.convidar([i + 10; 32]).unwrap(); }
        assert_eq!(g.tamanho(), MAX_PARTICIPANTES);
        assert_eq!(g.convidar([250; 32]), Err(ErroDeChamadaEmGrupo::GrupoCheio));
    }

    #[test]
    fn aplicar_evento_em_quem_nao_esta_na_chamada_e_recusado() {
        let mut g = ChamadaEmGrupo::nova();
        g.convidar(A).unwrap();
        assert_eq!(g.aplicar(&B, Evento::Aceitar), Err(ErroDeChamadaEmGrupo::NaoNaChamada));
    }
}
