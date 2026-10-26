//! # Módulo `custodia_mutua` (CTM1–CTM4, novo — rodada atual) — item sob guarda dos dois lados, qualquer um apaga para os dois
//!
//! A "aba trancada" de uma conversa 1:1: quando alguém encaminha, para essa aba, algo que já estava
//! autorizado a ser encaminhado (ver `liberacao` — nunca algo com `liberacao::Nivel::LevarParaFora`,
//! que já autoriza sair do Nimbus e tornaria este mecanismo inútil), o item deixa de ser "coisa que
//! eu compartilhei" e vira "coisa que nós dois guardamos": as duas pessoas enxergam o mesmo item ali,
//! e CADA UMA, sozinha, a qualquer momento antes do prazo-teto, pode apagá-lo **para as duas de uma
//! vez** — como o "apagar para todos" do WhatsApp, só que com uma garantia mais forte: apagar aqui
//! destrói a única chave que algum dia poderia abrir aquele conteúdo, não apenas tira da tela.
//!
//! Nome parecido com o módulo `custodia` (guardiões da chave mestra, k de n) é coincidência de
//! vocabulário, não parentesco: não há chave mestra nem partilha Shamir aqui.
//!
//! ## O desenho, em um parágrafo
//!
//! Cada item ganha um par de chaves **novo e descartável**, usado só para ele (nunca reaproveitado
//! entre itens, nunca derivado da identidade permanente de ninguém). O conteúdo é selado
//! (`cofre::selar` — mesmo esconde-tipo-e-tamanho de sempre) sob a chave pública desse par. O par
//! inteiro — pública e secreta — viaja, uma única vez, para os dois aparelhos pelo canal 1:1 já
//! cifrado que eles já têm (fora do escopo deste núcleo, que nunca trata transporte; `Referencia::
//! para_bytes`/`de_bytes`, abaixo, só simulam essa viagem). O servidor só guarda o bloco cifrado, sob
//! um ID aleatório, sem nada que diga de quem é, de que conversa é, ou que mais alguém guarda uma
//! cópia da mesma chave. **Apagar = destruir esse par de chaves nos dois aparelhos e remover o bloco
//! do servidor.** Sem a chave, o bloco — mesmo que alguém tenha guardado cópia antes de apagar — é
//! ruído para sempre.
//!
//! ## Por que não é "selar para a chave permanente de cada lado" (o desenho que eu descartei)
//!
//! Uma primeira versão deste módulo selava o conteúdo diretamente para a chave pública PERMANENTE de
//! cada lado (a mesma identidade que já usam no resto da conversa) — dois blocos, um por lado. Mais
//! simples, mas com um defeito real: a chave permanente de cada pessoa **nunca é destruída** (é a
//! identidade dela, usada para tudo mais na conversa) — então, se alguém tivesse guardado uma cópia
//! de um dos blocos *antes* do "apagar" (um vazamento do servidor, um backup, uma ordem judicial
//! executada num instante anterior), essa cópia continuaria abrível para sempre com a chave
//! permanente de qualquer um dos dois lados, mesmo anos depois do "apagar". Isso não é "destruir a
//! chave" — é só "parar de apontar para o dado", a mesma garantia mais fraca que `direto.rs` já
//! assume honestamente (entrega única, chave de identidade do destinatário, sem destruição de chave)
//! e que esta conversa já identificou como **diferente e mais fraca** do que a garantia de
//! `historico`/`cofre`. Como o pedido foi explicitamente "que sumam as chaves... como no WhatsApp,
//! mas com a garantia mais forte", a primeira versão não bastava: troquei para o par de chaves
//! descartável por item, acima, antes de escrever qualquer teste — para não entregar, em código, uma
//! garantia mais fraca do que a descrita em palavras.
//!
//! ## Por que não reaproveitar `historico` nem `grupo`
//!
//! * `historico::ChavesDoDono` guarda **uma chave por janela de tempo**: todo item cujo prazo vence
//!   na mesma janela compartilha a mesma chave — ótimo quando há um único dono, ruim aqui, porque
//!   apagar UM item sob custódia mútua antes do prazo natural não pode arriscar apagar, de brinde,
//!   outro item (de outra conversa) que por acaso vence na mesma janela. Por isso: par de chaves
//!   próprio, por item, nunca compartilhado — nem entre itens daqui, nem com `historico`.
//! * `grupo::membros: HashMap<IdParticipante, Preferencias>` é exatamente o tipo de metadado que o
//!   pedido quer evitar: uma lista, em algum lugar, dizendo "estas contas guardam isto juntas".
//!   Reaproveitar `grupo` aqui criaria, sem necessidade, o próprio rastro que se quer não deixar
//!   existir. Este módulo não usa `grupo` — o servidor nunca vê duas contas, só um bloco opaco.
//!
//! ## O que isto garante, e o que isto não garante (para não prometer além da conta)
//!
//! * **Garante:** uma cópia do bloco capturada de qualquer lugar que não seja um dos dois aparelhos
//!   legítimos (o servidor, um backup do servidor, uma interceptação de rede, uma ordem executada
//!   contra o operador) fica permanentemente ilegível assim que QUALQUER um dos dois lados chama
//!   `revogar` — porque a única chave que abriria aquele bloco específico deixa de ter uso em
//!   qualquer lugar que essa cópia alcance. Isso vale mesmo que a cópia tenha sido feita muito antes
//!   do `revogar`, e mesmo que, depois, alguém extraia a chave de identidade PERMANENTE de um dos
//!   dois lados — essa chave permanente nunca teve nada a ver com este item.
//! * **Não garante** (e nenhum mecanismo poderia): se um dos dois aparelhos, estando legitimamente
//!   com acesso, for examinado (Cellebrite/GrayKey) ENQUANTO ainda guarda, localmente, a chave deste
//!   item ou o conteúdo já aberto em cache — ou seja, antes de processar um comando de revogação
//!   recebido pelo canal 1:1 — o exame encontra o que aquele aparelho, legitimamente, podia ver. É o
//!   mesmo limite universal já identificado nesta conversa para mensagens comuns (`expiracao`) e para
//!   o cofre (`historico`): nenhuma proteção sobrevive a um exame do próprio aparelho autorizado,
//!   feito antes de a limpeza chegar nele. O que reduz essa janela é a trava do cofre com retrava por
//!   inatividade e por `integridade::Nivel::Vermelho` (ver módulos `sessoes`/`integridade` e a
//!   política de trava desta mesma rodada) — nunca este módulo sozinho.
//!
//! ## Pré-condições (responsabilidade de quem chama — não deste módulo)
//!
//! 1. Só item com autorização de encaminhamento vigente entra aqui (ver `liberacao::nivel_autorizado`
//!    e `liberacao::Nivel`); nunca algo com `Nivel::LevarParaFora`.
//! 2. Nenhum dos dois lados guardou cópia permanente do conteúdo por fora deste mecanismo (ex.: salvo
//!    na galeria do aparelho) — o mesmo limite de qualquer apagamento criptográfico, não uma fraqueza
//!    nova daqui (ver a nota equivalente em `cofre` e `historico`).
//! 3. A mensagem que carrega a `Referencia` inteira (`para_bytes`, com a chave secreta efêmera dentro)
//!    pelo canal 1:1 usa o prazo de retenção mais curto que a conversa oferecer (idealmente
//!    visualização única) — revogar apaga o bloco do servidor, mas não apaga uma cópia dessa
//!    mensagem-transporte que porventura sobreviva no histórico/backup do PRÓPRIO canal 1:1 (ele é
//!    anterior e externo a este módulo). Responsabilidade da ponte, não deste núcleo.
//!
//! ## Limite reconhecido, não resolvido aqui: consistência num servidor real distribuído
//!
//! `ServidorCego`, acima, é um `HashMap` de um processo só — útil para provar a lógica, mas não modela
//! um servidor real (réplicas, banco de dados, fila). Num servidor real, existe uma janela, em tese,
//! entre um `revogar` já aplicado num nó e um `abrir` ainda sendo servido por outro nó com uma cópia
//! desatualizada (consistência eventual) — isso quebraria "apagar já é apagado para os dois, na hora".
//! Este núcleo não pode fechar essa janela (é uma propriedade de infraestrutura, não de protocolo);
//! quem implementa o servidor real precisa de uma operação atômica de "ler-e-conferir-prazo" e
//! "remover" sobre o MESMO dado consistente (ex.: transação/leitura-com-trava no banco), não duas
//! chamadas independentes. Ver `CONTRATO_DA_PONTE.md` e `PENDENCIAS_E_DECISOES.md` para o registro
//! deste limite como item a verificar antes de produção.
//!
//! ## Metadado de horário: por que `valida_ate` nunca é gravado cru no servidor
//!
//! Achado em auditoria adversarial desta mesma rodada: como todo prazo (`PrazoDeCustodiaMutua`) é um
//! número inteiro de DIAS, `valida_ate = agora + dias*86_400` preserva `agora mod 86_400` — ou seja,
//! gravar `valida_ate` cru no servidor, sem decifrar nada, revelaria o SEGUNDO EXATO DO DIA em que
//! cada item foi encaminhado (e permitiria ligar blocos "sem relação" que por coincidência
//! compartilhem esse resto). Isso era inconsistente com o cuidado que `historico::ChavesDoDono::
//! janela_de` já tinha com granularidade de tempo. Corrigido: o servidor só recebe `valida_ate`
//! arredondado para CIMA até o próximo limite de dia UTC (`arredondar_para_o_servidor`, abaixo) —
//! nunca para baixo, para nunca negar acesso antes do prazo real. A `Referencia` de cada aparelho
//! continua com o `valida_ate` exato (não é o que vaza; só o que vai para o servidor é arredondado), e
//! a checagem local (sempre exata) continua sendo quem garante o prazo verdadeiro, no segundo certo —
//! a checagem do servidor vira uma rede de segurança um pouco mais larga (até ~1 dia), nunca mais
//! apertada; é a mesma troca, de propósito, que já existia entre as duas checagens (ver `abrir`).
//!
//! ## Padrão de tráfego: por que `encaminhar`/`revogar` têm que passar pela fila de `trafego::Batida`
//! (achado desta sessão, 10/10/2026)
//!
//! Este núcleo nunca trata transporte (ver "O desenho, em um parágrafo", acima) — `encaminhar`,
//! `abrir` e `revogar` recebem o `ServidorCego` já pronto, de quem chama; a chamada de rede real que
//! grava ou apaga um bloco no servidor de verdade é inteiramente responsabilidade da ponte, igual a
//! qualquer outro envio (ver `CONTRATO_DA_PONTE.md`, nota "núcleo nunca trata transporte" da regra 35).
//! Isso não dispensa uma exigência clara sobre COMO a ponte faz essa chamada — e, até esta correção,
//! ela não existia por escrito para este módulo. A regra 27 do contrato (localização ao vivo) já exige
//! que cada atualização de posição entre na mesma fila de cadência constante de `trafego::Batida`
//! (pacote a cada `INTERVALO_MS`, de verdade ou de enchimento, indistinguível de fora) — nunca um
//! canal de rede próprio — exatamente para que o INSTANTE de cada envio não seja, sozinho, um sinal
//! observável. O mesmo raciocínio vale aqui, ponto a ponto: sem essa exigência, um adversário com
//! acesso à rede (nunca ao conteúdo — o bloco já é cego e de tamanho padronizado, ver `cofre::selar`)
//! poderia ver o INSTANTE em que a ponte grava um bloco novo (via `encaminhar`) ou o apaga (via
//! `revogar`) como um evento isolado, de tamanho/cadência diferente de uma mensagem comum — vazando
//! QUANDO a aba trancada é usada, mesmo que o QUÊ nunca vaze. Não é uma categoria de risco nova: é a
//! mesma lacuna que a regra 27 já fecha para localização ao vivo, só que, por descuido, nunca
//! replicada aqui quando este módulo foi escrito. Corrigido nesta sessão: `CONTRATO_DA_PONTE.md`
//! passa a exigir, como quinta condição para a exceção de `historico` continuar valendo (§1) e na
//! própria linha da regra 34 (§2), que toda chamada de rede real de `encaminhar`/`revogar` entre na
//! fila de `trafego::Batida` — nunca uma chamada avulsa e imediata. Este módulo não muda de código por
//! causa disto (a fila já existe, já é genérica, e enfileirar é decisão de transporte, não de
//! protocolo) — só passa a ser, a partir de agora, uma obrigação contratual explícita da ponte também
//! para este módulo, e não apenas para a regra 27. Verificação (medição, não decisão) ainda pendente:
//! ver `CONTRATO_DA_PONTE.md`, "Verificar antes de usar", item 7.
//!
//! ## Conferência contra a clarificação de Gabriel sobre a aba trancada (achado desta sessão, 11/10/2026)
//!
//! Gabriel pediu, em detalhe, para confirmar que a aba trancada entrega: (a) UM chat persistente por
//! contato (não um item isolado por vez); (b) tudo que for liberado para reencaminhar aparece ali para
//! os dois lados; (c) apagar funciona item a item OU o chat inteiro de uma vez, sem rastro em ambos os
//! casos. Conferido linha a linha contra o que este módulo já implementa e testa:
//!
//! * **(b) e o apagar item a item já são exatamente isto, sem mudança de código.** `encaminhar` é
//!   quem cria o item compartilhado (só aceita o que `liberacao` já autorizou — ver "Pré-condições",
//!   acima); os dois lados o veem via `abrir`; `revogar`, chamado por QUALQUER um dos dois, destrói a
//!   única chave e remove o bloco do servidor — já testado nos dois sentidos
//!   (`revogar_fecha_para_os_dois_mesmo_so_um_lado_chamando`).
//! * **(a) e o "chat todo" de (c) NÃO existem, e não podem existir, como conceito deste módulo** — e
//!   isso não é uma lacuna, é a mesma decisão de design já registrada acima em "Por que não
//!   reaproveitar `historico` nem `grupo`": o servidor nunca vê duas contas nem uma lista do que elas
//!   guardam juntas, só blocos opacos sem relação declarada entre si. "Um chat por contato" só existe
//!   no aparelho de cada lado, que sabe localmente — porque foi ELE que encaminhou ou recebeu cada
//!   `Referencia` dentro daquela conversa 1:1 — quais itens pertencem a qual contato. "Apagar o chat
//!   todo" é, portanto, necessariamente a ponte chamando `revogar` em laço sobre essa lista
//!   LOCAL (nunca enviada ao servidor como lista), um item por vez; o resultado observável para o
//!   usuário (tudo desaparece de uma vez, para os dois lados, sem rastro) é idêntico a um apagamento
//!   atômico — a diferença é invisível na tela e existe só para não recriar, no servidor, o metadado
//!   que todo o resto deste módulo existe para evitar. Resposta direta às duas perguntas "ficou claro?"
//!   de Gabriel: sim, ficou claro, e o comportamento descrito é 100% entregável — como composição na
//!   ponte de uma primitiva de núcleo já implementada e testada, não como primitiva nova.
//! * **O mesmo vale para "pasta"/álbum.** Conferido em `cofre::Tipo` (`Foto, Video, Audio, Arquivo,
//!   Album, Nota, Compromisso, Senha, Ajuste`) e em `cofre::Item { tipo, campos, conteudo }`: `Album`
//!   não tem struct própria nem lista de filhos — é um `Item` comum cujos `campos` guardam `nome` e
//!   `capa` (referência à foto de capa), selado como qualquer outro. Não há, em nenhum módulo deste
//!   núcleo, um tipo "contém outros itens". Qualquer agrupamento por pasta, como por contato, é
//!   reconstrução local da ponte a partir do que o próprio aparelho já sabe — nunca um vínculo visível
//!   ao servidor.
//! * **Horário de mensagem/ligação exibido na aba trancada, e seu desaparecimento com a expiração**
//!   (o outro ponto que Gabriel pediu para confirmar, ligado à pergunta 8 da Rodada 20): o carimbo de
//!   horário não é — e não deve ser — um campo independente, gravado ou exibido separadamente da
//!   mensagem. Em `expiracao::Bilhete`, `aberta_em`/`fechada_em` só existem amarrados ao MESMO bilhete
//!   que `pode_exibir`/`expirou` consultam para decidir se a mensagem ainda pode aparecer; não há
//!   caminho, neste núcleo, para um horário sobreviver, sozinho, à mensagem que ele descreve. Na
//!   prática isso significa: a ponte exibe o horário como parte do MESMO elemento de UI da mensagem
//!   (ou da entrada de ligação), nunca como um registro à parte — quando `pode_exibir` vira falso, o
//!   horário desaparece junto, pela mesma razão que o conteúdo desaparece, sem exigir nenhuma lógica
//!   nova aqui. Mesma composição-na-ponte-sem-primitiva-nova do ponto anterior.
//!
//! Conclusão da conferência: nenhuma divergência encontrada entre o que este módulo entrega e o que
//! Gabriel descreveu — nenhuma correção de código foi necessária nesta rodada. O único risco real é
//! de EXPECTATIVA, não de engenharia: se "um chat por contato" ou "pasta" for lido como algo que o
//! SERVIDOR sabe organizar, a resposta é não, por design, e essa organização é sempre tão confiável
//! quanto o armazenamento local de CADA aparelho (ver `PENDENCIAS_E_DECISOES.md`, Rodada 21, para o
//! registro formal desta conferência e da consequência prática de um aparelho restaurado do zero: o
//! conteúdo cifrado sobrevive no servidor até o prazo-teto, mas o AGRUPAMENTO por contato/pasta, que
//! depende só do aparelho, precisa estar coberto pelo backup/sincronização local da ponte, não deste
//! núcleo).
use crate::cofre::{self, ErroDoCofre, Item};
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use rand_core::{OsRng, RngCore};
use std::collections::HashMap;
use zeroize::Zeroizing;

