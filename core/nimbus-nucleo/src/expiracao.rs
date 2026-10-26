//! # Módulo `expiracao` — mensagens temporárias (requisito CH2)
//!
//! * Modo A — destrói `segundos_apos_abrir` depois da primeira abertura (visualização única inclusa).
//! * Modo B — destrói `segundos_apos_enviar` depois do envio, aberta ou não.
//! * Combináveis: vale o prazo que vencer primeiro.
//! * O horário vem do **servidor** e é assinado: mudar o relógio do aparelho não prolonga nada.
//! * Assinatura híbrida Ed25519 + ML-DSA-87 (FIPS 204): as duas precisam conferir.
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};

const CONTEXTO: &[u8] = b"NimbusOne/v1/expiracao";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Politica {
    pub segundos_apos_abrir: Option<u64>,
    pub segundos_apos_enviar: Option<u64>,
    pub visualizacao_unica: bool,
    /// Se não for aberta até este prazo (contado do envio), a mensagem some. Limitado a 48 horas:
    /// nenhuma mensagem não lida vive mais que isso, mesmo que o remetente peça mais.
    pub segundos_para_ler: Option<u64>,
}

/// Prazo máximo de uma mensagem não lida: 48 horas depois de enviada.
pub const LIMITE_PARA_LER: u64 = 48 * 3_600;

/// Vida máxima de QUALQUER mensagem, lida ou não: 48 horas depois de enviada. Nenhuma regra alonga isso.
pub const VIDA_MAXIMA: u64 = 48 * 3_600;

/// Temporizadores oferecidos no momento do envio (some depois de aberta), em segundos:
/// 1, 2, 3, 4, 5, 10, 15 e 30 minutos; 1, 2, 4, 8, 12, 24 e 48 horas.
pub const TEMPORIZADORES: [u64; 15] = [60, 120, 180, 240, 300, 600, 900, 1_800, 3_600, 7_200, 14_400, 28_800, 43_200, 86_400, 172_800];

/// Bilhete de expiração emitido pelo servidor quando recebe a mensagem.
#[derive(Clone, Debug)]
pub struct Bilhete {
    pub id_mensagem: [u8; 16],
    pub enviada_em: u64,
    pub politica: Politica,
    pub assinatura: AssinaturaHibrida,
}

fn codificar(id: &[u8; 16], enviada: u64, p: &Politica) -> Vec<u8> {
    let mut v = b"NimbusOne/v1/expiracao".to_vec();
    v.extend_from_slice(id);
    v.extend_from_slice(&enviada.to_be_bytes());
    for o in [p.segundos_apos_abrir, p.segundos_apos_enviar, p.segundos_para_ler] {
        match o { Some(s) => { v.push(1); v.extend_from_slice(&s.to_be_bytes()); } None => v.push(0) }
    }
    v.push(p.visualizacao_unica as u8);
    v
}

/// Servidor: emite o bilhete assinado.
pub fn emitir(chave_servidor: &ChaveDeAssinatura, id_mensagem: [u8; 16], agora_servidor: u64, politica: Politica) -> Bilhete {
    let assinatura = chave_servidor.assinar(CONTEXTO, &codificar(&id_mensagem, agora_servidor, &politica)).expect("contexto fixo e curto");
    Bilhete { id_mensagem, enviada_em: agora_servidor, politica, assinatura }
}

/// Aparelho: confere a assinatura antes de aceitar a mensagem.
pub fn conferir(chave_publica_servidor: &ChaveDeVerificacao, b: &Bilhete) -> bool {
    chave_publica_servidor.verificar(CONTEXTO, &codificar(&b.id_mensagem, b.enviada_em, &b.politica), &b.assinatura)
}

/// Momento (relógio do servidor) em que a mensagem deve ser destruída. `None` = não expira.
pub fn prazo_final(b: &Bilhete, aberta_em: Option<u64>, fechada_em: Option<u64>) -> Option<u64> {
    let mut d: Option<u64> = None;
    let mut menor = |x: u64| d = Some(d.map_or(x, |c: u64| c.min(x)));
    menor(b.enviada_em.saturating_add(VIDA_MAXIMA));
    if let Some(s) = b.politica.segundos_apos_enviar { menor(b.enviada_em.saturating_add(s)); }
    if let (Some(s), Some(o)) = (b.politica.segundos_apos_abrir, aberta_em) { menor(o.saturating_add(s)); }
    if b.politica.visualizacao_unica { if let Some(f) = fechada_em { menor(f); } }
    if aberta_em.is_none() {
        // O teto de 48 horas vem de VIDA_MAXIMA, que vale para toda mensagem; aqui só o prazo mais curto pedido.
        let para_ler = b.politica.segundos_para_ler.unwrap_or(LIMITE_PARA_LER);
        menor(b.enviada_em.saturating_add(para_ler));
    }
    d
}

pub fn expirou(b: &Bilhete, agora_servidor: u64, aberta_em: Option<u64>, fechada_em: Option<u64>) -> bool {
    prazo_final(b, aberta_em, fechada_em).is_some_and(|d| agora_servidor >= d)
}

// ---------------------------------------------------------------------------------------------------------
// Hora assinada pelo servidor, presa a um desafio do aparelho (OFF2). Sem isso, quem controla a rede (ou uma
// ponte com defeito) poderia entregar uma hora antiga e esticar todos os prazos.
// ---------------------------------------------------------------------------------------------------------

const CONTEXTO_HORA: &[u8] = b"NimbusOne/v1/hora-do-servidor";

