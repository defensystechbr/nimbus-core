//! Ferramenta do selo de liberação (para o CI e para o responsável de segurança).
//!
//! cargo run --example selo -- gerar-semente <arquivo>        cria a semente de um assinante (guardar OFFLINE)
//! cargo run --example selo -- publica <semente>               mostra a chave pública (para fixar no código)
//! cargo run --example selo -- montar <versao> <commit> <artefato> <emitido_em> <portoes> <saida>
//!        <portoes> = "nome=1,nome=1,..." (1 aprovado, 0 reprovado)
//! cargo run --example selo -- assinar <selo> <semente>        acrescenta a assinatura deste assinante
//! cargo run --example selo -- conferir <selo> <artefato> <chaves_publicas> <versao_minima> <agora>
//!        <chaves_publicas> = arquivo com uma chave pública em hexadecimal por linha
use nimbus_nucleo::{assinatura, selo};
use std::{env, fs, process::exit};

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
fn de_hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("hexadecimal inválido".into()); }
    if !s.len().is_multiple_of(2) { return Err("hexadecimal com tamanho ímpar".into()); }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| "hexadecimal inválido".to_string())).collect()
}
fn falhar(m: &str) -> ! { eprintln!("ERRO: {m}"); exit(1) }

fn ler_selo(caminho: &str) -> (selo::Selo, Vec<([u8; 32], assinatura::AssinaturaHibrida)>) {
    let txt = fs::read_to_string(caminho).unwrap_or_else(|_| falhar("não consegui ler o selo"));
    let (mut versao, mut commit, mut artefato, mut emitido) = (None, None, None, None);
    let (mut portoes, mut ass) = (Vec::new(), Vec::new());
    for l in txt.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let (k, v) = l.split_once('=').unwrap_or_else(|| falhar("linha sem '='"));
        match k {
            "versao" => versao = v.parse().ok(),
            "commit" => commit = Some(v.to_string()),
            "artefato" => { let b = de_hex(v).unwrap_or_else(|e| falhar(&e)); let mut a = [0u8; 64]; if b.len() != 64 { falhar("resumo do artefato com tamanho errado") } a.copy_from_slice(&b); artefato = Some(a); }
            "emitido_em" => emitido = v.parse().ok(),
            "portao" => { let (n, ok) = v.split_once(':').unwrap_or_else(|| falhar("portão sem ':'")); portoes.push((n.to_string(), ok == "1")); }
            "assinatura" => {
                let p: Vec<&str> = v.split(':').collect();
                if p.len() != 3 { falhar("assinatura malformada") }
                let fp = de_hex(p[0]).unwrap_or_else(|e| falhar(&e)); let ed = de_hex(p[1]).unwrap_or_else(|e| falhar(&e)); let pq = de_hex(p[2]).unwrap_or_else(|e| falhar(&e));
                if fp.len() != 32 || ed.len() != 64 { falhar("assinatura com tamanho errado") }
                let (mut f, mut e) = ([0u8; 32], [0u8; 64]); f.copy_from_slice(&fp); e.copy_from_slice(&ed);
                ass.push((f, assinatura::AssinaturaHibrida::de_partes(e, pq)));
            }
            _ => falhar(&format!("campo desconhecido: {k}")),
        }
    }
    let s = selo::Selo { versao: versao.unwrap_or_else(|| falhar("sem versao")), commit: commit.unwrap_or_else(|| falhar("sem commit")),
                         resumo_do_artefato: artefato.unwrap_or_else(|| falhar("sem artefato")), emitido_em: emitido.unwrap_or_else(|| falhar("sem emitido_em")), portoes };
    (s, ass)
}

