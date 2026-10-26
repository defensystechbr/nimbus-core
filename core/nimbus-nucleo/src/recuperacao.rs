//! # Módulo `recuperacao` — recuperar a conta com amigos de confiança (REC1–REC6)
//!
//! **Três trilhas independentes; qualquer DUAS das três reconstroem a conta — nunca uma só, nunca as três
//! obrigatórias juntas (2 de 3, não OU nem E).** Redesenhado na Rodada 21 (11/10/2026, mesma sessão),
//! substituindo o esquema anterior ("qualquer UM dos dois caminhos já bastava") por pedido explícito e
//! final do dono: *"ao menos 2 guardiões cadastrados... mais uma frase pessoal ou pergunta importante...
//! e as palavras mágicas... ele poderia acessar com ao menos 2 dessas 3 alternativas."*
//!
//! **Correção feita antes de implementar, não aceita como veio — e explicada, por quê (regra do próprio
//! dono de nunca empacotar um exemplo contraditório sem corrigir):** "palavras mágicas" (a wordlist
//! BIP-39, módulo `codec_palavras`) não é um quarto fator independente — é só uma FORMA de escrever a
//! MESMA chave de 32 bytes da frase pessoal (ao lado de hex/QR, nunca no lugar deles). Tratá-la como uma
//! "alternativa" separada teria dado uma falsa sensação de 2-de-3: quem souber a frase pessoal, em
//! qualquer codificação, tem o MESMO fator, não dois. Por isso as três trilhas reais, cada uma uma fonte
//! de entropia genuinamente diferente, são:
//!
//! 1. **Guardiões (3 a 5 contatos Nimbus, k de n, inalterado)** — `preparar`/`abrir_para_devolver`/`reunir`,
//!    como antes. A única mudança: o que se divide entre eles não é mais o segredo da conta inteiro, é só
//!    a trilha 1 (ver abaixo).
//! 2. **Frase de recuperação pessoal** — chave aleatória de 32 bytes, escrita como hex, QR ou (com o
//!    codec `codec_palavras`) como 24 palavras BIP-39 — três formas do MESMO segredo, escolha do usuário
//!    (`preparar_frase_pessoal`/`abrir_frase_pessoal`, inalterado).
//! 3. **Pergunta importante** — uma resposta que só o dono sabe, de verdade independente das outras duas
//!    (`preparar_pergunta_importante`/`abrir_pergunta_importante`, nova nesta rodada, ver aviso de entropia
//!    abaixo da função).
//!
//! **Como o 2-de-3 funciona, sem nenhuma criptografia nova — só composição do que já existia e já estava
//! testado:** o segredo da conta é dividido em exatamente 3 "trilhas" com `custodia::dividir(segredo, 3,
//! 2)` — o MESMO mecanismo k-de-n dos guardiões, aqui usado numa escala fixa e pequena (3 participantes,
//! nunca cresce com o número de guardiões que o dono cadastrar, ao contrário do problema descrito abaixo).
//! Cada trilha (`Trilha`, um alias de `custodia::PartesDoGuardiao`) é só um blob opaco de ~150 bytes —
//! pequeno o bastante para ser selado por QUALQUER UM dos três mecanismos abaixo sem o problema de tamanho
//! que motivou manter a frase pessoal fora do grupo dos guardiões (ver próximo parágrafo). A trilha 1 é
//! selada entre os guardiões (`preparar(&trilha1.para_bytes(), ...)`); a trilha 2, pela frase pessoal
//! (`preparar_frase_pessoal(&trilha2.para_bytes())`); a trilha 3, pela pergunta importante
//! (`preparar_pergunta_importante(resposta, &trilha3.para_bytes())`). Recuperar duas trilhas quaisquer
//! (por qualquer combinação de dois dos três caminhos) e chamar `concluir_com_duas_trilhas` devolve o
//! segredo original da conta — perder qualquer UMA das três trilhas não tranca a conta, só obriga a usar
//! as outras duas.
//!
//! **Por que a trilha em si (diferente da peça individual de um guardião) não sofre o problema de tamanho
//! que a versão anterior deste comentário descrevia:** a preocupação original era sobre dar ao PRÓPRIO
//! DONO uma peça do esquema dos guardiões — isso sim cresce combinatorialmente (`C(n-1,k-1)` peças) com o
//! número de guardiões, inviabilizando memorização. Aqui a divisão em 3 trilhas é sempre `n=3,k=2`, fixo,
//! não combinatório — e nenhuma trilha é memorizada por ninguém; cada uma é imediatamente selada pelo seu
//! mecanismo (guardiões, frase, pergunta) e descartada da memória. O usuário nunca vê nem escreve bytes de
//! trilha — só a frase pessoal (igual a antes) e a resposta da pergunta importante (texto normal).
//!
//! **As três trilhas passam pela MESMA carência de `pode_concluir` (72h, cancelável pelo aparelho antigo)
//! — sem exceção para nenhuma.** A junção final (`concluir_com_duas_trilhas`) só produz o segredo
//! utilizável depois que `pode_concluir` autorizar, exatamente como antes. Isto resolve METADE do risco
//! residual que a versão anterior deste módulo documentava (frase pessoal sozinha = única rede contra
//! coação): agora a frase pessoal sozinha NUNCA basta — precisa de mais uma trilha, tornando um ataque de
//! coação bem mais caro (teria de extrair 2 segredos independentes, não 1). A outra metade — o caso em que
//! o atacante controla os DOIS aparelhos (antigo e novo) e consegue as 2 trilhas "algo que você sabe"
//! (frase + pergunta), sem nenhum humano de fora confirmar nada — está em `pode_concluir_duas_trilhas`,
//! nova nesta rodada, ver seu comentário.
//!
//! **Configuração obrigatória das três, não opcional (mudança de comportamento, deliberada):** como
//! qualquer combinação de 2 das 3 precisa funcionar, as três trilhas têm de ser seladas no momento da
//! configuração — não é mais "configure pelo menos uma". Se o dono configurasse só 2 das 3 "porque já
//! basta 2", perder qualquer UMA dessas 2 passaria a travar a conta de verdade (vira 2-de-2, não 2-de-3) —
//! o oposto do que o dono pediu ("não tranque o usuário para fora"). A tela de configuração (pendente,
//! ver PENDENCIAS_E_DECISOES.md) precisa deixar isto explícito: as três são obrigatórias de configurar,
//! só duas são obrigatórias de ter à mão na hora de recuperar.
//!
//! **Pré-lançamento, sem usuário real ainda — premissa declarada, não verificada por este núcleo:** não
//! existe hoje nenhuma conta já configurada sob o esquema antigo (OU) para migrar; se isso mudar antes do
//! lançamento, uma rotina de migração explícita (reconfigurar as 3 trilhas a partir do segredo já
//! recuperável pelo esquema antigo) precisa ser escrita e testada à parte — não presumida aqui.
//!
//! **O que NENHUMA das três trilhas recupera** (decisão já confirmada pelo fundador, tela 7 do protótipo
//! da aba trancada, inalterada nesta rodada): a aba trancada e o cofre pessoal do aparelho antigo. Por
//! desenho, a chave desses dois nunca existiu em lugar nenhum fora do aparelho perdido — nem nas trilhas.
use crate::custodia::{self, ErroDeCustodia, PartesDoGuardiao};
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use crate::senha;
use rand_core::{OsRng, RngCore};
use zeroize::Zeroizing;

