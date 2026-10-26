//! # Módulo `inspecao` — conferir todo arquivo antes de abrir ou importar (INS1–INS9)
//!
//! Não é um antivírus de assinaturas (que só reconhece ataques já catalogados): é mais forte contra ataques
//! novos. Descobre o tipo REAL pelos primeiros bytes — nunca pelo nome —, recusa programas, instaladores,
//! scripts e documentos com macro, e manda "achatar" (mostrar como imagem, num processo isolado) o que pode
//! conter código, como PDF e documentos do Office. Arquivo corrompido nem chega aqui: a cifra recusa qualquer
//! byte alterado. Vale para o que chega pela conversa, pelo envio direto e pelo que se importa do celular.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoReal { Jpeg, Png, Gif, Webp, Heic, Pdf, Video, Audio, Office, Compactado, Texto, Programa, InstaladorAndroid, Java, Script, Desconhecido }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motivo { Programa, Macro, ExtensaoFalsa, Vazio }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decisao {
    /// Abre, decodificado num processo isolado e sem metadados.
    Abrir,
    /// Mostra só como imagem das páginas: o código do documento nunca roda.
    AbrirComoImagem,
    /// Não abre no Nimbus; pode ser guardado cifrado.
    GuardarSemAbrir,
    Bloquear(Motivo),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Laudo { pub tipo_real: TipoReal, pub decisao: Decisao }

fn contem(d: &[u8], agulha: &[u8]) -> bool { d.windows(agulha.len()).any(|w| w == agulha) }

/// Tipo real, pelos primeiros bytes.
pub fn tipo_real(d: &[u8]) -> TipoReal {
    let c = |p: &[u8]| d.starts_with(p);
    if c(b"MZ") || c(&[0x7F, b'E', b'L', b'F']) || c(&[0xFE, 0xED, 0xFA, 0xCE]) || c(&[0xFE, 0xED, 0xFA, 0xCF])
        || c(&[0xCE, 0xFA, 0xED, 0xFE]) || c(&[0xCF, 0xFA, 0xED, 0xFE]) { return TipoReal::Programa; }
    if c(&[0xCA, 0xFE, 0xBA, 0xBE]) { return TipoReal::Java; }
    if c(b"#!") { return TipoReal::Script; }
    if c(&[0xFF, 0xD8, 0xFF]) { return TipoReal::Jpeg; }
    if c(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) { return TipoReal::Png; }
    if c(b"GIF87a") || c(b"GIF89a") { return TipoReal::Gif; }
    if c(b"RIFF") && d.len() >= 12 && &d[8..12] == b"WEBP" { return TipoReal::Webp; }
    if c(b"%PDF-") { return TipoReal::Pdf; }
    if d.len() >= 12 && &d[4..8] == b"ftyp" {
        let marca = &d[8..12];
        return if marca == b"heic" || marca == b"heix" || marca == b"mif1" { TipoReal::Heic } else { TipoReal::Video };
    }
    if c(b"ID3") || c(&[0xFF, 0xFB]) || c(b"OggS") { return TipoReal::Audio; }
    if c(&[b'P', b'K', 0x03, 0x04]) {
        if contem(d, b"AndroidManifest.xml") || contem(d, b"classes.dex") { return TipoReal::InstaladorAndroid; }
        if contem(d, b"[Content_Types].xml") { return TipoReal::Office; }
        if contem(d, b"META-INF/MANIFEST.MF") { return TipoReal::Java; }
        return TipoReal::Compactado;
    }
    if !d.is_empty() && !d.contains(&0) && core::str::from_utf8(d).is_ok() { return TipoReal::Texto; }
    TipoReal::Desconhecido
}

fn familia_da_extensao(nome: &str) -> Option<&'static [TipoReal]> {
    let ext = nome.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())?;
    Some(match ext.as_str() {
        "jpg" | "jpeg" => &[TipoReal::Jpeg],
        "png" => &[TipoReal::Png],
        "gif" => &[TipoReal::Gif],
        "webp" => &[TipoReal::Webp],
        "heic" | "heif" => &[TipoReal::Heic],
        "pdf" => &[TipoReal::Pdf],
        "mp4" | "mov" | "m4v" => &[TipoReal::Video],
        "mp3" | "ogg" | "m4a" => &[TipoReal::Audio, TipoReal::Video],
        "docx" | "xlsx" | "pptx" => &[TipoReal::Office],
        "zip" => &[TipoReal::Compactado],
        "txt" | "csv" | "md" => &[TipoReal::Texto],
        _ => return None,
    })
}

