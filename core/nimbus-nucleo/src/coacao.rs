//! # Módulo `coacao` — PIN de coação e negação plausível (AL2, AR6)
//!
//! O cliente cadastra dois segredos: o real, que abre o cofre verdadeiro, e o de coação, que abre um
//! **cofre-isca** com conteúdo inofensivo. Sob ameaça, o cliente entrega o de coação.
//!
//! Regras de blindagem:
//! * Comparação em **tempo constante**: o tempo de resposta não revela qual segredo foi digitado.
//! * O cofre-isca abre normalmente, sem aviso na tela: quem coage não percebe a diferença.
//! * Abrir com o segredo de coação dispara, de forma silenciosa, o **modo de proteção**: bloqueio do
//!   cofre real, corte das outras sessões e alerta assinado ao próprio dono (entregue depois).
//! * Segredo errado apenas erra: não existe terceiro comportamento a ser descoberto por tentativa.
//! * Os dois cofres têm tamanho de dados padronizado, para o volume não denunciar a existência do real.
//! * Os verificadores são derivados com Argon2id (módulo `senha`): vazar o arquivo não entrega os PINs.
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

/// Verificador guardado no aparelho (nunca no servidor).
pub struct Verificador { real: [u8; 64], coacao: [u8; 64], sal: [u8; 16] }

impl core::fmt::Debug for Verificador {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "Verificador(<oculto>)") }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeCoacao {
    /// Segredo de coação igual ao real: o cofre-isca nunca abriria. Recusado no cadastro.
    SegredosIguais,
    /// Segredo vazio.
    SegredoVazio,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Abertura {
    /// Abre o cofre verdadeiro.
    CofreReal,
    /// Abre o cofre-isca e dispara o modo de proteção, sem nada aparecer na tela.
    CofreIscaComProtecao,
    /// Segredo incorreto.
    Negada,
}

fn derivar(sal: &[u8; 16], segredo: &[u8]) -> [u8; 64] {
    // Argon2id (RFC 9106), 64 MiB: vazar o verificador não entrega o PIN por força bruta barata.
    let mut contexto = b"NimbusOne/v1/coacao".to_vec();
    contexto.extend_from_slice(sal);
    *crate::senha::derivar::<64>(segredo, &contexto).expect("sal de 35 bytes, parâmetros fixos")
}

impl Verificador {
    pub fn novo(sal: [u8; 16], segredo_real: &[u8], segredo_de_coacao: &[u8]) -> Result<Self, ErroDeCoacao> {
        if segredo_real.is_empty() || segredo_de_coacao.is_empty() { return Err(ErroDeCoacao::SegredoVazio); }
        if bool::from(segredo_real.ct_eq(segredo_de_coacao)) { return Err(ErroDeCoacao::SegredosIguais); }
        Ok(Verificador { real: derivar(&sal, segredo_real), coacao: derivar(&sal, segredo_de_coacao), sal })
    }

    /// Avalia o segredo digitado. Sempre executa as duas comparações, em tempo constante.
    pub fn avaliar(&self, digitado: &[u8]) -> Abertura {
        let tentativa = Zeroizing::new(derivar(&self.sal, digitado));
        let bate_real: bool = tentativa.ct_eq(&self.real).into();
        let bate_coacao: bool = tentativa.ct_eq(&self.coacao).into();
        if bate_real { Abertura::CofreReal }
        else if bate_coacao { Abertura::CofreIscaComProtecao }
        else { Abertura::Negada }
    }
}

/// Como o app reage a PINs errados. O usuário escolhe entre três níveis prontos, ou ajusta os números.
/// Em todos, o PIN de coação nunca conta como erro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Politica {
    /// Erros livres antes de começar a esperar.
    pub erros_sem_espera: u32,
    /// Espera inicial, em segundos (dobra a cada erro seguinte).
    pub espera_inicial: u64,
    /// Erros até apagar as chaves deste aparelho. `None` = nunca apaga (só espera cada vez mais).
    pub erros_para_apagar: Option<u32>,
    /// Teto da espera que dobra (v37, terceira rodada): se `Some(teto)`, a espera para de dobrar ao
    /// chegar neste valor e fica parada nele (nunca cresce mais, nunca encolhe). `None` (todos os modos
    /// prontos de antes desta rodada) mantém o comportamento de sempre: dobra para sempre, sem limite.
    pub teto_de_espera_segundos: Option<u64>,
}

