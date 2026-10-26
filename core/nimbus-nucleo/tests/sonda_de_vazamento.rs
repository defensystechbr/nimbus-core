//! Sonda de vazamento (anti-forense): o que um equipamento de perícia encontraria nos bytes que o núcleo entrega
//! para a camada nativa guardar no aparelho ou enviar ao servidor?
//!
//! Cada teste passa um CANÁRIO (texto único e reconhecível) pelas funções que produzem bytes persistidos e
//! procura o canário nos bytes de saída em todas as formas que uma ferramenta de busca tentaria: UTF-8, UTF-16
//! (as duas ordens), base64, hexadecimal e o próprio texto invertido. Achar qualquer forma = vazamento.
//! Também confere que o tamanho da saída não revela o tamanho do conteúdo dentro da mesma classe.
//!
//! Limite declarado: isto prova o que o NÚCLEO entrega. Cache, teclado, notificações e memória da camada nativa
//! se provam nos testes RSD em aparelho real (NIMBUSCHAT_SEM_RASTRO_NO_APARELHO.md).
use nimbus_nucleo::*;

const CANARIO: &str = "CANARIO-NIMBUS-7f3a9c-segredo-da-conversa";

fn formas(c: &str) -> Vec<Vec<u8>> {
    let utf16le: Vec<u8> = c.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    let utf16be: Vec<u8> = c.encode_utf16().flat_map(|u| u.to_be_bytes()).collect();
    let hex: Vec<u8> = c.bytes().flat_map(|b| format!("{b:02x}").into_bytes()).collect();
    let invertido: Vec<u8> = c.bytes().rev().collect();
    let b64 = base64_simples(c.as_bytes());
    vec![c.as_bytes().to_vec(), utf16le, utf16be, hex, invertido, b64, c.to_lowercase().into_bytes()]
}

fn base64_simples(d: &[u8]) -> Vec<u8> {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = Vec::new();
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 { if i <= c.len() { s.push(A[((n >> (18 - 6 * i)) & 63) as usize]); } }
    }
    s
}

/// Procura qualquer pedaço de 12 bytes de qualquer forma do canário (pega vazamento parcial também).
fn vaza(saida: &[u8]) -> Option<String> {
    for f in formas(CANARIO) {
        for jan in f.windows(12) {
            if saida.windows(12).any(|w| w == jan) { return Some(String::from_utf8_lossy(jan).into_owned()); }
        }
    }
    None
}

#[test]
fn cofre_nao_deixa_o_texto_nem_o_tipo_nem_o_tamanho() {
    let (publ, _sec) = envelope::gerar_par_de_chaves();
    let item = |t: &str| cofre::Item { tipo: cofre::Tipo::Nota, campos: vec![("titulo".into(), CANARIO.into())], conteudo: t.as_bytes().to_vec() };
    let a = cofre::selar(&publ, &item(CANARIO)).unwrap();
    assert_eq!(vaza(&a), None, "o cofre deixou o canário legível");
    let b = cofre::selar(&publ, &item("x")).unwrap();
    assert_eq!(a.len(), b.len(), "na mesma classe, o tamanho não revela o conteúdo");
    let foto = cofre::Item { tipo: cofre::Tipo::Foto, campos: vec![], conteudo: vec![0u8; 10] };
    assert_eq!(cofre::selar(&publ, &foto).unwrap().len(), b.len(), "o tipo não aparece no tamanho");
}

#[test]
fn contato_e_nome_de_perfil_nao_aparecem() {
    let (publ, _sec) = envelope::gerar_par_de_chaves();
    let c = contatos::Contato { apelido: CANARIO.into(), chave_da_pessoa: vec![1; 32], notas: CANARIO.into(), etiqueta: CANARIO.into() };
    let bloco = contatos::selar_contato(&publ, &c).unwrap();
    assert_eq!(vaza(&bloco), None, "contato guardado com texto legível");
    let k = contatos::ChaveDePerfil::nova();
    assert_eq!(vaza(&contatos::cifrar_nome_de_perfil(&k, CANARIO).unwrap()), None, "nome de perfil legível");
}

#[test]
fn indice_do_disco_e_bloco_selado_nao_aparecem() {
    let mut chaves = metadados::ChavesDeIndice::default();
    let endereco = [9u8; 32];
    let selado = chaves.selar_no_disco(10, &endereco, CANARIO.as_bytes());
    assert_eq!(vaza(&selado), None, "bloco do disco legível");
    let idx = chaves.indice(10, &endereco);
    assert!(!idx.windows(8).any(|w| endereco.windows(8).any(|e| e == w)), "o índice revela o endereço");
}

#[test]
fn historico_no_servidor_cego_nao_aparece() {
    let mut servidor = historico::ServidorCego::default();
    let mut chaves = historico::ChavesDoDono::novas(3_600);
    let _ = historico::guardar(&mut servidor, &mut chaves, CANARIO.as_bytes(), None).unwrap();
    for bloco in servidor.blocos.values() { assert_eq!(vaza(bloco), None, "o servidor recebeu texto legível"); }
}

#[test]
fn envelope_nao_deixa_o_canario_em_nenhuma_forma() {
    let (publ, sec) = envelope::gerar_par_de_chaves();
    let selado = envelope::selar(&publ, CANARIO.as_bytes(), b"sonda").unwrap();
    assert_eq!(vaza(&selado), None);
    assert_eq!(&envelope::abrir(&sec, &publ, &selado, b"sonda").unwrap()[..], CANARIO.as_bytes(), "e abre certo com a chave");
}

#[test]
fn a_propria_sonda_pega_vazamento_plantado() {
    // Conferidor atacado antes de ser usado como prova: cada forma plantada tem de ser achada.
    for f in formas(CANARIO) {
        let mut plantado = vec![0u8; 100];
        plantado.extend_from_slice(&f);
        plantado.extend_from_slice(&[0u8; 100]);
        assert!(vaza(&plantado).is_some(), "a sonda não achou uma forma plantada");
    }
    assert_eq!(vaza(&[0u8; 4096]), None, "falso positivo");
}
