//! # Módulo `autenticacao` — nenhuma rota do servidor sem prova (SRV1–SRV6) — lógica do servidor
//!
//! Três regras, nesta ordem:
//! 1. **Negar por padrão.** Só existem as rotas da tabela `ROTAS`, comparadas por igualdade exata de método e
//!    caminho. Qualquer outra coisa (caminho com `..`, `//`, `%2e`, maiúsculas, barra no fim) não existe.
//! 2. **Prova por recurso, não por pessoa.** Uma caixa de mensagens é aberta com a chave daquela caixa (o
//!    servidor só aprende "quem tem a chave desta caixa", nunca quem é a pessoa nem que duas caixas são da
//!    mesma pessoa). Só as rotas de conta (registrar ou revogar aparelho) usam a chave do aparelho. Depositar
//!    exige ficha anônima de licença, que não diz quem pagou.
//! 3. **Prova que não se reaproveita.** A assinatura cobre método, caminho, corpo, titular, horário e um
//!    número de uso único; fora da janela de 60 s ou repetida, é recusada. Recusa nunca diz "quase".
//!
//! Se mesmo assim uma porta abrir, o que existe atrás é só bloco cifrado com prazo (ver `NimbusOne_Checklist_de_Seguranca_do_Backend.md`).

use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha512};
use crate::limites::{Limitador, Limite};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Chave fictícia, gerada uma vez por processo, usada só para gastar o mesmo tempo quando o titular não existe.
fn chave_ficticia() -> &'static ChaveDeVerificacao {
    static FICTICIA: OnceLock<ChaveDeVerificacao> = OnceLock::new();
    FICTICIA.get_or_init(|| crate::assinatura::gerar().verificacao())
}

const CONTEXTO: &[u8] = b"NimbusOne/v1/pedido";
/// Janela de tolerância do horário do pedido, para mais ou para menos.
pub const JANELA_SEGUNDOS: u64 = 60;
/// Teto de números de uso único guardados; acima disso, o servidor recusa (falha fechada) até limpar os vencidos.
pub const MAXIMO_DE_NUMEROS_GUARDADOS: usize = 1_000_000;

/// Que prova a rota exige.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exigencia {
    /// Sem prova: só leitura de dado público e assinado na origem (hora assinada, chave de recebimento).
    Publica,
    /// Assinatura com a chave do recurso (a caixa de mensagens). Não revela a pessoa.
    Capacidade,
    /// Assinatura com a chave do aparelho: só para registrar e revogar aparelho.
    Aparelho,
    /// Ficha anônima de licença (pagamento sem identificação).
    Ficha,
}

/// Tipo de titular registrado. Chave de caixa não abre rota de aparelho e vice-versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDeTitular { Caixa, Aparelho }

/// Toda rota tem limite de uso obrigatório (`por_minuto`): não existe rota sem rate limit.
#[derive(Debug)]
pub struct Rota { pub metodo: &'static str, pub caminho: &'static str, pub exigencia: Exigencia, pub corpo_maximo: usize, pub por_minuto: u32 }

/// Teto de qualquer limite por rota; nenhuma rota pode ser mais permissiva que isto.
pub const LIMITE_MAXIMO_POR_MINUTO: u32 = 600;
/// Primeira espera depois de uma falha de prova; dobra a cada falha seguida (até 24 h).
pub const CASTIGO_INICIAL: u64 = 2;
/// Teto de chaves acompanhadas por rota no limitador (memória limitada).
pub const CHAVES_POR_ROTA: usize = 200_000;
/// Teto global (todas as rotas somadas) de tentativas com titular DESCONHECIDO por minuto — endurecimento
/// desta rodada (Rodada 23, 11/10/2026), ver `Porteiro::desconhecidos` abaixo para o porquê.
pub const TETO_TITULARES_DESCONHECIDOS_POR_MINUTO: u32 = 5_000;

/// Inventário completo das rotas (OWASP API9: nada de rota esquecida). O que não está aqui não existe.
pub const ROTAS: &[Rota] = &[
    Rota { metodo: "POST", caminho: "/v1/hora", exigencia: Exigencia::Publica, corpo_maximo: 32, por_minuto: 120 },
    Rota { metodo: "GET", caminho: "/v1/prechave", exigencia: Exigencia::Publica, corpo_maximo: 0, por_minuto: 120 },
    Rota { metodo: "POST", caminho: "/v1/convite/resgatar", exigencia: Exigencia::Publica, corpo_maximo: 32, por_minuto: 10 },
    Rota { metodo: "POST", caminho: "/v1/caixa/criar", exigencia: Exigencia::Ficha, corpo_maximo: 4096, por_minuto: 10 },
    Rota { metodo: "POST", caminho: "/v1/caixa/depositar", exigencia: Exigencia::Ficha, corpo_maximo: 64 * 1024 * 1024 + 8192, por_minuto: 120 },
    Rota { metodo: "POST", caminho: "/v1/caixa/retirar", exigencia: Exigencia::Capacidade, corpo_maximo: 64, por_minuto: 120 },
    Rota { metodo: "POST", caminho: "/v1/caixa/apagar", exigencia: Exigencia::Capacidade, corpo_maximo: 64, por_minuto: 120 },
    Rota { metodo: "POST", caminho: "/v1/prechave/publicar", exigencia: Exigencia::Aparelho, corpo_maximo: 16 * 1024, por_minuto: 10 },
    Rota { metodo: "POST", caminho: "/v1/aparelho/registrar", exigencia: Exigencia::Aparelho, corpo_maximo: 16 * 1024, por_minuto: 5 },
    Rota { metodo: "POST", caminho: "/v1/aparelho/revogar", exigencia: Exigencia::Aparelho, corpo_maximo: 64, por_minuto: 10 },
];