/// Depois de quantos erros o aparelho trava de vez (apaga as chaves, exige reconfigurar do zero) — as três
/// opções que a tela oferece para este limite, à escolha da pessoa. Nenhuma outra quantidade é oferecida na
/// tela por este caminho (a `Politica` em si aceita qualquer valor entre 2 e 100, para os modos prontos
/// `rigida`/`equilibrada`/`tolerante`, mas a ESCOLHA EXPLÍCITA de "travar depois de N erros" fica restrita a
/// estas três, para não virar uma tela com números demais).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimiteParaApagar { Dez, Quinze, Vinte }

impl LimiteParaApagar {
    pub fn tentativas(self) -> u32 { match self { LimiteParaApagar::Dez => 10, LimiteParaApagar::Quinze => 15, LimiteParaApagar::Vinte => 20 } }
}

/// Teto diurno da espera: de 1 a 12 horas, à escolha da pessoa (v37, terceira rodada) — substitui, para
/// quem escolher, o antigo comportamento de dobrar para sempre sem limite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TetoDiurno(u8);

/// Limites do teto diurno, em horas.
pub const TETO_DIURNO_MINIMO_HORAS: u8 = 1;
pub const TETO_DIURNO_MAXIMO_HORAS: u8 = 12;

impl TetoDiurno {
    /// `horas`: 1 a 12. Fora do intervalo, é ajustado para o limite mais próximo (mesmo espírito de `valida`).
    pub fn de_horas(horas: u8) -> Self { TetoDiurno(horas.clamp(TETO_DIURNO_MINIMO_HORAS, TETO_DIURNO_MAXIMO_HORAS)) }
    pub fn em_segundos(self) -> u64 { self.0 as u64 * 3_600 }
}

/// Por volta da meia-noite (hora CONFIRMADA pelo servidor — nunca a hora local do aparelho, pelo mesmo
/// motivo de `RelogioDeEspera`: quem pega o aparelho poderia só adiantar o relógio para fingir que são
/// 2 da manhã e ganhar a janela noturna o tempo todo), o app pode exigir o PIN de novo mesmo que já
/// estivesse desbloqueado. Esta constante é o piso de espera que esse evento, sozinho, pode introduzir —
/// NUNCA um teto: se já havia uma espera maior em curso por erros de verdade, ela continua valendo
/// inteira (ver `espera_apos_reinicio_noturno`). Interpretar isto como um teto abriria uma janela de força
/// bruta: ao perceber que a espera cai para no máximo 60 s toda madrugada, um atacante paciente ganharia
/// até ~300 tentativas rápidas por noite (5 h ÷ 60 s), todo santo dia, se a política nunca apagar. Por
/// isso aqui vale como PISO (nunca reduz uma espera maior já em curso), nunca como teto.
pub const PISO_NOTURNO_SEGUNDOS: u64 = 60;

impl Politica {
    /// Equilibrado (padrão): 3 livres, espera a partir de 30 s, apaga no 10º erro.
    pub fn equilibrada() -> Self { Politica { erros_sem_espera: 3, espera_inicial: 30, erros_para_apagar: Some(10), teto_de_espera_segundos: None } }
    /// Rígido: 2 livres, espera a partir de 60 s, apaga no 5º erro. Para quem topa perder o aparelho por segurança.
    pub fn rigida() -> Self { Politica { erros_sem_espera: 2, espera_inicial: 60, erros_para_apagar: Some(5), teto_de_espera_segundos: None } }
    /// Tolerante: 5 livres, espera a partir de 15 s, nunca apaga. Para quem tem medo de apagar sem querer.
    pub fn tolerante() -> Self { Politica { erros_sem_espera: 5, espera_inicial: 15, erros_para_apagar: None, teto_de_espera_segundos: None } }

