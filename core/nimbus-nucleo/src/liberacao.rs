//! # Módulo `liberacao` — nada sai da conversa sem autorização de quem enviou (LIB1–LIB16)
//!
//! **Regra de ouro: mensagem não sai.** Texto, mensagem de voz, vídeo circular, figurinha e localização
//! não se copiam, não se guardam e não se levam para fora — nunca. Print e gravação de tela não existem.
//!
//! **Única exceção, estreita de propósito: o cartão encaminhável.** Quem escreve pode marcar, ANTES de
//! enviar, um texto ou uma localização (ex.: um endereço) como encaminhável dentro do Nimbus, por um prazo
//! obrigatório de no máximo 4 horas. Nunca depois de enviado, nunca para fora do Nimbus, um salto só.
//!
//! **Arquivo** (foto, vídeo, documento) só sai com autorização do autor, arquivo por arquivo:
//! `Nenhum` (padrão) < `GuardarNoCofre` < `EncaminharNoNimbus` < `LevarParaFora`.
//!
//! Toda liberação é **assinada pelo autor** (assinatura híbrida pós-quântica) e pode ter **prazo**
//! (30 s, 1 min, 2 min, 1 h, 2 h, 4 h), contado a partir de quando quem recebe abre o conteúdo; ao fim,
//! volta a `Nenhum` e, se o autor quiser, o próprio item some. Quem encaminha nunca aumenta o nível, e o
//! que chega por encaminhamento não pode ser encaminhado de novo (um salto). O autor pode revogar e é
//! avisado quando o conteúdo se move. Sem marca d’água: nada no arquivo identifica quem o recebeu.
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use crate::higiene;
use crate::politica::TipoDeConteudo;

const CONTEXTO: &[u8] = b"NimbusOne/v1/liberacao";
/// Prazos oferecidos na tela, em segundos: 30 s, 1 min, 2 min, 1 h, 2 h, 4 h.
pub const PRAZOS: [u64; 6] = [30, 60, 120, 3_600, 7_200, 14_400];
/// Cartão encaminhável: prazo obrigatório, de no máximo 4 horas.
pub const PRAZO_MAXIMO_DO_CARTAO: u64 = 14_400;

/// Níveis em ordem crescente: cada um inclui os anteriores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nivel { Nenhum, GuardarNoCofre, EncaminharNoNimbus, LevarParaFora }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uso {
    GuardarNoCofre,
    EncaminharNoNimbus,
    LevarParaFora,
    /// RV4 — guardar na "Minha Pasta" pessoal de quem recebeu. Só existe para MENSAGENS (arquivo já usa
    /// `GuardarNoCofre`); usa a MESMA autorização assinada e o mesmo prazo do cartão encaminhável — o autor
    /// não precisa conceder duas permissões separadas, mas precisa conceder pelo menos uma.
    GuardarEmMinhaPasta,
}

/// `pub(crate)`: só para `pedido_de_guarda::responder_para_destino` saber a que `Nivel` cada `Destino`
/// corresponde — continua fora da API pública do núcleo, mesma visibilidade de sempre deste módulo.
pub(crate) fn exigido(u: Uso) -> Nivel {
    match u {
        Uso::GuardarNoCofre | Uso::GuardarEmMinhaPasta => Nivel::GuardarNoCofre,
        Uso::EncaminharNoNimbus => Nivel::EncaminharNoNimbus,
        Uso::LevarParaFora => Nivel::LevarParaFora,
    }
}

/// Os três destinos da tela única de liberação (pedido do dono em 11/10/2026): sempre que quem recebeu
/// quer tirar algo da conversa — baixar, guardar, reenviar para fora — a tela mostra exatamente estes três,
/// nunca escondidos atrás de um só botão "liberar", para quem decide (o autor) ver sempre para onde vai.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destino {
    /// Continua dentro da conversa: a aba trancada (`custodia_mutua`). Qualquer um dos dois lados ainda
    /// apaga para os dois a qualquer momento — é por isso que sair por aqui preocupa menos (regra 34 do
    /// CONTRATO_DA_PONTE: mecanismo próprio, nunca passa por `Uso`/`Autorizacao`; `uso_correspondente`
    /// devolve `None` de propósito).
    AbaTrancada,
    /// Cópia permanente no cofre pessoal de quem recebeu (`Uso::GuardarNoCofre`, via
    /// `pedido_de_guarda::responder`) — o autor perde controle dali para frente, mas o conteúdo nunca sai
    /// do ecossistema Nimbus.
    CofreNimbusOne,
    /// Sai do Nimbus de verdade: outro app, a galeria do aparelho, outro aparelho, e-mail etc.
    /// (`Uso::LevarParaFora`). Ver `CONTRATO_DA_PONTE.md` regra 42 sobre por que, hoje, este botão nunca
    /// produz uma liberação de verdade — é intencional, não um bug.
    ForaDoNimbus,
}

/// "Vai sair do domínio dele" (pedido explícito do dono, para aparecer na tela): só o destino que de fato
/// sai do ecossistema Nimbus mostra este aviso — os outros dois continuam dentro, mesmo que a aba trancada
/// permita apagar depois (apagar não é a mesma coisa que nunca ter saído).
pub fn sai_do_dominio_nimbus(d: Destino) -> bool { matches!(d, Destino::ForaDoNimbus) }

