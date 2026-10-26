//! # Módulo `checkup` — checkup de privacidade (PRIV1–PRIV9)
//!
//! Não inventa proteção nova: lê o que já está configurado em outros módulos (`coacao`, `recuperacao`,
//! `backup_frio` conceitual, `contatos`, `voz`, `conversa`) e diz, de forma objetiva e sem alarme, o que
//! falta configurar — nunca o que falta "construir" (isso é tarefa do roadmap, não do checkup).
//!
//! Cada item é independente: travar um não destrava os outros, e a ordem da lista é só de severidade
//! (crítico primeiro), nunca de obrigatoriedade — a pessoa pode ignorar qualquer um.
//!
//! ## Disco cifrado no computador vinculado (09/10/2026)
//!
//! `ComputadorSemDiscoCifrado` existe por causa de um limite já documentado em `memoria.rs`: no
//! computador (não no celular), hibernação do sistema grava a RAM inteira em disco, e o `mlock` do
//! módulo `memoria` não impede isso. Se o disco do sistema não estiver cifrado (FileVault/BitLocker), uma
//! chave momentaneamente em uso — inclusive uma chave de janela do `historico`, que o cofre com prazo
//! (ver `cofre.rs`) agora usa — pode sobreviver num arquivo de hibernação fora do controle do app. Isto
//! só é crítico quando **há** computador vinculado (PC1): aparelho recém-instalado, sem computador, não
//! tem nada a avisar aqui, e isso é correto, não uma lacuna.
use crate::coacao::Politica as PoliticaDePin;

/// Tudo que o checkup precisa saber para avaliar — um retrato do que está configurado agora, montado pelo
/// app a partir do que já guarda localmente. Nenhum campo aqui é lido do servidor: o checkup é sempre local.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub pin_configurado: bool,
    pub pin_de_coacao_configurado: bool,
    pub politica_de_tentativas: Option<PoliticaDePin>,
    pub guardioes_de_recuperacao: u32,
    pub backup_frio_cifrado_ativo: bool,
    pub nomes_proibidos_configurados: u32,
    pub disfarce_de_voz_ligado_em_alguma_conversa: bool,
    pub prazo_padrao_do_nome_na_lista_em_dias: u32,
    pub notificacao_de_aparelho_novo_ativa: bool,
    pub computador_vinculado: bool,
    pub disco_do_computador_cifrado: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severidade { Critico, Recomendado, Informativo }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    SemPin,
    SemPinDeCoacao,
    PoliticaDeTentativasAusente,
    PoucosGuardioesDeRecuperacao,
    SemBackupFrioCifrado,
    SemNomesProibidosConfigurados,
    PrazoDoNomeNaListaLongoDemais,
    SemNotificacaoDeAparelhoNovo,
    ComputadorSemDiscoCifrado,
}

impl Item {
    pub fn severidade(self) -> Severidade {
        use Item::*;
        match self {
            SemPin | PoliticaDeTentativasAusente | ComputadorSemDiscoCifrado => Severidade::Critico,
            SemPinDeCoacao | PoucosGuardioesDeRecuperacao | SemNotificacaoDeAparelhoNovo => Severidade::Recomendado,
            SemBackupFrioCifrado | SemNomesProibidosConfigurados | PrazoDoNomeNaListaLongoDemais => Severidade::Informativo,
        }
    }
}

/// Mínimo recomendado de guardiões de recuperação para não depender de uma pessoa só.
pub const MINIMO_RECOMENDADO_DE_GUARDIOES: u32 = 3;

/// Avalia a configuração atual e devolve o que falta, mais severo primeiro. Lista vazia = nada a avisar
/// (não é "100% seguro": é "nada que este checkup saiba cobrar está faltando").
pub fn avaliar(c: &Config) -> Vec<Item> {
    let mut itens = Vec::new();
    if !c.pin_configurado { itens.push(Item::SemPin); }
    match &c.politica_de_tentativas {
        None => itens.push(Item::PoliticaDeTentativasAusente),
        Some(_pol) => {}
    }
    if !c.pin_de_coacao_configurado { itens.push(Item::SemPinDeCoacao); }
    if c.guardioes_de_recuperacao < MINIMO_RECOMENDADO_DE_GUARDIOES { itens.push(Item::PoucosGuardioesDeRecuperacao); }
    if !c.notificacao_de_aparelho_novo_ativa { itens.push(Item::SemNotificacaoDeAparelhoNovo); }
    if !c.backup_frio_cifrado_ativo { itens.push(Item::SemBackupFrioCifrado); }
    if c.nomes_proibidos_configurados == 0 { itens.push(Item::SemNomesProibidosConfigurados); }
    if c.prazo_padrao_do_nome_na_lista_em_dias > 30 { itens.push(Item::PrazoDoNomeNaListaLongoDemais); }
    if c.computador_vinculado && !c.disco_do_computador_cifrado { itens.push(Item::ComputadorSemDiscoCifrado); }
    itens.sort_by_key(|i| i.severidade());
    itens
}

