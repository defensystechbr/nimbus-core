//! # Módulo `retirada_de_aparelho` — destruição total do aparelho ANTIGO após troca voluntária (novo, 10/10/2026)
//!
//! Resposta ao pedido do dono: depois de habilitar um aparelho novo (regra 35 do contrato da ponte —
//! `custodia_mutua::preparar_pacote_de_transferencia`/`abrir_pacote_de_transferencia`, que já move a aba
//! trancada e a chave local do cofre pessoal do aparelho antigo para o novo), o dono quer que o aparelho
//! ANTIGO sofra "destruição total": as conversas "nunca mais poderão ser lidas por nada, nem ninguém e
//! nem com o melhor equipamento de intrusão do mundo".
//!
//! **O que isto NÃO pode ser, dito sem meias-palavras (ver a resposta completa ao dono sobre a mesma
//! pergunta — "tem como exigir que o app se apague"):** nenhum app consegue apagar a si mesmo do sistema
//! operacional, nem no iOS nem no Android. Isto não é uma limitação do Nimbus — é uma proteção deliberada
//! da plataforma: um app com poder de se autodesinstalar (ou de desinstalar OUTRO app) seria, ele mesmo,
//! uma ferramenta de controle remoto de aparelho, exatamente o tipo de poder que um spyware quer ter e que
//! um app de mensagens não deveria pedir à Apple/Google. Nenhum nível de criptografia muda essa regra de
//! sistema operacional — por isso este módulo não tenta.
//!
//! **O que isto PODE ser, e é o equivalente real da promessa do dono:** depois de `pode_destruir_o_antigo`
//! devolver `Ok`, nenhuma chave capaz de abrir qualquer conteúdo deste aparelho continua existindo nele —
//! nem o conteúdo temporário (`historico`/`prechaves`/`metadados`, Cláusula 6a — reaproveitado, não
//! reimplementado), nem a identidade de sessão (`sessoes::Registro::revogar_aparelho`), nem a chave local
//! do cofre pessoal (Cláusula 26) nem a referência local da aba trancada — estas duas últimas vivem SÓ na
//! ponte (`custodia_mutua::Referencia` nunca é persistida pelo núcleo, por desenho; a chave do cofre vive
//! no Keychain/Keystore, nunca em claro — ver `custodia_do_cofre.rs`), então este núcleo não tem como
//! apagá-las diretamente: isto é o novo CONTRATO_DA_PONTE.md regra 39, não código Rust (mesma razão de
//! `trafego::Batida` ter virado regra do contrato em vez de função do núcleo — núcleo nunca trata
//! transporte nem armazenamento nativo). O app continua instalado — um casco vazio, sem segredo algum
//! dentro — nunca um app desinstalado por si mesmo.
//!
//! **Por que isto é deliberadamente SEPARADO de `panico::Reacao::ApagarChaves` (nunca a mesma coisa):**
//! `ApagarChaves` é uma reação de PÂNICO — a conta continua existindo e precisa voltar a funcionar num
//! aparelho novo pela frase de recuperação ou pelos guardiões (ver a doc daquele módulo); por isso ela
//! NUNCA toca a chave permanente do cofre (destruiria a única cópia sem ter como substituí-la a tempo).
//! "Retirada de aparelho" é o oposto: o dono JÁ TEM um aparelho novo com uma cópia confirmada de tudo, e
//! quer justamente que o antigo não guarde mais nada, nem a chave do cofre. Reaproveitar `ApagarChaves`
//! para isto seria misturar dois cenários com garantias diferentes — exatamente o erro que
//! `custodia_do_cofre.rs` já evita ao manter a Família 2 "deliberadamente separada" da Cláusula 6a.
//!
//! **O portão central, e por que ele existe:** destruir ANTES de o aparelho novo confirmar que recebeu o
//! pacote por inteiro transformaria o próprio recurso num jeito de perder dados — um Bluetooth cortado no
//! meio, ou o aparelho novo travando depois de receber só metade do pacote, apagaria a única cópia da
//! chave do cofre sem o novo ter ficado com nada. Por isso a destruição só é permitida depois de uma
//! `ConfirmacaoDeRecebimento` ASSINADA pelo aparelho novo — nunca por um sinal de transporte ("Bluetooth
//! disse que terminou"), que prova só que bytes chegaram, nunca que o aparelho novo conseguiu abrir o
//! pacote de verdade (`custodia_mutua::abrir_pacote_de_transferencia` devolveu `Ok`).
use crate::assinatura::{AssinaturaHibrida, ChaveDeAssinatura, ChaveDeVerificacao};
use sha2::{Digest, Sha512};

