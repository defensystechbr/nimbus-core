//! # Módulo `corporativo` — plano corporativo: suspender sem ler, recuperar só com a chave mestra (COR1–COR24)
//!
//! Dois poderes separados de propósito:
//!
//! 1. **Suspender o acesso** — rápido, por um administrador, **sem ler nada**. A empresa publica uma
//!    **lista de contas autorizadas**, assinada pelos administradores dela (nunca pelo servidor nem pela
//!    NimbusOne). O aparelho só abre se a conta estiver na lista mais recente e a lista estiver dentro da
//!    validade (padrão: 15 minutos). Conta suspensa: o aparelho apaga as chaves. Aparelho desligado da
//!    internet: quando a lista vence, o app trava sozinho.
//! 2. **Recuperar o conteúdo** — só com a **chave mestra**, que está dividida entre guardiões (módulo
//!    `custodia`), e só de conta **já suspensa**: a empresa não consegue ler escondido quem ainda trabalha.
//!    Exige ordem assinada por `k` administradores **e** `k` guardiões. Os dados vão para outra conta; a
//!    identidade do ex-funcionário fica encerrada para sempre (ninguém fala “como se fosse ele”).
//!
//! Regras que o próprio aparelho confere em cada lista nova (não depende de confiar no servidor):
//! numeração sem saltos (sem voltar a uma lista antiga), validade curta, nenhuma conta some sem ficar
//! registrada como suspensa, reativação e suspensão em massa só com várias assinaturas.
use crate::aprovacao::{self, Aprovacao, Pedido, Resultado, TipoDePedido};
use crate::assinatura::{ChaveDeAssinatura, ChaveDeVerificacao};
use crate::envelope::{self, ChavePublica, ChaveSecreta};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use zeroize::Zeroizing;

pub type IdConta = [u8; 32];

/// Política da empresa (assinada junto com a lista de administradores).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Politica {
    /// Validade máxima de cada lista, em segundos. Offline além disso, o app trava.
    pub validade_da_lista: u64,
    pub minimo_para_reativar: u8,
    pub minimo_para_recuperar: u8,
    pub minimo_para_suspensao_em_massa: u8,
    pub minimo_para_mudar_administradores: u8,
    /// Quantas contas uma ordem assinada por um só administrador pode suspender de uma vez.
    pub limite_de_suspensao_simples: usize,
}

impl Politica {
    pub fn padrao() -> Self {
        Politica { validade_da_lista: 900, minimo_para_reativar: 2, minimo_para_recuperar: 2,
                   minimo_para_suspensao_em_massa: 2, minimo_para_mudar_administradores: 2, limite_de_suspensao_simples: 5 }
    }
}

#[derive(Debug)]
pub struct Empresa {
    pub id: [u8; 16],
    pub administradores: Vec<ChaveDeVerificacao>,
    pub politica: Politica,
    /// Parte pública da chave mestra; `None` = plano sem recuperação (suspender apaga para sempre).
    pub chave_mestra_publica: Option<ChavePublica>,
    pub assentos_contratados: usize,
}

/// Lista de contas autorizadas, assinada pelos administradores da empresa.
#[derive(Clone, Debug)]
pub struct Lista {
    pub empresa: [u8; 16],
    pub numero: u64,
    pub emitida_em: u64,
    pub valida_ate: u64,
    pub ativas: BTreeSet<IdConta>,
    /// Contas suspensas ficam registradas para sempre (salvo reativação com várias assinaturas).
    pub suspensas: BTreeSet<IdConta>,
    /// Quantas assinaturas os administradores declararam exigir para esta lista (faz parte do assinado).
    pub minimo_declarado: u8,
    pub assinaturas: Vec<Aprovacao>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Violacao {
    EmpresaDiferente, SaltoOuVolta, ContaSumiu, ContaAtivaESuspensa, ContaApagadaDaHistoria,
    AssinaturasInsuficientes, Vencida, ValidadeLongaDemais, SemAssentos,
    SemChaveMestra, ContaNaoSuspensa, DestinoInvalido, ListaDeAdministradoresVazia,
    /// Ordem de recuperação ainda na espera de 24 h (EO5): todos os administradores veem e qualquer um cancela.
    EmCarencia,
}

fn hash_da_lista(l: &Lista) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"NimbusOne/v1/lista-de-contas");
    h.update(l.empresa);
    h.update(l.numero.to_be_bytes());
    h.update(l.emitida_em.to_be_bytes());
    h.update(l.valida_ate.to_be_bytes());
    h.update((l.ativas.len() as u64).to_be_bytes());
    for c in &l.ativas { h.update(c); }
    h.update((l.suspensas.len() as u64).to_be_bytes());
    for c in &l.suspensas { h.update(c); }
    h.finalize().into()
}