/// Dias mínimos — mesmo piso de `cofre::PRAZO_MINIMO_DIAS`.
pub const PRAZO_MINIMO_DIAS: u64 = 1;
/// Teto desta rodada: 6 meses (182 dias) — o mesmo teto do plano Max (ver tabela de planos do
/// produto). Foi cogitado subir para 12 meses, já que qualquer um dos dois lados pode apagar a
/// qualquer momento (o risco de "ficar para sempre" é bem menor que numa guarda unilateral) — por
/// ora o teto segue 182: decisão explícita, fácil de revisar mudando só esta constante.
pub const PRAZO_MAXIMO_DIAS: u64 = 182;

/// Prazo-teto de um item em custódia mútua: depois dele, o item fecha sozinho mesmo sem ninguém
/// revogar (ver `remover_vencidos`). Antes dele, qualquer um dos dois lados revoga quando quiser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrazoDeCustodiaMutua(u64);

impl PrazoDeCustodiaMutua {
    pub fn de_dias(dias: u64) -> Self { Self(dias.clamp(PRAZO_MINIMO_DIAS, PRAZO_MAXIMO_DIAS)) }
    pub fn em_dias(self) -> u64 { self.0 }
    pub fn em_segundos(self) -> u64 { self.0 * 86_400 }
}