    /// Primeiro modo que você pediu: "depois de 10 tentativas, trava e espera 15 min, depois vai aumentando" —
    /// nunca apaga as chaves sozinho, só espera cada vez mais (dobra a cada novo erro). As primeiras
    /// `apos_tentativas - 1` tentativas são livres (sem espera nenhuma); a partir da tentativa `apos_tentativas`,
    /// a espera começa em `espera_inicial_segundos` e dobra a cada erro seguinte, para sempre (sem teto).
    pub fn espera_crescente_sem_apagar(apos_tentativas: u32, espera_inicial_segundos: u64) -> Self {
        Politica { erros_sem_espera: apos_tentativas.saturating_sub(1).max(1), espera_inicial: espera_inicial_segundos, erros_para_apagar: None, teto_de_espera_segundos: None }.valida()
    }

    /// Mesmo modo acima, mas com o teto diurno que você pediu nesta rodada (1 a 12 horas): a espera dobra
    /// normalmente até bater no teto e para aí, em vez de dobrar para sempre.
    pub fn espera_crescente_com_teto_diurno(apos_tentativas: u32, espera_inicial_segundos: u64, teto: TetoDiurno) -> Self {
        Politica { erros_sem_espera: apos_tentativas.saturating_sub(1).max(1), espera_inicial: espera_inicial_segundos, erros_para_apagar: None, teto_de_espera_segundos: Some(teto.em_segundos()) }.valida()
    }

    /// Segundo modo que você pediu: "depois de 10, 15 ou 20 tentativas (à escolha da pessoa), o aparelho
    /// trava e precisa reiniciar tudo" — sem espera crescente no meio: as tentativas anteriores são livres, e
    /// a que atinge o limite apaga as chaves direto (`AposErro::ApagarChaves`), exigindo reconfigurar o
    /// aparelho do zero (novo PIN, nova recuperação com os guardiões).
    pub fn apagar_apos_escolha_do_usuario(limite: LimiteParaApagar) -> Self {
        let n = limite.tentativas();
        Politica { erros_sem_espera: n - 1, espera_inicial: 3_600, erros_para_apagar: Some(n), teto_de_espera_segundos: None }.valida()
    }

    /// Garante limites sãos (o app corrige valores absurdos vindos da tela).
    pub fn valida(self) -> Self {
        let erros_sem_espera = self.erros_sem_espera.clamp(1, 20);
        let espera_inicial = self.espera_inicial.clamp(5, 3_600);
        Politica {
            erros_sem_espera,
            espera_inicial,
            // o piso usa o valor JÁ corrigido, senão "apagar" poderia cair dentro dos erros livres
            erros_para_apagar: self.erros_para_apagar.map(|n| n.clamp(erros_sem_espera + 1, 100)),
            // o teto nunca pode ser menor que a própria espera inicial (senão nem o primeiro degrau cabe)
            teto_de_espera_segundos: self.teto_de_espera_segundos.map(|t| t.max(espera_inicial)),
        }
    }
}

impl Default for Politica { fn default() -> Self { Politica::equilibrada() } }

/// Contador de erros de PIN, guiado por uma `Politica`. O contador fica no armazenamento protegido pelo chip.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Tentativas { erros: u32, liberado_a_partir_de: u64 }

/// Relógio das esperas do PIN (v37, segunda rodada). As esperas não podem usar a hora do sistema: quem pega o
/// aparelho adiantaria o relógio e pularia todas as esperas — no modo Tolerante, que nunca apaga, isso permitiria
/// tentar todos os PINs. Este relógio só avança com o tempo realmente decorrido no relógio monotônico do
/// aparelho; reiniciar o aparelho não soma nada (a espera recomeça, nunca encurta). Guarde-o junto com as
/// `Tentativas`, no armazenamento protegido pelo chip, e passe `avancar(...)` como `agora` em `pode_tentar` e
/// `errou`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RelogioDeEspera { acumulado: u64, inicializacao: u64, ultimo_monotonico: u64 }

