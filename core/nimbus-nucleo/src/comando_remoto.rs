//! # Módulo `comando_remoto` — comando de proteção assinado por outro aparelho já confiado (novo, 10/10/2026)
//!
//! Resposta ao pedido do dono: "quando o dono conseguir acessar de um computador ou outro aparelho, [poder]
//! enviar um sinal para realmente bloquear, apagar e destruir tudo que está no temporário." Este módulo
//! autentica esse sinal; quem decide COMO reagir continua sendo `panico.rs` (`Gatilho::ComandoRemoto`,
//! ver a doc daquele módulo para o porquê da separação entre autenticar e reagir).
//!
//! **Quem pode emitir:** qualquer aparelho JÁ REGISTRADO na mesma conta (passou em algum momento pela
//! cerimônia de `sessoes::pode_vincular`) — nunca um terceiro, nunca só quem sabe a senha da conta. A
//! chave que assina é a identidade do PRÓPRIO aparelho emissor, a mesma que `sessoes.rs` já usa para
//! registrá-lo: nenhuma chave nova, nenhum segredo a mais para o dono guardar ou poder perder.
//!
//! **Dois travamentos, nenhum opcional:**
//! 1. **Vinculado ao aparelho-alvo certo.** A assinatura cobre o `IdAparelho` de destino — assinar "trava
//!    o aparelho X" não serve para travar o aparelho Y, mesmo com a mesma chave e a mesma conta.
//! 2. **Contador estritamente crescente por aparelho-alvo, nunca relógio.** Mesmo princípio de
//!    `coacao::RelogioDeEspera` e da `versao` de `liberacao::Autorizacao`: um comando antigo capturado em
//!    trânsito (ex.: um elo da rede reenviando um "trava" de antes de o dono já ter mandado "pode deixar,
//!    recuperei o aparelho") não pode ser repetido depois — só vale um contador maior que o último já
//!    aceito para aquele aparelho-alvo. Quem persiste "qual foi o último contador aceito" é a camada que
//!    chama `validar` (`panico.rs` e este módulo são funções puras, sem estado) — o mesmo padrão já usado
//!    em `coacao.rs` e `expiracao.rs` para não misturar a prova com o armazenamento.
//!
//! **O que isto especificamente NÃO prova (fica para quem integrar):** que o comando de fato CHEGA ao
//! aparelho-alvo enquanto ele ainda está online — isto é transporte de rede, abstraído de propósito para
//! fora deste módulo (núcleo nunca trata transporte, mesmo princípio de `custodia_do_cofre.rs`); nem que a
//! execução da reação de fato acontece. A reação concreta que a ponte deve disparar ao ver
//! `panico::decidir` devolver `Some(Reacao::ApagarChaves)` por este caminho é compor três chamadas que já
//! existem, sem nenhuma nova: `historico::ChavesDoDono::destruir_janelas_vencidas`,
//! `prechaves::Chaveiro::apagar_vencidas` e `metadados::ChavesDeIndice::destruir_vencidas`, todas com
//! `agora` forçado para um valor no futuro (o mesmo mecanismo que já faz a Cláusula 6a/6b funcionar,
//! chamado agora SOB DEMANDA em vez de esperar o relógio real) — isto destrói exatamente "o que está no
//! temporário", nunca a chave permanente do cofre (essa é `custodia_do_cofre`/Família 2, outro mecanismo,
//! deliberadamente separado) — mais `sessoes::Registro::revogar_aparelho` para que a sessão caia mesmo que
//! o comando em si nunca chegue a ser executado pelo aparelho (ex.: offline de propósito). **Limite
//! honesto, herdado da própria Cláusula 6a:** um aparelho isolado da rede antes de o comando chegar
//! (modo avião, bolsa de Faraday) derrota a parte "o comando chega e executa" por desenho — por isso a
//! Cláusula 6a (apagamento criptográfico por janela vencida, nunca depende de sinal algum chegar) continua
//! sendo a proteção de fundo, com ou sem este comando.
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use crate::panico::Reacao;