/// Procura a rota por igualdade exata. Sem prefixo, sem normalização, sem "parecido".
pub fn rota(metodo: &str, caminho: &str) -> Option<&'static Rota> {
    ROTAS.iter().find(|r| r.metodo == metodo && r.caminho == caminho)
}

/// Prova anexada ao pedido.
#[derive(Debug, Clone)]
pub struct Prova { pub titular: [u8; 32], pub carimbo: u64, pub numero: [u8; 16], pub assinatura: AssinaturaHibrida }

#[derive(Debug, PartialEq, Eq)]
pub enum Recusa { RotaInexistente, LimiteDeUso(u64), MemoriaDoLimiteCheia, CorpoGrandeDemais, SemProva, FichaInvalida, TitularDesconhecido, TipoErrado, ForaDaJanela, Repetido, AssinaturaInvalida, Sobrecarga }

#[derive(Debug, PartialEq, Eq)]
pub enum Autorizado { Publica, Ficha, Titular([u8; 32]) }

fn mensagem(metodo: &str, caminho: &str, corpo: &[u8], titular: &[u8; 32], carimbo: u64, numero: &[u8; 16]) -> Vec<u8> {
    let mut v = Vec::with_capacity(160);
    for parte in [metodo.as_bytes(), caminho.as_bytes()] {
        v.extend_from_slice(&(parte.len() as u32).to_be_bytes());
        v.extend_from_slice(parte);
    }
    v.extend_from_slice(&Sha512::digest(corpo));
    v.extend_from_slice(titular);
    v.extend_from_slice(&carimbo.to_be_bytes());
    v.extend_from_slice(numero);
    v
}

/// No aparelho: assina o pedido com a chave do recurso (ou do aparelho, nas rotas de conta).
pub fn provar(chave: &ChaveDeAssinatura, metodo: &str, caminho: &str, corpo: &[u8], agora: u64) -> Prova {
    let titular = chave.verificacao().impressao_digital();
    let mut numero = [0u8; 16];
    OsRng.fill_bytes(&mut numero);
    let assinatura = chave.assinar(CONTEXTO, &mensagem(metodo, caminho, corpo, &titular, agora, &numero)).expect("contexto fixo e curto");
    Prova { titular, carimbo: agora, numero, assinatura }
}

/// O porteiro do servidor: chaves registradas e números de uso único já vistos. Vive só em memória.
#[derive(Debug)]
pub struct Porteiro {
    titulares: HashMap<[u8; 32], (ChaveDeVerificacao, TipoDeTitular)>,
    vistos: HashMap<([u8; 32], [u8; 16]), u64>,
    limitadores: HashMap<&'static str, Limitador>,
    /// Freio agregado contra titular forjado em massa (Rodada 23, 11/10/2026) — ver `decidir` abaixo para
    /// o mecanismo e o motivo. Uma só chave fixa (`b"*"`), capacidade 1: não é um HashMap que um atacante
    /// possa crescer, é um contador de taxa.
    desconhecidos: Limitador,
}

impl Default for Porteiro {
    fn default() -> Self {
        Porteiro {
            titulares: HashMap::new(),
            vistos: HashMap::new(),
            limitadores: HashMap::new(),
            desconhecidos: Limitador::novo(TETO_TITULARES_DESCONHECIDOS_POR_MINUTO, 60, 1, 1),
        }
    }
}

impl Porteiro {
    /// Registra a chave de uma caixa (quando a caixa é criada) ou de um aparelho (vínculo aprovado).
    pub fn registrar(&mut self, chave: ChaveDeVerificacao, tipo: TipoDeTitular) -> [u8; 32] {
        let fp = chave.impressao_digital();
        let _ = self.titulares.insert(fp, (chave, tipo));
        fp
    }

    /// Revoga na hora: o próximo pedido já é recusado.
    pub fn revogar(&mut self, titular: &[u8; 32]) { let _ = self.titulares.remove(titular); }

    fn limpar(&mut self, agora: u64) { self.vistos.retain(|_, expira| *expira > agora); }

