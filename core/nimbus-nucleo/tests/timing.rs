//! # Tempo constante (prova C80 do manual)
//!
//! O que se prova: o PIN real, o PIN de coação e um PIN errado levam o mesmo tempo para avaliar. Quem mede
//! o tempo de resposta não descobre qual foi digitado, nem se existe um PIN de coação. Errar no primeiro ou
//! no último dígito também leva o mesmo tempo.
//!
//! Por que 15 medições bastam: cada avaliação roda o Argon2id de 64 MiB de propósito (≈ 0,1 s), que é o que
//! um atacante consegue observar. A comparação dos bytes usa `subtle::ConstantTimeEq`, e o verificador sempre
//! faz as duas comparações antes de decidir (`coacao::Verificador::avaliar`); uma diferença de nanossegundos
//! ali some diante do Argon2id. A mediana de 15 medições é estável.
//!
//! É pesado de propósito, por isso fica fora do `cargo test` do dia a dia. Rode:
//!     cargo test --profile verificacao --test timing -- --ignored --nocapture
//! (o `./verificar.sh --completo` já faz isso).
use nimbus_nucleo::coacao::{Abertura, Verificador};
use std::time::Instant;

const MEDICOES: usize = 15;
/// Maior diferença aceita entre o caso mais lento e o mais rápido.
const MARGEM: f64 = 1.25;

fn mediana_ms(v: &Verificador, entrada: &[u8], esperado: &Abertura) -> f64 {
    let mut amostras: Vec<f64> = Vec::with_capacity(MEDICOES);
    for _ in 0..MEDICOES {
        let inicio = Instant::now();
        let resultado = v.avaliar(entrada);
        amostras.push(inicio.elapsed().as_secs_f64() * 1_000.0);
        assert_eq!(&resultado, esperado, "a avaliação precisa dar o resultado certo em toda medição");
    }
    amostras.sort_by(f64::total_cmp);
    amostras[MEDICOES / 2]
}

#[test]
#[ignore = "pesado (Argon2id de 64 MiB por tentativa): cargo test --profile verificacao --test timing -- --ignored"]
fn c80_real_coacao_e_errado_levam_o_mesmo_tempo() {
    let v = Verificador::novo([7u8; 16], b"123456", b"999999").expect("segredos válidos");
    for _ in 0..2 { let _ = v.avaliar(b"000000"); } // aquece memória e caches antes de medir
    let casos = [
        ("PIN real", mediana_ms(&v, b"123456", &Abertura::CofreReal)),
        ("PIN de coação", mediana_ms(&v, b"999999", &Abertura::CofreIscaComProtecao)),
        ("errado no 1º dígito", mediana_ms(&v, b"923456", &Abertura::Negada)),
        ("errado no último dígito", mediana_ms(&v, b"123457", &Abertura::Negada)),
    ];
    for (nome, ms) in &casos { println!("  {nome}: {ms:.1} ms (mediana de {MEDICOES})"); }
    let maior = casos.iter().map(|c| c.1).fold(f64::MIN, f64::max);
    let menor = casos.iter().map(|c| c.1).fold(f64::MAX, f64::min);
    let razao = maior / menor;
    println!("  maior ÷ menor = {razao:.3} (limite {MARGEM})");
    assert!(razao < MARGEM, "diferença de {razao:.2}× entre os casos: o tempo pode revelar qual PIN foi digitado");
}