/// Identificador aleatório do bloco no servidor cego — não liga a conta, nem à conversa.
pub type Id = [u8; 32];

fn novo_id() -> Id {
    let mut id = [0u8; 32];
    OsRng.fill_bytes(&mut id);
    id
}

#[derive(Debug, PartialEq, Eq)]
pub enum Erro {
    AoSelar(ErroDoCofre),
    NaoEncontrado,
    Expirado,
    Ilegivel(ErroDoCofre),
    Malformado,
    Chave(envelope::Erro),
}

/// Simula o servidor: só vê um bloco cifrado de tamanho padronizado e um prazo de validade por
/// bloco — nada que ligue um bloco a uma conta, a uma conversa, ou a outro bloco.
#[derive(Default)]
pub struct ServidorCego {
    pub blocos: HashMap<Id, (Vec<u8>, u64)>,
}

impl core::fmt::Debug for ServidorCego {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ServidorCego({} blocos opacos)", self.blocos.len())
    }
}

/// O que cada um dos dois aparelhos guarda, localmente, para o item desta aba trancada — nunca fica
/// no servidor. Em produção, viaja do aparelho que encaminhou para o outro lado pelo canal 1:1 já
/// cifrado da conversa (fora do escopo deste núcleo); `para_bytes`/`de_bytes`, abaixo, simulam essa
/// viagem — serializar de um lado, reconstruir do outro. `chave_publica`/`chave_secreta` ficam
/// privados (só usados por `abrir`, aqui dentro); `id` e `valida_ate` são públicos porque são
/// inofensivos e úteis para a interface mostrar "expira em N dias" sem precisar abrir o item.
#[derive(Debug)]
pub struct Referencia {
    pub id: Id,
    pub valida_ate: u64,
    chave_publica: ChavePublica,
    chave_secreta: ChaveSecreta,
}