impl Config {
    /// Tudo zerado: o pior caso, usado em teste e como estado de um aparelho recém-instalado.
    pub fn vazia() -> Self {
        Config {
            pin_configurado: false, pin_de_coacao_configurado: false, politica_de_tentativas: None,
            guardioes_de_recuperacao: 0, backup_frio_cifrado_ativo: false, nomes_proibidos_configurados: 0,
            disfarce_de_voz_ligado_em_alguma_conversa: false, prazo_padrao_do_nome_na_lista_em_dias: 7,
            notificacao_de_aparelho_novo_ativa: false, computador_vinculado: false, disco_do_computador_cifrado: false,
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn config_vazia_acusa_todos_os_itens_criticos_primeiro() {
        let itens = avaliar(&Config::vazia());
        assert!(!itens.is_empty());
        assert_eq!(itens[0].severidade(), Severidade::Critico, "o primeiro item sempre é o mais severo");
        assert!(itens.windows(2).all(|w| w[0].severidade() <= w[1].severidade()), "ordem nunca piora");
    }

    #[test]
    fn pin_configurado_tira_o_item_critico_correspondente() {
        let mut c = Config::vazia();
        c.pin_configurado = true;
        c.politica_de_tentativas = Some(PoliticaDePin::equilibrada());
        assert!(!avaliar(&c).contains(&Item::SemPin));
        assert!(!avaliar(&c).contains(&Item::PoliticaDeTentativasAusente));
    }

    #[test]
    fn tres_ou_mais_guardioes_tira_o_aviso_de_recuperacao() {
        let mut c = Config::vazia();
        c.guardioes_de_recuperacao = MINIMO_RECOMENDADO_DE_GUARDIOES;
        assert!(!avaliar(&c).contains(&Item::PoucosGuardioesDeRecuperacao));
        c.guardioes_de_recuperacao = MINIMO_RECOMENDADO_DE_GUARDIOES - 1;
        assert!(avaliar(&c).contains(&Item::PoucosGuardioesDeRecuperacao));
    }

    #[test]
    fn configuracao_completa_nao_acusa_nada() {
        let c = Config {
            pin_configurado: true, pin_de_coacao_configurado: true, politica_de_tentativas: Some(PoliticaDePin::equilibrada()),
            guardioes_de_recuperacao: 3, backup_frio_cifrado_ativo: true, nomes_proibidos_configurados: 3,
            disfarce_de_voz_ligado_em_alguma_conversa: false, prazo_padrao_do_nome_na_lista_em_dias: 7,
            notificacao_de_aparelho_novo_ativa: true, computador_vinculado: true, disco_do_computador_cifrado: true,
        };
        assert!(avaliar(&c).is_empty());
    }

    #[test]
    fn computador_so_acusa_disco_sem_cifra_quando_ha_computador_vinculado() {
        let mut c = Config::vazia();
        c.pin_configurado = true;
        c.politica_de_tentativas = Some(PoliticaDePin::equilibrada());
        // Sem computador vinculado: mesmo com disco_do_computador_cifrado em false, nada a avisar.
        assert!(!avaliar(&c).contains(&Item::ComputadorSemDiscoCifrado), "sem computador vinculado não há o que checar");
        c.computador_vinculado = true;
        assert!(avaliar(&c).contains(&Item::ComputadorSemDiscoCifrado), "computador vinculado sem disco cifrado é crítico");
        assert_eq!(Item::ComputadorSemDiscoCifrado.severidade(), Severidade::Critico);
        c.disco_do_computador_cifrado = true;
        assert!(!avaliar(&c).contains(&Item::ComputadorSemDiscoCifrado), "disco cifrado resolve o item");
    }

    #[test]
    fn prazo_do_nome_na_lista_muito_longo_e_so_informativo_nunca_critico() {
        let mut c = Config::vazia();
        c.pin_configurado = true;
        c.politica_de_tentativas = Some(PoliticaDePin::equilibrada());
        c.prazo_padrao_do_nome_na_lista_em_dias = 90;
        let itens = avaliar(&c);
        assert!(itens.contains(&Item::PrazoDoNomeNaListaLongoDemais));
        assert_eq!(Item::PrazoDoNomeNaListaLongoDemais.severidade(), Severidade::Informativo);
    }
}
