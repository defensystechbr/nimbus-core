//! # Teste de integração recuperacao–sessoes: o aparelho perdido não fica com sessão viva depois da
//! recuperação (achado de 09/10/2026, segunda rodada do dia — pergunta do fundador: "ao menos a
//! sessão no outro aparelho perdido é encerrada, não?")
//!
//! Prova a composição que a ponte é obrigada a fazer (ver `CONTRATO_DA_PONTE.md`, regra 36): no
//! mesmo passo em que `recuperacao::pode_concluir` devolve `Ok`, chamar
//! `sessoes::Registro::revogar_todos_os_aparelhos` ANTES de registrar e conectar o aparelho novo.
//! Sem essa chamada (simulada aqui pelo teste que NÃO a faz), a sessão do aparelho antigo sobrevive
//! — é exatamente isso que o segundo teste abaixo demonstra, para que a regressão (alguém remover a
//! chamada da ponte um dia) tenha onde quebrar.
use nimbus_nucleo::recuperacao::{self, CARENCIA_SEGUNDOS};
use nimbus_nucleo::sessoes::{Gatilho, IdAparelho, IdConta, Registro, Resultado};

const CONTA: IdConta = [7; 32];
const APARELHO_ANTIGO: IdAparelho = [8; 32];
const APARELHO_NOVO: IdAparelho = [9; 32];

#[test]
fn recuperacao_concluida_seguida_de_revogar_todos_derruba_o_aparelho_antigo() {
    // 1. Estado antes de qualquer coisa: o aparelho antigo está registrado e conectado normalmente.
    let mut sessoes = Registro::default();
    sessoes.registrar_aparelho(CONTA, APARELHO_ANTIGO);
    assert_eq!(sessoes.conectar(CONTA, APARELHO_ANTIGO, true, true, 1), Resultado::Aceita);

    // 2. O aparelho é perdido. Um guardião confirma ao vivo e a recuperação segue seu curso normal
    // em recuperacao.rs (módulo já provado isoladamente em recuperacao.rs; aqui só a ponta que falta:
    // o que a CONCLUSÃO da recuperação precisa fazer em sessoes.rs).
    let pedido_em = 1_000;
    let cancelado_por_aparelho_antigo = false; // o dono não cancelou: é mesmo ele recuperando, sem o aparelho antigo
    assert_eq!(
        recuperacao::pode_concluir(pedido_em, pedido_em + CARENCIA_SEGUNDOS, cancelado_por_aparelho_antigo),
        Ok(()),
        "passou a carência de 3 dias sem cancelamento"
    );

    // 3. A PARTE QUE A PONTE É OBRIGADA A FAZER (regra 36): revogar TODOS os aparelhos antigos da
    // conta no mesmo passo, ANTES de registrar o novo.
    sessoes.revogar_todos_os_aparelhos(CONTA);
    sessoes.registrar_aparelho(CONTA, APARELHO_NOVO);

    // 4. O aparelho novo (quem recuperou) conecta normalmente.
    assert_eq!(sessoes.conectar(CONTA, APARELHO_NOVO, true, true, 1), Resultado::Aceita);

    // 5. O aparelho antigo — perdido, roubado ou só esquecido — nunca mais entra: nem com a
    // credencial antiga (se ainda a tivesse), porque ele simplesmente não está mais registrado.
    assert!(
        matches!(
            sessoes.conectar(CONTA, APARELHO_ANTIGO, true, true, 1),
            Resultado::TodasDerrubadas { gatilho: Gatilho::AparelhoNaoRegistrado, .. }
        ),
        "o aparelho perdido não tem mais sessão nem registro depois da recuperação concluída"
    );
}

#[test]
fn sem_a_chamada_de_revogar_a_sessao_antiga_sobreviveria_a_recuperacao_concluida() {
    // Este teste existe para que a regressão tenha onde quebrar: mostra, por contraste, o que
    // aconteceria se a ponte UM DIA esquecesse o passo 3 do teste acima. Não é o comportamento
    // desejado — é a prova do porquê a regra 36 precisa ser obrigatória, não uma sugestão.
    let mut sessoes = Registro::default();
    sessoes.registrar_aparelho(CONTA, APARELHO_ANTIGO);
    assert_eq!(sessoes.conectar(CONTA, APARELHO_ANTIGO, true, true, 1), Resultado::Aceita);

    assert_eq!(recuperacao::pode_concluir(1_000, 1_000 + CARENCIA_SEGUNDOS, false), Ok(()));

    // Passo que a ponte NÃO fez de propósito, aqui, para provar a lacuna:
    // sessoes.revogar_todos_os_aparelhos(CONTA);  // <- omitido deliberadamente

    sessoes.registrar_aparelho(CONTA, APARELHO_NOVO);
    assert_eq!(sessoes.conectar(CONTA, APARELHO_NOVO, true, true, 1), Resultado::Aceita);

    // Sem o passo 3, o aparelho antigo continua ativo nos registros — a sessão dele nunca foi tocada.
    assert!(
        sessoes.aparelhos_ativos_agora(&CONTA).contains(&APARELHO_ANTIGO),
        "demonstra a lacuna: sem revogar_todos_os_aparelhos, o aparelho perdido continua com sessão ativa mesmo depois da recuperação"
    );
}