impl Referencia {
    /// Serializa para viajar pelo canal 1:1 já cifrado da conversa (fora deste núcleo). O resultado
    /// se zera sozinho ao sair de escopo, porque carrega a chave secreta efêmera do item.
    pub fn para_bytes(&self) -> Zeroizing<Vec<u8>> {
        let mut v = Zeroizing::new(Vec::new());
        v.extend_from_slice(&self.id);
        v.extend_from_slice(&self.valida_ate.to_be_bytes());
        v.extend_from_slice(&self.chave_secreta.para_bytes());
        v.extend_from_slice(&self.chave_publica.para_bytes());
        v
    }

    /// Reconstrói do outro lado. A chave pública fica por último no formato (ver `para_bytes`)
    /// porque seu tamanho não precisa ser conhecido de antemão: `ChavePublica::de_bytes` confere o
    /// próprio tamanho.
    pub fn de_bytes(b: &[u8]) -> Result<Self, Erro> {
        const TAM_FIXO: usize = 32 + 8;
        if b.len() < TAM_FIXO + envelope::TAM_CHAVE_SECRETA {
            return Err(Erro::Malformado);
        }
        let mut id = [0u8; 32];
        id.copy_from_slice(&b[..32]);
        let mut t = [0u8; 8];
        t.copy_from_slice(&b[32..TAM_FIXO]);
        let valida_ate = u64::from_be_bytes(t);
        let fim_secreta = TAM_FIXO + envelope::TAM_CHAVE_SECRETA;
        let chave_secreta = ChaveSecreta::de_bytes(&b[TAM_FIXO..fim_secreta]).map_err(Erro::Chave)?;
        let chave_publica = ChavePublica::de_bytes(&b[fim_secreta..]).map_err(Erro::Chave)?;
        Ok(Referencia { id, valida_ate, chave_publica, chave_secreta })
    }
}

/// Arredonda `valida_ate` para CIMA até o próximo limite de dia UTC (nunca para baixo) — só o valor
/// que vai para o servidor passa por aqui; a `Referencia` de cada aparelho guarda o valor exato, sem
/// arredondar. Ver a nota "Metadado de horário", na doc do módulo, acima, para o motivo.
fn arredondar_para_o_servidor(valida_ate: u64) -> u64 {
    valida_ate.div_ceil(86_400).saturating_mul(86_400)
}

/// Encaminha `item` para a aba trancada: gera um par de chaves novo e descartável (só para este
/// item), sela o conteúdo sob a chave pública desse par e guarda o bloco no servidor cego. Quem
/// chama já deve ter checado `liberacao` (ver doc do módulo, acima) — este módulo não verifica.
pub fn encaminhar(
    servidor: &mut ServidorCego,
    item: &Item,
    agora: u64,
    prazo: PrazoDeCustodiaMutua,
) -> Result<Referencia, Erro> {
    let (chave_publica, chave_secreta) = envelope::gerar_par_de_chaves();
    let bloco = cofre::selar(&chave_publica, item).map_err(Erro::AoSelar)?;
    let valida_ate = agora.saturating_add(prazo.em_segundos());
    let id = novo_id();
    servidor.blocos.insert(id, (bloco, arredondar_para_o_servidor(valida_ate)));
    Ok(Referencia { id, valida_ate, chave_publica, chave_secreta })
}

/// Abre com a própria referência (recebida uma vez, pelo canal 1:1, no momento do encaminhamento).
/// Os dois lados chamam exatamente a mesma função, cada um com a própria cópia da `Referencia` —
/// não há operação "de remetente" nem "de destinatário": os dois têm o mesmo acesso.
///
/// Duas checagens de prazo, de propósito, não uma só: a local (`referencia.valida_ate`, o que ESTE
/// aparelho acredita) e a do servidor (o que está realmente gravado). Em uso honesto as duas sempre
/// batem — mas se alguma vez divergirem (cache local velho/corrompido, ou uma cópia local alterada
/// tentando "esticar" o próprio prazo), vale a mais restritiva: qualquer uma das duas que disser
/// "venceu" já é suficiente para recusar. Provado por teste (`checagem_local_recusa_sozinha_mesmo_
/// se_o_servidor_ainda_aceitaria`, `checagem_do_servidor_recusa_mesmo_se_o_campo_local_disser_prazo_
/// maior`) depois que a mutação [309]/[310] mostrou que, sem essas divergências testadas, as duas
/// checagens eram redundantes na prática e uma das duas podia quebrar em silêncio.
pub fn abrir(servidor: &ServidorCego, referencia: &Referencia, agora: u64) -> Result<Item, Erro> {
    if agora >= referencia.valida_ate {
        return Err(Erro::Expirado);
    }
    let (bloco, valida_ate_no_servidor) = servidor.blocos.get(&referencia.id).ok_or(Erro::NaoEncontrado)?;
    if agora >= *valida_ate_no_servidor {
        return Err(Erro::Expirado);
    }
    cofre::abrir(&referencia.chave_secreta, &referencia.chave_publica, bloco).map_err(Erro::Ilegivel)
}