/// Resposta do servidor ao pedido de hora: a hora atual e o desafio que o aparelho mandou, assinados juntos.
#[derive(Debug, Clone)]
pub struct HoraDoServidor { pub desafio: [u8; 32], pub segundos: u64, pub assinatura: AssinaturaHibrida }

fn codificar_hora(desafio: &[u8; 32], segundos: u64) -> Vec<u8> {
    let mut v = desafio.to_vec();
    v.extend_from_slice(&segundos.to_be_bytes());
    v
}

/// Servidor: assina a hora junto com o desafio de uso único que o aparelho acabou de mandar.
pub fn emitir_hora(chave_servidor: &ChaveDeAssinatura, desafio: [u8; 32], agora_servidor: u64) -> HoraDoServidor {
    let assinatura = chave_servidor.assinar(CONTEXTO_HORA, &codificar_hora(&desafio, agora_servidor)).expect("contexto fixo e curto");
    HoraDoServidor { desafio, segundos: agora_servidor, assinatura }
}

/// Por que o aparelho recusou a hora recebida.
#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeHora {
    /// Não responde ao desafio que este aparelho mandou: resposta antiga repetida ou de outro aparelho.
    DesafioErrado,
    /// Assinatura do servidor não confere: hora adulterada no caminho.
    AssinaturaInvalida,
    /// Mais antiga que a última hora aceita: ninguém faz o relógio do app voltar.
    VoltouNoTempo,
}

/// Aparelho: aceita a hora só se ela responde ao desafio que ele mandou, a assinatura confere e ela não é
/// anterior à última aceita. O valor aceito é o `ultimo_horario_do_servidor` de `hora_confiavel`; o tempo
/// decorrido conta a partir do momento em que a resposta chegou.
pub fn conferir_hora(chave_publica_servidor: &ChaveDeVerificacao, desafio_enviado: &[u8; 32], ultima_aceita: Option<u64>,
                     h: &HoraDoServidor) -> Result<u64, RecusaDeHora> {
    if &h.desafio != desafio_enviado { return Err(RecusaDeHora::DesafioErrado); }
    if !chave_publica_servidor.verificar(CONTEXTO_HORA, &codificar_hora(&h.desafio, h.segundos), &h.assinatura) {
        return Err(RecusaDeHora::AssinaturaInvalida);
    }
    if ultima_aceita.is_some_and(|u| h.segundos < u) { return Err(RecusaDeHora::VoltouNoTempo); }
    Ok(h.segundos)
}

/// Sem hora confirmada pelo servidor há mais de 15 minutos, o app não exibe conteúdo (OFF2).
pub const LIMITE_SEM_HORA_CONFIRMADA: u64 = 15 * 60;

/// Hora em que o aparelho pode confiar: a última hora assinada pelo servidor mais o tempo decorrido no relógio
/// monotônico do aparelho (que não muda quando alguém altera a hora do sistema). Adiantar a hora do celular só
/// faz o conteúdo sumir antes; atrasar não muda nada. `None` = sem confirmação recente: nada é exibido.
pub fn hora_confiavel(relogio_do_aparelho: u64, ultimo_horario_do_servidor: u64, decorrido_monotonico: Option<u64>) -> Option<u64> {
    let decorrido = decorrido_monotonico?;
    if decorrido > LIMITE_SEM_HORA_CONFIRMADA { return None; }
    Some(relogio_do_aparelho.max(ultimo_horario_do_servidor.saturating_add(decorrido)))
}

// ---------------------------------------------------------------------------------------------------------
// Relógio para DESTRUIR chave vencida mesmo offline (achado de 09/10/2026, segunda rodada do dia — pergunta
// do fundador: "existe uma maneira de, mesmo offline, apagar esse cofre lá?"). Nunca usar para EXIBIR nada
// nem para conceder qualquer acesso — isso continua sendo só `hora_confiavel`, acima, que corretamente nega
// depois de `LIMITE_SEM_HORA_CONFIRMADA` sem contato com o servidor.
// ---------------------------------------------------------------------------------------------------------

/// Tempo para alimentar operações que só DESTROEM chave vencida — nunca para exibir conteúdo ou
/// conceder acesso: `historico::ChavesDoDono::destruir_janelas_vencidas`, `prechaves::Chaveiro::
/// apagar_vencidas`, `metadados::...::destruir_vencidas`. Ao contrário de `hora_confiavel`, PODE e
/// DEVE continuar avançando mesmo sem nenhum contato recente com o servidor — de propósito, SEM o
/// teto de `LIMITE_SEM_HORA_CONFIRMADA`.
///
/// Por quê: destruir é seguro na direção monotônica — avançar este relógio só pode ACELERAR a
/// destruição de uma chave já vencida, nunca conceder acesso a algo que não deveria (não existe
/// "destruir cedo demais de um jeito que vaze alguma coisa"; o pior caso de um valor grande demais
/// aqui é perder um pouco antes do esperado algo que já estava marcado para morrer — disponibilidade,
/// nunca confidencialidade). Gatear DESTRUIR em `hora_confiavel` inverteria a garantia dos três
/// módulos acima (EF6, FS1, MET3: "isto fica ilegível para sempre, mesmo com o aparelho tomado"): um
/// atacante que isola o aparelho da rede (modo avião, bolsa de Faraday) IMEDIATAMENTE depois de
/// capturá-lo, ANTES dos 15 minutos de `LIMITE_SEM_HORA_CONFIRMADA`, impediria para sempre a
/// destruição — justo no caso em que ela mais importa. Ficar offline de propósito se tornaria a
/// melhor estratégia do atacante contra os três módulos, o oposto do que eles existem para garantir.
///
/// Ancora na última hora confirmada pelo servidor e soma só o relógio monotônico do aparelho dali —
/// o mesmo relógio monotônico que `hora_confiavel` já usa, nunca o relógio de parede (que alguém com
/// o aparelho em mãos poderia voltar). Sem nenhuma hora de servidor ainda confirmada nesta sessão
/// (aparelho nunca conectou), `ultimo_horario_do_servidor = 0` ainda funciona: o relógio monotônico
/// sozinho já avança a partir da instalação do app, o que basta para os três módulos acima, cujo
/// prazo de janela é contado a partir de quando o item foi guardado, não de uma época absoluta.
pub fn tempo_para_destruir_mesmo_offline(ultimo_horario_do_servidor: u64, decorrido_monotonico: u64) -> u64 {
    ultimo_horario_do_servidor.saturating_add(decorrido_monotonico)
}

