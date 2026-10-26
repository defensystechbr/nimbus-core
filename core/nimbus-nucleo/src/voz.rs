//! # Módulo `voz` — modulador de voz do NimbusChat (VZ4, VC4)
//!
//! Transforma a voz em tempo real **no aparelho, antes de cifrar**: a voz original nunca sai do celular.
//! É descaracterização prática, não anonimato à prova de perícia forense de áudio — declarar ao usuário.
//!
//! Nível 1 (pronto neste módulo, tempo real): deslocamento de tom por linha de atraso com duas cabeças
//! de leitura e janela cruzada, amostra a amostra, sem alterar a duração e sem latência de bloco.
//! Acrescenta um **desvio aleatório por chamada** (sorteado a cada ligação), para que o tom exato não
//! seja conhecido e revertido por quem gravar a outra ponta.
//!
//! Verdade técnica: deslocar o tom é reversível em boa parte por perícia (basta deslocar de volta,
//! com margem de erro). Por isso o Nível 2 do Manual (conversão neural de voz no aparelho, para uma
//! voz sintética fixa) é o que remove a identidade vocal de forma não reversível. Mesmo nele, sotaque,
//! vocabulário e ritmo de fala continuam identificando a pessoa.
use zeroize::Zeroize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Predefinicao { Desligado, Grave, Agudo, Neutro, Personalizado(i32) }

impl Predefinicao {
    /// Quantos semitons deslocar (limitado a -12..12).
    pub fn semitons(self) -> i32 {
        match self {
            Predefinicao::Desligado => 0,
            Predefinicao::Grave => -5,
            Predefinicao::Agudo => 5,
            Predefinicao::Neutro => -2,
            Predefinicao::Personalizado(s) => s.clamp(-12, 12),
        }
    }
    /// Fator de reamostragem correspondente: 2^(semitons/12).
    pub fn fator(self) -> f32 { 2f32.powf(self.semitons() as f32 / 12.0) }
}

#[derive(Clone, Copy, Debug)]
pub struct PlanoDeJanela { pub tamanho_janela: usize, pub avanco: usize, pub fator: f32 }

/// Janela de 2048 amostras com avanço de 1/4 (sobreposição de 75%).
pub fn planejar(p: Predefinicao, _taxa_amostragem: u32) -> PlanoDeJanela {
    PlanoDeJanela { tamanho_janela: 2048, avanco: 512, fator: p.fator() }
}

/// Deslocador de tom em tempo real (Nível 1).
pub struct DeslocadorDeTom {
    buffer: Vec<f32>,
    escrita: usize,
    atraso: f32,
    janela: f32,
    fator: f32,
}

impl DeslocadorDeTom {
    /// `janela_amostras`: tamanho da janela cruzada (ex.: 1024 amostras a 48 kHz ≈ 21 ms).
    pub fn novo(fator: f32, janela_amostras: usize) -> Self {
        let janela = janela_amostras.max(64);
        DeslocadorDeTom { buffer: vec![0.0; janela + 4], escrita: 0, atraso: 0.0, janela: janela as f32, fator: fator.clamp(0.5, 2.0) }
    }

    /// Cria com a predefinição e um desvio aleatório de até ±`desvio_semitons` sorteado para esta chamada.
    pub fn para_chamada(p: Predefinicao, janela_amostras: usize, desvio_semitons: f32) -> Self {
        use rand_core::{OsRng, RngCore};
        let sorteio = (OsRng.next_u32() as f32 / u32::MAX as f32) * 2.0 - 1.0; // -1..1
        let semitons = p.semitons() as f32 + sorteio * desvio_semitons.abs();
        Self::novo(2f32.powf(semitons / 12.0), janela_amostras)
    }

    pub fn fator(&self) -> f32 { self.fator }

