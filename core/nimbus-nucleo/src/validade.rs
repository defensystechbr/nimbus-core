//! # Módulo `validade` — apagar sozinho: agenda, notas, contatos, arquivos e fotos (VAL1–VAL8)
//!
//! Todo item dos apps Nimbus pode ter **validade**: depois de algumas horas, dias ou meses, ele
//! deixa de existir — sem rastro, inclusive nas cópias que alguém tenha guardado.
//!
//! Como funciona, sem depender de confiar em ninguém:
//! 1. O app calcula o instante de apagar (`instante_de_apagar`).
//! 2. O item é guardado no servidor cego cifrado com a **chave da janela de tempo** desse instante
//!    (módulo `historico`).
//! 3. Quando a janela vence, o aparelho destrói a chave. Qualquer cópia do bloco — no servidor, num
//!    backup, num grampo — fica ilegível para sempre (apagamento criptográfico).
//!
//! Duas referências de tempo:
//! * **a partir da criação** — notas, contatos, arquivos, fotos, itens do cofre;
//! * **a partir do fim do compromisso** — agenda: “apagar 3 horas depois que a reunião acabar”.
//!
//! Precisão declarada: o app esconde o item no instante exato; o apagamento criptográfico acontece ao
//! fim da janela (ex.: janela de 1 hora → até 1 hora depois). A janela é configurável.

use crate::historico::{self, ChavesDoDono, IdDeBloco, ServidorCego};

pub const MINUTO: u64 = 60;
pub const HORA: u64 = 3_600;
pub const DIA: u64 = 86_400;
pub const SEMANA: u64 = 7 * DIA;
/// Mês de 30 dias. O calendário real (28–31 dias) é calculado na camada nativa, se o usuário escolher
/// uma data exata; aqui a regra é por duração.
pub const MES_30_DIAS: u64 = 30 * DIA;
/// Limite máximo de validade: 10 anos (evita erro de digitação virar “nunca apaga” por acidente).
pub const VALIDADE_MAXIMA: u64 = 3_650 * DIA;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoDeItem { Compromisso, Nota, Contato, Arquivo, Foto, ItemDoCofre }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Referencia { Criacao, FimDoCompromisso }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Validade { pub apos_segundos: u64, pub referencia: Referencia }

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeValidade {
    /// “Depois do fim do compromisso” exige que o item seja um compromisso com horário de fim.
    SemFimDeCompromisso,
    /// Acima de 10 anos.
    LongaDemais,
    /// Validade zero não faz sentido: use “apagar agora”.
    Zero,
    /// Conteúdo acima do limite do envelope.
    GrandeDemais,
}

impl Validade {
    pub fn horas(n: u64) -> Self { Validade { apos_segundos: n * HORA, referencia: Referencia::Criacao } }
    pub fn dias(n: u64) -> Self { Validade { apos_segundos: n * DIA, referencia: Referencia::Criacao } }
    pub fn meses(n: u64) -> Self { Validade { apos_segundos: n * MES_30_DIAS, referencia: Referencia::Criacao } }
    /// Para a agenda: conta a partir do fim do compromisso.
    pub fn apos_o_compromisso(segundos: u64) -> Self { Validade { apos_segundos: segundos, referencia: Referencia::FimDoCompromisso } }
}

/// Opções que a interface oferece para cada tipo de item.
pub fn predefinicoes(tipo: TipoDeItem) -> Vec<(&'static str, Validade)> {
    match tipo {
        TipoDeItem::Compromisso => vec![
            ("1 hora depois do compromisso", Validade::apos_o_compromisso(HORA)),
            ("1 dia depois do compromisso", Validade::apos_o_compromisso(DIA)),
            ("1 semana depois do compromisso", Validade::apos_o_compromisso(SEMANA)),
            ("1 mês depois do compromisso", Validade::apos_o_compromisso(MES_30_DIAS)),
        ],
        _ => vec![
            ("24 horas", Validade::horas(24)),
            ("7 dias", Validade::dias(7)),
            ("30 dias", Validade::dias(30)),
            ("6 meses", Validade::meses(6)),
        ],
    }
}

/// Instante (relógio do servidor, em segundos) em que o item deve deixar de existir.
pub fn instante_de_apagar(v: &Validade, criado_em: u64, fim_do_compromisso: Option<u64>) -> Result<u64, ErroDeValidade> {
    if v.apos_segundos == 0 { return Err(ErroDeValidade::Zero); }
    if v.apos_segundos > VALIDADE_MAXIMA { return Err(ErroDeValidade::LongaDemais); }
    let base = match v.referencia {
        Referencia::Criacao => criado_em,
        Referencia::FimDoCompromisso => fim_do_compromisso.ok_or(ErroDeValidade::SemFimDeCompromisso)?,
    };
    Ok(base.saturating_add(v.apos_segundos))
}