    /// Decide o pedido. `ficha_valida` vem do verificador de fichas anônimas (só nas rotas de ficha).
    /// `chave_de_limite`: o que identifica quem bate na rota sem identificar a pessoa — a ficha anônima ou
    /// o retransmissor de onde o pedido veio. O limite é aplicado ANTES de conferir a assinatura (que é
    /// cara): martelar a rota não consome processamento do servidor.
    ///
    /// **Aviso obrigatório ao chamador (endurecido na Rodada 23, 11/10/2026):** `chave_de_limite` NUNCA
    /// deve ser os bytes do `titular` de uma prova ainda não autenticada. O titular de uma prova forjada
    /// é de graça para o atacante variar (32 bytes aleatórios, sem custo), então usá-lo como chave de
    /// limite permitiria inflar `limitadores` com uma entrada nova por tentativa — exatamente o ataque que
    /// este módulo evita para `vistos` ("só pedido com assinatura válida ocupa espaço"), mas que não
    /// estava igualmente fechado aqui. Use sempre algo que custe caro para o atacante variar: o
    /// retransmissor/conexão de origem, ou (só depois de confirmado válido) o titular de uma prova já
    /// aceita. Como defesa em profundidade — para o caso de um chamador não seguir este aviso —, titular
    /// DESCONHECIDO em massa já é contido abaixo por um freio agregado (`desconhecidos`), independente da
    /// escolha de `chave_de_limite`.
    #[allow(clippy::too_many_arguments)]
    pub fn autorizar(&mut self, metodo: &str, caminho: &str, corpo: &[u8], prova: Option<&Prova>, ficha_valida: bool,
                     chave_de_limite: &[u8], agora: u64) -> Result<Autorizado, Recusa> {
        let r = rota(metodo, caminho).ok_or(Recusa::RotaInexistente)?;
        if corpo.len() > r.corpo_maximo { return Err(Recusa::CorpoGrandeDemais); }
        let por_minuto = r.por_minuto.clamp(1, LIMITE_MAXIMO_POR_MINUTO);
        let limitador = self.limitadores.entry(r.caminho).or_insert_with(|| Limitador::novo(por_minuto, 60, CASTIGO_INICIAL, CHAVES_POR_ROTA));
        match limitador.permitir(chave_de_limite, agora) {
            Ok(()) => {}
            Err(Limite::EsperarAte(t)) => return Err(Recusa::LimiteDeUso(t)),
            Err(Limite::Cheio) => return Err(Recusa::MemoriaDoLimiteCheia),
        }
        let resultado = self.decidir(r, metodo, caminho, corpo, prova, ficha_valida, agora);
        if let Some(l) = self.limitadores.get_mut(r.caminho) {
            match &resultado {
                Err(Recusa::AssinaturaInvalida | Recusa::TitularDesconhecido | Recusa::TipoErrado | Recusa::FichaInvalida | Recusa::Repetido | Recusa::ForaDaJanela) => { let _ = l.registrar_falha(chave_de_limite, agora); }
                Ok(_) => l.registrar_sucesso(chave_de_limite),
                _ => {}
            }
        }
        resultado
    }

    #[allow(clippy::too_many_arguments)]
    fn decidir(&mut self, r: &'static Rota, metodo: &str, caminho: &str, corpo: &[u8], prova: Option<&Prova>, ficha_valida: bool,
               agora: u64) -> Result<Autorizado, Recusa> {
        let tipo_exigido = match r.exigencia {
            Exigencia::Publica => return Ok(Autorizado::Publica),
            Exigencia::Ficha => return if ficha_valida { Ok(Autorizado::Ficha) } else { Err(Recusa::FichaInvalida) },
            Exigencia::Capacidade => TipoDeTitular::Caixa,
            Exigencia::Aparelho => TipoDeTitular::Aparelho,
        };
        let p = prova.ok_or(Recusa::SemProva)?;
        // Tempo uniforme (ataque vermelho B1): a assinatura é SEMPRE conferida — contra a chave do titular, ou
        // contra uma chave fictícia quando o titular não existe —, e só depois se decide o motivo. Assim o
        // tempo da recusa não revela se uma caixa ou um aparelho estão registrados.
        let registro = self.titulares.get(&p.titular);
        let titular_conhecido = registro.is_some();
        let chave = registro.map(|(k, _)| k).unwrap_or_else(|| chave_ficticia());
        let assinatura_ok = chave.verificar(CONTEXTO, &mensagem(metodo, caminho, corpo, &p.titular, p.carimbo, &p.numero), &p.assinatura);
        let janela_ok = !(p.carimbo.saturating_add(JANELA_SEGUNDOS) < agora || p.carimbo > agora.saturating_add(JANELA_SEGUNDOS));
        // Freio agregado (Rodada 23): titular desconhecido custa zero para forjar, então a TAXA GLOBAL de
        // tentativas com titular desconhecido (de qualquer um, em qualquer rota) tem um teto próprio,
        // separado da chave de limite escolhida pelo chamador — ver o aviso acima em `autorizar`. Isto
        // corre ANTES do `let Some(...) else` abaixo para nunca deixar passar uma tentativa só porque o
        // freio ainda não foi conferido; e corre DEPOIS da assinatura (que já rodou, sempre, por tempo
        // uniforme) para não mudar o perfil de tempo da recusa.
        if !titular_conhecido && self.desconhecidos.permitir(b"*", agora).is_err() {
            return Err(Recusa::Sobrecarga);
        }
        let Some((_, tipo)) = registro else { return Err(Recusa::TitularDesconhecido) };
        if *tipo != tipo_exigido { return Err(Recusa::TipoErrado); }
        if !janela_ok { return Err(Recusa::ForaDaJanela); }
        if !assinatura_ok { return Err(Recusa::AssinaturaInvalida); }
        // Só pedido com assinatura válida ocupa espaço na memória de números vistos: quem forja não enche o cache.
        if self.vistos.contains_key(&(p.titular, p.numero)) { return Err(Recusa::Repetido); }
        if self.vistos.len() >= MAXIMO_DE_NUMEROS_GUARDADOS { self.limpar(agora); }
        if self.vistos.len() >= MAXIMO_DE_NUMEROS_GUARDADOS { return Err(Recusa::Sobrecarga); }
        let _ = self.vistos.insert((p.titular, p.numero), p.carimbo.saturating_add(2 * JANELA_SEGUNDOS));
        Ok(Autorizado::Titular(p.titular))
    }
}

