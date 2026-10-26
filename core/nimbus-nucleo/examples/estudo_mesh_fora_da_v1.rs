//! # ESTUDO — FORA DA V1 (decisão OFF1 do fundador). Não implementar nos apps.
//!
//! Este exemplo foi feito num chat paralelo como prova de conceito de rede mesh. Depois, o fundador decidiu
//! **não ter rede mesh nem modo offline na v1** ("não podemos ter brecha"). Ele fica no repositório só como
//! estudo, compilado a cada versão para não apodrecer, e documenta por que a rede mesh ficou de fora:
//!
//! 1. Sem servidor, três garantias caem: prazo pela hora do servidor (`expiracao::hora_confiavel`),
//!    derrubada de sessão paralela e reconfirmação do aparelho (`integridade`).
//! 2. Os intermediários guardam o pacote (abaixo, a "mochila") e veem o identificador, o tamanho, a hora e
//!    quem passou perto de quem — metadados que a arquitetura esconde no caminho normal.
//! 3. As análises públicas do Bridgefy (Royal Holloway, 2021 e 2022) mostraram rastreamento de usuários e
//!    montagem de grafos sociais em rede mesh.
//!
//! O que a v1 tem no lugar: NimbusDrop lacrado (`direto::lacrar` / `direto::abrir_lacrado`) — um salto,
//! direto entre dois aparelhos, e o arquivo só abre quando o servidor entrega a chave. A regra de varredura
//! `nimbus-sem-rede-mesh` reprova APIs de rede mesh nos apps.
//!
//! Rode só para estudo: cargo run --example estudo_mesh_fora_da_v1
//!
//! # Prova de conceito: comunicação sem internet, saltando de aparelho em aparelho (mesh)
//!
//! Mostra, de forma tangível, que a MESMA remessa cifrada do módulo `direto` viaja por vários aparelhos
//! intermediários — por Bluetooth ou Wi-Fi direto, sem servidor, sem internet — e só o destino consegue ler.
//! Cada aparelho no meio do caminho ("carteiro") apenas guarda e repassa um bloco opaco.
//!
//! Rode com: (nome original: mesh_sem_internet)
use nimbus_nucleo::direto::{self, Remessa};
use nimbus_nucleo::envelope::{self, ChavePublica, ChaveSecreta};
use std::collections::HashMap;

/// Um aparelho na malha. Guarda remessas para repassar (store-and-forward) e conta quantas viu.
struct Aparelho {
    nome: &'static str,
    publica: ChavePublica,
    secreta: ChaveSecreta,
    mochila: HashMap<[u8; 16], (Remessa, u8)>, // remessa + saltos restantes
}

impl Aparelho {
    fn novo(nome: &'static str) -> Self {
        let (publica, secreta) = envelope::gerar_par_de_chaves();
        Aparelho { nome, publica, secreta, mochila: HashMap::new() }
    }
    /// Recebe uma remessa por rádio de curto alcance. Não tenta ler: só guarda para repassar.
    fn recebe_para_repassar(&mut self, r: Remessa, saltos: u8) {
        let tentou_ler = direto::abrir(&self.secreta, &self.publica, &r, 0).is_ok();
        println!("   {} guardou o pacote {} (consegue ler? {}) — {} salto(s) restante(s)",
                 self.nome, hex4(&r.id), if tentou_ler { "SIM" } else { "NÃO" }, saltos);
        if saltos > 0 { let _ = self.mochila.insert(r.id, (r, saltos)); }
    }
}

fn hex4(id: &[u8; 16]) -> String { format!("{:02x}{:02x}…", id[0], id[1]) }

fn main() {
    println!("== Comunicação sem internet: a mensagem salta de aparelho em aparelho ==\n");

    // Ana quer mandar um recado para Bruno. Não há internet — nem sinal de celular.
    // Entre eles, há três desconhecidos que também usam o app e passam por perto: Carla, Davi e Elis.
    let _ana = Aparelho::novo("Ana"); // Ana é quem envia; aqui só precisamos da remessa que ela cria
    let bruno = Aparelho::novo("Bruno");
    let mut carla = Aparelho::novo("Carla (desconhecida)");
    let mut davi = Aparelho::novo("Davi (desconhecido)");
    let mut elis = Aparelho::novo("Elis (desconhecida)");

    // Ana prepara a remessa endereçada à CHAVE de Bruno. Vale por 48 h (o teto do app).
    let id = {
        use rand_core::{OsRng, RngCore};
        let mut v = [0u8; 16]; OsRng.fill_bytes(&mut v); v
    };
    let recado = "Chego às 18h no ponto combinado. — Ana";
    let remessa = direto::preparar(&bruno.publica, id, "recado.txt", recado.as_bytes(), 0, direto::PRAZO_MAXIMO)
        .expect("preparar remessa");
    println!("Ana criou um pacote cifrado de {} bytes, endereçado à chave de Bruno.", remessa.bloco.len());
    println!("Ninguém além de Bruno tem a chave para abrir. O pacote vai viajar de mão em mão.\n");

    // A malha: Ana entrega a quem estiver por perto. O pacote salta até encontrar Bruno.
    // "saltos" é o limite de vezes que o pacote pode ser repassado (impede vagar para sempre).
    println!("1) Ana passa por Carla (Bluetooth, sem internet):");
    carla.recebe_para_repassar(remessa.clone(), 3);
    println!("\n2) Carla, mais tarde, cruza com Davi e repassa:");
    davi.recebe_para_repassar(remessa.clone(), 2);
    println!("\n3) Davi cruza com Elis e repassa:");
    elis.recebe_para_repassar(remessa.clone(), 1);
    println!("\n4) Elis finalmente encontra Bruno e entrega:");
    // Bruno tenta abrir de verdade.
    match direto::abrir(&bruno.secreta, &bruno.publica, &remessa, 0) {
        Ok(recebido) => {
            let texto = String::from_utf8_lossy(&recebido.dados);
            println!("   Bruno ABRIU o pacote {}: \"{}\"", hex4(&remessa.id), texto);
        }
        Err(e) => println!("   Bruno não conseguiu abrir: {e:?}"),
    }

    // Prova de que os carteiros nunca leram nada.
    println!("\n== O que os intermediários viram ==");
    for a in [&carla, &davi, &elis] {
        let bloco = a.mochila.values().next().map(|(r, _)| r.bloco.len());
        println!("   {}: repassou sem ler — apenas um bloco opaco de {:?} bytes, ilegível",
                 a.nome, bloco);
    }

    // Prova de que um intermediário não consegue adulterar sem ser percebido.
    println!("\n== E se um intermediário tentar adulterar o pacote? ==");
    let mut adulterada = remessa.clone();
    let meio = adulterada.bloco.len() / 2;
    adulterada.bloco[meio] ^= 0x01;
    match direto::abrir(&bruno.secreta, &bruno.publica, &adulterada, 0) {
        Ok(_) => println!("   FALHA DE SEGURANÇA: abriu um pacote adulterado (não deveria)"),
        Err(e) => println!("   Bruno rejeita o pacote adulterado: {e:?} — a cifra autenticada protege"),
    }

    println!("\nResumo: o pacote chegou sem internet, passando por 3 desconhecidos,");
    println!("nenhum deles leu o conteúdo, e adulterar é impossível sem ser detectado.");
}