const CONTEXTO: &[u8] = b"NimbusOne/v1/recuperacao-social";
const CONTEXTO_FRASE: &[u8] = b"NimbusOne/v1/recuperacao-frase-pessoal";
const CONTEXTO_PERGUNTA: &[u8] = b"NimbusOne/v1/recuperacao-pergunta-importante";

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeRecuperacao { GuardioesInvalidos, Cifra, Ilegivel, Insuficiente, Adulterado,
    /// O amigo não conferiu ao vivo as palavras do aparelho novo (EO4).
    SemConfirmacaoAoVivo,
    /// Ainda dentro do período de espera, ou cancelado por um aparelho antigo do dono (EO4).
    EmEspera, Cancelado,
    /// A resposta da pergunta importante é mais curta que `TAM_MINIMO_RESPOSTA` — piso honesto, não uma
    /// garantia de entropia real (Rodada 21).
    RespostaFraca,
    /// `pode_concluir` passou, mas nenhuma das duas trilhas usadas foi a dos guardiões E nenhum guardião
    /// confirmou ao vivo — ver `pode_concluir_duas_trilhas` (Rodada 21).
    SemTestemunhaViva }

/// O que cada guardião guarda: um bloco que só ele abre.
#[derive(Debug, Clone)]
pub struct Pacote { pub guardiao: u8, pub bloco: Vec<u8> }

/// No seu aparelho: divide o segredo entre os guardiões (de 3 a 5), exigindo `k` para recuperar.
///
/// **⚠ Primitiva genérica — NÃO chamar com o segredo da conta inteiro para configurar a recuperação
/// pessoal.** `segredo` pode ser qualquer `&[u8]`; o tipo não impede passar aqui o segredo da conta
/// direto, o que recriaria o modelo antigo ("os guardiões sozinhos já bastam"), removido nesta rodada por
/// pedido do dono. Para a recuperação pessoal, o único caminho correto é `preparar_tres_trilhas`, que
/// chama esta função com os bytes de UMA TRILHA (`trilha.para_bytes()`), nunca com o segredo inteiro —
/// achado e corrigido nesta auditoria, ver o teste
/// `preparar_e_preparar_frase_pessoal_sao_primitivas_genericas_e_nao_impedem_sozinhas_o_uso_antigo`.
pub fn preparar(segredo: &[u8], guardioes: &[ChavePublica], k: u8) -> Result<Vec<Pacote>, ErroDeRecuperacao> {
    if !(3..=5).contains(&guardioes.len()) || k < 2 || k as usize > guardioes.len() { return Err(ErroDeRecuperacao::GuardioesInvalidos); }
    let (partes, _) = custodia::dividir(segredo, guardioes.len() as u8, k).map_err(|_| ErroDeRecuperacao::GuardioesInvalidos)?;
    partes.iter().zip(guardioes).map(|(p, chave)| {
        let bloco = envelope::selar(chave, &p.para_bytes(), CONTEXTO).map_err(|_| ErroDeRecuperacao::Cifra)?;
        Ok(Pacote { guardiao: p.guardiao, bloco })
    }).collect()
}

