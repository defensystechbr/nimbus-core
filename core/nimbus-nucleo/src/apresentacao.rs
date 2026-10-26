//! # Módulo `apresentacao` — como você aparece para cada contato (APR1–APR8)
//!
//! Você escolhe, contato por contato, o nome com que aparece (um apelido, uma inicial, um cargo) e se a pessoa
//! pode ou não trocar esse nome na agenda dela. O cartão é assinado por você e chega cifrado de ponta a ponta:
//! com o nome **travado**, o app dela mostra sempre o nome que você escolheu e não deixa editar. Quando você
//! muda o nome, a versão nova substitui a antiga no aparelho de quem recebeu — sem nenhum aviso na conversa.
//!
//! **Personas:** identidades separadas (ex.: Pessoal, Trabalho), cada uma com as próprias chaves. Quem conversa
//! com uma não consegue ligá-la à outra — não existe número, e-mail ou identificador em comum.
use crate::assinatura::{self, AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};

const CONTEXTO: &[u8] = b"NimbusOne/v1/apresentacao";
/// Nome com no máximo 64 bytes.
pub const TAM_MAXIMO_DO_NOME: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeApresentacao { NomeInvalido, NomeReservado, CaractereEnganoso }

/// Cartão de apresentação enviado a UM contato.
#[derive(Clone, Debug)]
pub struct Cartao {
    pub nome: String,
    /// Travado: quem recebe não consegue trocar o nome na agenda dele.
    pub nome_travado: bool,
    /// Versão crescente: a maior substitui as anteriores.
    pub versao: u64,
    /// Impressão digital da chave de quem se apresenta.
    pub autor: [u8; 32],
    pub assinatura: AssinaturaHibrida,
}

fn bytes_para_assinar(nome: &str, travado: bool, versao: u64) -> Vec<u8> {
    let mut v = (nome.len() as u16).to_be_bytes().to_vec();
    v.extend_from_slice(nome.as_bytes());
    v.push(travado as u8);
    v.extend_from_slice(&versao.to_be_bytes());
    v
}

/// Termos que ninguém pode usar como nome (EO3): o golpe mais barato é aparecer como "Suporte Nimbus",
/// "Segurança" ou "Administrador". A Nimbus não tem conta de atendimento dentro do app — nunca.
pub const TERMOS_RESERVADOS: [&str; 14] = ["nimbus", "suporte", "seguranca", "oficial", "administrador", "admin",
    "moderador", "verificado", "atendimento", "sistema", "equipe", "support", "security", "official"];

/// Forma de comparação: minúsculas, sem acento, sem espaço nem pontuação, e com as trocas visuais mais
/// usadas em golpe (0→o, 1→l, 3→e, 4→a, 5→s, 7→t, @→a, rn→m, vv→w).
/// Letras gregas e cirílicas que se parecem com letras latinas (maiúsculas e minúsculas) → a latina.
fn latina_parecida(c: char) -> char {
    match c {
        'Α' | 'А' | 'а' | 'α' => 'a', 'Β' | 'В' | 'в' => 'b', 'С' | 'с' | 'ϲ' => 'c', 'ԁ' => 'd', 'Ε' | 'Е' | 'е' | 'ε' => 'e',
        'Η' | 'Н' | 'н' => 'h', 'Ι' | 'І' | 'і' | 'ι' | 'Ӏ' => 'l', 'Ј' | 'ј' => 'j', 'Κ' | 'К' | 'к' | 'κ' => 'k',
        'Μ' | 'М' | 'м' => 'm', 'Ν' => 'n', 'Ο' | 'О' | 'о' | 'ο' => 'o', 'Ρ' | 'Р' | 'р' | 'ρ' => 'p', 'Ѕ' | 'ѕ' => 's',
        'Τ' | 'Т' | 'т' | 'τ' => 't', 'υ' => 'u', 'ν' => 'v', 'Χ' | 'Х' | 'х' | 'χ' => 'x', 'Υ' | 'У' | 'у' => 'y', 'Ζ' => 'z',
        c => c,
    }
}

