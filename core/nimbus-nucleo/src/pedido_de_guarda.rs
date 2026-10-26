//! # Módulo `pedido_de_guarda` — destinatário pede, remetente aprova ou nega (novo, 10/10/2026)
//!
//! Resposta a uma distinção que o dono pediu explicitamente ao ver o mapa das três camadas desta rodada:
//! a aba trancada e o cofre pessoal não podem ser confundidos, e merecem fluxos de aprovação DIFERENTES.
//!
//! **Atualização de 11/10/2026 (Rodada 20) — a frase acima ("não tocada aqui") deixou de valer para a
//! ABA TRANCADA EM SI; `custodia_mutua.rs` continua, sim, sem nenhuma mudança de código.** Pedido do
//! dono nesta rodada: entrar na aba trancada também passa a exigir pedido de quem não tem o item e
//! aprovação de quem já tem (regra 34 revista — ver `RespostaDeAbaTrancada`/`responder_pedido_de_aba_
//! trancada`, abaixo). O que NÃO mudou, de propósito, para não regredir o desenho mais forte que
//! `custodia_mutua.rs` já prova por teste: o par de chaves descartável por item, só gerado dentro de
//! `custodia_mutua::encaminhar`; esta função de pedido/aprovação só decide SE e com qual prazo essa
//! chamada deve acontecer — nunca produz ela mesma uma `liberacao::Autorizacao` (continua valendo,
//! também depois desta rodada, que `AbaTrancada` não é um `Uso`; ver `uso_correspondente`) nem chama
//! `custodia_mutua::encaminhar` diretamente (quem faz a chamada de verdade é a ponte, com o `Item` em
//! mãos — este núcleo nunca guarda conteúdo, só decide a autorização, mesmo princípio de sempre).
//!
//! O parágrafo original desta rodada anterior, sem alteração de sentido (ainda descreve exatamente o
//! fluxo do COFRE PESSOAL, abaixo): a aba trancada já funcionava por autorização de encaminhamento dada
//! pelo autor (ver `liberacao`), com controle dos DOIS lados para sempre (qualquer um apaga para os dois
//! a qualquer momento) — isto continua verdade; o que se soma agora é só um pedido explícito antes de o
//! item entrar lá, não uma substituição do mecanismo de controle mútuo já existente.
//!
//! **Guardar no cofre PESSOAL de quem recebe é uma categoria de decisão diferente: uma cópia permanente e
//! unilateral — o autor perde controle sobre ela dali para frente** (ao contrário da aba trancada). Por
//! isso, em vez de depender só de uma autorização genérica concedida de antemão pelo autor, este módulo
//! deixa o DESTINATÁRIO solicitar, item por item, no momento em que realmente quer guardar — e o autor
//! aprova ou nega, também item por item, vendo exatamente o que está sendo pedido. Pedido explícito, por
//! item, bate com o resto do núcleo: nunca uma permissão genérica, de propósito (mesmo espírito de
//! `custodia_por_categoria::definir`, nesta mesma rodada, que também proíbe um "ligar tudo de uma vez").
//!
//! **Zero criptografia nova.** O pedido é um objeto pequeno, assinado pelo próprio destinatário (mesma
//! assinatura híbrida de sempre). Aprovar não inventa um mecanismo de permissão novo: produz exatamente uma
//! `liberacao::Autorizacao` normal, `Nivel::GuardarNoCofre` — a mesma peça que `liberacao::pode` e
//! `liberacao::avisar_o_autor` já tratam em todo o resto do núcleo (a notificação ao autor, corrigida nesta
//! mesma rodada, continua valendo sem mudança a partir do momento em que o item é de fato guardado — pedir
//! e aprovar não guarda nada sozinho, só autoriza). Negar não produz `Autorizacao` nenhuma: sem ela,
//! `liberacao::nivel_autorizado` continua devolvendo `Nivel::Nenhum`, o mesmo padrão de sempre — recusa não
//! é um estado especial, é a ausência do único objeto que concede acesso.
//!
//! **O que nunca viaja aqui, para não haver o que expor (pedido explícito do dono: "sem deixar exposto
//! qualquer dado do remetente, nunca"):** o pedido carrega só o hash do conteúdo (como qualquer
//! `Autorizacao`) — nunca o conteúdo, nunca metadado de quem é a conversa. Como qualquer objeto deste
//! núcleo, quem vê o pedido e a resposta é só quem já está nos dois lados do canal 1:1 já cifrado (fora
//! deste núcleo, que nunca trata transporte); o servidor, se algum dia vir o bloco, vê algo opaco — nunca um
//! registro dizendo "fulano pediu a beltrano", igual a `custodia_mutua`.
//!
//! **O que isto NÃO prova (fica para quem integrar):** nenhum prazo de expiração do pedido em si — ao
//! contrário de `comando_remoto` (onde ninguém necessariamente está olhando quando o comando chega) ou de
//! `aprovacao::Pedido` (aprovação cumulativa entre várias pessoas, que de verdade precisa de prazo), aqui um
//! humano só decide, em tempo real, vendo o pedido na própria tela — expirar automaticamente é decisão de
//! UX (ex.: esconder pedidos com mais de N dias sem resposta), não uma garantia que faltasse ao núcleo;
//! nenhuma recusa assinada (a `Resposta::Negado` não carrega prova criptográfica da negativa — só a
//! ausência de autorização já basta para o núcleo recusar o acesso; uma recusa assinada e não-repudiável
//! seria útil só para uma disputa entre as duas pessoas, fora do escopo de segurança deste pedido).
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use crate::custodia_mutua::PrazoDeCustodiaMutua;
use crate::liberacao::{self, Autorizacao, Destino};