// ---------------------------------------------------------------------------------------------------------
// Cláusula 6b — aceleração do relógio de destruição por tentativas de desbloqueio malsucedidas (achado de
// 10/10/2026, pedido do fundador: "vínculo entre tentativas de desbloqueio malsucedidas e a destruição por
// prazo e aceleração do relógio, da maneira mais sincronizada, absoluta e perfeita possível"). Não é um
// relógio novo: ensina o MESMO relógio da Cláusula 6a (acima) a correr mais rápido quando
// `coacao::Tentativas` — contador local, protegido pelo chip, no mesmo aparelho — mostra um padrão de
// erros consistente com força bruta contra o PIN.
//
// O que isto NÃO substitui: `coacao::Politica::erros_para_apagar`/`apagar_apos_escolha_do_usuario` já
// apagam as chaves NA HORA ao bater um número fixo de erros — esse caminho continua existindo, inalterado,
// e dispara independentemente deste. A Cláusula 6b é aditiva e diferente: faz as JANELAS DE PRAZO que
// `historico`/`prechaves`/`metadados` já aplicam (EF6, FS1, MET3) vencerem mais rápido do que o tempo real
// passaria, na presença de um ataque sustentado — nunca cria prazo do nada para algo que seria permanente,
// e nunca acelera nada para quem só errou o PIN duas ou três vezes por descuido (ver `limiar`).
//
// Por que isto não é "absoluto" no sentido literal, dito abertamente aqui, não escondido: um aparelho que
// nunca gera um "erro de PIN" registrado — porque foi lido no nível de disco antes de qualquer tentativa
// pela tela (extração forense direta do armazenamento, sem passar pela interface que conta erros) — não
// alimenta este contador e não aciona esta aceleração. A Cláusula 6a (sempre ativa, sem depender de
// nenhum erro de PIN) continua sendo a garantia de base; a 6b é um reforço para o caso mais comum (alguém
// com o aparelho em mãos tentando o PIN pela tela), não uma substituta dela.
// ---------------------------------------------------------------------------------------------------------

/// Política de aceleração do relógio de destruição por tentativas malsucedidas (Cláusula 6b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AceleracaoPorTentativas {
    /// Erros consecutivos além deste número começam a acelerar o relógio. Com `erros_consecutivos` igual
    /// ou abaixo disto, zero efeito.
    pub limiar: u32,
    /// Segundos extras somados ao relógio de destruição, por erro além do limiar.
    pub segundos_extras_por_erro: u64,
    /// Teto da aceleração extra (satura, nunca ultrapassa) — protege contra um contador de erros
    /// corrompido ou uma sequência de ataque muito longa jogando o relógio para um valor absurdo de
    /// uma vez só.
    pub teto_de_segundos_extras: u64,
}

impl AceleracaoPorTentativas {
    /// Padrão sugerido desta rodada: usa o MESMO limiar de tentativas livres que `coacao::ERROS_SEM_ESPERA`
    /// já define — sincronizado por referência ao código, de propósito, nunca por dois números literais
    /// que um dia poderiam divergir um do outro. Cada erro além dele soma 1 hora ao relógio de destruição;
    /// teto de 30 dias de aceleração extra numa única sequência de ataque. Estes três números são uma
    /// escolha de engenharia razoável, não uma medição nem um valor calibrado com dados de uso real — ver
    /// "Verificar antes de usar" no CONTRATO_DA_PONTE.md antes de os citar em material comercial.
    /// Use `padrao()` só quando não houver uma `Politica` específica à mão. Quando houver — o caso
    /// normal, já que toda conta tem uma — use `a_partir_da_politica`, abaixo, em vez desta.
    pub fn padrao() -> Self {
        AceleracaoPorTentativas {
            limiar: crate::coacao::ERROS_SEM_ESPERA,
            segundos_extras_por_erro: 3_600,
            teto_de_segundos_extras: 30 * 86_400,
        }
    }

    /// Mitigação do maior risco deixado aberto ao escrever a Cláusula 6b (achado do dono em 10/10/2026,
    /// mesma rodada: "ela protege contra ataque, mas também cria um risco real e novo de perda de dados
    /// por acidente" — alguém destreinado, com pressa, sob efeito de álcool ou com dificuldade motora que
    /// erra o PIN repetidamente por engano, sem perceber que está acelerando o próprio relógio de
    /// destruição). **Correção de uma inconsistência real, não um ajuste cosmético:** `padrao()` fixava o
    /// limiar sempre em `coacao::ERROS_SEM_ESPERA` (3), mesmo para quem escolheu `Politica::tolerante()`
    /// (5 erros livres) exatamente para ter MENOS pressa — com `padrao()`, a Cláusula 6b já começava a
    /// acelerar o relógio no 4º erro, um erro ANTES de a própria tolerância escolhida pela pessoa nem
    /// começar a fazer a tela esperar (regra 13). `a_partir_da_politica` sincroniza o limiar da 6b com o
    /// `erros_sem_espera` da política REALMENTE em vigor para aquela conta — rígida acelera mais cedo (2),
    /// tolerante mais tarde (5), exatamente como cada uma já decide a espera entre tentativas. A taxa
    /// (1h/erro) e o teto (30 dias) continuam os mesmos de `padrao()` nesta rodada, de propósito: mudar só
    /// o limiar corrige a inconsistência encontrada; mudar também a taxa por política (ex.: tolerante
    /// também acelerar mais devagar) é uma decisão de produto maior, registrada como aberta — não
    /// decidida — em `PENDENCIAS_E_DECISOES.md`.
    pub fn a_partir_da_politica(pol: &crate::coacao::Politica) -> Self {
        AceleracaoPorTentativas { limiar: pol.erros_sem_espera, ..Self::padrao() }
    }

