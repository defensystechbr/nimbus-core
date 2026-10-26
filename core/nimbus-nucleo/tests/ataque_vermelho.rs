// Ataque vermelho permanente: cada teste reproduz um ataque real feito contra o próprio Nimbus.
// Se algum falhar, uma defesa foi perdida (ver NimbusOne_Relatorio_de_Ataque.md).
use nimbus_nucleo::{apresentacao, inspecao};

#[test] fn a1_nome_reservado_com_cirilico() { assert!(apresentacao::nome_permitido("Suporte N\u{0456}mbus").is_err(), "i cirílico"); }
#[test] fn a2_nome_reservado_largura_total() { assert!(apresentacao::nome_permitido("\u{FF2E}\u{FF49}\u{FF4D}\u{FF42}\u{FF55}\u{FF53}").is_err(), "Nimbus em largura total"); }
#[test] fn a3_nome_reservado_matematico() { assert!(apresentacao::nome_permitido("\u{1D40D}\u{1D422}\u{1D426}\u{1D41B}\u{1D42E}\u{1D42C}").is_err(), "Nimbus em letras matemáticas"); }
#[test] fn a4_parecido_com_cirilico() { assert_eq!(apresentacao::parecido_com("R\u{0435}nata", &["Renata"]), Some("Renata"), "e cirílico"); }
#[test] fn a5_largura_zero() { assert!(apresentacao::nome_permitido("Nim\u{200B}bus").is_err(), "espaço de largura zero"); }
#[test] fn a6_texto_bidi() { assert!(!inspecao::texto_seguro_para_exibir("abc\u{202E}def")); }
#[test] fn a7_grego() { assert!(apresentacao::nome_permitido("\u{039D}imbus").is_err(), "N grego"); }

#[test] fn a8_epoca_extrema_nao_derruba() {
    let mut k = nimbus_nucleo::metadados::ChavesDeIndice::default();
    let _ = k.indice(u64::MAX, &[1; 32]);
    let _ = k.destruir_vencidas(u64::MAX);
}
#[test] fn a9_porteiro_sob_martelo_aleatorio() {
    use nimbus_nucleo::{assinatura, autenticacao::*};
    use rand_core::{OsRng, RngCore};
    let mut p = Porteiro::default();
    let k = assinatura::gerar();
    let _ = p.registrar(k.verificacao(), TipoDeTitular::Caixa);
    let metodos = ["GET", "POST", "PUT", "DELETE", "", "post", "GET\0"];
    let rodadas: u32 = if cfg!(debug_assertions) { 2_000 } else { 20_000 };
    for i in 0..rodadas {
        let mut b = [0u8; 64]; OsRng.fill_bytes(&mut b);
        let n = (b[0] as usize) % 70;
        let caminho: String = if b[1].is_multiple_of(3) { ROTAS[(b[2] as usize) % ROTAS.len()].caminho.to_string() }
                              else { String::from_utf8_lossy(&b[3..3 + n.min(60)]).into_owned() };
        let metodo = metodos[(b[2] as usize) % metodos.len()];
        let corpo = vec![b[4]; (u32::from_be_bytes([b[5], b[6], b[7], b[8]]) % 70_000) as usize];
        let agora = u64::from_be_bytes([b[9], b[10], b[11], b[12], b[13], b[14], b[15], b[16]]);
        let prova = if b[17].is_multiple_of(2) { Some(provar(&k, metodo, &caminho, &corpo, agora.wrapping_add(b[18] as u64))) } else { None };
        let r = p.autorizar(metodo, &caminho, &corpo, prova.as_ref(), b[19].is_multiple_of(2), &i.to_be_bytes(), agora);
        let (codigo, corpo_resp) = resposta_publica(&r);
        assert!(matches!(codigo, 200 | 401 | 404 | 413 | 429 | 503) && corpo_resp.len() < 32);
    }
}

// Ataque B4: marcas combinantes, seletores de variação e espaços estranhos.
#[test] fn b4_marcas_e_seletores_nao_disfarcam_nome_reservado() {
    for ruim in ["N\u{0308}imbus", "Nimbus\u{FE0F}", "Nim\u{00AD}bus", "Nim\u{2060}bus", "N\u{200D}imbus", "Suporte\u{00A0}Nimbus", "SUPORTE  N I M B U S"] {
        assert!(apresentacao::nome_permitido(ruim).is_err(), "{ruim:?}");
    }
}