    fn ler(&self, atraso: f32) -> f32 {
        let n = self.buffer.len() as f32;
        let mut pos = self.escrita as f32 - atraso;
        while pos < 0.0 { pos += n; }
        let i = pos.floor() as usize % self.buffer.len();
        let j = (i + 1) % self.buffer.len();
        let f = pos - pos.floor();
        self.buffer[i] * (1.0 - f) + self.buffer[j] * f
    }

    /// Processa uma amostra.
    pub fn amostra(&mut self, x: f32) -> f32 {
        self.buffer[self.escrita] = x;
        let d1 = self.atraso;
        let d2 = (self.atraso + self.janela / 2.0) % self.janela;
        let w1 = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * d1 / self.janela).cos();
        let w2 = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * d2 / self.janela).cos();
        let y = w1 * self.ler(d1) + w2 * self.ler(d2);
        self.atraso += 1.0 - self.fator;
        while self.atraso < 0.0 { self.atraso += self.janela; }
        while self.atraso >= self.janela { self.atraso -= self.janela; }
        self.escrita = (self.escrita + 1) % self.buffer.len();
        y
    }

    /// Processa um bloco no lugar, sem alocar memória.
    pub fn processar(&mut self, bloco: &mut [f32]) {
        for a in bloco.iter_mut() { *a = self.amostra(*a); }
    }
}

impl Drop for DeslocadorDeTom { fn drop(&mut self) { self.buffer.zeroize(); } }

impl core::fmt::Debug for DeslocadorDeTom {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "DeslocadorDeTom(fator={:.3}, <voz oculta>)", self.fator) }
}

/// Buffer de áudio que se zera da memória ao ser descartado: a voz em claro não deixa rastro.
pub struct BufferDeAudio(pub Vec<f32>);
impl Drop for BufferDeAudio { fn drop(&mut self) { self.0.zeroize(); } }

impl core::fmt::Debug for BufferDeAudio {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "BufferDeAudio(<{} amostras ocultas>)", self.0.len()) }
}

/// Limitador suave contra estouro.
pub fn limitar(amostras: &mut [f32]) {
    for a in amostras.iter_mut() { *a = a.clamp(-1.0, 1.0); *a -= a.powi(3) / 3.0; }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn desligado_nao_altera_o_tom() {
        assert_eq!(Predefinicao::Desligado.fator(), 1.0);
    }

    #[test]
    fn grave_abaixa_e_agudo_levanta() {
        assert!(Predefinicao::Grave.fator() < 1.0);
        assert!(Predefinicao::Agudo.fator() > 1.0);
    }

    #[test]
    fn personalizado_e_limitado() {
        assert_eq!(Predefinicao::Personalizado(99).semitons(), 12);
        assert_eq!(Predefinicao::Personalizado(-99).semitons(), -12);
    }

    #[test]
    fn plano_tem_sobreposicao_de_75_por_cento() {
        let p = planejar(Predefinicao::Grave, 48_000);
        assert_eq!(p.tamanho_janela / p.avanco, 4);
    }

    fn frequencia_por_cruzamentos(sinal: &[f32], taxa: f32) -> f32 {
        let cruzamentos = sinal.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count();
        cruzamentos as f32 * taxa / sinal.len() as f32
    }

    fn seno(freq: f32, taxa: f32, n: usize) -> Vec<f32> {
        (0..n).map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / taxa).sin()).collect()
    }

    #[test]
    fn fator_um_preserva_a_frequencia() {
        let taxa = 16_000.0;
        let mut s = seno(200.0, taxa, 32_000);
        DeslocadorDeTom::novo(1.0, 1024).processar(&mut s);
        let f = frequencia_por_cruzamentos(&s[8000..], taxa);
        assert!((f - 200.0).abs() < 10.0, "medido {f}");
    }

    #[test]
    fn uma_oitava_acima_dobra_a_frequencia() {
        let taxa = 16_000.0;
        let mut s = seno(200.0, taxa, 32_000);
        DeslocadorDeTom::novo(2.0, 1024).processar(&mut s);
        let f = frequencia_por_cruzamentos(&s[8000..], taxa);
        assert!((f - 400.0).abs() < 40.0, "medido {f}");
    }

    #[test]
    fn grave_abaixa_a_frequencia_medida() {
        let taxa = 16_000.0;
        let mut s = seno(300.0, taxa, 32_000);
        DeslocadorDeTom::novo(Predefinicao::Grave.fator(), 1024).processar(&mut s);
        let f = frequencia_por_cruzamentos(&s[8000..], taxa);
        assert!(f < 260.0 && f > 190.0, "medido {f}");
    }

    #[test]
    fn cada_chamada_sorteia_um_tom_diferente() {
        let fatores: Vec<f32> = (0..8).map(|_| DeslocadorDeTom::para_chamada(Predefinicao::Grave, 1024, 1.0).fator()).collect();
        assert!(fatores.windows(2).any(|w| (w[0] - w[1]).abs() > 1e-4));
        assert!(fatores.iter().all(|f| *f > 0.5 && *f < 1.0));
    }

    #[test]
    fn saida_fica_dentro_da_faixa() {
        let mut s = seno(440.0, 48_000.0, 4800);
        DeslocadorDeTom::novo(1.3, 1024).processar(&mut s);
        assert!(s.iter().all(|v| v.abs() <= 1.01));
    }

    #[test]
    fn limitador_segura_em_um() {
        let mut a = [2.0f32, -2.0, 0.5];
        limitar(&mut a);
        assert!(a.iter().all(|v| v.abs() <= 1.0));
    }
}