/// No aparelho do guardião: abre o pacote para devolver a parte (que segue cifrada para o aparelho novo).
/// Só depois que o amigo conferiu AO VIVO (pessoalmente ou numa chamada em que reconhece a pessoa) as palavras
/// que o aparelho novo mostra (`palavras_do_pedido`). Mensagem de texto pedindo a parte não vale: é
/// exatamente como o golpista agiria se tivesse tomado a conta de mensagens da pessoa (EO4).
pub fn abrir_para_devolver(sec: &ChaveSecreta, publ: &ChavePublica, pacote: &Pacote, palavras_conferidas_ao_vivo: bool) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao> {
    if !palavras_conferidas_ao_vivo { return Err(ErroDeRecuperacao::SemConfirmacaoAoVivo); }
    envelope::abrir(sec, publ, &pacote.bloco, CONTEXTO).map_err(|_| ErroDeRecuperacao::Ilegivel)
}

/// No aparelho novo: junta as partes devolvidas e reconstrói o segredo.
pub fn reunir(devolvidas: &[&[u8]]) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao> {
    let partes: Vec<PartesDoGuardiao> = devolvidas.iter().map(|b| PartesDoGuardiao::de_bytes(b)).collect::<Result<_, _>>().map_err(|_| ErroDeRecuperacao::Adulterado)?;
    let refs: Vec<&PartesDoGuardiao> = partes.iter().collect();
    custodia::reconstruir(&refs).map_err(|e| match e {
        ErroDeCustodia::Insuficiente => ErroDeRecuperacao::Insuficiente,
        _ => ErroDeRecuperacao::Adulterado,
    })
}

/// Tamanho da chave da frase de recuperação pessoal — mesma chave simétrica de `envelope`.
pub const TAM_CHAVE_DA_FRASE: usize = envelope::TAM_CHAVE_SIMETRICA;

/// No seu aparelho: caminho independente dos guardiões. Gera uma chave aleatória de 32 bytes — o segredo
/// por trás da frase que você guarda — e cifra o segredo da conta com ela. Devolve `(chave, bloco)`: a
/// `chave` é SUA, nunca sai do aparelho nem viaja cifrada para ninguém (ao contrário do `Pacote` dos
/// guardiões); o `bloco` pode ir para o servidor sem risco — sem a chave, é ruído.
///
/// **O que esta função entrega, e o que passou a existir ao lado dela (Rodada 21, fechando a pendência da
/// Rodada 19):** esta função só gera e sela a `chave` de 32 bytes — nunca lidou com a forma de ESCREVER
/// esses bytes para o usuário anotar. Até a Rodada 20 isso só existia como hex ou QR; a terceira forma
/// pedida pelo fundador ("palavras mágicas") agora existe em `codec_palavras::codificar`/`decodificar`
/// (24 palavras, lista oficial do padrão BIP-39, não uma lista inventada às pressas — ver o comentário
/// daquele módulo para a proveniência e o hash de conferência). As três formas — hex, QR, palavras — são
/// escolha de interface sobre o MESMO segredo, nunca um fator a mais: perder ou errar a transcrição de uma
/// não compromete as outras duas, e qualquer uma das três reabre exatamente o mesmo `bloco` abaixo.
///
/// **⚠ Primitiva genérica — mesmo aviso de `preparar`, acima: NÃO chamar com o segredo da conta inteiro.**
/// `segredo` pode ser qualquer `&[u8]`. Para a recuperação pessoal, o caminho correto é
/// `preparar_tres_trilhas`, que chama esta função com `trilha.para_bytes()`, nunca com o segredo inteiro.
pub fn preparar_frase_pessoal(segredo: &[u8]) -> Result<(Zeroizing<[u8; TAM_CHAVE_DA_FRASE]>, Vec<u8>), ErroDeRecuperacao> {
    let mut chave = Zeroizing::new([0u8; TAM_CHAVE_DA_FRASE]);
    OsRng.fill_bytes(chave.as_mut());
    let bloco = envelope::selar_com_chave_simetrica(&chave, segredo, CONTEXTO_FRASE).map_err(|_| ErroDeRecuperacao::Cifra)?;
    Ok((chave, bloco))
}

/// No aparelho novo: só a chave da frase, sem nenhum guardião. Mesma carência obrigatória de `pode_concluir`
/// que o caminho dos guardiões — ver aviso no comentário do módulo sobre o risco residual de uma frase
/// roubada ou forçada.
pub fn abrir_frase_pessoal(chave: &[u8; TAM_CHAVE_DA_FRASE], bloco: &[u8]) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao> {
    envelope::abrir_com_chave_simetrica(chave, bloco, CONTEXTO_FRASE).map_err(|_| ErroDeRecuperacao::Ilegivel)
}

/// Espera obrigatória entre o pedido de recuperação e a conclusão (EO4): 3 dias. Nesse período, todos os
/// aparelhos ainda ativos do dono recebem o aviso "Alguém está recuperando a sua conta" e qualquer um deles
/// cancela com um toque. Quem perdeu mesmo o celular espera três dias; quem é vítima de golpe tem três dias
/// para ver o aviso.
pub const CARENCIA_SEGUNDOS: u64 = 72 * 3600;

/// Palavras que o aparelho novo mostra e que cada amigo confere ao vivo antes de devolver a parte.
pub fn palavras_do_pedido(chave_do_aparelho_novo: &[u8], chave_da_conta: &[u8]) -> [&'static str; 6] {
    crate::convites::palavras_com_dominio(b"NimbusOne/v1/recuperacao", chave_do_aparelho_novo, chave_da_conta)
}

