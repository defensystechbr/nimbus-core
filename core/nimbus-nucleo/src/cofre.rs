//! # Módulo `cofre` — tudo cifrado, até o tipo da coisa (COF1–COF8)
//!
//! Álbum (e o nome do álbum), nota (título e texto), compromisso (título, local, participantes), senha
//! (site, usuário, senha), ajuste do app: qualquer item vira **um único bloco cifrado** em que o servidor
//! não sabe o conteúdo, os nomes, nem **se é uma foto, uma nota ou uma senha** — o tipo vai dentro da cifra.
//!
//! O tamanho também é escondido: todo item é completado até a próxima **classe de tamanho** (1 KiB, 4 KiB,
//! 16 KiB, 64 KiB, 256 KiB, 1 MiB, 4 MiB, 16 MiB, 64 MiB). Uma senha e uma nota curta ficam idênticas por fora.
//!
//! ## Retenção por prazo: reaproveitando `historico`, nunca um apagamento próprio (09/10/2026)
//!
//! Guarda com prazo (o "cofre de guarda" do NimbusChat, por plano) **não** ganhou um mecanismo de
//! expiração ou de "apagar com segurança" escrito aqui. Dois motivos, nessa ordem:
//!
//! 1. **Sobrescrever bytes de verdade em armazenamento flash não é confiável.** O controlador do SSD/flash
//!    faz *wear-leveling*: "sobrescrever" um arquivo frequentemente grava em outra célula física e marca a
//!    antiga como livre, sem destruir o conteúdo original. Caches de miniatura do sistema, backup automático
//!    de nuvem e núcleos de falha (crash dump) podem guardar cópia sem o app saber. Escrever uma rotina
//!    própria de "apagar memória/cache" apostaria numa garantia que a própria plataforma não oferece — e
//!    seria código novo, não testado por mutação em nenhuma rodada anterior: mais superfície, não menos.
//! 2. O módulo `historico` já resolve isso **sem depender de sobrescrever nada**: apagamento criptográfico
//!    por janela de tempo. O conteúdo é cifrado sob a chave daquela janela; quando a janela vence, só a
//!    chave (pequena, vive em memória protegida) é destruída. A partir daí, **qualquer cópia do bloco —
//!    servidor, cache, backup, flash "mal apagado" — fica ilegível para sempre**, sem precisar confiar que
//!    alguém, em algum lugar, realmente apagou os bytes.
//!
//! `guardar_no_historico`/`abrir_do_historico`, abaixo, só compõem as duas peças que já existiam: `selar`/
//! `abrir` (esconder tipo e tamanho) por dentro, `historico::guardar`/`historico::ler` (janela com prazo,
//! apagamento criptográfico) por fora. Nenhuma das duas precisou mudar. `PrazoDeRetencaoDoCofre` é
//! deliberadamente genérico (um número de dias, com piso e teto sãos) — **qual número cada plano usa é
//! decisão da camada de produto/cobrança, não deste núcleo**, no mesmo espírito de `coacao::TetoDiurno` e
//! `sessoes::TetoDeSessaoDoComputador`: o núcleo garante que o valor nunca é absurdo (zero, ou "para
//! sempre"), quem escolhe o número dentro desse intervalo é quem decide o plano.
//!
//! **O que isto não resolve, por desenho, e por que isso é do módulo `checkup`, não daqui:** apagamento
//! criptográfico só vale enquanto a chave da janela não existir em nenhum outro lugar. No computador
//! vinculado (não no celular — ver `memoria.rs`), hibernação do sistema pode gravar a RAM inteira,
//! inclusive uma chave momentaneamente em uso, num arquivo de disco **não cifrado** se o disco do sistema
//! não tiver FileVault/BitLocker ativo. Isso não se resolve com mais código de cofre; resolve-se cobrando
//! essa verificação no `checkup` (ver `Item::ComputadorSemDiscoCifrado`).
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use crate::historico;
use zeroize::Zeroizing;

const CONTEXTO: &[u8] = b"NimbusOne/v1/cofre";
const MAGICO: &[u8; 4] = b"COF1";
/// Classes de tamanho (bytes) — potências de 4 a partir de 1 KiB.
pub const CLASSES: [usize; 9] = [1 << 10, 1 << 12, 1 << 14, 1 << 16, 1 << 18, 1 << 20, 1 << 22, 1 << 24, 1 << 26];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tipo { Foto, Video, Audio, Arquivo, Album, Nota, Compromisso, Senha, Ajuste }

