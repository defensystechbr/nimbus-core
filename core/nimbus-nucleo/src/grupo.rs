//! # Módulo `grupo` — conversa em grupo: só a camada de membros, papéis e regras (v37, quinta rodada)
//!
//! **O que este módulo NÃO é**: o protocolo de mensagens em grupo (quem cifra o quê, para quem, com que
//! chave) não é deste núcleo — é o MLS (RFC 9420), como o próprio `lib.rs` já declara desde antes desta
//! rodada ("As conversas 1:1 e em grupo do NimbusChat não usam este núcleo para o protocolo de mensagens:
//! usam a libsignal e o MLS"). Reimplementar um protocolo de acordo de chaves em grupo aqui, numa única
//! rodada, seria o oposto de "menor superfície de ataque": o MLS já existe, é auditado publicamente, tem
//! anos de análise formal (prova de segurança do TreeKEM) — e uma versão caseira, não revisada por
//! ninguém além de mim, seria estritamente mais arriscada que usar a biblioteca certa (ex. OpenMLS) na
//! ponte nativa. Este módulo cobre só o que é mesmo trabalho de política local: QUEM está no grupo agora,
//! QUEM administra, e QUAL É a regra efetiva da conversa.
//!
//! **Administração (v37, quinta rodada — pedido explícito: "quem fez o grupo é como um administrador no
//! WhatsApp").** Múltiplos admins, não só o criador — o criador continua sendo o único papel permanente
//! (nunca removido nem rebaixado por ninguém, nem por outro admin), exatamente para que um grupo nunca
//! fique sem ninguém com a palavra final numa disputa entre admins. Um admin comum pode: adicionar e
//! remover membros comuns, promover outro membro a admin, silenciar e dessilenciar, apagar mensagem de
//! qualquer um no grupo, e definir o teto de encaminhamento do grupo inteiro. Um admin comum NÃO pode
//! remover, rebaixar ou silenciar outro admin (só o criador pode) — isso fecha o golpe óbvio de um admin
//! recém-promovido expulsar os demais para ficar sozinho no controle.
use crate::conversa::{self, Preferencias, Regras};
use crate::liberacao::Nivel;
use std::collections::{HashMap, HashSet};

pub type IdParticipante = [u8; 32];
pub type IdGrupo = [u8; 16];

/// Tamanho máximo de um grupo. Não é um limite de segurança do núcleo propriamente — é o mesmo limite
/// prático que qualquer mensageiro com grupo pequeno e malha de chamada usa (ver `chamada_em_grupo`,
/// que reaproveita este mesmo número). Acima disso, a combinação de preferências e o escudo continuam
/// funcionando (ambos já são genéricos para N), mas a malha de chamada deixa de ser viável (ver o motivo
/// lá) — por isso o teto é o mesmo aqui, para o grupo nunca crescer além do que a chamada em grupo suporta.
pub const MAX_MEMBROS: usize = 8;

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeGrupo {
    GrupoCheio,
    JaEMembro,
    NaoEMembro,
    SemPermissao,
    GrupoVazio,
    /// O criador nunca pode ser removido, rebaixado ou silenciado — por ninguém, nem por outro admin.
    NaoPodeMexerNoCriador,
}

/// Um grupo: quem está dentro, quem administra, quem está silenciado, e as preferências de cada um
/// (a combinação delas é a regra efetiva — `conversa::combinar`, exatamente a mesma função já usada e
/// testada para 1:1, agora chamada aqui).
#[derive(Debug, Clone)]
pub struct Grupo {
    id: IdGrupo,
    /// Quem criou o grupo. Admin permanente: nunca removido nem rebaixado, nem por outro admin.
    criador: IdParticipante,
    membros: HashMap<IdParticipante, Preferencias>,
    /// Quem administra agora. O criador está sempre incluído aqui (invariante mantida por este módulo,
    /// nunca exposta a uma operação que poderia tirá-lo).
    admins: HashSet<IdParticipante>,
    /// Quem está impedido de enviar mensagem no grupo agora (continua podendo ler e continua membro).
    silenciados: HashSet<IdParticipante>,
    /// Teto de encaminhamento do grupo inteiro, definido por um admin — a mesma ideia da camada "padrão
    /// do contato" de `liberacao::resolver_nivel_de_encaminhamento`, só que para todos os membros de uma
    /// vez. `None` = sem teto extra do grupo (só o teto normal da conversa, de sempre, continua valendo).
    teto_de_encaminhamento: Option<Nivel>,
}

