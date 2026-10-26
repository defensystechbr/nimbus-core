//! # Módulo `autenticidade` — nunca ser imitado (AUT1–AUT5)
//!
//! Lição do caso Privnote (2020): um site quase idêntico, num endereço com uma letra a mais, lia as notas e
//! trocava endereços de bitcoin; como a nota some depois de lida, a vítima não tinha como conferir. A cifra do
//! original não ajudou porque a vítima nunca chegou ao original. Defesas deste módulo:
//!
//! * **AUT1** — o app confere, em tempo constante, que o certificado que o assinou é um dos fixados no código.
//!   Assinatura diferente = cópia adulterada: as chaves não são abertas.
//! * **AUT2** — todo endereço de site que chega numa mensagem é extraído com limites (tamanho, quantidade).
//! * **AUT3** — endereço parecido com um oficial (uma letra trocada, caractere de outro alfabeto, "rn" no lugar
//!   de "m", o oficial escondido dentro de outro domínio, codificação "xn--", usuário falso antes de "@") é
//!   marcado como imitação e o app avisa antes de abrir.
//! * **AUT4** — a lista de domínios oficiais vem da configuração assinada do app, nunca da rede.
//! * **AUT5** — nunca existe versão web que baixe o código a cada acesso (decisão de produto; o servidor
//!   poderia entregar outro código). Registrado aqui para a catraca e o Manual.
use subtle::ConstantTimeEq;

/// Máximo de texto examinado por mensagem e máximo de endereços extraídos (AUT2).
pub const MAXIMO_DE_TEXTO: usize = 64 * 1024;
pub const MAXIMO_DE_ENDERECOS: usize = 100;
/// Tamanho máximo de um nome de domínio (RFC 1035).
pub const MAXIMO_DO_DOMINIO: usize = 253;

/// AUT1: compara o resumo do certificado de assinatura do app com cada um dos fixados, sem atalho de tempo.
pub fn assinatura_do_app_confere(resumo: &[u8; 32], fixados: &[[u8; 32]]) -> bool {
    let mut algum = subtle::Choice::from(0u8);
    for f in fixados { algum |= resumo.ct_eq(f); }
    bool::from(algum) && !fixados.is_empty()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motivo {
    /// Parecido com um oficial sem ser ele (letra trocada, alfabeto misturado, "rn" por "m").
    Parecido,
    /// O nome oficial aparece dentro de outro domínio (oficial.com.golpe.io, oficial-login.com).
    OficialDisfarcado,
    /// Codificação "xn--": letras de outro alfabeto escondidas.
    CodificacaoEstrangeira,
    /// Usuário falso antes de "@" (https://oficial.com@golpe.io).
    UsuarioFalso,
}

/// Troca caracteres que enganam o olho pela letra que imitam (só para comparar; nunca para abrir).
fn desfazer_disfarce(s: &str) -> String {
    let mut t = String::with_capacity(s.len());
    for c in s.chars().flat_map(char::to_lowercase) {
        t.push(match c {
            '0' => 'o', '1' | 'ӏ' => 'l', 'ı' | 'і' | 'ɩ' => 'i', '3' => 'e', '5' => 's', '7' => 't', '@' => 'a',
            'а' => 'a', 'е' => 'e', 'о' => 'o', 'р' => 'p', 'с' => 'c', 'у' => 'y', 'х' => 'x', 'ј' => 'j',
            'ѕ' => 's', 'ԁ' => 'd', 'ɡ' => 'g', 'ո' => 'n', 'ν' => 'v', 'ο' => 'o', 'α' => 'a',
            outro => outro,
        });
    }
    t.replace("rn", "m").replace("vv", "w")
}

/// Distância de edição com teto (para cedo quando passa do teto).
fn distancia(a: &str, b: &str, teto: usize) -> usize {
    let a: Vec<char> = a.chars().collect(); let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > teto { return teto + 1; }
    let mut ant: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut atual = vec![i + 1; b.len() + 1];
        let mut menor = atual[0];
        for (j, cb) in b.iter().enumerate() {
            atual[j + 1] = (ant[j] + usize::from(ca != cb)).min(ant[j + 1] + 1).min(atual[j] + 1);
            menor = menor.min(atual[j + 1]);
        }
        if menor > teto { return teto + 1; }
        ant = atual;
    }
    ant[b.len()]
}