impl Tipo {
    fn codigo(self) -> u8 { self as u8 }
    fn de_codigo(c: u8) -> Option<Tipo> {
        use Tipo::*;
        [Foto, Video, Audio, Arquivo, Album, Nota, Compromisso, Senha, Ajuste].into_iter().find(|t| t.codigo() == c)
    }
}

/// Um item do cofre: tipo, campos nomeados (ex.: "titulo", "local", "usuario") e conteúdo binário.
#[derive(Clone, PartialEq, Eq)]
pub struct Item { pub tipo: Tipo, pub campos: Vec<(String, String)>, pub conteudo: Vec<u8> }

impl core::fmt::Debug for Item {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "Item(<oculto>)") }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDoCofre { GrandeDemais, Ilegivel, Malformado }

/// Classe de tamanho para `n` bytes úteis.
pub fn classe_para(n: usize) -> Option<usize> { CLASSES.iter().copied().find(|c| *c >= n) }

fn tamanho_serializado(i: &Item) -> usize {
    4 + 1 + 2 + i.campos.iter().map(|(k, v)| 2 + k.len() + 4 + v.len()).sum::<usize>() + 8 + i.conteudo.len()
}

/// Sela o item para a chave do dono, na classe de tamanho adequada.
pub fn selar(dono: &ChavePublica, item: &Item) -> Result<Vec<u8>, ErroDoCofre> {
    if item.campos.len() > u16::MAX as usize { return Err(ErroDoCofre::GrandeDemais); }
    for (k, v) in &item.campos { if k.len() > u16::MAX as usize || v.len() > u32::MAX as usize { return Err(ErroDoCofre::GrandeDemais); } }
    let usado = tamanho_serializado(item);
    // A classe inclui o prefixo de tamanho do envelope (4 bytes): o bloco final fica exatamente na classe.
    let classe = classe_para(usado + 4).ok_or(ErroDoCofre::GrandeDemais)? - 4;
    // Capacidade exata desde o início: sem realocação, sem cópia esquecida.
    let mut claro = Zeroizing::new(Vec::with_capacity(classe));
    claro.extend_from_slice(MAGICO);
    claro.push(item.tipo.codigo());
    claro.extend_from_slice(&(item.campos.len() as u16).to_be_bytes());
    for (k, v) in &item.campos {
        claro.extend_from_slice(&(k.len() as u16).to_be_bytes());
        claro.extend_from_slice(k.as_bytes());
        claro.extend_from_slice(&(v.len() as u32).to_be_bytes());
        claro.extend_from_slice(v.as_bytes());
    }
    claro.extend_from_slice(&(item.conteudo.len() as u64).to_be_bytes());
    claro.extend_from_slice(&item.conteudo);
    claro.resize(classe, 0);
    envelope::selar(dono, &claro, CONTEXTO).map_err(|_| ErroDoCofre::GrandeDemais)
}

fn ler<const N: usize>(d: &[u8], i: &mut usize) -> Result<[u8; N], ErroDoCofre> {
    if *i + N > d.len() { return Err(ErroDoCofre::Malformado); }
    let mut b = [0u8; N];
    b.copy_from_slice(&d[*i..*i + N]);
    *i += N;
    Ok(b)
}
fn ler_texto(d: &[u8], i: &mut usize, n: usize) -> Result<String, ErroDoCofre> {
    if *i + n > d.len() { return Err(ErroDoCofre::Malformado); }
    let s = String::from_utf8(d[*i..*i + n].to_vec()).map_err(|_| ErroDoCofre::Malformado)?;
    *i += n;
    Ok(s)
}

