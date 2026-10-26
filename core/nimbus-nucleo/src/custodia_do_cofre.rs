//! # Módulo `custodia_do_cofre` — chave do cofre pessoal dividida entre aparelho e servidor (2 de 2)
//!
//! **Proposta nova desta rodada (10/10/2026), ainda sem ID de requisito formal no SRS — se adotada,
//! precisa ganhar seu próprio prefixo e entrar no manual técnico como as demais.** Nasceu de uma
//! pergunta direta do dono: existe alguma forma de um VPS reduzir de verdade o que um aparelho
//! capturado entrega a uma extração forense (Cellebrite/GrayKey), ao contrário de um simples
//! apagamento remoto (que uma sala sem sinal já derrota por desenho — ver `PENDENCIAS_E_DECISOES.md`,
//! seção 26, e a Cláusula 6a/6b do `CONTRATO_DA_PONTE.md`)?
//!
//! A resposta é sim, mas só se nem o aparelho sozinho nem o servidor sozinho guardarem a chave
//! inteira. Este módulo não inventa criptografia nova: reaproveita `custodia::dividir`/`reconstruir`
//! (já provado em dois usos reais — `corporativo.rs`, chave mestra da empresa entre executivos, e
//! `recuperacao.rs`, recuperação de conta entre 3 a 5 amigos), aqui com o caso mais simples possível
//! do mesmo esquema: `n = k = 2`, uma peça para o aparelho, uma peça para o servidor. Nenhum dos dois
//! lados sozinho recupera nada — a peça que cada um guarda é, por construção (partilha XOR tipo
//! "one-time pad", Ito–Saito–Nishizeki), indistinguível de ruído aleatório sem a outra peça. Isto vale
//! inclusive contra o próprio servidor sob ordem judicial: o que ele entregaria, sozinho, não abre nada.
//!
//! **O que este módulo prova (testes abaixo) e o que ele NÃO prova:**
//! - Prova: dividir e reconstruir dá a volta certa; uma peça sozinha (aparelho OU servidor) nunca
//!   reconstrói nada (`ErroDeCustodia::Insuficiente`); uma peça adulterada é detectada pelo compromisso
//!   (`ErroDeCustodia::PecaAdulterada`); a peça do servidor sobrevive a serializar/deserializar em
//!   bytes (`para_bytes`/`de_bytes`, já existentes em `custodia::PartesDoGuardiao`), confirmando que dá
//!   para transportá-la pela rede sem tocar na matemática.
//! - NÃO prova (fica para quem for integrar, antes de declarar isto "pronto" em qualquer material):
//!   (1) [FECHADO em 10/10/2026] campanha de mutação própria deste módulo: 4 defeitos plantados
//!   específicos (limiar `K` caindo para 1, `N` subindo para 3, troca de qual peça vai para o aparelho e
//!   qual vai para o servidor, reconstrução esquecendo de usar a peça do servidor) — os 4 pegos pelos
//!   testes acima, zero sobrevivente; projeto total em 342 mutantes, suíte em 611 testes (0 falhas, 6
//!   ignorados), identidade do código sem mudança (ver `NUMEROS.json` e `PENDENCIAS_E_DECISOES.md`, seção
//!   28); (2) nenhum transporte de rede real — a busca da peça do
//!   servidor é responsabilidade de quem implementar a ponte, abstraída de propósito para fora deste
//!   módulo; (3) [FECHADO em 11/10/2026, Rodada 21 — ver `verificar_cache`/`PRAZO_MAXIMO_DO_CACHE_SEGUNDOS`
//!   logo abaixo] a rotina de cache local da chave já reconstruída, para uso offline entre uma busca e
//!   outra, agora tem prazo e invalidação — não mais "decisão de produto em aberto"; (4) nenhuma
//!   tabela/rota de servidor para guardar a peça por prazo mais longo — o esquema de backend documentado
//!   hoje (`05_Esquema_Backend_Nimbus.md`) só mostra `nimbus.blocos` com teto de 48 h e sem UPDATE, o que
//!   não parece servir para isto sem mudança; precisa confirmar com quem tem o repositório do backend.
//!
//! **Item (3) do parágrafo anterior, fechado nesta rodada (Rodada 21, 11/10/2026) — resposta ao pedido
//! explícito do fundador por um mecanismo novo contra extração forense, "algo até hoje não pensado":**
//! um cache local da chave reconstruída, sem prazo, devolveria exatamente o problema que este módulo
//! inteiro existe para evitar — um aparelho isolado da rede de propósito (modo avião, bolsa de Faraday,
//! o manual de qualquer extração forense) voltaria a carregar, sozinho, tudo que precisa para nunca mais
//! depender do servidor, equivalendo-se ao esquema antigo de uma única chave só local, sem prazo algum.
//! `verificar_cache` (abaixo) fecha essa brecha: o cache só é válido por `PRAZO_MAXIMO_DO_CACHE_SEGUNDOS`;
//! vencido esse prazo, a ponte É OBRIGADA a descartar a chave reconstruída e buscar a peça do servidor
//! de novo antes de reabrir o cofre — exatamente o padrão já usado pela Cláusula 6a (`expiracao.rs`)
//! para o conteúdo temporário, aplicado agora um nível acima, ao próprio DESBLOQUEIO do cofre. **Por que
//! isto muda o modelo de ataque de verdade, não é só "mais Argon2id":** hoje (Cláusula 26b,
//! `CONTRATO_DA_PONTE.md`), quem captura o aparelho pode tentar o PIN offline, no próprio hardware do
//! atacante, pelo tempo que
//! quiser — o único custo é o do Argon2id, fixo, e um laboratório forense bem financiado tem tempo e
//! paralelismo de sobra contra um PIN de poucos dígitos. Com a peça do servidor obrigatória depois do
//! prazo, o ataque deixa de ser "computação pura, sem ninguém vigiando" e passa a exigir uma ida à rede
//! que o SERVIDOR pode, de forma independente, limitar e negar (um número suspeito de pedidos da peça
//! para o mesmo aparelho é visível e recusável do lado de quem o atacante não controla) — algo que
//! nenhuma quantidade de poder de computação local supera. **O que isto NÃO resolve, dito com a mesma
//! honestidade do resto deste arquivo:** um aparelho capturado DESBLOQUEADO, em sessão ativa, com a
//! chave já reconstruída em RAM dentro do prazo — nenhum mecanismo, deste módulo ou de qualquer produto
//! real, protege o que já está, no instante da captura, legítima e ativamente acessível (mesmo limite
//! universal já documentado em `custodia_mutua.rs`). Para ESSE cenário, a peça prática já existente é o
//! `panico::Gatilho::ToqueDePanico`/`MovimentoBrusco` (apertar liga/desliga 5 vezes, ou o tranco de
//! arrancar o aparelho da mão — já implementados, `panico.rs`) configurado para `Reacao::ApagarChaves`:
//! quando a ponte traduzir essa reação em código, "apagar as chaves deste aparelho" passa a incluir
//! também zerar este cache (nunca esperar o prazo normal vencer) — item de integração para quem ligar a
//! ponte, não uma função nova aqui, pelo mesmo motivo de `comando_remoto.rs` não fazer a própria rede.
//! **O que fica para depois, não fabricado agora:** uma chamada de "revogação antecipada" da peça do
//! servidor no instante do gatilho (em vez de só esperar o prazo vencer) precisaria de uma rota de
//! backend que não existe hoje (mesma lacuna (4) já registrada acima sobre `05_Esquema_Backend_Nimbus.md`)
//! — o cache com prazo, abaixo, já funciona sem ela (defesa por prazo, não por notificação ativa), então
//! a ausência da rota não bloqueia este fechamento.
use crate::custodia::{self, ErroDeCustodia, PartesDoGuardiao};
use zeroize::Zeroizing;

