//! # Módulo `panico` — travar o app na hora do perigo (PAN1–PAN9)
//!
//! Vários jeitos de fechar tudo depressa, para situações diferentes. Toda decisão acontece **dentro do
//! aparelho**: nada é enviado para servidor nenhum, e nenhum gatilho conta o motivo real para lugar algum.
//!
//! Os gatilhos:
//! * **Palavra mágica** — uma palavra ou frase que você define; digitada no lugar do PIN, ou dita para o
//!   assistente, tranca na hora.
//! * **Toque de pânico** — um jeito combinado de segurar o botão (ex.: apertar liga/desliga 5 vezes).
//! * **Longe de casa** — ao sair de uma região marcada como segura, funções escolhidas se fecham sozinhas.
//! * **Movimento brusco** — um tranco forte (arrancar o aparelho da mão) tranca na hora.
//! * **Coação** — o PIN de coação (módulo `coacao`) abre um cofre-isca e dispara o modo de proteção.
//! * **Sem me mexer** — depois de um tempo sem você tocar no aparelho, tranca.
//! * **Ataque detectado** — enquanto o app está aberto e rodando, a ponte nativa combina os sinais que o
//!   módulo `integridade` já mede (depurador anexado, ganchos de instrumentação como Frida/Xposed/Substrate,
//!   depuração USB/ADB ligada, aparelho que virou root/jailbreak NO MEIO da sessão) e manda este sinal único.
//!   **Limite honesto, que precisa estar no material do cliente**: isto fecha a janela de um ataque ao vivo,
//!   contra um app aberto e autenticado num aparelho ligado — não é proteção contra ferramenta forense que
//!   extrai o aparelho desligado ou travado por exploração de firmware/bootloader (ex.: Cellebrite/GrayKey
//!   nesse modo), porque nesse caso nenhum código do app está rodando para reagir. Documentar os dois casos
//!   separadamente evita prometer o que a física do aparelho não permite.
//! * **Comando remoto** (novo, 10/10/2026, pedido do dono: "quando o dono conseguir acessar de um
//!   computador ou outro aparelho, enviar um sinal para bloquear, apagar e destruir o que está no
//!   temporário") — a ÚNICA exceção deliberada e estreita à regra "nada é enviado para servidor nenhum"
//!   do resto deste módulo: a existência de UM comando (quem é o aparelho-alvo, qual reação) precisa
//!   atravessar o servidor para chegar ao aparelho perdido/roubado ainda com rede — o motivo real nunca
//!   atravessa, só o endereçamento. A reação não é fixa na configuração como os outros gatilhos: ela vem
//!   DENTRO do próprio comando assinado (o dono escolhe, no momento de emitir, entre travar e apagar tudo —
//!   ver `comando_remoto.rs`), por isso este gatilho, sozinho, não carrega uma `Reacao` fixa. Autenticação,
//!   assinatura e proteção contra repetição são responsabilidade de `comando_remoto::validar` — chamado
//!   ANTES de este módulo ver o sinal; `panico` só recebe um resultado já verificado, nunca o comando bruto.
//!   **Limite honesto, herdado da própria Cláusula 6a do `CONTRATO_DA_PONTE.md`:** um aparelho isolado da
//!   rede de propósito (modo avião, bolsa de Faraday) antes de o comando chegar derrota isto por desenho —
//!   é exatamente por isso que a Cláusula 6a (apagamento criptográfico por janela vencida, nunca depende
//!   de sinal algum) continua sendo a proteção de fundo, com ou sem este comando chegando.
//!
//! Cada gatilho tem uma **reação**, do mais leve ao mais forte, escolhida por você — exceto o Comando
//! Remoto, cuja reação vem do próprio comando (ver acima).

/// O que o app faz quando um gatilho dispara. Da mais leve à mais forte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reacao {
    /// Só tranca a tela (desbloqueia com PIN/biometria).
    Trancar,
    /// Tranca e esconde as conversas e álbuns marcados como sensíveis: somem da lista até você reativar.
    EsconderSensiveis,
    /// Abre o cofre-isca, com conteúdo inofensivo (o mesmo do PIN de coação).
    MostrarIsca,
    /// Apaga as chaves deste aparelho: a conta volta pela frase de recuperação ou pelos amigos de confiança.
    ApagarChaves,
}