fn pedido(empresa: [u8; 16], hash: [u8; 32], numero: u64, emitida_em: u64, valida_ate: u64, minimo: u8) -> Pedido {
    let mut id = [0u8; 16];
    id[..8].copy_from_slice(&empresa[..8]);
    id[8..].copy_from_slice(&numero.to_be_bytes());
    Pedido { tipo: TipoDePedido::Administracao, id, hash_do_conteudo: hash, valor_centavos: None, beneficiario: None,
             criado_em: emitida_em, expira_em: valida_ate, minimo_de_aprovacoes: minimo }
}

fn pedido_da_lista(l: &Lista) -> Pedido { pedido(l.empresa, hash_da_lista(l), l.numero, l.emitida_em, l.valida_ate, l.minimo_declarado) }

/// Administrador assina a lista (no aparelho dele, com chave física e biometria).
pub fn assinar_lista(chave: &ChaveDeAssinatura, l: &Lista) -> Aprovacao { aprovacao::aprovar(chave, &pedido_da_lista(l)) }

/// Quantas assinaturas a mudança exige.
pub fn exigido_para_transicao(p: &Politica, anterior: &Lista, nova: &Lista) -> Result<u8, Violacao> {
    if nova.empresa != anterior.empresa { return Err(Violacao::EmpresaDiferente); }
    if nova.numero != anterior.numero.saturating_add(1) { return Err(Violacao::SaltoOuVolta); }
    if nova.ativas.intersection(&nova.suspensas).next().is_some() { return Err(Violacao::ContaAtivaESuspensa); }
    // Nenhuma conta ativa some sem ficar registrada como suspensa.
    if anterior.ativas.iter().any(|c| !nova.ativas.contains(c) && !nova.suspensas.contains(c)) { return Err(Violacao::ContaSumiu); }
    // Conta suspensa só sai do registro se for reativada.
    if anterior.suspensas.iter().any(|c| !nova.suspensas.contains(c) && !nova.ativas.contains(c)) { return Err(Violacao::ContaApagadaDaHistoria); }
    let reativadas = anterior.suspensas.iter().filter(|c| nova.ativas.contains(*c)).count();
    let suspensas_agora = nova.suspensas.iter().filter(|c| !anterior.suspensas.contains(*c)).count();
    let mut exigido = 1u8;
    if reativadas > 0 { exigido = exigido.max(p.minimo_para_reativar); }
    if suspensas_agora > p.limite_de_suspensao_simples { exigido = exigido.max(p.minimo_para_suspensao_em_massa); }
    Ok(exigido)
}

/// Valida uma lista nova (servidor ao receber; aparelho ao baixar). `anterior = None` só na primeira lista.
pub fn validar_lista(e: &Empresa, anterior: Option<&Lista>, nova: &Lista, agora: u64) -> Result<(), Violacao> {
    if nova.empresa != e.id { return Err(Violacao::EmpresaDiferente); }
    if nova.ativas.intersection(&nova.suspensas).next().is_some() { return Err(Violacao::ContaAtivaESuspensa); }
    let exigido = match anterior { Some(a) => exigido_para_transicao(&e.politica, a, nova)?, None => 1 };
    if nova.minimo_declarado < exigido { return Err(Violacao::AssinaturasInsuficientes); }
    if nova.valida_ate > nova.emitida_em.saturating_add(e.politica.validade_da_lista) { return Err(Violacao::ValidadeLongaDemais); }
    if agora >= nova.valida_ate { return Err(Violacao::Vencida); }
    match aprovacao::avaliar(&pedido_da_lista(nova), &nova.assinaturas, &e.administradores, agora) {
        Resultado::Aprovado { .. } => Ok(()),
        Resultado::Expirado => Err(Violacao::Vencida),
        _ => Err(Violacao::AssinaturasInsuficientes),
    }
}

