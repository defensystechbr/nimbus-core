//! # Módulo `cobranca` — ficha anônima de licença (pagamento sem identificação)
//!
//! Implementa a peça que faltava da arquitetura já decidida antes desta rodada (ver
//! `NimbusOne_Plano_de_Negocios.md` §1b e `autenticacao::Exigencia::Ficha`, cujo comentário já
//! dizia "ficha anônima de licença, que não diz quem pagou" — mas nenhum mecanismo real existia
//! ainda). **O que este módulo garante, e o que NÃO garante, dito sem maquiagem:**
//!
//! * O processador de pagamento (Stripe ou qualquer outro) SABE quem pagou — cartão, nome,
//!   endereço de cobrança. Isto não é uma lacuna deste desenho: é uma exigência legal (KYC/AML)
//!   que nenhum processador de pagamento sério opera sem cumprir, em nenhum país. Prometer
//!   "processador de pagamento anônimo" seria inventar algo que não existe — ver
//!   `RESPOSTA_RODADA24_PENDENCIAS_DO_DONO.md`, seção de pagamento, para o detalhe e as fontes.
//! * O que ESTE módulo quebra é a LIGAÇÃO entre "quem pagou" e "qual ficha foi emitida" — pela
//!   assinatura cega RSA (RFC 9474, IETF; suíte RSA-PSS com SHA-384, variante aleatorizada —
//!   `KeyPair<Sha384, PSS, Randomized>` do crate `blind-rsa-signatures`, mesma primitiva por trás
//!   do Privacy Pass/RFC 9578). Quem assina (domínio de EMISSÃO) nunca lê o token exato que está
//!   assinando; quem gastou dinheiro (domínio de PAGAMENTO) nunca vê a ficha final que resulta,
//!   porque o cliente só termina de montá-la (`finalizar_ficha`) depois de já ter saído do fluxo
//!   de pagamento. Separação de quatro domínios, como já registrado em memória do projeto:
//!   pagamento → emissão de credencial cega → conta → armazenamento — nenhum, sozinho, liga os
//!   dois lados.
//!
//! **Por que o plano e o período NUNCA estão dentro do token que o cliente cega:** numa
//! assinatura cega, quem assina nunca lê o que está assinando — é o ponto todo da primitiva. Se
//! "Max" estivesse dentro do token cego, um cliente malicioso poderia cegar um token que alega
//! "Max" depois de pagar só o Pro, e o emissor assinaria sem nunca saber o que assinou. Por isso
//! aqui o plano e o período vivem em QUAL PAR DE CHAVES assina, nunca no conteúdo cego: cada
//! `(PlanoPago, Periodo)` tem seu próprio par RSA (`gerar_chave_do_periodo`, gerado com
//! antecedência pela operação, nunca por pedido de cliente). O emissor escolhe a chave com base no
//! que `PagamentoConfirmado` afirma — e É RESPONSABILIDADE DO CHAMADOR (a camada de servidor, fora
//! deste núcleo) só construir um `PagamentoConfirmado` depois de conferir a assinatura do webhook
//! do processador E o valor pago contra o preço exato daquele plano (`NimbusOne_Plano_de_Negocios.
//! md` §1b) — este módulo confia no que `PagamentoConfirmado` diz, pela mesma razão que
//! `autenticacao::Porteiro` confia em `ficha_valida: bool` já verificado por fora: nenhum dos dois
//! tem, nem deveria ter, uma chave de API de processador de pagamento dentro do núcleo de
//! criptografia.
//!
//! **Fronteira deliberada (o que este módulo NÃO faz):** não fala com Stripe nem com nenhum outro
//! processador; não decide preço (vem de `NimbusOne_Plano_de_Negocios.md`); não decide quando uma
//! assinatura deve renovar (a camada de servidor chama `assinar_cego` de novo a cada cobrança
//! recorrente confirmada, com o `Periodo` seguinte); não persiste nada além do necessário para
//! impedir reutilizar o MESMO evento de pagamento duas vezes (`eventos_ja_usados`) — e mesmo isso
//! nunca é ligado à ficha final, só ao evento de pagamento em si.

use blind_rsa_signatures::{BlindSignature, BlindingResult, DefaultRng, KeyPair, MessageRandomizer, PSS, PublicKey, Randomized, SecretKey, Sha384, Signature};
use rand_core::{OsRng, RngCore};
use std::collections::{HashMap, HashSet};

