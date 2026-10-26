//! # Módulo `contatos` — o nome que você salva é segredo (CTT1–CTT8)
//!
//! Um contato guarda: o **nome que você deu** (“Mãe”, “Dr. Almeida”), as chaves públicas da pessoa, o
//! número de segurança, notas e etiquetas. Tudo isso vira **um único bloco cifrado de tamanho fixo**
//! para a chave do dono — o servidor não sabe quantos contatos você tem, como se chamam, nem se o nome
//! é curto ou longo. Só aparece decifrado na memória do seu aparelho, na hora de mostrar.
//!
//! O **nome que a própria pessoa escolheu** (perfil) também é cifrado: com uma chave de perfil que ela
//! entrega só a quem ela aceitou como contato (modelo de perfis cifrados do Signal).
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use chacha20poly1305::{aead::{Aead, KeyInit, Payload}, XChaCha20Poly1305, XNonce};
use rand_core::{OsRng, RngCore};
use zeroize::Zeroizing;

const CONTEXTO_CONTATO: &[u8] = b"NimbusOne/v1/contato";
const CONTEXTO_PERFIL: &[u8] = b"NimbusOne/v1/perfil";
/// Espaço fixo para os campos de texto do contato: nomes e notas curtos ou longos ocupam o mesmo tamanho.
pub const TAM_FIXO_DO_CONTATO: usize = 2048;
/// Tamanho fixo do nome de perfil cifrado.
pub const TAM_FIXO_DO_PERFIL: usize = 128;

/// CTT9 — contato nunca some sozinho sem aviso. Depois de 6 meses sem nenhuma troca (mensagem, chamada,
/// abertura da conversa), a tela avisa que vai ser removido para não acumular agenda; só é removido se
/// NADA mudar por mais 30 dias depois do aviso — tempo de sobra para o dono notar e decidir. Em qualquer
/// momento, antes ou depois do aviso, o dono pode apagar manualmente: isto nunca é bloqueado por nada aqui.
pub const AVISO_DE_CONTATO_INATIVO_SEGUNDOS: u64 = 180 * 24 * 3_600;
/// Carência depois do aviso antes de remover de fato (soma ao prazo acima: ~7 meses de silêncio total).
pub const CARENCIA_APOS_AVISO_SEGUNDOS: u64 = 30 * 24 * 3_600;

#[derive(Clone, PartialEq, Eq)]
pub struct Contato {
    /// O nome que VOCÊ deu ao contato.
    pub apelido: String,
    /// Chave pública da pessoa (bytes de `ChavePublica`).
    pub chave_da_pessoa: Vec<u8>,
    pub notas: String,
    pub etiqueta: String,
}

impl core::fmt::Debug for Contato {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "Contato(<oculto>)") }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeContato { CamposGrandesDemais, Ilegivel, Malformado }

fn escrever(v: &mut Vec<u8>, b: &[u8]) { v.extend_from_slice(&(b.len() as u16).to_be_bytes()); v.extend_from_slice(b); }
fn ler<'a>(d: &'a [u8], i: &mut usize) -> Result<&'a [u8], ErroDeContato> {
    if *i + 2 > d.len() { return Err(ErroDeContato::Malformado); }
    let n = u16::from_be_bytes([d[*i], d[*i + 1]]) as usize;
    *i += 2;
    if *i + n > d.len() { return Err(ErroDeContato::Malformado); }
    let s = &d[*i..*i + n];
    *i += n;
    Ok(s)
}