/// Servidor: publica a lista só se for válida e couber nos assentos contratados.
#[derive(Debug, Default)]
pub struct Quadro { pub atual: Option<Lista> }

impl Quadro {
    pub fn publicar(&mut self, e: &Empresa, nova: Lista, agora: u64) -> Result<(), Violacao> {
        if nova.ativas.len() > e.assentos_contratados { return Err(Violacao::SemAssentos); }
        validar_lista(e, self.atual.as_ref(), &nova, agora)?;
        self.atual = Some(nova);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decisao {
    Liberar,
    /// Trava o app. `apagar_chaves = true` quando a empresa suspendeu a conta.
    Travar { motivo: Motivo, apagar_chaves: bool },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Motivo { Suspensa, NaoConsta, ListaVencida, SemLista, ListaInvalida, SemHoraConfiavel }

/// Aparelho do funcionário, a cada abertura e periodicamente. Devolve a decisão e a lista a guardar.
pub fn decidir_no_aparelho<'a>(e: &Empresa, ultima_aceita: Option<&'a Lista>, nova: Option<&'a Lista>, conta: &IdConta, agora: u64) -> (Decisao, Option<&'a Lista>) {
    let atual = match (ultima_aceita, nova) {
        (a, Some(n)) => match validar_lista(e, a, n, agora) {
            Ok(()) => Some(n),
            // Lista nova inválida (forjada, antiga, sem assinaturas): ignorada; vale a última aceita.
            Err(_) => a,
        },
        (a, None) => a,
    };
    let Some(l) = atual else { return (Decisao::Travar { motivo: Motivo::SemLista, apagar_chaves: false }, None) };
    if l.suspensas.contains(conta) { return (Decisao::Travar { motivo: Motivo::Suspensa, apagar_chaves: true }, Some(l)); }
    if agora >= l.valida_ate { return (Decisao::Travar { motivo: Motivo::ListaVencida, apagar_chaves: false }, Some(l)); }
    if !l.ativas.contains(conta) { return (Decisao::Travar { motivo: Motivo::NaoConsta, apagar_chaves: false }, Some(l)); }
    (Decisao::Liberar, Some(l))
}

/// Relógio em que o funcionário não consegue mexer.
///
/// Voltar o relógio do celular manteria uma lista “válida” para sempre sem internet. Por isso a hora
/// usada é a maior entre o relógio do aparelho e (último horário visto do servidor + tempo decorrido no
/// relógio monotônico do sistema, que o usuário não altera). Se o aparelho foi reiniciado sem internet,
/// o tempo decorrido se perde: a conta corporativa só volta a abrir depois de conectar.
pub fn tempo_confiavel(relogio_do_aparelho: u64, ultimo_horario_do_servidor: u64, decorrido_monotonico: Option<u64>) -> Option<u64> {
    let decorrido = decorrido_monotonico?;
    Some(relogio_do_aparelho.max(ultimo_horario_do_servidor.saturating_add(decorrido)))
}

/// Versão que o app deve usar: calcula a hora confiável antes de decidir.
pub fn decidir_com_relogio_seguro<'a>(e: &Empresa, ultima_aceita: Option<&'a Lista>, nova: Option<&'a Lista>, conta: &IdConta,
                                     relogio_do_aparelho: u64, ultimo_horario_do_servidor: u64, decorrido_monotonico: Option<u64>) -> (Decisao, Option<&'a Lista>) {
    match tempo_confiavel(relogio_do_aparelho, ultimo_horario_do_servidor, decorrido_monotonico) {
        Some(agora) => decidir_no_aparelho(e, ultima_aceita, nova, conta, agora),
        None => (Decisao::Travar { motivo: Motivo::SemHoraConfiavel, apagar_chaves: false }, ultima_aceita),
    }
}

// ---------------------------------------------------------------- recuperação com a chave mestra

const CONTEXTO_RECUPERACAO: &[u8] = b"NimbusOne/v1/corporativo/recuperacao";

/// Aparelho do funcionário, ao entrar na empresa: cifra as chaves da conta corporativa para a chave
/// mestra. O pacote fica no servidor, ilegível para todos — inclusive para a NimbusOne.
pub fn selar_pacote_de_recuperacao(chave_mestra: &ChavePublica, conta: &IdConta, chaves_da_conta: &[u8]) -> Result<Vec<u8>, Violacao> {
    let mut aad = CONTEXTO_RECUPERACAO.to_vec();
    aad.extend_from_slice(conta);
    envelope::selar(chave_mestra, chaves_da_conta, &aad).map_err(|_| Violacao::SemChaveMestra)
}

/// Ordem de recuperação: dados da conta suspensa `conta` vão para a conta ativa `destino`.
#[derive(Clone, Debug)]
pub struct OrdemDeRecuperacao {
    pub empresa: [u8; 16], pub numero: u64, pub conta: IdConta, pub destino: IdConta,
    pub emitida_em: u64, pub valida_ate: u64, pub minimo_declarado: u8, pub assinaturas: Vec<Aprovacao>,
}

fn hash_da_ordem(o: &OrdemDeRecuperacao) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"NimbusOne/v1/ordem-de-recuperacao");
    h.update(o.empresa);
    h.update(o.numero.to_be_bytes());
    h.update(o.conta);
    h.update(o.destino);
    h.finalize().into()
}