impl RelogioDeEspera {
    /// `inicializacao`: identificador da inicialização atual do aparelho (muda a cada reinício); `monotonico`:
    /// segundos desde essa inicialização. Devolve o tempo acumulado, que é o `agora` das esperas.
    pub fn avancar(&mut self, inicializacao: u64, monotonico: u64) -> u64 {
        if inicializacao == self.inicializacao && monotonico >= self.ultimo_monotonico {
            self.acumulado = self.acumulado.saturating_add(monotonico - self.ultimo_monotonico);
        }
        self.inicializacao = inicializacao;
        self.ultimo_monotonico = monotonico;
        self.acumulado
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AposErro { TentarDeNovo, Esperar { ate: u64 }, ApagarChaves }

pub const ERROS_SEM_ESPERA: u32 = 3;

impl Tentativas {
    pub fn pode_tentar(&self, pol: &Politica, agora: u64) -> bool {
        pol.erros_para_apagar.is_none_or(|lim| self.erros < lim) && agora >= self.liberado_a_partir_de
    }

    /// Registra um erro e diz o que fazer. A espera dobra a cada erro depois dos livres, até o teto
    /// diurno da política (se houver um): dali para frente, fica parada nele.
    pub fn errou(&mut self, pol: &Politica, agora: u64) -> AposErro {
        self.erros = self.erros.saturating_add(1);
        if pol.erros_para_apagar.is_some_and(|lim| self.erros >= lim) { return AposErro::ApagarChaves; }
        if self.erros <= pol.erros_sem_espera { return AposErro::TentarDeNovo; }
        let passos = self.erros - pol.erros_sem_espera - 1;
        let mut espera = pol.espera_inicial.saturating_mul(1u64.checked_shl(passos.min(20)).unwrap_or(u64::MAX));
        if let Some(teto) = pol.teto_de_espera_segundos { espera = espera.min(teto); }
        self.liberado_a_partir_de = agora.saturating_add(espera);
        AposErro::Esperar { ate: self.liberado_a_partir_de }
    }

    pub fn acertou(&mut self) { *self = Tentativas::default(); }

    /// Quantos erros CONSECUTIVOS desde o último acerto (ou desde o início, se nunca acertou) — exatamente
    /// o mesmo contador que já decide a espera e o apagamento acima, exposto para o relógio de destruição
    /// por prazo usar como sinal de ataque em andamento (`expiracao::tempo_para_destruir_considerando_
    /// tentativas`, Cláusula 6b). Zera com `acertou()`, nunca com o tempo passando sozinho — só um PIN
    /// correto reseta a contagem, exatamente como já valia para a espera. Nenhum contador novo: esta
    /// função só lê o campo que `errou`/`acertou` já mantêm.
    pub fn erros_consecutivos(&self) -> u32 { self.erros }

    /// Reinício noturno (v37, terceira rodada): por volta da meia-noite — usando `hora_confiavel`, que
    /// SÓ pode vir de uma hora assinada pelo servidor, nunca do relógio local do aparelho (ver o porquê
    /// em `PISO_NOTURNO_SEGUNDOS`) — o app pode decidir pedir o PIN de novo mesmo que já estivesse
    /// desbloqueado. Esta função só garante um PISO de `PISO_NOTURNO_SEGUNDOS`: se a pessoa não tinha
    /// nenhuma espera pendente, o evento noturno introduz, sozinho, no máximo essa pequena pausa — mas
    /// se já havia uma espera maior em curso por erros reais, ela não encolhe nem um segundo. Chame isto
    /// no momento do reinício noturno; não chame em nenhum outro momento, e não use para decidir
    /// `pode_tentar`/`errou`, que continuam exatamente como antes.
    pub fn espera_apos_reinicio_noturno(&mut self, agora: u64) {
        self.liberado_a_partir_de = self.liberado_a_partir_de.max(agora.saturating_add(PISO_NOTURNO_SEGUNDOS));
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    const SAL: [u8; 16] = [7; 16];
    #[test]
    fn espera_do_pin_nao_acelera_com_o_relogio_do_sistema() {
        let pol = Politica::tolerante();
        let mut r = RelogioDeEspera::default();
        let mut t = Tentativas::default();
        let mut agora = r.avancar(1, 100);
        while let AposErro::TentarDeNovo = t.errou(&pol, agora) {}
        assert!(!t.pode_tentar(&pol, agora));
        // A hora do sistema não entra em lugar nenhum: só o tempo real decorrido conta.
        agora = r.avancar(1, 105);
        assert!(!t.pode_tentar(&pol, agora), "5 s reais não bastam");
        // Reiniciar o aparelho não soma tempo: a espera continua inteira.
        agora = r.avancar(2, 1_000_000);
        assert_eq!(agora, 5);
        assert!(!t.pode_tentar(&pol, agora));
        // Relógio monotônico que volta (ponte com defeito): não soma nada.
        assert_eq!(r.avancar(2, 10), 5);
        // Depois do tempo real de espera, pode tentar.
        agora = r.avancar(2, 10 + 3_600);
        assert!(t.pode_tentar(&pol, agora));
    }

    fn v() -> Verificador { Verificador::novo(SAL, b"segredo-verdadeiro", b"segredo-de-coacao").unwrap() }

    #[test]
    fn segredo_real_abre_o_cofre_real() {
        assert_eq!(v().avaliar(b"segredo-verdadeiro"), Abertura::CofreReal);
    }

    #[test]
    fn segredo_de_coacao_abre_a_isca_e_protege() {
        assert_eq!(v().avaliar(b"segredo-de-coacao"), Abertura::CofreIscaComProtecao);
    }

    #[test]
    fn segredo_errado_e_negado() {
        assert_eq!(v().avaliar(b"qualquer-outro"), Abertura::Negada);
    }

    #[test]
    fn verificadores_nao_revelam_os_segredos() {
        let ver = v();
        assert_ne!(ver.real, ver.coacao);
        assert!(!ver.real.starts_with(b"segredo"));
    }

    #[test]
    fn tentativas_de_pin_esperam_cada_vez_mais_e_no_limite_apagam() {
        let pol = Politica::equilibrada();
        let mut t = Tentativas::default();
        assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo);
        assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo);
        assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo);
        assert_eq!(t.errou(&pol, 100), AposErro::Esperar { ate: 130 }, "4º erro: espera 30 s");
        assert!(!t.pode_tentar(&pol, 129) && t.pode_tentar(&pol, 130));
        assert_eq!(t.errou(&pol, 130), AposErro::Esperar { ate: 190 }, "5º erro: espera dobra para 60 s");
        while t.erros < 9 { let _ = t.errou(&pol, 10_000); }
        assert_eq!(t.errou(&pol, 99_999), AposErro::ApagarChaves);
        assert!(!t.pode_tentar(&pol, u64::MAX), "apagou: nao pode mais tentar");
    }

