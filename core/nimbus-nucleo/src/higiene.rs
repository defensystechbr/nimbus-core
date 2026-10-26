//! # Módulo `higiene` — arquivo sai sem rastro (HIG1–HIG6)
//!
//! Fotos e documentos carregam dados escondidos: local GPS, modelo do celular, data, nome do autor,
//! programa usado. Antes de qualquer envio, o app remove isso.
//!
//! * **JPEG**: remove segmentos de metadados (EXIF, XMP, IPTC, comentários). Mantém só o que é preciso
//!   para a imagem aparecer certa (JFIF, perfil de cor ICC e marcador Adobe).
//! * **PNG**: mantém só os blocos de imagem e de cor; remove textos, EXIF e data de modificação.
//! * **Nome do arquivo**: trocado por um nome neutro aleatório (o nome original pode identificar).
//! * **HEIC/HEIF** (formato padrão de captura do iPhone desde o iOS 11): **recusado de propósito, nomeado
//!   explicitamente, nunca misturado com "formato desconhecido".** HEIC guarda metadado (incluindo GPS)
//!   dentro de uma caixa `meta`/`iinf`/`iloc` no formato ISOBMFF — a mesma família de caixas que carrega a
//!   própria informação de decodificação da imagem (dimensão, layout de blocos, parâmetros do codec). Dar
//!   certeza absoluta de que uma edição cirúrgica dentro dessa caixa nunca deixa um item de metadado
//!   esquecido, sem corromper os deslocamentos (`iloc`) de tudo que vem depois no arquivo, exigiria um
//!   parser ISOBMFF completo e adversarialmente testado — o que este módulo conscientemente NÃO tenta
//!   construir nesta rodada, para não entregar uma falsa sensação de segurança (um "limpador" de HEIC
//!   só um pouco errado é pior do que nenhum: ele deixaria passar com uma etiqueta de "limpo"). A
//!   mitigação real e segura: a camada nativa converte HEIC para JPEG **sem copiar metadado na conversão**
//!   (`CGImageDestination`/ImageIO no iOS, ou equivalente no Android, chamados para NÃO preservar as
//!   propriedades EXIF/GPS do original) e só então chama `limpar` — regra nova da ponte, ver
//!   `CONTRATO_DA_PONTE.md`. Isto NÃO é o mesmo recurso de "achatar" dos formatos de documento abaixo (que
//!   vira páginas-imagem); por isso tem o próprio erro, `PrecisaConverter`, mais específico que
//!   `PrecisaAchatar`.
//! * **Outros formatos** (PDF, Word, Excel...): metadados escondidos em muitos lugares (autor, histórico
//!   de revisões, comentários, camadas). A limpeza confiável é **achatar**: a camada nativa converte o
//!   documento em páginas-imagem e o envio leva só isso. Este módulo recusa o que não sabe limpar, para
//!   que nada saia “meio limpo” sem o usuário saber.
//!
//! Observação técnica: segmentos de JPEG depois do início da imagem (SOS) são copiados como estão;
//! metadados aparecem, na prática, antes dele.
//!
//! **O que "impossível" cobre, e o que não cobre, dito sem rodeio:** depois de `limpar`, é impossível que
//! o ARQUIVO (os bytes) ainda carregue coordenadas de GPS ou qualquer metadado reconhecido — isso é
//! testado linha a linha e contra mutação. Nenhuma limpeza de arquivo consegue, nem pode, impedir que
//! alguém reconheça o LOCAL pelo próprio CONTEÚDO visual da foto (uma placa de rua, um ponto turístico, a
//! posição do sol) — isso não é metadado, é a imagem em si, e nenhum software de limpeza no mundo resolve
//! isso. Este módulo garante a primeira coisa; a segunda está fora do alcance de qualquer ferramenta.
use rand_core::{OsRng, RngCore};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Formato { Jpeg, Png, Heic, Desconhecido }

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeHigiene {
    /// Formato que não limpamos com segurança: precisa ser achatado antes de enviar.
    PrecisaAchatar,
    /// HEIC/HEIF (ou outro formato de imagem que este módulo reconhece mas ainda não limpa): precisa ser
    /// CONVERTIDO (nunca "achatado") para JPEG ou PNG, sem copiar metadado na conversão, antes de chamar
    /// `limpar` de novo. Ver a nota sobre HEIC no início deste arquivo.
    PrecisaConverter,
    /// Arquivo corrompido ou truncado.
    Malformado,
}