fn pedido_da_ordem(o: &OrdemDeRecuperacao) -> Pedido { pedido(o.empresa, hash_da_ordem(o), o.numero, o.emitida_em, o.valida_ate, o.minimo_declarado) }

pub fn assinar_ordem(chave: &ChaveDeAssinatura, o: &OrdemDeRecuperacao) -> Aprovacao { aprovacao::aprovar(chave, &pedido_da_ordem(o)) }

/// Confere se a recuperação pode acontecer. A conta precisa estar SUSPENSA na lista vigente.
/// Espera entre a ordem de recuperação de dados e a abertura (EO5): 24 horas. Fecha o golpe do "diretor
/// com pressa" — alguém que se passa pelo presidente e pede, por telefone, para recuperar os dados de uma
/// conta "agora". Nesse período, a ordem aparece para todos os administradores, que podem cancelar.
pub const CARENCIA_DA_RECUPERACAO: u64 = 24 * 3600;

pub fn autorizar_recuperacao(e: &Empresa, lista_vigente: &Lista, o: &OrdemDeRecuperacao, agora: u64) -> Result<(), Violacao> {
    if e.chave_mestra_publica.is_none() { return Err(Violacao::SemChaveMestra); }
    if o.empresa != e.id || lista_vigente.empresa != e.id { return Err(Violacao::EmpresaDiferente); }
    if !lista_vigente.suspensas.contains(&o.conta) { return Err(Violacao::ContaNaoSuspensa); }
    if o.destino == o.conta || !lista_vigente.ativas.contains(&o.destino) { return Err(Violacao::DestinoInvalido); }
    if o.minimo_declarado < e.politica.minimo_para_recuperar { return Err(Violacao::AssinaturasInsuficientes); }
    if agora < o.emitida_em.saturating_add(CARENCIA_DA_RECUPERACAO) { return Err(Violacao::EmCarencia); }
    match aprovacao::avaliar(&pedido_da_ordem(o), &o.assinaturas, &e.administradores, agora) {
        Resultado::Aprovado { .. } => Ok(()),
        Resultado::Expirado => Err(Violacao::Vencida),
        _ => Err(Violacao::AssinaturasInsuficientes),
    }
}

/// Com a ordem autorizada e a chave mestra reconstruída pelos guardiões, abre o pacote da conta.
pub fn abrir_pacote_de_recuperacao(e: &Empresa, lista_vigente: &Lista, o: &OrdemDeRecuperacao, chave_mestra_secreta: &[u8], pacote: &[u8], agora: u64) -> Result<Zeroizing<Vec<u8>>, Violacao> {
    autorizar_recuperacao(e, lista_vigente, o, agora)?;
    let publica = e.chave_mestra_publica.as_ref().ok_or(Violacao::SemChaveMestra)?;
    let secreta = ChaveSecreta::de_bytes(chave_mestra_secreta).map_err(|_| Violacao::SemChaveMestra)?;
    let mut aad = CONTEXTO_RECUPERACAO.to_vec();
    aad.extend_from_slice(&o.conta);
    envelope::abrir(&secreta, publica, pacote, &aad).map_err(|_| Violacao::SemChaveMestra)
}