const CONTEXTO: &[u8] = b"NimbusOne/v1/pedido_de_guarda";

/// Pedido assinado por quem RECEBEU o conteúdo, pedindo para guardá-lo no próprio cofre pessoal.
#[derive(Clone, Debug)]
pub struct Pedido {
    pub hash_do_conteudo: [u8; 32],
    /// Impressão digital da chave de quem pede — a ponte resolve isto para saber a quem mostrar o pedido.
    pub solicitante: [u8; 32],
    pub criado_em: u64,
    pub assinatura: AssinaturaHibrida,
}

fn bytes_do_pedido(hash: &[u8; 32], criado_em: u64) -> Vec<u8> {
    let mut v = hash.to_vec();
    v.extend_from_slice(&criado_em.to_be_bytes());
    v
}

/// No aparelho de quem RECEBEU o conteúdo: monta o pedido.
pub fn solicitar(quem_pede: &ChaveDeAssinatura, hash_do_conteudo: [u8; 32], criado_em: u64) -> Pedido {
    let assinatura = quem_pede
        .assinar(CONTEXTO, &bytes_do_pedido(&hash_do_conteudo, criado_em))
        .expect("contexto fixo e curto");
    Pedido { hash_do_conteudo, solicitante: quem_pede.verificacao().impressao_digital(), criado_em, assinatura }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDePedido {
    /// Cobre tanto "não foi este solicitante que assinou" quanto "o pedido foi alterado depois de
    /// assinado" (ex.: trocar o hash do conteúdo) — qualquer mudança já invalida a assinatura.
    AssinaturaInvalida,
    /// **Achado na autovalidação desta rodada, corrigido antes de existir qualquer tela usando isto.**
    /// `Destino::AbaTrancada` não tem `Uso` correspondente — pedir para este destino aqui é erro de quem
    /// chama (a ponte nunca deveria oferecer este pedido para este destino), nunca uma decisão do autor.
    /// Primeira versão desta função devolvia `Resposta::Negado` neste caso — errado: `Negado` é
    /// documentado como "o autor negou, explícito, nunca silêncio", e isto atribuiria a uma PESSOA real
    /// uma recusa que ela nunca fez. Um erro próprio evita que a ponte um dia mostre "Fulano recusou seu
    /// pedido" quando, na verdade, nenhum humano decidiu nada.
    DestinoSemAutorizacaoGenerica,
}

fn validar(chave_do_solicitante: &ChaveDeVerificacao, p: &Pedido) -> Result<(), ErroDePedido> {
    let bytes = bytes_do_pedido(&p.hash_do_conteudo, p.criado_em);
    if p.solicitante != chave_do_solicitante.impressao_digital() || !chave_do_solicitante.verificar(CONTEXTO, &bytes, &p.assinatura) {
        return Err(ErroDePedido::AssinaturaInvalida);
    }
    Ok(())
}

/// Deliberadamente sem `PartialEq`/`Eq`: carrega uma `Autorizacao` (que também não deriva — ver
/// `liberacao.rs`), para não convidar um `==` ingênuo no lugar da validação de verdade
/// (`liberacao::nivel_autorizado`/`pode`). Testes comparam por `matches!`, não por igualdade.
#[derive(Debug)]
pub enum Resposta {
    /// O autor aprovou: a autorização de verdade, pronta para o destinatário guardar (mesmo caminho de
    /// `liberacao::pode`/`avisar_o_autor` de sempre — guardar em si só acontece depois, como ação própria).
    Autorizado(Autorizacao),
    /// O autor negou — explícito, nunca silêncio: sem isto, o destinatário não teria como distinguir
    /// "ainda pendente" de "já recusado".
    Negado,
}

/// No aparelho de quem ENVIOU o conteúdo originalmente, depois de ver o pedido e decidir — para qualquer
/// um dos três destinos da tela única de liberação (`liberacao::Destino`, pedido do dono em 11/10/2026:
/// unificar a mesma pergunta "destinatário pede, autor aprova" para os três, não só o cofre).
/// `responder` (abaixo) continua existindo sem mudança, agora como um atalho deste para `CofreNimbusOne`.
///
/// **`AbaTrancada` nunca chega a produzir uma `Autorizacao` POR ESTA FUNÇÃO** — `uso_correspondente`
/// devolve `None` de propósito (é o mecanismo próprio de `custodia_mutua`, regra 34); chamar esta função
/// com `AbaTrancada` devolve `Err(ErroDePedido::DestinoSemAutorizacaoGenerica)`, nunca `Resposta::Negado`
/// — é erro de quem chama (a ponte: deveria ter chamado `responder_pedido_de_aba_trancada`, abaixo, não
/// esta função genérica), não uma decisão do autor, e os dois nunca podem ser confundidos (ver o
/// comentário do próprio erro: a primeira versão desta função cometia exatamente essa confusão, corrigida
/// na autovalidação). **Atualização de 11/10/2026:** isto não quer dizer mais que a aba trancada nunca
/// passa por pedido/aprovação NENHUM — só que não passa por ESTE caminho genérico, que é feito sob
/// medida para destinos que terminam numa `liberacao::Autorizacao` normal; a aba trancada tem o próprio
/// caminho, com o próprio tipo de resposta, exatamente porque o desenho é diferente (ver abaixo).
///
/// **`ForaDoNimbus` continua sendo construído e assinado normalmente** — a assinatura em si nunca foi o
/// controle de verdade (nem para os outros usos: `liberacao::autorizar` sempre pôde assinar qualquer
/// `Nivel`). O controle real é o teto da conversa, aplicado depois, em `liberacao::efetivo`/`pode` — hoje
/// sempre abaixo de `LevarParaFora` (v37, terceira rodada, decisão permanente do dono). Ver
/// `liberacao::destino_bloqueado_pelo_teto`: é isso que a ponte deve checar ANTES de nem oferecer o botão
/// "outros aplicativos" como algo que vai funcionar — este módulo não duplica aquele teto aqui dentro para
/// não ter duas fontes de verdade sobre o mesmo limite.
///
/// `versao` vem de quem chama (a ponte), igual a qualquer outra autorização dada "depois do envio" — ver
/// `liberacao::autorizar`, que já suporta isso nativamente; este módulo não inventa versionamento próprio.
pub fn responder_para_destino(
    chave_do_solicitante: &ChaveDeVerificacao,
    pedido: &Pedido,
    autor: &ChaveDeAssinatura,
    versao: u64,
    aprovar: bool,
    prazo_segundos: Option<u64>,
    destino: Destino,
) -> Result<Resposta, ErroDePedido> {
    validar(chave_do_solicitante, pedido)?;
    let Some(uso) = liberacao::uso_correspondente(destino) else {
        // AbaTrancada: não é um Uso, não produz Autorizacao — e NUNCA é "o autor negou" (ver
        // ErroDePedido::DestinoSemAutorizacaoGenerica). Checado ANTES de `aprovar`: mesmo que a ponte
        // tivesse passado `aprovar: true`, o erro de chamada é o mesmo — não depende da decisão do autor.
        return Err(ErroDePedido::DestinoSemAutorizacaoGenerica);
    };
    if !aprovar {
        return Ok(Resposta::Negado);
    }
    let nivel = liberacao::exigido(uso);
    let autorizacao = liberacao::autorizar(autor, pedido.hash_do_conteudo, nivel, versao, prazo_segundos, false);
    Ok(Resposta::Autorizado(autorizacao))
}

/// Atalho de `responder_para_destino` para o cofre pessoal — mesma assinatura de antes desta rodada,
/// nenhum teste existente precisou mudar.
pub fn responder(
    chave_do_solicitante: &ChaveDeVerificacao,
    pedido: &Pedido,
    autor: &ChaveDeAssinatura,
    versao: u64,
    aprovar: bool,
    prazo_segundos: Option<u64>,
) -> Result<Resposta, ErroDePedido> {
    responder_para_destino(chave_do_solicitante, pedido, autor, versao, aprovar, prazo_segundos, Destino::CofreNimbusOne)
}

// ---------------------------------------------------------------------------------------------------------
// Aba trancada: pedido e aprovação (regra 34 revista — Rodada 20, 11/10/2026, pedido explícito do dono:
// "para pasta trancada também tem de ter solicitação"). Ver a atualização na doc do módulo, no alto deste
// arquivo, para o porquê de ser um caminho PRÓPRIO, não uma extensão de `responder_para_destino`.
// ---------------------------------------------------------------------------------------------------------

/// Resposta a um pedido de mover algo para a aba trancada. Deliberadamente NÃO é `Resposta` (acima):
/// `AbaTrancada` continua, mesmo depois desta rodada, nunca produzindo uma `liberacao::Autorizacao` — só
/// `Aprovado`/`Negado`, cada um carregando exatamente o que a ponte precisa para o próximo passo, nada
/// mais. Mesma razão de sempre para não derivar `PartialEq` em nada que carregue algo de
/// `custodia_mutua` (consistência com o resto do núcleo); testes comparam por `matches!`.
#[derive(Debug)]
pub enum RespostaDeAbaTrancada {
    /// Quem já tem o item aprovou e escolheu o prazo (1 a 182 dias, mesmo grampeamento de sempre de
    /// `custodia_mutua::PrazoDeCustodiaMutua::de_dias` — não um prazo novo, o mesmo). **O que a ponte faz
    /// a seguir, fora deste núcleo:** chama `custodia_mutua::encaminhar(servidor, item, agora, prazo)` ali
    /// mesmo, no aparelho de quem aprovou (é quem tem o `Item` em mãos) — exatamente como já acontecia
    /// antes desta rodada, só que agora depois de um pedido explícito, não automaticamente. A chamada
    /// devolve uma `Referencia`; quem aprovou manda os bytes dela (`Referencia::para_bytes`) para quem
    /// pediu, pelo canal 1:1 já cifrado — o mesmo transporte que a aba trancada sempre usou. **Por isso
    /// quem aprova também acaba com uma cópia/referência própria na própria aba trancada:** a chamada que
    /// cria o item roda no aparelho de quem aprova antes de qualquer coisa viajar — não é um efeito
    /// colateral novo desta rodada, é a mesma garantia mútua que `custodia_mutua` já provava por teste
    /// (`qualquer_um_dos_dois_lados_abre_com_a_propria_copia_da_referencia`) muito antes de existir
    /// qualquer pedido.
    Aprovado(PrazoDeCustodiaMutua),
    /// Quem já tem o item negou — explícito, nunca silêncio, mesmo raciocínio de `Resposta::Negado`.
    Negado,
}

/// No aparelho de quem JÁ TEM o item (quem vai decidir se ele entra na aba trancada), depois de ver o
/// pedido de quem ainda não tem. `prazo` só é usado quando `aprovar` é `true` — é quem aprova que escolhe
/// o prazo, nunca quem pede (mesmo princípio de sempre: pedir não é decidir).
///
/// Zero criptografia nova, mesmo espírito do resto deste módulo: `Pedido`/`solicitar`/`validar` já eram
/// genéricos (carregam só um hash, nunca mencionam destino) — reaproveitados aqui sem nenhuma mudança.
pub fn responder_pedido_de_aba_trancada(
    chave_do_solicitante: &ChaveDeVerificacao,
    pedido: &Pedido,
    aprovar: bool,
    prazo: PrazoDeCustodiaMutua,
) -> Result<RespostaDeAbaTrancada, ErroDePedido> {
    validar(chave_do_solicitante, pedido)?;
    if !aprovar {
        return Ok(RespostaDeAbaTrancada::Negado);
    }
    Ok(RespostaDeAbaTrancada::Aprovado(prazo))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;
    use crate::liberacao::Nivel;
    use crate::politica::TipoDeConteudo;

    const H: [u8; 32] = [7; 32];

    #[test]
    fn pedido_valido_aprovado_vira_autorizacao_de_guardar_no_cofre_e_o_nucleo_normal_ja_sabe_validar() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let pedido = solicitar(&destinatario, H, 1_000);
        let resp = responder(&destinatario.verificacao(), &pedido, &autor, 2, true, None).unwrap();
        let Resposta::Autorizado(a) = resp else { panic!("deveria ter aprovado") };
        assert_eq!(a.nivel, Nivel::GuardarNoCofre);
        assert_eq!(a.hash_do_conteudo, H);
        // A autorização resultante não é um objeto especial deste módulo: é a peça normal que o resto do
        // núcleo (liberacao::nivel_autorizado, liberacao::pode, avisar_o_autor) já sabe validar sem mudar nada.
        assert_eq!(
            liberacao::nivel_autorizado(&autor.verificacao(), std::slice::from_ref(&a), &H, TipoDeConteudo::Foto, Some(0), 1),
            Nivel::GuardarNoCofre
        );
        assert!(liberacao::pode(TipoDeConteudo::Foto, Nivel::GuardarNoCofre, liberacao::Uso::GuardarNoCofre, false));
    }

    #[test]
    fn pedido_negado_nao_produz_autorizacao_nenhuma() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let pedido = solicitar(&destinatario, H, 1_000);
        assert!(matches!(responder(&destinatario.verificacao(), &pedido, &autor, 2, false, None).unwrap(), Resposta::Negado));
    }

    #[test]
    fn pedido_assinado_por_outra_pessoa_e_recusado() {
        let destinatario = assinatura::gerar();
        let outra_pessoa = assinatura::gerar();
        let autor = assinatura::gerar();
        let pedido = solicitar(&destinatario, H, 1_000);
        // Confere contra a chave errada: nem sempre quem diz ter pedido foi quem realmente assinou.
        assert!(matches!(responder(&outra_pessoa.verificacao(), &pedido, &autor, 2, true, None), Err(ErroDePedido::AssinaturaInvalida)));
    }

    #[test]
    fn campo_solicitante_forjado_e_recusado_mesmo_com_a_chave_de_verificacao_certa() {
        // Achado da campanha de força bruta desta rodada (11/10/2026): `bytes_do_pedido` nunca incluiu
        // `p.solicitante` no que é assinado (só hash + criado_em) — então o campo, por si, NÃO é
        // protegido pela assinatura. Sem a checagem de impressão digital em `validar`
        // (`p.solicitante != chave_do_solicitante.impressao_digital()`), um pedido genuinamente assinado
        // por A, com o campo `solicitante` depois trocado para a impressão digital de B, passaria sem
        // erro sempre que a ponte resolvesse a chave de verificação pelo canal/sessão de A (a chave
        // CERTA, que realmente assinou) em vez de resolvê-la a partir do próprio campo — a tela do autor
        // mostraria "B pediu isto", uma atribuição falsa, mesmo com uma assinatura tecnicamente válida.
        // Nenhum teste anterior cobria esta combinação: os testes de "assinado por outra pessoa" sempre
        // passavam uma chave de verificação ERRADA (que já falha em `verificar`, por si só, sem depender
        // desta checagem); este teste passa a chave CERTA (a que realmente assinou) com o campo forjado,
        // isolando exatamente o que a checagem de impressão digital defende, sozinha.
        let quem_pede_de_verdade = assinatura::gerar();
        let outra_pessoa = assinatura::gerar();
        let autor = assinatura::gerar();
        let mut pedido = solicitar(&quem_pede_de_verdade, H, 1_000);
        pedido.solicitante = outra_pessoa.verificacao().impressao_digital(); // forja o campo, assinatura intacta
        assert!(matches!(
            responder(&quem_pede_de_verdade.verificacao(), &pedido, &autor, 2, true, None),
            Err(ErroDePedido::AssinaturaInvalida)
        ), "campo solicitante forjado tem que ser recusado mesmo com a chave que realmente assinou");
    }

    #[test]
    fn pedido_adulterado_depois_de_assinado_e_recusado() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let mut pedido = solicitar(&destinatario, H, 1_000);
        pedido.hash_do_conteudo = [9; 32]; // tenta pedir guarda de outro conteúdo com a assinatura antiga
        assert!(matches!(responder(&destinatario.verificacao(), &pedido, &autor, 2, true, None), Err(ErroDePedido::AssinaturaInvalida)));
    }

    #[test]
    fn aprovacao_pode_vir_com_prazo_igual_a_qualquer_outra_autorizacao() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let pedido = solicitar(&destinatario, H, 1_000);
        let resp = responder(&destinatario.verificacao(), &pedido, &autor, 2, true, Some(3_600)).unwrap();
        assert!(matches!(resp, Resposta::Autorizado(a) if a.prazo_segundos == Some(3_600)));
    }

    // ---- generalização para os três destinos (11/10/2026) ----

    #[test]
    fn responder_e_responder_para_destino_cofre_dao_exatamente_o_mesmo_resultado() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let pedido = solicitar(&destinatario, H, 1_000);
        let r1 = responder(&destinatario.verificacao(), &pedido, &autor, 2, true, None).unwrap();
        let r2 = responder_para_destino(&destinatario.verificacao(), &pedido, &autor, 2, true, None, Destino::CofreNimbusOne).unwrap();
        assert!(matches!((r1, r2), (Resposta::Autorizado(a), Resposta::Autorizado(b)) if a.nivel == b.nivel && a.hash_do_conteudo == b.hash_do_conteudo));
    }

    #[test]
    fn destino_aba_trancada_e_erro_de_chamada_nunca_uma_recusa_atribuida_ao_autor() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let pedido = solicitar(&destinatario, H, 1_000);
        // Achado da autovalidação: a primeira versão desta função devolvia Resposta::Negado aqui — errado,
        // porque atribuiria ao AUTOR uma recusa que ele nunca deu. AbaTrancada não é um Uso (mecanismo
        // próprio de custodia_mutua, regra 34); é erro de quem chama, um Err distinto, nunca um Negado.
        // (Result<Resposta, _> não deriva PartialEq de propósito — ver doc de `Resposta` — por isso
        // comparamos só o erro, com unwrap_err().)
        assert_eq!(
            responder_para_destino(&destinatario.verificacao(), &pedido, &autor, 2, true, None, Destino::AbaTrancada).unwrap_err(),
            ErroDePedido::DestinoSemAutorizacaoGenerica
        );
        // Mesmo resultado com aprovar: false — o erro é da CHAMADA, não depende da decisão do autor.
        assert_eq!(
            responder_para_destino(&destinatario.verificacao(), &pedido, &autor, 2, false, None, Destino::AbaTrancada).unwrap_err(),
            ErroDePedido::DestinoSemAutorizacaoGenerica
        );
    }

    #[test]
    fn destino_fora_do_nimbus_e_assinado_mas_fica_sem_efeito_apos_o_teto_da_conversa() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let pedido = solicitar(&destinatario, H, 1_000);
        let resp = responder_para_destino(&destinatario.verificacao(), &pedido, &autor, 2, true, None, Destino::ForaDoNimbus).unwrap();
        let Resposta::Autorizado(a) = resp else { panic!("a assinatura em si sempre é possível — o bloqueio é no teto, não aqui") };
        assert_eq!(a.nivel, Nivel::LevarParaFora);
        // Prova de ponta a ponta: mesmo "aprovado" aqui, o teto permanente da conversa (v37, 3ª rodada)
        // nunca deixa isso virar uma liberação de verdade — liberacao::destino_bloqueado_pelo_teto já
        // avisa a ponte disso antes de nem oferecer o botão.
        let teto = liberacao::teto_da_conversa(None);
        assert!(liberacao::destino_bloqueado_pelo_teto(Destino::ForaDoNimbus, teto));
        let n = liberacao::nivel_autorizado(&autor.verificacao(), std::slice::from_ref(&a), &H, TipoDeConteudo::Foto, Some(0), 1);
        let ef = liberacao::efetivo(TipoDeConteudo::Foto, false, teto, n);
        assert!(!liberacao::pode(TipoDeConteudo::Foto, ef, liberacao::Uso::LevarParaFora, false), "nunca sai de verdade, hoje");
    }

    #[test]
    fn dois_pedidos_do_mesmo_destinatario_para_itens_diferentes_nao_se_confundem() {
        let destinatario = assinatura::gerar();
        let autor = assinatura::gerar();
        let h2 = [8; 32];
        let p1 = solicitar(&destinatario, H, 1_000);
        let p2 = solicitar(&destinatario, h2, 1_000);
        let r1 = responder(&destinatario.verificacao(), &p1, &autor, 2, true, None).unwrap();
        let r2 = responder(&destinatario.verificacao(), &p2, &autor, 3, false, None).unwrap();
        assert!(matches!(r1, Resposta::Autorizado(a) if a.hash_do_conteudo == H));
        assert!(matches!(r2, Resposta::Negado), "negar um item não aprova o outro por engano");
    }

    // ---- aba trancada: pedido e aprovação (regra 34 revista, Rodada 20, 11/10/2026) ----

    #[test]
    fn pedido_de_aba_trancada_aprovado_devolve_o_prazo_escolhido_por_quem_aprova() {
        let quem_pede = assinatura::gerar();
        let pedido = solicitar(&quem_pede, H, 1_000);
        let prazo = crate::custodia_mutua::PrazoDeCustodiaMutua::de_dias(30);
        let resp = responder_pedido_de_aba_trancada(&quem_pede.verificacao(), &pedido, true, prazo).unwrap();
        assert!(matches!(resp, RespostaDeAbaTrancada::Aprovado(p) if p == prazo));
    }

    #[test]
    fn pedido_de_aba_trancada_negado_ignora_o_prazo_recebido_e_nunca_aprova() {
        let quem_pede = assinatura::gerar();
        let pedido = solicitar(&quem_pede, H, 1_000);
        // Mesmo passando um prazo "válido", `aprovar: false` nunca pode resultar em Aprovado — quem pede
        // nunca decide, só quem já tem o item decide (ver doc da função).
        let prazo = crate::custodia_mutua::PrazoDeCustodiaMutua::de_dias(182);
        let resp = responder_pedido_de_aba_trancada(&quem_pede.verificacao(), &pedido, false, prazo).unwrap();
        assert!(matches!(resp, RespostaDeAbaTrancada::Negado));
    }

    #[test]
    fn pedido_de_aba_trancada_assinado_por_outra_pessoa_e_recusado() {
        let quem_pede = assinatura::gerar();
        let outra_pessoa = assinatura::gerar();
        let pedido = solicitar(&quem_pede, H, 1_000);
        let prazo = crate::custodia_mutua::PrazoDeCustodiaMutua::de_dias(10);
        assert_eq!(
            responder_pedido_de_aba_trancada(&outra_pessoa.verificacao(), &pedido, true, prazo).unwrap_err(),
            ErroDePedido::AssinaturaInvalida
        );
    }

    #[test]
    fn pedido_de_aba_trancada_adulterado_depois_de_assinado_e_recusado() {
        let quem_pede = assinatura::gerar();
        let mut pedido = solicitar(&quem_pede, H, 1_000);
        pedido.hash_do_conteudo = [9; 32]; // tenta pedir outro item com a assinatura antiga
        let prazo = crate::custodia_mutua::PrazoDeCustodiaMutua::de_dias(10);
        assert_eq!(
            responder_pedido_de_aba_trancada(&quem_pede.verificacao(), &pedido, true, prazo).unwrap_err(),
            ErroDePedido::AssinaturaInvalida
        );
    }

    #[test]
    fn aprovar_aba_trancada_nunca_produz_uma_autorizacao_generica() {
        // Achado de desenho (não um bug): diferente de `responder_para_destino`, esta função não importa
        // `Autorizacao` em nenhum resultado possível — `RespostaDeAbaTrancada` não tem variante que a
        // carregue. Este teste documenta a garantia de tipo: se algum dia alguém "simplificar" os dois
        // caminhos num só, este teste para de compilar (não falha em tempo de execução) — o jeito certo
        // de perceber a regressão antes de qualquer teste rodar.
        let quem_pede = assinatura::gerar();
        let pedido = solicitar(&quem_pede, H, 1_000);
        let prazo = crate::custodia_mutua::PrazoDeCustodiaMutua::de_dias(5);
        let resp = responder_pedido_de_aba_trancada(&quem_pede.verificacao(), &pedido, true, prazo).unwrap();
        match resp {
            RespostaDeAbaTrancada::Aprovado(p) => assert_eq!(p, prazo),
            RespostaDeAbaTrancada::Negado => panic!("deveria ter aprovado"),
        }
    }
}