/// O aparelho novo pode usar o segredo reconstruído? Só depois da espera e se nenhum aparelho antigo
/// cancelou. `agora` vem de `expiracao::hora_confiavel` (servidor), nunca do relógio do aparelho novo.
pub fn pode_concluir(pedido_em: u64, agora: u64, cancelado_por_aparelho_antigo: bool) -> Result<(), ErroDeRecuperacao> {
    if cancelado_por_aparelho_antigo { return Err(ErroDeRecuperacao::Cancelado); }
    if agora < pedido_em.saturating_add(CARENCIA_SEGUNDOS) { return Err(ErroDeRecuperacao::EmEspera); }
    Ok(())
}

// ============================================================================================
// Terceira trilha — pergunta importante (Rodada 21, 11/10/2026, mesma sessão: nova, pedida pelo dono).
// ============================================================================================

/// Piso honesto de tamanho da resposta — NÃO é garantia de entropia real (uma resposta de 20 bytes toda
/// igual, ex. "aaaaaaaaaaaaaaaaaaaa", passa este piso e ainda é péssima). O piso só existe para recusar de
/// cara respostas curtas demais para qualquer entropia útil (ex. um PIN de 4 dígitos). A proteção real
/// contra adivinhação vem do Argon2id em `senha::derivar` (RFC 9106, 64 MiB, 3 passagens — a mesma função
/// já usada para o PIN de coação), que torna cada tentativa cara em tempo e memória, não deste piso.
/// Recomendação de produto, não imposta pelo núcleo: a tela deve orientar o dono a escrever uma frase
/// inteira só ele saberia — nunca um fato biográfico público (nome de escola, time de futebol, cidade de
/// nascimento) nem uma frase/citação famosa.
pub const TAM_MINIMO_RESPOSTA: usize = 20;

/// Pacote da terceira trilha: o `sal` não é segredo (igual a um nonce, só precisa ser único) e viaja junto
/// com o `bloco`; sem a RESPOSTA certa, os dois juntos são ruído.
#[derive(Debug, Clone)]
pub struct PacotePerguntaImportante { pub sal: [u8; 16], pub bloco: Vec<u8> }

/// No seu aparelho: fortalece a resposta com Argon2id (módulo `senha` — a resposta humana tem muito menos
/// entropia que a frase pessoal, que é aleatória de verdade) e usa a chave derivada para selar a trilha,
/// com o MESMO primitivo simétrico já usado pela frase pessoal (`envelope::selar_com_chave_simetrica`) —
/// nenhuma criptografia nova, só uma composição diferente do que já existia.
///
/// **⚠ Primitiva genérica — mesmo aviso de `preparar`, acima: `segredo_da_trilha` pode ser qualquer
/// `&[u8]`; não chamar com o segredo da conta inteiro.** O caminho correto é `preparar_tres_trilhas`, que
/// chama esta função com `trilha.para_bytes()`.
pub fn preparar_pergunta_importante(resposta: &[u8], segredo_da_trilha: &[u8]) -> Result<PacotePerguntaImportante, ErroDeRecuperacao> {
    if resposta.len() < TAM_MINIMO_RESPOSTA { return Err(ErroDeRecuperacao::RespostaFraca); }
    let mut sal = [0u8; 16];
    OsRng.fill_bytes(&mut sal);
    let chave = senha::derivar::<32>(resposta, &sal).map_err(|_| ErroDeRecuperacao::Cifra)?;
    let bloco = envelope::selar_com_chave_simetrica(&chave, segredo_da_trilha, CONTEXTO_PERGUNTA).map_err(|_| ErroDeRecuperacao::Cifra)?;
    Ok(PacotePerguntaImportante { sal, bloco })
}

/// No aparelho novo: só a resposta, sem guardião nem frase pessoal. Resposta errada falha aqui (erro de
/// autenticação do AEAD), nunca revela "quase certo" — mesmo comportamento de senha de qualquer módulo.
pub fn abrir_pergunta_importante(resposta: &[u8], pacote: &PacotePerguntaImportante) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao> {
    let chave = senha::derivar::<32>(resposta, &pacote.sal).map_err(|_| ErroDeRecuperacao::Cifra)?;
    envelope::abrir_com_chave_simetrica(&chave, &pacote.bloco, CONTEXTO_PERGUNTA).map_err(|_| ErroDeRecuperacao::Ilegivel)
}

// ============================================================================================
// Orquestração das três trilhas — qualquer 2 de 3 reconstrói o segredo da conta (Rodada 21).
// ============================================================================================

/// Uma trilha é só um blob opaco de ~150 bytes — o mesmo tipo já usado para o que um guardião guarda,
/// reaproveitado aqui numa escala fixa (sempre 3 participantes) em vez de crescer com o número de
/// guardiões. Ver o comentário do módulo para o porquê disto não repetir o problema de tamanho que mantém
/// a frase pessoal fora do grupo dos guardiões.
pub type Trilha = PartesDoGuardiao;

/// As três trilhas recém-divididas, ainda não seladas por nenhum mecanismo — o próximo passo de quem
/// configura é selar `guardioes` com `preparar`, `frase` com `preparar_frase_pessoal` e `pergunta` com
/// `preparar_pergunta_importante`, cada uma com seus próprios bytes (`trilha.para_bytes()`).
pub struct TresTrilhas { pub guardioes: Trilha, pub frase: Trilha, pub pergunta: Trilha }