/// Dias mínimos de prazo de retenção: menos que isso e um "guardar com prazo" fica indistinguível, na
/// prática, de um "apagar logo" por engano — nunca um erro de um dígito que vira silencioso.
pub const PRAZO_MINIMO_DIAS: u64 = 1;
/// Dias máximos: nunca "para sempre", nem para o maior plano — é princípio de produto, não limite técnico.
/// Se um plano futuro precisar de mais que isso, é uma decisão deliberada que deve mudar este número de
/// propósito, não um crescimento silencioso.
///
/// **182 dias (6 meses), definido nesta rodada** — o teto do plano Max (Grátis 3 meses, Pro 4 meses,
/// Max 6 meses; números de produto, não deste núcleo). Já foi cogitado 12 meses, já que o mecanismo de
/// custódia mútua (ver `custodia_mutua`) dá às duas partes de uma conversa o poder de apagar a
/// qualquer momento — o risco de "ficar para sempre" é bem menor que numa guarda unilateral — mas o
/// cofre individual (este módulo, guardado só pelo dono, não pelos dois lados) não tem essa mesma
/// válvula de escape: só o dono decide apagar antes do prazo. Por isso o cofre individual segue com o
/// mesmo teto do plano (182), e não foi esticado a 365 nem a "ilimitado" só porque a ideia de 12 meses
/// surgiu para o caso de custódia mútua — são dois mecanismos com garantias diferentes, mesmo número
/// por coincidência de produto, não por acoplamento técnico.
pub const PRAZO_MAXIMO_DIAS: u64 = 182;

/// Prazo de retenção de um item do cofre guardado via `guardar_no_historico`: por quanto tempo a janela
/// de `historico` continua válida antes do apagamento criptográfico. Quem decide o número de dias para
/// cada plano é a camada de produto — aqui só garantimos piso e teto sãos (ver módulo, acima).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrazoDeRetencaoDoCofre(u64);

impl PrazoDeRetencaoDoCofre {
    pub fn de_dias(dias: u64) -> Self {
        PrazoDeRetencaoDoCofre(dias.clamp(PRAZO_MINIMO_DIAS, PRAZO_MAXIMO_DIAS))
    }
    pub fn em_dias(self) -> u64 { self.0 }
    pub fn em_segundos(self) -> u64 { self.0 * 86_400 }
}