    /// Quantos segundos JÁ foram somados ao relógio de destruição pela Cláusula 6b, para este número de
    /// erros consecutivos — público desde esta rodada, para que a tela de desbloqueio possa mostrar algo
    /// como "a autodestruição deste cofre foi antecipada em N horas por tentativas incorretas", em vez de
    /// a pessoa só descobrir o efeito depois, sem nenhum aviso. Zero enquanto `erros_consecutivos` não
    /// passar do limiar (ver `tentativas_restantes_antes_de_acelerar`, abaixo, para o aviso ANTES disso).
    pub fn segundos_extras(&self, erros_consecutivos: u32) -> u64 {
        let alem_do_limiar = erros_consecutivos.saturating_sub(self.limiar) as u64;
        alem_do_limiar.saturating_mul(self.segundos_extras_por_erro).min(self.teto_de_segundos_extras)
    }

    /// Quantos erros de PIN ainda podem acontecer antes do PRÓXIMO já começar a acelerar o relógio de
    /// destruição — a peça que faltava para a tela avisar "você está a X tentativas disto" ANTES do fato,
    /// não só mostrar o estrago já feito depois. `0` significa que o próximo erro, seja ele qual for, já
    /// acelera (inclusive quando já se está além do limiar). Pensada para a tela mostrar um aviso quando
    /// isto chegar a `1` — nem antes (descuido comum não precisa de alarme, ver o teste de contraste em
    /// `integracao_poucos_erros_de_descuido_nao_aceleram_nada_e_a_janela_sobrevive`) nem só depois (tarde
    /// demais para quem ainda podia parar e usar a biometria em vez de insistir no PIN).
    pub fn tentativas_restantes_antes_de_acelerar(&self, erros_consecutivos: u32) -> u32 {
        self.limiar.saturating_add(1).saturating_sub(erros_consecutivos)
    }
}

/// Relógio da Cláusula 6a ACELERADO pela Cláusula 6b. Nunca retrocede, nunca fica abaixo do que
/// `tempo_para_destruir_mesmo_offline` sozinho já devolveria (só soma), nunca depende do relógio de parede
/// nem de contato com o servidor — a mesma garantia monotônica de sempre, mais os segundos extras que o
/// padrão de erros define. `erros_consecutivos` vem de `coacao::Tentativas::erros_consecutivos()`, o
/// MESMO contador que já decide a espera entre tentativas (regra 13 do CONTRATO_DA_PONTE.md) — nenhum
/// contador novo, nenhuma segunda fonte de verdade para "quantos erros houve".
pub fn tempo_para_destruir_considerando_tentativas(
    ultimo_horario_do_servidor: u64,
    decorrido_monotonico: u64,
    erros_consecutivos: u32,
    aceleracao: &AceleracaoPorTentativas,
) -> u64 {
    tempo_para_destruir_mesmo_offline(ultimo_horario_do_servidor, decorrido_monotonico)
        .saturating_add(aceleracao.segundos_extras(erros_consecutivos))
}

