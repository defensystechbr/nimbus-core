//! # Teste de integração servidor–aparelho: desafio de uso único (prova C77 do manual)
//!
//! Roda sozinho (não precisa de aparelho): prova que uma atestação capturada de um aparelho limpo não pode
//! ser reenviada a partir de um aparelho comprometido, e que o desafio vence. É a defesa contra "replay".
use nimbus_nucleo::integridade::{Desafios, RecusaDeDesafio, VALIDADE_DO_DESAFIO};

#[test]
fn c77_atestacao_reenviada_e_recusada() {
    let mut servidor = Desafios::default();
    // O servidor emite um desafio para o aparelho A (limpo), que responde a tempo.
    let desafio_a = servidor.emitir(1_000);
    assert_eq!(servidor.consumir(&desafio_a, 1_010), Ok(()), "aparelho limpo passa uma vez");
    // Um atacante captura essa resposta e tenta reenviá-la de um aparelho B (comprometido).
    assert_eq!(servidor.consumir(&desafio_a, 1_011), Err(RecusaDeDesafio::Desconhecido),
               "a mesma atestação não pode ser usada de novo");
}

#[test]
fn c77_desafio_vence_no_prazo() {
    let mut servidor = Desafios::default();
    let d = servidor.emitir(0);
    assert_eq!(servidor.consumir(&d, VALIDADE_DO_DESAFIO), Err(RecusaDeDesafio::Vencido));
    let d2 = servidor.emitir(0);
    assert_eq!(servidor.consumir(&d2, VALIDADE_DO_DESAFIO - 1), Ok(()));
}

#[test]
fn c77_cada_aparelho_recebe_um_desafio_diferente() {
    let mut servidor = Desafios::default();
    let a = servidor.emitir(0);
    let b = servidor.emitir(0);
    assert_ne!(a, b, "dois aparelhos nunca recebem o mesmo desafio");
}