// ===========================================================================================
// Nível 2 — conversão neural de voz (VM5).
//
// O que é, sem exagero: um modelo roda NO APARELHO e substitui a voz da pessoa por uma voz-alvo
// sintética fixa. Ao contrário do deslocamento de tom (Nível 1), não há "deslocar de volta": o timbre
// original não está mais no sinal. A voz-alvo é a mesma sempre, escolhida pelo usuário, e o modelo
// nunca sai do aparelho — a voz real jamais é enviada.
//
// Verdade que o app precisa declarar ao usuário (item VM5b do Manual):
//  * Sotaque, escolha de palavras, ritmo e hesitações continuam sendo da pessoa e ainda identificam.
//  * A conversão preserva o CONTEÚDO da fala; ela esconde o TIMBRE, não o que é dito.
//  * A qualidade depende do modelo embarcado; medir latência antes de habilitar em chamada.
//
// Este trecho traz a PARTE DETERMINÍSTICA e testável: seleção de voz-alvo, orçamento de latência,
// divisão em blocos com sobreposição para o modelo, e a garantia de que nada é retido. A inferência
// da rede em si é entregue pela camada nativa (Core ML no iOS, NNAPI/LiteRT no Android) — Rust não
// roda o modelo, apenas orquestra e protege a memória.

use zeroize::Zeroizing;

/// Voz-alvo sintética fixa. Não representa nenhuma pessoa real.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VozAlvo { Neutra, GraveCalma, ClaraAguda, Roboavatar }

impl VozAlvo {
    /// Identificador estável do conjunto de parâmetros do modelo (o "embedding" da voz-alvo).
    pub fn id(self) -> u16 {
        match self { VozAlvo::Neutra => 1, VozAlvo::GraveCalma => 2, VozAlvo::ClaraAguda => 3, VozAlvo::Roboavatar => 4 }
    }
}

/// Orçamento de latência de uma chamada. A conversa fica desconfortável acima de ~150 ms de ida.
#[derive(Clone, Copy, Debug)]
pub struct OrcamentoLatencia { pub bloco_ms: u32, pub inferencia_ms: u32, pub rede_ms: u32 }

impl OrcamentoLatencia {
    pub fn total_ida_ms(&self) -> u32 { self.bloco_ms + self.inferencia_ms + self.rede_ms }
    /// Aceitável para chamada em tempo real.
    pub fn aceitavel_para_chamada(&self) -> bool { self.total_ida_ms() <= 150 }
    /// Aceitável ao menos para mensagem de voz gravada (sem exigência de tempo real).
    pub fn aceitavel_para_mensagem(&self) -> bool { self.total_ida_ms() <= 1000 }
}