/// Um gatilho ligado, com a reação escolhida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gatilho {
    PalavraMagica(Reacao),
    ToqueDePanico(Reacao),
    LongeDeCasa(Reacao),
    MovimentoBrusco(Reacao),
    Coacao(Reacao),
    SemMeMexer { segundos: u64, reacao: Reacao },
    /// Sinal único da ponte nativa (ver `integridade`), combinando depurador anexado, ganchos de
    /// instrumentação, depuração USB ligada ou guinada para root/jailbreak durante a sessão em curso.
    AtaqueDetectado(Reacao),
    /// Comando assinado por outro aparelho/computador já confiado na conta (`comando_remoto.rs`), validado
    /// ANTES de chegar aqui. Sem `Reacao` embutida de propósito: ao contrário dos outros gatilhos (a reação
    /// é decidida uma vez, nos ajustes), aqui quem decide é o dono, a cada emissão — "só travar" e "apagar
    /// tudo" são escolhas diferentes que ele faz no momento de mandar o comando, não de configurar o aparelho.
    /// Ligar este gatilho (incluí-lo em `Config.gatilhos`) é o que torna o aparelho RECEPTIVO ao comando —
    /// quem não quiser que outro aparelho da própria conta possa travar/apagar este remotamente pode deixar
    /// de fora, mesmo já com os outros gatilhos ligados.
    ComandoRemoto,
}

impl Gatilho {
    /// Reação fixa deste gatilho, se houver uma — `None` só para `ComandoRemoto`, cuja reação vem do
    /// sinal (`Sinais.comando_remoto`), nunca da configuração. `decidir`, abaixo, trata os dois casos.
    pub fn reacao_fixa(&self) -> Option<Reacao> {
        match *self {
            Gatilho::PalavraMagica(r) | Gatilho::ToqueDePanico(r) | Gatilho::LongeDeCasa(r)
            | Gatilho::MovimentoBrusco(r) | Gatilho::Coacao(r) | Gatilho::AtaqueDetectado(r) => Some(r),
            Gatilho::SemMeMexer { reacao, .. } => Some(reacao),
            Gatilho::ComandoRemoto => None,
        }
    }
}

/// Config de pânico do usuário: os gatilhos ligados.
#[derive(Clone, Debug, Default)]
pub struct Config { pub gatilhos: Vec<Gatilho> }

/// Sinais que o app (camada Swift/Kotlin) lê dos sensores e passa para cá. Nada aqui é enviado a servidor.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sinais {
    /// O que foi digitado no lugar do PIN bate com a palavra mágica.
    pub palavra_magica_digitada: bool,
    /// O padrão do botão de pânico foi feito.
    pub toque_de_panico_feito: bool,
    /// O aparelho está fora de todas as regiões seguras.
    pub fora_de_casa: bool,
    /// A aceleração passou do limite de "tranco" (arranque brusco).
    pub tranco_detectado: bool,
    /// O PIN de coação foi digitado.
    pub coacao_digitada: bool,
    /// Segundos desde o último toque na tela.
    pub segundos_sem_toque: u64,
    /// A ponte nativa viu depurador anexado, gancho de instrumentação, depuração USB ligada ou guinada
    /// para root/jailbreak enquanto o app já estava aberto e autenticado (ver `integridade::Sinais`).
    pub ataque_detectado: bool,
    /// Comando remoto já validado (`comando_remoto::validar` devolveu `Ok`) nesta rodada de avaliação —
    /// a reação que ELE pede, não uma configurada de antemão. `None` = nenhum comando chegou agora. Um
    /// comando que falhou a validação (assinatura errada, aparelho errado, contador repetido) nunca chega
    /// a aparecer aqui — `panico.rs` nunca vê um comando bruto, só o resultado já autenticado.
    pub comando_remoto: Option<Reacao>,
}

