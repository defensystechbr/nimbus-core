//! # Testes em aparelho real — proteção contra celular corrompido (provas C76, C77 do manual)
//!
//! Estes testes NÃO rodam sozinhos (`cargo test`): eles precisam de um aparelho físico preparado pelo time
//! de segurança. Rode com `cargo test --test aparelho_real -- --ignored --nocapture` no aparelho de teste.
//!
//! Cada teste chama a MESMA função de decisão do app (`integridade::avaliar`) com os sinais que o aparelho
//! comprometido produz. Assim, a prova é sobre o código que vai para produção, não sobre uma cópia. O time
//! preenche os sinais lendo App Attest (iOS) ou Play Integrity + Key Attestation (Android) no aparelho.
//!
//! Roteiro de preparação de cada cenário está em `docs/roteiro-aparelho-real.md` do kit.
use nimbus_nucleo::integridade::{avaliar, permissoes, sessao_permitida, Inicio, Nivel, OrigemDaAtestacao, Plataforma, Sinais};

/// Lê os sinais reais do aparelho. NO KIT, o time substitui o corpo desta função pela leitura da atestação
/// da plataforma (ponte da Tarefa T1). Aqui ela devolve sinais fixos por cenário, para o teste ser executável.
fn sinais_do_aparelho(cenario: &str) -> Sinais {
    let base = Sinais { app_atestado: true, aparelho_integro: true, ..Default::default() };
    match cenario {
        "limpo" => base,
        "root_escondido_magisk" => Sinais { inicio: Some(Inicio::NaoVerificado), aparelho_integro: false, ..base },
        "frida" => Sinais { ganchos_detectados: true, ..base },
        "emulador" => Sinais { emulador: true, app_atestado: false, ..base },
        "chave_atestacao_vazada" => Sinais { atestacao_revogada: true, ..base },
        "atestacao_software" => Sinais { origem_da_atestacao: Some(OrigemDaAtestacao::Software), ..base },
        "jailbreak" => Sinais { root_ou_jailbreak: true, ..base },
        "bootloader_destravado" => Sinais { inicio: Some(Inicio::NaoVerificado), ..base },
        "sem_senha" => Sinais { sem_bloqueio_de_tela: true, ..base },
        "grapheneos" => Sinais { inicio: Some(Inicio::AlternativoPermitido), origem_da_atestacao: Some(OrigemDaAtestacao::ChipDedicado), ..base },
        outro => panic!("cenário desconhecido: {outro}"),
    }
}

fn plataforma() -> Plataforma { Plataforma::Android }

/// Todo aparelho comprometido precisa ficar VERMELHO e não receber sessão no servidor.
#[test]
#[ignore = "requer aparelho físico preparado"]
fn c76_aparelhos_comprometidos_sao_vermelhos() {
    let comprometidos = ["root_escondido_magisk", "frida", "emulador", "chave_atestacao_vazada",
                         "atestacao_software", "jailbreak", "bootloader_destravado", "sem_senha"];
    for cenario in comprometidos {
        let v = avaliar(plataforma(), &sinais_do_aparelho(cenario));
        assert_eq!(v.nivel, Nivel::Vermelho, "cenário '{cenario}' deveria ser vermelho; motivos: {:?}", v.motivos);
        assert!(!sessao_permitida(&v), "cenário '{cenario}' não pode receber sessão");
        let p = permissoes(v.nivel);
        assert!(!p.abrir_conteudo && !p.celular_virtual, "vermelho não abre conteúdo nem celular virtual");
        println!("OK  {cenario}: VERMELHO — {:?}", v.motivos);
    }
}

/// O aparelho de teste limpo precisa passar (senão o teste não prova nada).
#[test]
#[ignore = "requer aparelho físico preparado"]
fn c76_aparelho_limpo_de_referencia_e_verde() {
    let v = avaliar(plataforma(), &sinais_do_aparelho("limpo"));
    assert_eq!(v.nivel, Nivel::Verde, "o aparelho de referência precisa estar limpo; motivos: {:?}", v.motivos);
    println!("OK  aparelho de referência: VERDE");
}

/// Sistema alternativo endurecido (GrapheneOS) com atestação de hardware é aceito.
#[test]
#[ignore = "requer aparelho físico com GrapheneOS"]
fn c76_grapheneos_com_atestacao_de_hardware_e_verde() {
    let v = avaliar(plataforma(), &sinais_do_aparelho("grapheneos"));
    assert_eq!(v.nivel, Nivel::Verde, "GrapheneOS com chave permitida deve ser verde; motivos: {:?}", v.motivos);
    println!("OK  GrapheneOS: VERDE");
}
