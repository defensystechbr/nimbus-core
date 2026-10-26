//! # Módulo `vigia_de_sensores` — vigia contínuo enquanto o app está em uso (VSE1–VSE8)
//!
//! O módulo `integridade` decide ao abrir o app; o `escudo` decide o que cada conversa mostra. Este módulo
//! decide **a cada instante de uso**, com leituras que a camada nativa (Swift/Kotlin) faz das interfaces
//! oficiais do sistema: microfone ou câmera em uso por outro app, tela gravada, espelhada ou capturada, tela
//! externa ligada, sobreposição, acessibilidade não confiável, teclado de terceiros, áudio saindo para outro
//! aparelho e app fora do primeiro plano.
//!
//! Princípio: **na dúvida, fecha**. Cada sinal tem quatro estados. "Não sei" fecha o que depende dele. Leitura
//! vencida ou vinda do futuro fecha tudo. O sistema que garante exclusividade (ex.: `FLAG_SECURE` faz a
//! captura sair preta) é declarado como tal pela camada nativa, nunca presumido aqui.
//!
//! Limites (declarados no Manual): o iPhone não informa se outro app usa o microfone ou a câmera; nele a
//! camada nativa só sabe quando o próprio áudio do Nimbus é interrompido. Câmera externa apontada para a tela
//! continua fora do alcance de qualquer app. Um aparelho com root pode mentir para a camada nativa — por isso
//! a integridade e a atestação continuam valendo antes deste vigia.

/// O que a camada nativa sabe sobre um sinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sinal {
    /// O risco está presente agora.
    Sim,
    /// Conferido: o risco não está presente.
    Nao,
    /// O sistema operacional impede o risco por construção (ex.: captura sai preta com FLAG_SECURE).
    SistemaImpede,
    /// Não foi possível saber. Tratado como risco (falha fechada).
    Desconhecido,
}

impl Sinal {
    fn arriscado(self) -> bool { matches!(self, Sinal::Sim | Sinal::Desconhecido) }
}

/// O que a pessoa está fazendo no Nimbus neste instante.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uso { Lendo, Chamada, GravandoVoz, UsandoCamera, Digitando }

/// Uma leitura dos sensores, feita pela camada nativa.
#[derive(Clone, Copy, Debug)]
pub struct Leitura {
    /// Segundos (relógio sincronizado com o servidor) em que a leitura foi feita.
    pub carimbo: u64,
    pub app_em_primeiro_plano: bool,
    pub microfone_de_outro_app: Sinal,
    pub camera_de_outro_app: Sinal,
    /// Tela gravada, espelhada ou transmitida.
    pub tela_capturada: Sinal,
    /// Uma captura de tela acabou de ser tirada (evento).
    pub captura_de_tela_tirada: bool,
    pub tela_externa_conectada: Sinal,
    pub sobreposicao: Sinal,
    pub acessibilidade_nao_confiavel: Sinal,
    pub teclado_de_terceiros: Sinal,
    /// Áudio saindo para um aparelho que não é o próprio celular (fone, carro, caixa de som, AirPlay).
    pub audio_para_aparelho_externo: Sinal,
}

/// Validade de uma leitura. A camada nativa lê de novo a cada mudança de sinal e, no mínimo, neste intervalo.
pub const VALIDADE_DA_LEITURA_SEGUNDOS: u64 = 5;
/// Tolerância para relógio adiantado.
pub const TOLERANCIA_DE_RELOGIO: u64 = 2;

/// O que o app faz agora. Tudo começa fechado; cada liberação precisa de todos os sinais limpos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decisao {
    pub mostrar_conteudo: bool,
    pub microfone: bool,
    pub camera: bool,
    /// Só o teclado do sistema pode ser usado (o de terceiros fica bloqueado nos campos do Nimbus).
    pub teclado_permitido: bool,
    /// Sinal de ataque ativo: tirar as chaves de sessão da memória e pedir o PIN de novo.
    pub bloquear_sessao: bool,
    /// Avisar os outros participantes pelo Escudo de Conversa.
    pub avisar_participantes: bool,
    pub motivos: Vec<&'static str>,
}

fn tudo_fechado(motivo: &'static str) -> Decisao {
    Decisao { mostrar_conteudo: false, microfone: false, camera: false, teclado_permitido: false,
              bloquear_sessao: false, avisar_participantes: false, motivos: vec![motivo] }
}