impl Grupo {
    /// Cria um grupo só com o criador dentro, já como admin. Grupo de 1 pessoa é válido (estado
    /// transitório antes de convidar o resto) — a regra efetiva, nesse caso, é só a preferência do criador.
    pub fn criar(id: IdGrupo, criador: IdParticipante, preferencias_do_criador: Preferencias) -> Self {
        let mut membros = HashMap::new();
        membros.insert(criador, preferencias_do_criador);
        let mut admins = HashSet::new();
        admins.insert(criador);
        Grupo { id, criador, membros, admins, silenciados: HashSet::new(), teto_de_encaminhamento: None }
    }

    pub fn id(&self) -> IdGrupo { self.id }
    pub fn criador(&self) -> IdParticipante { self.criador }
    pub fn tamanho(&self) -> usize { self.membros.len() }
    pub fn e_membro(&self, quem: &IdParticipante) -> bool { self.membros.contains_key(quem) }
    pub fn e_admin(&self, quem: &IdParticipante) -> bool { self.admins.contains(quem) }
    pub fn esta_silenciado(&self, quem: &IdParticipante) -> bool { self.silenciados.contains(quem) }
    pub fn admins(&self) -> impl Iterator<Item = &IdParticipante> { self.admins.iter() }

    fn exige_admin(&self, solicitante: &IdParticipante) -> Result<(), ErroDeGrupo> {
        if !self.e_admin(solicitante) { return Err(ErroDeGrupo::SemPermissao); }
        Ok(())
    }

    /// Qualquer ADMIN pode adicionar (antes era só o criador; agora, como pedido, qualquer admin tem
    /// esse poder, igual a um grupo de WhatsApp).
    pub fn adicionar_membro(&mut self, solicitante: &IdParticipante, novo: IdParticipante, preferencias: Preferencias) -> Result<(), ErroDeGrupo> {
        self.exige_admin(solicitante)?;
        if self.membros.len() >= MAX_MEMBROS { return Err(ErroDeGrupo::GrupoCheio); }
        if self.membros.contains_key(&novo) { return Err(ErroDeGrupo::JaEMembro); }
        self.membros.insert(novo, preferencias);
        Ok(())
    }