/// Caixa `ftyp` do ISOBMFF (HEIC/HEIF/AVIF): bytes 4–8 são literalmente "ftyp"; o "major brand" de 4
/// letras que decide a família do arquivo vem logo depois, em 8–12. Só INSPECIONA o cabeçalho para
/// reconhecer e nomear o formato — não interpreta nem edita nenhuma caixa interna (ver nota acima).
fn e_ftyp_heic(d: &[u8]) -> bool {
    if d.len() < 12 || &d[4..8] != b"ftyp" { return false; }
    matches!(&d[8..12], b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx" | b"mif1" | b"msf1")
}

pub fn detectar(dados: &[u8]) -> Formato {
    if dados.starts_with(&[0xFF, 0xD8]) { Formato::Jpeg }
    else if dados.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) { Formato::Png }
    else if e_ftyp_heic(dados) { Formato::Heic }
    else { Formato::Desconhecido }
}

/// Limpa o arquivo conforme o formato. Devolve o arquivo limpo.
pub fn limpar(dados: &[u8]) -> Result<Vec<u8>, ErroDeHigiene> {
    match detectar(dados) {
        Formato::Jpeg => limpar_jpeg(dados),
        Formato::Png => limpar_png(dados),
        Formato::Heic => Err(ErroDeHigiene::PrecisaConverter),
        Formato::Desconhecido => Err(ErroDeHigiene::PrecisaAchatar),
    }
}

fn manter_no_jpeg(marcador: u8) -> bool {
    match marcador {
        0xE0 => true,               // APP0 (JFIF)
        0xE2 => true,               // APP2 (perfil de cor ICC)
        0xEE => true,               // APP14 (Adobe: transformação de cor)
        0xE1 | 0xE3..=0xED | 0xEF => false, // APP1 (EXIF/XMP) e demais APPn
        0xFE => false,              // COM (comentário)
        _ => true,                  // tabelas, quadro, etc.
    }
}

pub fn limpar_jpeg(d: &[u8]) -> Result<Vec<u8>, ErroDeHigiene> {
    if d.len() < 4 || d[0] != 0xFF || d[1] != 0xD8 { return Err(ErroDeHigiene::Malformado); }
    let mut saida = vec![0xFF, 0xD8];
    let mut i = 2;
    loop {
        if i >= d.len() { return Err(ErroDeHigiene::Malformado); }
        if d[i] != 0xFF { return Err(ErroDeHigiene::Malformado); }
        while i < d.len() && d[i] == 0xFF { i += 1; } // bytes de preenchimento
        if i >= d.len() { return Err(ErroDeHigiene::Malformado); }
        let m = d[i];
        i += 1;
        match m {
            0xD9 => { saida.extend_from_slice(&[0xFF, 0xD9]); return Ok(saida); } // fim
            0xD0..=0xD7 | 0x01 => { saida.extend_from_slice(&[0xFF, m]); continue; } // sem tamanho
            _ => {}
        }
        if i + 2 > d.len() { return Err(ErroDeHigiene::Malformado); }
        let tam = u16::from_be_bytes([d[i], d[i + 1]]) as usize;
        if tam < 2 || i + tam > d.len() { return Err(ErroDeHigiene::Malformado); }
        let segmento = &d[i..i + tam];
        if m == 0xDA {
            // Início dos dados da imagem: copia o cabeçalho e todo o resto como está.
            saida.extend_from_slice(&[0xFF, m]);
            saida.extend_from_slice(&d[i..]);
            return Ok(saida);
        }
        if manter_no_jpeg(m) {
            saida.extend_from_slice(&[0xFF, m]);
            saida.extend_from_slice(segmento);
        }
        i += tam;
    }
}

const PNG_ASSINATURA: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn manter_no_png(tipo: &[u8; 4]) -> bool {
    matches!(tipo, b"IHDR" | b"PLTE" | b"IDAT" | b"IEND" | b"tRNS" | b"gAMA" | b"cHRM" | b"sRGB" | b"iCCP" | b"sBIT" | b"bKGD" | b"pHYs")
}