/// Laudo de um arquivo: o que o app faz com ele.
pub fn inspecionar(nome: &str, dados: &[u8]) -> Laudo {
    if dados.is_empty() { return Laudo { tipo_real: TipoReal::Desconhecido, decisao: Decisao::Bloquear(Motivo::Vazio) }; }
    let t = tipo_real(dados);
    let decisao = match t {
        TipoReal::Programa | TipoReal::InstaladorAndroid | TipoReal::Java | TipoReal::Script => Decisao::Bloquear(Motivo::Programa),
        TipoReal::Office if contem(dados, b"vbaProject.bin") => Decisao::Bloquear(Motivo::Macro),
        _ if familia_da_extensao(nome).is_some_and(|f| !f.contains(&t)) => Decisao::Bloquear(Motivo::ExtensaoFalsa),
        TipoReal::Pdf | TipoReal::Office => Decisao::AbrirComoImagem,
        TipoReal::Compactado | TipoReal::Desconhecido => Decisao::GuardarSemAbrir,
        _ => Decisao::Abrir,
    };
    Laudo { tipo_real: t, decisao }
}

// ---- Saneamento de texto e links contra caracteres enganosos (fecha o CVE-2022-28345 do Signal) ----

/// Pontos de código Unicode que reordenam ou escondem texto na tela, usados para forjar links e nomes
/// (RTLO e amigos). Fonte: Unicode UAX #9 (bidirecional) e UTS #39 (segurança). A lista cobre as marcas
/// bidirecionais e os caracteres invisíveis mais explorados.
pub fn e_caractere_enganoso(c: char) -> bool {
    matches!(c as u32,
        0x202A..=0x202E |  // LRE, RLE, PDF, LRO, RLO — sobrescrita de direção (o RTLO clássico)
        0x2066..=0x2069 |  // LRI, RLI, FSI, PDI — isolamento de direção
        0x200E | 0x200F |  // LRM, RLM — marcas de direção
        0x200B..=0x200D |  // espaços de largura zero e juntores
        0x2060 | 0xFEFF |  // juntor de largura zero e BOM/espaço sem quebra de largura zero
        0x115F | 0x1160 |  // preenchedores Hangul (parecem espaço)
        0x3164 | 0xFFA0    // preenchedores Hangul compatíveis
    )
}

/// Remove caracteres enganosos de um texto. Devolve o texto limpo e quantos foram removidos, para a tela
/// poder avisar "este texto continha caracteres ocultos". Nunca deve ser aplicado ao conteúdo cifrado —
/// só ao que vai ser MOSTRADO (prévia de link, nome exibido, legenda).
pub fn sanear_texto_exibido(texto: &str) -> (String, usize) {
    let limpo: String = texto.chars().filter(|c| !e_caractere_enganoso(*c)).collect();
    let removidos = texto.chars().count() - limpo.chars().count();
    (limpo, removidos)
}

