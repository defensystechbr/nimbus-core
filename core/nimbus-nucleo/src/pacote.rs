//! # Módulo `pacote` — até o nome do arquivo é segredo (MTA1–MTA3)
//!
//! Nome da foto, nome do arquivo, tipo, data, pasta ou álbum: tudo isso vai **dentro** do mesmo envelope
//! cifrado do conteúdo. O servidor recebe só um bloco opaco, de tamanho padronizado, com identificador
//! aleatório. No aparelho, o nome só aparece depois de decifrado na memória — nunca vira nome de arquivo
//! no disco, nunca aparece em registro, nunca vai para índice do sistema.
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use crate::higiene;
use zeroize::Zeroizing;

const MAGICO: &[u8; 4] = b"NIT1";
const CONTEXTO: &[u8] = b"NimbusOne/v1/item-selado";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadados {
    pub nome: String,
    pub tipo: String,
    pub criado_em: u64,
    pub pasta: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDePacote { CampoGrandeDemais, GrandeDemais, Ilegivel, Malformado }

fn campo(v: &mut Vec<u8>, s: &str) -> Result<(), ErroDePacote> {
    let b = s.as_bytes();
    if b.len() > u16::MAX as usize { return Err(ErroDePacote::CampoGrandeDemais); }
    v.extend_from_slice(&(b.len() as u16).to_be_bytes());
    v.extend_from_slice(b);
    Ok(())
}

fn ler_campo(d: &[u8], i: &mut usize) -> Result<String, ErroDePacote> {
    if *i + 2 > d.len() { return Err(ErroDePacote::Malformado); }
    let n = u16::from_be_bytes([d[*i], d[*i + 1]]) as usize;
    *i += 2;
    if *i + n > d.len() { return Err(ErroDePacote::Malformado); }
    let s = std::str::from_utf8(&d[*i..*i + n]).map_err(|_| ErroDePacote::Malformado)?.to_string();
    *i += n;
    Ok(s)
}

/// Sela metadados e conteúdo juntos, num único bloco opaco.
pub fn selar_item(chave: &ChavePublica, meta: &Metadados, conteudo: &[u8]) -> Result<Vec<u8>, ErroDePacote> {
    // Capacidade exata calculada antes: o vetor nunca realoca, então nenhuma cópia do nome fica esquecida
    // em memória liberada.
    let total = 4 + 2 + meta.nome.len() + 2 + meta.tipo.len() + 8 + 2 + meta.pasta.len() + conteudo.len();
    let mut claro = Zeroizing::new(Vec::with_capacity(total));
    claro.extend_from_slice(MAGICO);
    campo(&mut claro, &meta.nome)?;
    campo(&mut claro, &meta.tipo)?;
    claro.extend_from_slice(&meta.criado_em.to_be_bytes());
    campo(&mut claro, &meta.pasta)?;
    claro.extend_from_slice(conteudo);
    envelope::selar(chave, &claro, CONTEXTO).map_err(|_| ErroDePacote::GrandeDemais)
}

/// Abre no aparelho do dono. Metadados e conteúdo só existem aqui, em memória.
pub fn abrir_item(sec: &ChaveSecreta, publ: &ChavePublica, bloco: &[u8]) -> Result<(Metadados, Zeroizing<Vec<u8>>), ErroDePacote> {
    let claro = envelope::abrir(sec, publ, bloco, CONTEXTO).map_err(|_| ErroDePacote::Ilegivel)?;
    if claro.len() < 4 || &claro[..4] != MAGICO { return Err(ErroDePacote::Malformado); }
    let mut i = 4;
    let nome = ler_campo(&claro, &mut i)?;
    let tipo = ler_campo(&claro, &mut i)?;
    if i + 8 > claro.len() { return Err(ErroDePacote::Malformado); }
    let mut t = [0u8; 8];
    t.copy_from_slice(&claro[i..i + 8]);
    let criado_em = u64::from_be_bytes(t);
    i += 8;
    let pasta = ler_campo(&claro, &mut i)?;
    Ok((Metadados { nome, tipo, criado_em, pasta }, Zeroizing::new(claro[i..].to_vec())))
}

/// Erro de `selar_midia`: ou a higienização recusou o arquivo, ou o selamento em si falhou.
#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeSelarMidia {
    Higiene(higiene::ErroDeHigiene),
    Pacote(ErroDePacote),
}

