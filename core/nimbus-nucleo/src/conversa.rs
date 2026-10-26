//! # Módulo `conversa` — o que fica, por quanto tempo, e para os dois lados (HIS1–HIS13)
//!
//! Cada pessoa escolhe as próprias preferências; a conversa usa sempre a combinação **mais restritiva**
//! entre os participantes. Por isso, quando você define que chamadas não ficam registradas, isso vale
//! também no aparelho de quem liga para você; quando define que as mensagens somem em 1 hora, somem
//! para todos. Em grupo, vale a preferência mais restritiva entre todos os membros.
//!
//! Padrões do NimbusChat: mensagens somem 48 horas depois de enviadas — lidas ou não, nenhuma vive mais que
//! isso; chamada atendida não deixa registro em nenhum dos dois aparelhos; chamada não atendida fica
//! 48 horas na lista de chamadas de quem recebeu (para ele saber quem ligou) e depois some; conversa sem
//! mensagens desaparece da lista dos dois; nenhuma marca dentro da conversa.
//!
//! Chamada não atendida é a exceção à regra “a mais restritiva vence”: no aparelho de quem RECEBEU, vale
//! a preferência dele — quem ligou não consegue apagar o aviso de que ligou. No aparelho de quem LIGOU,
//! vale a mais restritiva entre os dois.
//!
//! Duas exceções de segurança, de propósito:
//! * **Troca de aparelho do contato** nunca é silenciada (é o alerta contra aparelho fantasma): aparece
//!   como faixa temporária no topo, some ao confirmar e não vira histórico.
//! * **Apagar tudo, na hora, dos aparelhos dos dois** só existe se OS DOIS ligarem essa opção. Sem
//!   consentimento, alguém poderia apagar à força a prova de um assédio no aparelho da vítima.
use crate::politica::{self, PrazoDeVisibilidade};

pub const HORA: u64 = 3_600;
pub const PADRAO_MENSAGENS: u64 = 48 * HORA;
pub const PADRAO_CHAMADAS_NAO_ATENDIDAS: u64 = 48 * HORA;
pub const PADRAO_NAO_LIDAS: u64 = 48 * HORA;
/// Mensagem não entregue ou não lida nunca espera mais que isto na caixa do servidor (48 horas).
pub const RETENCAO_MAXIMA_NA_CAIXA: u64 = crate::expiracao::LIMITE_PARA_LER;

/// Preferências de uma pessoa (e, depois de combinadas, as regras da conversa).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preferencias {
    /// Apagar mensagens N segundos depois de enviadas (`None` = sem prazo automático).
    pub mensagens_apos_enviar: Option<u64>,
    /// Mensagem não lida some neste prazo (limitado a 48 horas pelo próprio bilhete assinado).
    pub nao_lidas_somem_em: Option<u64>,
    /// Chamadas atendidas: `Some(0)` = não registrar (padrão); `Some(n)` = apagar em n segundos.
    pub chamadas_atendidas: Option<u64>,
    /// Chamadas não atendidas: por quanto tempo aparecem na lista de chamadas (padrão 48 horas).
    pub chamadas_nao_atendidas: Option<u64>,
    /// A conversa some da lista quando não restar nenhuma mensagem.
    pub remover_conversa_vazia: bool,
    /// Nenhuma marca dentro da conversa: chamada, chamada perdida, mensagem expirada ou apagada.
    pub sem_marcas: bool,
    /// Autoriza qualquer participante a apagar tudo, na hora, dos aparelhos de todos.
    pub apagar_tudo_a_pedido: bool,
    /// Quem enviou pode retirar a mensagem dos aparelhos de todos mesmo DEPOIS de lida (antes de lida pode
    /// sempre). Padrão: sim. Qualquer participante pode desligar — por exemplo, para guardar a prova de um
    /// assédio: basta um desligar para valer para a conversa inteira.
    pub retirar_lidas_a_pedido: bool,
    /// O ÚNICO aviso que existe: "visualizada". Opcional, com reciprocidade (quem desliga não envia e também
    /// não vê o do outro) e desligado por padrão. Por decisão do dono (06/10/2026) não existem "entregue",
    /// "digitando", "online" nem "visto por último": cada um deles seria um sinal saindo do aparelho.
    pub aviso_de_visualizacao: bool,
    /// RV1 — por quanto tempo o NOME desta conversa fica visível na lista de conversas de quem recebe.
    /// Independente da expiração do conteúdo das mensagens (`expiracao`, teto de 48 h): mesmo com o nome
    /// visível por mais tempo, nenhuma mensagem sobrevive além do próprio prazo de expiração. Combinado
    /// pelo menor entre os participantes, como todo o resto desta política — mesma regra, nova dimensão.
    pub prazo_do_nome_na_lista: PrazoDeVisibilidade,
}