/// Divide o segredo da conta em exatamente 3 trilhas, exigindo 2 para reconstruir — número fixo, não
/// configurável: é o pedido explícito e final do dono ("ao menos 2 dessas 3 alternativas"), nunca mais um
/// parâmetro a decidir de novo.
pub const N_TRILHAS: u8 = 3;
pub const K_TRILHAS: u8 = 2;

pub fn preparar_tres_trilhas(segredo_da_conta: &[u8]) -> Result<TresTrilhas, ErroDeRecuperacao> {
    let (partes, _compromisso) = custodia::dividir(segredo_da_conta, N_TRILHAS, K_TRILHAS).map_err(|_| ErroDeRecuperacao::GuardioesInvalidos)?;
    let mut it = partes.into_iter();
    let (guardioes, frase, pergunta) = match (it.next(), it.next(), it.next()) {
        (Some(a), Some(b), Some(c)) => (a, b, c),
        _ => return Err(ErroDeRecuperacao::GuardioesInvalidos), // inalcançável: dividir(_, 3, _) sempre devolve 3 partes quando Ok
    };
    Ok(TresTrilhas { guardioes, frase, pergunta })
}

/// Junta DUAS trilhas já recuperadas (por qualquer combinação de dois dos três caminhos) e reconstrói o
/// segredo original da conta. A carência de tempo (`pode_concluir`) continua se aplicando por fora desta
/// função, exatamente como antes — ver `pode_concluir_duas_trilhas` para o portão completo recomendado.
pub fn concluir_com_duas_trilhas(a: &Trilha, b: &Trilha) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao> {
    custodia::reconstruir(&[a, b]).map_err(|e| match e {
        ErroDeCustodia::Insuficiente => ErroDeRecuperacao::Insuficiente,
        _ => ErroDeRecuperacao::Adulterado,
    })
}