/// Está dentro do período final de aviso antes de a janela vencer? Puramente informativo — a interface
/// pode usar isto para mostrar "faltam N dias" nos últimos `duracao_do_aviso` segundos antes do prazo;
/// isso não adianta nem atrasa em um segundo o apagamento real, que continua sendo só a janela do
/// `historico`. `expira_em` e `agora` nas mesmas unidades que o resto do núcleo usa (segundos).
pub fn dentro_do_periodo_de_aviso(expira_em: u64, agora: u64, duracao_do_aviso: u64) -> bool {
    agora < expira_em && expira_em - agora <= duracao_do_aviso
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeRetencao {
    AoSelar(ErroDoCofre),
    AoGuardar(envelope::Erro),
    AoAbrir(ErroDoCofre),
    AoLer(historico::ErroDeLeitura),
}

/// Guarda um item do cofre com prazo de retenção, reaproveitando o apagamento criptográfico por janela
/// de `historico` — ver a nota do módulo, acima, sobre por que não existe uma rotina de expiração própria
/// aqui. Prazo é sempre obrigatório (nunca `None`/permanente): quem quiser guardar para sempre não usa
/// esta função — isso evita o erro de, por engano, guardar um item do cofre sem prazo nenhum.
pub fn guardar_no_historico(
    servidor: &mut historico::ServidorCego,
    chaves: &mut historico::ChavesDoDono,
    dono: &ChavePublica,
    item: &Item,
    prazo: PrazoDeRetencaoDoCofre,
) -> Result<historico::IdDeBloco, ErroDeRetencao> {
    let bloco = selar(dono, item).map_err(ErroDeRetencao::AoSelar)?;
    historico::guardar(servidor, chaves, &bloco, Some(prazo.em_segundos())).map_err(ErroDeRetencao::AoGuardar)
}

/// Lê e abre um item guardado via `guardar_no_historico`. Se a janela já venceu (chave destruída), falha
/// com `ErroDeRetencao::AoLer(historico::ErroDeLeitura::ChaveDestruida)` — mesmo que o bloco cifrado ainda
/// exista em algum lugar, como qualquer leitura de `historico` depois do prazo.
pub fn abrir_do_historico(
    servidor: &historico::ServidorCego,
    chaves: &historico::ChavesDoDono,
    sec: &ChaveSecreta,
    publ: &ChavePublica,
    id: &historico::IdDeBloco,
) -> Result<Item, ErroDeRetencao> {
    let bloco = historico::ler(servidor, chaves, id).map_err(ErroDeRetencao::AoLer)?;
    abrir(sec, publ, &bloco[..]).map_err(ErroDeRetencao::AoAbrir)
}

/// Abre no aparelho do dono.
pub fn abrir(sec: &ChaveSecreta, publ: &ChavePublica, bloco: &[u8]) -> Result<Item, ErroDoCofre> {
    let claro = envelope::abrir(sec, publ, bloco, CONTEXTO).map_err(|_| ErroDoCofre::Ilegivel)?;
    let mut i = 0;
    if ler::<4>(&claro, &mut i)? != *MAGICO { return Err(ErroDoCofre::Malformado); }
    let tipo = Tipo::de_codigo(ler::<1>(&claro, &mut i)?[0]).ok_or(ErroDoCofre::Malformado)?;
    let n = u16::from_be_bytes(ler::<2>(&claro, &mut i)?) as usize;
    let mut campos = Vec::with_capacity(n.min(64));
    for _ in 0..n {
        let tk = u16::from_be_bytes(ler::<2>(&claro, &mut i)?) as usize;
        let k = ler_texto(&claro, &mut i, tk)?;
        let tv = u32::from_be_bytes(ler::<4>(&claro, &mut i)?) as usize;
        let v = ler_texto(&claro, &mut i, tv)?;
        campos.push((k, v));
    }
    let tc = u64::from_be_bytes(ler::<8>(&claro, &mut i)?) as usize;
    if i + tc > claro.len() { return Err(ErroDoCofre::Malformado); }
    Ok(Item { tipo, campos, conteudo: claro[i..i + tc].to_vec() })
}

// --- trava da aba trancada: inatividade e retrava por integridade (rodada atual) ---

/// Segundos mínimos de inatividade antes de trancar — nunca zero: uma trava que fecha a cada toque
/// seria inutilizável, e o dono acabaria desativando a trava de verdade por cansaço, o problema
/// oposto ao que se quer resolver.
pub const TETO_DE_INATIVIDADE_MINIMO_SEGUNDOS: u32 = 5;
/// Segundos máximos: nunca "sem trava". 15 minutos é o maior tempo que ainda faz sentido chamar de
/// "trancado por inatividade" — acima disso, na prática, é a trava geral do aparelho que protege,
/// não esta.
pub const TETO_DE_INATIVIDADE_MAXIMO_SEGUNDOS: u32 = 900;

/// Depois de quanto tempo sem uso a aba trancada se tranca sozinha — independente da trava geral do
/// app/aparelho. Grampeado, nunca "sem teto" (mesmo padrão de `sessoes::TetoDeSessaoDoComputador`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TetoDeInatividadeDoCofre(u32);

impl TetoDeInatividadeDoCofre {
    pub fn de_segundos(s: u32) -> Self {
        TetoDeInatividadeDoCofre(s.clamp(TETO_DE_INATIVIDADE_MINIMO_SEGUNDOS, TETO_DE_INATIVIDADE_MAXIMO_SEGUNDOS))
    }
    pub fn em_segundos(self) -> u32 { self.0 }
}

/// Padrão quando o dono não escolheu outro: 30 segundos — curto o bastante para proteger um aparelho
/// largado na mesa com a aba aberta, longo o bastante para não trancar no meio da leitura.
impl Default for TetoDeInatividadeDoCofre {
    fn default() -> Self { TetoDeInatividadeDoCofre(30) }
}

/// Política de trava da aba trancada (cofre e `custodia_mutua`). `reautenticar_a_cada_abertura`: se
/// verdadeiro, mesmo reabrindo poucos segundos depois do último uso, a aba exige nova biometria —
/// nunca confia em "ainda estava desbloqueada" (a segunda trava facial pedida, independente da trava
/// geral do app). A exigência de chave em hardware seguro (Secure Enclave/StrongBox) **não** ganhou
/// um campo próprio aqui: já é coberta por `integridade::avaliar`, que manda o aparelho para Vermelho
/// quando `origem_da_atestacao == Software` — duplicar essa checagem criaria duas fontes de verdade
/// que poderiam discordar entre si. `deve_trancar`, abaixo, já herda essa proteção ao checar o nível.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoliticaDeTrancaDoCofre {
    pub teto_de_inatividade: TetoDeInatividadeDoCofre,
    pub reautenticar_a_cada_abertura: bool,
}

impl Default for PoliticaDeTrancaDoCofre {
    fn default() -> Self {
        PoliticaDeTrancaDoCofre { teto_de_inatividade: TetoDeInatividadeDoCofre::default(), reautenticar_a_cada_abertura: true }
    }
}