    /// Sair sozinho sempre vale, para qualquer um (sair do próprio grupo nunca pode depender da boa
    /// vontade de um admin). Remover OUTRO exige ser admin; ninguém remove o criador; um admin comum
    /// não remove outro admin — só o próprio criador remove outro admin.
    pub fn remover_membro(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo> {
        let e_a_propria_saida = *solicitante == alvo;
        if !e_a_propria_saida {
            self.exige_admin(solicitante)?;
            if alvo == self.criador { return Err(ErroDeGrupo::NaoPodeMexerNoCriador); }
            if self.admins.contains(&alvo) && *solicitante != self.criador { return Err(ErroDeGrupo::SemPermissao); }
        }
        if self.membros.remove(&alvo).is_none() { return Err(ErroDeGrupo::NaoEMembro); }
        self.admins.remove(&alvo);
        self.silenciados.remove(&alvo);
        Ok(())
    }

    /// Promove um membro existente a admin. Qualquer admin pode promover.
    pub fn promover_admin(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo> {
        self.exige_admin(solicitante)?;
        if !self.membros.contains_key(&alvo) { return Err(ErroDeGrupo::NaoEMembro); }
        self.admins.insert(alvo);
        Ok(())
    }

    /// Rebaixa um admin a membro comum. O criador nunca pode ser rebaixado. Um admin comum só rebaixa
    /// a si mesmo (abrir mão do próprio papel); só o criador rebaixa outro admin.
    pub fn rebaixar_admin(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo> {
        self.exige_admin(solicitante)?;
        if alvo == self.criador { return Err(ErroDeGrupo::NaoPodeMexerNoCriador); }
        if *solicitante != alvo && *solicitante != self.criador { return Err(ErroDeGrupo::SemPermissao); }
        self.admins.remove(&alvo);
        Ok(())
    }

    /// Impede o alvo de enviar mensagem no grupo (continua lendo e continua membro). Só admin silencia;
    /// ninguém silencia um admin (nem o criador) — evita um admin calar outro antes de agir sem oposição.
    pub fn silenciar(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo> {
        self.exige_admin(solicitante)?;
        if !self.membros.contains_key(&alvo) { return Err(ErroDeGrupo::NaoEMembro); }
        if self.admins.contains(&alvo) { return Err(ErroDeGrupo::SemPermissao); }
        self.silenciados.insert(alvo);
        Ok(())
    }

    pub fn dessilenciar(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo> {
        self.exige_admin(solicitante)?;
        self.silenciados.remove(&alvo);
        Ok(())
    }

    /// Pode enviar mensagem no grupo agora? Precisa ser membro e não estar silenciado.
    pub fn pode_enviar(&self, quem: &IdParticipante) -> bool {
        self.membros.contains_key(quem) && !self.silenciados.contains(quem)
    }

    /// Define (ou remove, com `None`) o teto de encaminhamento do grupo inteiro. Só admin define. A
    /// escolha de cada mensagem individual continua podendo apertar ainda mais este teto (nunca afrouxar
    /// — mesma regra de sempre de `liberacao::resolver_nivel_de_encaminhamento`).
    pub fn definir_teto_de_encaminhamento(&mut self, solicitante: &IdParticipante, teto: Option<Nivel>) -> Result<(), ErroDeGrupo> {
        self.exige_admin(solicitante)?;
        self.teto_de_encaminhamento = teto;
        Ok(())
    }

    pub fn teto_de_encaminhamento(&self) -> Option<Nivel> { self.teto_de_encaminhamento }

    /// Um membro atualiza as próprias preferências (nunca as de outro — cada um só mexe nas suas).
    pub fn atualizar_preferencias(&mut self, quem: &IdParticipante, novas: Preferencias) -> Result<(), ErroDeGrupo> {
        match self.membros.get_mut(quem) {
            Some(p) => { *p = novas; Ok(()) }
            None => Err(ErroDeGrupo::NaoEMembro),
        }
    }

    /// A regra efetiva do grupo agora: a combinação mais restritiva entre TODOS os membros atuais —
    /// mesma função, mesma garantia, que já existia para conversas 1:1. Recalculada toda vez que alguém
    /// entra, sai, ou muda a própria preferência.
    pub fn regras_efetivas(&self) -> Result<Regras, ErroDeGrupo> {
        if self.membros.is_empty() { return Err(ErroDeGrupo::GrupoVazio); }
        let prefs: Vec<Preferencias> = self.membros.values().copied().collect();
        Ok(conversa::combinar(&prefs))
    }

    pub fn membros(&self) -> impl Iterator<Item = &IdParticipante> { self.membros.keys() }

    /// Apagar a mensagem de outra pessoa dentro do grupo (ex.: conteúdo que viola as regras do grupo).
    /// Fora do caso de admin, vale exatamente a mesma regra de sempre (`conversa::aceitar_retirada`: só o
    /// próprio autor, e só se a conversa permite retirar já lida) — isto é uma ADIÇÃO sobre o que já
    /// existe, não uma mudança: `conversa::aceitar_retirada` continua igual para 1:1, que não tem e não
    /// deve ter noção de admin.
    pub fn aceitar_retirada_no_grupo(&self, regras: &Regras, autor_da_original: &IdParticipante, quem_pede: &IdParticipante, ja_lida: bool) -> bool {
        conversa::aceitar_retirada(regras, autor_da_original, quem_pede, ja_lida) || self.e_admin(quem_pede)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::politica::PrazoDeVisibilidade;

    const A: IdParticipante = [1; 32];
    const B: IdParticipante = [2; 32];
    const C: IdParticipante = [3; 32];
    const D: IdParticipante = [4; 32];
    const ID: IdGrupo = [9; 16];

    fn prefs_com_prazo(prazo: Option<u64>) -> Preferencias {
        Preferencias { mensagens_apos_enviar: prazo, ..Preferencias::padrao() }
    }

    #[test]
    fn grupo_de_tres_combina_a_mais_restritiva_entre_todos_os_membros_atuais() {
        let mut g = Grupo::criar(ID, A, prefs_com_prazo(Some(3_600)));
        g.adicionar_membro(&A, B, prefs_com_prazo(Some(600))).unwrap();
        g.adicionar_membro(&A, C, prefs_com_prazo(Some(7_200))).unwrap();
        assert_eq!(g.regras_efetivas().unwrap().mensagens_apos_enviar, Some(600), "o menor prazo dos três vence");
    }

    #[test]
    fn so_admin_adiciona_membro_mas_qualquer_admin_pode() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        assert_eq!(g.adicionar_membro(&B, C, Preferencias::padrao()), Err(ErroDeGrupo::SemPermissao), "B não é membro nem admin ainda");
        assert!(g.adicionar_membro(&A, B, Preferencias::padrao()).is_ok());
        assert_eq!(g.adicionar_membro(&B, C, Preferencias::padrao()), Err(ErroDeGrupo::SemPermissao), "B é membro comum, não admin");
        g.promover_admin(&A, B).unwrap();
        assert!(g.adicionar_membro(&B, C, Preferencias::padrao()).is_ok(), "B agora é admin e pode adicionar");
    }

    #[test]
    fn nao_adiciona_o_mesmo_membro_duas_vezes_nem_alem_do_teto() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        assert_eq!(g.adicionar_membro(&A, B, Preferencias::padrao()), Err(ErroDeGrupo::JaEMembro));
        for i in 0..(MAX_MEMBROS - 2) as u8 { g.adicionar_membro(&A, [i + 50; 32], Preferencias::padrao()).unwrap(); }
        assert_eq!(g.tamanho(), MAX_MEMBROS);
        assert_eq!(g.adicionar_membro(&A, [99; 32], Preferencias::padrao()), Err(ErroDeGrupo::GrupoCheio));
    }

    #[test]
    fn qualquer_membro_sai_sozinho_sem_pedir_permissao() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        assert!(g.remover_membro(&B, B).is_ok(), "B sai sozinho, sem precisar de A");
        assert!(!g.e_membro(&B));
    }

    #[test]
    fn membro_comum_nao_remove_outro_so_admin_remove() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        g.adicionar_membro(&A, C, Preferencias::padrao()).unwrap();
        assert_eq!(g.remover_membro(&B, C), Err(ErroDeGrupo::SemPermissao), "B é membro comum, não remove C");
        assert!(g.remover_membro(&A, C).is_ok(), "o criador (admin) pode remover C");
    }

    #[test]
    fn ninguem_remove_rebaixa_ou_silencia_o_criador() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        g.promover_admin(&A, B).unwrap();
        assert_eq!(g.remover_membro(&B, A), Err(ErroDeGrupo::NaoPodeMexerNoCriador), "outro admin não remove o criador");
        assert_eq!(g.rebaixar_admin(&B, A), Err(ErroDeGrupo::NaoPodeMexerNoCriador), "outro admin não rebaixa o criador");
        assert_eq!(g.silenciar(&B, A), Err(ErroDeGrupo::SemPermissao), "ninguém silencia um admin, nem o criador");
    }

    #[test]
    fn admin_comum_nao_remove_nem_rebaixa_nem_silencia_outro_admin() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        g.adicionar_membro(&A, C, Preferencias::padrao()).unwrap();
        g.promover_admin(&A, B).unwrap();
        g.promover_admin(&A, C).unwrap();
        assert_eq!(g.remover_membro(&B, C), Err(ErroDeGrupo::SemPermissao), "B não remove outro admin C");
        assert_eq!(g.rebaixar_admin(&B, C), Err(ErroDeGrupo::SemPermissao), "B não rebaixa outro admin C");
        assert!(g.rebaixar_admin(&C, C).is_ok(), "C pode abrir mão do próprio papel de admin");
        assert!(g.remover_membro(&B, C).is_ok(), "C já não é mais admin, B (admin) pode removê-lo agora");
    }