/// Configuração da conversão neural.
#[derive(Clone, Copy, Debug)]
pub struct ConfigNeural { pub alvo: VozAlvo, pub taxa: u32, pub bloco: usize, pub sobreposicao: usize }

impl ConfigNeural {
    pub fn novo(alvo: VozAlvo, taxa: u32) -> Self {
        // Bloco de 20 ms com 25% de sobreposição, padrão para modelos de voz em tempo real.
        let bloco = (taxa as usize / 50).max(64);
        ConfigNeural { alvo, taxa, bloco, sobreposicao: bloco / 4 }
    }
    pub fn avanco(&self) -> usize { self.bloco - self.sobreposicao }
}

/// Divide o áudio em blocos com sobreposição, para o modelo processar sem estalos nas junções.
pub fn blocos_com_sobreposicao(total_amostras: usize, cfg: &ConfigNeural) -> usize {
    if total_amostras <= cfg.bloco { return if total_amostras == 0 { 0 } else { 1 }; }
    1 + (total_amostras - cfg.bloco).div_ceil(cfg.avanco())
}

/// Sessão de conversão. Guarda o áudio de entrada em memória de **capacidade fixa**, que se zera ao
/// descartar. Capacidade fixa importa: um vetor que cresce realoca e deixa a cópia antiga da voz real
/// esquecida em memória liberada.
pub struct SessaoNeural {
    cfg: ConfigNeural,
    entrada: Zeroizing<Vec<f32>>,
    capacidade: usize,
    /// Quantas amostras de voz REAL já passaram (métrica interna; nunca sai do aparelho).
    processadas: u64,
}

impl core::fmt::Debug for SessaoNeural {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SessaoNeural(alvo={}, pendentes={}, <voz oculta>)", self.cfg.alvo.id(), self.entrada.len())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeVoz { SemEspaco }

impl SessaoNeural {
    /// Reserva espaço para 16 blocos de uma vez, e nunca mais cresce.
    pub fn nova(cfg: ConfigNeural) -> Self {
        let capacidade = cfg.bloco * 16;
        SessaoNeural { cfg, entrada: Zeroizing::new(Vec::with_capacity(capacidade)), capacidade, processadas: 0 }
    }

    pub fn id_alvo(&self) -> u16 { self.cfg.alvo.id() }
    pub fn amostras_processadas(&self) -> u64 { self.processadas }

    /// Recebe um bloco de voz real; devolve quantas amostras estão prontas para o modelo.
    /// Sem espaço, recusa em vez de crescer: a camada nativa deve consumir antes de alimentar de novo.
    pub fn alimentar(&mut self, bloco_real: &[f32]) -> Result<usize, ErroDeVoz> {
        if self.entrada.len() + bloco_real.len() > self.capacidade { return Err(ErroDeVoz::SemEspaco); }
        self.entrada.extend_from_slice(bloco_real);
        self.processadas += bloco_real.len() as u64;
        Ok((self.entrada.len() / self.cfg.avanco()) * self.cfg.avanco())
    }

    /// Consome as amostras já enviadas ao modelo: zera e desloca dentro do mesmo espaço (sem realocar).
    pub fn consumir(&mut self, amostras: usize) {
        let n = amostras.min(self.entrada.len());
        for x in self.entrada.iter_mut().take(n) { *x = 0.0; }
        let _ = self.entrada.drain(..n);
        // O deslocamento deixa cópias no espaço livre do fim: zerar também.
        self.entrada.spare_capacity_mut().zeroize();
    }