/// Decide se a aba trancada deve trancar AGORA — por inatividade (`ultimo_uso_em` + o teto da
/// política) OU porque o nível de integridade do aparelho não é mais Verde. Usa "não é mais Verde",
/// não só "é Vermelho": `integridade::permissoes` já nega `album_cofre` tanto no Amarelo quanto no
/// Vermelho (ver `integridade.rs`) — travar só no Vermelho seria uma regra MAIS FRACA que a já
/// estabelecida para o resto do cofre, uma inconsistência introduzida aqui, não uma herança do que já
/// existe. Como a avaliação de integridade se reavalia sozinha a cada 10 minutos ou ao voltar à tela
/// (`integridade::precisa_reavaliar`), qualquer sinal novo de comprometimento já tranca no próximo
/// uso, mesmo sem ter passado o teto de inatividade.
pub fn deve_trancar(
    politica: &PoliticaDeTrancaDoCofre,
    ultimo_uso_em: u64,
    agora: u64,
    nivel: crate::integridade::Nivel,
) -> bool {
    if nivel != crate::integridade::Nivel::Verde {
        return true;
    }
    agora >= ultimo_uso_em.saturating_add(politica.teto_de_inatividade.em_segundos() as u64)
}

#[cfg(test)]
mod testes {
    use super::*;
    fn contem(h: &[u8], a: &[u8]) -> bool { h.windows(a.len()).any(|w| w == a) }
    fn item(tipo: Tipo, campos: &[(&str, &str)], conteudo: &[u8]) -> Item {
        Item { tipo, campos: campos.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), conteudo: conteudo.to_vec() }
    }

    #[test]
    fn nome_do_album_nunca_aparece() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let b = selar(&p, &item(Tipo::Album, &[("nome", "Viagem Espanha 2027"), ("capa", "foto-7")], b"")).unwrap();
        assert!(!contem(&b, b"Espanha") && !contem(&b, b"nome"));
    }

    #[test]
    fn compromisso_nota_e_senha_nunca_aparecem() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let c = selar(&p, &item(Tipo::Compromisso, &[("titulo", "Reunião reservada"), ("local", "Sala 4"), ("participantes", "Marina")], b"")).unwrap();
        let n = selar(&p, &item(Tipo::Nota, &[("titulo", "Código do alarme")], b"4321")).unwrap();
        let s = selar(&p, &item(Tipo::Senha, &[("site", "banco.exemplo"), ("usuario", "gabriel")], b"S3nh@-F0rte")).unwrap();
        for (bloco, rastros) in [(&c, vec![&b"reservada"[..], b"Sala", b"Marina"]), (&n, vec![&b"alarme"[..], b"4321"]), (&s, vec![&b"banco"[..], b"gabriel", b"S3nh"])] {
            for r in rastros { assert!(!contem(bloco, r)); }
        }
    }

    #[test]
    fn por_fora_uma_senha_e_uma_nota_sao_identicas() {
        let (p, _) = envelope::gerar_par_de_chaves();
        let s = selar(&p, &item(Tipo::Senha, &[("site", "x")], b"y")).unwrap();
        let n = selar(&p, &item(Tipo::Nota, &[("titulo", "lista de compras do mês")], b"arroz, feijao")).unwrap();
        assert_eq!(s.len(), n.len(), "mesmo tamanho: o servidor não sabe o tipo");
    }

    #[test]
    fn itens_de_tamanhos_diferentes_na_mesma_classe_ficam_iguais() {
        // ~300 e ~900 bytes úteis: os dois caem na classe de 1 KiB e saem com o mesmo tamanho.
        let (p, _) = envelope::gerar_par_de_chaves();
        let a = selar(&p, &item(Tipo::Nota, &[], &[1u8; 300])).unwrap();
        let b = selar(&p, &item(Tipo::Senha, &[], &[1u8; 900])).unwrap();
        assert_eq!(a.len(), b.len());
    }

    #[test]
    fn so_o_dono_abre_e_volta_igual() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let it = item(Tipo::Compromisso, &[("titulo", "Médico"), ("local", "Clínica")], b"");
        assert_eq!(abrir(&s, &p, &selar(&p, &it).unwrap()).unwrap(), it);
        let (p2, s2) = envelope::gerar_par_de_chaves();
        assert_eq!(abrir(&s2, &p2, &selar(&p, &it).unwrap()).unwrap_err(), ErroDoCofre::Ilegivel);
    }

    #[test]
    fn classes_de_tamanho() {
        assert_eq!(classe_para(1), Some(1024));
        assert_eq!(classe_para(1025), Some(4096));
        assert_eq!(classe_para(CLASSES[8] + 1), None);
    }

    #[test]
    fn bloco_adulterado_nao_abre() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let mut b = selar(&p, &item(Tipo::Nota, &[], b"x")).unwrap();
        let n = b.len() - 1;
        b[n] ^= 1;
        assert!(abrir(&s, &p, &b).is_err());
    }

    #[test]
    fn debug_nunca_mostra_o_item() {
        let it = item(Tipo::Senha, &[("site", "x")], b"valor-sensivel");
        assert_eq!(format!("{:?}", it), "Item(<oculto>)");
    }

    // --- retenção por prazo via `historico` (09/10/2026) ---

    #[test]
    fn prazo_de_retencao_e_grampeado_entre_1_e_182_dias() {
        assert_eq!(PrazoDeRetencaoDoCofre::de_dias(0).em_dias(), PRAZO_MINIMO_DIAS);
        assert_eq!(PrazoDeRetencaoDoCofre::de_dias(90).em_dias(), 90);
        assert_eq!(PrazoDeRetencaoDoCofre::de_dias(10_000).em_dias(), PRAZO_MAXIMO_DIAS);
        assert_eq!(PRAZO_MAXIMO_DIAS, 182, "teto desta rodada: 6 meses, igual ao plano Max");
    }

    #[test]
    fn guardar_e_abrir_do_historico_faz_ida_e_volta() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let (mut servidor, mut chaves) = (historico::ServidorCego::default(), historico::ChavesDoDono::novas(86_400));
        let it = item(Tipo::Foto, &[("legenda", "praia")], b"bytes-da-foto");
        let id = guardar_no_historico(&mut servidor, &mut chaves, &p, &it, PrazoDeRetencaoDoCofre::de_dias(90)).unwrap();
        assert_eq!(abrir_do_historico(&servidor, &chaves, &s, &p, &id).unwrap(), it);
    }

    #[test]
    fn janela_vencida_torna_item_do_cofre_ilegivel_mesmo_com_copia_guardada() {
        let (p, s) = envelope::gerar_par_de_chaves();
        let (mut servidor, mut chaves) = (historico::ServidorCego::default(), historico::ChavesDoDono::novas(3_600));
        let it = item(Tipo::Arquivo, &[], b"contrato-assinado");
        let id = guardar_no_historico(&mut servidor, &mut chaves, &p, &it, PrazoDeRetencaoDoCofre::de_dias(1)).unwrap();
        let copia_guardada_por_alguem = servidor.blocos.clone();
        assert!(abrir_do_historico(&servidor, &chaves, &s, &p, &id).is_ok(), "ainda dentro do prazo");
        let _ = chaves.destruir_janelas_vencidas(2 * 86_400);
        let servidor_com_copia = historico::ServidorCego { blocos: copia_guardada_por_alguem };
        assert_eq!(
            abrir_do_historico(&servidor_com_copia, &chaves, &s, &p, &id),
            Err(ErroDeRetencao::AoLer(historico::ErroDeLeitura::ChaveDestruida)),
            "mesmo com cópia do bloco, sem a chave da janela fica ilegível para sempre"
        );
    }

    #[test]
    fn esconder_tamanho_do_cofre_sobrevive_ao_envelope_extra_do_historico() {
        // Mesma ideia de `itens_de_tamanhos_diferentes_na_mesma_classe_ficam_iguais`, mas depois de passar
        // pela camada extra de `historico::guardar` — prova que o envelope de fora não reabre o vazamento
        // de tamanho que o envelope de dentro (cofre) já tinha fechado.
        let (p, _) = envelope::gerar_par_de_chaves();
        let (mut servidor, mut chaves) = (historico::ServidorCego::default(), historico::ChavesDoDono::novas(86_400));
        let a = item(Tipo::Nota, &[], &[1u8; 300]);
        let b = item(Tipo::Senha, &[], &[1u8; 900]);
        let prazo = PrazoDeRetencaoDoCofre::de_dias(30);
        let ida = guardar_no_historico(&mut servidor, &mut chaves, &p, &a, prazo).unwrap();
        let idb = guardar_no_historico(&mut servidor, &mut chaves, &p, &b, prazo).unwrap();
        assert_eq!(servidor.blocos[&ida].len(), servidor.blocos[&idb].len(), "servidor não aprende o tamanho real nem depois do historico");
    }

    #[test]
    fn periodo_de_aviso_so_liga_perto_do_fim_e_nunca_depois_de_vencer() {
        assert!(!dentro_do_periodo_de_aviso(1_000, 0, 100), "longe do prazo: sem aviso");
        assert!(dentro_do_periodo_de_aviso(1_000, 950, 100), "faltando menos que a janela de aviso");
        assert!(!dentro_do_periodo_de_aviso(1_000, 1_000, 100), "no instante exato do vencimento: já venceu, não é mais 'aviso'");
        assert!(!dentro_do_periodo_de_aviso(1_000, 1_001, 100), "depois de vencido: não é aviso, é passado");
        assert!(dentro_do_periodo_de_aviso(1_000, 900, 100), "na borda exata do início do aviso (inclusive)");
    }

    // --- trava da aba trancada (rodada atual) ---
    use crate::integridade::Nivel;

    #[test]
    fn teto_de_inatividade_e_grampeado_entre_5_e_900_segundos() {
        assert_eq!(TetoDeInatividadeDoCofre::de_segundos(0).em_segundos(), TETO_DE_INATIVIDADE_MINIMO_SEGUNDOS);
        assert_eq!(TetoDeInatividadeDoCofre::de_segundos(60).em_segundos(), 60);
        assert_eq!(TetoDeInatividadeDoCofre::de_segundos(10_000).em_segundos(), TETO_DE_INATIVIDADE_MAXIMO_SEGUNDOS);
    }

    #[test]
    fn padrao_da_trava_e_30_segundos_e_exige_reautenticacao() {
        assert_eq!(TetoDeInatividadeDoCofre::default().em_segundos(), 30);
        assert!(PoliticaDeTrancaDoCofre::default().reautenticar_a_cada_abertura, "sem confiar em 'ainda estava desbloqueada'");
    }

    #[test]
    fn nao_tranca_dentro_do_teto_com_nivel_verde() {
        let p = PoliticaDeTrancaDoCofre { teto_de_inatividade: TetoDeInatividadeDoCofre::de_segundos(30), ..Default::default() };
        assert!(!deve_trancar(&p, 1_000, 1_029, Nivel::Verde), "ainda dentro dos 30s");
    }

    #[test]
    fn tranca_por_inatividade_mesmo_com_nivel_verde() {
        let p = PoliticaDeTrancaDoCofre { teto_de_inatividade: TetoDeInatividadeDoCofre::de_segundos(30), ..Default::default() };
        assert!(deve_trancar(&p, 1_000, 1_030, Nivel::Verde), "no limite exato dos 30s, já tranca");
    }

    #[test]
    fn tranca_de_imediato_no_amarelo_mesmo_sem_qualquer_inatividade() {
        // Decisão deliberada: `integridade::permissoes` já nega `album_cofre` no Amarelo, não só no
        // Vermelho — travar só no Vermelho seria mais fraco que essa regra já existente.
        let p = PoliticaDeTrancaDoCofre::default();
        assert!(deve_trancar(&p, 1_000, 1_000, Nivel::Amarelo), "mesmo instante, zero inatividade, ainda assim tranca");
    }

    #[test]
    fn tranca_de_imediato_no_vermelho_mesmo_sem_qualquer_inatividade() {
        let p = PoliticaDeTrancaDoCofre::default();
        assert!(deve_trancar(&p, 1_000, 1_000, Nivel::Vermelho));
    }

    #[test]
    fn integridade_permissoes_ja_nega_album_cofre_fora_do_verde() {
        // Confere a premissa que `deve_trancar` assume: se isto um dia mudar em `integridade.rs`,
        // este teste quebra primeiro, antes de `deve_trancar` ficar inconsistente em silêncio.
        assert!(crate::integridade::permissoes(Nivel::Verde).album_cofre);
        assert!(!crate::integridade::permissoes(Nivel::Amarelo).album_cofre);
        assert!(!crate::integridade::permissoes(Nivel::Vermelho).album_cofre);
    }
}