    #[test]
    fn silenciado_nao_pode_enviar_mas_continua_membro() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        assert!(g.pode_enviar(&B));
        g.silenciar(&A, B).unwrap();
        assert!(!g.pode_enviar(&B), "B silenciado não pode mais enviar");
        assert!(g.e_membro(&B), "mas continua membro (lê normalmente)");
        g.dessilenciar(&A, B).unwrap();
        assert!(g.pode_enviar(&B));
    }

    #[test]
    fn so_admin_silencia_e_so_admin_define_teto_de_encaminhamento() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        g.adicionar_membro(&A, C, Preferencias::padrao()).unwrap();
        assert_eq!(g.silenciar(&B, C), Err(ErroDeGrupo::SemPermissao), "B não é admin");
        assert_eq!(g.definir_teto_de_encaminhamento(&B, Some(Nivel::GuardarNoCofre)), Err(ErroDeGrupo::SemPermissao));
        assert!(g.definir_teto_de_encaminhamento(&A, Some(Nivel::GuardarNoCofre)).is_ok());
        assert_eq!(g.teto_de_encaminhamento(), Some(Nivel::GuardarNoCofre));
    }

    #[test]
    fn admin_apaga_mensagem_de_qualquer_um_membro_comum_so_a_propria() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap(); // B: membro comum (autor da mensagem)
        g.adicionar_membro(&A, D, Preferencias::padrao()).unwrap(); // D: membro comum (não é autor, não é admin)
        let regras = Regras { retirar_lidas_a_pedido: false, ..Regras::padrao() };
        // D (membro comum, não autor, não admin) não pode apagar a mensagem de B já lida.
        assert!(!conversa::aceitar_retirada(&regras, &B, &D, true), "D não é o autor e a conversa não permite retirar já lida");
        assert!(!g.aceitar_retirada_no_grupo(&regras, &B, &D, true), "D (membro comum) não pode apagar mensagem de B já lida");
        // Mas A é admin (criador): pode apagar a mensagem de B mesmo já lida, mesmo com a regra desligada.
        assert!(g.aceitar_retirada_no_grupo(&regras, &B, &A, true), "A é admin, pode apagar qualquer mensagem no grupo");
        // B continua podendo apagar a própria mensagem antes de lida, igual a sempre (regra normal, sem admin).
        assert!(g.aceitar_retirada_no_grupo(&regras, &B, &B, false), "B apaga a própria mensagem antes de lida, regra normal");
    }

    #[test]
    fn preferencias_mudam_a_regra_efetiva_na_hora() {
        let mut g = Grupo::criar(ID, A, prefs_com_prazo(Some(3_600)));
        g.adicionar_membro(&A, B, prefs_com_prazo(Some(3_600))).unwrap();
        assert_eq!(g.regras_efetivas().unwrap().mensagens_apos_enviar, Some(3_600));
        g.atualizar_preferencias(&B, prefs_com_prazo(Some(60))).unwrap();
        assert_eq!(g.regras_efetivas().unwrap().mensagens_apos_enviar, Some(60), "B apertou, e já vale para o grupo inteiro");
    }

    #[test]
    fn grupo_vazio_nao_tem_regra_efetiva() {
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.remover_membro(&A, A).unwrap();
        assert_eq!(g.regras_efetivas(), Err(ErroDeGrupo::GrupoVazio));
    }

    #[test]
    fn prazo_do_nome_na_lista_tambem_combina_em_grupo_de_tres() {
        // Confirma que a regra de ouro vale também para a dimensão nova (RV1) com 3 membros de verdade
        // dentro de um Grupo de verdade, não só numa chamada direta de combinar().
        let curto = Preferencias { prazo_do_nome_na_lista: PrazoDeVisibilidade::UmaHora, ..Preferencias::padrao() };
        let longo = Preferencias { prazo_do_nome_na_lista: PrazoDeVisibilidade::TresMeses, ..Preferencias::padrao() };
        let mut g = Grupo::criar(ID, A, longo);
        g.adicionar_membro(&A, B, longo).unwrap();
        g.adicionar_membro(&A, C, curto).unwrap();
        assert_eq!(g.regras_efetivas().unwrap().prazo_do_nome_na_lista, PrazoDeVisibilidade::UmaHora);
    }

    #[test]
    fn quarto_membro_so_entra_com_admin_e_respeita_o_teto_com_malha_de_chamada() {
        // D só entra se quem pede for admin — sanity adicional de que a checagem de admin cobre todo
        // membro novo, não só os dois primeiros dos outros testes.
        let mut g = Grupo::criar(ID, A, Preferencias::padrao());
        g.adicionar_membro(&A, B, Preferencias::padrao()).unwrap();
        g.adicionar_membro(&A, C, Preferencias::padrao()).unwrap();
        assert_eq!(g.adicionar_membro(&C, D, Preferencias::padrao()), Err(ErroDeGrupo::SemPermissao), "C é membro comum");
        g.promover_admin(&A, C).unwrap();
        assert!(g.adicionar_membro(&C, D, Preferencias::padrao()).is_ok(), "C agora é admin");
    }
}