    #[test]
    fn politica_rigida_apaga_mais_cedo_e_tolerante_nunca_apaga() {
        let rig = Politica::rigida();
        let mut t = Tentativas::default();
        assert_eq!(t.errou(&rig, 0), AposErro::TentarDeNovo);
        assert_eq!(t.errou(&rig, 0), AposErro::TentarDeNovo);
        assert_eq!(t.errou(&rig, 0), AposErro::Esperar { ate: 60 }, "3º erro: espera 60 s");
        assert_eq!(t.errou(&rig, 60), AposErro::Esperar { ate: 180 }, "4º erro: 120 s");
        assert_eq!(t.errou(&rig, 180), AposErro::ApagarChaves, "rígida apaga no 5º");
        let tol = Politica::tolerante();
        let mut t2 = Tentativas::default();
        for _ in 0..50 { assert_ne!(t2.errou(&tol, 0), AposErro::ApagarChaves); }
        assert!(t2.pode_tentar(&tol, u64::MAX), "tolerante nunca apaga: passado o tempo de espera, pode tentar de novo");
    }

    #[test]
    fn politica_corrige_valores_absurdos_vindos_da_tela() {
        let absurda = Politica { erros_sem_espera: 0, espera_inicial: 0, erros_para_apagar: Some(1), teto_de_espera_segundos: None }.valida();
        assert_eq!(absurda.erros_sem_espera, 1, "0 vira 1");
        assert!(absurda.espera_inicial >= 5, "0 s vira 5 s");
        assert_eq!(absurda.erros_para_apagar, Some(2), "apagar no 1 vira 2 (sempre depois dos livres)");
        let enorme = Politica { erros_sem_espera: 999, espera_inicial: 999_999, erros_para_apagar: Some(999), teto_de_espera_segundos: None }.valida();
        assert!(enorme.erros_sem_espera <= 20 && enorme.espera_inicial <= 3_600 && enorme.erros_para_apagar.unwrap() <= 100);
    }