/// Alfabeto de uma letra: 1 latino, 2 grego, 3 cirílico, 4 outro; 0 se não for letra.
fn escrita(c: char) -> u8 {
    if !c.is_alphabetic() { return 0; }
    match c as u32 {
        0x41..=0x5A | 0x61..=0x7A | 0xC0..=0x24F | 0x1E00..=0x1EFF => 1,
        0x370..=0x3FF | 0x1F00..=0x1FFF => 2,
        0x400..=0x52F => 3,
        _ => 4,
    }
}

/// Faixas de caracteres que imitam letras comuns: largura total, letras matemáticas, letras circuladas e
/// símbolos com forma de letra. Nunca entram em nome.
fn imitacao_de_letra(c: char) -> bool {
    matches!(c as u32, 0xFF00..=0xFFEF | 0x1D400..=0x1D7FF | 0x2460..=0x24FF | 0x1F100..=0x1F1FF | 0x2100..=0x214F)
}

pub fn forma_de_comparacao(nome: &str) -> String {
    let base: String = nome.chars().map(latina_parecida).flat_map(char::to_lowercase).filter_map(|c| {
        let c = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | '@' | '4' => 'a', 'é' | 'ê' | 'è' | 'ë' | '3' => 'e', 'í' | 'ì' | 'î' | 'ï' | '1' | '|' | '!' => 'l',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | '0' => 'o', 'ú' | 'ù' | 'û' | 'ü' => 'u', 'ç' => 'c', '5' | '$' => 's', '7' => 't',
            'i' => 'l', c => c,
        };
        c.is_alphanumeric().then_some(c)
    }).collect();
    base.replace("rn", "m").replace("vv", "w")
}

/// O nome pode ser usado? Recusa termos reservados (mesmo disfarçados) e caracteres que enganam a leitura.
pub fn nome_permitido(nome: &str) -> Result<(), ErroDeApresentacao> {
    if nome.trim().is_empty() || nome.len() > TAM_MAXIMO_DO_NOME || nome.chars().any(char::is_control) { return Err(ErroDeApresentacao::NomeInvalido); }
    if !crate::inspecao::texto_seguro_para_exibir(nome) { return Err(ErroDeApresentacao::CaractereEnganoso); }
    // Letras de imitação, ou mistura de alfabetos (latino com grego ou cirílico): forma clássica de se passar por outro.
    if nome.chars().any(imitacao_de_letra) { return Err(ErroDeApresentacao::CaractereEnganoso); }
    let escritas: Vec<u8> = nome.chars().map(escrita).filter(|e| *e != 0).collect();
    let tem = |e: u8| escritas.contains(&e);
    if (tem(1) && (tem(2) || tem(3))) || (tem(2) && tem(3)) { return Err(ErroDeApresentacao::CaractereEnganoso); }
    let f = forma_de_comparacao(nome);
    if TERMOS_RESERVADOS.iter().any(|t| f.contains(&forma_de_comparacao(t))) { return Err(ErroDeApresentacao::NomeReservado); }
    Ok(())
}

/// O nome se parece com o de outro contato (igual depois da normalização, ou a uma letra de distância)?
/// O app mostra o aviso "parecido com <contato>" e o nome que você salvou, para ninguém se passar por outro.
pub fn parecido_com<'a>(nome: &str, outros: &[&'a str]) -> Option<&'a str> {
    let f = forma_de_comparacao(nome);
    outros.iter().copied().find(|o| { let g = forma_de_comparacao(o); g == f || (f.chars().count() >= 4 && distancia_de_uma_letra(&f, &g)) })
}

fn distancia_de_uma_letra(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    match a.len() as i64 - b.len() as i64 {
        0 => a.iter().zip(&b).filter(|(x, y)| x != y).count() == 1,
        1 | -1 => {
            let (l, c) = if a.len() > b.len() { (&a, &b) } else { (&b, &a) };
            (0..l.len()).any(|i| { let mut t = l.clone(); let _ = t.remove(i); &t == c })
        }
        _ => false,
    }
}

