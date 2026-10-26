//! # Fuzzing determinístico (prova C79) — o mesmo princípio do cargo-fuzz, rodando em Rust estável.
//!
//! Todo ponto do código que recebe bytes de fora (do servidor, de um arquivo, de um QR) é bombardeado com
//! milhões de entradas geradas por um gerador reproduzível (xoshiro). A ÚNICA exigência: nunca entrar em
//! pânico, nunca travar, nunca ler fora dos limites. Erros são resultado normal; pânico é falha grave.
//!
//! Semente fixa (o CI repete exatamente o mesmo ataque). Para caçar mais fundo, aumente RODADAS.
//! Rode com: cargo test --profile verificacao --test fuzzing -- --ignored (o ./verificar.sh já faz isso).
//! Mais fundo: FUZZ_RODADAS=2000000 (o ./verificar.sh --completo faz isso).
use nimbus_nucleo::*;

/// Entradas por alvo: 50 mil por padrão (segundos); o --completo usa 2 milhões.
fn rodadas() -> usize { std::env::var("FUZZ_RODADAS").ok().and_then(|v| v.parse().ok()).unwrap_or(50_000) }

/// Gerador reproduzível (xoshiro256++), sem dependência externa, para o fuzzing ser idêntico em toda máquina.
struct Rng { s: [u64; 4] }
impl Rng {
    fn nova(semente: u64) -> Self {
        // SplitMix64 para semear os quatro estados.
        let mut z = semente;
        let mut prox = || { z = z.wrapping_add(0x9E3779B97F4A7C15); let mut x = z; x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9); x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB); x ^ (x >> 31) };
        Rng { s: [prox(), prox(), prox(), prox()] }
    }
    fn prox(&mut self) -> u64 {
        let r = self.s[0].wrapping_add(self.s[3]).rotate_left(23).wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0]; self.s[3] ^= self.s[1]; self.s[1] ^= self.s[2]; self.s[0] ^= self.s[3];
        self.s[2] ^= t; self.s[3] = self.s[3].rotate_left(45);
        r
    }
    /// Bytes de tamanho variado, às vezes com estrutura parcialmente válida (prefixos conhecidos).
    fn bytes(&mut self, ate: usize) -> Vec<u8> {
        let n = (self.prox() as usize) % (ate + 1);
        let mut v = vec![0u8; n];
        for c in v.iter_mut() { *c = (self.prox() & 0xFF) as u8; }
        // 1 em 8: começa com um prefixo de formato real, para exercitar os caminhos de parsing.
        if n >= 8 && self.prox().is_multiple_of(8) {
            let prefixos: [&[u8]; 6] = [&[0xFF, 0xD8, 0xFF], b"%PDF-", &[0x89, b'P', b'N', b'G'], b"PK\x03\x04", &[1u8], &[0u8, 0, 0, 2]];
            let p = prefixos[(self.prox() as usize) % prefixos.len()];
            v[..p.len().min(n)].copy_from_slice(&p[..p.len().min(n)]);
        }
        v
    }
}

/// Executa `f` com muitas entradas aleatórias; qualquer pânico estoura o teste com a semente para reproduzir.
fn fuzz(nome: &str, semente: u64, tam: usize, mut f: impl FnMut(&[u8])) {
    let mut rng = Rng::nova(semente);
    for i in 0..rodadas() {
        let entrada = rng.bytes(tam);
        // Captura pânico para reportar a entrada exata que quebrou (em produção, panic = abort).
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&entrada)));
        assert!(r.is_ok(), "PÂNICO em '{nome}' na rodada {i} (semente {semente}); entrada de {} bytes: {:02x?}", entrada.len(), &entrada[..entrada.len().min(64)]);
    }
}