/// Decide a reação mais forte entre todos os gatilhos disparados. `None` = nada dispara.
/// Regra: quando vários disparam, vale a reação MAIS forte (a ordem de `Reacao` garante isso) — inclusive
/// quando um comando remoto chega ao mesmo tempo que um gatilho local (ex.: o próprio ladrão tentando o
/// PIN enquanto o dono já mandou o comando de outro aparelho): vale a reação mais forte entre as duas,
/// nunca uma substituindo silenciosamente a outra.
pub fn decidir(cfg: &Config, s: &Sinais) -> Option<Reacao> {
    cfg.gatilhos.iter().filter_map(|&g| reacao_se_disparou(g, s)).max()
}

fn reacao_se_disparou(g: Gatilho, s: &Sinais) -> Option<Reacao> {
    match g {
        Gatilho::ComandoRemoto => s.comando_remoto,
        _ if disparou(g, s) => g.reacao_fixa(),
        _ => None,
    }
}

fn disparou(g: Gatilho, s: &Sinais) -> bool {
    match g {
        Gatilho::PalavraMagica(_) => s.palavra_magica_digitada,
        Gatilho::ToqueDePanico(_) => s.toque_de_panico_feito,
        Gatilho::LongeDeCasa(_) => s.fora_de_casa,
        Gatilho::MovimentoBrusco(_) => s.tranco_detectado,
        Gatilho::Coacao(_) => s.coacao_digitada,
        Gatilho::SemMeMexer { segundos, .. } => s.segundos_sem_toque >= segundos,
        Gatilho::AtaqueDetectado(_) => s.ataque_detectado,
        Gatilho::ComandoRemoto => false, // tratado à parte em reacao_se_disparou, nunca por aqui
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn palavra_magica_tranca_na_hora() {
        let cfg = Config { gatilhos: vec![Gatilho::PalavraMagica(Reacao::EsconderSensiveis)] };
        let s = Sinais { palavra_magica_digitada: true, ..Default::default() };
        assert_eq!(decidir(&cfg, &s), Some(Reacao::EsconderSensiveis));
        assert_eq!(decidir(&cfg, &Sinais::default()), None);
    }

    #[test]
    fn longe_de_casa_fecha_funcoes_escolhidas() {
        let cfg = Config { gatilhos: vec![Gatilho::LongeDeCasa(Reacao::EsconderSensiveis)] };
        assert_eq!(decidir(&cfg, &Sinais { fora_de_casa: true, ..Default::default() }), Some(Reacao::EsconderSensiveis));
        assert_eq!(decidir(&cfg, &Sinais { fora_de_casa: false, ..Default::default() }), None);
    }

    #[test]
    fn movimento_brusco_tranca() {
        let cfg = Config { gatilhos: vec![Gatilho::MovimentoBrusco(Reacao::Trancar)] };
        assert_eq!(decidir(&cfg, &Sinais { tranco_detectado: true, ..Default::default() }), Some(Reacao::Trancar));
    }

    #[test]
    fn sem_me_mexer_tranca_depois_do_tempo() {
        let cfg = Config { gatilhos: vec![Gatilho::SemMeMexer { segundos: 60, reacao: Reacao::Trancar }] };
        assert_eq!(decidir(&cfg, &Sinais { segundos_sem_toque: 59, ..Default::default() }), None);
        assert_eq!(decidir(&cfg, &Sinais { segundos_sem_toque: 60, ..Default::default() }), Some(Reacao::Trancar));
    }

    #[test]
    fn quando_varios_disparam_vale_a_reacao_mais_forte() {
        let cfg = Config { gatilhos: vec![
            Gatilho::MovimentoBrusco(Reacao::Trancar),
            Gatilho::Coacao(Reacao::ApagarChaves),
            Gatilho::LongeDeCasa(Reacao::EsconderSensiveis),
        ] };
        let s = Sinais { tranco_detectado: true, coacao_digitada: true, fora_de_casa: true, ..Default::default() };
        assert_eq!(decidir(&cfg, &s), Some(Reacao::ApagarChaves), "apagar é a mais forte");
    }

    #[test]
    fn ataque_detectado_tranca_mesmo_sem_nenhum_outro_gatilho() {
        let cfg = Config { gatilhos: vec![Gatilho::AtaqueDetectado(Reacao::ApagarChaves)] };
        assert_eq!(decidir(&cfg, &Sinais { ataque_detectado: true, ..Default::default() }), Some(Reacao::ApagarChaves));
        assert_eq!(decidir(&cfg, &Sinais::default()), None, "sem o sinal, nada dispara");
    }

    #[test]
    fn ataque_detectado_e_so_mais_um_gatilho_entre_os_outros() {
        // Não é tratado como mais "especial": entra na mesma disputa pela reação mais forte que os demais.
        let cfg = Config { gatilhos: vec![Gatilho::AtaqueDetectado(Reacao::Trancar), Gatilho::Coacao(Reacao::ApagarChaves)] };
        let s = Sinais { ataque_detectado: true, coacao_digitada: true, ..Default::default() };
        assert_eq!(decidir(&cfg, &s), Some(Reacao::ApagarChaves));
    }

    #[test]
    fn sem_gatilhos_nada_dispara() {
        assert_eq!(decidir(&Config::default(), &Sinais { coacao_digitada: true, tranco_detectado: true, ..Default::default() }), None);
    }

    #[test]
    fn ordem_das_reacoes_do_mais_leve_ao_mais_forte() {
        assert!(Reacao::Trancar < Reacao::EsconderSensiveis);
        assert!(Reacao::EsconderSensiveis < Reacao::MostrarIsca);
        assert!(Reacao::MostrarIsca < Reacao::ApagarChaves);
    }

    // ---- Comando Remoto (novo, 10/10/2026) ----

    #[test]
    fn comando_remoto_dispara_a_reacao_que_ele_proprio_carrega() {
        let cfg = Config { gatilhos: vec![Gatilho::ComandoRemoto] };
        let s = Sinais { comando_remoto: Some(Reacao::ApagarChaves), ..Default::default() };
        assert_eq!(decidir(&cfg, &s), Some(Reacao::ApagarChaves));
        let s2 = Sinais { comando_remoto: Some(Reacao::Trancar), ..Default::default() };
        assert_eq!(decidir(&cfg, &s2), Some(Reacao::Trancar), "o dono escolhe a reação ao emitir, não na configuração");
    }

    #[test]
    fn sem_comando_nenhum_o_gatilho_fica_quieto() {
        let cfg = Config { gatilhos: vec![Gatilho::ComandoRemoto] };
        assert_eq!(decidir(&cfg, &Sinais::default()), None);
    }

    #[test]
    fn gatilho_desligado_ignora_comando_mesmo_que_tenha_chegado_validado() {
        // Aparelho que não optou por aceitar comando remoto (Config sem o gatilho): mesmo que um comando
        // JÁ validado chegue (campo preenchido), este aparelho não reage — é a forma de o dono desligar
        // a receptividade sem mexer em nada além dos próprios ajustes.
        let cfg = Config { gatilhos: vec![Gatilho::Coacao(Reacao::MostrarIsca)] };
        let s = Sinais { comando_remoto: Some(Reacao::ApagarChaves), ..Default::default() };
        assert_eq!(decidir(&cfg, &s), None);
    }

    #[test]
    fn comando_remoto_disputa_a_reacao_mais_forte_com_gatilhos_locais_como_qualquer_outro() {
        let cfg = Config { gatilhos: vec![Gatilho::ComandoRemoto, Gatilho::MovimentoBrusco(Reacao::Trancar)] };
        let s = Sinais { comando_remoto: Some(Reacao::EsconderSensiveis), tranco_detectado: true, ..Default::default() };
        assert_eq!(decidir(&cfg, &s), Some(Reacao::EsconderSensiveis), "comando (mais forte) vence o gatilho local");
        let s2 = Sinais { comando_remoto: Some(Reacao::Trancar), tranco_detectado: true, ..Default::default() };
        // Empate de força: continua valendo a mesma reação, não importa de qual gatilho veio.
        assert_eq!(decidir(&cfg, &s2), Some(Reacao::Trancar));
    }
}
