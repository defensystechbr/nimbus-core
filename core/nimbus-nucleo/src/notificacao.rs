//! # Módulo `notificacao` — o horário das notificações não revela nada (NOT1–NOT7, EXP2)
//!
//! A Apple e o Google entregam as notificações, então veem **quando** cada aparelho recebe uma.
//! Solução: **cadência fixa**. A cada janela de tempo, o servidor manda **exatamente uma** notificação a
//! cada aparelho ativo — real ou de cobertura —, todas do mesmo tamanho e cifradas para o aparelho.
//! Quem entrega vê o mesmo ritmo em todo mundo, haja mensagem ou não. No aparelho, a de cobertura é
//! descartada em silêncio; a real mostra só um texto genérico.
//!
//! Requisito de plataforma: no iPhone, descartar em silêncio exige a permissão especial da Apple de
//! filtragem de notificações (com.apple.developer.usernotifications.filtering), concedida caso a caso.
//! Sem ela, usar o modo sem notificação ou a entrega em janelas (Manual, NOT5). No Android, dispensa o
//! Google: entrega por conexão própria (NOT6).
use crate::envelope::{self, ChavePublica, ChaveSecreta};

const CONTEXTO: &[u8] = b"NimbusOne/v1/notificacao";
/// Texto único exibido para toda notificação real: não diz de quem, nem do quê.
pub const TEXTO_GENERICO: &str = "Nova atividade";
/// Tela de chamada recebida com o aparelho bloqueado: nunca o nome de quem liga.
pub const TEXTO_CHAMADA_BLOQUEADO: &str = "Chamada Nimbus";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plataforma { Android, Iphone, Computador }

/// Por onde a notificação chega.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Canal {
    /// Conexão própria do Nimbus: Google, Apple ou terceiros não participam.
    Proprio,
    /// Serviço da Apple: obrigatório no iPhone para acordar o app. A Apple vê só um bloco cifrado de
    /// tamanho fixo, na cadência fixa, sem remetente, sem conteúdo, sem tipo (mensagem ou chamada).
    ApnsCifrado,
}

/// Android e computador: canal próprio. iPhone: o sistema não permite acordar o app sem o serviço da
/// Apple — por isso o que passa por ele é só ruído cifrado.
pub fn canal_para(p: Plataforma) -> Canal { match p { Plataforma::Iphone => Canal::ApnsCifrado, _ => Canal::Proprio } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tipo { Real, Cobertura }

/// Servidor: monta a rodada da janela — uma notificação por aparelho, sem exceção.
/// `tem_novidade[i]` diz se o aparelho i tem algo real nesta janela.
pub fn montar_rodada(aparelhos: &[ChavePublica], tem_novidade: &[bool]) -> Vec<Vec<u8>> {
    aparelhos.iter().enumerate().map(|(i, chave)| {
        let tipo = if tem_novidade.get(i).copied().unwrap_or(false) { 1u8 } else { 0u8 };
        // Carga fixa de 1 byte e chave pública sempre válida por construção: selar não tem como falhar.
        envelope::selar(chave, &[tipo], CONTEXTO).expect("carga fixa de 1 byte para chave validada")
    }).collect()
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decisao { MostrarGenerica(&'static str), DescartarEmSilencio, Invalida }

/// Aparelho (extensão de notificação): decide o que fazer.
pub fn ao_receber(sec: &ChaveSecreta, publ: &ChavePublica, carga: &[u8]) -> Decisao {
    match envelope::abrir(sec, publ, carga, CONTEXTO) {
        Ok(v) if v.as_slice() == [1] => Decisao::MostrarGenerica(TEXTO_GENERICO),
        Ok(v) if v.as_slice() == [0] => Decisao::DescartarEmSilencio,
        _ => Decisao::Invalida,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn todo_aparelho_recebe_exatamente_uma_por_janela() {
        let pares: Vec<_> = (0..5).map(|_| envelope::gerar_par_de_chaves()).collect();
        let publicas: Vec<ChavePublica> = pares.iter().map(|p| p.0.clone()).collect();
        let rodada = montar_rodada(&publicas, &[true, false, false, true, false]);
        assert_eq!(rodada.len(), 5);
    }

    #[test]
    fn real_e_cobertura_tem_o_mesmo_tamanho() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let r = montar_rodada(&[p.clone(), p.clone()], &[true, false]);
        assert_eq!(r[0].len(), r[1].len(), "quem entrega não distingue real de cobertura");
    }

    #[test]
    fn aparelho_mostra_so_a_real_e_com_texto_generico() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let r = montar_rodada(&[p.clone(), p.clone()], &[true, false]);
        assert_eq!(ao_receber(&s, &p, &r[0]), Decisao::MostrarGenerica("Nova atividade"));
        assert_eq!(ao_receber(&s, &p, &r[1]), Decisao::DescartarEmSilencio);
    }

    #[test]
    fn notificacao_de_outro_aparelho_e_invalida() {
        let (p1, _) = envelope::gerar_par_de_chaves();
        let (p2, s2) = envelope::gerar_par_de_chaves();
        let r = montar_rodada(&[p1], &[true]);
        assert_eq!(ao_receber(&s2, &p2, &r[0]), Decisao::Invalida);
    }

    #[test]
    fn android_e_computador_sem_apple_nem_google() {
        assert_eq!(canal_para(Plataforma::Android), Canal::Proprio);
        assert_eq!(canal_para(Plataforma::Computador), Canal::Proprio);
        assert_eq!(canal_para(Plataforma::Iphone), Canal::ApnsCifrado);
        assert!(!TEXTO_CHAMADA_BLOQUEADO.contains("de "));
    }

    #[test]
    fn texto_nunca_revela_remetente_nem_assunto() {
        assert!(!TEXTO_GENERICO.contains("mensagem de") && TEXTO_GENERICO.len() < 20);
    }
}