    #[test]
    fn teto_diurno_para_a_espera_de_dobrar_e_fica_parado_nele() {
        // Pedido desta rodada: teto de 1 a 12 horas em vez de dobrar para sempre.
        let pol = Politica::espera_crescente_com_teto_diurno(10, 900, TetoDiurno::de_horas(1)); // teto = 3600 s
        let mut t = Tentativas::default();
        for n in 1..=9 { assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo, "tentativa {n} deveria ser livre"); }
        assert_eq!(t.errou(&pol, 0), AposErro::Esperar { ate: 900 }, "10ª: 15 min, ainda abaixo do teto de 1h");
        assert_eq!(t.errou(&pol, 900), AposErro::Esperar { ate: 900 + 1_800 }, "11ª: dobra para 30 min, ainda abaixo do teto");
        assert_eq!(t.errou(&pol, 2_700), AposErro::Esperar { ate: 2_700 + 3_600 }, "12ª: dobraria para 1h e já bate exatamente no teto");
        assert_eq!(t.errou(&pol, 6_300), AposErro::Esperar { ate: 6_300 + 3_600 }, "13ª: dobraria para 2h, mas fica travada no teto de 1h");
        for _ in 0..20 { assert_eq!(t.errou(&pol, 999_999), AposErro::Esperar { ate: 999_999 + 3_600 }, "nunca passa do teto, por mais erros que haja"); }
    }

    #[test]
    fn teto_diurno_e_ajustado_para_1_a_12_horas() {
        assert_eq!(TetoDiurno::de_horas(0).em_segundos(), 3_600, "0 vira o mínimo, 1h");
        assert_eq!(TetoDiurno::de_horas(1).em_segundos(), 3_600);
        assert_eq!(TetoDiurno::de_horas(12).em_segundos(), 12 * 3_600);
        assert_eq!(TetoDiurno::de_horas(255).em_segundos(), 12 * 3_600, "acima de 12 vira o máximo, 12h");
    }

    #[test]
    fn reinicio_noturno_nunca_encurta_uma_espera_real_so_garante_um_piso_pequeno() {
        let pol = Politica::espera_crescente_sem_apagar(2, 3_000); // espera real de 50 min já em curso (> piso de 60s)
        let mut t = Tentativas::default();
        assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo, "1ª tentativa é livre");
        assert_eq!(t.errou(&pol, 0), AposErro::Esperar { ate: 3_000 }, "2ª: espera real de 50 min, por erro de verdade");
        // O reinício noturno, sozinho, NUNCA encurta essa espera de 50 min para os 60s do piso noturno:
        t.espera_apos_reinicio_noturno(100);
        assert!(!t.pode_tentar(&pol, 100 + PISO_NOTURNO_SEGUNDOS), "a espera de 50 min continua valendo inteira");
        assert_eq!(t.liberado_a_partir_de, 3_000, "o piso noturno não reduziu nada");
        // Mas para quem NÃO tinha nenhuma espera pendente, o reinício noturno garante pelo menos 60s:
        let mut t2 = Tentativas::default();
        t2.espera_apos_reinicio_noturno(1_000);
        assert_eq!(t2.liberado_a_partir_de, 1_000 + PISO_NOTURNO_SEGUNDOS);
    }

    #[test]
    fn espera_crescente_sem_apagar_trava_na_tentativa_pedida_e_nunca_apaga() {
        // "Depois de 10 tentativas, trava e espera 15 min, depois vai aumentando": as 9 primeiras são
        // livres, a 10ª já espera 15 min, a 11ª dobra para 30 min, e nunca apaga sozinha.
        let pol = Politica::espera_crescente_sem_apagar(10, 900);
        let mut t = Tentativas::default();
        for n in 1..=9 { assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo, "tentativa {n} deveria ser livre"); }
        assert_eq!(t.errou(&pol, 0), AposErro::Esperar { ate: 900 }, "10ª tentativa: 15 min de espera");
        assert_eq!(t.errou(&pol, 900), AposErro::Esperar { ate: 900 + 1_800 }, "11ª tentativa: dobra para 30 min");
        for _ in 0..100 { assert_ne!(t.errou(&pol, 10_000_000), AposErro::ApagarChaves, "este modo nunca apaga"); }
    }

    #[test]
    fn apagar_apos_escolha_do_usuario_trava_exatamente_no_limite_escolhido_sem_espera_no_meio() {
        // "Depois de 10, 15 ou 20 tentativas (à escolha), o aparelho trava e precisa reiniciar tudo": as
        // tentativas anteriores são todas livres (sem espera), e a que bate o limite apaga direto.
        for (limite, n) in [(LimiteParaApagar::Dez, 10), (LimiteParaApagar::Quinze, 15), (LimiteParaApagar::Vinte, 20)] {
            let pol = Politica::apagar_apos_escolha_do_usuario(limite);
            let mut t = Tentativas::default();
            for i in 1..n { assert_eq!(t.errou(&pol, 0), AposErro::TentarDeNovo, "tentativa {i} de {n} deveria ser livre"); }
            assert_eq!(t.errou(&pol, 0), AposErro::ApagarChaves, "a {n}ª tentativa trava e apaga, sem espera no meio");
        }
    }

    #[test]
    fn acertar_zera_as_tentativas() {
        let mut t = Tentativas::default();
        let pol = Politica::equilibrada();
        for _ in 0..5 { let _ = t.errou(&pol, 0); }
        t.acertou();
        assert_eq!(t, Tentativas::default());
        assert!(t.pode_tentar(&Politica::equilibrada(), 0));
    }

    #[test]
    fn erros_consecutivos_acompanha_os_erros_e_zera_ao_acertar() {
        let pol = Politica::tolerante();
        let mut t = Tentativas::default();
        assert_eq!(t.erros_consecutivos(), 0, "nenhum erro ainda");
        let _ = t.errou(&pol, 0);
        let _ = t.errou(&pol, 0);
        assert_eq!(t.erros_consecutivos(), 2);
        let _ = t.errou(&pol, 0);
        assert_eq!(t.erros_consecutivos(), 3);
        t.acertou();
        assert_eq!(t.erros_consecutivos(), 0, "um PIN correto zera a contagem, exatamente como já zera a espera");
    }

    #[test]
    fn segredo_de_coacao_igual_ao_real_e_recusado() {
        assert_eq!(Verificador::novo(SAL, b"123456", b"123456").unwrap_err(), ErroDeCoacao::SegredosIguais);
        assert_eq!(Verificador::novo(SAL, b"", b"1").unwrap_err(), ErroDeCoacao::SegredoVazio);
        assert_eq!(format!("{:?}", v()), "Verificador(<oculto>)");
    }

    #[test]
    fn sais_diferentes_geram_verificadores_diferentes() {
        let a = Verificador::novo([1; 16], b"x", b"y").unwrap();
        let b = Verificador::novo([2; 16], b"x", b"y").unwrap();
        assert_ne!(a.real, b.real);
    }
}