/// Pode mostrar a mensagem agora? Só com hora confiável e dentro do prazo.
pub fn pode_exibir(b: &Bilhete, hora: Option<u64>, aberta_em: Option<u64>, fechada_em: Option<u64>) -> bool {
    hora.is_some_and(|h| !expirou(b, h, aberta_em, fechada_em))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;
    fn chave() -> ChaveDeAssinatura { assinatura::gerar() }

    #[test]
    fn hora_do_servidor_presa_ao_desafio() {
        let k = chave(); let v = k.verificacao();
        let desafio = [3u8; 32];
        let h = emitir_hora(&k, desafio, 5_000);
        assert_eq!(conferir_hora(&v, &desafio, None, &h), Ok(5_000));
        assert_eq!(conferir_hora(&v, &desafio, Some(5_000), &h), Ok(5_000), "a mesma hora pode ser reaceita");
        // Resposta antiga, guardada por quem controla a rede, para outro desafio: recusada.
        let antiga = emitir_hora(&k, [9u8; 32], 1_000);
        assert_eq!(conferir_hora(&v, &desafio, None, &antiga), Err(RecusaDeHora::DesafioErrado));
        // Mesmo desafio, hora trocada no caminho: assinatura não confere.
        let mut adulterada = h.clone(); adulterada.segundos = 1_000;
        assert_eq!(conferir_hora(&v, &desafio, None, &adulterada), Err(RecusaDeHora::AssinaturaInvalida));
        // Assinada por outro servidor: recusada.
        let outro = assinatura::gerar();
        assert_eq!(conferir_hora(&v, &desafio, None, &emitir_hora(&outro, desafio, 5_000)), Err(RecusaDeHora::AssinaturaInvalida));
        // Hora legítima, mas anterior à última aceita: o relógio do app não volta.
        let cedo = emitir_hora(&k, desafio, 4_000);
        assert_eq!(conferir_hora(&v, &desafio, Some(5_000), &cedo), Err(RecusaDeHora::VoltouNoTempo));
    }

    #[test]
    fn atrasar_o_relogio_do_celular_nao_estende_nada() {
        let k = chave();
        let b = emitir(&k, [9; 16], 1_000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(60), visualizacao_unica: false, segundos_para_ler: None });
        // Servidor disse 1_050; passaram 20 s no relógio monotônico; o celular foi atrasado para 500.
        let h = hora_confiavel(500, 1_050, Some(20));
        assert_eq!(h, Some(1_070));
        assert!(!pode_exibir(&b, h, None, None));
    }

    #[test]
    fn adiantar_o_relogio_so_faz_sumir_antes() {
        let k = chave();
        let b = emitir(&k, [10; 16], 1_000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(600), visualizacao_unica: false, segundos_para_ler: None });
        assert!(pode_exibir(&b, hora_confiavel(1_010, 1_000, Some(10)), None, None));
        assert!(!pode_exibir(&b, hora_confiavel(99_999, 1_000, Some(10)), None, None));
    }

    #[test]
    fn sem_hora_confirmada_nada_aparece() {
        let k = chave();
        let b = emitir(&k, [11; 16], 1_000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: None, visualizacao_unica: false, segundos_para_ler: None });
        assert_eq!(hora_confiavel(1_000, 1_000, Some(LIMITE_SEM_HORA_CONFIRMADA)), Some(1_000 + LIMITE_SEM_HORA_CONFIRMADA));
        assert_eq!(hora_confiavel(1_000, 1_000, Some(LIMITE_SEM_HORA_CONFIRMADA + 1)), None);
        assert_eq!(hora_confiavel(1_000, 1_000, None), None, "reiniciou sem internet: sem relógio monotônico");
        assert!(!pode_exibir(&b, None, None, None));
        assert!(pode_exibir(&b, Some(1_001), None, None));
    }

    #[test]
    fn modo_b_expira_aberta_ou_nao() {
        let k = chave();
        let b = emitir(&k, [1; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(60), visualizacao_unica: false, segundos_para_ler: None });
        assert!(!expirou(&b, 1059, None, None));
        assert!(expirou(&b, 1060, None, None));
    }

    #[test]
    fn modo_a_conta_a_partir_da_abertura() {
        let k = chave();
        let b = emitir(&k, [2; 16], 1000, Politica { segundos_apos_abrir: Some(30), segundos_apos_enviar: None, visualizacao_unica: false, segundos_para_ler: None });
        assert!(expirou(&b, 5030, Some(5000), None), "aberta às 5000, some 30 s depois");
        assert!(!expirou(&b, 5029, Some(5000), None));
    }

    #[test]
    fn nao_lida_some_em_48_horas_no_maximo() {
        let k = chave();
        // Conversa com prazo longo (7 dias) e ninguém abre: some em 48 horas mesmo assim.
        let b = emitir(&k, [6; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(7 * 86_400), visualizacao_unica: false, segundos_para_ler: None });
        assert!(!expirou(&b, 1000 + LIMITE_PARA_LER - 1, None, None));
        assert!(expirou(&b, 1000 + LIMITE_PARA_LER, None, None));
    }

    #[test]
    fn remetente_pode_pedir_prazo_menor_para_ler_mas_nunca_maior_que_48_horas() {
        let k = chave();
        let curto = emitir(&k, [7; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: None, visualizacao_unica: false, segundos_para_ler: Some(6 * 3_600) });
        assert!(expirou(&curto, 1000 + 6 * 3_600, None, None));
        let longo = emitir(&k, [8; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: None, visualizacao_unica: false, segundos_para_ler: Some(5 * 86_400) });
        assert!(expirou(&longo, 1000 + LIMITE_PARA_LER, None, None), "pedido de 5 dias é limitado a 48 horas");
    }

    #[test]
    fn depois_de_aberta_o_prazo_para_ler_nao_vale_mais() {
        let k = chave();
        let b = emitir(&k, [9; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(24 * 3_600), visualizacao_unica: false, segundos_para_ler: Some(3_600) });
        assert!(!expirou(&b, 1000 + 10 * 3_600, Some(1500), None), "lida a tempo: segue o prazo de 24 horas");
        assert!(expirou(&b, 1000 + 24 * 3_600, Some(1500), None));
    }

    #[test]
    fn nenhuma_mensagem_vive_mais_de_48_horas_nem_lida() {
        let k = chave();
        let b = emitir(&k, [11; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(30 * 86_400), visualizacao_unica: false, segundos_para_ler: None });
        assert!(!expirou(&b, 1000 + VIDA_MAXIMA - 1, Some(1100), None));
        assert!(expirou(&b, 1000 + VIDA_MAXIMA, Some(1100), None), "lida e com prazo de 30 dias: some em 48 horas");
        let sem_prazo = emitir(&k, [12; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: None, visualizacao_unica: false, segundos_para_ler: None });
        assert_eq!(prazo_final(&sem_prazo, Some(1100), None), Some(1000 + VIDA_MAXIMA));
    }

    #[test]
    fn temporizadores_do_envio_vao_de_1_minuto_a_48_horas() {
        assert_eq!((TEMPORIZADORES[0], *TEMPORIZADORES.last().unwrap()), (60, VIDA_MAXIMA));
        assert!(TEMPORIZADORES.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(&TEMPORIZADORES[..5], &[60, 120, 180, 240, 300], "1, 2, 3, 4 e 5 minutos");
    }

    #[test]
    fn prazo_para_ler_faz_parte_do_que_o_servidor_assina() {
        let k = chave();
        let mut b = emitir(&k, [10; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: None, visualizacao_unica: false, segundos_para_ler: Some(3_600) });
        assert!(conferir(&k.verificacao(), &b));
        b.politica.segundos_para_ler = Some(40 * 3_600);
        assert!(!conferir(&k.verificacao(), &b), "alterar o prazo para ler invalida a assinatura");
    }

    #[test]
    fn combinado_vale_o_primeiro() {
        let k = chave();
        let b = emitir(&k, [3; 16], 1000, Politica { segundos_apos_abrir: Some(600), segundos_apos_enviar: Some(100), visualizacao_unica: false, segundos_para_ler: None });
        assert_eq!(prazo_final(&b, Some(1010), None), Some(1100));
    }

    #[test]
    fn visualizacao_unica_expira_ao_fechar() {
        let k = chave();
        let b = emitir(&k, [4; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(86_400), visualizacao_unica: true, segundos_para_ler: None });
        assert!(expirou(&b, 1200, Some(1100), Some(1150)));
    }

    #[test]
    fn prazo_ou_horario_adulterado_e_rejeitado() {
        let k = chave();
        let mut b = emitir(&k, [5; 16], 1000, Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(60), visualizacao_unica: false, segundos_para_ler: None });
        assert!(conferir(&k.verificacao(), &b));
        b.politica.segundos_apos_enviar = Some(999_999);
        assert!(!conferir(&k.verificacao(), &b), "aumentar o prazo invalida a assinatura");
        b.politica.segundos_apos_enviar = Some(60);
        b.enviada_em = 9_999_999;
        assert!(!conferir(&k.verificacao(), &b), "mudar o horário invalida a assinatura");
        b.enviada_em = 1000;
        assert!(conferir(&k.verificacao(), &b));
        b.politica.visualizacao_unica = true;
        assert!(!conferir(&k.verificacao(), &b), "ligar ou desligar a visualização única invalida a assinatura");
    }

    // --- relógio para destruir mesmo offline (achado desta rodada) ---

    #[test]
    fn tempo_para_destruir_continua_avancando_onde_hora_confiavel_ja_pararia() {
        // Bem depois do teto de 15 minutos sem contato com o servidor: hora_confiavel já nega.
        let muito_tempo_offline = LIMITE_SEM_HORA_CONFIRMADA * 1000; // ~10 dias sem nenhum contato
        assert_eq!(hora_confiavel(0, 1_000, Some(muito_tempo_offline)), None, "exibir conteúdo: corretamente negado depois de 15 min offline");
        // O relógio de destruir, com os MESMOS dados, continua avançando sem nenhum teto.
        assert_eq!(
            tempo_para_destruir_mesmo_offline(1_000, muito_tempo_offline),
            1_000 + muito_tempo_offline,
            "destruir chave vencida: nunca para só porque o aparelho está offline"
        );
    }

    #[test]
    fn tempo_para_destruir_ancora_na_ultima_hora_do_servidor_mais_o_monotonico() {
        assert_eq!(tempo_para_destruir_mesmo_offline(10_000, 500), 10_500);
        assert_eq!(tempo_para_destruir_mesmo_offline(10_000, 0), 10_000, "sem tempo decorrido, nenhuma destruição extra");
    }

    #[test]
    fn tempo_para_destruir_funciona_mesmo_sem_nenhuma_hora_de_servidor_ainda_confirmada() {
        // Aparelho que nunca conectou nesta sessão: ultimo_horario_do_servidor = 0. O relógio
        // monotônico sozinho, contado desde a instalação do app, já basta para os três módulos que
        // usam isto (o prazo deles conta a partir de quando o item foi guardado, não de uma época
        // absoluta de calendário).
        assert_eq!(tempo_para_destruir_mesmo_offline(0, 7 * 86_400), 7 * 86_400);
    }

    #[test]
    fn integracao_destruir_vencida_mesmo_sem_contato_com_servidor_historico() {
        // Prova de ponta a ponta com o módulo que motivou o achado: uma janela de historico que
        // venceria em 1 hora é destruída usando só o relógio monotônico — nenhuma chamada a
        // hora_confiavel, nenhum contato com o servidor em todo o teste.
        use crate::historico::{ChavesDoDono, ServidorCego};
        let (mut servidor, mut chaves) = (ServidorCego::default(), ChavesDoDono::novas(3_600));
        let id = crate::historico::guardar(&mut servidor, &mut chaves, b"no cofre do aparelho perdido", Some(1_800)).unwrap();
        assert!(crate::historico::ler(&servidor, &chaves, &id).is_ok(), "ainda dentro do prazo");
        // O aparelho fica offline por muito mais que os 15 min de hora_confiavel — mas o relógio
        // monotônico da instalação continua contando por conta própria.
        let agora_para_destruir = tempo_para_destruir_mesmo_offline(0, 2 * 3_600);
        assert_eq!(chaves.destruir_janelas_vencidas(agora_para_destruir), 1, "destruída usando só o relógio monotônico, sem nenhum contato com o servidor");
        assert!(crate::historico::ler(&servidor, &chaves, &id).is_err(), "ilegível para sempre a partir daqui, mesmo que o aparelho nunca mais conecte");
    }

    // --- Cláusula 6b: aceleração do relógio de destruição por tentativas malsucedidas (achado desta rodada) ---

    #[test]
    fn aceleracao_nao_faz_nada_no_limiar_ou_abaixo_dele() {
        let pol = AceleracaoPorTentativas::padrao(); // limiar = coacao::ERROS_SEM_ESPERA = 3
        assert_eq!(tempo_para_destruir_considerando_tentativas(0, 1_000, 0, &pol), 1_000, "nenhum erro: zero aceleração");
        assert_eq!(tempo_para_destruir_considerando_tentativas(0, 1_000, 3, &pol), 1_000, "exatamente no limiar: ainda zero aceleração");
    }

    #[test]
    fn aceleracao_cresce_uma_hora_por_erro_depois_do_limiar() {
        let pol = AceleracaoPorTentativas::padrao();
        assert_eq!(tempo_para_destruir_considerando_tentativas(0, 0, 4, &pol), 3_600, "1º erro além do limiar: +1h");
        assert_eq!(tempo_para_destruir_considerando_tentativas(0, 0, 5, &pol), 7_200, "2º erro além do limiar: +2h");
        assert_eq!(tempo_para_destruir_considerando_tentativas(0, 0, 13, &pol), 10 * 3_600, "10º erro além do limiar: +10h");
    }

    #[test]
    fn aceleracao_tem_teto_e_satura_mesmo_com_milhares_de_erros() {
        let pol = AceleracaoPorTentativas::padrao();
        assert_eq!(
            tempo_para_destruir_considerando_tentativas(0, 0, 100_000, &pol),
            pol.teto_de_segundos_extras,
            "satura no teto de 30 dias, não cresce sem limite"
        );
        assert_eq!(
            tempo_para_destruir_considerando_tentativas(0, 0, u32::MAX, &pol),
            pol.teto_de_segundos_extras,
            "nem um contador no limite do tipo passa do teto"
        );
    }

    #[test]
    fn aceleracao_padrao_usa_o_mesmo_limiar_de_coacao_por_referencia_nao_por_numero_duplicado() {
        assert_eq!(
            AceleracaoPorTentativas::padrao().limiar,
            crate::coacao::ERROS_SEM_ESPERA,
            "sincronizado com a regra 13: um só número, nunca dois que podem divergir um dia"
        );
    }

    #[test]
    fn aceleracao_nunca_deixa_o_relogio_abaixo_do_que_a_clausula_6a_sozinha_ja_daria() {
        let pol = AceleracaoPorTentativas::padrao();
        for erros in [0, 1, 3, 4, 10, 1_000, u32::MAX] {
            let base = tempo_para_destruir_mesmo_offline(5_000, 2_000);
            let acelerado = tempo_para_destruir_considerando_tentativas(5_000, 2_000, erros, &pol);
            assert!(acelerado >= base, "a aceleração nunca reduz o relógio de destruição, só pode somar (erros={erros})");
        }
    }

    #[test]
    fn integracao_ataque_de_forca_bruta_acelera_destruicao_de_janela_que_o_relogio_base_ainda_nao_alcancaria() {
        // Prova de ponta a ponta, composta com os dois módulos de verdade: `coacao` conta os erros do PIN
        // exatamente como a regra 13 já faz, e `historico` é quem de fato destrói a chave da janela.
        use crate::historico::{ChavesDoDono, ServidorCego};
        let (mut servidor, mut chaves) = (ServidorCego::default(), ChavesDoDono::novas(3_600));
        // Janela de 2 horas: pelo relógio real, sem nenhum erro de PIN, só venceria em 7_200 s.
        let id = crate::historico::guardar(&mut servidor, &mut chaves, b"no cofre sob ataque", Some(7_200)).unwrap();

        // Só 1h de tempo monotônico passou — pelo relógio BASE da Cláusula 6a (sem ataque), a janela de 2h
        // ainda não venceu.
        let base = tempo_para_destruir_mesmo_offline(0, 3_600);
        assert_eq!(chaves.destruir_janelas_vencidas(base), 0, "sem ataque, 1h de relógio real não alcança uma janela de 2h");
        assert!(crate::historico::ler(&servidor, &chaves, &id).is_ok(), "ainda legível: a janela de 2h não venceu de verdade");

        // No mesmo intervalo, 10 tentativas de PIN malsucedidas foram registradas — padrão consistente
        // com força bruta, contado pelo MESMO mecanismo da regra 13, nunca um contador separado.
        let mut tentativas = crate::coacao::Tentativas::default();
        let politica = crate::coacao::Politica::tolerante(); // nunca apaga: aqui só interessa a contagem de erros
        for _ in 0..10 { let _ = tentativas.errou(&politica, 0); }
        assert_eq!(tentativas.erros_consecutivos(), 10);

        let acelerado = tempo_para_destruir_considerando_tentativas(0, 3_600, tentativas.erros_consecutivos(), &AceleracaoPorTentativas::padrao());
        // limiar=3; 10-3=7 erros além do limiar × 3_600 s = 25_200 s extras. 3_600 (base) + 25_200 = 28_800.
        assert_eq!(acelerado, 28_800);
        assert_eq!(chaves.destruir_janelas_vencidas(acelerado), 1, "o ataque acelerou o relógio além do prazo de 2h da janela, destruindo-a antes do tempo real equivalente");
        assert!(
            crate::historico::ler(&servidor, &chaves, &id).is_err(),
            "ilegível para sempre a partir daqui — destruída pela aceleração, não pelo tempo real decorrido"
        );
    }

    #[test]
    fn integracao_poucos_erros_de_descuido_nao_aceleram_nada_e_a_janela_sobrevive() {
        // Contraste deliberado com o teste acima: 2 erros (abaixo do limiar de 3) são o tipo de descuido
        // comum (dedo errou o PIN) — a Cláusula 6b não pode punir isso com perda de dados.
        use crate::historico::{ChavesDoDono, ServidorCego};
        let (mut servidor, mut chaves) = (ServidorCego::default(), ChavesDoDono::novas(3_600));
        let id = crate::historico::guardar(&mut servidor, &mut chaves, b"no cofre de uso normal", Some(7_200)).unwrap();

        let mut tentativas = crate::coacao::Tentativas::default();
        let politica = crate::coacao::Politica::equilibrada();
        let _ = tentativas.errou(&politica, 0);
        let _ = tentativas.errou(&politica, 0);
        assert_eq!(tentativas.erros_consecutivos(), 2, "abaixo do limiar de 3 (ERROS_SEM_ESPERA)");

        let acelerado = tempo_para_destruir_considerando_tentativas(0, 3_600, tentativas.erros_consecutivos(), &AceleracaoPorTentativas::padrao());
        assert_eq!(acelerado, 3_600, "2 erros de descuido: zero aceleração, idêntico ao relógio base");
        assert_eq!(chaves.destruir_janelas_vencidas(acelerado), 0, "a janela de 2h não é afetada por 2 erros de descuido");
        assert!(crate::historico::ler(&servidor, &chaves, &id).is_ok(), "continua legível: descuido não é ataque");
    }

    // --- mitigação do risco de perda acidental da Cláusula 6b (achado do dono em 10/10/2026, mesma rodada) ---

    #[test]
    fn a_partir_da_politica_sincroniza_o_limiar_com_a_politica_escolhida_em_vez_do_numero_global_fixo() {
        assert_eq!(AceleracaoPorTentativas::a_partir_da_politica(&crate::coacao::Politica::rigida()).limiar, 2);
        assert_eq!(AceleracaoPorTentativas::a_partir_da_politica(&crate::coacao::Politica::equilibrada()).limiar, 3);
        assert_eq!(AceleracaoPorTentativas::a_partir_da_politica(&crate::coacao::Politica::tolerante()).limiar, 5);
        assert_eq!(AceleracaoPorTentativas::padrao().limiar, 3, "padrao() continua fixo em ERROS_SEM_ESPERA, de propósito, para quem não tiver Politica à mão");
    }

    #[test]
    fn a_partir_da_politica_mantem_a_mesma_taxa_e_teto_do_padrao_so_o_limiar_muda() {
        let a = AceleracaoPorTentativas::a_partir_da_politica(&crate::coacao::Politica::tolerante());
        let p = AceleracaoPorTentativas::padrao();
        assert_eq!(a.segundos_extras_por_erro, p.segundos_extras_por_erro);
        assert_eq!(a.teto_de_segundos_extras, p.teto_de_segundos_extras);
        assert_ne!(a.limiar, p.limiar, "só o limiar diferencia as duas, para a política tolerante");
    }

    #[test]
    fn politica_tolerante_nao_acelera_mais_cedo_do_que_a_propria_tolerancia_escolhida() {
        // O bug real que esta função corrige: com padrao() (limiar fixo em 3), quem escolheu tolerante()
        // (5 erros livres, de propósito, para ter MENOS pressa) ainda assim via a 6b acelerar no 4º erro —
        // um erro ANTES da própria tolerância escolhida nem começar a fazer a tela esperar. Com
        // a_partir_da_politica, isso não acontece mais.
        let tolerante = crate::coacao::Politica::tolerante();
        let com_bug = AceleracaoPorTentativas::padrao();
        let corrigido = AceleracaoPorTentativas::a_partir_da_politica(&tolerante);
        assert!(
            tempo_para_destruir_considerando_tentativas(0, 1_000, 4, &com_bug) > 1_000,
            "demonstração do bug antigo: padrao() já acelerava no 4º erro mesmo para quem escolheu tolerante"
        );
        assert_eq!(
            tempo_para_destruir_considerando_tentativas(0, 1_000, 4, &corrigido), 1_000,
            "corrigido: no 4º erro, ainda dentro dos 5 livres da própria política tolerante escolhida, zero aceleração"
        );
        assert!(
            tempo_para_destruir_considerando_tentativas(0, 1_000, 6, &corrigido) > 1_000,
            "mas continua acelerando normalmente depois do limiar da própria política (6º erro, além dos 5 livres)"
        );
    }

    #[test]
    fn tentativas_restantes_antes_de_acelerar_avisa_antes_do_fato_nao_so_depois() {
        let pol = AceleracaoPorTentativas::padrao(); // limiar = 3
        assert_eq!(pol.tentativas_restantes_antes_de_acelerar(0), 4, "zero erros: 4 tentativas de margem");
        assert_eq!(pol.tentativas_restantes_antes_de_acelerar(2), 2);
        assert_eq!(pol.tentativas_restantes_antes_de_acelerar(3), 1, "no limiar: só mais 1 erro para começar a acelerar — o momento de avisar");
        assert_eq!(pol.tentativas_restantes_antes_de_acelerar(4), 0, "já passou do limiar: o próximo erro, qualquer um, já acelera");
        assert_eq!(pol.tentativas_restantes_antes_de_acelerar(1_000), 0, "nunca fica negativo");
    }

    #[test]
    fn segundos_extras_e_publico_e_mostra_quanto_ja_foi_acelerado_sozinho() {
        // Público desde esta rodada: a tela pode mostrar "antecipado em N horas" sem precisar passar pelo
        // relógio inteiro de destruição — só com o contador de erros que já tem em mãos.
        let pol = AceleracaoPorTentativas::padrao();
        assert_eq!(pol.segundos_extras(3), 0, "no limiar, nada ainda");
        assert_eq!(pol.segundos_extras(4), 3_600, "1h no primeiro erro além do limiar");
        assert_eq!(pol.segundos_extras(5), 7_200);
    }
}