/// Um texto é seguro para exibir sem aviso quando não tem nenhum caractere enganoso.
pub fn texto_seguro_para_exibir(texto: &str) -> bool {
    !texto.chars().any(e_caractere_enganoso)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn zip_com(nome: &[u8]) -> Vec<u8> { let mut v = b"PK\x03\x04".to_vec(); v.extend_from_slice(&[0; 26]); v.extend_from_slice(nome); v }

    #[test]
    fn programas_e_instaladores_sao_bloqueados_mesmo_disfarcados() {
        for (nome, d) in [("foto.jpg", b"MZ\x90\x00".to_vec()), ("doc.pdf", b"\x7fELF\x02".to_vec()), ("app.apk", zip_com(b"classes.dex")),
                          ("x.jar", zip_com(b"META-INF/MANIFEST.MF")), ("run.txt", b"#!/bin/sh\nrm".to_vec()), ("a.class", vec![0xCA, 0xFE, 0xBA, 0xBE])] {
            assert_eq!(inspecionar(nome, &d).decisao, Decisao::Bloquear(Motivo::Programa), "{nome}");
        }
    }

    #[test]
    fn office_com_macro_e_bloqueado_e_sem_macro_vira_imagem() {
        let mut com = zip_com(b"[Content_Types].xml");
        com.extend_from_slice(b"word/vbaProject.bin");
        assert_eq!(inspecionar("planilha.docx", &com).decisao, Decisao::Bloquear(Motivo::Macro));
        assert_eq!(inspecionar("contrato.docx", &zip_com(b"[Content_Types].xml")).decisao, Decisao::AbrirComoImagem);
    }

    #[test]
    fn extensao_mentirosa_e_bloqueada() {
        assert_eq!(inspecionar("contrato.pdf", &zip_com(b"arquivo.txt")).decisao, Decisao::Bloquear(Motivo::ExtensaoFalsa));
        assert_eq!(inspecionar("foto.png", &[0xFF, 0xD8, 0xFF, 0xE0]).decisao, Decisao::Bloquear(Motivo::ExtensaoFalsa));
    }

    #[test]
    fn fotos_videos_e_texto_abrem_pdf_vira_imagem_e_compactado_so_se_guarda() {
        assert_eq!(inspecionar("f.jpg", &[0xFF, 0xD8, 0xFF, 0xE0, 1]).decisao, Decisao::Abrir);
        assert_eq!(inspecionar("f.png", &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]).decisao, Decisao::Abrir);
        let mut mp4 = vec![0, 0, 0, 0x18];
        mp4.extend_from_slice(b"ftypisom");
        assert_eq!(inspecionar("v.mp4", &mp4).tipo_real, TipoReal::Video);
        assert_eq!(inspecionar("nota.txt", "olá".as_bytes()).decisao, Decisao::Abrir);
        assert_eq!(inspecionar("c.pdf", b"%PDF-1.7 /JavaScript").decisao, Decisao::AbrirComoImagem, "o código do PDF nunca roda");
        assert_eq!(inspecionar("pacote.zip", &zip_com(b"a.txt")).decisao, Decisao::GuardarSemAbrir);
        assert_eq!(inspecionar("sem-nome", &[0, 1, 2, 3]).decisao, Decisao::GuardarSemAbrir);
    }

    /// ATK-A7 — HEIC/HEIF real nunca pode ser classificado como Vídeo (as duas trilhas têm
    /// tratamento de limpeza diferente; confundir uma com a outra é exatamente o tipo de furo que
    /// este módulo existe para fechar). Cobre as três marcas de HEIC aceitas pelo formato
    /// (ISO/IEC 23008-12): "heic" (HEIC puro), "heix" (variante de extensão) e "mif1" (MIAF/
    /// contêiner genérico, usado por iPhones mais novos), e confere que uma marca de vídeo
    /// qualquer continua caindo em Vídeo.
    fn ftyp(marca: &[u8; 4]) -> Vec<u8> {
        let mut v = vec![0, 0, 0, 0x18];
        v.extend_from_slice(b"ftyp");
        v.extend_from_slice(marca);
        v
    }

    #[test]
    fn heic_real_nunca_vira_video() {
        for marca in [b"heic", b"heix", b"mif1"] {
            let laudo = inspecionar("foto.heic", &ftyp(marca));
            assert_eq!(laudo.tipo_real, TipoReal::Heic, "marca {:?} tem de ser HEIC", String::from_utf8_lossy(marca));
            assert_ne!(laudo.tipo_real, TipoReal::Video, "marca {:?} nunca pode virar Vídeo", String::from_utf8_lossy(marca));
        }
        // Controle: uma marca de vídeo de verdade (isom) continua sendo Vídeo, não HEIC.
        assert_eq!(inspecionar("v.mp4", &ftyp(b"isom")).tipo_real, TipoReal::Video);
    }

    #[test]
    fn texto_com_reordenacao_de_direita_para_esquerda_e_saneado() {
        let ataque = "photo_\u{202E}gpj.exe"; // RTLO disfarça photo_gpj.exe de photo_exe.jpg
        assert!(!texto_seguro_para_exibir(ataque));
        let (limpo, n) = sanear_texto_exibido(ataque);
        assert_eq!(n, 1);
        assert_eq!(limpo, "photo_gpj.exe", "sem o RTLO, o nome real aparece");
    }

    #[test]
    fn espacos_de_largura_zero_e_marcas_de_direcao_saem() {
        let ataque = "ex\u{200B}em\u{200E}plo.com\u{FEFF}";
        let (limpo, n) = sanear_texto_exibido(ataque);
        assert_eq!(limpo, "exemplo.com");
        assert_eq!(n, 3);
    }

    #[test]
    fn texto_normal_passa_intacto() {
        let ok = "Reunião amanhã às 9h em https://exemplo.com/sala";
        assert!(texto_seguro_para_exibir(ok));
        assert_eq!(sanear_texto_exibido(ok), (ok.to_string(), 0));
    }

    #[test]
    fn arquivo_vazio_e_recusado() {
        assert_eq!(inspecionar("x.jpg", &[]).decisao, Decisao::Bloquear(Motivo::Vazio));
    }
}