/// A que `Uso` cada destino corresponde quando passa pela autorização genérica de `liberacao`/
/// `pedido_de_guarda`. `AbaTrancada` devolve `None` de propósito: não é um `Uso`, é o mecanismo separado
/// de `custodia_mutua` (regra 34) — nunca produz uma `Autorizacao`.
pub fn uso_correspondente(d: Destino) -> Option<Uso> {
    match d {
        Destino::AbaTrancada => None,
        Destino::CofreNimbusOne => Some(Uso::GuardarNoCofre),
        Destino::ForaDoNimbus => Some(Uso::LevarParaFora),
    }
}

/// **Achado desta rodada, não uma decisão nova:** diz se `destino`, mesmo aprovado pelo autor, ainda fica
/// bloqueado pelo teto desta conversa (`teto_da_conversa`). Hoje isto é sempre verdadeiro para
/// `ForaDoNimbus`: a v37 (terceira rodada) fixou o teto de QUALQUER conversa em `EncaminharNoNimbus` —
/// abaixo de `LevarParaFora` — "não uma trava que dá para destravar depois". `AbaTrancada` nunca é
/// bloqueada por este teto (não usa `Uso`; tem a regra própria de `custodia_mutua`).
///
/// **Escopo exato, para não confundir com `efetivo` (achado da autovalidação, documentado em vez de só
/// corrigido em silêncio):** esta função só modela o caminho de ARQUIVO (`Uso`/`exigido`), que é tudo que
/// `Destino`/`pedido_de_guarda` cobrem. Conteúdo de MENSAGEM tem, em `efetivo`, um teto PRÓPRIO e mais
/// estrito (`.min(Nivel::EncaminharNoNimbus)`), independente do `teto` passado aqui — não é o mesmo
/// cálculo, só coincide com `ForaDoNimbus` porque `teto_da_conversa` já é `EncaminharNoNimbus` hoje para
/// as duas coisas. Nunca reaproveitar esta função para decidir o que uma MENSAGEM pode fazer.
pub fn destino_bloqueado_pelo_teto(d: Destino, teto: Nivel) -> bool {
    match uso_correspondente(d) {
        Some(uso) => teto < exigido(uso),
        None => false,
    }
}

/// Mensagens: nunca se copiam, guardam ou levam para fora.
pub fn e_mensagem(t: TipoDeConteudo) -> bool {
    matches!(t, TipoDeConteudo::Texto | TipoDeConteudo::Audio | TipoDeConteudo::VideoCircular | TipoDeConteudo::Figurinha | TipoDeConteudo::Localizacao | TipoDeConteudo::LocalizacaoAoVivo)
}

/// Tipos de mensagem que podem virar cartão encaminhável, sempre com autorização assinada do autor e o
/// mesmo teto de 4 horas. Figurinha entra porque é um recurso genérico e pré-pronto, sem conteúdo pessoal
/// do autor — ao contrário de voz e vídeo circular, que identificam quem gravou e por isso ficam de fora.
/// `LocalizacaoAoVivo` (RV7) também fica de fora, deliberadamente diferente de `Localizacao` (um ponto só):
/// reencaminhar um rastreamento ao vivo deixaria uma TERCEIRA pessoa acompanhar alguém que nunca autorizou
/// isso — o cartão de localização pontual não tem esse risco, porque é um instante, não um acompanhamento
/// contínuo.
pub fn pode_ser_cartao(t: TipoDeConteudo) -> bool { matches!(t, TipoDeConteudo::Texto | TipoDeConteudo::Localizacao | TipoDeConteudo::Figurinha) }

/// Teto da conversa: o máximo que qualquer autor pode liberar nela.
///
/// **v37, terceira rodada — Conversa Blindada deixou de ser opcional.** Antes, isto tinha um parâmetro
/// `conversa_blindada: bool` que a pessoa podia ligar ou desligar por conversa. Decisão desta rodada,
/// pedida e confirmada pelo dono: TODA conversa agora tem o teto da Blindada sempre aplicado, sem opção
/// de desligar — não existe mais "conversa não blindada" em lugar nenhum do app.
///
/// O que isso muda na prática: nenhum arquivo pode ser **levado para fora do Nimbus** (`LevarParaFora`),
/// em NENHUMA conversa, nunca — essa opção fica estruturalmente fora de alcance, não é uma trava que dá
/// para destravar depois. O teto escolhido para o "sempre ligado" é `EncaminharNoNimbus`, não
/// `GuardarNoCofre` (o teto do antigo modo "Blindada" opcional): se o teto fosse `GuardarNoCofre`,
/// encaminhar um arquivo DENTRO do próprio Nimbus também ficaria impossível para sempre, o que anularia
/// a facilidade de encaminhar que foi pedida na mesma rodada (ver `ESPECIFICACAO_FLUXO_1_4...`). Com o
/// teto em `EncaminharNoNimbus`: guardar no cofre e encaminhar dentro do Nimbus continuam possíveis, com
/// a autorização assinada do autor de sempre; só sair do ecossistema Nimbus (para a galeria do aparelho,
/// outro app, e-mail etc.) é que deixa de existir como opção, para qualquer conversa, sem exceção.
pub fn teto_da_conversa(limite_da_empresa: Option<Nivel>) -> Nivel {
    let t = Nivel::EncaminharNoNimbus;
    match limite_da_empresa { Some(l) => t.min(l), None => t }
}