/// Parte registrável aproximada: os dois últimos rótulos ("golpe.io" em "a.b.golpe.io").
fn base(dominio: &str) -> &str {
    let mut pontos = dominio.rmatch_indices('.');
    match (pontos.next(), pontos.next()) { (Some(_), Some((i, _))) => &dominio[i + 1..], _ => dominio }
}

/// AUT3: diz se `dominio` imita algum dos `oficiais` (que devem estar em minúsculas, sem esquema).
pub fn imitacao(dominio: &str, oficiais: &[&str]) -> Option<Motivo> {
    let d = dominio.trim_end_matches('.').to_lowercase();
    if d.is_empty() || d.len() > MAXIMO_DO_DOMINIO { return None; }
    for o in oficiais {
        if d == *o || d.ends_with(&format!(".{o}")) { return None; } // o próprio ou um subdomínio dele
    }
    if d.split('.').any(|r| r.starts_with("xn--")) { return Some(Motivo::CodificacaoEstrangeira); }
    let limpo = desfazer_disfarce(&d);
    let base_limpa = base(&limpo).to_string();
    for o in oficiais {
        // O oficial inteiro dentro de outro domínio: oficial.com.golpe.io
        if limpo.contains(&format!("{o}.")) && base_limpa != *o { return Some(Motivo::OficialDisfarcado); }
        // Igual ao oficial depois de desfazer o disfarce, ou a até 2 letras dele: privn0te.com, privnotes.com
        if base_limpa == *o || distancia(&base_limpa, o, 2) <= 2 { return Some(Motivo::Parecido); }
        // A marca (primeiro pedaço do nome oficial, com 4 letras ou mais) usada em outro domínio: marca-login.com
        let marca = o.split(['.', '-']).next().unwrap_or(o);
        let rotulo = base_limpa.split('.').next().unwrap_or("");
        if marca.chars().count() >= 4 && rotulo.split('-').any(|p| p == marca) { return Some(Motivo::OficialDisfarcado); }
    }
    None
}

/// AUT2: extrai domínios de um texto de mensagem, com limites. Devolve (domínio, havia usuário antes de "@").
pub fn extrair_dominios(texto: &str) -> Vec<(String, bool)> {
    let mut fim = texto.len().min(MAXIMO_DE_TEXTO);
    while !texto.is_char_boundary(fim) { fim -= 1; }
    let mut achados = vec![];
    for palavra in texto[..fim].split(|c: char| c.is_whitespace() || "<>()[]{}\"'`,;".contains(c)) {
        if achados.len() >= MAXIMO_DE_ENDERECOS { break; }
        let sem_esquema = palavra.strip_prefix("https://").or_else(|| palavra.strip_prefix("http://")).unwrap_or(palavra);
        let autoridade = sem_esquema.split(['/', '?', '#']).next().unwrap_or("");
        let (usuario, hospedeiro) = match autoridade.rsplit_once('@') { Some((_, h)) => (true, h), None => (false, autoridade) };
        let hospedeiro = hospedeiro.split(':').next().unwrap_or("").trim_end_matches(['.', '!', '?']);
        let parece_dominio = hospedeiro.contains('.') && hospedeiro.len() <= MAXIMO_DO_DOMINIO
            && hospedeiro.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-')
            && hospedeiro.split('.').all(|r| !r.is_empty() && r.len() <= 63)
            && hospedeiro.rsplit('.').next().is_some_and(|tld| tld.chars().any(|c| c.is_alphabetic()));
        if parece_dominio && (usuario || palavra.starts_with("http") || hospedeiro.matches('.').count() >= 1) {
            achados.push((hospedeiro.to_lowercase(), usuario));
        }
    }
    achados
}

/// AUT2 + AUT3: todos os endereços suspeitos de uma mensagem, com o motivo. Usuário falso antes de "@" é
/// sempre suspeito, mesmo apontando para domínio desconhecido.
pub fn avaliar_mensagem(texto: &str, oficiais: &[&str]) -> Vec<(String, Motivo)> {
    let mut suspeitos = vec![];
    for (d, usuario) in extrair_dominios(texto) {
        if usuario { suspeitos.push((d, Motivo::UsuarioFalso)); continue; }
        if let Some(m) = imitacao(&d, oficiais) { suspeitos.push((d, m)); }
    }
    suspeitos
}