/// Mudou o nome? Devolve (antigo, novo) para o app mostrar a faixa "X agora aparece como Y" — que não some
/// sozinha antes de o dono ver. Mudança de nome nunca é silenciosa (EO3).
pub fn mudanca_de_nome(atual: Option<&Cartao>, novo: &Cartao) -> Option<(String, String)> {
    match atual { Some(a) if a.nome != novo.nome && novo.versao > a.versao => Some((a.nome.clone(), novo.nome.clone())), _ => None }
}

/// No seu aparelho: cria o cartão para um contato.
pub fn emitir(chave: &ChaveDeAssinatura, nome: &str, nome_travado: bool, versao: u64) -> Result<Cartao, ErroDeApresentacao> {
    nome_permitido(nome)?;
    let assinatura = chave.assinar(CONTEXTO, &bytes_para_assinar(nome, nome_travado, versao)).expect("contexto fixo e curto");
    Ok(Cartao { nome: nome.to_string(), nome_travado, versao, autor: chave.verificacao().impressao_digital(), assinatura })
}

/// Confere se o cartão é mesmo desse contato e não foi alterado.
pub fn conferir(chave_do_contato: &ChaveDeVerificacao, c: &Cartao) -> bool {
    c.autor == chave_do_contato.impressao_digital() && nome_permitido(&c.nome).is_ok()
        && chave_do_contato.verificar(CONTEXTO, &bytes_para_assinar(&c.nome, c.nome_travado, c.versao), &c.assinatura)
}

/// No aparelho de quem recebe: guarda o cartão novo só se for válido e mais recente.
pub fn atualizar(chave_do_contato: &ChaveDeVerificacao, atual: Option<Cartao>, novo: Cartao) -> Option<Cartao> {
    if !conferir(chave_do_contato, &novo) { return atual; }
    match atual {
        Some(a) if a.versao >= novo.versao => Some(a),
        _ => Some(novo),
    }
}

/// Qual nome o app mostra para este contato. Com o nome travado, vale o nome que a pessoa escolheu (decisão de
/// produto: privacidade de quem se apresenta). Para isso não virar ferramenta de golpe (EO3): nomes
/// reservados e caracteres enganosos são recusados (`nome_permitido`), nome parecido com outro contato ganha
/// aviso (`parecido_com`), mudança de nome nunca é silenciosa (`mudanca_de_nome`) e o nome que VOCÊ salvou
/// aparece sempre embaixo (`subtitulo`).
pub fn nome_exibido<'a>(cartao: Option<&'a Cartao>, nome_que_salvei: Option<&'a str>) -> Option<&'a str> {
    match (cartao, nome_que_salvei) {
        (Some(c), _) if c.nome_travado => Some(c.nome.as_str()),
        (_, Some(s)) => Some(s),
        (Some(c), None) => Some(c.nome.as_str()),
        (None, None) => None,
    }
}

/// Linha de baixo, sempre visível quando o nome travado da pessoa difere do que você salvou: "salvo por você
/// como Mãe". O travado nunca apaga o seu rótulo.
pub fn subtitulo<'a>(cartao: Option<&'a Cartao>, nome_que_salvei: Option<&'a str>) -> Option<&'a str> {
    match (cartao, nome_que_salvei) {
        (Some(c), Some(s)) if c.nome_travado && c.nome != s => Some(s),
        _ => None,
    }
}

/// Posso trocar o nome deste contato na minha agenda? Com o nome travado, não — mas o seu rótulo continua
/// aparecendo embaixo e você pode bloquear o contato a qualquer momento.
pub fn pode_editar_nome(cartao: Option<&Cartao>) -> bool { !cartao.is_some_and(|c| c.nome_travado) }

/// Uma identidade separada, com chaves próprias.
pub struct Persona { pub apelido: String, chave: ChaveDeAssinatura }

impl core::fmt::Debug for Persona {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "Persona({}, <chave oculta>)", self.apelido) }
}