/// Apagar para ambos: quem chama (remetente OU destinatário, tanto faz) só precisa da própria
/// `Referencia` para derrubar o bloco do servidor. A partir daqui, nenhum dos dois lados consegue
/// mais abrir o item a partir do servidor, mesmo que alguém tenha guardado cópia do bloco à parte:
/// a chave que o abriria era só esta, e nunca existiu em nenhum outro lugar. Devolve `true` se havia
/// mesmo algo para apagar (uma segunda chamada, depois que já foi revogado, devolve `false`, sem
/// pane).
pub fn revogar(servidor: &mut ServidorCego, referencia: &Referencia) -> bool {
    servidor.blocos.remove(&referencia.id).is_some()
}

/// Faxina de rotina: remove do servidor todo bloco cujo prazo-teto já passou e que ninguém revogou
/// antes. Não é preciso chamar isto para a garantia de "apagar para ambos" valer — isso já vale a
/// qualquer momento via `revogar`, e `abrir` já recusa um item vencido mesmo que o bloco ainda esteja
/// fisicamente ali — só evita acumular blocos mortos no servidor.
pub fn remover_vencidos(servidor: &mut ServidorCego, agora: u64) -> usize {
    let antes = servidor.blocos.len();
    servidor.blocos.retain(|_, (_, valida_ate)| agora < *valida_ate);
    antes - servidor.blocos.len()
}

// ---------------------------------------------------------------------------------------------------------
// Transferência na troca de aparelho (regra 35 da ponte, rodada 09/10/2026). Achado da rodada anterior: a
// regra já existia como decisão de arquitetura, mas a função que ela descrevia ainda não tinha sido escrita
// — as peças (`sessoes::pode_vincular`, `Referencia::para_bytes`/`de_bytes`, `envelope::selar`/`abrir`) já
// existiam isoladas; nada as ligava. Esta seção fecha essa lacuna.
// ---------------------------------------------------------------------------------------------------------

const CONTEXTO_TRANSFERENCIA: &[u8] = b"NimbusOne/v1/custodia_mutua/transferencia-de-aparelho";

fn ler_u32(d: &[u8], i: &mut usize) -> Result<u32, Erro> {
    if *i + 4 > d.len() { return Err(Erro::Malformado); }
    let mut b = [0u8; 4];
    b.copy_from_slice(&d[*i..*i + 4]);
    *i += 4;
    Ok(u32::from_be_bytes(b))
}

/// Prepara, para a cerimônia `sessoes::pode_vincular`, o pacote selado que leva do aparelho antigo
/// para o novo: cada `Referencia` AINDA VÁLIDA (uma já vencida é descartada aqui — não há razão para
/// entregar ao aparelho novo algo que já não abre em lugar nenhum) e a chave local do cofre pessoal,
/// os dois selados para a chave pública do aparelho novo — a MESMA que a cerimônia já confirmou pelas
/// palavras conferidas nos dois aparelhos (`sessoes::palavras_do_vinculo`). Só chamar depois de
/// `sessoes::pode_vincular` devolver `Ok`, nunca antes, nunca como caminho alternativo a ela — a
/// catraca do kit reprova qualquer tela ou função que ofereça um caminho que não passe por ali
/// primeiro (ver `CONTRATO_DA_PONTE.md`, regra 35).
///
/// Como o resto deste núcleo (ver a nota do módulo, no alto deste arquivo: "núcleo... nunca trata
/// transporte"), esta função NÃO abre Bluetooth nem envia nada — devolve só os bytes selados; a ponte
/// entrega esses bytes pelo canal que a própria cerimônia já confirmou (Bluetooth/Wi-Fi Direct, nunca
/// pelo servidor em claro, nunca por um canal novo). É o mesmo limite que `Referencia::para_bytes` já
/// tinha ("só simula a viagem"); isto aqui acrescenta a camada de sigilo que faltava para a viagem de
/// verdade ser segura mesmo que o sinal seja capturado no ar — sem essa camada, qualquer um perto o
/// bastante para captar o Bluetooth leria as chaves efêmeras de cada item em texto claro.
pub fn preparar_pacote_de_transferencia(
    referencias: &[Referencia],
    agora: u64,
    chave_local_do_cofre: &[u8],
    chave_publica_do_aparelho_novo: &ChavePublica,
) -> Result<Vec<u8>, Erro> {
    let validas: Vec<&Referencia> = referencias.iter().filter(|r| agora < r.valida_ate).collect();
    if validas.len() > u32::MAX as usize || chave_local_do_cofre.len() > u32::MAX as usize {
        return Err(Erro::Malformado);
    }
    let mut claro = Zeroizing::new(Vec::new());
    claro.extend_from_slice(&(validas.len() as u32).to_be_bytes());
    for r in &validas {
        let bytes = r.para_bytes();
        claro.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        claro.extend_from_slice(&bytes);
    }
    claro.extend_from_slice(&(chave_local_do_cofre.len() as u32).to_be_bytes());
    claro.extend_from_slice(chave_local_do_cofre);
    envelope::selar(chave_publica_do_aparelho_novo, &claro, CONTEXTO_TRANSFERENCIA).map_err(Erro::Chave)
}

/// No aparelho novo: abre o pacote que `preparar_pacote_de_transferencia` produziu e devolve as
/// `Referencia`s que ele trazia e a chave local do cofre. Esta função só desembrulha — não julga
/// prazo de novo (cada `Referencia` já carrega o próprio `valida_ate`, e `abrir` já checa isso
/// quando a referência for de fato usada); como o preparo já descarta vencidas no momento em que é
/// chamado, só chegam aqui referências que eram válidas naquele instante.
pub fn abrir_pacote_de_transferencia(
    secreta_do_aparelho_novo: &ChaveSecreta,
    publica_do_aparelho_novo: &ChavePublica,
    pacote: &[u8],
) -> Result<(Vec<Referencia>, Zeroizing<Vec<u8>>), Erro> {
    let claro =
        envelope::abrir(secreta_do_aparelho_novo, publica_do_aparelho_novo, pacote, CONTEXTO_TRANSFERENCIA)
            .map_err(Erro::Chave)?;
    let mut i = 0usize;
    let n = ler_u32(&claro, &mut i)? as usize;
    let mut referencias = Vec::with_capacity(n.min(1024));
    for _ in 0..n {
        let tam = ler_u32(&claro, &mut i)? as usize;
        if i + tam > claro.len() { return Err(Erro::Malformado); }
        referencias.push(Referencia::de_bytes(&claro[i..i + tam])?);
        i += tam;
    }
    let tam_chave = ler_u32(&claro, &mut i)? as usize;
    if i + tam_chave > claro.len() { return Err(Erro::Malformado); }
    let chave_local_do_cofre = Zeroizing::new(claro[i..i + tam_chave].to_vec());
    Ok((referencias, chave_local_do_cofre))
}

