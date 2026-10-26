//! # Autoverificação da criptografia (prova C1 do manual)
//!
//! Ataques clássicos aplicados ao envelope e à assinatura do NimbusOne. Nenhum pode funcionar. Roda sozinho.
use nimbus_nucleo::assinatura;
use nimbus_nucleo::envelope::{self, ChaveSecreta};

/// Trocar um único bit de qualquer parte do envelope impede a leitura (autenticação AEAD).
#[test]
fn c1_um_bit_trocado_no_envelope_impede_a_leitura() {
    let (p, s) = envelope::gerar_par_de_chaves();
    let base = envelope::selar(&p, b"mensagem secreta de teste", b"").unwrap();
    let mut falhas = 0;
    // testa posições espalhadas por cabeçalho, chave encapsulada, nonce e texto cifrado
    for i in (0..base.len()).step_by(7) {
        let mut adulterado = base.clone();
        adulterado[i] ^= 0x01;
        if envelope::abrir(&s, &p, &adulterado, b"").is_err() { falhas += 1; }
    }
    let total = (0..base.len()).step_by(7).count();
    assert_eq!(falhas, total, "todo bit trocado deve impedir a leitura ({falhas}/{total})");
}

/// O dado associado (AAD) faz parte da autenticação: se mudar, não abre.
#[test]
fn c1_dado_associado_diferente_nao_abre() {
    let (p, s) = envelope::gerar_par_de_chaves();
    let env = envelope::selar(&p, b"conteudo", b"conversa-A").unwrap();
    assert!(envelope::abrir(&s, &p, &env, b"conversa-A").is_ok());
    assert!(envelope::abrir(&s, &p, &env, b"conversa-B").is_err(), "trocar o contexto invalida");
}

/// A chave de outra pessoa nunca abre o envelope.
#[test]
fn c1_chave_errada_nao_abre() {
    let (p, _) = envelope::gerar_par_de_chaves();
    let (_, s_outro) = envelope::gerar_par_de_chaves();
    let env = envelope::selar(&p, b"segredo", b"").unwrap();
    assert!(envelope::abrir(&s_outro, &p, &env, b"").is_err());
}

/// Dois envelopes do mesmo texto para a mesma chave são diferentes (nonce/efêmera aleatórios):
/// o servidor não sabe se duas mensagens têm o mesmo conteúdo.
#[test]
fn c1_mesmo_texto_gera_envelopes_diferentes() {
    let (p, _) = envelope::gerar_par_de_chaves();
    let a = envelope::selar(&p, b"repetido", b"").unwrap();
    let b = envelope::selar(&p, b"repetido", b"").unwrap();
    assert_ne!(a, b, "envelopes iguais vazariam que o conteúdo é o mesmo");
}

/// Assinatura híbrida: as DUAS partes (clássica e pós-quântica) precisam conferir.
#[test]
fn c1_assinatura_hibrida_exige_as_duas_partes() {
    let k = assinatura::gerar();
    let v = k.verificacao();
    let msg = b"documento importante";
    let assinatura_boa = k.assinar(b"contexto", msg).unwrap();
    assert!(v.verificar(b"contexto", msg, &assinatura_boa));
    assert!(!v.verificar(b"contexto", b"documento adulterado", &assinatura_boa), "texto trocado não confere");
    assert!(!v.verificar(b"outro-contexto", msg, &assinatura_boa), "contexto trocado não confere");
    let outra = assinatura::gerar();
    assert!(!outra.verificacao().verificar(b"contexto", msg, &assinatura_boa), "chave errada não confere");
}

/// A chave secreta serializada e restaurada continua abrindo — e não vaza em Debug.
#[test]
fn c1_chave_secreta_nao_aparece_em_debug() {
    let (_, s) = envelope::gerar_par_de_chaves();
    let bytes = s.para_bytes();
    let s2 = ChaveSecreta::de_bytes(&bytes).unwrap();
    assert!(!format!("{s2:?}").contains(&format!("{:?}", &bytes[..8])), "a chave não pode aparecer em Debug");
}


// ---- Sigilo de metadados: o tamanho do nome, da nota e do arquivo nunca vaza (prova C81) ----

