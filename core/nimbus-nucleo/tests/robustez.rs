//! # Robustez: entradas maliciosas ou corrompidas nunca derrubam o app
//!
//! Cada leitor de dados recebe milhares de entradas aleatórias — puras e com cabeçalhos válidos
//! seguidos de lixo. Resultado exigido: sempre um erro controlado, nunca uma queda (panic).
use nimbus_nucleo::{custodia, direto, recuperacao, envelope, higiene, inspecao, notificacao, pacote, registro};
use rand_core::{OsRng, RngCore};

fn lixo(max: usize) -> Vec<u8> {
    let n = (OsRng.next_u32() as usize) % (max + 1);
    let mut v = vec![0u8; n];
    OsRng.fill_bytes(&mut v);
    v
}

#[test]
fn higiene_nunca_cai_com_arquivo_corrompido() {
    for _ in 0..3_000 {
        let _ = higiene::limpar(&lixo(512));
        let mut jpeg = vec![0xFF, 0xD8];
        jpeg.extend(lixo(512));
        let _ = higiene::limpar(&jpeg);
        let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend(lixo(512));
        let _ = higiene::limpar(&png);
    }
}

#[test]
fn envelope_nunca_cai_com_bloco_adulterado() {
    let (p, s) = envelope::gerar_par_de_chaves();
    let valido = envelope::selar(&p, b"mensagem", b"").unwrap();
    for _ in 0..300 {
        let _ = envelope::abrir(&s, &p, &lixo(4_000), b"");
        let mut adulterado = valido.clone();
        let i = (OsRng.next_u32() as usize) % adulterado.len();
        adulterado[i] ^= 1 + (OsRng.next_u32() % 255) as u8;
        assert!(envelope::abrir(&s, &p, &adulterado, b"").is_err());
        let _ = envelope::ChavePublica::de_bytes(&lixo(1_700));
    }
}

#[test]
fn pacote_e_notificacao_nunca_caem() {
    let (p, s) = envelope::gerar_par_de_chaves();
    for _ in 0..300 {
        let _ = pacote::abrir_item(&s, &p, &lixo(3_000));
        let _ = notificacao::ao_receber(&s, &p, &lixo(3_000));
    }
}

#[test]
fn registro_nunca_aceita_prova_aleatoria() {
    let mut r = registro::Registro::default();
    for i in 0..20 { let _ = r.acrescentar(format!("chave-{i}").as_bytes()); }
    let raiz = r.raiz();
    for _ in 0..2_000 {
        let tamanho = (OsRng.next_u32() % 40) as usize;
        let indice = (OsRng.next_u32() % 45) as usize;
        let prova: Vec<[u8; 32]> = (0..(OsRng.next_u32() % 8)).map(|_| { let mut h = [0u8; 32]; OsRng.fill_bytes(&mut h); h }).collect();
        assert!(!registro::verificar_inclusao(b"chave-inventada", indice, tamanho, &prova, &raiz));
    }
    assert!(!registro::verificar_inclusao(b"x", 0, 0, &[], &raiz), "registro vazio não prova nada");
}

#[test]
fn custodia_nunca_cai_com_pecas_estranhas() {
    let (mut partes, _) = custodia::dividir(&[7u8; 64], 5, 3).unwrap();
    for _ in 0..500 {
        for p in partes.iter_mut() {
            for peca in p.pecas.iter_mut() {
                let novo = lixo(80);
                peca.dados = zeroize::Zeroizing::new(novo);
                peca.grupo = (OsRng.next_u32() % 20) as u16;
            }
            p.guardiao = (OsRng.next_u32() % 9) as u8;
        }
        let refs: Vec<&custodia::PartesDoGuardiao> = partes.iter().collect();
        let _ = custodia::reconstruir(&refs); // exigido: nunca cair, qualquer que seja a entrada
    }
}

#[test]
fn inspecao_e_envio_direto_nunca_caem_com_lixo() {
    let (p, sec) = envelope::gerar_par_de_chaves();
    for _ in 0..3_000 {
        let d = lixo(300);
        let _ = inspecao::inspecionar("x.jpg", &d); // exigido: nunca cair
        let r = direto::Remessa { id: [0; 16], bloco: lixo(3_000), valida_ate: u64::MAX };
        assert!(direto::abrir(&sec, &p, &r, 0).is_err());
    }
}

#[test]
fn recuperacao_e_partes_de_guardiao_nunca_caem_com_lixo() {
    for _ in 0..3_000 {
        let a = lixo(400);
        let b = lixo(400);
        let _ = custodia::PartesDoGuardiao::de_bytes(&a); // exigido: nunca cair
        assert!(recuperacao::reunir(&[&a, &b]).is_err());
    }
}