/// Sela o contato inteiro para a chave do dono, em bloco de tamanho fixo.
pub fn selar_contato(dono: &ChavePublica, c: &Contato) -> Result<Vec<u8>, ErroDeContato> {
    let usado = 2 + c.apelido.len() + 2 + c.chave_da_pessoa.len() + 2 + c.notas.len() + 2 + c.etiqueta.len();
    if usado > TAM_FIXO_DO_CONTATO || c.chave_da_pessoa.len() > u16::MAX as usize { return Err(ErroDeContato::CamposGrandesDemais); }
    // Capacidade exata desde o início: nunca realoca, nenhuma cópia do nome fica esquecida.
    let mut claro = Zeroizing::new(Vec::with_capacity(TAM_FIXO_DO_CONTATO));
    escrever(&mut claro, c.apelido.as_bytes());
    escrever(&mut claro, &c.chave_da_pessoa);
    escrever(&mut claro, c.notas.as_bytes());
    escrever(&mut claro, c.etiqueta.as_bytes());
    claro.resize(TAM_FIXO_DO_CONTATO, 0);
    envelope::selar(dono, &claro, CONTEXTO_CONTATO).map_err(|_| ErroDeContato::CamposGrandesDemais)
}

/// Abre no aparelho do dono.
pub fn abrir_contato(sec: &ChaveSecreta, publ: &ChavePublica, bloco: &[u8]) -> Result<Contato, ErroDeContato> {
    let claro = envelope::abrir(sec, publ, bloco, CONTEXTO_CONTATO).map_err(|_| ErroDeContato::Ilegivel)?;
    if claro.len() != TAM_FIXO_DO_CONTATO { return Err(ErroDeContato::Malformado); }
    let mut i = 0;
    let texto = |b: &[u8]| String::from_utf8(b.to_vec()).map_err(|_| ErroDeContato::Malformado);
    let apelido = texto(ler(&claro, &mut i)?)?;
    let chave_da_pessoa = ler(&claro, &mut i)?.to_vec();
    let notas = texto(ler(&claro, &mut i)?)?;
    let etiqueta = texto(ler(&claro, &mut i)?)?;
    Ok(Contato { apelido, chave_da_pessoa, notas, etiqueta })
}

/// Decide se a tela deve avisar que este contato está inativo (nenhuma troca há 6 meses). Quem chama
/// decide o que é "troca" (mensagem, chamada, abrir a conversa) e passa a data da mais recente.
pub fn deve_avisar_inatividade(ultima_interacao: u64, agora: u64) -> bool {
    agora.saturating_sub(ultima_interacao) >= AVISO_DE_CONTATO_INATIVO_SEGUNDOS
}

/// Decide se o contato deve ser removido automaticamente: só depois de JÁ ter avisado (`avisado_em`) e de
/// mais `CARENCIA_APOS_AVISO_SEGUNDOS` terem passado sem nenhuma troca nova. Sem aviso registrado, nunca
/// remove sozinho — o aviso é sempre o primeiro passo, nunca uma remoção direta e silenciosa.
pub fn deve_remover_por_inatividade(ultima_interacao: u64, avisado_em: Option<u64>, agora: u64) -> bool {
    match avisado_em {
        Some(aviso) => deve_avisar_inatividade(ultima_interacao, agora) && agora.saturating_sub(aviso) >= CARENCIA_APOS_AVISO_SEGUNDOS,
        None => false,
    }
}

/// Resultado de comparar a identidade de quem mandou esta mensagem com a identidade que este contato
/// tinha registrada da última vez (novo, 10/10/2026 — pedido do dono: avisar os contatos quando o dono
/// troca de aparelho ou de conta). Detecção do lado de QUEM RECEBE, nunca um autorrelato de quem mudou:
/// um aparelho clonado ou comprometido não tem como suprimir isto silenciando o próprio aviso, porque não
/// é ele quem decide mandar o aviso — é cada contato que, ao verificar a mensagem recebida, nota a
/// diferença por conta própria. Mesmo princípio de "número de segurança mudou" já consagrado em apps de
/// mensagem cifrada ponta a ponta; o Nimbus não inventa um mecanismo novo aqui, aplica o mesmo padrão já
/// validado pelo mercado sobre a impressão digital que `assinatura::ChaveDeVerificacao::impressao_digital`
/// já calcula.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvisoDeIdentidade {
    /// Nunca houve impressão digital registrada antes para este contato — não é "mudança", é o primeiro
    /// contato. Tratado separado de `Mudou` de propósito: toda conversa nova teria uma primeira mensagem,
    /// e tratar isso como "mudou" encheria toda conversa nova de um aviso de segurança sem necessidade,
    /// cansando o sinal até a pessoa parar de prestar atenção nele quando importa de verdade.
    PrimeiroContato,
    Inalterada,
    /// A chave de quem assinou esta mensagem é diferente da última registrada para este contato: troca de
    /// aparelho (voluntária ou não), reinstalação, recuperação por guardiões, ou — no caso que este aviso
    /// existe para cobrir — um aparelho clonado se passando pelo contato.
    Mudou,
}