/// Prazo máximo que a chave reconstruída (a partir das duas peças) pode ficar em cache local antes de a
/// ponte ser obrigada a descartá-la e buscar a peça do servidor de novo. Mesma ordem de grandeza da
/// carência de recuperação (`recuperacao::CARENCIA_SEGUNDOS`, 72h) — escolha deliberada de simetria com
/// o resto do produto, não coincidência: ambas equilibram o mesmo par de forças (uma janela curta demais
/// pesa no uso comum sem rede; uma longa demais devolve ao atacante offline o tempo que este módulo
/// existe para tirar dele). Produto, não núcleo: o dono pode querer outro número — ver FECHO.
pub const PRAZO_MAXIMO_DO_CACHE_SEGUNDOS: u64 = 72 * 3600;

/// Decide, nunca executa — o mesmo desenho de `expiracao::tempo_para_destruir_mesmo_offline`: este
/// núcleo não apaga nada sozinho (não tem acesso a onde a ponte guarda o cache), só informa se o prazo
/// já venceu. `reconstruida_em`/`agora`: segundos de relógio monotônico do aparelho (nunca `hora_confiavel`
/// — o mesmo motivo da Cláusula 6a: a checagem que PROTEGE tem de continuar avançando mesmo sem rede;
/// só a checagem que CONCEDE acesso é que trava sem confirmação do servidor).
#[derive(Debug, PartialEq, Eq)]
pub enum ErroDoCacheLocal {
    /// Prazo vencido — a ponte deve descartar a chave cacheada e buscar a peça do servidor de novo antes
    /// de reabrir o cofre. Nunca significa "chave errada" ou "peça adulterada": a chave em si continua
    /// válida, só o uso dela SEM reconfirmar com o servidor é que deixou de ser permitido.
    Vencido,
}

