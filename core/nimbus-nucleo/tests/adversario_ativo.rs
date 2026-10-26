//! # Adversário ativo no meio do caminho (prova C86)
//!
//! Até aqui, os testes provaram que o envelope resiste a bytes adulterados e a lixo aleatório. Falta o ataque
//! que quebra protocolos reais: um adversário que controla o transporte e pode GUARDAR, REPETIR, REORDENAR e
//! DEVOLVER mensagens legítimas — sem nunca quebrar a cifra. É o modelo Dolev-Yao (o atacante é a rede).
//!
//! A defesa não é a cifra em si — é o dado associado autenticado (AAD): cada mensagem carrega, no AAD,
//! a identidade da conversa e um número de sequência. Quem recebe só aceita o próximo número esperado.
//! Assim, uma mensagem repetida ou fora de ordem é rejeitada mesmo estando perfeitamente cifrada.
//!
//! Estes testes montam esse AAD e provam as três defesas. Se algum falhar, o protocolo de sessão precisa
//! prender sequência+conversa no AAD antes do lançamento — e o teste diz exatamente isso.
use nimbus_nucleo::envelope::{self, ChavePublica, ChaveSecreta};

/// O AAD de cada mensagem: identidade da conversa (32 bytes) + número de sequência (8 bytes, big-endian).
fn aad(id_conversa: &[u8; 32], sequencia: u64) -> Vec<u8> {
    let mut v = Vec::with_capacity(40);
    v.extend_from_slice(id_conversa);
    v.extend_from_slice(&sequencia.to_be_bytes());
    v
}

/// Quem recebe: aceita só o próximo número esperado; devolve o texto e avança, ou rejeita.
struct Receptor { id_conversa: [u8; 32], proximo: u64, s: ChaveSecreta, p_propria: ChavePublica }
impl Receptor {
    fn receber(&mut self, envelope: &[u8], sequencia_alegada: u64) -> Result<zeroize::Zeroizing<Vec<u8>>, &'static str> {
        // O AAD é reconstruído a partir do que o receptor ESPERA, não do que o atacante diz.
        let esperado = self.proximo;
        let claro = envelope::abrir(&self.s, &self.p_propria, envelope, &aad(&self.id_conversa, esperado))
            .map_err(|_| "rejeitada: cifra ou AAD não conferem")?;
        // Se abriu, a sequência embutida no AAD era a esperada. Uma mensagem repetida ou fora de ordem
        // teria outro número no AAD e não abriria. Ignora o número alegado pelo atacante.
        let _ = sequencia_alegada;
        self.proximo = esperado.saturating_add(1);
        Ok(claro)
    }
}

fn cenario() -> (ChavePublica, Receptor) {
    let (p_dst, s_dst) = envelope::gerar_par_de_chaves();
    (p_dst.clone(), Receptor { id_conversa: [7u8; 32], proximo: 0, s: s_dst, p_propria: p_dst })
}

#[test]
fn c86_sequencia_normal_e_aceita() {
    let (p_dst, mut rx) = cenario();
    for i in 0..5u64 {
        let env = envelope::selar(&p_dst, format!("mensagem {i}").as_bytes(), &aad(&rx.id_conversa, i)).unwrap();
        assert_eq!(rx.receber(&env, i).unwrap().as_slice(), format!("mensagem {i}").as_bytes());
    }
}

#[test]
fn c86_mensagem_repetida_e_rejeitada() {
    let (p_dst, mut rx) = cenario();
    let env0 = envelope::selar(&p_dst, b"primeira", &aad(&rx.id_conversa, 0)).unwrap();
    assert!(rx.receber(&env0, 0).is_ok());
    // O atacante guardou env0 e reenvia. Está perfeitamente cifrada — mas o receptor já espera a de número 1.
    assert!(rx.receber(&env0, 0).is_err(), "replay de uma mensagem legítima não pode ser aceito");
    assert!(rx.receber(&env0, 1).is_err(), "nem alegando outro número: o AAD embutido continua sendo 0");
}

#[test]
fn c86_reordenacao_e_rejeitada() {
    let (p_dst, mut rx) = cenario();
    let env0 = envelope::selar(&p_dst, b"zero", &aad(&rx.id_conversa, 0)).unwrap();
    let env1 = envelope::selar(&p_dst, b"um", &aad(&rx.id_conversa, 1)).unwrap();
    // O atacante entrega a segunda antes da primeira.
    assert!(rx.receber(&env1, 1).is_err(), "mensagem fora de ordem é rejeitada");
    // Na ordem certa, ambas passam.
    assert!(rx.receber(&env0, 0).is_ok());
    assert!(rx.receber(&env1, 1).is_ok());
}

#[test]
fn c86_mensagem_de_outra_conversa_nao_entra_aqui() {
    let (p_dst, mut rx) = cenario();
    // Mensagem válida, mesmo remetente e destinatário, mas de OUTRA conversa (outro id).
    let outra = [9u8; 32];
    let env = envelope::selar(&p_dst, b"vazou?", &aad(&outra, 0)).unwrap();
    assert!(rx.receber(&env, 0).is_err(), "uma mensagem não pode ser replantada em outra conversa");
}

#[test]
fn c86_texto_adulterado_com_sequencia_certa_nao_abre() {
    let (p_dst, mut rx) = cenario();
    let mut env = envelope::selar(&p_dst, b"conteudo real", &aad(&rx.id_conversa, 0)).unwrap();
    let meio = env.len() / 2;
    env[meio] ^= 0x01;
    assert!(rx.receber(&env, 0).is_err(), "adulterar o texto invalida, mesmo com a sequência certa");
}