/// `conhecida`: a impressão digital que este contato tinha registrada da última vez que uma mensagem dele
/// foi verificada (`None` só no primeiro contato de todos, antes de qualquer mensagem). `desta_mensagem`:
/// a impressão digital de quem efetivamente assinou/cifrou a mensagem que chegou agora.
pub fn avaliar_identidade(conhecida: Option<[u8; 32]>, desta_mensagem: [u8; 32]) -> AvisoDeIdentidade {
    match conhecida {
        None => AvisoDeIdentidade::PrimeiroContato,
        Some(fp) if fp == desta_mensagem => AvisoDeIdentidade::Inalterada,
        Some(_) => AvisoDeIdentidade::Mudou,
    }
}

/// Só `Mudou` merece aparecer na tela da conversa como aviso de segurança. `PrimeiroContato` nunca avisa
/// (ver o comentário do variant); `Inalterada` obviamente não é aviso de nada.
pub fn deve_mostrar_aviso_de_identidade(a: AvisoDeIdentidade) -> bool {
    matches!(a, AvisoDeIdentidade::Mudou)
}

/// CTT10 — nomes que ninguém pode salvar PARA VOCÊ no contato delas: o dono escolhe de 3 a 5 (nunca menos,
/// para ter como recusar de verdade; nunca mais que 5, para não virar uma lista gigante difícil de lembrar).
/// Várias entradas, mesmo que só uma importe de verdade, é de propósito: quem tenta adivinhar por tentativa
/// e erro não consegue distinguir qual delas é a que você realmente queria proteger — todas recusam igual,
/// com a mesma mensagem genérica.
///
/// **Limite que precisa estar dito, não só codificado:** isto é aplicado pelo app oficial do Nimbus na hora
/// de salvar o apelido; o apelido em si é dado local e privado de quem salva (o servidor nunca vê nenhum
/// nome salvo, por desenho — ver o comentário do módulo). Um cliente adulterado, feito pela própria pessoa
/// para burlar a própria cópia dela do app, pode ignorar esta checagem — não existe forma de impedir isso
/// sem o servidor passar a enxergar nomes salvos, o que destruiria a privacidade que este módulo garante.
/// A proteção real é contra o uso normal do app oficial, não uma prova criptográfica contra um app hostil.
pub const MIN_NOMES_PROIBIDOS: usize = 3;
pub const MAX_NOMES_PROIBIDOS: usize = 5;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeNomesProibidos { QuantidadeInvalida, NomeVazioAposNormalizar }

/// Normalização só de acentos latinos comuns (bom o bastante para nomes em português; não é uma dobra
/// Unicode completa — o núcleo não tem dependência para isso sem auditoria de nova biblioteca). Minúsculas,
/// sem espaço nas pontas, espaços internos colapsados, e só letras/dígitos/espaço sobrevivem.
pub fn normalizar_nome(nome: &str) -> String {
    fn sem_acento(c: char) -> char {
        match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a', 'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i', 'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u', 'ç' => 'c', 'ñ' => 'n', outro => outro,
        }
    }
    let limpo: String = nome.to_lowercase().chars().map(sem_acento).filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
    limpo.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Valida a quantidade (3 a 5) e normaliza cada nome. Recusa nome que, depois de normalizado, sobra vazio.