/// RV5 — autorização de encaminhamento em três camadas, da mais geral para a mais específica:
/// 1. **Master**: a regra geral do app inteiro, escolhida uma vez na configuração e trocável depois.
///    Padrão de fábrica recomendado: `Nivel::Nenhum` — nada é encaminhável até o dono ligar.
/// 2. **Padrão do contato**: substitui o Master só para as conversas com aquele contato específico.
/// 3. **Escolha na mensagem**: o quadradinho que aparece ao escrever, substitui tudo o mais só para
///    AQUELA mensagem — é a camada final, a mais específica sempre vence.
///
/// Nenhuma camada aumenta o que a conversa permite: o resultado ainda passa pelo teto normal
/// (`teto_da_conversa`) antes de virar uma `Autorizacao` de verdade.
pub fn resolver_nivel_de_encaminhamento(master: Nivel, padrao_do_contato: Option<Nivel>, escolha_na_mensagem: Option<Nivel>, teto: Nivel) -> Nivel {
    escolha_na_mensagem.or(padrao_do_contato).unwrap_or(master).min(teto)
}

/// Autorização assinada pelo AUTOR. Viaja junto com o conteúdo, cifrada ponta a ponta.
#[derive(Clone, Debug)]
pub struct Autorizacao {
    pub hash_do_conteudo: [u8; 32],
    pub nivel: Nivel,
    /// 1 = feita no envio. Versões maiores substituem as anteriores (revogação, mudança de nível).
    pub versao: u64,
    /// Prazo da liberação, contado a partir da primeira abertura por quem recebe.
    pub prazo_segundos: Option<u64>,
    /// Ao fim do prazo, o próprio item some da conversa.
    pub some_ao_fim: bool,
    /// Impressão digital da chave do autor.
    pub autor: [u8; 32],
    pub assinatura: AssinaturaHibrida,
}

fn bytes_para_assinar(hash: &[u8; 32], nivel: Nivel, versao: u64, prazo: Option<u64>, some_ao_fim: bool) -> Vec<u8> {
    let mut v = hash.to_vec();
    v.push(nivel as u8);
    v.extend_from_slice(&versao.to_be_bytes());
    match prazo { Some(p) => { v.push(1); v.extend_from_slice(&p.to_be_bytes()); } None => v.push(0) }
    v.push(some_ao_fim as u8);
    v
}

/// No aparelho do autor: ao enviar (versão 1) ou depois (versão maior).
pub fn autorizar(autor: &ChaveDeAssinatura, hash_do_conteudo: [u8; 32], nivel: Nivel, versao: u64, prazo_segundos: Option<u64>, some_ao_fim: bool) -> Autorizacao {
    let assinatura = autor.assinar(CONTEXTO, &bytes_para_assinar(&hash_do_conteudo, nivel, versao, prazo_segundos, some_ao_fim)).expect("contexto fixo e curto");
    Autorizacao { hash_do_conteudo, nivel, versao, prazo_segundos, some_ao_fim, autor: autor.verificacao().impressao_digital(), assinatura }
}

fn valida(chave_do_autor: &ChaveDeVerificacao, a: &Autorizacao, hash: &[u8; 32]) -> bool {
    a.hash_do_conteudo == *hash && a.autor == chave_do_autor.impressao_digital()
        && chave_do_autor.verificar(CONTEXTO, &bytes_para_assinar(&a.hash_do_conteudo, a.nivel, a.versao, a.prazo_segundos, a.some_ao_fim), &a.assinatura)
}

/// Nível que o autor realmente liberou, já com prazo, revogação e a regra das mensagens.
/// `aberto_em`: quando quem recebe abriu o conteúdo pela primeira vez (relógio confiável).
pub fn nivel_autorizado(chave_do_autor: &ChaveDeVerificacao, autorizacoes: &[Autorizacao], hash: &[u8; 32],
                        tipo: TipoDeConteudo, aberto_em: Option<u64>, agora: u64) -> Nivel {
    let validas: Vec<&Autorizacao> = autorizacoes.iter().filter(|a| valida(chave_do_autor, a, hash)).collect();
    let Some(ultima) = validas.iter().max_by_key(|a| a.versao) else { return Nivel::Nenhum };
    let (nivel, prazo) = if e_mensagem(tipo) {
        // Mensagem: só vale o que foi liberado NO ENVIO (versão 1); versões depois só podem reduzir.
        if !pode_ser_cartao(tipo) { return Nivel::Nenhum; }
        let Some(no_envio) = validas.iter().find(|a| a.versao == 1) else { return Nivel::Nenhum };
        let Some(p) = no_envio.prazo_segundos.filter(|p| *p <= PRAZO_MAXIMO_DO_CARTAO) else { return Nivel::Nenhum };
        (ultima.nivel.min(no_envio.nivel).min(Nivel::EncaminharNoNimbus), Some(p))
    } else {
        (ultima.nivel, ultima.prazo_segundos)
    };
    if let Some(p) = prazo {
        match aberto_em {
            Some(o) if agora < o.saturating_add(p) => {}
            _ => return Nivel::Nenhum,
        }
    }
    nivel
}