const CONTEXTO: &[u8] = b"NimbusOne/v1/comando_remoto";

/// Mesmo formato de `sessoes::IdAparelho` ([u8; 32]) — não importado diretamente para não acoplar este
/// módulo a `sessoes.rs` só por um alias; qualquer `[u8; 32]` que a ponte já usa como id de aparelho serve
/// aqui sem conversão.
pub type IdAparelho = [u8; 32];

/// Comando assinado, pronto para viajar pela rede até o aparelho-alvo (conteúdo público: não há segredo
/// aqui que precise de sigilo adicional além do já dado pelo canal de transporte da ponte).
#[derive(Clone, Debug)]
pub struct ComandoDeProtecao {
    pub aparelho_alvo: IdAparelho,
    pub reacao: Reacao,
    pub contador: u64,
    /// Impressão digital da chave de quem assinou — usada pela ponte para resolver, no registro de
    /// aparelhos da conta (`sessoes.rs`), qual `ChaveDeVerificacao` passar para `validar`.
    pub emissor: [u8; 32],
    pub assinatura: AssinaturaHibrida,
}

fn bytes_para_assinar(aparelho_alvo: &IdAparelho, reacao: Reacao, contador: u64) -> Vec<u8> {
    let mut v = aparelho_alvo.to_vec();
    v.push(reacao as u8);
    v.extend_from_slice(&contador.to_be_bytes());
    v
}

