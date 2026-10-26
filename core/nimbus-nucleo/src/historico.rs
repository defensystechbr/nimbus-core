//! # Módulo `historico` — histórico guardado no servidor, aberto só no aparelho (EF5, EF6, CH4)
//!
//! Responde à pergunta: dá para deixar tudo no servidor e só abrir no celular da pessoa? **Sim.**
//!
//! * O servidor guarda **blocos cifrados** com identificadores aleatórios: não sabe de quem são,
//!   de que conversa são, nem o que contêm. O tamanho é padronizado pelo `envelope`.
//! * A chave fica só com o dono. No app real, a chave-mestra é protegida pelo chip de segurança do
//!   aparelho; o celular baixa o bloco, abre **na memória**, mostra e apaga. Nada vai para o disco.
//! * **Destruição por janela de tempo (apagamento criptográfico):** o conteúdo é cifrado com a chave da
//!   janela em que ele expira. Quando a janela vence, a chave é destruída no aparelho. A partir daí,
//!   mesmo que alguém tenha guardado cópia do bloco — no servidor ou em qualquer lugar —, ele fica
//!   ilegível para sempre. Não depende de confiar que o servidor apagou.
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use rand_core::{OsRng, RngCore};
use std::collections::{BTreeMap, HashMap};
use zeroize::Zeroizing;

/// Identificador aleatório do bloco no servidor (não liga o bloco a conta nem a conversa).
pub type IdDeBloco = [u8; 32];

/// Simula o servidor: só vê identificadores aleatórios e bytes cifrados.
#[derive(Default)]
pub struct ServidorCego { pub blocos: HashMap<IdDeBloco, Vec<u8>> }

impl core::fmt::Debug for ServidorCego {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ServidorCego({} blocos opacos)", self.blocos.len()) }
}

/// Chaves que ficam com o dono. `janelas` = uma chave por janela de expiração (por exemplo, por hora).
pub struct ChavesDoDono {
    permanente: (ChavePublica, ChaveSecreta),
    janelas: BTreeMap<u64, (ChavePublica, ChaveSecreta)>,
    pub duracao_da_janela: u64,
}

impl core::fmt::Debug for ChavesDoDono {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ChavesDoDono(<ocultas>, {} janelas ativas)", self.janelas.len()) }
}

impl ChavesDoDono {
    pub fn novas(duracao_da_janela: u64) -> Self {
        ChavesDoDono { permanente: envelope::gerar_par_de_chaves(), janelas: BTreeMap::new(), duracao_da_janela: duracao_da_janela.max(1) }
    }

    fn janela_de(&self, expira_em: u64) -> u64 {
        // Arredonda para cima: o bloco vive até o fim da janela em que expira.
        expira_em.div_ceil(self.duracao_da_janela)
    }

    /// Destrói as chaves de todas as janelas vencidas até `agora`. Devolve quantas foram destruídas.
    pub fn destruir_janelas_vencidas(&mut self, agora: u64) -> usize {
        let limite = agora / self.duracao_da_janela;
        let vencidas: Vec<u64> = self.janelas.range(..=limite).map(|(k, _)| *k).collect();
        for k in &vencidas { let _ = self.janelas.remove(k); } // as chaves se zeram ao sair de escopo
        vencidas.len()
    }

    pub fn janelas_ativas(&self) -> usize { self.janelas.len() }

    /// Apagar a conta: destrói todas as chaves (permanente e de janelas). Todo bloco já guardado —
    /// no servidor ou em qualquer cópia — vira ruído para sempre. Consome o valor: não há como usar depois.
    pub fn apagar_conta(self) {}
}

fn novo_id() -> IdDeBloco { let mut id = [0u8; 32]; OsRng.fill_bytes(&mut id); id }