#[cfg(test)]
mod testes {
    use super::*;

    fn contem(h: &[u8], a: &[u8]) -> bool { h.windows(a.len()).any(|w| w == a) }
    fn item(tipo: cofre::Tipo, campos: &[(&str, &str)], conteudo: &[u8]) -> Item {
        Item { tipo, campos: campos.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), conteudo: conteudo.to_vec() }
    }

    #[test]
    fn encaminhar_e_abrir_fazem_ida_e_volta() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[("nome", "contrato.pdf")], b"bytes-do-contrato");
        let referencia = encaminhar(&mut servidor, &it, 1_000, PrazoDeCustodiaMutua::de_dias(30)).unwrap();
        assert_eq!(abrir(&servidor, &referencia, 1_000).unwrap(), it);
    }

    #[test]
    fn qualquer_um_dos_dois_lados_abre_com_a_propria_copia_da_referencia() {
        // Simula a viagem pelo canal 1:1: serializa de um lado, reconstrói do outro.
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Foto, &[("legenda", "praia")], b"bytes-da-foto");
        let referencia_remetente = encaminhar(&mut servidor, &it, 1_000, PrazoDeCustodiaMutua::de_dias(30)).unwrap();
        let referencia_destinatario = Referencia::de_bytes(&referencia_remetente.para_bytes()).unwrap();
        assert_eq!(abrir(&servidor, &referencia_remetente, 1_000).unwrap(), it);
        assert_eq!(abrir(&servidor, &referencia_destinatario, 1_000).unwrap(), it);
    }

    #[test]
    fn revogar_fecha_para_os_dois_mesmo_so_um_lado_chamando() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Nota, &[], b"segredo-dos-dois");
        let referencia_remetente = encaminhar(&mut servidor, &it, 1_000, PrazoDeCustodiaMutua::de_dias(30)).unwrap();
        let referencia_destinatario = Referencia::de_bytes(&referencia_remetente.para_bytes()).unwrap();
        assert!(revogar(&mut servidor, &referencia_destinatario), "o destinatário revoga sozinho");
        assert_eq!(abrir(&servidor, &referencia_remetente, 1_000), Err(Erro::NaoEncontrado), "fecha para o remetente também");
        assert_eq!(abrir(&servidor, &referencia_destinatario, 1_000), Err(Erro::NaoEncontrado));
    }

    #[test]
    fn revogar_antes_do_prazo_natural_funciona_de_imediato() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let referencia = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(182)).unwrap();
        assert!(abrir(&servidor, &referencia, 100).is_ok(), "bem dentro do prazo de 182 dias");
        assert!(revogar(&mut servidor, &referencia));
        assert_eq!(abrir(&servidor, &referencia, 100), Err(Erro::NaoEncontrado));
    }

    #[test]
    fn prazo_vencido_fecha_sozinho_sem_revogar() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let referencia = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(1)).unwrap();
        let depois_do_prazo = 2 * 86_400;
        assert_eq!(abrir(&servidor, &referencia, depois_do_prazo), Err(Erro::Expirado));
        // Ninguém chamou revogar: o bloco ainda está fisicamente no servidor, só não é mais servido.
        assert!(servidor.blocos.contains_key(&referencia.id));
    }

    #[test]
    fn prazo_e_grampeado_entre_1_e_182_dias() {
        assert_eq!(PrazoDeCustodiaMutua::de_dias(0).em_dias(), PRAZO_MINIMO_DIAS);
        assert_eq!(PrazoDeCustodiaMutua::de_dias(90).em_dias(), 90);
        assert_eq!(PrazoDeCustodiaMutua::de_dias(10_000).em_dias(), PRAZO_MAXIMO_DIAS);
        assert_eq!(PRAZO_MAXIMO_DIAS, 182, "teto desta rodada: 6 meses, igual ao plano Max");
    }

    #[test]
    fn servidor_nunca_ve_o_conteudo() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Nota, &[("titulo", "SEGREDO-VISIVEL")], b"conteudo-bem-secreto");
        let _ = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        for (bloco, _) in servidor.blocos.values() {
            assert!(!contem(bloco, b"SEGREDO-VISIVEL"));
            assert!(!contem(bloco, b"conteudo-bem-secreto"));
        }
    }

    #[test]
    fn id_certo_com_chave_errada_nao_abre() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let referencia_de_verdade = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        let (chave_publica_errada, chave_secreta_errada) = envelope::gerar_par_de_chaves();
        let referencia_forjada = Referencia {
            id: referencia_de_verdade.id,
            valida_ate: referencia_de_verdade.valida_ate,
            chave_publica: chave_publica_errada,
            chave_secreta: chave_secreta_errada,
        };
        assert!(matches!(abrir(&servidor, &referencia_forjada, 0), Err(Erro::Ilegivel(_))));
    }

    #[test]
    fn bloco_adulterado_nao_abre() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Nota, &[], b"x");
        let referencia = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        {
            let (bloco, _) = servidor.blocos.get_mut(&referencia.id).unwrap();
            let n = bloco.len() - 1;
            bloco[n] ^= 1;
        }
        assert!(matches!(abrir(&servidor, &referencia, 0), Err(Erro::Ilegivel(_))));
    }

    #[test]
    fn remover_vencidos_limpa_so_quem_venceu() {
        let mut servidor = ServidorCego::default();
        let vence_logo = item(cofre::Tipo::Arquivo, &[], b"a");
        let vence_depois = item(cofre::Tipo::Arquivo, &[], b"b");
        let ref_a = encaminhar(&mut servidor, &vence_logo, 0, PrazoDeCustodiaMutua::de_dias(1)).unwrap();
        let ref_b = encaminhar(&mut servidor, &vence_depois, 0, PrazoDeCustodiaMutua::de_dias(100)).unwrap();
        let agora = 2 * 86_400; // passou do prazo de 1 dia, não do de 100
        assert_eq!(remover_vencidos(&mut servidor, agora), 1);
        assert!(!servidor.blocos.contains_key(&ref_a.id));
        assert!(servidor.blocos.contains_key(&ref_b.id));
    }

    #[test]
    fn remover_vencidos_remove_no_instante_exato_do_prazo() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let referencia = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(1)).unwrap();
        let no_limite_exato = 86_400; // exatamente valida_ate, nem um segundo depois
        assert_eq!(remover_vencidos(&mut servidor, no_limite_exato), 1, "no instante exato já é hora de sumir, não um segundo depois");
        assert!(!servidor.blocos.contains_key(&referencia.id));
    }

    #[test]
    fn revogar_duas_vezes_nao_da_pane() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let referencia = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        assert!(revogar(&mut servidor, &referencia));
        assert!(!revogar(&mut servidor, &referencia), "a segunda vez não acha mais nada, mas não quebra");
    }

    #[test]
    fn esconder_tamanho_sobrevive_aqui_tambem() {
        let mut servidor = ServidorCego::default();
        let a = item(cofre::Tipo::Nota, &[], &[1u8; 300]);
        let b = item(cofre::Tipo::Senha, &[], &[1u8; 900]);
        let ref_a = encaminhar(&mut servidor, &a, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        let ref_b = encaminhar(&mut servidor, &b, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        assert_eq!(servidor.blocos[&ref_a.id].0.len(), servidor.blocos[&ref_b.id].0.len());
    }

    #[test]
    fn debug_da_referencia_nunca_mostra_a_chave_secreta() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Senha, &[], b"valor-sensivel");
        let referencia = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        assert!(format!("{referencia:?}").contains("ChaveSecreta(<oculta>)"));
    }

    #[test]
    fn referencia_serializada_e_reconstruida_continua_abrindo() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Compromisso, &[("titulo", "Reunião")], b"");
        let referencia = encaminhar(&mut servidor, &it, 500, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        let bytes = referencia.para_bytes();
        let reconstruida = Referencia::de_bytes(&bytes).unwrap();
        assert_eq!(reconstruida.id, referencia.id);
        assert_eq!(reconstruida.valida_ate, referencia.valida_ate);
        assert_eq!(abrir(&servidor, &reconstruida, 500).unwrap(), it);
    }

    #[test]
    fn referencia_de_bytes_recusa_tamanho_curto_demais() {
        // `Referencia` não deriva `PartialEq` (carrega uma `ChaveSecreta`, que não deriva de
        // propósito) — compara só o lado do erro, extraído com `unwrap_err`.
        assert_eq!(Referencia::de_bytes(&[0u8; 10]).unwrap_err(), Erro::Malformado);
    }

    #[test]
    fn checagem_local_recusa_sozinha_mesmo_se_o_servidor_ainda_aceitaria() {
        // Corrompe só o campo local `valida_ate` para um valor MENOR que o realmente gravado no
        // servidor (ex.: cache local velho ou corrompido) — a checagem local tem que recusar
        // sozinha, "falhar fechado", no instante exato do SEU próprio prazo, mesmo que o servidor
        // ainda aceitasse o bloco.
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let real = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(100)).unwrap();
        let mut bytes = real.para_bytes().to_vec();
        bytes[32..40].copy_from_slice(&50_000u64.to_be_bytes()); // valida_ate local forjado: bem menor
        let forjada = Referencia::de_bytes(&bytes).unwrap();
        assert_eq!(abrir(&servidor, &forjada, 50_000), Err(Erro::Expirado));
    }

    #[test]
    fn checagem_do_servidor_recusa_mesmo_se_o_campo_local_disser_prazo_maior() {
        // Corrompe o campo local `valida_ate` para um valor MAIOR que o realmente gravado no
        // servidor (simula um cliente malicioso tentando "esticar" o próprio prazo sozinho) — o
        // servidor tem a palavra final: expira no prazo REAL, nunca no que o cliente afirma.
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let real = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(1)).unwrap(); // valida_ate real = 86_400
        let mut bytes = real.para_bytes().to_vec();
        bytes[32..40].copy_from_slice(&1_000_000u64.to_be_bytes()); // valida_ate local forjado: bem maior
        let forjada = Referencia::de_bytes(&bytes).unwrap();
        assert_eq!(abrir(&servidor, &forjada, 90_000), Err(Erro::Expirado));
    }

    #[test]
    fn referencia_de_bytes_recusa_chave_publica_cortada() {
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let referencia = encaminhar(&mut servidor, &it, 0, PrazoDeCustodiaMutua::de_dias(10)).unwrap();
        let bytes = referencia.para_bytes();
        let corte = (32 + 8 + envelope::TAM_CHAVE_SECRETA + 100).min(bytes.len());
        let cortado = &bytes[..corte];
        assert!(matches!(Referencia::de_bytes(cortado), Err(Erro::Chave(_))));
    }

    #[test]
    fn servidor_so_ve_valida_ate_arredondado_para_o_dia_nunca_o_segundo_exato() {
        // `agora` propositalmente NÃO alinhado a um limite de dia (nem múltiplo de 86_400) — o valor
        // gravado no servidor tem que, mesmo assim, cair exatamente num limite de dia UTC: achado da
        // auditoria adversarial desta rodada (ver a nota "Metadado de horário" na doc do módulo) — sem
        // isso, o servidor aprenderia o segundo exato do dia em que cada item foi encaminhado.
        let mut servidor = ServidorCego::default();
        for agora in [1u64, 12_345, 86_399, 99_999, 200_000_003] {
            let it = item(cofre::Tipo::Nota, &[], b"x");
            let referencia = encaminhar(&mut servidor, &it, agora, PrazoDeCustodiaMutua::de_dias(5)).unwrap();
            assert_eq!(
                servidor.blocos[&referencia.id].1 % 86_400,
                0,
                "agora={agora}: servidor não pode ver o segundo exato do dia"
            );
        }
    }

    #[test]
    fn arredondamento_no_servidor_nunca_fecha_antes_do_prazo_real_combinado() {
        // O arredondamento só torna a checagem do servidor uma rede de segurança até ~1 dia MAIS
        // LARGA, nunca mais apertada: um segundo antes do prazo verdadeiro (ainda dentro do prazo),
        // abrir tem que continuar funcionando — mesmo com `agora` fora de um limite de dia, caso em
        // que o valor gravado no servidor fica mais alto que o exato.
        let mut servidor = ServidorCego::default();
        let it = item(cofre::Tipo::Arquivo, &[], b"x");
        let agora = 50_000; // não é múltiplo de 86_400 — força o arredondamento a mudar o valor de fato
        let referencia = encaminhar(&mut servidor, &it, agora, PrazoDeCustodiaMutua::de_dias(3)).unwrap();
        let valida_ate_no_servidor = servidor.blocos[&referencia.id].1;
        assert!(valida_ate_no_servidor >= referencia.valida_ate, "arredondar para cima nunca encurta o prazo real");
        assert!(abrir(&servidor, &referencia, referencia.valida_ate - 1).is_ok(), "um segundo antes do prazo exato ainda abre");
    }

    // --- transferência na troca de aparelho (regra 35, achado da rodada anterior corrigido nesta) ---

    #[test]
    fn transferencia_ida_e_volta_preserva_referencias_validas_e_a_chave_do_cofre() {
        let mut servidor = ServidorCego::default();
        let it1 = item(cofre::Tipo::Nota, &[], b"item 1");
        let it2 = item(cofre::Tipo::Foto, &[], b"item 2");
        let r1 = encaminhar(&mut servidor, &it1, 1_000, PrazoDeCustodiaMutua::de_dias(30)).unwrap();
        let r2 = encaminhar(&mut servidor, &it2, 1_000, PrazoDeCustodiaMutua::de_dias(60)).unwrap();
        let (publica_novo, secreta_novo) = envelope::gerar_par_de_chaves();
        let chave_do_cofre = b"chave-local-do-cofre-pessoal-32b";
        let pacote = preparar_pacote_de_transferencia(&[r1, r2], 1_000, chave_do_cofre, &publica_novo).unwrap();
        let (referencias, chave_recebida) = abrir_pacote_de_transferencia(&secreta_novo, &publica_novo, &pacote).unwrap();
        assert_eq!(referencias.len(), 2, "as duas referências válidas chegaram");
        assert_eq!(&chave_recebida[..], chave_do_cofre, "a chave do cofre chegou intacta");
        // As referências recebidas no aparelho novo continuam funcionando normalmente.
        assert_eq!(abrir(&servidor, &referencias[0], 1_000).unwrap(), it1);
        assert_eq!(abrir(&servidor, &referencias[1], 1_000).unwrap(), it2);
    }

    #[test]
    fn transferencia_descarta_referencias_ja_vencidas() {
        let mut servidor = ServidorCego::default();
        let valida = encaminhar(&mut servidor, &item(cofre::Tipo::Nota, &[], b"ainda vale"), 0, PrazoDeCustodiaMutua::de_dias(30)).unwrap();
        let vencida = encaminhar(&mut servidor, &item(cofre::Tipo::Nota, &[], b"ja venceu"), 0, PrazoDeCustodiaMutua::de_dias(1)).unwrap();
        let (publica_novo, secreta_novo) = envelope::gerar_par_de_chaves();
        let depois_do_prazo_da_vencida = 2 * 86_400;
        let pacote = preparar_pacote_de_transferencia(&[valida, vencida], depois_do_prazo_da_vencida, b"chave", &publica_novo).unwrap();
        let (referencias, _) = abrir_pacote_de_transferencia(&secreta_novo, &publica_novo, &pacote).unwrap();
        assert_eq!(referencias.len(), 1, "só a referência ainda válida no momento do preparo entra no pacote");
    }

    #[test]
    fn transferencia_vazia_funciona_sem_pane_quando_nao_ha_nada_a_levar() {
        let (publica_novo, secreta_novo) = envelope::gerar_par_de_chaves();
        let pacote = preparar_pacote_de_transferencia(&[], 1_000, b"", &publica_novo).unwrap();
        let (referencias, chave) = abrir_pacote_de_transferencia(&secreta_novo, &publica_novo, &pacote).unwrap();
        assert!(referencias.is_empty());
        assert!(chave.is_empty());
    }

    #[test]
    fn pacote_de_transferencia_so_abre_com_a_chave_do_aparelho_novo_certo() {
        let mut servidor = ServidorCego::default();
        let r = encaminhar(&mut servidor, &item(cofre::Tipo::Nota, &[], b"x"), 1_000, PrazoDeCustodiaMutua::de_dias(30)).unwrap();
        let (publica_novo, _secreta_novo) = envelope::gerar_par_de_chaves();
        let (publica_de_outro, secreta_de_outro) = envelope::gerar_par_de_chaves();
        let pacote = preparar_pacote_de_transferencia(&[r], 1_000, b"chave", &publica_novo).unwrap();
        assert!(matches!(
            abrir_pacote_de_transferencia(&secreta_de_outro, &publica_de_outro, &pacote),
            Err(Erro::Chave(_))
        ), "selado para o aparelho novo: a chave de um terceiro aparelho não abre");
    }

    #[test]
    fn pacote_de_transferencia_adulterado_nao_abre() {
        let mut servidor = ServidorCego::default();
        let r = encaminhar(&mut servidor, &item(cofre::Tipo::Nota, &[], b"x"), 1_000, PrazoDeCustodiaMutua::de_dias(30)).unwrap();
        let (publica_novo, secreta_novo) = envelope::gerar_par_de_chaves();
        let mut pacote = preparar_pacote_de_transferencia(&[r], 1_000, b"chave", &publica_novo).unwrap();
        let ultimo = pacote.len() - 1;
        pacote[ultimo] ^= 1;
        assert!(abrir_pacote_de_transferencia(&secreta_novo, &publica_novo, &pacote).is_err());
    }

    #[test]
    fn pacote_de_transferencia_com_prefixo_de_tamanho_mentiroso_e_rejeitado_sem_panico() {
        // O envelope (AEAD) está intacto e abre normalmente — o problema está um nível ACIMA: o
        // conteúdo decifrado diz "1 referência de 999 bytes" mas não entrega os 999 bytes depois.
        // Simula um pacote corrompido de outra forma (não por bit-flip, que o teste acima já cobre)
        // ou um bug de outra implementação da ponte. Tem que devolver erro tratado — nunca um
        // `panic!` por indexar fora dos limites do vetor (`claro[i..i + tam]`).
        let (publica_novo, secreta_novo) = envelope::gerar_par_de_chaves();
        let mut claro_mentiroso = Vec::new();
        claro_mentiroso.extend_from_slice(&1u32.to_be_bytes()); // "1 referência"
        claro_mentiroso.extend_from_slice(&999u32.to_be_bytes()); // "...de 999 bytes" — mas não há nenhum depois disto
        let pacote = envelope::selar(&publica_novo, &claro_mentiroso, CONTEXTO_TRANSFERENCIA).unwrap();
        assert!(
            matches!(abrir_pacote_de_transferencia(&secreta_novo, &publica_novo, &pacote), Err(Erro::Malformado)),
            "prefixo de tamanho maior que o que realmente sobra no pacote: erro tratado, nunca pânico"
        );
    }
}