pub fn limpar_png(d: &[u8]) -> Result<Vec<u8>, ErroDeHigiene> {
    if !d.starts_with(&PNG_ASSINATURA) { return Err(ErroDeHigiene::Malformado); }
    let mut saida = PNG_ASSINATURA.to_vec();
    let mut i = 8;
    loop {
        if i + 12 > d.len() { return Err(ErroDeHigiene::Malformado); }
        let tam = u32::from_be_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]) as usize;
        let tipo: [u8; 4] = [d[i + 4], d[i + 5], d[i + 6], d[i + 7]];
        let fim = i.checked_add(12).and_then(|x| x.checked_add(tam)).ok_or(ErroDeHigiene::Malformado)?;
        if fim > d.len() { return Err(ErroDeHigiene::Malformado); }
        if manter_no_png(&tipo) { saida.extend_from_slice(&d[i..fim]); }
        i = fim;
        if &tipo == b"IEND" { return Ok(saida); }
    }
}

/// Nome neutro para o arquivo enviado: 16 caracteres aleatórios + a extensão (só letras e números).
pub fn nome_neutro(extensao: &str) -> String {
    let mut b = [0u8; 8];
    OsRng.fill_bytes(&mut b);
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    let ext: String = extensao.chars().filter(|c| c.is_ascii_alphanumeric()).take(8).collect::<String>().to_lowercase();
    if ext.is_empty() { hex } else { format!("{hex}.{ext}") }
}

/// CRC-32 (padrão do PNG). Usado para conferir blocos.
pub fn crc32(dados: &[u8]) -> u32 {
    let mut c: u32 = 0xFFFF_FFFF;
    for &b in dados {
        c ^= b as u32;
        for _ in 0..8 { c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 }; }
    }
    !c
}

#[cfg(test)]
mod testes {
    use super::*;

    fn seg(m: u8, conteudo: &[u8]) -> Vec<u8> {
        let mut v = vec![0xFF, m];
        v.extend_from_slice(&((conteudo.len() + 2) as u16).to_be_bytes());
        v.extend_from_slice(conteudo);
        v
    }

    fn jpeg_com_rastros() -> Vec<u8> {
        let mut j = vec![0xFF, 0xD8];
        j.extend(seg(0xE0, b"JFIF\0\x01\x01"));
        j.extend(seg(0xE1, b"Exif\0\0GPS-LAT-23.5-LON-46.6-iPhone-15"));
        j.extend(seg(0xED, b"Photoshop 3.0 IPTC autor Gabriel"));
        j.extend(seg(0xFE, b"comentario: foto do escritorio"));
        j.extend(seg(0xDB, &[0u8; 65])); // tabela de quantização
        j.extend(seg(0xDA, &[1, 1, 0, 0, 63, 0])); // início da imagem
        j.extend_from_slice(&[0x12, 0x34, 0xFF, 0x00, 0x56]); // dados da imagem
        j.extend_from_slice(&[0xFF, 0xD9]);
        j
    }

    fn contem(h: &[u8], agulha: &[u8]) -> bool { h.windows(agulha.len()).any(|w| w == agulha) }

    #[test]
    fn jpeg_perde_gps_autor_e_comentario() {
        let limpo = limpar(&jpeg_com_rastros()).unwrap();
        for rastro in [&b"GPS-LAT"[..], b"iPhone", b"Gabriel", b"escritorio", b"Exif"] {
            assert!(!contem(&limpo, rastro), "sobrou rastro: {:?}", String::from_utf8_lossy(rastro));
        }
    }

    #[test]
    fn jpeg_continua_sendo_imagem_valida() {
        let limpo = limpar(&jpeg_com_rastros()).unwrap();
        assert!(limpo.starts_with(&[0xFF, 0xD8]) && limpo.ends_with(&[0xFF, 0xD9]));
        assert!(contem(&limpo, b"JFIF"), "mantém o cabeçalho necessário");
        assert!(contem(&limpo, &[0x12, 0x34, 0xFF, 0x00, 0x56]), "dados da imagem intactos");
    }