pub fn verificar_cache(reconstruida_em: u64, agora: u64) -> Result<(), ErroDoCacheLocal> {
    if agora < reconstruida_em.saturating_add(PRAZO_MAXIMO_DO_CACHE_SEGUNDOS) { Ok(()) } else { Err(ErroDoCacheLocal::Vencido) }
}

/// Participantes fixos: o aparelho e o servidor, nunca mais, nunca menos — por isso `N = K = 2`, o caso
/// degenerado (sem redundância combinatória) do esquema geral `n` de `k` de `custodia.rs`.
pub const N: u8 = 2;
pub const K: u8 = 2;

/// Índice do participante dentro da dupla devolvida por `custodia::dividir` — nomeado aqui para nunca
/// depender de "a peça 0 é sempre o aparelho" ficar implícito em algum lugar distante do código.
const INDICE_APARELHO: u8 = 0;
const INDICE_SERVIDOR: u8 = 1;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeCustodiaDoCofre {
    Custodia(ErroDeCustodia),
    /// `dividir`/`reconstruir` devolveram uma dupla com os índices trocados ou incompletos — bug de
    /// quem chamou, nunca esperado na prática, mas conferido explicitamente em vez de usar `.unwrap()`.
    FormatoInesperado,
}

/// Divide a chave do cofre em duas peças: uma para guardar no aparelho (do jeito que a Cláusula 26
/// já guarda a chave hoje — nunca em claro, sempre atrás do PIN), outra para enviar ao servidor.
/// Chamado uma vez, na ativação do recurso (decisão do dono, nunca automática — mesmo espírito da
/// Família 2 já recomendada: existe, mas quem liga é o usuário, de olhos abertos sobre a troca).
pub fn dividir_entre_aparelho_e_servidor(
    chave_do_cofre: &[u8],
) -> Result<(PartesDoGuardiao, PartesDoGuardiao), ErroDeCustodiaDoCofre> {
    let (mut partes, _compromisso) =
        custodia::dividir(chave_do_cofre, N, K).map_err(ErroDeCustodiaDoCofre::Custodia)?;
    if partes.len() != 2 {
        return Err(ErroDeCustodiaDoCofre::FormatoInesperado);
    }
    let peca_servidor = partes.pop().ok_or(ErroDeCustodiaDoCofre::FormatoInesperado)?;
    let peca_aparelho = partes.pop().ok_or(ErroDeCustodiaDoCofre::FormatoInesperado)?;
    if peca_aparelho.guardiao != INDICE_APARELHO || peca_servidor.guardiao != INDICE_SERVIDOR {
        return Err(ErroDeCustodiaDoCofre::FormatoInesperado);
    }
    Ok((peca_aparelho, peca_servidor))
}