/// Troca da lista de administradores: exige `minimo_para_mudar_administradores` dos administradores ATUAIS.
// 8 argumentos: cada um é um campo distinto do pedido assinado (COR1–COR24); agrupar num struct só para
// calar o lint tornaria a chamada menos legível no único call site de produção e nos testes.
#[allow(clippy::too_many_arguments)]
pub fn validar_troca_de_administradores(e: &Empresa, novos: &[ChaveDeVerificacao], numero: u64, emitida_em: u64, valida_ate: u64,
                                        minimo_declarado: u8, assinaturas: &[Aprovacao], agora: u64) -> Result<(), Violacao> {
    if novos.is_empty() { return Err(Violacao::ListaDeAdministradoresVazia); }
    if minimo_declarado < e.politica.minimo_para_mudar_administradores { return Err(Violacao::AssinaturasInsuficientes); }
    let mut h = Sha256::new();
    h.update(b"NimbusOne/v1/administradores");
    for k in novos { h.update(k.para_bytes()); }
    let p = pedido(e.id, h.finalize().into(), numero, emitida_em, valida_ate, minimo_declarado);
    match aprovacao::avaliar(&p, assinaturas, &e.administradores, agora) {
        Resultado::Aprovado { .. } => Ok(()),
        _ => Err(Violacao::AssinaturasInsuficientes),
    }
}