#[test]
#[ignore = "pesado: cargo test --profile verificacao --test fuzzing -- --ignored (o verificar.sh já faz isso)"]
fn c79_decodificadores_nunca_entram_em_panico() {
    // Silencia o hook de pânico durante o fuzzing (senão polui a saída); o catch_unwind ainda pega tudo.
    let anterior = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    let (publ, sec) = envelope::gerar_par_de_chaves();
    let aad = b"NimbusOne/v1/direto";

    // Envelope e chaves — o coração da criptografia, alimentado direto do servidor.
    fuzz("envelope::abrir", 1, 4096, |b| { let _ = envelope::abrir(&sec, &publ, b, aad); });
    fuzz("ChaveSecreta::de_bytes", 2, 3200, |b| { let _ = envelope::ChaveSecreta::de_bytes(b); });
    fuzz("ChavePublica::de_bytes", 3, 1600, |b| { let _ = envelope::ChavePublica::de_bytes(b); });
    fuzz("assinatura::ChaveDeVerificacao::de_bytes", 4, 5000, |b| { let _ = assinatura::ChaveDeVerificacao::de_bytes(b); });

    // Cofre, contatos, pacote — blocos guardados no servidor.
    fuzz("cofre::abrir", 5, 4096, |b| { let _ = cofre::abrir(&sec, &publ, b); });
    fuzz("contatos::abrir_contato", 6, 4096, |b| { let _ = contatos::abrir_contato(&sec, &publ, b); });
    fuzz("pacote::abrir_item", 7, 4096, |b| { let _ = pacote::abrir_item(&sec, &publ, b); });

    // Custódia e recuperação — partes de chave vindas de guardiões.
    fuzz("custodia::PartesDoGuardiao::de_bytes", 8, 4096, |b| { let _ = custodia::PartesDoGuardiao::de_bytes(b); });
    fuzz("recuperacao::reunir", 9, 2048, |b| { let _ = recuperacao::reunir(&[b, b]); });

    // Inspeção de arquivos — o que chega para o usuário abrir; precisa ser à prova de tudo.
    fuzz("inspecao::tipo_real", 10, 8192, |b| { let _ = inspecao::tipo_real(b); });
    fuzz("inspecao::inspecionar", 11, 8192, |b| { let _ = inspecao::inspecionar("arquivo.pdf", b); });

    // Higiene de imagem — remove metadados de fotos que podem ser maliciosas.
    fuzz("higiene::limpar", 12, 8192, |b| { let _ = higiene::limpar(b); });
    fuzz("higiene::detectar", 13, 8192, |b| { let _ = higiene::detectar(b); });

    // Nome de arquivo cifrado no Nimbus Envio.

    // Envio direto — remessa inteira montada por um remetente possivelmente malicioso.
    fuzz("direto::abrir", 15, 4096, |b| {
        let r = direto::Remessa { id: [0u8; 16], bloco: b.to_vec(), valida_ate: u64::MAX };
        let _ = direto::abrir(&sec, &publ, &r, 0);
    });

    // Registro público (árvore de Merkle) — provas de inclusão vindas do servidor.
    fuzz("registro::verificar_inclusao", 16, 2048, |b| {
        let indice = (b.first().copied().unwrap_or(0)) as usize;
        let tamanho = (b.get(1).copied().unwrap_or(1)) as usize;
        let prova: Vec<registro::Hash> = b.chunks_exact(32).map(|c| { let a: [u8; 32] = c.try_into().unwrap(); a }).collect();
        let raiz = registro::hash_de_folha(b);
        let _ = registro::verificar_inclusao(b, indice, tamanho, &prova, &raiz);
    });

    // Notificação — ruído cifrado que chega pelo APNs/canal próprio.
    fuzz("notificacao::ao_receber", 18, 2048, |b| { let _ = notificacao::ao_receber(&sec, &publ, b); });

    // Lista de bloqueio lida do armazenamento do aparelho (pode ter sido adulterada).
    fuzz("protecao_do_usuario::ListaDeBloqueio::de_bytes", 21, 4 + protecao_do_usuario::MAXIMO_DE_BLOQUEADOS * 32 + 8, |b| {
        let _ = protecao_do_usuario::ListaDeBloqueio::de_bytes(b);
    });
    // Endereços dentro de mensagens recebidas — texto de qualquer remetente (caso Privnote).
    fuzz("autenticidade::avaliar_mensagem", 19, 8192, |b| {
        let t = String::from_utf8_lossy(b);
        let _ = autenticidade::avaliar_mensagem(&t, &["nimbus-teste.example", "privnote.com"]);
    });
    fuzz("autenticidade::imitacao", 20, 512, |b| {
        let t = String::from_utf8_lossy(b);
        let _ = autenticidade::imitacao(&t, &["nimbus-teste.example"]);
    });

    std::panic::set_hook(anterior);
}