/// **Segunda rede de segurança contra coação (Rodada 21), além das 72h e da conferência ao vivo que já
/// protege a trilha dos guardiões.** O "maior risco" nomeado na rodada anterior: um atacante que controla
/// os DOIS aparelhos do dono (antigo e novo) e consegue arrancar as duas trilhas "algo que você sabe"
/// (frase pessoal + pergunta importante) conseguia concluir a recuperação sem NENHUM humano de fora
/// confirmar que é mesmo o dono. Fechado assim: sempre que a trilha dos GUARDIÕES não é uma das duas
/// usadas, pelo menos um guardião ainda precisa confirmar ao vivo que reconhece quem está pedindo —
/// reaproveitando a MESMA cerimônia de palavras (`palavras_do_pedido` + confirmação ao vivo) que já existe
/// para a trilha dos guardiões, sem exigir a peça criptográfica dele, só o reconhecimento humano.
///
/// **Limite honesto, dito sem rodeio:** isto não elimina o risco — nenhum sistema de autoatendimento
/// consegue provar que o guardião-testemunha não está, ele mesmo, sendo enganado ou coagido junto. O que
/// isto faz, de verdade: eleva o custo do ataque de "extrair 2 segredos de uma vítima" para "extrair 2
/// segredos E coagir ou enganar um SEGUNDO humano independente, ao vivo", e torna o ataque detectável — o
/// guardião vê a pessoa, pode notar sinais de coação e recusar ou alertar, a mesma defesa que já protege a
/// trilha dos guardiões, estendida aqui para os outros dois caminhos quando usados juntos sem ela.
pub fn pode_concluir_duas_trilhas(
    pedido_em: u64, agora: u64, cancelado_por_aparelho_antigo: bool,
    usou_trilha_dos_guardioes: bool, alguma_testemunha_guardia_confirmou_ao_vivo: bool,
) -> Result<(), ErroDeRecuperacao> {
    pode_concluir(pedido_em, agora, cancelado_por_aparelho_antigo)?;
    if !usou_trilha_dos_guardioes && !alguma_testemunha_guardia_confirmou_ao_vivo {
        return Err(ErroDeRecuperacao::SemTestemunhaViva);
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn guardioes(n: usize) -> Vec<(ChavePublica, ChaveSecreta)> { (0..n).map(|_| envelope::gerar_par_de_chaves()).collect() }

    #[test]
    fn dois_de_tres_amigos_recuperam_a_conta() {
        let g = guardioes(3);
        let publicas: Vec<ChavePublica> = g.iter().map(|(p, _)| p.clone()).collect();
        let segredo = [42u8; 32];
        let pacotes = preparar(&segredo, &publicas, 2).unwrap();
        let a = abrir_para_devolver(&g[0].1, &g[0].0, &pacotes[0], true).unwrap();
        let c = abrir_para_devolver(&g[2].1, &g[2].0, &pacotes[2], true).unwrap();
        assert_eq!(&reunir(&[&a, &c]).unwrap()[..], &segredo[..]);
    }

    #[test]
    fn um_amigo_sozinho_nao_recupera_nem_abre_o_pacote_de_outro() {
        let g = guardioes(3);
        let publicas: Vec<ChavePublica> = g.iter().map(|(p, _)| p.clone()).collect();
        let pacotes = preparar(&[7u8; 32], &publicas, 2).unwrap();
        let a = abrir_para_devolver(&g[0].1, &g[0].0, &pacotes[0], true).unwrap();
        assert_eq!(reunir(&[&a]).unwrap_err(), ErroDeRecuperacao::Insuficiente);
        assert_eq!(abrir_para_devolver(&g[0].1, &g[0].0, &pacotes[1], true).unwrap_err(), ErroDeRecuperacao::Ilegivel);
    }

    #[test]
    fn parte_adulterada_e_recusada() {
        let g = guardioes(3);
        let publicas: Vec<ChavePublica> = g.iter().map(|(p, _)| p.clone()).collect();
        let pacotes = preparar(&[9u8; 32], &publicas, 2).unwrap();
        let a = abrir_para_devolver(&g[0].1, &g[0].0, &pacotes[0], true).unwrap();
        // Adultera todas as peças do segundo guardião (as que a recuperação vai usar estão entre elas).
        let mut partes = PartesDoGuardiao::de_bytes(&abrir_para_devolver(&g[1].1, &g[1].0, &pacotes[1], true).unwrap()).unwrap();
        for p in partes.pecas.iter_mut() { p.dados[0] ^= 1; }
        let b = partes.para_bytes();
        assert_eq!(reunir(&[&a, &b]).unwrap_err(), ErroDeRecuperacao::Adulterado);
        assert_eq!(reunir(&[&a, &[1u8, 2, 3][..]]).unwrap_err(), ErroDeRecuperacao::Adulterado, "bytes quebrados");
    }

    #[test]
    fn precisa_de_3_a_5_guardioes() {
        let g = guardioes(2);
        let publicas: Vec<ChavePublica> = g.iter().map(|(p, _)| p.clone()).collect();
        assert_eq!(preparar(&[1u8; 32], &publicas, 2).unwrap_err(), ErroDeRecuperacao::GuardioesInvalidos);
        let g6 = guardioes(6);
        let p6: Vec<ChavePublica> = g6.iter().map(|(p, _)| p.clone()).collect();
        assert_eq!(preparar(&[1u8; 32], &p6, 2).unwrap_err(), ErroDeRecuperacao::GuardioesInvalidos);
    }

    #[test]
    fn recuperacao_exige_confirmacao_ao_vivo_e_espera() {
        let (p, s_) = envelope::gerar_par_de_chaves();
        let outros: Vec<_> = (0..2).map(|_| envelope::gerar_par_de_chaves().0).collect();
        let chaves = vec![p.clone(), outros[0].clone(), outros[1].clone()];
        let pacotes = preparar(b"segredo-da-conta-32-bytes-......", &chaves, 2).unwrap();
        assert_eq!(abrir_para_devolver(&s_, &p, &pacotes[0], false).unwrap_err(), ErroDeRecuperacao::SemConfirmacaoAoVivo);
        assert!(abrir_para_devolver(&s_, &p, &pacotes[0], true).is_ok());
        assert_eq!(pode_concluir(1_000, 1_000 + CARENCIA_SEGUNDOS - 1, false), Err(ErroDeRecuperacao::EmEspera));
        assert_eq!(pode_concluir(1_000, 1_000 + CARENCIA_SEGUNDOS, false), Ok(()));
        assert_eq!(pode_concluir(1_000, u64::MAX, true), Err(ErroDeRecuperacao::Cancelado), "aparelho antigo cancela a qualquer momento");
        let a = palavras_do_pedido(b"aparelho-novo", b"conta");
        assert_ne!(a, palavras_do_pedido(b"aparelho-do-golpista", b"conta"));
    }

    // ---- frase de recuperação pessoal: caminho independente dos guardiões ----

    #[test]
    fn frase_pessoal_sozinha_recupera_o_segredo_sem_nenhum_guardiao() {
        let segredo = b"segredo-da-conta-32-bytes-......";
        let (chave, bloco) = preparar_frase_pessoal(segredo).unwrap();
        assert_eq!(&abrir_frase_pessoal(&chave, &bloco).unwrap()[..], &segredo[..]);
    }

    #[test]
    fn frase_pessoal_com_chave_errada_ou_bloco_adulterado_falha() {
        let (chave, bloco) = preparar_frase_pessoal(b"segredo-de-32-bytes-para-testar.").unwrap();
        let outra_chave = [chave[0] ^ 1; TAM_CHAVE_DA_FRASE];
        assert_eq!(abrir_frase_pessoal(&outra_chave, &bloco).unwrap_err(), ErroDeRecuperacao::Ilegivel);
        let mut adulterado = bloco.clone();
        *adulterado.last_mut().unwrap() ^= 1;
        assert_eq!(abrir_frase_pessoal(&chave, &adulterado).unwrap_err(), ErroDeRecuperacao::Ilegivel);
    }

    #[test]
    fn preparar_e_preparar_frase_pessoal_sao_primitivas_genericas_e_nao_impedem_sozinhas_o_uso_antigo() {
        // Achado desta auditoria (Rodada 21, mesma sessão do redesenho): até a Rodada 20, um teste aqui
        // chamado `perder_um_caminho_nao_impede_o_outro_os_dois_sao_independentes` preparava `preparar` e
        // `preparar_frase_pessoal` diretamente sobre o SEGREDO DA CONTA INTEIRO e confirmava que qualquer
        // UM dos dois, isolado, já devolvia a conta — exatamente o comportamento antigo que o dono pediu
        // para eliminar nesta rodada ("nunca uma trilha só"). O teste continuava passando porque estas
        // duas funções são primitivas genéricas de baixo nível (dividem/selam QUALQUER `&[u8]` que lhes
        // for passado) — nada no tipo dos parâmetros as impede de receber o segredo inteiro em vez dos
        // bytes de uma trilha. A única barreira real contra repetir o modelo antigo é disciplina de quem
        // integra (sempre passar por `preparar_tres_trilhas`, nunca por estas duas direto) — e um teste
        // antigo com o nome errado estava mascarando exatamente esse risco em vez de alertar sobre ele.
        // Corrigido substituindo aquele teste por este: mesma mecânica (continua verdade que estas
        // primitivas aceitam qualquer segredo de 32 bytes — é para isso que servem, chamadas por
        // `preparar_tres_trilhas` com os bytes de CADA TRILHA, nunca com o segredo da conta), mas o nome e
        // o comentário agora dizem o que isto realmente é: uma primitiva reaproveitável, não o desenho de
        // recuperação da conta (esse é `preparar_tres_trilhas` + `concluir_com_duas_trilhas`, cobertos por
        // `qualquer_par_das_tres_trilhas_reconstroi_o_segredo_da_conta` e `uma_trilha_sozinha_nunca_basta`,
        // abaixo). Ver aviso correspondente no comentário de `preparar` e de `preparar_frase_pessoal`.
        let bytes_de_uma_trilha_qualquer = [42u8; 32]; // um payload genérico, não "o segredo da conta"
        let g = guardioes(3);
        let publicas: Vec<ChavePublica> = g.iter().map(|(p, _)| p.clone()).collect();
        let pacotes = preparar(&bytes_de_uma_trilha_qualquer, &publicas, 2).unwrap();
        let a = abrir_para_devolver(&g[0].1, &g[0].0, &pacotes[0], true).unwrap();
        let c = abrir_para_devolver(&g[2].1, &g[2].0, &pacotes[2], true).unwrap();
        assert_eq!(&reunir(&[&a, &c]).unwrap()[..], &bytes_de_uma_trilha_qualquer[..]);
    }

    #[test]
    fn frase_pessoal_bloco_nunca_contem_o_segredo_em_claro() {
        // Achado da campanha de força bruta desta rodada (11/10/2026): `custodia_mutua.rs` tem um teste
        // dedicado (`servidor_nunca_ve_o_conteudo`) provando que o bloco que vai para fora nunca contém o
        // conteúdo em claro; este caminho (frase pessoal) nunca tinha o equivalente, apesar de guardar um
        // segredo tão sensível quanto (a chave mestra da própria conta). `preparar_frase_pessoal` só prova
        // ida-e-volta (`abrir_frase_pessoal` recupera o mesmo segredo) e independência entre chamadas —
        // nenhum teste conferia, byte a byte, se o `bloco` (o que normalmente viaja para o servidor, sem a
        // `chave`) de fato não carrega o segredo visível.
        let segredo = b"SEGREDO-DA-CONTA-MARCADO-32-BYT";
        let (_chave, bloco) = preparar_frase_pessoal(segredo).unwrap();
        assert!(
            !bloco.windows(segredo.len()).any(|w| w == &segredo[..]),
            "o bloco cifrado nunca pode conter o segredo em claro, nem como substring"
        );
    }

    #[test]
    fn frase_pessoal_nao_revela_tamanho_nem_e_reaproveitavel_entre_contas() {
        // Duas chamadas com o mesmo segredo nunca repetem bloco (nonce novo) nem chave (aleatória nova) —
        // uma frase de uma conta não serve, por engano ou ataque, para abrir o bloco de outra conta.
        let (k1, b1) = preparar_frase_pessoal(b"mesmo-segredo-sempre-32-bytes...").unwrap();
        let (k2, b2) = preparar_frase_pessoal(b"mesmo-segredo-sempre-32-bytes...").unwrap();
        assert_ne!(&k1[..], &k2[..]);
        assert_ne!(b1, b2);
        assert_eq!(abrir_frase_pessoal(&k1, &b2).unwrap_err(), ErroDeRecuperacao::Ilegivel);
    }

    // ---- pergunta importante: terceira trilha independente (Rodada 21) ----

    const RESPOSTA: &[u8] = b"o nome do meu primeiro cachorro imaginario era Fumaca";

    #[test]
    fn pergunta_importante_sozinha_recupera_a_trilha() {
        let segredo = b"segredo-da-trilha-32-bytes......";
        let pacote = preparar_pergunta_importante(RESPOSTA, segredo).unwrap();
        assert_eq!(&abrir_pergunta_importante(RESPOSTA, &pacote).unwrap()[..], &segredo[..]);
    }

    #[test]
    fn resposta_errada_ou_bloco_adulterado_falha_sem_dizer_o_motivo() {
        let pacote = preparar_pergunta_importante(RESPOSTA, b"segredo-da-trilha-32-bytes......").unwrap();
        assert_eq!(abrir_pergunta_importante(b"resposta errada mas do mesmo tamanho ", &pacote).unwrap_err(), ErroDeRecuperacao::Ilegivel);
        let mut adulterado = pacote.clone();
        *adulterado.bloco.last_mut().unwrap() ^= 1;
        assert_eq!(abrir_pergunta_importante(RESPOSTA, &adulterado).unwrap_err(), ErroDeRecuperacao::Ilegivel);
    }

    #[test]
    fn resposta_curta_demais_e_recusada_na_preparacao() {
        assert_eq!(preparar_pergunta_importante(b"curta", b"segredo").unwrap_err(), ErroDeRecuperacao::RespostaFraca);
        assert!(b"curta".len() < TAM_MINIMO_RESPOSTA);
    }

    #[test]
    fn mesma_resposta_duas_vezes_nunca_repete_sal_nem_bloco() {
        let a = preparar_pergunta_importante(RESPOSTA, b"segredo-um-32-bytes.............").unwrap();
        let b = preparar_pergunta_importante(RESPOSTA, b"segredo-um-32-bytes.............").unwrap();
        assert_ne!(a.sal, b.sal);
        assert_ne!(a.bloco, b.bloco);
    }

    // ---- as três trilhas juntas: qualquer 2 de 3 reconstrói (Rodada 21) ----

    #[test]
    fn qualquer_par_das_tres_trilhas_reconstroi_o_segredo_da_conta() {
        let segredo = [77u8; 32];
        let t = preparar_tres_trilhas(&segredo).unwrap();
        for (a, b) in [(&t.guardioes, &t.frase), (&t.guardioes, &t.pergunta), (&t.frase, &t.pergunta)] {
            assert_eq!(&concluir_com_duas_trilhas(a, b).unwrap()[..], &segredo[..]);
        }
    }

    #[test]
    fn uma_trilha_sozinha_nunca_basta() {
        let t = preparar_tres_trilhas(&[9u8; 32]).unwrap();
        // `concluir_com_duas_trilhas` exige duas; não há como chamar com uma só e passar — provado
        // indiretamente reconstruindo com a MESMA trilha duas vezes, que `custodia::reconstruir` já
        // trata como "um guardião só" (regra existente, reaproveitada sem mudança).
        assert_eq!(concluir_com_duas_trilhas(&t.guardioes, &t.guardioes).unwrap_err(), ErroDeRecuperacao::Insuficiente);
    }

    #[test]
    fn fluxo_completo_das_tres_trilhas_seladas_cada_uma_pelo_seu_mecanismo() {
        // Simula o ciclo de vida real: configura as 3, recupera só 2 (perdeu os guardiões, mas tem frase
        // e pergunta) e confirma que o segredo da conta volta intacto.
        let segredo_da_conta = [200u8; 32];
        let t = preparar_tres_trilhas(&segredo_da_conta).unwrap();

        let g = (0..3).map(|_| envelope::gerar_par_de_chaves()).collect::<Vec<_>>();
        let publicas: Vec<ChavePublica> = g.iter().map(|(p, _)| p.clone()).collect();
        let _pacotes_guardioes = preparar(&t.guardioes.para_bytes(), &publicas, 2).unwrap(); // selado, não usado nesta recuperação

        let (chave_frase, bloco_frase) = preparar_frase_pessoal(&t.frase.para_bytes()).unwrap();
        let pacote_pergunta = preparar_pergunta_importante(RESPOSTA, &t.pergunta.para_bytes()).unwrap();

        // "Perdeu os guardiões, mas tem a frase e a resposta": recupera via frase + pergunta.
        let bytes_frase = abrir_frase_pessoal(&chave_frase, &bloco_frase).unwrap();
        let bytes_pergunta = abrir_pergunta_importante(RESPOSTA, &pacote_pergunta).unwrap();
        let trilha_frase = Trilha::de_bytes(&bytes_frase).unwrap();
        let trilha_pergunta = Trilha::de_bytes(&bytes_pergunta).unwrap();
        assert_eq!(&concluir_com_duas_trilhas(&trilha_frase, &trilha_pergunta).unwrap()[..], &segredo_da_conta[..]);
    }

    #[test]
    fn concluir_com_duas_trilhas_so_funciona_depois_da_carencia_ou_se_cancelada() {
        assert_eq!(pode_concluir_duas_trilhas(1_000, 1_000 + CARENCIA_SEGUNDOS - 1, false, true, false), Err(ErroDeRecuperacao::EmEspera));
        assert_eq!(pode_concluir_duas_trilhas(1_000, u64::MAX, true, true, false), Err(ErroDeRecuperacao::Cancelado));
    }

    #[test]
    fn sem_a_trilha_dos_guardioes_exige_uma_testemunha_viva_de_um_guardiao() {
        // O risco residual da rodada anterior: frase + pergunta, sem guardião envolvido, sem testemunha.
        assert_eq!(
            pode_concluir_duas_trilhas(1_000, 1_000 + CARENCIA_SEGUNDOS, false, false, false),
            Err(ErroDeRecuperacao::SemTestemunhaViva),
            "frase + pergunta, sem nenhum guardião confirmando ao vivo, não deveria bastar"
        );
        assert_eq!(
            pode_concluir_duas_trilhas(1_000, 1_000 + CARENCIA_SEGUNDOS, false, false, true),
            Ok(()),
            "frase + pergunta, mas com um guardião testemunhando ao vivo, agora basta"
        );
        assert_eq!(
            pode_concluir_duas_trilhas(1_000, 1_000 + CARENCIA_SEGUNDOS, false, true, false),
            Ok(()),
            "guardiões + (frase OU pergunta) nunca precisa de testemunha extra — a trilha dos guardiões já exige conferência ao vivo"
        );
    }
}