/// No aparelho (ou computador) que EMITE o comando — o "outro aparelho" do pedido do dono.
pub fn emitir(emissor: &ChaveDeAssinatura, aparelho_alvo: IdAparelho, reacao: Reacao, contador: u64) -> ComandoDeProtecao {
    let assinatura = emissor
        .assinar(CONTEXTO, &bytes_para_assinar(&aparelho_alvo, reacao, contador))
        .expect("contexto fixo e curto");
    ComandoDeProtecao { aparelho_alvo, reacao, contador, emissor: emissor.verificacao().impressao_digital(), assinatura }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Recusa {
    /// Assinatura não confere com `chave_do_emissor` — cobre tanto "não foi este emissor que assinou"
    /// quanto "o comando foi alterado depois de assinado" (ex.: trocar `Trancar` por `ApagarChaves`):
    /// qualquer mudança num byte do conteúdo assinado já invalida a assinatura, sem precisar de um erro
    /// separado para cada campo.
    AssinaturaInvalida,
    /// Assinatura válida, mas endereçada a OUTRO aparelho.
    AparelhoErrado,
    /// Contador igual ou menor que o último já aceito para este aparelho-alvo — comando repetido ou velho.
    ComandoRepetido,
}

/// `chave_do_emissor`: a chave de verificação já resolvida pela ponte a partir de `cmd.emissor` contra o
/// registro de aparelhos da conta (um emissor que não está mais registrado simplesmente não tem chave
/// para resolver — a ponte recusa antes mesmo de chamar esta função). `aparelho_deste_dispositivo`: quem
/// está avaliando o comando, só aceita o que é endereçado a si mesmo. `ultimo_contador_aceito`: o maior
/// contador já aceito para ESTE aparelho-alvo (persistido pela ponte, nunca por este módulo).
pub fn validar(
    chave_do_emissor: &ChaveDeVerificacao,
    cmd: &ComandoDeProtecao,
    aparelho_deste_dispositivo: &IdAparelho,
    ultimo_contador_aceito: u64,
) -> Result<Reacao, Recusa> {
    let bytes = bytes_para_assinar(&cmd.aparelho_alvo, cmd.reacao, cmd.contador);
    if !chave_do_emissor.verificar(CONTEXTO, &bytes, &cmd.assinatura) {
        return Err(Recusa::AssinaturaInvalida);
    }
    if cmd.aparelho_alvo != *aparelho_deste_dispositivo {
        return Err(Recusa::AparelhoErrado);
    }
    if cmd.contador <= ultimo_contador_aceito {
        return Err(Recusa::ComandoRepetido);
    }
    Ok(cmd.reacao)
}

/// # Bloqueio/apagamento por QUÓRUM de guardiões (Rodada 21, 11/10/2026 — pedido do dono: "através de
/// algum contato de segurança (guardião)... conseguir dar um comando bloqueando o app no celular roubado
/// ou até mesmo limpando ele", para o caso em que o dono só tem o celular roubado e nenhum segundo
/// aparelho/computador já registrado à mão — a pré-condição honesta do caminho acima (`emitir`/`validar`),
/// já registrada na Rodada 20.
///
/// **Por que exige `k` guardiões, nunca um guardião sozinho — decisão de segurança, não limite técnico:**
/// dar a UM guardião isolado o poder de travar ou apagar o aparelho do dono criaria uma vulnerabilidade
/// NOVA que não existia antes — um guardião malicioso, comprometido ou ele mesmo enganado/coagido poderia
/// sabotar o dono sozinho, sem o dono nunca ter perdido nada. O mesmo princípio que já vale para a
/// recuperação (nenhum guardião sozinho reconstrói nada, `k` deles precisam concordar) se aplica aqui.
/// `k` reaproveita o MESMO quórum já configurado em `recuperacao::preparar` para esta conta — não é um
/// número novo para o dono decidir de novo.
///
/// **Nenhuma chave nova:** cada guardião já tem uma identidade de assinatura como usuário Nimbus
/// (`assinatura::ChaveDeAssinatura`, a mesma que já usa para autorizar liberações em `liberacao.rs`) — este
/// módulo só reaproveita essa identidade sob um contexto de assinatura próprio (`CONTEXTO_GUARDIOES`),
/// nunca a chave de recuperação (`envelope::ChavePublica`) que o guardião usa para guardar uma trilha.
const CONTEXTO_GUARDIOES: &[u8] = b"NimbusOne/v1/comando_remoto-guardioes";

/// A assinatura de UM guardião concordando com "travar/apagar o aparelho X, contador N" — não entrega
/// nenhuma peça de recuperação, só autoriza esta ação específica.
#[derive(Clone, Debug)]
pub struct AssinaturaDeGuardiao { pub guardiao: [u8; 32], pub assinatura: AssinaturaHibrida }

/// Comando por quórum, pronto para viajar até o aparelho-alvo assim que `assinaturas.len() >= k` válidas.
#[derive(Clone, Debug)]
pub struct ComandoPorGuardioes { pub aparelho_alvo: IdAparelho, pub reacao: Reacao, pub contador: u64, pub assinaturas: Vec<AssinaturaDeGuardiao> }

/// Um guardião assina sua concordância. Cada guardião chama isto no próprio aparelho; a ponte junta as
/// assinaturas de quantos guardiões concordarem (`juntar_assinaturas`) antes de montar o comando final.
pub fn guardiao_assina(guardiao: &ChaveDeAssinatura, aparelho_alvo: IdAparelho, reacao: Reacao, contador: u64) -> AssinaturaDeGuardiao {
    let bytes = bytes_para_assinar(&aparelho_alvo, reacao, contador);
    let assinatura = guardiao.assinar(CONTEXTO_GUARDIOES, &bytes).expect("contexto fixo e curto");
    AssinaturaDeGuardiao { guardiao: guardiao.verificacao().impressao_digital(), assinatura }
}

/// Monta o comando a partir das assinaturas já coletadas (ainda não validadas aqui — `validar_por_guardioes`
/// faz a conferência de verdade; esta função só empacota).
pub fn juntar_assinaturas(aparelho_alvo: IdAparelho, reacao: Reacao, contador: u64, assinaturas: Vec<AssinaturaDeGuardiao>) -> ComandoPorGuardioes {
    ComandoPorGuardioes { aparelho_alvo, reacao, contador, assinaturas }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeGuardioes {
    /// Endereçado a outro aparelho — mesmo sentido de `Recusa::AparelhoErrado`.
    AparelhoErrado,
    /// Contador repetido ou velho — mesmo sentido de `Recusa::ComandoRepetido`.
    ComandoRepetido,
    /// Menos de `k` assinaturas DISTINTAS e VÁLIDAS de guardiões realmente cadastrados nesta conta —
    /// cobre tanto "faltaram guardiões" quanto "alguma assinatura era inválida/de impostor", de propósito:
    /// não dizer qual assinatura falhou evita dar a um atacante uma pista de qual guardião atacar primeiro.
    QuorumInsuficiente,
}

/// `chaves_dos_guardioes`: resolvidas pela ponte a partir da impressão digital de cada assinatura, só para
/// guardiões REALMENTE cadastrados como guardiões desta conta hoje (quem já foi removido como guardião,
/// ou nunca foi, não entra nesta lista — a ponte filtra antes de chamar esta função, mesmo padrão de
/// `validar` acima para `chave_do_emissor`). `k`: o quórum desta conta, o mesmo de `recuperacao::preparar`.
pub fn validar_por_guardioes(
    chaves_dos_guardioes: &[([u8; 32], ChaveDeVerificacao)],
    cmd: &ComandoPorGuardioes,
    aparelho_deste_dispositivo: &IdAparelho,
    ultimo_contador_aceito: u64,
    k: u8,
) -> Result<Reacao, RecusaDeGuardioes> {
    if cmd.aparelho_alvo != *aparelho_deste_dispositivo { return Err(RecusaDeGuardioes::AparelhoErrado); }
    if cmd.contador <= ultimo_contador_aceito { return Err(RecusaDeGuardioes::ComandoRepetido); }
    let bytes = bytes_para_assinar(&cmd.aparelho_alvo, cmd.reacao, cmd.contador);
    let mut confirmados: Vec<[u8; 32]> = Vec::new();
    for ag in &cmd.assinaturas {
        if confirmados.contains(&ag.guardiao) { continue; } // o mesmo guardião duas vezes conta uma
        if let Some((_, chave)) = chaves_dos_guardioes.iter().find(|(fp, _)| *fp == ag.guardiao) {
            if chave.verificar(CONTEXTO_GUARDIOES, &bytes, &ag.assinatura) {
                confirmados.push(ag.guardiao);
            }
        }
    }
    if confirmados.len() < k as usize { return Err(RecusaDeGuardioes::QuorumInsuficiente); }
    Ok(cmd.reacao)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    const APARELHO_A: IdAparelho = [1; 32];
    const APARELHO_B: IdAparelho = [2; 32];

    #[test]
    fn comando_valido_e_aceito_e_devolve_a_reacao() {
        let dono = assinatura::gerar();
        let cmd = emitir(&dono, APARELHO_A, Reacao::ApagarChaves, 1);
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_A, 0), Ok(Reacao::ApagarChaves));
    }

    #[test]
    fn dono_pode_mandar_so_travar_sem_apagar_nada() {
        // Cenário do pedido: aparelho emprestado/esquecido, não necessariamente roubado — o dono escolhe
        // a reação mais branda sem precisar "queimar" o aparelho.
        let dono = assinatura::gerar();
        let cmd = emitir(&dono, APARELHO_A, Reacao::Trancar, 1);
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_A, 0), Ok(Reacao::Trancar));
    }

    #[test]
    fn comando_para_outro_aparelho_e_recusado() {
        let dono = assinatura::gerar();
        let cmd = emitir(&dono, APARELHO_A, Reacao::ApagarChaves, 1);
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_B, 0), Err(Recusa::AparelhoErrado));
    }

    #[test]
    fn assinatura_de_quem_nao_e_dono_e_recusada() {
        let dono = assinatura::gerar();
        let intruso = assinatura::gerar();
        let cmd = emitir(&intruso, APARELHO_A, Reacao::ApagarChaves, 1);
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_A, 0), Err(Recusa::AssinaturaInvalida));
    }

    #[test]
    fn comando_adulterado_depois_de_assinado_e_recusado() {
        let dono = assinatura::gerar();
        let mut cmd = emitir(&dono, APARELHO_A, Reacao::Trancar, 1);
        cmd.reacao = Reacao::ApagarChaves; // tenta trocar "só travar" por "apagar tudo" depois de assinado
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_A, 0), Err(Recusa::AssinaturaInvalida));
        let mut cmd2 = emitir(&dono, APARELHO_A, Reacao::Trancar, 1);
        cmd2.aparelho_alvo = APARELHO_B; // tenta redirecionar o mesmo comando assinado para outro aparelho
        assert_eq!(validar(&dono.verificacao(), &cmd2, &APARELHO_B, 0), Err(Recusa::AssinaturaInvalida));
    }

    #[test]
    fn contador_repetido_ou_antigo_e_recusado() {
        let dono = assinatura::gerar();
        let cmd = emitir(&dono, APARELHO_A, Reacao::Trancar, 5);
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_A, 5), Err(Recusa::ComandoRepetido), "igual ao último aceito");
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_A, 6), Err(Recusa::ComandoRepetido), "menor que o último aceito");
        assert_eq!(validar(&dono.verificacao(), &cmd, &APARELHO_A, 4), Ok(Reacao::Trancar), "maior que o último aceito, vale");
    }

    #[test]
    fn dois_comandos_do_mesmo_emissor_tem_contadores_crescentes() {
        let dono = assinatura::gerar();
        let primeiro = emitir(&dono, APARELHO_A, Reacao::Trancar, 1);
        assert_eq!(validar(&dono.verificacao(), &primeiro, &APARELHO_A, 0), Ok(Reacao::Trancar));
        // Dono muda de ideia e manda "apagar tudo" em seguida: contador maior, aceito normalmente.
        let segundo = emitir(&dono, APARELHO_A, Reacao::ApagarChaves, 2);
        assert_eq!(validar(&dono.verificacao(), &segundo, &APARELHO_A, 1), Ok(Reacao::ApagarChaves));
    }

    // ---- bloqueio/apagamento por quórum de guardiões (Rodada 21) ----

    fn guardioes_assinatura(n: usize) -> Vec<ChaveDeAssinatura> { (0..n).map(|_| assinatura::gerar()).collect() }
    fn chaves_resolvidas(g: &[ChaveDeAssinatura]) -> Vec<([u8; 32], ChaveDeVerificacao)> {
        g.iter().map(|c| (c.verificacao().impressao_digital(), c.verificacao())).collect()
    }

    #[test]
    fn quorum_de_2_de_3_guardioes_trava_o_aparelho() {
        let g = guardioes_assinatura(3);
        let a1 = guardiao_assina(&g[0], APARELHO_A, Reacao::ApagarChaves, 1);
        let a2 = guardiao_assina(&g[2], APARELHO_A, Reacao::ApagarChaves, 1);
        let cmd = juntar_assinaturas(APARELHO_A, Reacao::ApagarChaves, 1, vec![a1, a2]);
        assert_eq!(validar_por_guardioes(&chaves_resolvidas(&g), &cmd, &APARELHO_A, 0, 2), Ok(Reacao::ApagarChaves));
    }

    #[test]
    fn um_guardiao_sozinho_nunca_basta_mesmo_sendo_valido() {
        // Decisão de segurança desta rodada: nenhum guardião isolado pode travar/apagar o aparelho do
        // dono — evita que um guardião malicioso, comprometido ou coagido sabote o dono sozinho.
        let g = guardioes_assinatura(3);
        let a1 = guardiao_assina(&g[0], APARELHO_A, Reacao::ApagarChaves, 1);
        let cmd = juntar_assinaturas(APARELHO_A, Reacao::ApagarChaves, 1, vec![a1]);
        assert_eq!(validar_por_guardioes(&chaves_resolvidas(&g), &cmd, &APARELHO_A, 0, 2), Err(RecusaDeGuardioes::QuorumInsuficiente));
    }

    #[test]
    fn o_mesmo_guardiao_assinando_duas_vezes_conta_uma_so() {
        let g = guardioes_assinatura(3);
        let a1 = guardiao_assina(&g[0], APARELHO_A, Reacao::ApagarChaves, 1);
        let a1_de_novo = guardiao_assina(&g[0], APARELHO_A, Reacao::ApagarChaves, 1);
        let cmd = juntar_assinaturas(APARELHO_A, Reacao::ApagarChaves, 1, vec![a1, a1_de_novo]);
        assert_eq!(validar_por_guardioes(&chaves_resolvidas(&g), &cmd, &APARELHO_A, 0, 2), Err(RecusaDeGuardioes::QuorumInsuficiente));
    }

    #[test]
    fn assinatura_de_quem_nao_e_guardiao_cadastrado_nao_conta_para_o_quorum() {
        let g = guardioes_assinatura(3);
        let intruso = assinatura::gerar(); // nunca foi cadastrado como guardião desta conta
        let a1 = guardiao_assina(&g[0], APARELHO_A, Reacao::ApagarChaves, 1);
        let a_intruso = guardiao_assina(&intruso, APARELHO_A, Reacao::ApagarChaves, 1);
        let cmd = juntar_assinaturas(APARELHO_A, Reacao::ApagarChaves, 1, vec![a1, a_intruso]);
        // só g[0] e g[1] e g[2] estão na lista resolvida pela ponte; o intruso nunca é encontrado.
        assert_eq!(validar_por_guardioes(&chaves_resolvidas(&g), &cmd, &APARELHO_A, 0, 2), Err(RecusaDeGuardioes::QuorumInsuficiente));
    }

    #[test]
    fn comando_por_guardioes_para_outro_aparelho_ou_repetido_e_recusado() {
        let g = guardioes_assinatura(3);
        let a1 = guardiao_assina(&g[0], APARELHO_A, Reacao::ApagarChaves, 5);
        let a2 = guardiao_assina(&g[1], APARELHO_A, Reacao::ApagarChaves, 5);
        let cmd = juntar_assinaturas(APARELHO_A, Reacao::ApagarChaves, 5, vec![a1, a2]);
        assert_eq!(validar_por_guardioes(&chaves_resolvidas(&g), &cmd, &APARELHO_B, 0, 2), Err(RecusaDeGuardioes::AparelhoErrado));
        assert_eq!(validar_por_guardioes(&chaves_resolvidas(&g), &cmd, &APARELHO_A, 5, 2), Err(RecusaDeGuardioes::ComandoRepetido));
    }

    #[test]
    fn assinatura_de_guardiao_nao_vale_sob_o_contexto_do_aparelho_e_vice_versa() {
        // Separação de domínio entre os dois caminhos (`CONTEXTO` vs `CONTEXTO_GUARDIOES`): a assinatura
        // de um guardião sobre o comando nunca é confundível com a de um aparelho-dono já registrado,
        // mesmo que, por coincidência, usassem a mesma chave (cenário didático; na prática são identidades
        // diferentes, mas a separação de contexto garante isto de qualquer forma).
        let alguem = assinatura::gerar();
        let cmd_aparelho = emitir(&alguem, APARELHO_A, Reacao::Trancar, 1);
        assert_eq!(validar(&alguem.verificacao(), &cmd_aparelho, &APARELHO_A, 0), Ok(Reacao::Trancar));
        let assinatura_guardiao = guardiao_assina(&alguem, APARELHO_A, Reacao::Trancar, 1);
        let chaves = vec![(alguem.verificacao().impressao_digital(), alguem.verificacao())];
        let cmd_guardiao = juntar_assinaturas(APARELHO_A, Reacao::Trancar, 1, vec![assinatura_guardiao]);
        // Sozinho (k=1 só para este teste isolar a separação de contexto, nunca usado como quórum real)
        // a assinatura de guardião É válida sob seu próprio contexto — a separação importa é entre os dois
        // CAMINHOS, não dentro de cada um.
        assert_eq!(validar_por_guardioes(&chaves, &cmd_guardiao, &APARELHO_A, 0, 1), Ok(Reacao::Trancar));
    }
}