fn main() {
    let a: Vec<String> = env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("gerar-semente") if a.len() == 2 => {
            let mut s = [0u8; 64];
            rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut s);
            fs::write(&a[1], hex(&s)).unwrap_or_else(|_| falhar("não consegui gravar"));
            let k = assinatura::de_semente(&s);
            println!("Semente gravada em {} — guarde OFFLINE e apague desta máquina.\nImpressão digital: {}", a[1], hex(&k.verificacao().impressao_digital()));
        }
        Some("publica") if a.len() == 2 => {
            let b = de_hex(fs::read_to_string(&a[1]).unwrap_or_else(|_| falhar("não li a semente")).trim()).unwrap_or_else(|e| falhar(&e));
            if b.len() != 64 { falhar("semente deve ter 64 bytes") }
            let mut s = [0u8; 64]; s.copy_from_slice(&b);
            println!("{}", hex(&assinatura::de_semente(&s).verificacao().para_bytes()));
        }
        Some("montar") if a.len() == 7 => {
            let artefato = fs::read(&a[3]).unwrap_or_else(|_| falhar("não li o artefato"));
            let mut t = format!("versao={}\ncommit={}\nartefato={}\nemitido_em={}\n", a[1], a[2], hex(&selo::resumir(&artefato)), a[4]);
            for p in a[5].split(',') { let (n, ok) = p.split_once('=').unwrap_or_else(|| falhar("portão sem '='")); t.push_str(&format!("portao={n}:{ok}\n")); }
            fs::write(&a[6], t).unwrap_or_else(|_| falhar("não gravei o selo"));
            let (s, _) = ler_selo(&a[6]);
            let _forma_canonica = selo::canonico(&s).unwrap_or_else(|_| falhar("selo malformado"));
            println!("Selo montado em {}", a[6]);
        }
        Some("assinar") if a.len() == 3 => {
            let (s, _) = ler_selo(&a[1]);
            let b = de_hex(fs::read_to_string(&a[2]).unwrap_or_else(|_| falhar("não li a semente")).trim()).unwrap_or_else(|e| falhar(&e));
            if b.len() != 64 { falhar("semente deve ter 64 bytes") }
            let mut sm = [0u8; 64]; sm.copy_from_slice(&b);
            let (fp, ass) = selo::assinar(&assinatura::de_semente(&sm), &s).unwrap_or_else(|_| falhar("não assinei"));
            let (ed, pq) = ass.partes();
            let mut t = fs::read_to_string(&a[1]).unwrap_or_default();
            t.push_str(&format!("assinatura={}:{}:{}\n", hex(&fp), hex(&ed), hex(pq)));
            fs::write(&a[1], t).unwrap_or_else(|_| falhar("não gravei"));
            println!("Assinado por {}", hex(&fp));
        }
        Some("conferir") if a.len() == 6 => {
            let (s, ass) = ler_selo(&a[1]);
            let artefato = selo::resumir(&fs::read(&a[2]).unwrap_or_else(|_| falhar("não li o artefato")));
            let chaves: Vec<assinatura::ChaveDeVerificacao> = fs::read_to_string(&a[3]).unwrap_or_else(|_| falhar("não li as chaves"))
                .lines().filter(|l| !l.trim().is_empty())
                .map(|l| assinatura::ChaveDeVerificacao::de_bytes(&de_hex(l.trim()).unwrap_or_else(|e| falhar(&e))).unwrap_or_else(|_| falhar("chave pública inválida"))).collect();
            let vmin: u64 = a[4].parse().unwrap_or_else(|_| falhar("versão mínima inválida"));
            let agora: u64 = a[5].parse().unwrap_or_else(|_| falhar("hora inválida"));
            match selo::conferir(&s, &ass, &artefato, &chaves, vmin, agora) {
                Ok(()) => println!("SELO VÁLIDO: todos os {} portões aprovados, {} assinaturas conferidas.", selo::PORTOES_OBRIGATORIOS.len(), ass.len()),
                Err(e) => falhar(&format!("SELO RECUSADO: {e:?}")),
            }
        }
        _ => { eprintln!("Uso: veja o cabeçalho de examples/selo.rs"); exit(2) }
    }
}