/// Prazo padrão de visibilidade do nome na lista, para quem não escolhe nada: 7 dias. É uma escolha de
/// produto (não uma regra de segurança como as outras), sujeita à confirmação do dono do produto.
pub const PADRAO_PRAZO_DO_NOME_NA_LISTA: PrazoDeVisibilidade = PrazoDeVisibilidade::SeteDias;

/// Regras efetivas de uma conversa (mesma forma das preferências, já combinadas).
pub type Regras = Preferencias;

impl Preferencias {
    pub fn padrao() -> Self {
        Preferencias { mensagens_apos_enviar: Some(PADRAO_MENSAGENS), nao_lidas_somem_em: Some(PADRAO_NAO_LIDAS),
                       chamadas_atendidas: Some(0), chamadas_nao_atendidas: Some(PADRAO_CHAMADAS_NAO_ATENDIDAS),
                       remover_conversa_vazia: true, sem_marcas: true, apagar_tudo_a_pedido: false,
                       retirar_lidas_a_pedido: true, aviso_de_visualizacao: false,
                       prazo_do_nome_na_lista: PADRAO_PRAZO_DO_NOME_NA_LISTA }
    }
}

fn menor(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

/// Combina as preferências de todos os participantes: prazos pelo menor, “sem marcas” e “remover
/// conversa vazia” se QUALQUER UM pedir, “apagar tudo a pedido” só se TODOS consentirem.
pub fn combinar(participantes: &[Preferencias]) -> Regras {
    let mut r = Preferencias { mensagens_apos_enviar: None, nao_lidas_somem_em: None, chamadas_atendidas: None, chamadas_nao_atendidas: None,
                               remover_conversa_vazia: false, sem_marcas: false, apagar_tudo_a_pedido: !participantes.is_empty(),
                               retirar_lidas_a_pedido: !participantes.is_empty(), aviso_de_visualizacao: !participantes.is_empty(),
                               prazo_do_nome_na_lista: PrazoDeVisibilidade::Indeterminado };
    for p in participantes {
        r.mensagens_apos_enviar = menor(r.mensagens_apos_enviar, p.mensagens_apos_enviar);
        r.nao_lidas_somem_em = menor(r.nao_lidas_somem_em, p.nao_lidas_somem_em);
        r.chamadas_atendidas = menor(r.chamadas_atendidas, p.chamadas_atendidas);
        r.chamadas_nao_atendidas = menor(r.chamadas_nao_atendidas, p.chamadas_nao_atendidas);
        r.remover_conversa_vazia |= p.remover_conversa_vazia;
        r.sem_marcas |= p.sem_marcas;
        r.apagar_tudo_a_pedido &= p.apagar_tudo_a_pedido;
        r.retirar_lidas_a_pedido &= p.retirar_lidas_a_pedido;
        r.aviso_de_visualizacao &= p.aviso_de_visualizacao;
        r.prazo_do_nome_na_lista = r.prazo_do_nome_na_lista.min(p.prazo_do_nome_na_lista);
    }
    r
}

/// O nome desta conversa deve sumir da lista agora? `exibido_desde`: quando o nome passou a aparecer
/// (criação da conversa, ou renovação ao chegar mensagem nova — decisão do app).
pub fn nome_deve_sumir_da_lista(regras: &Regras, exibido_desde: u64, agora: u64) -> bool {
    agora >= exibido_desde.saturating_add(regras.prazo_do_nome_na_lista.em_segundos())
}

/// RV4 — o prazo de visibilidade pode ser ajustado contato a contato, sobrepondo o padrão geral das
/// `Preferencias` só para esta conversa. `None` = usa o padrão geral; `Some(p)` = vale `p` para este
/// contato específico. O valor devolvido aqui ainda passa por `combinar()` normalmente — o ajuste por
/// contato troca o que ESTE lado propõe, nunca pula a regra "a mais restritiva entre os dois vence".
pub fn prazo_efetivo_para_contato(preferencias: &Preferencias, excecao_do_contato: Option<PrazoDeVisibilidade>) -> PrazoDeVisibilidade {
    excecao_do_contato.unwrap_or(preferencias.prazo_do_nome_na_lista)
}

/// Retirar uma mensagem dos aparelhos de todos. Só quem a escreveu (conferido pela chave que assinou a
/// original); antes de lida, sempre; depois de lida, só se a conversa permitir. O pedido viaja cifrado como
/// qualquer mensagem e é cumprido pelo app de quem recebe; nada fica marcado se a conversa for "sem marcas".
/// Limite que continua escrito: foto da tela tirada por outro aparelho nunca pode ser retirada.
pub fn aceitar_retirada(regras: &Regras, autor_da_original: &[u8; 32], quem_pede: &[u8; 32], ja_lida: bool) -> bool {
    autor_da_original == quem_pede && (!ja_lida || regras.retirar_lidas_a_pedido)
}

/// Reciprocidade: o aviso "visualizada" só existe se os dois lados (todos, em grupo) o ligaram.
pub fn mostrar_visualizada(regras: &Regras) -> bool { regras.aviso_de_visualizacao }

/// Aplica as regras combinadas à política de expiração da conversa, sem nunca afrouxá-la.
pub fn aplicar_na_politica(regras: &Regras, conversa: &mut politica::Regras) {
    conversa.expiracao.segundos_apos_enviar = menor(conversa.expiracao.segundos_apos_enviar, regras.mensagens_apos_enviar);
    conversa.expiracao.segundos_para_ler = menor(conversa.expiracao.segundos_para_ler, regras.nao_lidas_somem_em);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direcao {
    /// Liguei e foi atendida.
    Feita,
    /// Recebi e atendi.
    Recebida,
    /// Recebi e NÃO atendi.
    Perdida,
    /// Liguei e NÃO foi atendida.
    NaoAtendida,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Midia { Voz, Video }

/// Registro de uma chamada no aparelho (guardado no cofre cifrado enquanto existir).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistroDeChamada {
    pub conversa: [u8; 16], pub inicio: u64, pub duracao_segundos: u64, pub direcao: Direcao, pub midia: Midia,
    /// Quando o registro some (`None` = sem prazo).
    pub apagar_em: Option<u64>,
}

/// Cada aparelho chama isto ao fim da chamada, com as regras combinadas da conversa e as próprias
/// preferências. `None` = nada é gravado.
pub fn registrar_chamada(regras: &Regras, minhas: &Preferencias, conversa: [u8; 16], inicio: u64, fim: u64, direcao: Direcao, midia: Midia) -> Option<RegistroDeChamada> {
    let prazo = match direcao {
        Direcao::Feita | Direcao::Recebida => regras.chamadas_atendidas,
        // Quem recebeu e não atendeu decide por quanto tempo sabe disso: quem ligou não apaga esse aviso.
        Direcao::Perdida => minhas.chamadas_nao_atendidas,
        Direcao::NaoAtendida => regras.chamadas_nao_atendidas,
    };
    match prazo {
        Some(0) => None,
        prazo => Some(RegistroDeChamada { conversa, inicio, duracao_segundos: fim.saturating_sub(inicio), direcao, midia,
                                          apagar_em: prazo.map(|p| fim.saturating_add(p)) }),
    }
}

/// Registros vencidos saem da lista (e a chave da janela do cofre é destruída no prazo).
pub fn registros_vigentes(registros: Vec<RegistroDeChamada>, agora: u64) -> Vec<RegistroDeChamada> {
    registros.into_iter().filter(|r| r.apagar_em.is_none_or(|t| agora < t)).collect()
}

/// RV2 — variante de `registrar_chamada` em que quem está envolvido na ligação escolhe, da lista fechada de
/// opções (`PrazoDeVisibilidade`), por quanto tempo o registro fica na LISTA DE LIGAÇÕES DO NIMBUS (nunca a
/// nativa do sistema — ver nota de isolamento em `chamada`). O prazo escolhido aqui SUBSTITUI o das
/// preferências da conversa para este registro específico, mas nunca passa de 3 meses — o mesmo teto que
/// vale para o nome na lista de conversas. `None` em `prazo_escolhido` usa as regras normais da conversa
/// (idêntico a `registrar_chamada`).
// Mesmo formato de argumentos de `registrar_chamada` (já 6), mais o prazo escolhido: manter a mesma ordem
// posicional ajuda quem está migrando uma chamada a `registrar_chamada` para esta variante.
#[allow(clippy::too_many_arguments)]
pub fn registrar_chamada_com_prazo_de_visibilidade(regras: &Regras, minhas: &Preferencias, conversa: [u8; 16], inicio: u64, fim: u64,
                                                    direcao: Direcao, midia: Midia, prazo_escolhido: Option<PrazoDeVisibilidade>) -> Option<RegistroDeChamada> {
    let Some(prazo) = prazo_escolhido else { return registrar_chamada(regras, minhas, conversa, inicio, fim, direcao, midia) };
    Some(RegistroDeChamada { conversa, inicio, duracao_segundos: fim.saturating_sub(inicio), direcao, midia,
                              apagar_em: Some(fim.saturating_add(prazo.em_segundos())) })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evento { Mensagem, ChamadaTerminada, ChamadaPerdida, MensagemExpirou, MensagemApagada, RegrasMudaram, NovoAparelhoDoContato }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exibicao {
    /// Aparece como item da conversa.
    NaConversa,
    /// Faixa temporária no topo: some ao ser confirmada e não fica no histórico.
    FaixaDeAlerta,
    /// Não aparece em lugar nenhum.
    Nada,
}

/// Onde (e se) cada acontecimento aparece.
pub fn onde_mostrar(regras: &Regras, e: Evento) -> Exibicao {
    match e {
        Evento::Mensagem => Exibicao::NaConversa,
        // Alerta contra aparelho fantasma: nunca pode ser silenciado.
        Evento::NovoAparelhoDoContato => Exibicao::FaixaDeAlerta,
        // Quem conversa precisa saber que as mensagens vão sumir mais cedo.
        Evento::RegrasMudaram => if regras.sem_marcas { Exibicao::FaixaDeAlerta } else { Exibicao::NaConversa },
        Evento::ChamadaTerminada | Evento::ChamadaPerdida | Evento::MensagemExpirou | Evento::MensagemApagada =>
            if regras.sem_marcas { Exibicao::Nada } else { Exibicao::NaConversa },
    }
}

/// A conversa inteira some da lista (dos dois) quando não resta nenhuma mensagem.
pub fn conversa_deve_sumir(regras: &Regras, mensagens_restantes: usize) -> bool {
    regras.remover_conversa_vazia && mensagens_restantes == 0
}

/// Pedido de “apagar tudo dos dois aparelhos, agora”: só vale com o consentimento de todos.
pub fn pode_apagar_tudo_de_todos(regras: &Regras) -> bool { regras.apagar_tudo_a_pedido }

/// Quanto tempo uma mensagem não entregue pode esperar na caixa do servidor.
pub fn prazo_na_caixa_do_servidor(regras: &Regras) -> u64 {
    let mut prazo = RETENCAO_MAXIMA_NA_CAIXA;
    for x in [regras.mensagens_apos_enviar, regras.nao_lidas_somem_em].into_iter().flatten() { prazo = prazo.min(x); }
    prazo
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::expiracao;

    fn p(msg: Option<u64>, cham: Option<u64>, apagar_tudo: bool) -> Preferencias {
        Preferencias { mensagens_apos_enviar: msg, nao_lidas_somem_em: None, chamadas_atendidas: cham, chamadas_nao_atendidas: cham,
                       remover_conversa_vazia: false, sem_marcas: false, apagar_tudo_a_pedido: apagar_tudo, retirar_lidas_a_pedido: true, aviso_de_visualizacao: true,
                       prazo_do_nome_na_lista: PADRAO_PRAZO_DO_NOME_NA_LISTA }
    }

    #[test]
    fn padroes_do_nimbuschat() {
        let d = Preferencias::padrao();
        assert_eq!((d.mensagens_apos_enviar, d.nao_lidas_somem_em), (Some(48 * HORA), Some(48 * HORA)));
        assert_eq!((d.chamadas_atendidas, d.chamadas_nao_atendidas), (Some(0), Some(48 * HORA)));
        assert!(d.remover_conversa_vazia && d.sem_marcas && !d.apagar_tudo_a_pedido);
    }

    #[test]
    fn chamada_atendida_nao_fica_em_nenhum_dos_dois_aparelhos() {
        let r = combinar(&[Preferencias::padrao(), Preferencias::padrao()]);
        let eu = Preferencias::padrao();
        assert_eq!(registrar_chamada(&r, &eu, [1; 16], 100, 400, Direcao::Recebida, Midia::Voz), None);
        assert_eq!(registrar_chamada(&r, &eu, [1; 16], 100, 400, Direcao::Feita, Midia::Voz), None);
    }

    #[test]
    fn chamada_perdida_fica_para_quem_recebeu_e_depois_some() {
        let eu = Preferencias::padrao();
        let r = combinar(&[eu, Preferencias::padrao()]);
        let reg = registrar_chamada(&r, &eu, [1; 16], 1_000, 1_030, Direcao::Perdida, Midia::Voz).unwrap();
        assert_eq!(reg.apagar_em, Some(1_030 + 48 * HORA), "sei que me ligaram, por 48 horas");
        assert!(registros_vigentes(vec![reg.clone()], 1_030 + 48 * HORA).is_empty());
    }

    #[test]
    fn quem_ligou_nao_apaga_o_aviso_de_quem_nao_atendeu() {
        let eu = Preferencias::padrao();
        let quem_ligou = Preferencias { chamadas_nao_atendidas: Some(0), ..Preferencias::padrao() };
        let r = combinar(&[eu, quem_ligou]);
        assert!(registrar_chamada(&r, &eu, [1; 16], 0, 20, Direcao::Perdida, Midia::Voz).is_some(), "no meu aparelho, vale a minha escolha");
        assert_eq!(registrar_chamada(&r, &quem_ligou, [1; 16], 0, 20, Direcao::NaoAtendida, Midia::Voz), None, "no aparelho dele, vale a mais restritiva");
    }

    #[test]
    fn minha_regra_encurta_o_registro_de_quem_me_ligou_e_nao_foi_atendido() {
        let eu = Preferencias { chamadas_nao_atendidas: Some(HORA), ..Preferencias::padrao() };
        let quem_ligou = Preferencias::padrao();
        let r = combinar(&[eu, quem_ligou]);
        let reg = registrar_chamada(&r, &quem_ligou, [1; 16], 0, 20, Direcao::NaoAtendida, Midia::Voz).unwrap();
        assert_eq!(reg.apagar_em, Some(20 + HORA));
    }

    #[test]
    fn meu_prazo_mais_curto_vale_para_os_dois() {
        let r = combinar(&[p(Some(HORA), Some(2 * HORA), false), p(Some(72 * HORA), Some(48 * HORA), false)]);
        assert_eq!((r.mensagens_apos_enviar, r.chamadas_atendidas, r.chamadas_nao_atendidas), (Some(HORA), Some(2 * HORA), Some(2 * HORA)));
        assert_eq!(r, combinar(&[p(Some(72 * HORA), Some(48 * HORA), false), p(Some(HORA), Some(2 * HORA), false)]), "a ordem não importa");
    }

    #[test]
    fn prazo_de_um_lado_so_vale_para_os_dois() {
        let r = combinar(&[p(None, None, false), p(Some(5 * HORA), Some(HORA), false)]);
        assert_eq!((r.mensagens_apos_enviar, r.chamadas_nao_atendidas), (Some(5 * HORA), Some(HORA)));
    }

    #[test]
    fn grupo_usa_o_mais_restritivo_de_todos() {
        let r = combinar(&[Preferencias::padrao(), p(Some(10 * HORA), None, false), p(Some(3 * HORA), Some(0), false)]);
        assert_eq!((r.mensagens_apos_enviar, r.chamadas_nao_atendidas), (Some(3 * HORA), Some(0)));
        assert!(r.sem_marcas && r.remover_conversa_vazia, "basta um pedir");
    }

    #[test]
    fn retirar_antes_de_lida_sempre_e_depois_so_se_ninguem_desligou() {
        let autor = [1u8; 32]; let outro = [2u8; 32];
        let todos_permitem = combinar(&[Preferencias::padrao(), Preferencias::padrao()]);
        assert!(aceitar_retirada(&todos_permitem, &autor, &autor, false), "antes de lida");
        assert!(aceitar_retirada(&todos_permitem, &autor, &autor, true), "depois de lida, padrão permite");
        assert!(!aceitar_retirada(&todos_permitem, &autor, &outro, false), "só quem escreveu retira");
        let vitima = Preferencias { retirar_lidas_a_pedido: false, ..Preferencias::padrao() };
        let protegida = combinar(&[Preferencias::padrao(), vitima]);
        assert!(!aceitar_retirada(&protegida, &autor, &autor, true), "basta um desligar para guardar o que já foi lido");
        assert!(aceitar_retirada(&protegida, &autor, &autor, false), "antes de lida continua podendo");
    }

    #[test]
    fn so_existe_o_aviso_de_visualizada_opcional_e_reciproco() {
        let padrao = combinar(&[Preferencias::padrao(), Preferencias::padrao()]);
        assert!(!mostrar_visualizada(&padrao), "desligado por padrão");
        let quer = Preferencias { aviso_de_visualizacao: true, ..Preferencias::padrao() };
        assert!(mostrar_visualizada(&combinar(&[quer, quer])), "os dois ligaram");
        assert!(!mostrar_visualizada(&combinar(&[quer, Preferencias::padrao()])), "quem desliga não envia nem vê");
        assert!(!mostrar_visualizada(&combinar(&[quer, quer, Preferencias::padrao()])), "em grupo, basta um desligar");
    }

    #[test]
    fn apagar_tudo_dos_dois_so_com_consentimento_dos_dois() {
        assert!(!pode_apagar_tudo_de_todos(&combinar(&[p(None, None, true), p(None, None, false)])));
        assert!(pode_apagar_tudo_de_todos(&combinar(&[p(None, None, true), p(None, None, true)])));
        assert!(!pode_apagar_tudo_de_todos(&combinar(&[])));
    }

    #[test]
    fn registro_de_chamada_some_no_prazo() {
        let eu = Preferencias { chamadas_atendidas: Some(48 * HORA), ..Preferencias::padrao() };
        let r = combinar(&[eu]);
        let reg = registrar_chamada(&r, &eu, [1; 16], 1_000, 1_300, Direcao::Recebida, Midia::Video).unwrap();
        assert_eq!((reg.duracao_segundos, reg.apagar_em), (300, Some(1_300 + 48 * HORA)));
        let t = reg.apagar_em.unwrap();
        assert_eq!(registros_vigentes(vec![reg.clone()], t - 1).len(), 1);
        assert_eq!(registros_vigentes(vec![reg], t).len(), 0, "no instante do prazo já não aparece");
    }

    #[test]
    fn sem_marcas_nenhum_rastro_de_chamada_ou_mensagem_apagada() {
        let r = combinar(&[Preferencias::padrao()]);
        for e in [Evento::ChamadaTerminada, Evento::ChamadaPerdida, Evento::MensagemExpirou, Evento::MensagemApagada] {
            assert_eq!(onde_mostrar(&r, e), Exibicao::Nada, "{e:?}");
        }
        assert_eq!(onde_mostrar(&r, Evento::Mensagem), Exibicao::NaConversa);
    }

    #[test]
    fn com_marcas_ligadas_os_eventos_aparecem() {
        let r = combinar(&[p(None, None, false)]);
        assert_eq!(onde_mostrar(&r, Evento::ChamadaTerminada), Exibicao::NaConversa);
        assert_eq!(onde_mostrar(&r, Evento::RegrasMudaram), Exibicao::NaConversa);
    }

    #[test]
    fn alerta_de_aparelho_novo_nunca_e_silenciado() {
        for r in [combinar(&[Preferencias::padrao()]), combinar(&[p(None, None, false)])] {
            assert_eq!(onde_mostrar(&r, Evento::NovoAparelhoDoContato), Exibicao::FaixaDeAlerta);
        }
        assert_eq!(onde_mostrar(&combinar(&[Preferencias::padrao()]), Evento::RegrasMudaram), Exibicao::FaixaDeAlerta);
    }

    #[test]
    fn conversa_vazia_some_da_lista() {
        let r = combinar(&[Preferencias::padrao()]);
        assert!(conversa_deve_sumir(&r, 0));
        assert!(!conversa_deve_sumir(&r, 1));
        assert!(!conversa_deve_sumir(&combinar(&[p(None, None, false)]), 0));
    }

    // ---- RV1–RV3: prazo de visibilidade escolhido pelo remetente, para a lista de conversas e de ligações ----

    #[test]
    fn padrao_do_nome_na_lista_e_sete_dias() {
        assert_eq!(Preferencias::padrao().prazo_do_nome_na_lista, PrazoDeVisibilidade::SeteDias);
    }

    #[test]
    fn prazo_do_nome_na_lista_combina_pelo_menor_como_o_resto() {
        let curto = Preferencias { prazo_do_nome_na_lista: PrazoDeVisibilidade::UmaHora, ..Preferencias::padrao() };
        let longo = Preferencias { prazo_do_nome_na_lista: PrazoDeVisibilidade::TresMeses, ..Preferencias::padrao() };
        assert_eq!(combinar(&[curto, longo]).prazo_do_nome_na_lista, PrazoDeVisibilidade::UmaHora);
        assert_eq!(combinar(&[longo, curto]).prazo_do_nome_na_lista, PrazoDeVisibilidade::UmaHora, "a ordem não importa");
        // Ninguém escolheu "indeterminado" (o teto interno de combinar()) explicitamente: nunca aparece sozinho.
        assert_ne!(combinar(&[longo]).prazo_do_nome_na_lista, PrazoDeVisibilidade::Indeterminado);
    }

    #[test]
    fn excecao_por_contato_sobrepoe_o_padrao_mas_ainda_combina_pelo_menor() {
        let base = Preferencias::padrao();
        assert_eq!(prazo_efetivo_para_contato(&base, None), PrazoDeVisibilidade::SeteDias, "sem exceção, vale o padrão geral");
        assert_eq!(prazo_efetivo_para_contato(&base, Some(PrazoDeVisibilidade::UmaHora)), PrazoDeVisibilidade::UmaHora);
        // O valor ajustado por contato ainda é só uma PROPOSTA: continua perdendo para um prazo mais curto do outro lado.
        let eu_com_excecao = Preferencias { prazo_do_nome_na_lista: prazo_efetivo_para_contato(&base, Some(PrazoDeVisibilidade::TresMeses)), ..base };
        let o_outro = Preferencias { prazo_do_nome_na_lista: PrazoDeVisibilidade::UmaHora, ..base };
        assert_eq!(combinar(&[eu_com_excecao, o_outro]).prazo_do_nome_na_lista, PrazoDeVisibilidade::UmaHora);
    }

    #[test]
    fn nome_some_da_lista_no_prazo_escolhido_e_nao_antes() {
        let r = Preferencias { prazo_do_nome_na_lista: PrazoDeVisibilidade::VinteQuatroHoras, ..Preferencias::padrao() };
        assert!(!nome_deve_sumir_da_lista(&r, 1_000, 1_000 + 24 * HORA - 1));
        assert!(nome_deve_sumir_da_lista(&r, 1_000, 1_000 + 24 * HORA));
    }

    #[test]
    fn registro_de_ligacao_com_prazo_escolhido_ignora_a_preferencia_da_conversa_mas_respeita_o_teto() {
        let r = combinar(&[Preferencias::padrao()]);
        let eu = Preferencias::padrao();
        // Sem escolha: comportamento idêntico ao de sempre (chamada atendida não registra).
        assert_eq!(registrar_chamada_com_prazo_de_visibilidade(&r, &eu, [1; 16], 100, 400, Direcao::Recebida, Midia::Voz, None), None);
        // Com escolha explícita de 60 dias: o registro passa a existir, mesmo sendo uma chamada atendida
        // (o padrão da conversa era "não registrar") — a escolha do prazo é também a escolha de registrar.
        let reg = registrar_chamada_com_prazo_de_visibilidade(&r, &eu, [1; 16], 100, 400, Direcao::Recebida, Midia::Voz, Some(PrazoDeVisibilidade::SessentaDias)).unwrap();
        assert_eq!(reg.apagar_em, Some(400 + PrazoDeVisibilidade::SessentaDias.em_segundos()));
        // "Indeterminado" nunca passa de 3 meses, nem aqui.
        let indet = registrar_chamada_com_prazo_de_visibilidade(&r, &eu, [1; 16], 100, 400, Direcao::Recebida, Midia::Voz, Some(PrazoDeVisibilidade::Indeterminado)).unwrap();
        assert_eq!(indet.apagar_em, Some(400 + politica::TRES_MESES_EM_SEGUNDOS));
    }

    #[test]
    fn caixa_do_servidor_nunca_guarda_mais_que_o_prazo_nem_mais_que_48_horas() {
        assert_eq!(prazo_na_caixa_do_servidor(&combinar(&[p(Some(HORA), None, false)])), HORA);
        assert_eq!(prazo_na_caixa_do_servidor(&combinar(&[p(Some(30 * 24 * HORA), None, false)])), 48 * HORA, "nunca mais que 48 horas");
        assert_eq!(prazo_na_caixa_do_servidor(&combinar(&[p(None, None, false)])), 48 * HORA);
        let nao_lidas_em_6h = Preferencias { nao_lidas_somem_em: Some(6 * HORA), ..p(None, None, false) };
        assert_eq!(prazo_na_caixa_do_servidor(&combinar(&[nao_lidas_em_6h])), 6 * HORA);
    }

    #[test]
    fn aplicar_na_politica_nunca_afrouxa() {
        let base = |prazo| politica::Regras { expiracao: expiracao::Politica { segundos_apos_abrir: None, segundos_apos_enviar: prazo, visualizacao_unica: false, segundos_para_ler: None },
            permitir_citar: false, conversa_blindada: false };
        let r = combinar(&[Preferencias::padrao()]);
        let mut longa = base(Some(7 * 24 * HORA));
        aplicar_na_politica(&r, &mut longa);
        assert_eq!(longa.expiracao.segundos_apos_enviar, Some(48 * HORA));
        let mut curta = base(Some(HORA));
        aplicar_na_politica(&r, &mut curta);
        assert_eq!(curta.expiracao.segundos_apos_enviar, Some(HORA), "a regra mais curta continua valendo");
        assert_eq!(curta.expiracao.segundos_para_ler, Some(48 * HORA), "prazo para ler aplicado à conversa");
    }
}