const CONTEXTO: &[u8] = b"NimbusOne/v1/retirada_de_aparelho";

/// Mesmo formato de `sessoes::IdAparelho` / `comando_remoto::IdAparelho` ([u8; 32]) — não importado por
/// alias para não acoplar este módulo a `sessoes.rs` só por isso; qualquer `[u8; 32]` que a ponte já usa
/// como id de aparelho serve aqui sem conversão, mesmo raciocínio já documentado em `comando_remoto.rs`.
pub type IdAparelho = [u8; 32];

/// Impressão digital do pacote selado (SHA-512 truncado a 32 bytes — mesma construção de
/// `assinatura::ChaveDeVerificacao::impressao_digital`, reaproveitada aqui por consistência, não
/// reinventada). Calculada nos bytes EXATOS que `custodia_mutua::preparar_pacote_de_transferencia`
/// devolveu, antes de qualquer transporte — amarra a confirmação a ESTE pacote, nunca a "um pacote
/// qualquer, algum dia" (fecha o caso de um aparelho assinar uma confirmação genérica antes de receber).
pub fn hash_do_pacote(pacote: &[u8]) -> [u8; 32] {
    let h = Sha512::digest(pacote);
    let mut saida = [0u8; 32];
    saida.copy_from_slice(&h[..32]);
    saida
}

/// Prova assinada, pelo PRÓPRIO aparelho novo, de que chamou `custodia_mutua::abrir_pacote_de_transferencia`
/// e recebeu `Ok` — ou seja, já tem em mãos as `Referencia`s e a chave do cofre que o pacote carregava.
/// Assinada pela MESMA chave de identidade que a cerimônia de vínculo (`sessoes::palavras_do_vinculo`) já
/// confirmou nos dois aparelhos — nenhum segredo novo, nenhuma chave extra para o dono guardar ou perder.
#[derive(Clone, Debug)]
pub struct ConfirmacaoDeRecebimento {
    pub aparelho_novo: IdAparelho,
    pub hash_do_pacote: [u8; 32],
    pub contador: u64,
    /// Impressão digital da chave de quem assinou — a ponte resolve, com isto, qual `ChaveDeVerificacao`
    /// passar para `validar_confirmacao` (mesmo papel do campo `emissor` em `comando_remoto::ComandoDeProtecao`).
    pub emissor: [u8; 32],
    pub assinatura: AssinaturaHibrida,
}

fn bytes_para_assinar(aparelho_novo: &IdAparelho, hash_do_pacote: &[u8; 32], contador: u64) -> Vec<u8> {
    let mut v = aparelho_novo.to_vec();
    v.extend_from_slice(hash_do_pacote);
    v.extend_from_slice(&contador.to_be_bytes());
    v
}