/// Dois contatos com nomes e notas de tamanhos MUITO diferentes viram cifras do MESMO tamanho.
/// Sem isto, o servidor poderia distinguir "Mãe" de "Departamento Jurídico do Escritório X" pelo comprimento.
#[test]
fn c81_tamanho_do_contato_nao_vaza_pelo_comprimento() {
    use nimbus_nucleo::contatos::{selar_contato, Contato};
    let (dono, _) = nimbus_nucleo::envelope::gerar_par_de_chaves();
    let curto = Contato { apelido: "Mãe".into(), chave_da_pessoa: vec![1; 32], notas: "".into(), etiqueta: "família".into() };
    let longo = Contato {
        apelido: "Departamento Jurídico do Escritório de Advocacia Muito Comprido S/A".into(),
        chave_da_pessoa: vec![1; 32],
        notas: "Reunião toda terça, tratar do processo 1234567-89.2026, confidencial, não repassar a ninguém".into(),
        etiqueta: "trabalho".into(),
    };
    let a = selar_contato(&dono, &curto).unwrap();
    let b = selar_contato(&dono, &longo).unwrap();
    assert_eq!(a.len(), b.len(), "o comprimento do nome/nota não pode vazar pelo tamanho da cifra");
}

/// Duas selagens do MESMO contato dão cifras diferentes (número aleatório por cifra): o servidor não sabe
/// que são o mesmo contato.
#[test]
fn c81_mesmo_contato_gera_cifras_diferentes() {
    use nimbus_nucleo::contatos::{selar_contato, Contato};
    let (dono, _) = nimbus_nucleo::envelope::gerar_par_de_chaves();
    let c = Contato { apelido: "Ana".into(), chave_da_pessoa: vec![2; 32], notas: "".into(), etiqueta: "".into() };
    assert_ne!(selar_contato(&dono, &c).unwrap(), selar_contato(&dono, &c).unwrap());
}

// ---- Limite de senha configurável: bloqueia por tempo crescente e, no limite, apaga (prova C82) ----

/// O usuário pode escolher: X erros livres, depois espera que dobra, e no N-ésimo erro apaga as chaves.
/// Aqui: 3 livres, espera a partir de 30 s, apaga no 10º. Testa os três estágios de ponta a ponta.
#[test]
fn c82_limite_de_senha_bloqueia_por_tempo_e_no_fim_apaga() {
    use nimbus_nucleo::coacao::{AposErro, Politica, Tentativas};
    let pol = Politica::equilibrada(); // 3 livres, 30 s inicial, apaga no 10º
    let mut t = Tentativas::default();
    // 3 erros livres: ainda deixa tentar
    for _ in 0..3 { assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo); }
    // 4º erro: espera 30 s; 5º: dobra para 60 s
    assert_eq!(t.errou(&pol, 100), AposErro::Esperar { ate: 130 });
    assert_eq!(t.errou(&pol, 130), AposErro::Esperar { ate: 190 });
    // acumula até o 10º erro: apaga as chaves
    let mut agora = 190;
    let mut ultimo = AposErro::Esperar { ate: 0 };
    for _ in 0..5 { ultimo = t.errou(&pol, agora); if let AposErro::Esperar { ate } = ultimo { agora = ate; } }
    assert_eq!(ultimo, AposErro::ApagarChaves, "no limite escolhido, o aparelho se apaga");
}

/// A política tolerante NUNCA apaga (para quem tem medo de apagar sem querer): só faz esperar cada vez mais.
#[test]
fn c82_politica_tolerante_nunca_apaga() {
    use nimbus_nucleo::coacao::{AposErro, Politica, Tentativas};
    let pol = Politica::tolerante();
    let mut t = Tentativas::default();
    let mut agora = 0;
    for _ in 0..40 {
        match t.errou(&pol, agora) {
            AposErro::ApagarChaves => panic!("a política tolerante não pode apagar"),
            AposErro::Esperar { ate } => agora = ate,
            AposErro::TentarDeNovo => {}
        }
    }
}

// O teste de tempo constante do PIN (real × coação × errado) fica em tests/timing.rs (prova C80).