/// Decide o que o app pode fazer agora. VSE1–VSE8.
pub fn decidir(uso: Uso, l: &Leitura, agora: u64) -> Decisao {
    // VSE1: leitura vencida ou do futuro fecha tudo.
    if l.carimbo.saturating_add(VALIDADE_DA_LEITURA_SEGUNDOS) < agora { return tudo_fechado("leitura dos sensores vencida"); }
    if l.carimbo > agora.saturating_add(TOLERANCIA_DE_RELOGIO) { return tudo_fechado("leitura dos sensores com relógio adiantado"); }
    // VSE2: fora do primeiro plano nada aparece, nada grava, nada filma.
    if !l.app_em_primeiro_plano { return tudo_fechado("app fora do primeiro plano"); }

    let mut d = Decisao { mostrar_conteudo: true, microfone: false, camera: false, teclado_permitido: true,
                          bloquear_sessao: false, avisar_participantes: false, motivos: vec![] };
    // VSE3: captura, tela externa, sobreposição ou acessibilidade não confiável escondem o conteúdo.
    if l.tela_capturada.arriscado() { d.mostrar_conteudo = false; d.avisar_participantes = true; d.motivos.push("tela gravada, espelhada ou transmitida"); }
    if l.captura_de_tela_tirada { d.avisar_participantes = true; d.motivos.push("captura de tela tirada"); }
    if l.tela_externa_conectada.arriscado() { d.mostrar_conteudo = false; d.motivos.push("tela externa conectada"); }
    // VSE4: sobreposição e acessibilidade não confiável são sinais de ataque ativo: bloqueia a sessão.
    if l.sobreposicao.arriscado() { d.mostrar_conteudo = false; d.bloquear_sessao = true; d.motivos.push("sobreposição de tela"); }
    if l.acessibilidade_nao_confiavel.arriscado() { d.mostrar_conteudo = false; d.bloquear_sessao = true; d.motivos.push("acessibilidade não confiável"); }
    // VSE5: teclado de terceiros nunca recebe o que se digita no Nimbus.
    if l.teclado_de_terceiros.arriscado() { d.teclado_permitido = false; d.motivos.push("teclado de terceiros"); }
    // VSE6: microfone só no uso que pede microfone e só sem outro app gravando junto. Disputa de microfone
    // durante uma CHAMADA é mais grave que durante uma gravação de voz solitária: pode ser um app espião
    // escutando a ligação, e quem está do outro lado precisa saber que algo pode estar interferindo.
    if matches!(uso, Uso::Chamada | Uso::GravandoVoz) {
        if l.microfone_de_outro_app.arriscado() {
            d.motivos.push("outro app com o microfone");
            if uso == Uso::Chamada { d.avisar_participantes = true; }
        } else { d.microfone = true; }
        // VSE7: áudio indo para outro aparelho não bloqueia (fones são comuns), mas o app avisa sempre.
        if l.audio_para_aparelho_externo.arriscado() { d.motivos.push("áudio saindo para outro aparelho"); }
    }
    // VSE8: câmera só no uso que pede câmera, sem outro app com a câmera e sem a tela sendo capturada.
    // Disputa de câmera é sempre avisada: é o sinal mais direto de outro app tentando gravar a mesma coisa.
    if uso == Uso::UsandoCamera {
        if l.camera_de_outro_app.arriscado() { d.motivos.push("outro app com a câmera"); d.avisar_participantes = true; }
        else if l.tela_capturada.arriscado() { d.motivos.push("câmera fechada: tela capturada"); }
        else { d.camera = true; }
    }
    // Sessão bloqueada fecha tudo de uma vez (uma regra só, testada; a outra barreira é a camada nativa e o servidor).
    if d.bloquear_sessao { d.mostrar_conteudo = false; d.microfone = false; d.camera = false; d.motivos.push("sessão bloqueada: tudo fechado"); }
    d
}

#[cfg(test)]
mod testes {
    use super::*;

    fn limpa(t: u64) -> Leitura {
        Leitura { carimbo: t, app_em_primeiro_plano: true, microfone_de_outro_app: Sinal::Nao, camera_de_outro_app: Sinal::Nao,
                  tela_capturada: Sinal::SistemaImpede, captura_de_tela_tirada: false, tela_externa_conectada: Sinal::Nao,
                  sobreposicao: Sinal::Nao, acessibilidade_nao_confiavel: Sinal::Nao, teclado_de_terceiros: Sinal::Nao,
                  audio_para_aparelho_externo: Sinal::Nao }
    }

    #[test]
    fn tudo_limpo_libera_so_o_que_o_uso_pede() {
        let d = decidir(Uso::Lendo, &limpa(100), 100);
        assert!(d.mostrar_conteudo && d.teclado_permitido && !d.microfone && !d.camera && !d.bloquear_sessao, "{d:?}");
        assert!(decidir(Uso::Chamada, &limpa(100), 100).microfone);
        assert!(!decidir(Uso::Chamada, &limpa(100), 100).camera, "chamada de voz não liga a câmera");
        assert!(decidir(Uso::UsandoCamera, &limpa(100), 100).camera);
        assert!(!decidir(Uso::UsandoCamera, &limpa(100), 100).microfone, "câmera não liga o microfone sozinha");
    }