pub fn validar_nomes_proibidos(nomes: &[String]) -> Result<Vec<String>, ErroDeNomesProibidos> {
    if nomes.len() < MIN_NOMES_PROIBIDOS || nomes.len() > MAX_NOMES_PROIBIDOS { return Err(ErroDeNomesProibidos::QuantidadeInvalida); }
    let normalizados: Vec<String> = nomes.iter().map(|n| normalizar_nome(n)).collect();
    if normalizados.iter().any(String::is_empty) { return Err(ErroDeNomesProibidos::NomeVazioAposNormalizar); }
    Ok(normalizados)
}

/// A tela chama isto antes de aceitar um apelido digitado para ESTE contato específico. Percorre a lista
/// inteira sempre (nunca retorna no meio): a resposta não deve variar de forma observável conforme qual
/// entrada bateu, só se alguma bateu.
pub fn nome_permitido(proibidos_normalizados: &[String], tentativa: &str) -> bool {
    let alvo = normalizar_nome(tentativa);
    let mut bateu_em_algum = false;
    for n in proibidos_normalizados { bateu_em_algum |= *n == alvo; }
    !bateu_em_algum
}

/// Chave de perfil: a pessoa cifra o próprio nome com ela e só a entrega a quem aceitou como contato.
pub struct ChaveDePerfil(Zeroizing<[u8; 32]>);

impl core::fmt::Debug for ChaveDePerfil {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "ChaveDePerfil(<oculta>)") }
}

impl ChaveDePerfil {
    pub fn nova() -> Self { let mut k = [0u8; 32]; OsRng.fill_bytes(&mut k); ChaveDePerfil(Zeroizing::new(k)) }
    pub fn de_bytes(b: [u8; 32]) -> Self { ChaveDePerfil(Zeroizing::new(b)) }
    /// Para entregar ao novo contato, dentro do canal cifrado ponta a ponta.
    pub fn exportar(&self) -> Zeroizing<[u8; 32]> { Zeroizing::new(*self.0) }
}

/// Cifra o nome de perfil (tamanho fixo, com número aleatório único por cifra).
pub fn cifrar_nome_de_perfil(k: &ChaveDePerfil, nome: &str) -> Result<Vec<u8>, ErroDeContato> {
    if nome.len() + 2 > TAM_FIXO_DO_PERFIL { return Err(ErroDeContato::CamposGrandesDemais); }
    let mut claro = Zeroizing::new(Vec::with_capacity(TAM_FIXO_DO_PERFIL));
    escrever(&mut claro, nome.as_bytes());
    claro.resize(TAM_FIXO_DO_PERFIL, 0);
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let c = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&k.0[..]));
    let mut saida = nonce.to_vec();
    saida.extend(c.encrypt(XNonce::from_slice(&nonce), Payload { msg: &claro, aad: CONTEXTO_PERFIL }).expect("cifra"));
    Ok(saida)
}