/// No aparelho NOVO, imediatamente depois de `custodia_mutua::abrir_pacote_de_transferencia` devolver
/// `Ok`. `contador`: estritamente crescente por aparelho novo — mesmo princípio de `comando_remoto.rs`,
/// `coacao::RelogioDeEspera` e a `versao` de `liberacao::Autorizacao`; impede que uma confirmação antiga
/// capturada em trânsito seja reaproveitada depois (ex.: contra um pacote diferente, futuro).
pub fn confirmar_recebimento(
    chave_do_novo: &ChaveDeAssinatura,
    aparelho_novo: IdAparelho,
    hash_do_pacote: [u8; 32],
    contador: u64,
) -> ConfirmacaoDeRecebimento {
    let assinatura = chave_do_novo
        .assinar(CONTEXTO, &bytes_para_assinar(&aparelho_novo, &hash_do_pacote, contador))
        .expect("contexto fixo e curto");
    ConfirmacaoDeRecebimento {
        aparelho_novo,
        hash_do_pacote,
        contador,
        emissor: chave_do_novo.verificacao().impressao_digital(),
        assinatura,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeConfirmacao {
    /// Assinatura não confere — cobre tanto "não foi este aparelho que assinou" quanto "a confirmação foi
    /// alterada depois de assinada" (mesmo raciocínio de `comando_remoto::Recusa::AssinaturaInvalida`).
    AssinaturaInvalida,
    /// Assinada por um aparelho diferente do que a ponte esperava receber a confirmação.
    AparelhoErrado,
    /// A confirmação é de um pacote diferente do que este aparelho (o antigo) de fato enviou — nunca
    /// aceitar "recebi algo" como prova de "recebi O QUE EU MANDEI".
    PacoteErrado,
    /// Contador igual ou menor que o último já aceito para este aparelho novo — confirmação repetida ou velha.
    ConfirmacaoRepetida,
}

/// `chave_do_novo`: a chave de verificação do aparelho novo, já confirmada pela cerimônia de vínculo
/// (`sessoes::pode_vincular`) antes mesmo de a transferência começar — a ponte nunca aceita a chave de
/// verificação de dentro da própria confirmação sem checar primeiro que é a mesma chave da cerimônia.
/// `hash_do_pacote_enviado`: o hash que ESTE aparelho (o antigo) calculou sobre o pacote que ele de fato
/// selou e mandou — nunca o hash que vem dentro da confirmação sem comparação (senão qualquer hash
/// "confirmaria" qualquer pacote). `ultimo_contador_aceito`: persistido pela ponte, nunca por este módulo
/// (mesmo padrão sem-estado de `comando_remoto::validar` e `panico::decidir`).
pub fn validar_confirmacao(
    chave_do_novo: &ChaveDeVerificacao,
    confirmacao: &ConfirmacaoDeRecebimento,
    aparelho_novo_esperado: &IdAparelho,
    hash_do_pacote_enviado: &[u8; 32],
    ultimo_contador_aceito: u64,
) -> Result<(), RecusaDeConfirmacao> {
    let bytes = bytes_para_assinar(&confirmacao.aparelho_novo, &confirmacao.hash_do_pacote, confirmacao.contador);
    if !chave_do_novo.verificar(CONTEXTO, &bytes, &confirmacao.assinatura) {
        return Err(RecusaDeConfirmacao::AssinaturaInvalida);
    }
    if confirmacao.aparelho_novo != *aparelho_novo_esperado {
        return Err(RecusaDeConfirmacao::AparelhoErrado);
    }
    if &confirmacao.hash_do_pacote != hash_do_pacote_enviado {
        return Err(RecusaDeConfirmacao::PacoteErrado);
    }
    if confirmacao.contador <= ultimo_contador_aceito {
        return Err(RecusaDeConfirmacao::ConfirmacaoRepetida);
    }
    Ok(())
}

/// Estado desta retirada — guardado pela ponte; este módulo não tem estado próprio (mesmo princípio de
/// `panico.rs` e `comando_remoto.rs`, funções puras, sem efeito colateral).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fase {
    AguardandoConfirmacaoDoNovo,
    ConfirmadoPeloNovo,
    AntigoDestruido,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecusaDeDestruicao {
    /// Ainda não há `ConfirmacaoDeRecebimento` validada para este pacote.
    SemConfirmacaoValida,
    /// Esta retirada já foi concluída antes — nunca destrói duas vezes.
    JaDestruido,
}

/// A ponte só chama isto depois de `validar_confirmacao` devolver `Ok`. Esta função não valida nada de
/// novo por si (isso já é papel de `validar_confirmacao`) — ela é o AVANÇO DE ESTADO, propositalmente
/// separado da validação, para que "verificar a assinatura" e "decidir destruir" nunca sejam o mesmo
/// passo arriscado de misturar.
pub fn avancar_apos_confirmacao_valida(fase_atual: Fase) -> Result<Fase, RecusaDeDestruicao> {
    match fase_atual {
        Fase::AguardandoConfirmacaoDoNovo => Ok(Fase::ConfirmadoPeloNovo),
        // Confirmação chegou de novo (ex.: reenvio de rede) depois de já confirmado: idempotente, não é erro.
        Fase::ConfirmadoPeloNovo => Ok(Fase::ConfirmadoPeloNovo),
        Fase::AntigoDestruido => Err(RecusaDeDestruicao::JaDestruido),
    }
}

/// Único portão antes de destruir. Só devolve `Ok` saindo de `ConfirmadoPeloNovo` — nunca de
/// `AguardandoConfirmacaoDoNovo` (não é possível destruir sem confirmação) nem de `AntigoDestruido`
/// (não destrói duas vezes; chamar de novo depois de já ter destruído indica um bug de quem chama, não
/// uma segunda destruição real sobre algo que já não existe mais).
pub fn pode_destruir_o_antigo(fase_atual: Fase) -> Result<(), RecusaDeDestruicao> {
    match fase_atual {
        Fase::AguardandoConfirmacaoDoNovo => Err(RecusaDeDestruicao::SemConfirmacaoValida),
        Fase::ConfirmadoPeloNovo => Ok(()),
        Fase::AntigoDestruido => Err(RecusaDeDestruicao::JaDestruido),
    }
}

/// Checklist executável (não só prosa) do que a ponte precisa compor, em ordem, depois que
/// `pode_destruir_o_antigo` devolver `Ok`. Esta struct não executa nada — nomeia a origem de cada passo
/// para a ponte não ter que redescobrir isto lendo quatro módulos e um contrato diferentes, e para que
/// uma mudança que tire um passo da lista quebre um teste aqui, não só uma frase de documentação que
/// alguém poderia esquecer de atualizar.
///
/// Os 4 passos de dentro do núcleo reaproveitam funções que já existem (nenhuma nova): os 3 primeiros são
/// exatamente os já citados pela doc de `comando_remoto::Reacao::ApagarChaves` para "o temporário"
/// (`historico`, `prechaves`, `metadados`, cada um com `agora` forçado para o futuro); o 4º é
/// `sessoes::Registro::revogar_aparelho`. Os 2 passos "só na ponte" são NOVOS (não existiam pedido
/// nenhum antes desta rodada) e ficam fora do núcleo porque o material que apagam só existe lá — ver
/// CONTRATO_DA_PONTE.md, regra 39.
pub struct SequenciaDeDestruicao;

impl SequenciaDeDestruicao {
    pub const PASSOS_NO_NUCLEO: [&'static str; 4] = [
        "historico::ChavesDoDono::destruir_janelas_vencidas(agora_forcado_no_futuro)",
        "prechaves::Chaveiro::apagar_vencidas(agora_forcado_no_futuro)",
        "metadados::ChavesDeIndice::destruir_vencidas(agora_forcado_no_futuro)",
        "sessoes::Registro::revogar_aparelho(conta, aparelho_antigo)",
    ];
    pub const PASSOS_SO_NA_PONTE: [&'static str; 2] = [
        "apagar do Keychain/Keystore a chave local do cofre pessoal deste aparelho (Cláusula 26)",
        "apagar o armazenamento local das Referencias da aba trancada deste aparelho (custodia_mutua::Referencia nunca é persistida pelo núcleo)",
    ];
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::assinatura;

    const APARELHO_NOVO: IdAparelho = [7; 32];
    const APARELHO_OUTRO: IdAparelho = [9; 32];

    fn pacote_e_hash() -> (Vec<u8>, [u8; 32]) {
        let pacote = b"bytes-selados-do-pacote-de-transferencia-simulado".to_vec();
        let hash = hash_do_pacote(&pacote);
        (pacote, hash)
    }

    #[test]
    fn confirmacao_valida_e_aceita() {
        let novo = assinatura::gerar();
        let (_, hash) = pacote_e_hash();
        let conf = confirmar_recebimento(&novo, APARELHO_NOVO, hash, 1);
        assert_eq!(validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash, 0), Ok(()));
    }

    #[test]
    fn confirmacao_de_pacote_errado_e_recusada() {
        let novo = assinatura::gerar();
        let (_, hash_enviado) = pacote_e_hash();
        let hash_de_outro_pacote = hash_do_pacote(b"pacote completamente diferente");
        let conf = confirmar_recebimento(&novo, APARELHO_NOVO, hash_de_outro_pacote, 1);
        assert_eq!(
            validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash_enviado, 0),
            Err(RecusaDeConfirmacao::PacoteErrado)
        );
    }

    #[test]
    fn confirmacao_de_aparelho_errado_e_recusada() {
        let novo = assinatura::gerar();
        let (_, hash) = pacote_e_hash();
        let conf = confirmar_recebimento(&novo, APARELHO_OUTRO, hash, 1);
        assert_eq!(
            validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash, 0),
            Err(RecusaDeConfirmacao::AparelhoErrado)
        );
    }

    #[test]
    fn confirmacao_de_quem_nao_e_o_aparelho_novo_e_recusada() {
        let novo = assinatura::gerar();
        let impostor = assinatura::gerar();
        let (_, hash) = pacote_e_hash();
        let conf = confirmar_recebimento(&impostor, APARELHO_NOVO, hash, 1);
        assert_eq!(
            validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash, 0),
            Err(RecusaDeConfirmacao::AssinaturaInvalida)
        );
    }

    #[test]
    fn confirmacao_adulterada_depois_de_assinada_e_recusada() {
        let novo = assinatura::gerar();
        let (_, hash) = pacote_e_hash();
        let mut conf = confirmar_recebimento(&novo, APARELHO_NOVO, hash, 1);
        conf.hash_do_pacote = hash_do_pacote(b"tentativa de troca depois de assinado");
        assert_eq!(
            validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash, 0),
            Err(RecusaDeConfirmacao::AssinaturaInvalida),
            "mudar o hash depois de assinado invalida a assinatura, nunca passa a valer para outro pacote"
        );
    }

    #[test]
    fn confirmacao_repetida_ou_antiga_e_recusada() {
        let novo = assinatura::gerar();
        let (_, hash) = pacote_e_hash();
        let conf = confirmar_recebimento(&novo, APARELHO_NOVO, hash, 5);
        assert_eq!(validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash, 5), Err(RecusaDeConfirmacao::ConfirmacaoRepetida));
        assert_eq!(validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash, 6), Err(RecusaDeConfirmacao::ConfirmacaoRepetida));
        assert_eq!(validar_confirmacao(&novo.verificacao(), &conf, &APARELHO_NOVO, &hash, 4), Ok(()));
    }

    #[test]
    fn destruicao_recusada_sem_confirmacao() {
        assert_eq!(pode_destruir_o_antigo(Fase::AguardandoConfirmacaoDoNovo), Err(RecusaDeDestruicao::SemConfirmacaoValida));
    }

    #[test]
    fn destruicao_permitida_so_depois_de_confirmado() {
        let fase = avancar_apos_confirmacao_valida(Fase::AguardandoConfirmacaoDoNovo).unwrap();
        assert_eq!(fase, Fase::ConfirmadoPeloNovo);
        assert_eq!(pode_destruir_o_antigo(fase), Ok(()));
    }

    #[test]
    fn destruicao_nunca_acontece_duas_vezes() {
        assert_eq!(pode_destruir_o_antigo(Fase::AntigoDestruido), Err(RecusaDeDestruicao::JaDestruido));
        assert_eq!(avancar_apos_confirmacao_valida(Fase::AntigoDestruido), Err(RecusaDeDestruicao::JaDestruido));
    }

    #[test]
    fn confirmacao_repetida_apos_destruido_nao_reabre_a_fase() {
        // Reenvio de rede de uma confirmação já processada, chegando DEPOIS da destruição: idempotência
        // só vale enquanto ainda não destruiu — depois, é sempre JaDestruido, nunca uma regressão de fase.
        assert_eq!(avancar_apos_confirmacao_valida(Fase::AntigoDestruido), Err(RecusaDeDestruicao::JaDestruido));
    }

    #[test]
    fn hash_do_pacote_e_deterministico_e_sensivel_a_qualquer_byte() {
        let (pacote, hash1) = pacote_e_hash();
        let hash2 = hash_do_pacote(&pacote);
        assert_eq!(hash1, hash2, "o mesmo pacote sempre produz o mesmo hash");
        let mut alterado = pacote.clone();
        let n = alterado.len() - 1;
        alterado[n] ^= 1;
        assert_ne!(hash1, hash_do_pacote(&alterado), "um byte alterado já muda o hash");
    }

    #[test]
    fn sequencia_de_destruicao_lista_4_passos_reaproveitados_no_nucleo_e_2_novos_so_na_ponte() {
        assert_eq!(SequenciaDeDestruicao::PASSOS_NO_NUCLEO.len(), 4);
        assert_eq!(SequenciaDeDestruicao::PASSOS_SO_NA_PONTE.len(), 2);
        assert!(SequenciaDeDestruicao::PASSOS_NO_NUCLEO.iter().any(|p| p.contains("historico")));
        assert!(SequenciaDeDestruicao::PASSOS_NO_NUCLEO.iter().any(|p| p.contains("prechaves")));
        assert!(SequenciaDeDestruicao::PASSOS_NO_NUCLEO.iter().any(|p| p.contains("metadados")));
        assert!(SequenciaDeDestruicao::PASSOS_NO_NUCLEO.iter().any(|p| p.contains("revogar_aparelho")));
        assert!(SequenciaDeDestruicao::PASSOS_SO_NA_PONTE.iter().any(|p| p.contains("Keychain")));
        assert!(SequenciaDeDestruicao::PASSOS_SO_NA_PONTE.iter().any(|p| p.contains("aba trancada")));
    }
}