impl Persona {
    pub fn nova(apelido: &str) -> Self { Persona { apelido: apelido.to_string(), chave: assinatura::gerar() } }
    pub fn verificacao(&self) -> ChaveDeVerificacao { self.chave.verificacao() }
    pub fn cartao(&self, nome: &str, nome_travado: bool, versao: u64) -> Result<Cartao, ErroDeApresentacao> { emitir(&self.chave, nome, nome_travado, versao) }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn cartao_valido_e_adulterado() {
        let eu = assinatura::gerar();
        let c = emitir(&eu, "Dr. G", true, 1).unwrap();
        assert!(conferir(&eu.verificacao(), &c));
        let mut trocado = c.clone();
        trocado.nome = "Gabriel Toffoli".into();
        assert!(!conferir(&eu.verificacao(), &trocado), "ninguém troca o nome no caminho");
        let mut destravado = c;
        destravado.nome_travado = false;
        assert!(!conferir(&eu.verificacao(), &destravado));
    }

    #[test]
    fn nome_travado_nao_se_edita_e_prevalece_sobre_o_que_a_pessoa_digitou() {
        let eu = assinatura::gerar();
        let c = emitir(&eu, "G.", true, 1).unwrap();
        assert_eq!(nome_exibido(Some(&c), Some("Gabriel Mazon Toffoli")), Some("G."));
        assert!(!pode_editar_nome(Some(&c)));
    }

    #[test]
    fn nome_livre_sugere_mas_quem_recebe_pode_trocar() {
        let eu = assinatura::gerar();
        let c = emitir(&eu, "Gabriel", false, 1).unwrap();
        assert_eq!(nome_exibido(Some(&c), None), Some("Gabriel"), "o nome sugerido vem preenchido");
        assert_eq!(nome_exibido(Some(&c), Some("Primo")), Some("Primo"));
        assert!(pode_editar_nome(Some(&c)) && pode_editar_nome(None));
        assert_eq!(nome_exibido(None, None), None);
    }

    #[test]
    fn mudar_meu_nome_atualiza_em_quem_recebeu_e_versao_antiga_nao_volta() {
        let eu = assinatura::gerar();
        let v = eu.verificacao();
        let v1 = emitir(&eu, "Gabriel", false, 1).unwrap();
        let v2 = emitir(&eu, "G.", true, 2).unwrap();
        let atual = atualizar(&v, None, v1.clone());
        let atual = atualizar(&v, atual, v2).unwrap();
        assert_eq!((atual.nome.as_str(), atual.nome_travado), ("G.", true));
        let depois = atualizar(&v, Some(atual), v1).unwrap();
        assert_eq!(depois.nome, "G.", "reenviar a versão antiga não desfaz a mudança");
    }

    #[test]
    fn cartao_de_outra_pessoa_nao_entra() {
        let eu = assinatura::gerar();
        let intruso = assinatura::gerar();
        let falso = emitir(&intruso, "Banco", true, 99).unwrap();
        assert!(atualizar(&eu.verificacao(), None, falso).is_none());
    }

    #[test]
    fn nome_vazio_ou_longo_e_recusado() {
        let eu = assinatura::gerar();
        assert_eq!(emitir(&eu, "   ", false, 1).unwrap_err(), ErroDeApresentacao::NomeInvalido);
        assert_eq!(emitir(&eu, &"x".repeat(TAM_MAXIMO_DO_NOME + 1), false, 1).unwrap_err(), ErroDeApresentacao::NomeInvalido);
    }

    #[test]
    fn personas_nao_se_ligam_uma_a_outra() {
        let trabalho = Persona::nova("Trabalho");
        let pessoal = Persona::nova("Pessoal");
        assert_ne!(trabalho.verificacao().impressao_digital(), pessoal.verificacao().impressao_digital());
        let c = trabalho.cartao("Consultor", true, 1).unwrap();
        assert!(conferir(&trabalho.verificacao(), &c));
        assert!(!conferir(&pessoal.verificacao(), &c), "o cartão de uma persona não serve para a outra");
        assert!(format!("{trabalho:?}").contains("oculta"));
    }