pub fn decifrar_nome_de_perfil(k: &ChaveDePerfil, bloco: &[u8]) -> Result<String, ErroDeContato> {
    if bloco.len() != 24 + TAM_FIXO_DO_PERFIL + 16 { return Err(ErroDeContato::Malformado); }
    let c = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&k.0[..]));
    let claro = Zeroizing::new(c.decrypt(XNonce::from_slice(&bloco[..24]), Payload { msg: &bloco[24..], aad: CONTEXTO_PERFIL }).map_err(|_| ErroDeContato::Ilegivel)?);
    let mut i = 0;
    String::from_utf8(ler(&claro, &mut i)?.to_vec()).map_err(|_| ErroDeContato::Malformado)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn contato(apelido: &str) -> Contato {
        let (pessoa, _) = envelope::gerar_par_de_chaves();
        Contato { apelido: apelido.into(), chave_da_pessoa: pessoa.para_bytes(), notas: "advogado do inventário".into(), etiqueta: "Família".into() }
    }
    fn contem(h: &[u8], a: &[u8]) -> bool { h.windows(a.len()).any(|w| w == a) }

    #[test]
    fn nome_salvo_nunca_aparece_no_que_vai_ao_servidor() {
        let (dono, _) = envelope::gerar_par_de_chaves();
        let bloco = selar_contato(&dono, &contato("Mãe")).unwrap();
        for rastro in ["Mãe".as_bytes(), b"advogado", b"Fam"] { assert!(!contem(&bloco, rastro)); }
    }

    #[test]
    fn so_o_dono_abre_e_tudo_volta_igual() {
        let (dono_p, dono_s) = envelope::gerar_par_de_chaves();
        let c = contato("Dr. Almeida");
        let bloco = selar_contato(&dono_p, &c).unwrap();
        assert_eq!(abrir_contato(&dono_s, &dono_p, &bloco).unwrap(), c);
        let (outro_p, outro_s) = envelope::gerar_par_de_chaves();
        assert_eq!(abrir_contato(&outro_s, &outro_p, &bloco).unwrap_err(), ErroDeContato::Ilegivel);
    }

    #[test]
    fn nome_curto_ou_longo_ocupa_o_mesmo_tamanho() {
        let (dono, _) = envelope::gerar_par_de_chaves();
        let a = selar_contato(&dono, &contato("A")).unwrap();
        let b = selar_contato(&dono, &contato("Dra. Maria Fernanda de Albuquerque Sampaio Neto")).unwrap();
        assert_eq!(a.len(), b.len());
    }

    #[test]
    fn contato_adulterado_nao_abre() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let mut bloco = selar_contato(&p, &contato("Mãe")).unwrap();
        let n = bloco.len() - 1;
        bloco[n] ^= 1;
        assert!(abrir_contato(&s, &p, &bloco).is_err());
    }

    #[test]
    fn campos_gigantes_sao_recusados() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let mut c = contato("x");
        c.notas = "n".repeat(TAM_FIXO_DO_CONTATO);
        assert_eq!(selar_contato(&p, &c).unwrap_err(), ErroDeContato::CamposGrandesDemais);
    }

    #[test]
    fn nome_de_perfil_so_abre_com_a_chave_entregue_ao_contato() {
        let k = ChaveDePerfil::nova();
        let bloco = cifrar_nome_de_perfil(&k, "Gabriel").unwrap();
        assert!(!contem(&bloco, b"Gabriel"));
        assert_eq!(decifrar_nome_de_perfil(&ChaveDePerfil::de_bytes(*k.exportar()), &bloco).unwrap(), "Gabriel");
        assert!(decifrar_nome_de_perfil(&ChaveDePerfil::nova(), &bloco).is_err());
    }

    #[test]
    fn nomes_de_perfil_de_tamanhos_diferentes_tem_o_mesmo_tamanho_cifrado() {
        let k = ChaveDePerfil::nova();
        assert_eq!(cifrar_nome_de_perfil(&k, "Al").unwrap().len(), cifrar_nome_de_perfil(&k, "Alessandra Cristina").unwrap().len());
    }

    #[test]
    fn aviso_de_inatividade_so_depois_de_6_meses_remocao_so_depois_de_mais_30_dias_de_aviso() {
        let interacao = 1_000u64;
        let seis_meses = interacao + AVISO_DE_CONTATO_INATIVO_SEGUNDOS;
        assert!(!deve_avisar_inatividade(interacao, seis_meses - 1));
        assert!(deve_avisar_inatividade(interacao, seis_meses));
        // Sem aviso registrado, nunca remove sozinho, por mais tempo que passe.
        assert!(!deve_remover_por_inatividade(interacao, None, seis_meses + 10 * CARENCIA_APOS_AVISO_SEGUNDOS));
        // Com aviso, só remove depois da carência extra.
        assert!(!deve_remover_por_inatividade(interacao, Some(seis_meses), seis_meses + CARENCIA_APOS_AVISO_SEGUNDOS - 1));
        assert!(deve_remover_por_inatividade(interacao, Some(seis_meses), seis_meses + CARENCIA_APOS_AVISO_SEGUNDOS));
    }

    #[test]
    fn nomes_proibidos_exige_de_3_a_5_e_normaliza_acento_e_caixa() {
        assert_eq!(validar_nomes_proibidos(&["Antonio".into(), "Beto".into()]).unwrap_err(), ErroDeNomesProibidos::QuantidadeInvalida, "menos de 3 não vale");
        let seis = ["a", "b", "c", "d", "e", "f"].map(String::from);
        assert_eq!(validar_nomes_proibidos(&seis).unwrap_err(), ErroDeNomesProibidos::QuantidadeInvalida, "mais de 5 não vale");
        let nomes = validar_nomes_proibidos(&["Antônio".into(), "  JOÃO  ".into(), "cesar".into()]).unwrap();
        assert_eq!(nomes, vec!["antonio", "joao", "cesar"]);
    }

    #[test]
    fn nome_proibido_recusa_variacoes_de_acento_espaco_e_caixa_mas_aceita_o_resto() {
        let proibidos = validar_nomes_proibidos(&["Antônio".into(), "Carlos Eduardo".into(), "zeca".into()]).unwrap();
        for tentativa in ["antonio", "ANTÔNIO", "  Antonio  ", "AnTôNiO"] {
            assert!(!nome_permitido(&proibidos, tentativa), "{tentativa} deveria ser recusado");
        }
        assert!(!nome_permitido(&proibidos, "carlos eduardo"));
        assert!(nome_permitido(&proibidos, "Antonio Carlos"), "nome diferente, mesmo parecido, é aceito");
        assert!(nome_permitido(&proibidos, "Mãe"));
    }

    #[test]
    fn nome_proibido_recusa_em_qualquer_posicao_da_lista_nao_so_a_primeira() {
        let proibidos = validar_nomes_proibidos(&["um".into(), "dois".into(), "tres".into()]).unwrap();
        for (i, p) in proibidos.iter().enumerate() {
            assert!(!nome_permitido(&proibidos, p), "posição {i} deveria recusar");
        }
    }

    #[test]
    fn primeiro_contato_nunca_e_tratado_como_mudanca() {
        let fp = [3u8; 32];
        assert_eq!(avaliar_identidade(None, fp), AvisoDeIdentidade::PrimeiroContato);
        assert!(!deve_mostrar_aviso_de_identidade(AvisoDeIdentidade::PrimeiroContato));
    }

    #[test]
    fn identidade_igual_nunca_avisa() {
        let fp = [5u8; 32];
        assert_eq!(avaliar_identidade(Some(fp), fp), AvisoDeIdentidade::Inalterada);
        assert!(!deve_mostrar_aviso_de_identidade(AvisoDeIdentidade::Inalterada));
    }

    #[test]
    fn identidade_diferente_sempre_avisa_mesmo_com_um_unico_byte_de_diferenca() {
        let antiga = [9u8; 32];
        let mut nova = antiga;
        nova[31] ^= 1;
        assert_eq!(avaliar_identidade(Some(antiga), nova), AvisoDeIdentidade::Mudou);
        assert!(deve_mostrar_aviso_de_identidade(AvisoDeIdentidade::Mudou));
    }

    #[test]
    fn debug_nunca_mostra_o_contato() {
        assert_eq!(format!("{:?}", contato("Mãe")), "Contato(<oculto>)");
        assert_eq!(format!("{:?}", ChaveDePerfil::nova()), "ChaveDePerfil(<oculta>)");
    }
}