/// Suíte fixa: RSA-PSS/SHA-384, variante aleatorizada (RFC 9474 §4.4) — a mesma suíte do exemplo
/// de referência da biblioteca; "aleatorizada" é a opção que a própria RFC recomenda por padrão.
type ParDeChaves = KeyPair<Sha384, PSS, Randomized>;
/// Chave pública de uma suíte RSA-PSS/SHA-384 aleatorizada.
pub type ChavePublicaDeEmissao = PublicKey<Sha384, PSS, Randomized>;
type ChaveSecretaDeEmissao = SecretKey<Sha384, PSS, Randomized>;

/// Os planos pagos do Cofre Pessoal (preço e armazenamento: `NimbusOne_Plano_de_Negocios.md` §1b).
/// O plano Grátis não emite ficha — não há pagamento a confirmar; as rotas do Grátis continuam
/// `autenticacao::Exigencia::Publica` ou `Exigencia::Capacidade`, nunca `Exigencia::Ficha`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanoPago {
    Pro,
    Max,
}

/// Período de faturamento: um mês civil. `mes` de 1 a 12 — ver `valido`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Periodo {
    pub ano: u16,
    pub mes: u8,
}

impl Periodo {
    pub fn valido(&self) -> bool {
        (1..=12).contains(&self.mes)
    }