// ---------------------------------------------------------------------------------------------------------
// Resposta pública e cabeçalhos (OWASP: enumeração, CORS, clickjacking, token na URL, dado demais na resposta)
// ---------------------------------------------------------------------------------------------------------

/// O que o servidor devolve ao mundo. Toda falha de autorização vira a MESMA resposta: quem tenta não
/// descobre se a caixa existe, se a chave é conhecida, se o horário estava errado ou se repetiu o pedido.
/// O corpo nunca traz dado além do necessário (nada de identificador, resumo, chave ou detalhe interno).
pub fn resposta_publica(r: &Result<Autorizado, Recusa>) -> (u16, &'static str) {
    match r {
        Ok(_) => (200, ""),
        Err(Recusa::RotaInexistente) => (404, "nao encontrado"),
        Err(Recusa::CorpoGrandeDemais) => (413, "grande demais"),
        Err(Recusa::LimiteDeUso(_)) => (429, "tente mais tarde"),
        Err(Recusa::MemoriaDoLimiteCheia | Recusa::Sobrecarga) => (503, "tente mais tarde"),
        Err(Recusa::SemProva | Recusa::FichaInvalida | Recusa::TitularDesconhecido | Recusa::TipoErrado
            | Recusa::ForaDaJanela | Recusa::Repetido | Recusa::AssinaturaInvalida) => (401, "nao autorizado"),
    }
}

/// Cabeçalhos de toda resposta da API. Não existe CORS (nenhum cabeçalho Access-Control-*): navegador
/// nenhum lê a API de outra origem. Nada é guardado em cache, nada pode ser exibido dentro de moldura.
pub fn cabecalhos_da_api() -> Vec<(&'static str, &'static str)> {
    vec![
        ("cache-control", "no-store"),
        ("content-type", "application/octet-stream"),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("content-security-policy", "default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'"),
        ("referrer-policy", "no-referrer"),
        ("cross-origin-resource-policy", "same-origin"),
        ("strict-transport-security", "max-age=63072000; includeSubDomains; preload"),
    ]
}