    #[test]
    fn nome_nao_vira_ferramenta_de_golpe() {
        for ruim in ["Suporte Nimbus", "SEGURANÇA", "N1mbus", "Equipe de atendimento", "admin", "Ofícial", "nimbus\u{202E}moc", "a\u{0007}b"] {
            assert!(nome_permitido(ruim).is_err(), "{ruim}");
        }
        for bom in ["G.", "Gabriel", "Consultor", "Mãe", "Dr. Almeida"] { assert!(nome_permitido(bom).is_ok(), "{bom}"); }
        let eu = assinatura::gerar();
        assert!(emitir(&eu, "Suporte Nimbus", true, 1).is_err(), "não dá para emitir cartão com nome reservado");
        assert_eq!(nome_permitido("Ana\u{202E}lim"), Err(ErroDeApresentacao::CaractereEnganoso));
        // Cartão forjado por um app adulterado, com nome reservado e assinatura válida: recusado na chegada.
        let forjado = Cartao { nome: "Suporte Nimbus".into(), nome_travado: true, versao: 9, autor: eu.verificacao().impressao_digital(),
                               assinatura: eu.assinar(CONTEXTO, &bytes_para_assinar("Suporte Nimbus", true, 9)).unwrap() };
        assert!(!conferir(&eu.verificacao(), &forjado));
        let atual = emitir(&eu, "G.", true, 1).unwrap();
        assert_eq!(atualizar(&eu.verificacao(), Some(atual), forjado).map(|c| c.nome), Some("G.".to_string()), "fica o nome anterior");
        // Parecido com outro contato: aviso.
        assert_eq!(parecido_com("Rnae", &["Mae", "Renata"]), Some("Mae"), "rn parece m");
        assert_eq!(parecido_com("Renatta", &["Mae", "Renata"]), Some("Renata"));
        assert_eq!(parecido_com("R3nata", &["Renata"]), Some("Renata"));
        assert_eq!(parecido_com("Dr Almeida", &["Dr. Almeida"]), Some("Dr. Almeida"));
        assert_eq!(parecido_com("Carla", &["Renata"]), None);
        // O rótulo que você salvou nunca some.
        let c = emitir(&eu, "G.", true, 1).unwrap();
        assert_eq!(subtitulo(Some(&c), Some("Mãe")), Some("Mãe"));
        assert_eq!(subtitulo(Some(&c), None), None);
        // Mudança de nome nunca é silenciosa.
        let c2 = emitir(&eu, "Gabriel", true, 2).unwrap();
        assert_eq!(mudanca_de_nome(Some(&c), &c2), Some(("G.".to_string(), "Gabriel".to_string())));
        assert_eq!(mudanca_de_nome(Some(&c2), &c), None, "versão antiga não conta");
        assert_eq!(mudanca_de_nome(None, &c), None);
    }


    #[test]
    fn letras_de_outros_alfabetos_e_imitacoes_nao_enganam() {
        for ruim in ["Suporte N\u{0456}mbus", "\u{039D}imbus", "\u{FF2E}\u{FF49}\u{FF4D}\u{FF42}\u{FF55}\u{FF53}",
                     "\u{1D40D}\u{1D422}\u{1D426}\u{1D41B}\u{1D42E}\u{1D42C}", "\u{24C3}imbus", "Nim\u{200B}bus", "R\u{0435}nata"] {
            assert!(nome_permitido(ruim).is_err(), "{ruim}");
        }
        // Nome inteiro num só alfabeto continua permitido (pessoas reais têm nomes gregos e cirílicos).
        assert!(nome_permitido("\u{0410}\u{043D}\u{043D}\u{0430}").is_ok(), "Анна");
        assert!(nome_permitido("\u{0394}\u{03B7}\u{03BC}\u{03AE}\u{03C4}\u{03C1}\u{03B7}\u{03C2}").is_ok(), "Δημήτρης");
        assert!(nome_permitido("José Ângelo").is_ok());
        // Letras gregas e cirílicas parecidas com latinas são comparadas como latinas.
        assert_eq!(forma_de_comparacao("\u{0410}\u{0412}\u{0421}"), "abc");
        assert_eq!(parecido_com("\u{0420}\u{0435}\u{043D}\u{0430}\u{0442}\u{0430}", &["Renata"]), None, "Ренатa soa diferente: Р→p");
        assert_eq!(parecido_com("R\u{0435}nata", &["Renata"]), Some("Renata"));
        assert_eq!(forma_de_comparacao("\u{039D}\u{0399}\u{039C}\u{0392}\u{03A5}S"), "nlmbys");
    }

}