    pub fn pendentes(&self) -> usize { self.entrada.len() }
    pub fn capacidade(&self) -> usize { self.capacidade }
}

// ---- RV6: o disfarce só liga com acordo dos dois lados, por conversa e por tempo determinado ----

/// Por que isto existe: se o disfarce de voz ficasse sempre ligado e silencioso, alguém com o aparelho
/// destravado de outra pessoa (roubo, coação, esquecido destravado) poderia ligar para um contato dela e
/// se passar por ela — e o próprio disfarce, que o contato já espera ouvir, tiraria exatamente o sinal
/// ("a voz não é dela") que ajudaria a notar a troca. A correção: o disfarce só fica ativo quando os DOIS
/// lados aprovaram, só PARA aquela conversa, e só até um prazo — vencido, exige aprovação nova dos dois,
/// o que cria um momento visível em que uma troca de identidade tende a aparecer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AcordoDeDisfarce {
    /// Quando EU aprovei, nesta conversa (carimbo do relógio confiável). `None` = nunca aprovei (ou revoguei).
    pub eu_aprovei_em: Option<u64>,
    /// Quando O OUTRO LADO aprovou, confirmado por assinatura dele (não é algo que eu possa fingir).
    pub o_outro_aprovou_em: Option<u64>,
    /// Validade de cada aprovação. Pequena de propósito — isto não é uma preferência "para sempre".
    pub valido_por_segundos: u64,
}

/// Validade recomendada de cada aprovação: 24 horas. Curto o bastante para que o disfarce nunca fique
/// "esquecido ligado" por meses; longo o bastante para não irritar quem usa isto todo dia.
pub const VALIDADE_PADRAO_DO_ACORDO: u64 = 24 * 3_600;

impl AcordoDeDisfarce {
    /// O disfarce só pode ser aplicado nesta chamada/mensagem de voz se AMBAS as aprovações existirem e
    /// NENHUMA delas tiver vencido. Qualquer outro caso (ninguém aprovou, só um lado, ou uma venceu):
    /// a voz sai real, sem disfarce — "na dúvida, mostra a voz verdadeira", nunca o contrário.
    pub fn ativo(&self, agora: u64) -> bool {
        match (self.eu_aprovei_em, self.o_outro_aprovou_em) {
            (Some(eu), Some(outro)) => agora < eu.saturating_add(self.valido_por_segundos) && agora < outro.saturating_add(self.valido_por_segundos),
            _ => false,
        }
    }
}

#[cfg(test)]
mod testes_acordo_de_disfarce {
    use super::*;

    fn acordo(eu: Option<u64>, outro: Option<u64>) -> AcordoDeDisfarce {
        AcordoDeDisfarce { eu_aprovei_em: eu, o_outro_aprovou_em: outro, valido_por_segundos: VALIDADE_PADRAO_DO_ACORDO }
    }

    #[test]
    fn sem_os_dois_lados_nunca_fica_ativo() {
        assert!(!acordo(None, None).ativo(1_000));
        assert!(!acordo(Some(1_000), None).ativo(1_000), "só eu aprovei: não basta");
        assert!(!acordo(None, Some(1_000)).ativo(1_000), "só o outro aprovou: não basta");
    }

    #[test]
    fn com_os_dois_lados_fica_ativo_so_dentro_do_prazo() {
        let a = acordo(Some(1_000), Some(1_000));
        assert!(a.ativo(1_000 + VALIDADE_PADRAO_DO_ACORDO - 1));
        assert!(!a.ativo(1_000 + VALIDADE_PADRAO_DO_ACORDO), "vencido para os dois ao mesmo tempo");
    }

    #[test]
    fn uma_aprovacao_vencida_desliga_mesmo_que_a_outra_ainda_valha() {
        // Eu aprovei há muito tempo, o outro aprovou agora: minha aprovação venceu, então fica inativo —
        // cada lado precisa estar dentro do PRÓPRIO prazo, não basta um dos dois ser recente.
        let a = acordo(Some(0), Some(100_000));
        assert!(!a.ativo(VALIDADE_PADRAO_DO_ACORDO + 1));
    }