#[cfg(test)]
mod testes {
    use super::*;
    // Domínios de teste reservados (RFC 2606); os oficiais de verdade vêm da configuração assinada (AUT4).
    const OFICIAIS: &[&str] = &["nimbus-teste.example"];
    const CURTO: &[&str] = &["privnote.com"];

    #[test]
    fn assinatura_do_app_so_confere_com_um_fixado() {
        let a = [7u8; 32]; let b = [8u8; 32];
        assert!(assinatura_do_app_confere(&a, &[b, a]));
        assert!(!assinatura_do_app_confere(&a, &[b]));
        assert!(!assinatura_do_app_confere(&a, &[]), "lista vazia nunca aprova");
        let mut quase = a; quase[31] ^= 1;
        assert!(!assinatura_do_app_confere(&quase, &[a]), "um bit diferente reprova");
    }

    #[test]
    fn o_caso_privnote_e_pego() {
        assert_eq!(imitacao("privnotes.com", CURTO), Some(Motivo::Parecido));
        assert_eq!(imitacao("privn0te.com", CURTO), Some(Motivo::Parecido));
        assert_eq!(imitacao("prіvnote.com", CURTO), Some(Motivo::Parecido), "i cirílico");
        assert_eq!(imitacao("privnote.com", CURTO), None);
        assert_eq!(imitacao("blog.privnote.com", CURTO), None, "subdomínio verdadeiro");
        // Três letras "а" do alfabeto cirílico: sem desfazer o disfarce, a distância seria 3 e passaria.
        assert_eq!(imitacao("b\u{430}n\u{430}n\u{430}-teste.example", &["banana-teste.example"]), Some(Motivo::Parecido));
    }

    #[test]
    fn disfarces_classicos_sao_pegos() {
        assert_eq!(imitacao("nimbus-teste.example.golpe.io", OFICIAIS), Some(Motivo::OficialDisfarcado));
        assert_eq!(imitacao("nimbus-login.example", OFICIAIS), Some(Motivo::OficialDisfarcado));
        assert_eq!(imitacao("xn--nimbus-teste-xyz.example", OFICIAIS), Some(Motivo::CodificacaoEstrangeira));
        assert_eq!(imitacao("nirnbus-teste.example", OFICIAIS), Some(Motivo::Parecido), "rn no lugar de m");
        assert_eq!(imitacao("nimbus-teste.exarnple", OFICIAIS), Some(Motivo::Parecido));
        assert_eq!(imitacao("wikipedia.org", OFICIAIS), None, "domínio sem relação não acusa");
        assert_eq!(imitacao("", OFICIAIS), None);
        assert_eq!(imitacao(&"a".repeat(300), OFICIAIS), None, "longo demais é ignorado sem pânico");
    }

    #[test]
    fn mensagem_com_usuario_falso_e_endereco_imitador() {
        let m = "pague aqui https://nimbus-teste.example@golpe.io/pix ou veja nimbus-teste.exarnple/login, ou o certo https://nimbus-teste.example/ajuda";
        let s = avaliar_mensagem(m, OFICIAIS);
        assert!(s.contains(&("golpe.io".to_string(), Motivo::UsuarioFalso)), "{s:?}");
        assert!(s.contains(&("nimbus-teste.exarnple".to_string(), Motivo::Parecido)), "{s:?}");
        assert_eq!(s.len(), 2, "o endereço oficial não é acusado: {s:?}");
    }

    #[test]
    fn extracao_tem_limites() {
        let muitos = "a.com ".repeat(1000);
        assert_eq!(extrair_dominios(&muitos).len(), MAXIMO_DE_ENDERECOS);
        let grande = format!("{}x.com", "é".repeat(MAXIMO_DE_TEXTO));
        assert!(extrair_dominios(&grande).is_empty(), "texto além do limite não é examinado e não quebra no meio de um caractere");
        assert!(extrair_dominios("versão 1.2 e 3.4").is_empty(), "número com ponto não é domínio");
    }

    #[test]
    fn distancia_respeita_o_teto() {
        assert_eq!(distancia("abc", "abc", 2), 0);
        assert_eq!(distancia("abc", "abd", 2), 1);
        assert_eq!(distancia("abc", "xyzw", 2), 3);
        assert_eq!(distancia("a", "abcdef", 2), 3);
    }
}