/// Mesmo cálculo que `validar_troca_de_administradores` usa — para o administrador assinar a troca.
pub fn assinar_troca_de_administradores(chave: &ChaveDeAssinatura, e: &Empresa, novos: &[ChaveDeVerificacao], numero: u64, emitida_em: u64, valida_ate: u64, minimo_declarado: u8) -> Aprovacao {
    let mut h = Sha256::new();
    h.update(b"NimbusOne/v1/administradores");
    for k in novos { h.update(k.para_bytes()); }
    aprovacao::aprovar(chave, &pedido(e.id, h.finalize().into(), numero, emitida_em, valida_ate, minimo_declarado))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::{assinatura, custodia};

    const EMPRESA: [u8; 16] = [9; 16];
    fn conta(n: u8) -> IdConta { [n; 32] }

    struct Cenario { admins: Vec<ChaveDeAssinatura>, empresa: Empresa, mestra_secreta: Vec<u8> }

    fn cenario() -> Cenario {
        let admins: Vec<ChaveDeAssinatura> = (0..3).map(|_| assinatura::gerar()).collect();
        let (publica, secreta) = envelope::gerar_par_de_chaves();
        let empresa = Empresa { id: EMPRESA, administradores: admins.iter().map(|a| a.verificacao()).collect(),
                                politica: Politica::padrao(), chave_mestra_publica: Some(publica), assentos_contratados: 50 };
        Cenario { admins, empresa, mestra_secreta: secreta.para_bytes().to_vec() }
    }

    fn lista(numero: u64, emitida: u64, ativas: &[u8], suspensas: &[u8], minimo: u8) -> Lista {
        Lista { empresa: EMPRESA, numero, emitida_em: emitida, valida_ate: emitida + 900,
                ativas: ativas.iter().map(|n| conta(*n)).collect(), suspensas: suspensas.iter().map(|n| conta(*n)).collect(),
                minimo_declarado: minimo, assinaturas: vec![] }
    }

    fn assinada(c: &Cenario, mut l: Lista, quantos: usize) -> Lista {
        l.assinaturas = c.admins.iter().take(quantos).map(|a| assinar_lista(a, &l)).collect();
        l
    }

    #[test]
    fn um_administrador_suspende_e_o_aparelho_apaga_as_chaves() {
        let c = cenario();
        let l1 = assinada(&c, lista(1, 0, &[1, 2, 3], &[], 1), 1);
        let l2 = assinada(&c, lista(2, 10, &[1, 3], &[2], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l1), &l2, 20), Ok(()));
        let (d, _) = decidir_no_aparelho(&c.empresa, Some(&l1), Some(&l2), &conta(2), 20);
        assert_eq!(d, Decisao::Travar { motivo: Motivo::Suspensa, apagar_chaves: true });
        let (d_outro, _) = decidir_no_aparelho(&c.empresa, Some(&l1), Some(&l2), &conta(1), 20);
        assert_eq!(d_outro, Decisao::Liberar, "os outros funcionários seguem normalmente");
    }

    #[test]
    fn aparelho_offline_trava_quando_a_lista_vence() {
        let c = cenario();
        let l1 = assinada(&c, lista(1, 0, &[1], &[], 1), 1);
        let (d, _) = decidir_no_aparelho(&c.empresa, Some(&l1), None, &conta(1), 899);
        assert_eq!(d, Decisao::Liberar);
        let (d, _) = decidir_no_aparelho(&c.empresa, Some(&l1), None, &conta(1), 900);
        assert_eq!(d, Decisao::Travar { motivo: Motivo::ListaVencida, apagar_chaves: false });
    }

    #[test]
    fn funcionario_nao_consegue_se_reativar_nem_forjar_lista() {
        let c = cenario();
        let l1 = assinada(&c, lista(1, 0, &[1, 2], &[], 1), 1);
        let l2 = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        // Funcionário (sem chave de administrador) tenta publicar lista em que volta a estar ativo.
        let funcionario = assinatura::gerar();
        let mut forjada = lista(3, 20, &[1, 2], &[], 2);
        forjada.assinaturas = vec![assinar_lista(&funcionario, &forjada), assinar_lista(&funcionario, &forjada)];
        assert_eq!(validar_lista(&c.empresa, Some(&l2), &forjada, 30), Err(Violacao::AssinaturasInsuficientes));
        let (d, _) = decidir_no_aparelho(&c.empresa, Some(&l2), Some(&forjada), &conta(2), 30);
        assert_eq!(d, Decisao::Travar { motivo: Motivo::Suspensa, apagar_chaves: true }, "lista forjada é ignorada");
        let _ = l1;
    }

    #[test]
    fn reativar_exige_dois_administradores() {
        let c = cenario();
        let l2 = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        let so_um = assinada(&c, lista(3, 20, &[1, 2], &[], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l2), &so_um, 30), Err(Violacao::AssinaturasInsuficientes));
        let dois = assinada(&c, lista(3, 20, &[1, 2], &[], 2), 2);
        assert_eq!(validar_lista(&c.empresa, Some(&l2), &dois, 30), Ok(()));
    }

    #[test]
    fn suspensao_em_massa_exige_dois_administradores() {
        let c = cenario();
        let todos: Vec<u8> = (1..=10).collect();
        let l1 = assinada(&c, lista(1, 0, &todos, &[], 1), 1);
        // Suspender 6 de uma vez (acima do limite de 5) com uma assinatura só: recusado.
        let massa = assinada(&c, lista(2, 10, &[7, 8, 9, 10], &[1, 2, 3, 4, 5, 6], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l1), &massa, 20), Err(Violacao::AssinaturasInsuficientes));
        // Exatamente 5 (o limite): um administrador basta.
        let cinco = assinada(&c, lista(2, 10, &[6, 7, 8, 9, 10], &[1, 2, 3, 4, 5], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l1), &cinco, 20), Ok(()));
    }

    #[test]
    fn voltar_a_lista_antiga_ou_pular_numero_e_recusado() {
        let c = cenario();
        let l1 = assinada(&c, lista(1, 0, &[1, 2], &[], 1), 1);
        let l2 = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        let antiga = assinada(&c, lista(1, 20, &[1, 2], &[], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l2), &antiga, 25), Err(Violacao::SaltoOuVolta));
        let pulo = assinada(&c, lista(5, 20, &[1], &[2], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l2), &pulo, 25), Err(Violacao::SaltoOuVolta));
        let _ = l1;
    }

    #[test]
    fn nenhuma_conta_some_sem_registro() {
        let c = cenario();
        let l1 = assinada(&c, lista(1, 0, &[1, 2], &[], 1), 1);
        let some = assinada(&c, lista(2, 10, &[1], &[], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l1), &some, 20), Err(Violacao::ContaSumiu));
        let l2 = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        let apaga = assinada(&c, lista(3, 20, &[1], &[], 1), 1);
        assert_eq!(validar_lista(&c.empresa, Some(&l2), &apaga, 30), Err(Violacao::ContaApagadaDaHistoria));
    }

    #[test]
    fn validade_longa_e_recusada_para_nao_burlar_o_travamento_offline() {
        let c = cenario();
        let mut l = lista(1, 0, &[1], &[], 1);
        l.valida_ate = 60 * 60 * 24 * 30;
        let l = assinada(&c, l, 1);
        assert_eq!(validar_lista(&c.empresa, None, &l, 10), Err(Violacao::ValidadeLongaDemais));
    }

    #[test]
    fn servidor_nao_publica_alem_dos_assentos() {
        let mut c = cenario();
        c.empresa.assentos_contratados = 2;
        let mut q = Quadro::default();
        assert_eq!(q.publicar(&c.empresa, assinada(&c, lista(1, 0, &[1, 2, 3], &[], 1), 1), 5), Err(Violacao::SemAssentos));
        assert_eq!(q.publicar(&c.empresa, assinada(&c, lista(1, 0, &[1, 2], &[], 1), 1), 5), Ok(()));
    }

    const DEPOIS: u64 = 100 + CARENCIA_DA_RECUPERACAO + 100;

    #[test]
    fn recuperacao_de_dados_espera_24_horas() {
        let c = cenario();
        let vigente = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        let o = ordem(&c, 2, 1, 2, 2);
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &o, 200), Err(Violacao::EmCarencia), "pedido com pressa não passa");
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &o, DEPOIS), Ok(()));
    }

    fn ordem(c: &Cenario, conta_alvo: u8, destino: u8, assinantes: usize, minimo: u8) -> OrdemDeRecuperacao {
        let mut o = OrdemDeRecuperacao { empresa: EMPRESA, numero: 1, conta: conta(conta_alvo), destino: conta(destino),
                                         emitida_em: 100, valida_ate: 100 + CARENCIA_DA_RECUPERACAO + 1_000, minimo_declarado: minimo, assinaturas: vec![] };
        o.assinaturas = c.admins.iter().take(assinantes).map(|a| assinar_ordem(a, &o)).collect();
        o
    }

    #[test]
    fn recuperacao_completa_com_dois_administradores_e_dois_guardioes() {
        let c = cenario();
        let chaves_do_funcionario = b"chaves da conta corporativa do Marcos".to_vec();
        let pacote = selar_pacote_de_recuperacao(c.empresa.chave_mestra_publica.as_ref().unwrap(), &conta(2), &chaves_do_funcionario).unwrap();
        let (partes, _) = custodia::dividir(&c.mestra_secreta, 3, 2).unwrap();
        let vigente = assinada(&c, lista(2, 10, &[1, 3], &[2], 1), 1);
        let o = ordem(&c, 2, 1, 2, 2);
        let mestra = custodia::reconstruir(&[&partes[0], &partes[2]]).unwrap();
        let aberto = abrir_pacote_de_recuperacao(&c.empresa, &vigente, &o, &mestra, &pacote, DEPOIS).unwrap();
        assert_eq!(&aberto[..], &chaves_do_funcionario[..]);
    }

    #[test]
    fn empresa_nao_le_escondido_quem_ainda_trabalha() {
        let c = cenario();
        let vigente = assinada(&c, lista(1, 0, &[1, 2], &[], 1), 1);
        let o = ordem(&c, 2, 1, 2, 2);
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &o, DEPOIS), Err(Violacao::ContaNaoSuspensa));
    }

    #[test]
    fn um_administrador_sozinho_nao_recupera() {
        let c = cenario();
        let vigente = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &ordem(&c, 2, 1, 1, 1), DEPOIS), Err(Violacao::AssinaturasInsuficientes));
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &ordem(&c, 2, 1, 1, 2), DEPOIS), Err(Violacao::AssinaturasInsuficientes));
    }

    #[test]
    fn destino_precisa_ser_outra_conta_ativa() {
        let c = cenario();
        let vigente = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &ordem(&c, 2, 2, 2, 2), DEPOIS), Err(Violacao::DestinoInvalido));
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &ordem(&c, 2, 7, 2, 2), DEPOIS), Err(Violacao::DestinoInvalido));
    }

    #[test]
    fn sem_chave_mestra_nada_se_recupera() {
        let mut c = cenario();
        c.empresa.chave_mestra_publica = None;
        let vigente = assinada(&c, lista(2, 10, &[1], &[2], 1), 1);
        assert_eq!(autorizar_recuperacao(&c.empresa, &vigente, &ordem(&c, 2, 1, 2, 2), DEPOIS), Err(Violacao::SemChaveMestra));
    }

    #[test]
    fn pacote_de_uma_conta_nao_abre_como_se_fosse_de_outra() {
        let c = cenario();
        let pacote_do_2 = selar_pacote_de_recuperacao(c.empresa.chave_mestra_publica.as_ref().unwrap(), &conta(2), b"k2").unwrap();
        let vigente = assinada(&c, lista(2, 10, &[1], &[2, 3], 1), 1);
        let o_para_3 = ordem(&c, 3, 1, 2, 2);
        assert_eq!(abrir_pacote_de_recuperacao(&c.empresa, &vigente, &o_para_3, &c.mestra_secreta, &pacote_do_2, DEPOIS).unwrap_err(), Violacao::SemChaveMestra);
    }

    #[test]
    fn troca_de_administradores_exige_dois_atuais() {
        let c = cenario();
        let novo = assinatura::gerar();
        let novos = vec![c.admins[0].verificacao(), novo.verificacao()];
        let uma = vec![assinar_troca_de_administradores(&c.admins[0], &c.empresa, &novos, 1, 0, 100, 2)];
        assert_eq!(validar_troca_de_administradores(&c.empresa, &novos, 1, 0, 100, 2, &uma, 10), Err(Violacao::AssinaturasInsuficientes));
        let duas: Vec<Aprovacao> = c.admins.iter().take(2).map(|a| assinar_troca_de_administradores(a, &c.empresa, &novos, 1, 0, 100, 2)).collect();
        assert_eq!(validar_troca_de_administradores(&c.empresa, &novos, 1, 0, 100, 2, &duas, 10), Ok(()));
        // Um administrador sozinho declarando “basta uma assinatura”: recusado pela política.
        let sozinho = vec![assinar_troca_de_administradores(&c.admins[0], &c.empresa, &novos, 1, 0, 100, 1)];
        assert_eq!(validar_troca_de_administradores(&c.empresa, &novos, 1, 0, 100, 1, &sozinho, 10), Err(Violacao::AssinaturasInsuficientes));
        // Quem não é administrador (o próprio candidato) não conta.
        let do_candidato = vec![assinar_troca_de_administradores(&novo, &c.empresa, &novos, 1, 0, 100, 2), uma[0].clone()];
        assert_eq!(validar_troca_de_administradores(&c.empresa, &novos, 1, 0, 100, 2, &do_candidato, 10), Err(Violacao::AssinaturasInsuficientes));
    }

    #[test]
    fn voltar_o_relogio_do_celular_nao_estende_o_acesso() {
        let c = cenario();
        let l1 = assinada(&c, lista(1, 0, &[1], &[], 1), 1);
        // Último contato com o servidor às 100; passaram 2.000 s no relógio monotônico; o funcionário
        // voltou o relógio do celular para 50.
        let (d, _) = decidir_com_relogio_seguro(&c.empresa, Some(&l1), None, &conta(1), 50, 100, Some(2_000));
        assert_eq!(d, Decisao::Travar { motivo: Motivo::ListaVencida, apagar_chaves: false });
    }

    #[test]
    fn reiniciar_sem_internet_exige_conectar() {
        let c = cenario();
        let l1 = assinada(&c, lista(1, 0, &[1], &[], 1), 1);
        let (d, _) = decidir_com_relogio_seguro(&c.empresa, Some(&l1), None, &conta(1), 10, 5, None);
        assert_eq!(d, Decisao::Travar { motivo: Motivo::SemHoraConfiavel, apagar_chaves: false });
        assert_eq!(tempo_confiavel(10, 5, Some(3)), Some(10));
        assert_eq!(tempo_confiavel(1, 5, Some(3)), Some(8));
    }
}