    #[test]
    fn leitura_vencida_ou_do_futuro_fecha_tudo() {
        let d = decidir(Uso::Chamada, &limpa(100), 100 + VALIDADE_DA_LEITURA_SEGUNDOS + 1);
        assert_eq!(d, tudo_fechado("leitura dos sensores vencida"));
        assert!(decidir(Uso::Chamada, &limpa(100), 100 + VALIDADE_DA_LEITURA_SEGUNDOS).microfone, "no limite ainda vale");
        let f = decidir(Uso::Lendo, &limpa(100 + TOLERANCIA_DE_RELOGIO + 1), 100);
        assert!(!f.mostrar_conteudo && !f.teclado_permitido);
        assert!(decidir(Uso::Lendo, &limpa(100 + TOLERANCIA_DE_RELOGIO), 100).mostrar_conteudo);
        assert!(!decidir(Uso::Lendo, &limpa(u64::MAX), 0).mostrar_conteudo, "sem estouro de conta");
    }

    #[test]
    fn fora_do_primeiro_plano_nada_aparece_nem_grava() {
        let mut l = limpa(100); l.app_em_primeiro_plano = false;
        for u in [Uso::Lendo, Uso::Chamada, Uso::GravandoVoz, Uso::UsandoCamera, Uso::Digitando] {
            let d = decidir(u, &l, 100);
            assert!(!d.mostrar_conteudo && !d.microfone && !d.camera && !d.teclado_permitido, "{u:?}");
        }
    }

    #[test]
    fn nao_sei_fecha_como_sim() {
        for s in [Sinal::Sim, Sinal::Desconhecido] {
            let mut l = limpa(100); l.tela_capturada = s;
            let d = decidir(Uso::Lendo, &l, 100);
            assert!(!d.mostrar_conteudo && d.avisar_participantes, "{s:?}");
            let mut l = limpa(100); l.microfone_de_outro_app = s;
            assert!(!decidir(Uso::Chamada, &l, 100).microfone, "{s:?}");
            let mut l = limpa(100); l.camera_de_outro_app = s;
            assert!(!decidir(Uso::UsandoCamera, &l, 100).camera, "{s:?}");
            let mut l = limpa(100); l.tela_externa_conectada = s;
            assert!(!decidir(Uso::Lendo, &l, 100).mostrar_conteudo, "{s:?}");
            let mut l = limpa(100); l.teclado_de_terceiros = s;
            assert!(!decidir(Uso::Digitando, &l, 100).teclado_permitido, "{s:?}");
        }
        let mut l = limpa(100); l.tela_capturada = Sinal::Nao;
        assert!(decidir(Uso::Lendo, &l, 100).mostrar_conteudo);
    }

    #[test]
    fn sobreposicao_e_acessibilidade_bloqueiam_a_sessao_e_fecham_sensores() {
        for campo in 0..2 {
            for s in [Sinal::Sim, Sinal::Desconhecido] {
                let mut l = limpa(100);
                if campo == 0 { l.sobreposicao = s } else { l.acessibilidade_nao_confiavel = s }
                for u in [Uso::Lendo, Uso::Chamada, Uso::UsandoCamera] {
                    let d = decidir(u, &l, 100);
                    assert!(d.bloquear_sessao && !d.mostrar_conteudo && !d.microfone && !d.camera, "{campo} {s:?} {u:?}");
                }
            }
        }
    }

    #[test]
    fn captura_de_tela_tirada_avisa_os_outros() {
        let mut l = limpa(100); l.captura_de_tela_tirada = true;
        let d = decidir(Uso::Lendo, &l, 100);
        assert!(d.avisar_participantes && d.motivos.contains(&"captura de tela tirada"));
    }

    #[test]
    fn camera_fecha_com_tela_capturada_e_audio_externo_so_avisa() {
        let mut l = limpa(100); l.tela_capturada = Sinal::Sim;
        assert!(!decidir(Uso::UsandoCamera, &l, 100).camera);
        let mut l = limpa(100); l.audio_para_aparelho_externo = Sinal::Sim;
        let d = decidir(Uso::Chamada, &l, 100);
        assert!(d.microfone && d.motivos.contains(&"áudio saindo para outro aparelho"));
    }

    /// ATK-D1 — disputa de microfone ou câmera com outro app é um sinal forte de espionagem; quem está do
    /// outro lado da chamada precisa ser avisado, não só ter o próprio acesso negado em silêncio.
    #[test]
    fn disputa_de_microfone_na_chamada_avisa_o_outro_participante() {
        let mut l = limpa(100); l.microfone_de_outro_app = Sinal::Sim;
        let d = decidir(Uso::Chamada, &l, 100);
        assert!(!d.microfone && d.avisar_participantes, "{d:?}");
    }

    #[test]
    fn disputa_de_microfone_fora_de_chamada_nao_precisa_avisar_ninguem() {
        // Gravando uma mensagem de voz sozinho: não há "outro participante" para avisar.
        let mut l = limpa(100); l.microfone_de_outro_app = Sinal::Sim;
        let d = decidir(Uso::GravandoVoz, &l, 100);
        assert!(!d.microfone && !d.avisar_participantes, "{d:?}");
    }

    #[test]
    fn disputa_de_camera_sempre_avisa() {
        let mut l = limpa(100); l.camera_de_outro_app = Sinal::Desconhecido;
        let d = decidir(Uso::UsandoCamera, &l, 100);
        assert!(!d.camera && d.avisar_participantes, "{d:?}");
    }
}