/// Origem do pedido. Os apps nativos não mandam o cabeçalho Origin; navegador manda. Desde 06/10/2026 não existe
/// nenhuma página web do Nimbus (decisão do dono: envio só dentro do app; quem recebe instala o app), então a API
/// recusa TODO pedido que venha de um navegador, de qualquer origem.
pub fn origem_permitida(origem: Option<&str>) -> bool {
    origem.is_none()
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    fn porteiro_com(tipo: TipoDeTitular) -> (Porteiro, ChaveDeAssinatura) {
        let k = assinatura::gerar();
        let mut p = Porteiro::default();
        let _ = p.registrar(k.verificacao(), tipo);
        (p, k)
    }

    #[test]
    fn negar_por_padrao_e_caminhos_tortos() {
        let mut p = Porteiro::default();
        for (m, c) in [("POST", "/v1/caixa/retirar/"), ("POST", "/v1//caixa/retirar"), ("POST", "/v1/caixa/../aparelho/revogar"),
                       ("POST", "/V1/caixa/retirar"), ("POST", "/v1/caixa/%72etirar"), ("GET", "/v1/caixa/retirar"), ("POST", "/admin"), ("DELETE", "/v1/caixa/apagar")] {
            assert_eq!(p.autorizar(m, c, b"", None, true, b"origem-1", 1_000), Err(Recusa::RotaInexistente), "{m} {c}");
        }
        assert_eq!(p.autorizar("POST", "/v1/hora", &[0; 32], None, false, b"origem-2", 1_000), Ok(Autorizado::Publica));
        assert_eq!(p.autorizar("POST", "/v1/hora", &[0; 33], None, false, b"origem-3", 1_000), Err(Recusa::CorpoGrandeDemais));
    }

    #[test]
    fn rota_protegida_exige_prova_valida() {
        let (mut p, k) = porteiro_com(TipoDeTitular::Caixa);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"x", None, false, b"origem-4", 1_000), Err(Recusa::SemProva));
        let prova = provar(&k, "POST", "/v1/caixa/retirar", b"x", 1_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&prova), false, b"origem-5", 1_010), Ok(Autorizado::Titular(k.verificacao().impressao_digital())));
        // A mesma prova de novo: repetida.
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&prova), false, b"origem-6", 1_011), Err(Recusa::Repetido));
        // Prova de uma rota usada em outra; corpo trocado: assinatura não confere.
        let outra = provar(&k, "POST", "/v1/caixa/retirar", b"x", 1_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/apagar", b"x", Some(&outra), false, b"origem-7", 1_010), Err(Recusa::AssinaturaInvalida));
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"y", Some(&outra), false, b"origem-8", 1_010), Err(Recusa::AssinaturaInvalida));
        // Fora da janela de 60 s, para trás e para frente.
        let velha = provar(&k, "POST", "/v1/caixa/retirar", b"x", 1_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&velha), false, b"origem-9", 1_061), Err(Recusa::ForaDaJanela));
        // Exatamente 60 s depois do carimbo (o limite em si) ainda é DENTRO da janela — limite incluído, não
        // excluído. Até a Rodada 23 nada testava esta borda exata (só carimbo+61, já fora); o cargo-mutants
        // achou que trocar o `<` desta conta por `<=` sobrevivia, porque só muda o resultado bem neste ponto.
        // `velha` ainda não foi usada com sucesso (a tentativa acima foi recusada por janela, não gastou o
        // número de uso único), então reaproveitá-la aqui não conflita com proteção a repetição.
        assert!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&velha), false, b"origem-9b", 1_060).is_ok(), "carimbo+60 é a borda da janela e deveria ainda ser aceito");
        let futura = provar(&k, "POST", "/v1/caixa/retirar", b"x", 2_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&futura), false, b"origem-10", 1_939), Err(Recusa::ForaDaJanela));
        assert!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&futura), false, b"origem-11", 1_940).is_ok());
    }

    #[test]
    fn chave_de_caixa_nao_abre_rota_de_aparelho_e_revogada_nao_entra() {
        let (mut p, k) = porteiro_com(TipoDeTitular::Caixa);
        let prova = provar(&k, "POST", "/v1/aparelho/revogar", b"", 1_000);
        assert_eq!(p.autorizar("POST", "/v1/aparelho/revogar", b"", Some(&prova), false, b"origem-12", 1_000), Err(Recusa::TipoErrado));
        let estranho = assinatura::gerar();
        let prova2 = provar(&estranho, "POST", "/v1/caixa/retirar", b"", 1_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&prova2), false, b"origem-13", 1_000), Err(Recusa::TitularDesconhecido));
        p.revogar(&k.verificacao().impressao_digital());
        let prova3 = provar(&k, "POST", "/v1/caixa/retirar", b"", 1_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&prova3), false, b"origem-14", 1_000), Err(Recusa::TitularDesconhecido));
    }

    #[test]
    fn titular_forjado_com_assinatura_de_outro_e_recusado() {
        let (mut p, k) = porteiro_com(TipoDeTitular::Caixa);
        let atacante = assinatura::gerar();
        let mut prova = provar(&atacante, "POST", "/v1/caixa/retirar", b"", 1_000);
        prova.titular = k.verificacao().impressao_digital();
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&prova), false, b"origem-15", 1_000), Err(Recusa::AssinaturaInvalida));
    }

    #[test]
    fn deposito_exige_ficha_e_inventario_nao_tem_duplicata() {
        let mut p = Porteiro::default();
        assert_eq!(p.autorizar("POST", "/v1/caixa/depositar", b"bloco", None, false, b"origem-16", 1), Err(Recusa::FichaInvalida));
        assert_eq!(p.autorizar("POST", "/v1/caixa/depositar", b"bloco", None, true, b"origem-17", 1), Ok(Autorizado::Ficha));
        for (i, a) in ROTAS.iter().enumerate() {
            for b in &ROTAS[i + 1..] { assert!(!(a.metodo == b.metodo && a.caminho == b.caminho), "rota duplicada"); }
            assert!(a.caminho.starts_with("/v1/") && !a.caminho.ends_with('/') && !a.caminho.contains(".."));
        }
    }

    #[test]
    fn tabela_de_rotas_tem_exatamente_os_numeros_documentados() {
        // Trava o valor FINAL de cada campo de ROTAS, não a fórmula de onde ele vem: achado real do
        // cargo-mutants (Rodada 23, 11/10/2026) em `corpo_maximo` — nenhum teste existente fixava o
        // resultado das contas (`64 * 1024 * 1024 + 8192`, `16 * 1024`), então um mutante que trocasse
        // `*` por `+`/`/` ou `+` por `-`/`*` nessas linhas sobrevivia: o número final mudava (de 64 MiB +
        // 8 KiB para, por exemplo, exatos 64 MiB, ou para 8 KiB) e nenhum teste reprovava. Os números
        // abaixo são fixos e vieram de conta feita por fora (python3 -c "64*1024*1024+8192"), não repetem
        // a expressão do código-fonte — então uma mutação na tabela não pode "se cancelar" com uma
        // mutação equivalente aqui, porque aqui não há nenhuma conta para mutar.
        assert_eq!(ROTAS.len(), 10, "rota nova ou removida: atualize a lista `esperado` abaixo também");
        let esperado: &[(&str, &str, Exigencia, usize, u32)] = &[
            ("POST", "/v1/hora", Exigencia::Publica, 32, 120),
            ("GET", "/v1/prechave", Exigencia::Publica, 0, 120),
            ("POST", "/v1/convite/resgatar", Exigencia::Publica, 32, 10),
            ("POST", "/v1/caixa/criar", Exigencia::Ficha, 4_096, 10),
            ("POST", "/v1/caixa/depositar", Exigencia::Ficha, 67_117_056, 120), // 64 MiB + 8 KiB
            ("POST", "/v1/caixa/retirar", Exigencia::Capacidade, 64, 120),
            ("POST", "/v1/caixa/apagar", Exigencia::Capacidade, 64, 120),
            ("POST", "/v1/prechave/publicar", Exigencia::Aparelho, 16_384, 10), // 16 KiB
            ("POST", "/v1/aparelho/registrar", Exigencia::Aparelho, 16_384, 5), // 16 KiB
            ("POST", "/v1/aparelho/revogar", Exigencia::Aparelho, 64, 10),
        ];
        for (r, (metodo, caminho, exigencia, corpo_maximo, por_minuto)) in ROTAS.iter().zip(esperado.iter()) {
            assert_eq!(r.metodo, *metodo, "{}: método mudou", r.caminho);
            assert_eq!(r.caminho, *caminho, "ordem ou caminho da tabela ROTAS mudou");
            assert_eq!(r.exigencia, *exigencia, "{}: exigência mudou", r.caminho);
            assert_eq!(r.corpo_maximo, *corpo_maximo, "{}: corpo_maximo mudou de valor", r.caminho);
            assert_eq!(r.por_minuto, *por_minuto, "{}: por_minuto mudou de valor", r.caminho);
        }
    }

    #[test]
    fn faxina_roda_exatamente_quando_a_memoria_de_vistos_enche_e_libera_espaco() {
        // Achado real do cargo-mutants (Rodada 23, 11/10/2026): trocar o `>=` desta condição por `<`
        // sobrevivia. Mutado, a faxina (`limpar`) deixaria de rodar bem na hora em que a memória de
        // números vistos enche — e aí, mesmo havendo entradas já vencidas para liberar, o servidor
        // recusaria com `Sobrecarga` para sempre, sem nunca tentar limpar nada. Preenche `vistos` direto
        // (campo privado, visível aqui por `testes` ser submódulo de `autenticacao`) até o teto, todas já
        // vencidas, para não gastar tempo com 1.000.000 assinaturas de verdade.
        let (mut p, k) = porteiro_com(TipoDeTitular::Caixa);
        for i in 0..MAXIMO_DE_NUMEROS_GUARDADOS as u64 {
            let mut titular = [0u8; 32];
            titular[..8].copy_from_slice(&i.to_be_bytes());
            let mut numero = [0u8; 16];
            numero[..8].copy_from_slice(&i.to_be_bytes());
            let _ = p.vistos.insert((titular, numero), 1); // vence em 1: já vencida bem antes de agora=1_000
        }
        assert_eq!(p.vistos.len(), MAXIMO_DE_NUMEROS_GUARDADOS);
        let prova = provar(&k, "POST", "/v1/caixa/retirar", b"x", 1_000);
        assert!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&prova), false, b"origem-cheio", 1_000).is_ok(),
            "memória cheia de entradas JÁ VENCIDAS deveria ser limpa e liberar espaço, não recusar com Sobrecarga");
    }

    #[test]
    fn numero_visto_fica_guardado_pelo_dobro_exato_da_janela() {
        // Acesso direto a `vistos` (privado, visível aqui). Achado do cargo-mutants (Rodada 23,
        // 11/10/2026): a conta `2 * JANELA_SEGUNDOS` em `decidir` — por quanto tempo um número de uso
        // único fica guardado depois de aceito — não tinha nenhum teste lendo o valor realmente gravado;
        // trocar `*` por `+` ou por `/` sobrevivia, e os dois ENCURTAM essa memória (de 120 s para 62 s
        // ou para 0 s), encurtando a proteção contra reapresentação do mesmo número antes do previsto.
        let (mut p, k) = porteiro_com(TipoDeTitular::Caixa);
        let prova = provar(&k, "POST", "/v1/caixa/retirar", b"x", 1_000);
        assert!(p.autorizar("POST", "/v1/caixa/retirar", b"x", Some(&prova), false, b"origem-visto", 1_000).is_ok());
        assert_eq!(p.vistos.get(&(prova.titular, prova.numero)).copied(), Some(1_120),
            "deveria ficar guardado até carimbo + 2×60s = 1120, não até 1062 (+) nem 1000 (/)");
    }

    #[test]
    fn limpar_remove_so_os_vencidos_e_preserva_o_resto() {
        // Acesso direto a `vistos`/`limpar` (ambos privados, mas visíveis aqui: `testes` é submódulo de
        // `autenticacao`). Sem isto só um teste que encha `vistos` de verdade até 1.000.000 de entradas
        // exercitaria `limpar` — caro e lento, então nenhum teste existente fazia isso. Achado real do
        // cargo-mutants (Rodada 23, 11/10/2026): sem este teste sobreviviam trocar `limpar` por uma função
        // vazia (não remove nada) e trocar `>` por `==`/`<`/`>=` na comparação de validade — ou seja, nada
        // detectava se a faxina da memória de números vistos parasse de funcionar. Isso é sério: com
        // `limpar` quebrado, depois de `vistos` encher uma única vez o servidor recusaria TODO pedido com
        // `Recusa::Sobrecarga` para sempre (linha 223-224 abaixo), mesmo muito depois de todas as entradas
        // antigas terem vencido — negação de serviço permanente a partir de um único pico de uso legítimo.
        let mut p = Porteiro::default();
        p.vistos.insert(([1; 32], [1; 16]), 100); // vence exatamente em `agora` — deve ser removida
        p.vistos.insert(([2; 32], [2; 16]), 101); // vence 1s depois de `agora` — deve sobreviver
        p.vistos.insert(([3; 32], [3; 16]), 1_000); // bem no futuro — deve sobreviver
        p.limpar(100);
        assert_eq!(p.vistos.len(), 2, "limpar(100) deveria remover exatamente a entrada vencida em 100");
        assert!(!p.vistos.contains_key(&([1; 32], [1; 16])), "expira==agora é vencida e deveria ter sido removida");
        assert!(p.vistos.contains_key(&([2; 32], [2; 16])), "expira==agora+1 ainda não venceu");
        assert!(p.vistos.contains_key(&([3; 32], [3; 16])));
    }

    #[test]
    fn sucesso_zera_o_castigo_acumulado_para_a_mesma_chave_de_limite() {
        // Achado real do cargo-mutants (Rodada 23, 11/10/2026): apagar o ramo `Ok(_) => l.registrar_sucesso(...)`
        // dentro de `autorizar` sobrevivia — nenhum teste existente detectava se um pedido bem-sucedido
        // deixava de perdoar o castigo acumulado por falhas anteriores da MESMA chave de limite. Sem isto,
        // uma chave legítima que falhasse uma vez (relógio um pouco dessincronizado, pacote repetido) e na
        // sequência tivesse um pedido válido continuaria, depois, sendo castigada como se aquele perdão
        // nunca tivesse existido: a próxima falha dobraria a espera a partir de onde a anterior parou, em
        // vez de recomeçar do castigo inicial. É um bug de disponibilidade para o usuário legítimo, não uma
        // brecha de segurança — mas o pedido foi para não ignorar risco real, e este é verificável e real.
        let (mut p, k) = porteiro_com(TipoDeTitular::Caixa);
        let ruim = provar(&assinatura::gerar(), "POST", "/v1/caixa/retirar", b"", 1_000); // titular não registrado
        let boa = provar(&k, "POST", "/v1/caixa/retirar", b"", 1_002);
        // 1ª falha em 1000: castigo inicial (2 s) bloqueia a chave "chave-sucesso" até 1002.
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&ruim), false, b"chave-sucesso", 1_000), Err(Recusa::TitularDesconhecido));
        // Sucesso em 1002 (exatamente quando o castigo anterior libera), mesma chave de limite.
        assert!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&boa), false, b"chave-sucesso", 1_002).is_ok());
        // 2ª falha, logo após o sucesso: se o sucesso zerou as falhas, o novo castigo é o INICIAL (até
        // 1004) — não o castigo em dobro (até 1006) que valeria se a falha anterior nunca tivesse sido
        // perdoada. A prova em si nem chega a ser olhada quando já castigado, então `None` basta aqui.
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&ruim), false, b"chave-sucesso", 1_002), Err(Recusa::TitularDesconhecido));
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", None, false, b"chave-sucesso", 1_003), Err(Recusa::LimiteDeUso(1_004)));
    }

    #[test]
    fn toda_rota_tem_limite_e_martelar_nao_custa_processamento() {
        for r in ROTAS { assert!(r.por_minuto >= 1 && r.por_minuto <= LIMITE_MAXIMO_POR_MINUTO, "{} sem limite válido", r.caminho); }
        let (mut p, k) = porteiro_com(TipoDeTitular::Caixa);
        // 10 resgates de convite por minuto por retransmissor; o 11º é recusado antes de qualquer outra conta.
        for _ in 0..10 { assert!(p.autorizar("POST", "/v1/convite/resgatar", &[1; 32], None, false, b"relay-1", 1_000).is_ok()); }
        assert!(matches!(p.autorizar("POST", "/v1/convite/resgatar", &[1; 32], None, false, b"relay-1", 1_000), Err(Recusa::LimiteDeUso(_))));
        assert!(p.autorizar("POST", "/v1/convite/resgatar", &[1; 32], None, false, b"relay-2", 1_000).is_ok(), "outro retransmissor não é afetado");
        // Prova inválida castiga: a próxima tentativa, mesmo válida, espera.
        let ruim = provar(&assinatura::gerar(), "POST", "/v1/caixa/retirar", b"", 2_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&ruim), false, b"atacante", 2_000), Err(Recusa::TitularDesconhecido));
        let boa = provar(&k, "POST", "/v1/caixa/retirar", b"", 2_000);
        assert!(matches!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&boa), false, b"atacante", 2_000), Err(Recusa::LimiteDeUso(_))));
        assert!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&boa), false, b"atacante", 2_002).is_ok());
    }


    #[test]
    fn falha_de_autorizacao_nao_conta_o_motivo() {
        let todas = [Recusa::SemProva, Recusa::FichaInvalida, Recusa::TitularDesconhecido, Recusa::TipoErrado, Recusa::ForaDaJanela, Recusa::Repetido, Recusa::AssinaturaInvalida];
        for r in todas { assert_eq!(resposta_publica(&Err(r)), (401, "nao autorizado")); }
        assert_eq!(resposta_publica(&Err(Recusa::LimiteDeUso(5))), (429, "tente mais tarde"));
        assert_eq!(resposta_publica(&Ok(Autorizado::Publica)), (200, ""));
    }

    #[test]
    fn titular_desconhecido_em_massa_e_contido_sem_esgotar_a_memoria() {
        // Rodada 23: um atacante que varia o titular a cada tentativa (de graça, 32 bytes à vontade) E
        // varia `chave_de_limite` a cada tentativa (o PIOR caso: nem um retransmissor fixo) não deveria
        // conseguir crescer `limitadores` sem controle. Reusa uma prova/assinatura real (não importa que
        // não bata com o titular sorteado: titular desconhecido já recusa antes de a assinatura decidir
        // qualquer coisa) para não pagar o custo de assinar 20.000 vezes num teste.
        let mut p = Porteiro::default();
        let base = provar(&assinatura::gerar(), "POST", "/v1/caixa/retirar", b"", 1_000);
        let mut viu_sobrecarga = false;
        for i in 0..20_000u32 {
            let mut prova = base.clone();
            prova.titular[..4].copy_from_slice(&i.to_be_bytes());
            let chave = i.to_be_bytes();
            match p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&prova), false, &chave, 1_000) {
                Err(Recusa::Sobrecarga) => { viu_sobrecarga = true; break; }
                Err(Recusa::TitularDesconhecido) => {}
                outro => panic!("inesperado: {outro:?}"),
            }
        }
        assert!(viu_sobrecarga, "o freio agregado deveria entrar em ação bem antes de 20.000 titulares forjados");
        assert!(p.limitadores.get("/v1/caixa/retirar").map(|l| l.acompanhadas()).unwrap_or(0) < TETO_TITULARES_DESCONHECIDOS_POR_MINUTO as usize + 10,
            "o limitador principal não deveria crescer muito além do próprio teto do freio agregado só por titulares forjados");
    }

    #[test]
    fn titular_desconhecido_isolado_continua_passando_normalmente() {
        // O freio agregado não deve incomodar o caso comum: uma única tentativa isolada (ex.: aparelho
        // revogado tentando de novo) continua recusada por TitularDesconhecido, não por Sobrecarga.
        let mut p = Porteiro::default();
        let prova = provar(&assinatura::gerar(), "POST", "/v1/caixa/retirar", b"", 1_000);
        assert_eq!(p.autorizar("POST", "/v1/caixa/retirar", b"", Some(&prova), false, b"origem-unica", 1_000), Err(Recusa::TitularDesconhecido));
    }

    #[test]
    fn sem_cors_sem_moldura_sem_token_na_url() {
        let c = cabecalhos_da_api();
        assert!(c.iter().all(|(k, _)| !k.starts_with("access-control-")), "nenhum cabeçalho de CORS");
        assert!(c.contains(&("x-frame-options", "DENY")) && c.iter().any(|(k, v)| *k == "content-security-policy" && v.contains("frame-ancestors 'none'")));
        assert!(c.contains(&("cache-control", "no-store")));
        assert!(origem_permitida(None), "app nativo não manda Origin");
        for qualquer in ["https://envio.exemplo-oficial", "https://evil.example", "null", "*", ""] {
            assert!(!origem_permitida(Some(qualquer)), "nenhum navegador fala com a API: {qualquer}");
        }
        assert!(ROTAS.iter().all(|r| !r.caminho.contains("envio")), "não existe rota de link para navegador");
        // Token ou qualquer coisa na URL: a rota simplesmente não existe.
        let mut p = Porteiro::default();
        for c in ["/v1/caixa/retirar?token=abc", "/v1/caixa/retirar#x", "/v1/caixa/retirar;jsessionid=1"] {
            assert_eq!(p.autorizar("POST", c, b"", None, false, b"origem-url", 1_000), Err(Recusa::RotaInexistente));
        }
    }

}
