//! # Módulo `integridade` — verificação do aparelho ao abrir o app (DV2, DV3)
//!
//! Nenhum app consegue examinar outros apps no iOS ou no Android. O que é possível: coletar **sinais**
//! oficiais e locais, classificar o risco e **restringir o próprio NimbusOne**.
//! A decisão é calculada no aparelho e **reconfirmada no servidor** com a atestação.
//! A lista de apps do cliente NUNCA sai do aparelho: o servidor recebe só o nível e os motivos.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Só aplicativos instalados. Não existe cliente web (decisão do dono em 06/10/2026: "somente app"); uma página
/// web vem do servidor a cada acesso e nunca poderia ser tão confiável quanto um app assinado.
pub enum Plataforma { Ios, Android, Computador }

#[derive(Clone, Debug, Default)]
pub struct Sinais {
    /// Atestação do app (App Attest no iOS, Play Integrity no Android) válida para o app original.
    pub app_atestado: bool,
    /// Veredito de integridade do aparelho físico.
    pub aparelho_integro: bool,
    pub root_ou_jailbreak: bool,
    pub depurador_anexado: bool,
    /// Tela sendo gravada, espelhada ou transmitida.
    pub tela_sendo_capturada: bool,
    /// Janela sobreposta sobre o app (toque obscurecido no Android).
    pub sobreposicao_detectada: bool,
    /// Serviço de acessibilidade ativo fora da lista de confiáveis (vetor de controle remoto).
    pub acessibilidade_nao_confiavel: bool,
    /// Depuração USB (ADB) ligada.
    pub depuracao_usb: bool,
    /// Sinal opcional: apps capazes de capturar ou controlar a tela.
    pub risco_de_apps_com_acesso: Option<bool>,
    /// Sinal opcional: proteção do sistema desligada ou com alerta.
    pub protecao_do_sistema_ok: Option<bool>,
    pub sistema_desatualizado: bool,
    /// Perfil de configuração ou certificado de origem desconhecida instalado (iOS).
    pub perfil_desconhecido_instalado: bool,
    /// Loja alternativa ou carregamento lateral de apps ativado.
    pub instalacao_lateral_ativa: bool,
    /// Atestação de hardware (Android Key Attestation / App Attest): estado da inicialização verificada.
    pub inicio: Option<Inicio>,
    /// Onde a atestação foi feita. Só em software pode ser forjada.
    pub origem_da_atestacao: Option<OrigemDaAtestacao>,
    /// Certificado de atestação na lista de revogados (chaves vazadas usadas para falsificar).
    pub atestacao_revogada: bool,
    /// Meses desde a última atualização de segurança do sistema.
    pub meses_sem_atualizacao: Option<u32>,
    /// Aparelho sem senha, PIN ou padrão de bloqueio: o chip não protege as chaves.
    pub sem_bloqueio_de_tela: bool,
    /// Ferramentas de interceptação dentro do app (Frida, Xposed, Substrate e semelhantes).
    pub ganchos_detectados: bool,
    pub emulador: bool,
    /// O sistema esconde o conteúdo marcado como sensível de serviços de acessibilidade que não são
    /// ferramentas de acessibilidade declaradas (Android 14+). Leitores de tela oficiais continuam funcionando.
    pub acessibilidade_contida_pelo_sistema: bool,
    /// Computador: disco do sistema cifrado (FileVault no Mac, BitLocker no Windows). A hibernação grava a RAM
    /// em disco; sem disco cifrado, o conteúdo aberto pode ir parar em claro no disco (RZ0c, RZ12).
    pub disco_cifrado: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inicio {
    /// Sistema original do fabricante, bootloader travado.
    Verificado,
    /// Sistema alternativo com bootloader travado e chave de verificação na lista permitida (ex.: GrapheneOS).
    AlternativoPermitido,
    /// Sistema com chave desconhecida ou bootloader destravado.
    NaoVerificado,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrigemDaAtestacao { Software, AmbienteSeguro, ChipDedicado }

/// Meses sem atualização de segurança a partir dos quais o aparelho fica amarelo.
pub const MESES_TOLERADOS_SEM_ATUALIZACAO: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nivel { Verde, Amarelo, Vermelho }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Veredito { pub nivel: Nivel, pub motivos: Vec<&'static str> }

/// O que cada nível permite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissoes {
    pub abrir_conteudo: bool,
    pub album_cofre: bool,
    pub visualizacao_unica: bool,
    pub celular_virtual: bool,
    pub aplicativos_bancarios: bool,
    pub biometria_em_acao_sensivel: bool,
    /// Receber arquivo por proximidade (NimbusDrop lacrado, ND2–ND5). Só no verde.
    pub receber_por_proximidade: bool,
}

/// Plataformas em que o fabricante atesta o app e o aparelho (App Attest no iOS; atestação de chave e
/// Play Integrity no Android). No Mac, a Apple documenta que o App Attest não está disponível para apps comuns;
/// no Windows não há equivalente. Sem essa prova, o computador nunca chega ao verde.
pub fn tem_atestacao_do_fabricante(p: Plataforma) -> bool { matches!(p, Plataforma::Ios | Plataforma::Android) }

pub fn avaliar(p: Plataforma, s: &Sinais) -> Veredito {
    let mut vermelho = vec![];
    let mut amarelo = vec![];
    if tem_atestacao_do_fabricante(p) {
        if !s.app_atestado { vermelho.push("app não atestado (adulterado, emulador ou script)"); }
        if !s.aparelho_integro { vermelho.push("aparelho sem integridade atestada"); }
    } else {
        // Teto amarelo: sem Cofre, sem visualização única e sem NimbusDrop (PC1–PC6).
        amarelo.push("plataforma sem atestação do app pelo fabricante (teto amarelo)");
        if p == Plataforma::Computador {
            if !matches!(s.origem_da_atestacao, Some(OrigemDaAtestacao::ChipDedicado) | Some(OrigemDaAtestacao::AmbienteSeguro)) {
                vermelho.push("chave fora do chip de segurança do computador (TPM 2.0 ou Secure Enclave)");
            }
            if s.disco_cifrado != Some(true) { vermelho.push("disco do sistema sem cifra (FileVault ou BitLocker)"); }
        }
    }
    if s.root_ou_jailbreak { vermelho.push("root ou jailbreak"); }
    if s.depurador_anexado { vermelho.push("depurador anexado ao app"); }
    if s.acessibilidade_nao_confiavel {
        if s.acessibilidade_contida_pelo_sistema { amarelo.push("serviço de acessibilidade de terceiros (conteúdo escondido dele pelo sistema)"); }
        else { vermelho.push("serviço de acessibilidade não confiável (risco de controle remoto)"); }
    }
    if s.inicio == Some(Inicio::NaoVerificado) { vermelho.push("sistema modificado ou bootloader destravado"); }
    if s.origem_da_atestacao == Some(OrigemDaAtestacao::Software) { vermelho.push("atestação só em software (pode ser forjada)"); }
    if s.atestacao_revogada { vermelho.push("certificado de atestação revogado"); }
    if s.sem_bloqueio_de_tela { vermelho.push("aparelho sem senha de bloqueio"); }
    if s.ganchos_detectados { vermelho.push("ferramenta de interceptação no app"); }
    if s.emulador { vermelho.push("emulador"); }
    if s.meses_sem_atualizacao.is_some_and(|m| m > MESES_TOLERADOS_SEM_ATUALIZACAO) { amarelo.push("mais de 12 meses sem atualização de segurança"); }
    if s.sobreposicao_detectada { vermelho.push("sobreposição de tela"); }
    if s.perfil_desconhecido_instalado { vermelho.push("perfil ou certificado de origem desconhecida instalado"); }
    if s.tela_sendo_capturada { amarelo.push("tela sendo gravada ou espelhada"); }
    if s.depuracao_usb { amarelo.push("depuração USB ligada"); }
    if s.risco_de_apps_com_acesso == Some(true) { amarelo.push("apps capazes de capturar ou controlar a tela"); }
    if s.protecao_do_sistema_ok == Some(false) { amarelo.push("proteção do sistema desligada ou com alerta"); }
    if s.sistema_desatualizado { amarelo.push("sistema desatualizado"); }
    if s.instalacao_lateral_ativa { amarelo.push("instalação de apps fora da loja oficial ativada"); }
    if !vermelho.is_empty() { vermelho.extend(amarelo); return Veredito { nivel: Nivel::Vermelho, motivos: vermelho }; }
    if !amarelo.is_empty() { return Veredito { nivel: Nivel::Amarelo, motivos: amarelo }; }
    Veredito { nivel: Nivel::Verde, motivos: vec![] }
}

pub fn permissoes(nivel: Nivel) -> Permissoes {
    match nivel {
        Nivel::Verde => Permissoes { abrir_conteudo: true, album_cofre: true, visualizacao_unica: true, celular_virtual: true, aplicativos_bancarios: true, biometria_em_acao_sensivel: false, receber_por_proximidade: true },
        Nivel::Amarelo => Permissoes { abrir_conteudo: true, album_cofre: false, visualizacao_unica: false, celular_virtual: true, aplicativos_bancarios: false, biometria_em_acao_sensivel: true, receber_por_proximidade: false },
        // Vermelho: só sair, revogar aparelhos e ver orientações.
        Nivel::Vermelho => Permissoes { abrir_conteudo: false, album_cofre: false, visualizacao_unica: false, celular_virtual: false, aplicativos_bancarios: false, biometria_em_acao_sensivel: true, receber_por_proximidade: false },
    }
}

/// Sessão no servidor: aparelho vermelho não recebe credencial — não conversa, não baixa, não sincroniza.
pub fn sessao_permitida(v: &Veredito) -> bool { v.nivel != Nivel::Vermelho }

/// Um veredito vale por pouco tempo: o app reavalia ao voltar para a tela e a cada 10 minutos.
pub const VALIDADE_DO_VEREDITO: u64 = 600;
pub fn precisa_reavaliar(avaliado_em: u64, agora: u64) -> bool { agora >= avaliado_em.saturating_add(VALIDADE_DO_VEREDITO) }

/// Desafio de uso único do servidor: a atestação precisa conter este valor. Impede reenviar uma atestação
/// antiga de um aparelho limpo a partir de um aparelho comprometido.
#[derive(Debug, Default)]
pub struct Desafios { pendentes: std::collections::HashMap<[u8; 32], u64> }

/// Tempo para o aparelho responder ao desafio.
pub const VALIDADE_DO_DESAFIO: u64 = 60;

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeDesafio { Desconhecido, Vencido }

impl Desafios {
    pub fn emitir(&mut self, agora: u64) -> [u8; 32] {
        use rand_core::{OsRng, RngCore};
        let mut v = [0u8; 32];
        OsRng.fill_bytes(&mut v);
        let _ = self.pendentes.insert(v, agora);
        v
    }
    /// Consome o desafio: vale uma vez só e só dentro do prazo.
    pub fn consumir(&mut self, valor: &[u8; 32], agora: u64) -> Result<(), RecusaDeDesafio> {
        let emitido = self.pendentes.remove(valor).ok_or(RecusaDeDesafio::Desconhecido)?;
        if agora >= emitido.saturating_add(VALIDADE_DO_DESAFIO) { return Err(RecusaDeDesafio::Vencido); }
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    fn limpo() -> Sinais { Sinais { app_atestado: true, aparelho_integro: true, ..Default::default() } }

    fn computador_limpo() -> Sinais {
        Sinais { origem_da_atestacao: Some(OrigemDaAtestacao::ChipDedicado), disco_cifrado: Some(true), ..Default::default() }
    }

    #[test]
    fn computador_nunca_chega_ao_verde() {
        assert!(tem_atestacao_do_fabricante(Plataforma::Ios) && tem_atestacao_do_fabricante(Plataforma::Android));
        assert!(!tem_atestacao_do_fabricante(Plataforma::Computador));
        let v = avaliar(Plataforma::Computador, &computador_limpo());
        assert_eq!(v.nivel, Nivel::Amarelo);
        let p = permissoes(v.nivel);
        assert!(p.abrir_conteudo);
        assert!(!p.album_cofre && !p.visualizacao_unica && !p.receber_por_proximidade);
    }

    #[test]
    fn computador_com_atestacao_declarada_continua_amarelo() {
        // Mesmo que a ponte diga "app atestado", o computador não tem prova do fabricante.
        let s = Sinais { app_atestado: true, aparelho_integro: true, ..computador_limpo() };
        assert_eq!(avaliar(Plataforma::Computador, &s).nivel, Nivel::Amarelo);
    }

    #[test]
    fn computador_sem_chip_ou_sem_disco_cifrado_e_vermelho() {
        let sem_chip = Sinais { origem_da_atestacao: None, ..computador_limpo() };
        assert_eq!(avaliar(Plataforma::Computador, &sem_chip).nivel, Nivel::Vermelho);
        let chave_em_software = Sinais { origem_da_atestacao: Some(OrigemDaAtestacao::Software), ..computador_limpo() };
        assert_eq!(avaliar(Plataforma::Computador, &chave_em_software).nivel, Nivel::Vermelho);
        let sem_disco = Sinais { disco_cifrado: Some(false), ..computador_limpo() };
        assert_eq!(avaliar(Plataforma::Computador, &sem_disco).nivel, Nivel::Vermelho);
        let disco_desconhecido = Sinais { disco_cifrado: None, ..computador_limpo() };
        assert_eq!(avaliar(Plataforma::Computador, &disco_desconhecido).nivel, Nivel::Vermelho);
    }

    #[test]
    fn nimbusdrop_so_no_verde() {
        assert!(permissoes(Nivel::Verde).receber_por_proximidade);
        assert!(!permissoes(Nivel::Amarelo).receber_por_proximidade);
        assert!(!permissoes(Nivel::Vermelho).receber_por_proximidade);
    }

    #[test]
    fn celular_sem_atestacao_continua_vermelho() {
        assert_eq!(avaliar(Plataforma::Ios, &Sinais::default()).nivel, Nivel::Vermelho);
        assert_eq!(avaliar(Plataforma::Android, &Sinais::default()).nivel, Nivel::Vermelho);
    }

    #[test]
    fn aparelho_limpo_e_verde() {
        let v = avaliar(Plataforma::Ios, &limpo());
        assert_eq!(v.nivel, Nivel::Verde);
        assert!(permissoes(v.nivel).album_cofre);
    }

    #[test]
    fn gravacao_de_tela_e_amarelo_e_fecha_o_cofre() {
        let v = avaliar(Plataforma::Ios, &Sinais { tela_sendo_capturada: true, ..limpo() });
        assert_eq!(v.nivel, Nivel::Amarelo);
        let p = permissoes(v.nivel);
        assert!(!p.album_cofre && !p.visualizacao_unica && !p.aplicativos_bancarios && p.biometria_em_acao_sensivel);
    }

    #[test]
    fn acessibilidade_suspeita_e_vermelho() {
        let v = avaliar(Plataforma::Android, &Sinais { acessibilidade_nao_confiavel: true, ..limpo() });
        assert_eq!(v.nivel, Nivel::Vermelho);
        assert!(!permissoes(v.nivel).abrir_conteudo);
    }

    #[test]
    fn perfil_desconhecido_e_vermelho() {
        let v = avaliar(Plataforma::Ios, &Sinais { perfil_desconhecido_instalado: true, ..limpo() });
        assert_eq!(v.nivel, Nivel::Vermelho);
    }

    #[test]
    fn app_nao_atestado_e_vermelho() {
        assert_eq!(avaliar(Plataforma::Android, &Sinais { app_atestado: false, ..limpo() }).nivel, Nivel::Vermelho);
    }

    #[test]
    fn acessibilidade_de_terceiros_contida_pelo_sistema_so_amarela() {
        let v = avaliar(Plataforma::Android, &Sinais { acessibilidade_nao_confiavel: true, acessibilidade_contida_pelo_sistema: true, ..limpo() });
        assert_eq!(v.nivel, Nivel::Amarelo, "gerenciador de senhas não tira ninguém do app");
    }

    #[test]
    fn atestacao_de_hardware_ruim_e_vermelho() {
        for s in [Sinais { inicio: Some(Inicio::NaoVerificado), ..limpo() }, Sinais { origem_da_atestacao: Some(OrigemDaAtestacao::Software), ..limpo() },
                  Sinais { atestacao_revogada: true, ..limpo() }, Sinais { sem_bloqueio_de_tela: true, ..limpo() },
                  Sinais { ganchos_detectados: true, ..limpo() }, Sinais { emulador: true, ..limpo() }] {
            assert_eq!(avaliar(Plataforma::Android, &s).nivel, Nivel::Vermelho, "{s:?}");
        }
    }

    #[test]
    fn sistema_alternativo_travado_e_permitido_e_chip_dedicado_e_verde() {
        let s = Sinais { inicio: Some(Inicio::AlternativoPermitido), origem_da_atestacao: Some(OrigemDaAtestacao::ChipDedicado), ..limpo() };
        assert_eq!(avaliar(Plataforma::Android, &s).nivel, Nivel::Verde);
    }

    #[test]
    fn atualizacao_de_seguranca_atrasada_amarela_so_depois_de_12_meses() {
        assert_eq!(avaliar(Plataforma::Android, &Sinais { meses_sem_atualizacao: Some(12), ..limpo() }).nivel, Nivel::Verde);
        assert_eq!(avaliar(Plataforma::Android, &Sinais { meses_sem_atualizacao: Some(13), ..limpo() }).nivel, Nivel::Amarelo);
    }

    #[test]
    fn aparelho_vermelho_nao_recebe_sessao() {
        assert!(!sessao_permitida(&avaliar(Plataforma::Ios, &Sinais { root_ou_jailbreak: true, ..limpo() })));
        assert!(sessao_permitida(&avaliar(Plataforma::Ios, &Sinais { tela_sendo_capturada: true, ..limpo() })));
    }

    #[test]
    fn veredito_vence_em_10_minutos() {
        assert!(!precisa_reavaliar(1_000, 1_599) && precisa_reavaliar(1_000, 1_600));
    }

    #[test]
    fn desafio_vale_uma_vez_e_por_60_segundos() {
        let mut d = Desafios::default();
        let v = d.emitir(100);
        assert_eq!(d.consumir(&v, 150), Ok(()));
        assert_eq!(d.consumir(&v, 151), Err(RecusaDeDesafio::Desconhecido), "reenviar a mesma atestação não passa");
        let w = d.emitir(100);
        assert_eq!(d.consumir(&w, 160), Err(RecusaDeDesafio::Vencido));
    }

    #[test]
    fn vermelho_lista_todos_os_motivos() {
        let v = avaliar(Plataforma::Android, &Sinais { root_ou_jailbreak: true, depuracao_usb: true, ..limpo() });
        assert_eq!(v.motivos.len(), 2);
    }
}