/// Sela foto, vídeo ou documento depois de passar, SEM EXCEÇÃO, por `higiene::limpar` primeiro — nunca os
/// bytes originais. Isto é o que torna "nunca sai com metadado/GPS" uma garantia ESTRUTURAL, não uma
/// convenção que a ponte poderia esquecer de seguir: quem for selar mídia bruta (foto, vídeo, documento —
/// vinda de importação OU tirada pela câmera do próprio app) chama só esta função, nunca `selar_item`
/// direto com bytes originais. `selar_item` continua existindo (usado também para selar algo que já não é
/// mídia bruta, como texto, que não passa por higienização). O `CONTRATO_DA_PONTE.md` (regra nova desta
/// rodada) exige que a ponte use `selar_midia` para todo conteúdo de mídia, nunca `selar_item` com bytes
/// não higienizados. Ver a nota no topo de `higiene.rs` para o que "impossível" cobre de verdade (o ARQUIVO
/// nunca carrega GPS/metadado depois de limpo — testável) e o que não cobre (reconhecer o local pelo
/// CONTEÚDO visual da foto — fora do alcance de qualquer software).
pub fn selar_midia(
    chave: &ChavePublica,
    meta: &Metadados,
    bytes_originais: &[u8],
) -> Result<Vec<u8>, ErroDeSelarMidia> {
    let limpo = higiene::limpar(bytes_originais).map_err(ErroDeSelarMidia::Higiene)?;
    selar_item(chave, meta, &limpo).map_err(ErroDeSelarMidia::Pacote)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn contem_nos_bytes(h: &[u8], a: &[u8]) -> bool { h.windows(a.len()).any(|w| w == a) }

    fn seg(m: u8, conteudo: &[u8]) -> Vec<u8> {
        let mut v = vec![0xFFu8, m];
        v.extend_from_slice(&((conteudo.len() + 2) as u16).to_be_bytes());
        v.extend_from_slice(conteudo);
        v
    }

    /// JPEG com um rastro de GPS no EXIF — mesma estrutura já provada em `higiene.rs`, reconstruída aqui
    /// porque os módulos de teste são isolados entre si.
    fn jpeg_com_gps() -> Vec<u8> {
        let mut j = vec![0xFFu8, 0xD8];
        j.extend(seg(0xE0, b"JFIF\0\x01\x01")); // APP0, mantido pela higienização
        j.extend(seg(0xE1, b"Exif\0\0GPS-LAT-23.5-LON-46.6")); // APP1/EXIF com GPS, removido
        j.extend(seg(0xDA, &[1, 1, 0, 0, 63, 0])); // início da imagem
        j.extend_from_slice(&[0x12, 0x34, 0xFF, 0x00, 0x56]); // dados da imagem (copiados como estão)
        j.extend_from_slice(&[0xFF, 0xD9]); // fim
        j
    }

    #[test]
    fn selar_midia_remove_gps_mesmo_que_quem_chama_esqueca_de_limpar_primeiro() {
        // A prova central deste módulo: quem chama `selar_midia` com os bytes ORIGINAIS (ainda com GPS)
        // nunca consegue selar o rastro — a higienização acontece DENTRO da função, sem exceção. Isto é o
        // que torna "nunca sai com metadado" impossível de burlar por esquecimento da ponte.
        let (p, s) = envelope::gerar_par_de_chaves();
        let bruto = jpeg_com_gps();
        assert!(contem_nos_bytes(&bruto, b"GPS-LAT"), "a foto de teste de fato carrega o rastro antes de selar");
        let bloco = selar_midia(&p, &meta(), &bruto).unwrap();
        let (_, conteudo) = abrir_item(&s, &p, &bloco).unwrap();
        assert!(!contem_nos_bytes(&conteudo, b"GPS-LAT"), "GPS sobreviveu ao selamento: higiene não foi aplicada");
        assert!(!contem_nos_bytes(&conteudo, b"Exif"), "marcador EXIF sobreviveu ao selamento");
        assert!(contem_nos_bytes(&conteudo, b"JFIF"), "mas o cabeçalho necessário para a imagem continua lá");
    }

    #[test]
    fn selar_midia_recusa_formato_que_a_higiene_nao_sabe_limpar() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let erro = selar_midia(&p, &meta(), b"%PDF-1.7 ... /Author (Gabriel)").unwrap_err();
        assert_eq!(erro, ErroDeSelarMidia::Higiene(higiene::ErroDeHigiene::PrecisaAchatar));
    }

    #[test]
    fn selar_midia_recusa_heic_com_o_erro_especifico_de_converter() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let heic = [0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c'];
        let erro = selar_midia(&p, &meta(), &heic).unwrap_err();
        assert_eq!(erro, ErroDeSelarMidia::Higiene(higiene::ErroDeHigiene::PrecisaConverter));
    }

    fn meta() -> Metadados {
        Metadados { nome: "Passaporte_Gabriel_2026.jpg".into(), tipo: "image/jpeg".into(), criado_em: 1_790_000_000, pasta: "Viagem Espanha".into() }
    }
    fn contem(h: &[u8], a: &[u8]) -> bool { h.windows(a.len()).any(|w| w == a) }

    #[test]
    fn ida_e_volta_com_nome_e_pasta() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let bloco = selar_item(&p, &meta(), b"bytes da foto").unwrap();
        let (m, c) = abrir_item(&s, &p, &bloco).unwrap();
        assert_eq!(m, meta());
        assert_eq!(&c[..], b"bytes da foto");
    }

    #[test]
    fn servidor_nao_ve_nome_tipo_nem_pasta() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let bloco = selar_item(&p, &meta(), b"x").unwrap();
        for rastro in [&b"Passaporte"[..], b"Gabriel", b"jpeg", b"Espanha", b"NIT1"] {
            assert!(!contem(&bloco, rastro));
        }
    }

    #[test]
    fn nome_longo_ou_curto_nao_muda_o_tamanho_do_bloco() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let curto = Metadados { nome: "a".into(), ..meta() };
        let longo = Metadados { nome: "Contrato_de_Cessao_Final_Assinado_versao_7.pdf".into(), ..meta() };
        assert_eq!(selar_item(&p, &curto, b"x").unwrap().len(), selar_item(&p, &longo, b"x").unwrap().len());
    }

    #[test]
    fn bloco_alterado_nao_abre() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let mut bloco = selar_item(&p, &meta(), b"x").unwrap();
        let n = bloco.len() - 1;
        bloco[n] ^= 1;
        assert_eq!(abrir_item(&s, &p, &bloco).unwrap_err(), ErroDePacote::Ilegivel);
    }

    #[test]
    fn campo_absurdo_e_recusado() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let m = Metadados { nome: "x".repeat(70_000), ..meta() };
        assert_eq!(selar_item(&p, &m, b"x").unwrap_err(), ErroDePacote::CampoGrandeDemais);
    }
}
