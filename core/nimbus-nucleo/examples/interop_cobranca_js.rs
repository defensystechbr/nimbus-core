//! Prova de interoperabilidade Rust↔JavaScript da assinatura cega RFC 9474 (RSABSSA-SHA384-PSS-
//! Randomized) usada em `cobranca.rs` e, do lado do navegador, em `@cloudflare/blindrsa-ts` (a
//! biblioteca que a página de pagamento usa). São duas implementações INDEPENDENTES da mesma RFC
//! — isto só prova que elas se entendem de verdade se um byte que um lado produziu for
//! literalmente consumido pelo outro, nunca só por os dois "parecerem certos" sozinhos. A chave
//! secreta NUNCA sai deste processo Rust (nem serializada, nem em disco) — exatamente como no
//! `EmissorDeFichas` real: só os bytes que o protocolo de verdade troca pela rede (chave pública
//! em SPKI DER, mensagem cega, assinatura cega) cruzam a fronteira entre os dois processos, via
//! dois scripts `node` chamados como subprocesso.
//!
//! Uso: `cargo run --example interop_cobranca_js -- <diretório com etapa1_cegar.mjs e
//! etapa2_finalizar.mjs>`

use blind_rsa_signatures::BlindMessage;
use nimbus_nucleo::cobranca::{chave_publica_para_navegador, EmissorDeFichas, PagamentoConfirmado, PlanoPago, Periodo};
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let dir_scripts = env::args().nth(1).expect("uso: interop_cobranca_js <diretório dos .mjs>");
    let escala = env::temp_dir().join("nimbus_interop_prova");
    fs::create_dir_all(&escala).expect("criar diretório de troca");

    let periodo = Periodo { ano: 2026, mes: 11 };
    let mut emissor = EmissorDeFichas::novo();
    emissor.gerar_chave_do_periodo(PlanoPago::Pro, periodo).expect("gerar chave do período");
    let pk = emissor.chave_publica(PlanoPago::Pro, periodo).expect("chave pública existe");
    // A MESMA função que o endpoint real do servidor de pagamento chama para mandar a chave ao
    // navegador — ver a doc de `chave_publica_para_navegador` para por que NÃO é `pk.to_spki()`.
    let pk_der = chave_publica_para_navegador(pk).expect("exportar chave pública em SPKI DER (OID genérico)");
    let caminho_pk = escala.join("pk.der");
    fs::write(&caminho_pk, &pk_der).expect("escrever pk.der");
    println!("[rust] chave pública exportada: {} bytes SPKI DER", pk_der.len());

    // Etapa 1 (JS, "navegador"): cega um token contra a chave pública acima.
    let caminho_blinded = escala.join("blinded.bin");
    let caminho_estado = escala.join("estado.json");
    executar_node(&dir_scripts, "etapa1_cegar.mjs", &[&caminho_pk, &caminho_blinded, &caminho_estado]);

    // Etapa 2 (Rust, "emissor"): assina CEGAMENTE — nunca descriptografa nem lê o token.
    let blinded_bytes = fs::read(&caminho_blinded).expect("ler blinded.bin produzido pelo JS");
    println!("[rust] bytes cegos recebidos do JS: {} bytes (nunca vistos em claro)", blinded_bytes.len());
    let pagamento = PagamentoConfirmado {
        id_do_evento: b"evt_prova_de_interop_rust_js".to_vec(),
        plano: PlanoPago::Pro,
        periodo,
    };
    let msg_cega = BlindMessage(blinded_bytes);
    let assinatura_cega = emissor.assinar_cego(&pagamento, &msg_cega).expect("assinar cegamente");
    let caminho_blindsig = escala.join("blindsig.bin");
    fs::write(&caminho_blindsig, &assinatura_cega.0).expect("escrever blindsig.bin");
    println!("[rust] assinatura cega escrita: {} bytes", assinatura_cega.0.len());

    // Etapa 3 (JS, "navegador"): descega, finaliza e verifica — contra a MESMA chave pública,
    // numa biblioteca totalmente diferente da que assinou.
    let saida = executar_node(&dir_scripts, "etapa2_finalizar.mjs", &[&caminho_pk, &caminho_blindsig, &caminho_estado]);

    let verificacao_ok = saida.contains("VERIFICACAO=true");
    let token_trocado_recusado = saida.contains("VERIFICACAO_TOKEN_TROCADO=false");
    println!();
    println!("=== RESULTADO DA PROVA DE INTEROPERABILIDADE RUST<->JS (RFC 9474) ===");
    println!("Ficha assinada pelo núcleo Rust, verificada em JS (@cloudflare/blindrsa-ts): {}", if verificacao_ok { "PASSOU" } else { "FALHOU" });
    println!("Ficha de outro token recusada (controle negativo): {}", if token_trocado_recusado { "PASSOU" } else { "FALHOU" });

    if !verificacao_ok || !token_trocado_recusado {
        eprintln!("\nSAÍDA COMPLETA DO NODE (etapa 2), para diagnóstico:\n{saida}");
        std::process::exit(1);
    }
}

fn executar_node(dir_scripts: &str, script: &str, args: &[&std::path::Path]) -> String {
    let caminho_script = std::path::Path::new(dir_scripts).join(script);
    let saida = Command::new("node")
        .arg(&caminho_script)
        .args(args)
        .current_dir(dir_scripts)
        .output()
        .unwrap_or_else(|e| panic!("falha ao rodar {script}: {e}"));
    let stdout = String::from_utf8_lossy(&saida.stdout).to_string();
    let stderr = String::from_utf8_lossy(&saida.stderr).to_string();
    println!("[node:{script}] {}", stdout.trim());
    if !saida.status.success() {
        panic!("{script} falhou (status {:?}):\nstdout: {stdout}\nstderr: {stderr}", saida.status);
    }
    stdout
}