    #[test]
    fn vencido_exige_aprovacao_nova_dos_dois_nao_so_renovar_um_lado() {
        let vencido = acordo(Some(0), Some(0));
        assert!(!vencido.ativo(VALIDADE_PADRAO_DO_ACORDO + 1));
        // Só eu renovar não é o bastante — o outro lado precisa aprovar de novo também.
        let so_eu_renovei = AcordoDeDisfarce { eu_aprovei_em: Some(VALIDADE_PADRAO_DO_ACORDO + 1), ..vencido };
        assert!(!so_eu_renovei.ativo(VALIDADE_PADRAO_DO_ACORDO + 1));
    }
}

#[cfg(test)]
mod testes_neural {
    use super::*;

    #[test]
    fn latencia_de_chamada_dentro_do_orcamento() {
        let ok = OrcamentoLatencia { bloco_ms: 20, inferencia_ms: 25, rede_ms: 60 };
        assert!(ok.aceitavel_para_chamada());
        let ruim = OrcamentoLatencia { bloco_ms: 20, inferencia_ms: 200, rede_ms: 60 };
        assert!(!ruim.aceitavel_para_chamada());
        assert!(ruim.aceitavel_para_mensagem(), "ainda serve para mensagem de voz gravada");
    }

    #[test]
    fn cada_voz_alvo_tem_id_estavel_e_distinto() {
        let ids: Vec<u16> = [VozAlvo::Neutra, VozAlvo::GraveCalma, VozAlvo::ClaraAguda, VozAlvo::Roboavatar].iter().map(|v| v.id()).collect();
        let mut u = ids.clone(); u.sort(); u.dedup();
        assert_eq!(ids.len(), u.len(), "nenhum id repetido");
    }

    #[test]
    fn contagem_de_blocos_cobre_todo_o_audio() {
        let cfg = ConfigNeural::novo(VozAlvo::Neutra, 48_000);
        assert_eq!(blocos_com_sobreposicao(0, &cfg), 0);
        assert_eq!(blocos_com_sobreposicao(cfg.bloco, &cfg), 1);
        assert!(blocos_com_sobreposicao(cfg.bloco * 10, &cfg) >= 10);
    }

    #[test]
    fn a_voz_real_e_consumida_e_nao_se_acumula() {
        let cfg = ConfigNeural::novo(VozAlvo::GraveCalma, 48_000);
        let mut s = SessaoNeural::nova(cfg);
        let bloco = vec![0.3f32; cfg.bloco];
        let _ = s.alimentar(&bloco).unwrap();
        assert!(s.pendentes() > 0);
        s.consumir(s.pendentes());
        assert_eq!(s.pendentes(), 0, "nada de voz real fica retido");
    }

    #[test]
    fn sem_espaco_recusa_em_vez_de_crescer() {
        let cfg = ConfigNeural::novo(VozAlvo::Neutra, 16_000);
        let mut s = SessaoNeural::nova(cfg);
        let cap = s.capacidade();
        let bloco = vec![0.1f32; cfg.bloco];
        for _ in 0..16 { let _ = s.alimentar(&bloco).unwrap(); }
        assert_eq!(s.alimentar(&bloco), Err(ErroDeVoz::SemEspaco));
        assert_eq!(s.capacidade(), cap, "o espaço nunca cresce, então nunca realoca");
        assert!(format!("{s:?}").contains("voz oculta"));
    }

    #[test]
    fn sessao_so_expoe_o_id_da_voz_alvo_nunca_a_voz_real() {
        // Garantia de superfície: a API pública devolve o id do alvo e contadores, nunca amostras da voz real.
        let mut s = SessaoNeural::nova(ConfigNeural::novo(VozAlvo::Roboavatar, 16_000));
        let _ = s.alimentar(&vec![0.9f32; 320]).unwrap();
        assert_eq!(s.id_alvo(), VozAlvo::Roboavatar.id());
        assert_eq!(s.amostras_processadas(), 320);
        // Não existe getter que devolva o conteúdo de `entrada`: a voz real não tem saída pela API.
    }
}