/// Guarda um conteúdo no servidor. `expira_em = None` usa a chave permanente.
pub fn guardar(servidor: &mut ServidorCego, chaves: &mut ChavesDoDono, conteudo: &[u8], expira_em: Option<u64>) -> Result<IdDeBloco, envelope::Erro> {
    let (publica, marcador) = match expira_em {
        None => (&chaves.permanente.0, 0u64),
        Some(t) => {
            let j = chaves.janela_de(t);
            let par = chaves.janelas.entry(j).or_insert_with(envelope::gerar_par_de_chaves);
            (&par.0, j)
        }
    };
    let bloco = envelope::selar(publica, conteudo, &marcador.to_be_bytes())?;
    let mut com_marcador = marcador.to_be_bytes().to_vec();
    com_marcador.extend_from_slice(&bloco);
    let id = novo_id();
    let _ = servidor.blocos.insert(id, com_marcador);
    Ok(id)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeLeitura { NaoEncontrado, ChaveDestruida, Ilegivel }

/// Lê no aparelho: devolve o conteúdo numa área de memória que se zera ao ser descartada.
pub fn ler(servidor: &ServidorCego, chaves: &ChavesDoDono, id: &IdDeBloco) -> Result<Zeroizing<Vec<u8>>, ErroDeLeitura> {
    let dado = servidor.blocos.get(id).ok_or(ErroDeLeitura::NaoEncontrado)?;
    if dado.len() < 8 { return Err(ErroDeLeitura::Ilegivel); }
    let mut t = [0u8; 8];
    t.copy_from_slice(&dado[..8]);
    let marcador = u64::from_be_bytes(t);
    let par = if marcador == 0 { &chaves.permanente } else { chaves.janelas.get(&marcador).ok_or(ErroDeLeitura::ChaveDestruida)? };
    envelope::abrir(&par.1, &par.0, &dado[8..], &marcador.to_be_bytes())
        .map_err(|_| ErroDeLeitura::Ilegivel)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn guarda_no_servidor_e_abre_so_com_a_chave_do_dono() {
        let (mut s, mut c) = (ServidorCego::default(), ChavesDoDono::novas(3600));
        let id = guardar(&mut s, &mut c, b"foto do contrato", None).unwrap();
        assert_eq!(&ler(&s, &c, &id).unwrap()[..], b"foto do contrato");
        let outro = ChavesDoDono::novas(3600);
        assert_eq!(ler(&s, &outro, &id), Err(ErroDeLeitura::Ilegivel));
    }

    #[test]
    fn servidor_nunca_ve_o_conteudo() {
        let (mut s, mut c) = (ServidorCego::default(), ChavesDoDono::novas(3600));
        let _ = guardar(&mut s, &mut c, b"SEGREDO-VISIVEL", None).unwrap();
        for bloco in s.blocos.values() {
            assert!(!bloco.windows(15).any(|w| w == b"SEGREDO-VISIVEL"));
        }
    }

    #[test]
    fn identificadores_nao_revelam_ligacao_entre_mensagens() {
        let (mut s, mut c) = (ServidorCego::default(), ChavesDoDono::novas(3600));
        let a = guardar(&mut s, &mut c, b"mensagem 1", None).unwrap();
        let b = guardar(&mut s, &mut c, b"mensagem 2", None).unwrap();
        assert_ne!(a, b);
        assert_ne!(a[..8], b[..8]);
    }

    #[test]
    fn janela_vencida_torna_copias_ilegiveis_para_sempre() {
        let (mut s, mut c) = (ServidorCego::default(), ChavesDoDono::novas(3600));
        let id = guardar(&mut s, &mut c, b"temporaria", Some(5000)).unwrap();
        let copia_guardada_por_alguem = s.blocos.clone();
        assert!(ler(&s, &c, &id).is_ok());
        assert_eq!(c.destruir_janelas_vencidas(7200), 1);
        let s2 = ServidorCego { blocos: copia_guardada_por_alguem };
        assert_eq!(ler(&s2, &c, &id), Err(ErroDeLeitura::ChaveDestruida), "mesmo com cópia, não abre");
    }

    #[test]
    fn janela_ainda_valida_nao_e_destruida() {
        let (mut s, mut c) = (ServidorCego::default(), ChavesDoDono::novas(3600));
        let id = guardar(&mut s, &mut c, b"ainda vale", Some(10_000)).unwrap();
        let _ = c.destruir_janelas_vencidas(3600);
        assert!(ler(&s, &c, &id).is_ok());
        assert_eq!(c.janelas_ativas(), 1);
    }

    #[test]
    fn apagar_a_conta_deixa_tudo_ilegivel_para_sempre() {
        let (mut s, mut c) = (ServidorCego::default(), ChavesDoDono::novas(3600));
        let id = guardar(&mut s, &mut c, b"tudo da minha vida", None).unwrap();
        let copia = s.blocos.clone();
        c.apagar_conta();
        // Nenhuma chave existe mais; uma conta nova não abre nada da antiga.
        let nova = ChavesDoDono::novas(3600);
        assert_eq!(ler(&ServidorCego { blocos: copia }, &nova, &id), Err(ErroDeLeitura::Ilegivel));
    }

    #[test]
    fn tamanhos_padronizados_no_servidor() {
        let (mut s, mut c) = (ServidorCego::default(), ChavesDoDono::novas(3600));
        let a = guardar(&mut s, &mut c, b"ok", None).unwrap();
        let b = guardar(&mut s, &mut c, &[1u8; 150], None).unwrap();
        assert_eq!(s.blocos[&a].len(), s.blocos[&b].len());
    }
}