/// Nível que vale neste aparelho para este conteúdo.
pub fn efetivo(tipo: TipoDeConteudo, visualizacao_unica: bool, teto: Nivel, autorizado: Nivel) -> Nivel {
    if visualizacao_unica { return Nivel::Nenhum; }
    if e_mensagem(tipo) {
        if !pode_ser_cartao(tipo) { return Nivel::Nenhum; }
        return autorizado.min(teto).min(Nivel::EncaminharNoNimbus);
    }
    autorizado.min(teto)
}

/// Decisão final de cada uso. `recebido_por_encaminhamento`: o autor da autorização não é quem enviou
/// a este aparelho — então não pode ser encaminhado de novo (um salto só). "Minha Pasta" é guardar para
/// SI MESMO, não espalhar para outra conversa: o limite de um salto não se aplica a ela.
pub fn pode(tipo: TipoDeConteudo, nivel: Nivel, uso: Uso, recebido_por_encaminhamento: bool) -> bool {
    if uso == Uso::EncaminharNoNimbus && recebido_por_encaminhamento { return false; }
    if e_mensagem(tipo) { return matches!(uso, Uso::EncaminharNoNimbus | Uso::GuardarEmMinhaPasta) && nivel >= Nivel::EncaminharNoNimbus; }
    nivel >= exigido(uso)
}

/// Ao fim do prazo, o item some da conversa se o autor pediu.
pub fn item_deve_sumir(a: &Autorizacao, aberto_em: Option<u64>, agora: u64) -> bool {
    match (a.some_ao_fim, a.prazo_segundos, aberto_em) {
        (true, Some(p), Some(o)) => agora >= o.saturating_add(p),
        _ => false,
    }
}

/// Quem encaminha pode reduzir o nível para o próximo destinatário, nunca aumentar.
pub fn ao_encaminhar(recebido: Nivel, escolhido: Nivel) -> Nivel { recebido.min(escolhido) }