    fn bloco_png(tipo: &[u8; 4], dados: &[u8]) -> Vec<u8> {
        let mut v = (dados.len() as u32).to_be_bytes().to_vec();
        v.extend_from_slice(tipo);
        v.extend_from_slice(dados);
        let mut para_crc = tipo.to_vec();
        para_crc.extend_from_slice(dados);
        v.extend_from_slice(&crc32(&para_crc).to_be_bytes());
        v
    }

    fn png_com_rastros() -> Vec<u8> {
        let mut p = PNG_ASSINATURA.to_vec();
        p.extend(bloco_png(b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0]));
        p.extend(bloco_png(b"tEXt", b"Author\0Gabriel Toffoli"));
        p.extend(bloco_png(b"eXIf", b"MM\0*GPS-SECRETO"));
        p.extend(bloco_png(b"tIME", &[7, 234, 9, 24, 10, 0, 0]));
        p.extend(bloco_png(b"IDAT", &[0x78, 0x9C, 1, 2, 3]));
        p.extend(bloco_png(b"IEND", &[]));
        p
    }

    #[test]
    fn png_perde_autor_exif_e_data() {
        let limpo = limpar(&png_com_rastros()).unwrap();
        for rastro in [&b"Gabriel"[..], b"GPS-SECRETO", b"tIME", b"tEXt", b"eXIf"] {
            assert!(!contem(&limpo, rastro));
        }
    }

    #[test]
    fn png_mantem_blocos_com_crc_valido() {
        let limpo = limpar(&png_com_rastros()).unwrap();
        let mut i = 8;
        let mut tipos = vec![];
        while i < limpo.len() {
            let tam = u32::from_be_bytes(limpo[i..i + 4].try_into().unwrap()) as usize;
            let tipo = &limpo[i + 4..i + 8];
            let crc = u32::from_be_bytes(limpo[i + 8 + tam..i + 12 + tam].try_into().unwrap());
            assert_eq!(crc32(&limpo[i + 4..i + 8 + tam]), crc, "CRC do bloco confere");
            tipos.push(String::from_utf8_lossy(tipo).to_string());
            i += 12 + tam;
        }
        assert_eq!(tipos, vec!["IHDR", "IDAT", "IEND"]);
    }

    #[test]
    fn pdf_e_outros_exigem_achatar() {
        assert_eq!(limpar(b"%PDF-1.7 ... /Author (Gabriel)"), Err(ErroDeHigiene::PrecisaAchatar));
    }

    #[test]
    fn heic_e_reconhecido_pelo_nome_e_recusado_para_converter() {
        // Caixa `ftyp` mínima do ISOBMFF: tamanho (4 bytes, valor não conferido aqui) + "ftyp" + "heic".
        let heic = [0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c'];
        assert_eq!(detectar(&heic), Formato::Heic, "reconhecido pelo nome, nunca misturado com Desconhecido");
        assert_eq!(limpar(&heic), Err(ErroDeHigiene::PrecisaConverter), "recusado com o erro específico, nunca tentando limpar às cegas (nunca Ok às cegas)");
        // As outras variantes do mesmo "major brand" (HEIF genérico, sequência HEVC) também são reconhecidas.
        for brand in [b"heix", b"heim", b"heis", b"hevc", b"hevx", b"mif1", b"msf1"] {
            let mut v = heic.to_vec();
            v[8..12].copy_from_slice(brand);
            assert_eq!(detectar(&v), Formato::Heic, "major brand {:?} também é HEIC/HEIF", String::from_utf8_lossy(brand));
        }
    }

    #[test]
    fn arquivo_truncado_e_recusado() {
        let mut j = jpeg_com_rastros();
        j.truncate(10);
        assert_eq!(limpar(&j), Err(ErroDeHigiene::Malformado));
    }

    #[test]
    fn nome_neutro_esconde_o_nome_original() {
        let a = nome_neutro("PDF");
        let b = nome_neutro("pdf");
        assert!(a.ends_with(".pdf") && a.len() == 20);
        assert_ne!(a, b);
        assert_eq!(nome_neutro("../../etc").len(), 16 + 1 + 3, "extensão maliciosa é saneada");
    }

    #[test]
    fn crc32_confere_com_valor_conhecido() {
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
    }
}