/// Guarda o item no servidor cego já amarrado à sua validade (apagamento criptográfico).
/// `validade = None` guarda sem prazo (chave permanente do dono).
pub fn guardar_com_validade(
    servidor: &mut ServidorCego,
    chaves: &mut ChavesDoDono,
    conteudo: &[u8],
    validade: Option<&Validade>,
    criado_em: u64,
    fim_do_compromisso: Option<u64>,
) -> Result<IdDeBloco, ErroDeValidade> {
    let expira = match validade { Some(v) => Some(instante_de_apagar(v, criado_em, fim_do_compromisso)?), None => None };
    historico::guardar(servidor, chaves, conteudo, expira).map_err(|_| ErroDeValidade::GrandeDemais)
}

/// A interface usa isto para esconder o item no instante exato, antes mesmo da destruição da chave.
pub fn deve_esconder(instante_de_apagar: u64, agora: u64) -> bool { agora >= instante_de_apagar }

#[cfg(test)]
mod testes {
    use super::*;
    use crate::historico::ErroDeLeitura;

    #[test]
    fn compromisso_apaga_depois_do_fim_e_nao_da_criacao() {
        // Criado às 1000, reunião termina às 50_000, apagar 1 hora depois do fim.
        let v = Validade::apos_o_compromisso(HORA);
        assert_eq!(instante_de_apagar(&v, 1_000, Some(50_000)), Ok(50_000 + HORA));
    }

    #[test]
    fn compromisso_sem_fim_e_recusado() {
        assert_eq!(instante_de_apagar(&Validade::apos_o_compromisso(HORA), 1_000, None), Err(ErroDeValidade::SemFimDeCompromisso));
    }

    #[test]
    fn nota_conta_a_partir_da_criacao() {
        assert_eq!(instante_de_apagar(&Validade::dias(7), 1_000, None), Ok(1_000 + 7 * DIA));
    }

    #[test]
    fn horas_dias_e_meses() {
        assert_eq!(Validade::horas(5).apos_segundos, 5 * HORA);
        assert_eq!(Validade::dias(2).apos_segundos, 2 * DIA);
        assert_eq!(Validade::meses(3).apos_segundos, 90 * DIA);
    }

    #[test]
    fn exatamente_dez_anos_e_aceito() {
        assert_eq!(instante_de_apagar(&Validade { apos_segundos: VALIDADE_MAXIMA, referencia: Referencia::Criacao }, 5, None), Ok(5 + VALIDADE_MAXIMA));
        assert_eq!(instante_de_apagar(&Validade { apos_segundos: VALIDADE_MAXIMA + 1, referencia: Referencia::Criacao }, 5, None), Err(ErroDeValidade::LongaDemais));
    }

    #[test]
    fn validade_zero_e_exagerada_sao_recusadas() {
        assert_eq!(instante_de_apagar(&Validade::horas(0), 0, None), Err(ErroDeValidade::Zero));
        assert_eq!(instante_de_apagar(&Validade::dias(4_000), 0, None), Err(ErroDeValidade::LongaDemais));
    }

    #[test]
    fn toda_categoria_tem_opcoes_de_validade() {
        for t in [TipoDeItem::Compromisso, TipoDeItem::Nota, TipoDeItem::Contato, TipoDeItem::Arquivo, TipoDeItem::Foto, TipoDeItem::ItemDoCofre] {
            assert_eq!(predefinicoes(t).len(), 4);
        }
        assert!(predefinicoes(TipoDeItem::Compromisso).iter().all(|(_, v)| v.referencia == Referencia::FimDoCompromisso));
    }

    #[test]
    fn compromisso_vencido_fica_ilegivel_mesmo_com_copia_guardada() {
        let mut s = ServidorCego::default();
        let mut c = ChavesDoDono::novas(HORA);
        let fim = 10_000;
        let id = guardar_com_validade(&mut s, &mut c, b"Reuniao com o advogado", Some(&Validade::apos_o_compromisso(HORA)), 1_000, Some(fim)).unwrap();
        let copia = s.blocos.clone();
        assert!(historico::ler(&s, &c, &id).is_ok(), "antes do prazo, abre");
        assert!(deve_esconder(fim + HORA, fim + HORA));
        let _ = c.destruir_janelas_vencidas(fim + 2 * HORA);
        let s2 = ServidorCego { blocos: copia };
        assert_eq!(historico::ler(&s2, &c, &id), Err(ErroDeLeitura::ChaveDestruida));
    }

    #[test]
    fn nota_sem_validade_continua_existindo() {
        let mut s = ServidorCego::default();
        let mut c = ChavesDoDono::novas(HORA);
        let id = guardar_com_validade(&mut s, &mut c, b"lista de compras", None, 0, None).unwrap();
        let _ = c.destruir_janelas_vencidas(999_999_999);
        assert!(historico::ler(&s, &c, &id).is_ok());
    }
}