    /// Período seguinte — para gerar a chave da PRÓXIMA cobrança antes da atual vencer, nunca
    /// depois (sem a chave do período seguinte já existir, uma renovação em dia não teria como
    /// emitir a ficha a tempo).
    pub fn proximo(&self) -> Periodo {
        if self.mes == 12 {
            Periodo { ano: self.ano + 1, mes: 1 }
        } else {
            Periodo { ano: self.ano, mes: self.mes + 1 }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeCobranca {
    /// `Periodo.mes` fora de 1..=12.
    PeriodoInvalido,
    /// Não existe par de chaves gerado para este `(PlanoPago, Periodo)` — ver `gerar_chave_do_periodo`.
    ChaveDoPeriodoInexistente,
    /// O mesmo `id_do_evento` já emitiu uma ficha antes; cada evento de pagamento emite exatamente uma.
    PagamentoJaUsado,
    /// Falha da primitiva criptográfica (biblioteca); nunca deveria acontecer com entrada válida.
    FalhaCriptografica,
    /// A ficha apresentada não confere contra a chave pública do plano/período informado.
    FichaInvalida,
}

/// Fato que o CHAMADOR jura verdadeiro: um pagamento real, confirmado pelo processador (assinatura
/// do webhook já conferida, valor já confrontado com o preço do plano), para este plano e este
/// período exatos. Ver a fronteira deliberada na doc do módulo — este núcleo nunca verifica isto
/// sozinho, pela mesma razão que não tem credencial de processador de pagamento.
#[derive(Debug, Clone)]
pub struct PagamentoConfirmado {
    /// Identificador único do evento no processador (ex.: `evt_...` do Stripe). Usado só para
    /// impedir emitir duas fichas do mesmo pagamento — nunca persistido além disso, nunca ligado à
    /// ficha resultante (a ficha em si não carrega nenhum dado do pagamento; ver doc do módulo).
    pub id_do_evento: Vec<u8>,
    pub plano: PlanoPago,
    pub periodo: Periodo,
}

/// A ficha pronta, do lado do cliente: o token em claro, a assinatura final (já descegada) e o
/// randomizador que `verify`/`finalize` exigem (RFC 9474, variante aleatorizada — ver o tipo
/// `ParDeChaves` no topo do arquivo). As três partes juntas, verificadas contra a chave pública
/// certa (`ficha_e_valida`), são o que alimenta `ficha_valida: bool` em `autenticacao::Porteiro`.
///
/// **Reconstrução a partir dos bytes que o NAVEGADOR produz (`@cloudflare/blindrsa-ts`,
/// `pagamento_web/cliente_cobranca.mjs`) — contrato de fio, confirmado empiricamente na Rodada
/// 24, não deduzido da RFC de memória:** os três campos acima são todos públicos de propósito,
/// exatamente para que um "porteiro"/gateway em Rust monte uma `Ficha` direto dos bytes que o
/// cliente apresenta, sem nenhuma função de conversão extra:
/// - `token` → os 32 bytes aleatórios originais que o cliente gerou antes de cegar.
/// - `assinatura` → `Signature(bytes)`, onde `bytes` são exatamente os 256 bytes (RSA 2048 bits)
///   que `suite.finalize()` devolve no navegador — a biblioteca JS NUNCA embute o randomizador
///   dentro desses bytes (confirmado rodando o round-trip real: a assinatura final tem sempre o
///   tamanho cru de uma assinatura RSA, nunca +32).
/// - `randomizador` → `Some(MessageRandomizer(bytes))`, onde `bytes` são os PRIMEIROS 32 bytes de
///   `preparedMsg` (o valor que `suite.prepare(token)` devolveu no navegador, ANTES de cegar) —
///   confirmado empiricamente que `preparedMsg = randomizador(32) || token(32)`, nessa ordem; o
///   cliente precisa guardar esses 32 bytes (ou o `preparedMsg` inteiro, de que eles são só o
///   prefixo) ele mesmo, porque depois de `finalize()` eles não voltam a aparecer em nenhum outro
///   valor devolvido pela biblioteca JS.
#[derive(Debug, Clone)]
pub struct Ficha {
    pub token: Vec<u8>,
    pub assinatura: Signature,
    pub randomizador: Option<MessageRandomizer>,
}

/// Emite fichas: um par de chaves RSA por `(PlanoPago, Periodo)`, gerado com antecedência. Vive no
/// domínio de EMISSÃO — deliberadamente isolado do domínio de PAGAMENTO (ver doc do módulo): o
/// processo que fala com o processador de pagamento e o processo que assina cegamente não
/// precisam ser, e não deveriam ser, o mesmo serviço, para que o rompimento de um não comprometa
/// o segredo que o outro guarda. A fronteira entre os dois é só desenhada aqui (os dois TIPOS que a
/// tornam possível); qual serviço roda onde é decisão de infraestrutura, fora deste núcleo.
#[derive(Default)]
pub struct EmissorDeFichas {
    chaves: HashMap<(PlanoPago, Periodo), ParDeChaves>,
    eventos_ja_usados: HashSet<Vec<u8>>,
}

impl EmissorDeFichas {
    pub fn novo() -> Self {
        Self::default()
    }

    /// Gera (ou substitui) o par de chaves de um `(plano, período)`. Chamado com antecedência pela
    /// operação (ex.: todo dia 1, gerar a chave do mês seguinte para os dois planos pagos), nunca
    /// por pedido de cliente. 2048 bits RSA: o mínimo que a própria biblioteca aceita, e o mesmo
    /// tamanho dos vetores de teste oficiais da RFC 9474.
    pub fn gerar_chave_do_periodo(&mut self, plano: PlanoPago, periodo: Periodo) -> Result<(), ErroDeCobranca> {
        if !periodo.valido() {
            return Err(ErroDeCobranca::PeriodoInvalido);
        }
        let kp = ParDeChaves::generate(&mut DefaultRng, 2048).map_err(|_| ErroDeCobranca::FalhaCriptografica)?;
        self.chaves.insert((plano, periodo), kp);
        Ok(())
    }

    /// Chave pública de um `(plano, período)` — o cliente precisa dela para cegar o token
    /// (`preparar_pedido_de_ficha`) e, depois, para verificar a ficha pronta.
    pub fn chave_publica(&self, plano: PlanoPago, periodo: Periodo) -> Option<&ChavePublicaDeEmissao> {
        self.chaves.get(&(plano, periodo)).map(|kp| &kp.pk)
    }

    fn chave_secreta(&self, plano: PlanoPago, periodo: Periodo) -> Option<&ChaveSecretaDeEmissao> {
        self.chaves.get(&(plano, periodo)).map(|kp| &kp.sk)
    }

    /// Assina CEGAMENTE o token que o cliente já cegou — só depois de confirmar que o pagamento é
    /// real (`PagamentoConfirmado`, já verificado pelo chamador) e que este evento de pagamento
    /// ainda não emitiu outra ficha. Nunca vê o token em claro: `msg_cega` já saiu da máquina do
    /// cliente cego (ver `preparar_pedido_de_ficha`).
    ///
    /// ATENÇÃO — vulnerabilidade conhecida, sem patch upstream, no crate `rsa` (dependência
    /// transitiva de `blind-rsa-signatures`, via `sk.blind_sign` logo abaixo): RUSTSEC-2023-0071
    /// ("Marvin Attack"), canal lateral de tempo. `patched = []` deliberado no advisório — sem
    /// correção disponível hoje (13/10/2026) para `rsa 0.10.0-rc.19`, a versão que o `Cargo.lock`
    /// deste projeto de fato resolve (confirmado por leitura direta do lock nesta revisão — o
    /// próprio advisório só nomeia por data a rc.18, uma versão anterior).
    ///
    /// **Revisão de 13/10/2026 (mesma sessão, a pedido do fundador — "revise tudo novamente...
    /// dentro de toda a maior verdade absoluta"), por leitura do código-fonte real de
    /// `blind-rsa-signatures` 0.18.0, não só da documentação gerada:** `blind_sign` constrói seu
    /// próprio `DefaultRng` e chama `rsa::hazmat::rsa_decrypt_and_check(&self.inner, Some(rng),
    /// ...)` — ou seja, ESTA chamada específica já passa `Some(rng)` (caminho COM blinding), nunca
    /// o caminho sem RNG que a PR upstream ainda-não-mesclada `RustCrypto/RSA#702` existe para
    /// corrigir PARA QUEM NÃO fornece RNG. Isto reduz, mas não elimina, a exposição ao componente
    /// de "exponenciação sem blinding" do achado original. Três limites honestos permanecem, nenhum
    /// contornado por este achado: (1) o advisório não dá nenhuma exceção para quem já fornece RNG
    /// — `patched = []` vale para o crate inteiro; (2) `rsa_decrypt_and_check` é função `hazmat`
    /// (convenção do próprio RustCrypto para API de baixo nível que exige cuidado extra) — esta
    /// auditoria confirmou que um RNG É passado, mas não auditou a aritmética de blinding em si;
    /// (3) o outro mecanismo rastreado pelo advisório — padding PKCS#1v1.5 não-constante-no-tempo
    /// (`RustCrypto/RSA#626`, aberto; `#680`, não mesclado) — provavelmente não se aplica a uma
    /// assinatura CEGA (nunca decifra nem ramifica sobre padding de entrada do atacante), mas
    /// nenhuma fonte primária confirma essa exceção por escrito para este crate.
    ///
    /// A rotação mensal de chave por `(PlanoPago, Periodo)` e a emissão só após pagamento
    /// confirmado continuam atenuando, sem eliminar, a janela de exposição — `eventos_ja_usados`
    /// (abaixo) impede reusar o MESMO evento, mas não limita quantos eventos distintos um mesmo
    /// pagante pode gerar contra a MESMA chave do período. Mitigação de engenharia recomendada
    /// nesta revisão, **implementada em 14/10/2026** a pedido explícito do fundador ("construção e
    /// execução... sem deixar nenhuma atividade para depois"): jitter aleatório e espaçamento
    /// mínimo entre operações de assinatura na CAMADA DE SERVIDOR (`pagamento_web/servidor-cobranca`
    /// — função `aguardar_mitigacao_de_tempo`; nunca aqui no núcleo, ver "Fronteira deliberada" na
    /// doc do módulo), que reduzem a TAXA de amostras de tempo que um atacante consegue colher, sem
    /// eliminar a exposição teórica residual do crate em si (continua sem patch upstream). Deixado
    /// de fora de propósito: identificar o pagante por conta/IP para limitar por identidade quebraria
    /// a garantia de anonimato que esta função existe para proteger — por isso o espaçamento é
    /// servidor inteiro, não por identidade. Monitoramento ativo de
    /// `RustCrypto/RSA#626`/`#680`/`#702` continua recomendado. Narrativa completa da decisão
    /// original e da implementação, incluindo por que troca de primitiva NÃO é recomendada agora,
    /// em `PENDENCIAS_E_DECISOES.md` §50 e §52. Não tratar esta função como "pronta para produção
    /// sem ressalva": o espaçamento reduz a taxa de coleta, não prova ausência de vazamento.
    pub fn assinar_cego(
        &mut self,
        pagamento: &PagamentoConfirmado,
        msg_cega: &blind_rsa_signatures::BlindMessage,
    ) -> Result<BlindSignature, ErroDeCobranca> {
        if self.eventos_ja_usados.contains(&pagamento.id_do_evento) {
            return Err(ErroDeCobranca::PagamentoJaUsado);
        }
        let sk = self
            .chave_secreta(pagamento.plano, pagamento.periodo)
            .ok_or(ErroDeCobranca::ChaveDoPeriodoInexistente)?;
        let assinatura = sk.blind_sign(msg_cega).map_err(|_| ErroDeCobranca::FalhaCriptografica)?;
        // Só marca o evento como usado DEPOIS de assinar com sucesso — se a assinatura falhar, o
        // chamador pode tentar de novo com o mesmo evento, sem precisar de um novo pagamento.
        let _ = self.eventos_ja_usados.insert(pagamento.id_do_evento.clone());
        Ok(assinatura)
    }
}

/// Lado do cliente, passo 1: gera um token aleatório de 32 bytes e o cega contra a chave pública
/// do plano/período desejado, antes de mandar ao emissor. O token em claro e o `BlindingResult`
/// (que carrega o segredo de descegar) ficam só no cliente — nunca saem daqui.
pub fn preparar_pedido_de_ficha(pk: &ChavePublicaDeEmissao) -> Result<(Vec<u8>, BlindingResult), ErroDeCobranca> {
    let mut token = vec![0u8; 32];
    OsRng.fill_bytes(&mut token);
    let resultado = pk.blind(&mut DefaultRng, &token).map_err(|_| ErroDeCobranca::FalhaCriptografica)?;
    Ok((token, resultado))
}

/// Lado do cliente, passo 2: depois de receber `BlindSignature` do emissor, desfaz a cegueira e
/// devolve a `Ficha` pronta — isto É a "ficha anônima de licença" que `autenticacao::Porteiro`
/// espera (hoje representada ali só como `ficha_valida: bool`; `ficha_e_valida`, abaixo, é quem
/// produz esse booleano a partir da ficha real). `finalize` já confere a assinatura por dentro
/// (biblioteca) antes de devolver — uma ficha que sai daqui com `Ok` já está provada válida.
pub fn finalizar_ficha(
    pk: &ChavePublicaDeEmissao,
    assinatura_cega: &BlindSignature,
    resultado_de_cegar: &BlindingResult,
    token: Vec<u8>,
) -> Result<Ficha, ErroDeCobranca> {
    let assinatura = pk
        .finalize(assinatura_cega, resultado_de_cegar, &token)
        .map_err(|_| ErroDeCobranca::FichaInvalida)?;
    Ok(Ficha { token, assinatura, randomizador: resultado_de_cegar.msg_randomizer })
}

/// Verificação: qualquer um com a chave pública do `(plano, período)` confirma que uma ficha é
/// válida — sem jamais ter visto o pagamento que a originou. É isto que alimenta o
/// `ficha_valida: bool` que `autenticacao::Porteiro::autorizar` já espera na rota `Ficha`.
pub fn ficha_e_valida(pk: &ChavePublicaDeEmissao, ficha: &Ficha) -> bool {
    pk.verify(&ficha.assinatura, ficha.randomizador, &ficha.token).is_ok()
}

/// Serializa a chave pública de um `(plano, período)` no ÚNICO formato DER que o `WebCrypto` do
/// navegador aceita para importar uma chave RSA-PSS — é isto que o servidor de pagamento manda ao
/// cliente (junto com o `BillingSetupIntent`/preço do plano) para que o navegador cegue o token
/// (`@cloudflare/blindrsa-ts`, `RSABSSA.SHA384.PSS.Randomized()`, mesma suíte deste módulo).
///
/// **Por que esta função existe, em vez de cada chamador decidir sozinho entre
/// `PublicKey::to_der()` e `PublicKey::to_spki()` (ambos públicos na biblioteca):** os dois
/// produzem SPKI DER válido, mas DIFERENTE — e só um dos dois funciona num navegador. Achado e
/// provado empiricamente na Rodada 24 (`examples/interop_cobranca_js.rs`), depois de uma
/// investigação que começou com `to_spki()` (o nome mais "oficial"-parecendo) falhando no
/// navegador:
///
/// * `to_spki()` empacota deliberadamente o OID `rsassaPss` (1.2.840.113549.1.1.10) com os
///   parâmetros de PSS (hash, MGF, comprimento do salt) embutidos dentro do próprio DER — uma
///   forma SPKI mais auto-descritiva, mas que `crypto.subtle.importKey('spki', ..., {name:
///   'RSA-PSS', hash: 'SHA-384'})` RECUSA, com `DOMException DataError: Invalid key type`.
/// * `to_der()` usa o OID genérico `rsaEncryption` (1.2.840.113549.1.1.1) — é o que
///   `rsa::RsaPublicKey::to_public_key_der()` (RustCrypto, a biblioteca de base por dentro de
///   `blind-rsa-signatures`) produz por padrão para qualquer chave RSA, sem nada específico de
///   PSS. É exatamente essa forma que o `crypto.subtle.generateKey({name: 'RSA-PSS', ...})` do
///   PRÓPRIO navegador usa para exportar as chaves que ele mesmo gera (confirmado comparando com a
///   saída de `crypto.subtle.exportKey('spki', ...)` sobre uma chave gerada no navegador) — o
///   WebCrypto deixa hash/MGF/salt como parâmetro de `importKey`/`sign`/`verify`, nunca dentro do
///   DER. É esta forma que esta função expõe.
///
/// A segunda causa (irmã desta, achada na mesma investigação, mas do lado do chamador, não desta
/// função): ao importar esta chave, o navegador precisa pedir o uso `['verify']` explicitamente
/// (`crypto.subtle.importKey(..., ['verify'])`) — uma chave importada com usos vazios compila e
/// importa sem erro, mas falha mais tarde, dentro de `finalize()` (que verifica por dentro), com
/// `DOMException InvalidAccessError: Unable to use this key to verify`. Isto não é responsabilidade
/// desta função Rust (é uma chamada só do lado do navegador), mas fica registrado aqui porque as
/// duas causas, juntas, levaram ao mesmo sintoma inicial e só a investigação completa revelou que
/// eram duas, não uma.
pub fn chave_publica_para_navegador(pk: &ChavePublicaDeEmissao) -> Result<Vec<u8>, ErroDeCobranca> {
    pk.to_der().map_err(|_| ErroDeCobranca::FalhaCriptografica)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn periodo_exemplo() -> Periodo {
        Periodo { ano: 2026, mes: 11 }
    }

    #[test]
    fn periodo_valida_mes_1_a_12_e_calcula_o_proximo_corretamente_inclusive_na_virada_do_ano() {
        assert!(Periodo { ano: 2026, mes: 1 }.valido());
        assert!(Periodo { ano: 2026, mes: 12 }.valido());
        assert!(!Periodo { ano: 2026, mes: 0 }.valido());
        assert!(!Periodo { ano: 2026, mes: 13 }.valido());
        assert_eq!(Periodo { ano: 2026, mes: 11 }.proximo(), Periodo { ano: 2026, mes: 12 });
        assert_eq!(Periodo { ano: 2026, mes: 12 }.proximo(), Periodo { ano: 2027, mes: 1 });
    }

    #[test]
    fn ida_e_volta_emite_uma_ficha_valida_contra_a_chave_certa() {
        let mut emissor = EmissorDeFichas::novo();
        emissor.gerar_chave_do_periodo(PlanoPago::Pro, periodo_exemplo()).unwrap();
        let pk = emissor.chave_publica(PlanoPago::Pro, periodo_exemplo()).unwrap().clone();

        let (token, resultado) = preparar_pedido_de_ficha(&pk).unwrap();
        let pagamento = PagamentoConfirmado { id_do_evento: b"evt_1".to_vec(), plano: PlanoPago::Pro, periodo: periodo_exemplo() };
        let assinatura_cega = emissor.assinar_cego(&pagamento, &resultado.blind_message).unwrap();
        let ficha = finalizar_ficha(&pk, &assinatura_cega, &resultado, token).unwrap();

        assert!(ficha_e_valida(&pk, &ficha));
    }

    #[test]
    fn ficha_de_um_plano_nao_vale_para_outro_plano_nem_para_outro_periodo() {
        let mut emissor = EmissorDeFichas::novo();
        emissor.gerar_chave_do_periodo(PlanoPago::Pro, periodo_exemplo()).unwrap();
        emissor.gerar_chave_do_periodo(PlanoPago::Max, periodo_exemplo()).unwrap();
        emissor.gerar_chave_do_periodo(PlanoPago::Pro, periodo_exemplo().proximo()).unwrap();

        let pk_pro = emissor.chave_publica(PlanoPago::Pro, periodo_exemplo()).unwrap().clone();
        let (token, resultado) = preparar_pedido_de_ficha(&pk_pro).unwrap();
        let pagamento = PagamentoConfirmado { id_do_evento: b"evt_2".to_vec(), plano: PlanoPago::Pro, periodo: periodo_exemplo() };
        let assinatura_cega = emissor.assinar_cego(&pagamento, &resultado.blind_message).unwrap();
        let ficha = finalizar_ficha(&pk_pro, &assinatura_cega, &resultado, token).unwrap();

        // Válida contra a própria chave.
        assert!(ficha_e_valida(&pk_pro, &ficha));
        // Inválida contra a chave do Max do MESMO período (um atacante não pode "promover" a
        // própria ficha de Pro para Max só reapresentando-a contra outra chave pública).
        let pk_max = emissor.chave_publica(PlanoPago::Max, periodo_exemplo()).unwrap().clone();
        assert!(!ficha_e_valida(&pk_max, &ficha));
        // Inválida contra a chave do Pro do PRÓXIMO período (uma ficha não sobrevive à renovação
        // sozinha — precisa de uma nova emissão a cada período, mesmo dentro do mesmo plano).
        let pk_pro_proximo = emissor.chave_publica(PlanoPago::Pro, periodo_exemplo().proximo()).unwrap().clone();
        assert!(!ficha_e_valida(&pk_pro_proximo, &ficha));
    }

    #[test]
    fn mesmo_evento_de_pagamento_nunca_emite_duas_fichas() {
        let mut emissor = EmissorDeFichas::novo();
        emissor.gerar_chave_do_periodo(PlanoPago::Max, periodo_exemplo()).unwrap();
        let pk = emissor.chave_publica(PlanoPago::Max, periodo_exemplo()).unwrap().clone();

        let (_t1, r1) = preparar_pedido_de_ficha(&pk).unwrap();
        let pagamento = PagamentoConfirmado { id_do_evento: b"evt_so_uma_vez".to_vec(), plano: PlanoPago::Max, periodo: periodo_exemplo() };
        assert!(emissor.assinar_cego(&pagamento, &r1.blind_message).is_ok());

        // Mesmo id_do_evento de novo (ex.: webhook duplicado, reentrega do processador) — mesmo
        // com um token DIFERENTE cegado, a segunda tentativa é recusada.
        let (_t2, r2) = preparar_pedido_de_ficha(&pk).unwrap();
        assert_eq!(emissor.assinar_cego(&pagamento, &r2.blind_message), Err(ErroDeCobranca::PagamentoJaUsado));
    }

    #[test]
    fn assinar_sem_chave_gerada_para_o_periodo_e_recusado() {
        let mut emissor = EmissorDeFichas::novo();
        emissor.gerar_chave_do_periodo(PlanoPago::Pro, periodo_exemplo()).unwrap();
        let pk = emissor.chave_publica(PlanoPago::Pro, periodo_exemplo()).unwrap().clone();
        let (_t, r) = preparar_pedido_de_ficha(&pk).unwrap();

        // Pagamento confirmado para um período cuja chave NUNCA foi gerada nesta instância.
        let pagamento = PagamentoConfirmado { id_do_evento: b"evt_3".to_vec(), plano: PlanoPago::Pro, periodo: periodo_exemplo().proximo() };
        assert_eq!(emissor.assinar_cego(&pagamento, &r.blind_message), Err(ErroDeCobranca::ChaveDoPeriodoInexistente));
    }

    #[test]
    fn gerar_chave_com_mes_invalido_e_recusado() {
        let mut emissor = EmissorDeFichas::novo();
        assert_eq!(emissor.gerar_chave_do_periodo(PlanoPago::Pro, Periodo { ano: 2026, mes: 13 }), Err(ErroDeCobranca::PeriodoInvalido));
        assert_eq!(emissor.gerar_chave_do_periodo(PlanoPago::Pro, Periodo { ano: 2026, mes: 0 }), Err(ErroDeCobranca::PeriodoInvalido));
    }

    #[test]
    fn chave_publica_para_navegador_usa_o_oid_generico_rsa_encryption_nunca_o_oid_rsassapss() {
        // Trava por bytes, não só "importa sem erro": um regresso futuro para `to_spki()` (ex.:
        // alguém "simplificando" para a chamada que parece mais oficial) precisa ser pego aqui,
        // sem depender de rodar `examples/interop_cobranca_js.rs` (que exige `node` instalado e
        // não roda em `cargo test`). Ver a doc da função para a investigação completa (Rodada 24).
        let mut emissor = EmissorDeFichas::novo();
        emissor.gerar_chave_do_periodo(PlanoPago::Pro, periodo_exemplo()).unwrap();
        let pk = emissor.chave_publica(PlanoPago::Pro, periodo_exemplo()).unwrap().clone();
        let der = chave_publica_para_navegador(&pk).unwrap();

        // Estrutura fixa de um AlgorithmIdentifier de RSA genérico, sempre nos mesmos deslocamentos
        // (30 82 LL LL = cabeçalho da SEQUENCE externa, sempre 4 bytes para uma chave de 2048 bits):
        //   byte  4.. 5 : 30 0D            (SEQUENCE, 13 bytes — o AlgorithmIdentifier completo)
        //   byte  6.. 7 : 06 09            (OID, 9 bytes de conteúdo)
        //   byte  8..16 : 2A 86 48 86 F7 0D 01 01 01   (rsaEncryption — NUNCA termina em 0A, que
        //                                                seria rsassaPss)
        //   byte 17..18 : 05 00            (NULL — parâmetros do AlgorithmIdentifier, vazios)
        const OID_RSA_ENCRYPTION: [u8; 11] = [0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];
        const OID_RSASSA_PSS: [u8; 11] = [0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0a];
        assert!(der.len() > 19, "DER curto demais para conter um AlgorithmIdentifier: {} bytes", der.len());
        assert_eq!(&der[6..17], &OID_RSA_ENCRYPTION, "esperava o OID genérico rsaEncryption no deslocamento fixo do AlgorithmIdentifier");
        assert_eq!(&der[17..19], [0x05, 0x00], "esperava NULL (parâmetros vazios) logo após o OID genérico");
        assert_ne!(&der[6..17], &OID_RSASSA_PSS, "regresso: isto é to_spki(), não to_der() — o navegador recusa este formato (ver doc da função)");

        // E, claro: a chave continua funcionalmente idêntica à do fluxo normal — isto não é um
        // formato "só para o navegador" que perdeu a capacidade de cegar/verificar no próprio Rust.
        let (token, resultado) = preparar_pedido_de_ficha(&pk).unwrap();
        let pagamento = PagamentoConfirmado { id_do_evento: b"evt_der_navegador".to_vec(), plano: PlanoPago::Pro, periodo: periodo_exemplo() };
        let assinatura_cega = emissor.assinar_cego(&pagamento, &resultado.blind_message).unwrap();
        let ficha = finalizar_ficha(&pk, &assinatura_cega, &resultado, token).unwrap();
        assert!(ficha_e_valida(&pk, &ficha));
    }

    #[test]
    fn ficha_reconstruida_so_dos_bytes_crus_que_um_cliente_mandaria_pela_rede_ainda_verifica() {
        // Prova o contrato de fio documentado em `Ficha`: um "porteiro"/gateway em Rust nunca
        // recebe um `BlindingResult` nem nada do nosso próprio fluxo de teste — recebe só bytes
        // crus (token, assinatura, randomizador) que vieram de algum cliente, possivelmente em
        // JavaScript. Este teste simula exatamente isso: produz a ficha pelo fluxo normal, acha
        // os bytes crus de cada campo, DESCARTA os valores tipados originais, e reconstrói uma
        // `Ficha` nova só a partir dos três `Vec<u8>`/`[u8;32]` — do jeito que `serde_json` ou
        // qualquer desserialização de uma requisição HTTP produziria.
        let mut emissor = EmissorDeFichas::novo();
        emissor.gerar_chave_do_periodo(PlanoPago::Max, periodo_exemplo()).unwrap();
        let pk = emissor.chave_publica(PlanoPago::Max, periodo_exemplo()).unwrap().clone();

        let (token, resultado) = preparar_pedido_de_ficha(&pk).unwrap();
        let pagamento = PagamentoConfirmado { id_do_evento: b"evt_reconstrucao".to_vec(), plano: PlanoPago::Max, periodo: periodo_exemplo() };
        let assinatura_cega = emissor.assinar_cego(&pagamento, &resultado.blind_message).unwrap();
        let ficha_original = finalizar_ficha(&pk, &assinatura_cega, &resultado, token).unwrap();

        // Bytes crus, exatamente como um corpo de requisição HTTP os carregaria (base64 decodado).
        let token_bytes: Vec<u8> = ficha_original.token.clone();
        let assinatura_bytes: Vec<u8> = ficha_original.assinatura.0.clone();
        let randomizador_bytes: Option<[u8; 32]> = ficha_original.randomizador.map(|r| r.0);

        // Reconstrução, do jeito documentado — construção direta dos campos públicos, sem
        // nenhuma função de conversão (porque não existe, nem precisa existir, nenhuma).
        let ficha_reconstruida = Ficha {
            token: token_bytes,
            assinatura: Signature(assinatura_bytes),
            randomizador: randomizador_bytes.map(MessageRandomizer),
        };

        assert!(ficha_e_valida(&pk, &ficha_reconstruida));
    }

    #[test]
    fn token_adulterado_apos_cegar_nao_produz_ficha_valida() {
        let mut emissor = EmissorDeFichas::novo();
        emissor.gerar_chave_do_periodo(PlanoPago::Pro, periodo_exemplo()).unwrap();
        let pk = emissor.chave_publica(PlanoPago::Pro, periodo_exemplo()).unwrap().clone();

        let (mut token, resultado) = preparar_pedido_de_ficha(&pk).unwrap();
        let pagamento = PagamentoConfirmado { id_do_evento: b"evt_4".to_vec(), plano: PlanoPago::Pro, periodo: periodo_exemplo() };
        let assinatura_cega = emissor.assinar_cego(&pagamento, &resultado.blind_message).unwrap();

        // Adultera o token DEPOIS de cegar, antes de descegar — finalizar deve falhar, nunca
        // produzir silenciosamente uma ficha para um token diferente do que foi realmente assinado.
        token[0] ^= 0x01;
        assert!(finalizar_ficha(&pk, &assinatura_cega, &resultado, token).is_err());
    }
}