/// Reconstrói a chave do cofre a partir das duas peças. Pensado para ser chamado só numa "sessão
/// fria" (depois de reiniciar o aparelho, ou quando o cache local da chave já reconstruída — se algum
/// dia existir, ver nota no topo do arquivo — tiver sido invalidado), nunca a cada abertura comum do
/// app: isto é o que torna o mecanismo viável sem exigir rede o tempo todo.
pub fn reconstruir_a_partir_das_duas_pecas(
    peca_aparelho: &PartesDoGuardiao,
    peca_servidor: &PartesDoGuardiao,
) -> Result<Zeroizing<Vec<u8>>, ErroDeCustodiaDoCofre> {
    custodia::reconstruir(&[peca_aparelho, peca_servidor]).map_err(ErroDeCustodiaDoCofre::Custodia)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn chave_de_teste() -> Vec<u8> {
        b"chave-do-cofre-de-32-bytes-teste".to_vec()
    }

    #[test]
    fn divide_e_reconstroi_a_mesma_chave() {
        let chave = chave_de_teste();
        let (aparelho, servidor) = dividir_entre_aparelho_e_servidor(&chave).unwrap();
        let reconstruida = reconstruir_a_partir_das_duas_pecas(&aparelho, &servidor).unwrap();
        assert_eq!(&reconstruida[..], &chave[..]);
    }

    #[test]
    fn peca_do_aparelho_sozinha_nao_reconstroi_nada() {
        let chave = chave_de_teste();
        let (aparelho, _servidor) = dividir_entre_aparelho_e_servidor(&chave).unwrap();
        let erro = custodia::reconstruir(&[&aparelho]).unwrap_err();
        assert_eq!(erro, ErroDeCustodia::Insuficiente, "uma peça só, abaixo do limiar k=2, não pode bastar");
    }

    #[test]
    fn peca_do_servidor_sozinha_nao_reconstroi_nada() {
        let chave = chave_de_teste();
        let (_aparelho, servidor) = dividir_entre_aparelho_e_servidor(&chave).unwrap();
        let erro = custodia::reconstruir(&[&servidor]).unwrap_err();
        assert_eq!(erro, ErroDeCustodia::Insuficiente, "o servidor sozinho — nem sob ordem judicial — reconstrói nada");
    }

    #[test]
    fn peca_do_servidor_e_ruido_independente_da_chave() {
        // Não dá para provar "é aleatório" com um teste determinístico, mas dá para provar que a MESMA
        // chave, dividida duas vezes, produz peças de servidor DIFERENTES — se fosse determinístico
        // (ex.: derivado só da chave, sem aleatoriedade nova), isto pegaria o bug na hora.
        let chave = chave_de_teste();
        let (_a1, servidor1) = dividir_entre_aparelho_e_servidor(&chave).unwrap();
        let (_a2, servidor2) = dividir_entre_aparelho_e_servidor(&chave).unwrap();
        assert_ne!(
            &servidor1.para_bytes()[..],
            &servidor2.para_bytes()[..],
            "duas divisões da mesma chave têm de usar aleatoriedade nova a cada vez"
        );
    }

    #[test]
    fn peca_adulterada_e_detectada_pelo_compromisso() {
        let chave = chave_de_teste();
        let (aparelho, mut servidor) = dividir_entre_aparelho_e_servidor(&chave).unwrap();
        // Simula um servidor malicioso ou corrompido devolvendo uma peça alterada.
        servidor.pecas[0].dados[0] ^= 0xFF;
        let erro = reconstruir_a_partir_das_duas_pecas(&aparelho, &servidor).unwrap_err();
        assert_eq!(erro, ErroDeCustodiaDoCofre::Custodia(ErroDeCustodia::PecaAdulterada));
    }

    #[test]
    fn peca_do_servidor_sobrevive_a_serializar_e_desserializar() {
        // Prova que a peça do servidor pode de fato viajar pela rede como bytes (relevante: é
        // exatamente isto que teria de ir no corpo de um POST novo, ainda não desenhado no esquema
        // de backend documentado hoje).
        let chave = chave_de_teste();
        let (aparelho, servidor) = dividir_entre_aparelho_e_servidor(&chave).unwrap();
        let bytes = servidor.para_bytes();
        let servidor_de_volta = PartesDoGuardiao::de_bytes(&bytes).unwrap();
        let reconstruida = reconstruir_a_partir_das_duas_pecas(&aparelho, &servidor_de_volta).unwrap();
        assert_eq!(&reconstruida[..], &chave[..]);
    }

    #[test]
    fn chave_vazia_e_rejeitada_como_em_custodia() {
        assert_eq!(
            dividir_entre_aparelho_e_servidor(&[]).unwrap_err(),
            ErroDeCustodiaDoCofre::Custodia(ErroDeCustodia::ParametrosInvalidos)
        );
    }

    // ---- cache local com prazo (Rodada 21, 11/10/2026) ----

    #[test]
    fn cache_recem_reconstruido_e_valido() {
        assert_eq!(verificar_cache(1_000, 1_000), Ok(()));
        assert_eq!(verificar_cache(1_000, 1_000 + PRAZO_MAXIMO_DO_CACHE_SEGUNDOS - 1), Ok(()));
    }

    #[test]
    fn cache_vence_exatamente_no_prazo_e_nunca_antes() {
        assert_eq!(verificar_cache(1_000, 1_000 + PRAZO_MAXIMO_DO_CACHE_SEGUNDOS), Err(ErroDoCacheLocal::Vencido));
        assert_eq!(verificar_cache(0, PRAZO_MAXIMO_DO_CACHE_SEGUNDOS - 1), Ok(()), "um segundo antes do prazo ainda vale");
    }

    #[test]
    fn cache_continua_vencido_por_mais_tempo_que_passe_nunca_volta_a_valer() {
        assert_eq!(verificar_cache(1_000, 1_000 + PRAZO_MAXIMO_DO_CACHE_SEGUNDOS + 1), Err(ErroDoCacheLocal::Vencido));
        assert_eq!(verificar_cache(0, u64::MAX), Err(ErroDoCacheLocal::Vencido), "mesmo um aparelho isolado por anos não reabre sozinho");
    }

    #[test]
    fn prazo_do_cache_e_a_mesma_ordem_de_grandeza_da_carencia_de_recuperacao() {
        // Não precisa ser IGUAL — só confirma a simetria deliberada citada no comentário do módulo, e
        // serve de sentinela: se um dos dois mudar no futuro sem o outro, este teste força a decisão a
        // ser explícita, não um desvio acidental.
        assert_eq!(PRAZO_MAXIMO_DO_CACHE_SEGUNDOS, crate::recuperacao::CARENCIA_SEGUNDOS);
    }
}
