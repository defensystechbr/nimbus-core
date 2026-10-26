use nimbus_nucleo::{assinatura, autenticacao::*};
use std::time::Instant;
// Ataque vermelho B1: o tempo da recusa não pode revelar se um titular (caixa ou aparelho) está registrado.
// Rodado por ./verificar.sh --completo (perfil otimizado).
#[test] #[ignore] fn tempo_de_recusa_nao_revela_titular() {
    let mut p = Porteiro::default();
    let k = assinatura::gerar();
    let _ = p.registrar(k.verificacao(), TipoDeTitular::Caixa);
    let estranho = assinatura::gerar();
    let mut med = |prova_de: &assinatura::ChaveDeAssinatura, forjar_titular: bool, base: u32| -> u128 {
        let mut v = Vec::new();
        for i in 0..200u32 {
            let mut pr = provar(prova_de, "POST", "/v1/caixa/retirar", b"", 1_000);
            if forjar_titular { pr.titular = k.verificacao().impressao_digital(); }
            let t = Instant::now();
            let _ = p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&pr), false, &(base + i).to_be_bytes(), 1_000);
            v.push(t.elapsed().as_nanos());
        }
        v.sort(); v[v.len() / 2]
    };
    let desconhecido = med(&estranho, false, 0);      // titular não registrado
    let registrado = med(&estranho, true, 10_000);         // titular registrado, assinatura errada
    let razao = registrado as f64 / desconhecido as f64;
    println!("mediana titular desconhecido: {desconhecido} ns; titular registrado com assinatura errada: {registrado} ns; razão {razao:.2}");
    assert!((0.67..=1.5).contains(&razao), "o tempo revela se o titular existe (razão {razao:.2})");
}