/// O autor é avisado (pelo canal cifrado) quando o conteúdo é encaminhado, levado para fora, ou guardado
/// — na Minha Pasta de quem recebeu (mensagem) ou no cofre pessoal de quem recebeu (arquivo) — é o mesmo
/// tipo de movimento: sair do lugar onde foi enviado e passar a existir em outro lugar, por mais tempo.
///
/// **Correção de 10/10/2026, pedido do dono ("ver que existe informação reencaminhada dos dois lados para
/// dentro do próprio nimbus"):** antes desta rodada, `GuardarNoCofre` (o uso de ARQUIVO equivalente a
/// `GuardarEmMinhaPasta` para mensagem — ambos exigem o mesmo `Nivel::GuardarNoCofre`, ver `exigido`) não
/// estava nesta lista — uma assimetria real: uma mensagem salva na Minha Pasta de quem recebeu já avisava
/// o autor, mas uma foto ou documento salvo no cofre pessoal de quem recebeu, não. O autor não vê O QUE
/// foi guardado nem onde — só que algo que ele enviou passou a existir além da conversa, dos dois lados
/// (o seu próprio ato de guardar já é óbvio para ele mesmo; o que faltava era o aviso quando é o OUTRO
/// lado que guarda).
pub fn avisar_o_autor(uso: Uso) -> bool {
    matches!(uso, Uso::EncaminharNoNimbus | Uso::LevarParaFora | Uso::GuardarEmMinhaPasta | Uso::GuardarNoCofre)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Recusa { NaoAutorizado }

/// Arquivo pronto para sair do Nimbus.
#[derive(Debug)]
pub struct Exportacao {
    pub dados: Vec<u8>,
    /// `false` quando o formato não permite limpar metadados (PDF, Office…): o app avisa antes de sair.
    pub metadados_limpos: bool,
}

/// Única porta de saída para fora do Nimbus (componente ExportacaoAutorizada nos apps).
pub fn preparar_para_fora(tipo: TipoDeConteudo, nivel: Nivel, dados: &[u8]) -> Result<Exportacao, Recusa> {
    if !pode(tipo, nivel, Uso::LevarParaFora, false) { return Err(Recusa::NaoAutorizado); }
    Ok(match higiene::limpar(dados) {
        Ok(limpo) => Exportacao { dados: limpo, metadados_limpos: true },
        Err(_) => Exportacao { dados: dados.to_vec(), metadados_limpos: false },
    })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;
    use TipoDeConteudo::*;

    const TODOS: [Uso; 3] = [Uso::GuardarNoCofre, Uso::EncaminharNoNimbus, Uso::LevarParaFora];
    const H: [u8; 32] = [5; 32];

    #[test]
    fn mensagem_nunca_se_guarda_nem_sai_do_nimbus() {
        for tipo in [Texto, Audio, VideoCircular, Figurinha, Localizacao, LocalizacaoAoVivo] {
            let n = efetivo(tipo, false, Nivel::LevarParaFora, Nivel::LevarParaFora);
            assert!(!pode(tipo, n, Uso::GuardarNoCofre, false) && !pode(tipo, n, Uso::LevarParaFora, false), "{tipo:?}");
        }
    }

    #[test]
    fn voz_e_video_circular_nunca_viram_cartao() {
        for tipo in [Audio, VideoCircular] {
            for uso in TODOS { assert!(!pode(tipo, efetivo(tipo, false, Nivel::LevarParaFora, Nivel::LevarParaFora), uso, false)); }
        }
    }

    #[test]
    fn arquivo_sem_autorizacao_nao_sai() {
        for tipo in [Foto, Video, Arquivo] {
            let n = efetivo(tipo, false, Nivel::LevarParaFora, Nivel::Nenhum);
            for uso in TODOS { assert!(!pode(tipo, n, uso, false)); }
        }
    }

    #[test]
    fn niveis_incluem_os_anteriores() {
        assert!(pode(Foto, Nivel::GuardarNoCofre, Uso::GuardarNoCofre, false) && !pode(Foto, Nivel::GuardarNoCofre, Uso::EncaminharNoNimbus, false));
        assert!(pode(Foto, Nivel::EncaminharNoNimbus, Uso::EncaminharNoNimbus, false) && !pode(Foto, Nivel::EncaminharNoNimbus, Uso::LevarParaFora, false));
        for uso in TODOS { assert!(pode(Foto, Nivel::LevarParaFora, uso, false)); }
    }

    #[test]
    fn visualizacao_unica_nunca_sai() {
        assert_eq!(efetivo(Foto, true, Nivel::LevarParaFora, Nivel::LevarParaFora), Nivel::Nenhum);
        assert_eq!(efetivo(Texto, true, Nivel::LevarParaFora, Nivel::EncaminharNoNimbus), Nivel::Nenhum);
    }

    #[test]
    fn teto_da_conversa_e_da_empresa_vencem_a_autorizacao() {
        // v37, terceira rodada: Blindada sempre ligada — o teto normal, sem limite de empresa, já é
        // EncaminharNoNimbus (nunca mais LevarParaFora), e a empresa só pode apertar ainda mais, nunca afrouxar.
        assert_eq!(efetivo(Foto, false, teto_da_conversa(None), Nivel::LevarParaFora), Nivel::EncaminharNoNimbus);
        assert_eq!(efetivo(Foto, false, teto_da_conversa(Some(Nivel::Nenhum)), Nivel::LevarParaFora), Nivel::Nenhum);
        assert_eq!(teto_da_conversa(None), Nivel::EncaminharNoNimbus, "sem empresa, o teto é sempre EncaminharNoNimbus — LevarParaFora nunca mais é alcançável");
    }

    #[test]
    fn autorizacao_do_autor_vale_e_adulterada_nao() {
        let autor = assinatura::gerar();
        let a = autorizar(&autor, H, Nivel::EncaminharNoNimbus, 1, None, false);
        assert_eq!(nivel_autorizado(&autor.verificacao(), std::slice::from_ref(&a), &H, Foto, Some(0), 1), Nivel::EncaminharNoNimbus);
        let mut subiu = a.clone();
        subiu.nivel = Nivel::LevarParaFora;
        assert_eq!(nivel_autorizado(&autor.verificacao(), &[subiu], &H, Foto, Some(0), 1), Nivel::Nenhum, "quem recebe não aumenta");
        let mut sem_prazo_forjado = autorizar(&autor, H, Nivel::LevarParaFora, 1, Some(60), false);
        sem_prazo_forjado.prazo_segundos = None;
        assert_eq!(nivel_autorizado(&autor.verificacao(), &[sem_prazo_forjado], &H, Foto, Some(0), 1), Nivel::Nenhum, "tirar o prazo invalida");
        let intruso = assinatura::gerar();
        assert_eq!(nivel_autorizado(&autor.verificacao(), &[autorizar(&intruso, H, Nivel::LevarParaFora, 9, None, false)], &H, Foto, Some(0), 1), Nivel::Nenhum);
        assert_eq!(nivel_autorizado(&autor.verificacao(), &[autorizar(&autor, [6; 32], Nivel::LevarParaFora, 1, None, false)], &H, Foto, Some(0), 1), Nivel::Nenhum);
    }

    #[test]
    fn prazo_conta_da_abertura_e_ao_fim_volta_a_nada() {
        let autor = assinatura::gerar();
        let a = [autorizar(&autor, H, Nivel::LevarParaFora, 1, Some(120), false)];
        let v = autor.verificacao();
        assert_eq!(nivel_autorizado(&v, &a, &H, Foto, None, 50), Nivel::Nenhum, "precisa abrir antes");
        assert_eq!(nivel_autorizado(&v, &a, &H, Foto, Some(1_000), 1_119), Nivel::LevarParaFora);
        assert_eq!(nivel_autorizado(&v, &a, &H, Foto, Some(1_000), 1_120), Nivel::Nenhum, "2 minutos depois, acabou");
    }

    #[test]
    fn item_some_ao_fim_se_o_autor_pediu() {
        let autor = assinatura::gerar();
        let some = autorizar(&autor, H, Nivel::EncaminharNoNimbus, 1, Some(30), true);
        assert!(!item_deve_sumir(&some, Some(100), 129) && item_deve_sumir(&some, Some(100), 130));
        let fica = autorizar(&autor, H, Nivel::EncaminharNoNimbus, 1, Some(30), false);
        assert!(!item_deve_sumir(&fica, Some(100), 999));
    }

    #[test]
    fn cartao_de_endereco_encaminhavel_por_prazo_dentro_do_nimbus() {
        let autor = assinatura::gerar();
        let v = autor.verificacao();
        let cartao = [autorizar(&autor, H, Nivel::LevarParaFora, 1, Some(120), true)];
        let n = efetivo(Localizacao, false, Nivel::LevarParaFora, nivel_autorizado(&v, &cartao, &H, Localizacao, Some(0), 60));
        assert!(pode(Localizacao, n, Uso::EncaminharNoNimbus, false));
        assert!(!pode(Localizacao, n, Uso::LevarParaFora, false) && !pode(Localizacao, n, Uso::GuardarNoCofre, false), "nunca para fora, nunca guardado");
        assert_eq!(nivel_autorizado(&v, &cartao, &H, Localizacao, Some(0), 120), Nivel::Nenhum, "acabou o prazo");
    }

    #[test]
    fn mensagem_so_vira_cartao_no_envio_e_com_prazo_de_ate_4_horas() {
        let autor = assinatura::gerar();
        let v = autor.verificacao();
        let depois = [autorizar(&autor, H, Nivel::EncaminharNoNimbus, 2, Some(60), false)];
        assert_eq!(nivel_autorizado(&v, &depois, &H, Texto, Some(0), 1), Nivel::Nenhum, "sem liberação no envio, nada");
        let subiu_depois = [autorizar(&autor, H, Nivel::Nenhum, 1, Some(60), false), autorizar(&autor, H, Nivel::EncaminharNoNimbus, 2, Some(60), false)];
        assert_eq!(nivel_autorizado(&v, &subiu_depois, &H, Texto, Some(0), 1), Nivel::Nenhum, "depois de enviado, só reduz");
        let sem_prazo = [autorizar(&autor, H, Nivel::EncaminharNoNimbus, 1, None, false)];
        assert_eq!(nivel_autorizado(&v, &sem_prazo, &H, Texto, Some(0), 1), Nivel::Nenhum, "cartão exige prazo");
        let longo = [autorizar(&autor, H, Nivel::EncaminharNoNimbus, 1, Some(PRAZO_MAXIMO_DO_CARTAO + 1), false)];
        assert_eq!(nivel_autorizado(&v, &longo, &H, Texto, Some(0), 1), Nivel::Nenhum, "no máximo 4 horas");
        let voz = [autorizar(&autor, H, Nivel::EncaminharNoNimbus, 1, Some(60), false)];
        assert_eq!(nivel_autorizado(&v, &voz, &H, Audio, Some(0), 1), Nivel::Nenhum, "voz nunca vira cartão");
    }

    #[test]
    fn arquivo_pode_ser_liberado_depois_do_envio_e_revogado() {
        let autor = assinatura::gerar();
        let v = autor.verificacao();
        let depois = [autorizar(&autor, H, Nivel::Nenhum, 1, None, false), autorizar(&autor, H, Nivel::GuardarNoCofre, 2, None, false)];
        assert_eq!(nivel_autorizado(&v, &depois, &H, Foto, Some(0), 1), Nivel::GuardarNoCofre);
        let revogou = [autorizar(&autor, H, Nivel::LevarParaFora, 1, None, false), autorizar(&autor, H, Nivel::Nenhum, 2, None, false)];
        assert_eq!(nivel_autorizado(&v, &revogou, &H, Foto, Some(0), 1), Nivel::Nenhum);
    }

    #[test]
    fn um_salto_so_o_que_chegou_encaminhado_nao_se_encaminha_de_novo() {
        assert!(!pode(Foto, Nivel::LevarParaFora, Uso::EncaminharNoNimbus, true));
        assert!(!pode(Texto, Nivel::EncaminharNoNimbus, Uso::EncaminharNoNimbus, true));
        assert!(pode(Foto, Nivel::LevarParaFora, Uso::GuardarNoCofre, true), "guardar continua valendo");
        assert_eq!(ao_encaminhar(Nivel::GuardarNoCofre, Nivel::LevarParaFora), Nivel::GuardarNoCofre);
    }

    #[test]
    fn autor_e_avisado_quando_o_conteudo_se_move() {
        // 10/10/2026: GuardarNoCofre agora TAMBÉM avisa (correção da assimetria com GuardarEmMinhaPasta,
        // ver doc de `avisar_o_autor`) — a lista completa dos quatro usos que tiram o conteúdo do lugar
        // onde foi enviado.
        assert!(avisar_o_autor(Uso::EncaminharNoNimbus) && avisar_o_autor(Uso::LevarParaFora) && avisar_o_autor(Uso::GuardarNoCofre));
    }

    #[test]
    fn guardar_no_cofre_e_guardar_em_minha_pasta_avisam_igual_por_exigirem_o_mesmo_nivel() {
        // Arquivo (GuardarNoCofre) e mensagem (GuardarEmMinhaPasta) são o MESMO movimento — guardar para
        // si mesmo, além da conversa — só com nomes diferentes por tipo de conteúdo (ver `exigido`); por
        // isso avisar o autor não pode tratar os dois de forma diferente.
        assert_eq!(avisar_o_autor(Uso::GuardarNoCofre), avisar_o_autor(Uso::GuardarEmMinhaPasta));
    }

    #[test]
    fn saida_para_fora_exige_autorizacao_e_limpa_foto() {
        assert_eq!(preparar_para_fora(Foto, Nivel::EncaminharNoNimbus, b"x").unwrap_err(), Recusa::NaoAutorizado);
        assert_eq!(preparar_para_fora(Texto, Nivel::LevarParaFora, b"x").unwrap_err(), Recusa::NaoAutorizado, "mensagem nunca sai");
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x0A];
        jpeg.extend_from_slice(b"GPS-SEGR");
        jpeg.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x04, 0x01, 0x00, 0x11, 0xFF, 0xD9]);
        let e = preparar_para_fora(Foto, Nivel::LevarParaFora, &jpeg).unwrap();
        assert!(e.metadados_limpos && !e.dados.windows(3).any(|w| w == b"GPS"));
        assert!(!preparar_para_fora(Arquivo, Nivel::LevarParaFora, b"%PDF-1.7 /Author (x)").unwrap().metadados_limpos);
    }

    // ---- RV4: "Minha Pasta" exige a mesma autorização assinada do autor que o cartão encaminhável ----

    #[test]
    fn mensagem_nao_vai_para_minha_pasta_sem_autorizacao() {
        for tipo in [Texto, Audio, VideoCircular, Figurinha, Localizacao, LocalizacaoAoVivo] {
            let n = efetivo(tipo, false, Nivel::LevarParaFora, Nivel::Nenhum);
            assert!(!pode(tipo, n, Uso::GuardarEmMinhaPasta, false), "{tipo:?}");
        }
    }

    #[test]
    fn localizacao_ao_vivo_nunca_vira_cartao_nem_vai_para_minha_pasta_mesmo_com_autorizacao() {
        // Diferente da localização de um instante só (que pode virar cartão): aqui a autorização máxima
        // não adianta, porque `pode_ser_cartao` exclui este tipo de propósito (RV7) — mesmo com
        // `nivel_autorizado` no máximo, `efetivo()` barra em Nivel::Nenhum por não ser cartão.
        assert!(!pode_ser_cartao(LocalizacaoAoVivo));
        let ef = efetivo(LocalizacaoAoVivo, false, Nivel::LevarParaFora, Nivel::EncaminharNoNimbus);
        assert_eq!(ef, Nivel::Nenhum);
        assert!(!pode(LocalizacaoAoVivo, ef, Uso::EncaminharNoNimbus, false));
        assert!(!pode(LocalizacaoAoVivo, ef, Uso::GuardarEmMinhaPasta, false));
    }

    #[test]
    fn mensagem_autorizada_como_cartao_tambem_pode_ir_para_minha_pasta() {
        let autor = assinatura::gerar();
        let v = autor.verificacao();
        let cartao = [autorizar(&autor, H, Nivel::EncaminharNoNimbus, 1, Some(120), false)];
        let n = nivel_autorizado(&v, &cartao, &H, Texto, Some(0), 60);
        let ef = efetivo(Texto, false, Nivel::LevarParaFora, n);
        assert!(pode(Texto, ef, Uso::GuardarEmMinhaPasta, false), "a mesma autorização de cartão libera guardar na Minha Pasta");
    }

    #[test]
    fn minha_pasta_nao_sofre_a_regra_de_um_salto_so() {
        // Diferente de encaminhar de novo: guardar para SI MESMO não é "espalhar" a mensagem.
        assert!(pode(Texto, Nivel::EncaminharNoNimbus, Uso::GuardarEmMinhaPasta, true));
    }

    #[test]
    fn autor_e_avisado_quando_vai_para_minha_pasta() {
        assert!(avisar_o_autor(Uso::GuardarEmMinhaPasta));
    }

    #[test]
    fn minha_pasta_tambem_e_bloqueada_por_voz_e_video_circular() {
        // Mesma regra do cartão: voz e vídeo circular identificam quem gravou e nunca podem virar cartão,
        // logo nunca vão à Minha Pasta. O bloqueio é em `efetivo()` (nunca chegam a Nivel::EncaminharNoNimbus).
        for tipo in [Audio, VideoCircular] {
            let ef = efetivo(tipo, false, Nivel::LevarParaFora, Nivel::EncaminharNoNimbus);
            assert_eq!(ef, Nivel::Nenhum, "{tipo:?}");
            assert!(!pode(tipo, ef, Uso::GuardarEmMinhaPasta, false), "{tipo:?}");
        }
    }

    #[test]
    fn figurinha_pode_virar_cartao_e_ir_para_minha_pasta_com_autorizacao() {
        // Diferente de voz e vídeo circular: figurinha é um recurso genérico, sem conteúdo pessoal do
        // autor, então PODE virar cartão — com a mesma autorização assinada e o mesmo teto de 4 horas.
        let ef = efetivo(Figurinha, false, Nivel::LevarParaFora, Nivel::EncaminharNoNimbus);
        assert_eq!(ef, Nivel::EncaminharNoNimbus);
        assert!(pode(Figurinha, ef, Uso::EncaminharNoNimbus, false));
        assert!(pode(Figurinha, ef, Uso::GuardarEmMinhaPasta, false));
        // Sem autorização nenhuma, continua bloqueada como qualquer outro tipo.
        let sem_autorizacao = efetivo(Figurinha, false, Nivel::LevarParaFora, Nivel::Nenhum);
        assert_eq!(sem_autorizacao, Nivel::Nenhum);
    }

    #[test]
    fn prazos_oferecidos_na_tela() {
        assert_eq!(PRAZOS, [30, 60, 120, 3_600, 7_200, 14_400]);
        assert_eq!(*PRAZOS.last().unwrap(), PRAZO_MAXIMO_DO_CARTAO);
    }

    // ---- RV5: três camadas de autorização de encaminhamento (Master / contato / mensagem) ----

    #[test]
    fn sem_nenhuma_camada_especifica_vale_o_master() {
        assert_eq!(resolver_nivel_de_encaminhamento(Nivel::Nenhum, None, None, Nivel::LevarParaFora), Nivel::Nenhum);
        assert_eq!(resolver_nivel_de_encaminhamento(Nivel::EncaminharNoNimbus, None, None, Nivel::LevarParaFora), Nivel::EncaminharNoNimbus);
    }

    #[test]
    fn padrao_do_contato_vence_o_master() {
        assert_eq!(resolver_nivel_de_encaminhamento(Nivel::Nenhum, Some(Nivel::EncaminharNoNimbus), None, Nivel::LevarParaFora), Nivel::EncaminharNoNimbus);
        assert_eq!(resolver_nivel_de_encaminhamento(Nivel::EncaminharNoNimbus, Some(Nivel::Nenhum), None, Nivel::LevarParaFora), Nivel::Nenhum, "o contato também pode ser mais restritivo que o Master");
    }

    #[test]
    fn escolha_na_mensagem_vence_tudo_o_mais() {
        assert_eq!(resolver_nivel_de_encaminhamento(Nivel::Nenhum, Some(Nivel::Nenhum), Some(Nivel::EncaminharNoNimbus), Nivel::LevarParaFora), Nivel::EncaminharNoNimbus);
        assert_eq!(resolver_nivel_de_encaminhamento(Nivel::EncaminharNoNimbus, Some(Nivel::EncaminharNoNimbus), Some(Nivel::Nenhum), Nivel::LevarParaFora), Nivel::Nenhum, "desligar na própria mensagem sempre vale");
    }

    #[test]
    fn nenhuma_camada_passa_do_teto_da_conversa() {
        assert_eq!(resolver_nivel_de_encaminhamento(Nivel::LevarParaFora, Some(Nivel::LevarParaFora), Some(Nivel::LevarParaFora), Nivel::GuardarNoCofre), Nivel::GuardarNoCofre, "conversa blindada limita mesmo que todas as camadas liberem mais");
    }

    // ---- RV? (11/10/2026): os três destinos da tela única de liberação ----

    #[test]
    fn so_fora_do_nimbus_mostra_o_aviso_de_sair_do_dominio() {
        assert!(!sai_do_dominio_nimbus(Destino::AbaTrancada));
        assert!(!sai_do_dominio_nimbus(Destino::CofreNimbusOne));
        assert!(sai_do_dominio_nimbus(Destino::ForaDoNimbus));
    }

    #[test]
    fn aba_trancada_nao_tem_uso_correspondente_os_outros_dois_tem() {
        assert_eq!(uso_correspondente(Destino::AbaTrancada), None, "mecanismo próprio de custodia_mutua, regra 34 — nunca uma Autorizacao");
        assert_eq!(uso_correspondente(Destino::CofreNimbusOne), Some(Uso::GuardarNoCofre));
        assert_eq!(uso_correspondente(Destino::ForaDoNimbus), Some(Uso::LevarParaFora));
    }

    #[test]
    fn fora_do_nimbus_fica_estruturalmente_bloqueado_pelo_teto_permanente_da_conversa() {
        // v37, terceira rodada: o teto de QUALQUER conversa, sem limite de empresa, já é EncaminharNoNimbus
        // — nunca mais LevarParaFora. Este teste prova, de ponta a ponta, que o terceiro botão da tela
        // ("outros aplicativos") nunca produz uma liberação de verdade enquanto essa decisão permanente
        // valer — mesmo que o autor "aprove" (ver pedido_de_guarda::responder_para_destino).
        let teto_normal = teto_da_conversa(None);
        assert!(destino_bloqueado_pelo_teto(Destino::ForaDoNimbus, teto_normal), "ForaDoNimbus sempre bloqueado pelo teto atual — decisão permanente, não um bug");
        assert!(!destino_bloqueado_pelo_teto(Destino::CofreNimbusOne, teto_normal), "cofre fica abaixo do teto, nunca bloqueado por ele");
        assert!(!destino_bloqueado_pelo_teto(Destino::AbaTrancada, teto_normal), "aba trancada não usa este teto — mecanismo próprio");
        // Mesmo um autor que "aprovasse" LevarParaFora (Uso::LevarParaFora assinado) continua sem efeito:
        // efetivo() sempre aplica o teto por cima do que foi autorizado.
        assert_eq!(efetivo(TipoDeConteudo::Foto, false, teto_normal, Nivel::LevarParaFora), Nivel::EncaminharNoNimbus);
        assert!(!pode(TipoDeConteudo::Foto, efetivo(TipoDeConteudo::Foto, false, teto_normal, Nivel::LevarParaFora), Uso::LevarParaFora, false));
    }

    #[test]
    fn empresa_mais_restritiva_bloqueia_tambem_o_cofre() {
        // Prova que destino_bloqueado_pelo_teto reage ao teto de verdade, não é um `match` fixo por destino.
        let teto_restrito = teto_da_conversa(Some(Nivel::Nenhum));
        assert!(destino_bloqueado_pelo_teto(Destino::CofreNimbusOne, teto_restrito));
        assert!(destino_bloqueado_pelo_teto(Destino::ForaDoNimbus, teto_restrito));
    }
}
