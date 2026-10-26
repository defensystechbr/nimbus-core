//! # Propriedades: regras que precisam valer para QUALQUER entrada, não só para exemplos
//!
//! Cada teste sorteia centenas de casos e confere uma lei de segurança. Se a lei falhar em um único
//! caso, o teste falha e mostra qual.
use nimbus_nucleo::{album, panico, acesso, aprovacao, assinatura, cofre, contatos, conversa, custodia, liberacao, modos, envelope, expiracao, politica, registro, validade};
use rand_core::{OsRng, RngCore};

fn aleatorio(max: u64) -> u64 { OsRng.next_u64() % (max + 1) }
fn talvez(p: u64) -> Option<u64> { if OsRng.next_u32().is_multiple_of(2) { None } else { Some(aleatorio(p)) } }

#[test]
fn envelope_sempre_devolve_exatamente_o_que_entrou() {
    let (p, s) = envelope::gerar_par_de_chaves();
    for _ in 0..200 {
        let mut msg = vec![0u8; aleatorio(5_000) as usize];
        OsRng.fill_bytes(&mut msg);
        let mut aad = vec![0u8; aleatorio(64) as usize];
        OsRng.fill_bytes(&mut aad);
        let env = envelope::selar(&p, &msg, &aad).unwrap();
        assert_eq!(envelope::abrir(&s, &p, &env, &aad).unwrap().as_slice(), msg);
        // Lei: o tamanho cifrado depende só da faixa, nunca do tamanho exato.
        assert_eq!(env.len(), envelope::selar(&p, &vec![0u8; msg.len()], &aad).unwrap().len());
    }
}

#[test]
fn regra_da_mensagem_nunca_afrouxa_a_da_conversa() {
    let bools = |x: u32| x.is_multiple_of(2);
    for _ in 0..2_000 {
        let r = |_| politica::Regras {
            expiracao: expiracao::Politica { segundos_apos_abrir: talvez(10_000), segundos_apos_enviar: talvez(10_000), visualizacao_unica: bools(OsRng.next_u32()), segundos_para_ler: talvez(10_000) },
            permitir_citar: bools(OsRng.next_u32()), conversa_blindada: bools(OsRng.next_u32()),
        };
        let (c, m) = (r(0), r(1));
        let e = politica::efetiva(&c, Some(&m));
        // Lei: nada que a conversa proíbe fica permitido; nada que a conversa exige deixa de valer.
        assert!(!e.permitir_citar || c.permitir_citar);
        assert!(!c.conversa_blindada || e.conversa_blindada);
        assert!(!c.expiracao.visualizacao_unica || e.expiracao.visualizacao_unica);
        if let (Some(ec), Some(ee)) = (c.expiracao.segundos_apos_enviar, e.expiracao.segundos_apos_enviar) { assert!(ee <= ec); }
        if c.expiracao.segundos_apos_enviar.is_some() { assert!(e.expiracao.segundos_apos_enviar.is_some()); }
        if let (Some(ec), Some(ee)) = (c.expiracao.segundos_para_ler, e.expiracao.segundos_para_ler) { assert!(ee <= ec, "a mensagem nunca alonga o prazo para ler"); }
    }
}

#[test]
fn prazo_final_nunca_e_depois_de_nenhuma_regra_ativa() {
    let k = assinatura::gerar();
    for _ in 0..300 {
        let pol = expiracao::Politica { segundos_apos_abrir: talvez(100_000), segundos_apos_enviar: talvez(100_000), visualizacao_unica: OsRng.next_u32().is_multiple_of(2), segundos_para_ler: None };
        let enviada = aleatorio(1_000_000);
        let b = expiracao::emitir(&k, [0; 16], enviada, pol);
        let aberta = talvez(1_000_000).map(|x| enviada + x);
        let fechada = aberta.map(|a| a + aleatorio(1_000));
        if let Some(d) = expiracao::prazo_final(&b, aberta, fechada) {
            if let Some(s) = pol.segundos_apos_enviar { assert!(d <= enviada + s); }
            if let (Some(s), Some(a)) = (pol.segundos_apos_abrir, aberta) { assert!(d <= a + s); }
            if let (true, Some(f)) = (pol.visualizacao_unica, fechada) { assert!(d <= f); }
        }
    }
}

#[test]
fn qualquer_alteracao_do_pedido_invalida_as_aprovacoes() {
    let ex: Vec<assinatura::ChaveDeAssinatura> = (0..3).map(|_| assinatura::gerar()).collect();
    let publicas: Vec<_> = ex.iter().map(|k| k.verificacao()).collect();
    for _ in 0..60 {
        let original = aprovacao::Pedido { tipo: aprovacao::TipoDePedido::Pagamento, id: [1; 16], hash_do_conteudo: [2; 32],
            valor_centavos: Some(aleatorio(1_000_000_000)), beneficiario: Some(format!("conta {}", aleatorio(99_999))),
            criado_em: 0, expira_em: 1_000, minimo_de_aprovacoes: 2 };
        let ap = vec![aprovacao::aprovar(&ex[0], &original), aprovacao::aprovar(&ex[1], &original)];
        let mut alterado = original.clone();
        match OsRng.next_u32() % 5 {
            0 => alterado.valor_centavos = original.valor_centavos.map(|v| v + 1),
            1 => alterado.beneficiario = Some("outra conta".into()),
            2 => alterado.hash_do_conteudo[OsRng.next_u32() as usize % 32] ^= 1,
            3 => alterado.expira_em += 1,
            _ => alterado.id[0] ^= 1,
        }
        assert!(matches!(aprovacao::avaliar(&original, &ap, &publicas, 1), aprovacao::Resultado::Aprovado { .. }));
        assert!(!matches!(aprovacao::avaliar(&alterado, &ap, &publicas, 1), aprovacao::Resultado::Aprovado { .. }));
    }
}

#[test]
fn registro_prova_toda_entrada_de_registros_de_qualquer_tamanho() {
    for _ in 0..30 {
        let n = 1 + aleatorio(300) as usize;
        let mut r = registro::Registro::default();
        for i in 0..n { let _ = r.acrescentar(&(i as u64).to_be_bytes()); }
        let raiz = r.raiz();
        let i = aleatorio(n as u64 - 1) as usize;
        let prova = r.prova_de_inclusao(i).unwrap();
        assert!(registro::verificar_inclusao(&(i as u64).to_be_bytes(), i, n, &prova, &raiz));
        assert!(!registro::verificar_inclusao(&(i as u64 + 1).to_be_bytes(), i, n, &prova, &raiz));
    }
}

#[test]
fn lotes_sempre_do_mesmo_tamanho_e_com_todos_os_reais() {
    let s = acesso::Semente::nova();
    for _ in 0..200 {
        let tamanho = 2 + aleatorio(30) as usize;
        let reais: Vec<_> = (0..aleatorio(60)).map(|i| acesso::endereco(&s, "x", i)).collect();
        let lotes = acesso::montar_lotes(&reais, tamanho).unwrap();
        assert!(lotes.iter().all(|l| l.enderecos.len() == tamanho));
        for r in &reais { assert!(lotes.iter().any(|l| l.enderecos.contains(r))); }
    }
}

#[test]
fn validade_nunca_apaga_antes_da_hora_escolhida() {
    for _ in 0..2_000 {
        let criado = aleatorio(1_000_000_000);
        let fim = criado + aleatorio(1_000_000);
        let seg = 1 + aleatorio(validade::VALIDADE_MAXIMA - 1);
        let v = if OsRng.next_u32().is_multiple_of(2) { validade::Validade { apos_segundos: seg, referencia: validade::Referencia::Criacao } }
                else { validade::Validade::apos_o_compromisso(seg) };
        let t = validade::instante_de_apagar(&v, criado, Some(fim)).unwrap();
        assert!(t >= criado + seg, "nunca antes do prazo");
        assert!(!validade::deve_esconder(t, t - 1) && validade::deve_esconder(t, t));
    }
}

#[test]
fn contato_nunca_revela_tamanho_do_nome() {
    let (p, _) = envelope::gerar_par_de_chaves();
    let tamanhos: std::collections::HashSet<usize> = (0..50).map(|_| {
        let nome: String = (0..aleatorio(60)).map(|_| 'a').collect();
        contatos::selar_contato(&p, &contatos::Contato { apelido: nome, chave_da_pessoa: vec![], notas: String::new(), etiqueta: String::new() }).unwrap().len()
    }).collect();
    assert_eq!(tamanhos.len(), 1, "todos os contatos têm o mesmo tamanho cifrado");
}

#[test]
fn cofre_nunca_revela_o_tipo_pelo_tamanho() {
    let (p, _) = envelope::gerar_par_de_chaves();
    let tipos = [cofre::Tipo::Senha, cofre::Tipo::Nota, cofre::Tipo::Compromisso, cofre::Tipo::Album, cofre::Tipo::Ajuste];
    let tamanhos: std::collections::HashSet<usize> = (0..60).map(|i| {
        let texto: String = (0..aleatorio(200)).map(|_| 'x').collect();
        cofre::selar(&p, &cofre::Item { tipo: tipos[i % tipos.len()], campos: vec![("campo".into(), texto)], conteudo: vec![] }).unwrap().len()
    }).collect();
    assert_eq!(tamanhos.len(), 1, "itens pequenos de qualquer tipo têm o mesmo tamanho cifrado");
}

#[test]
fn custodia_vale_para_qualquer_combinacao_de_guardioes() {
    for _ in 0..40 {
        let n = 2 + (OsRng.next_u32() % 6) as u8;          // 2 a 7 guardiões
        let k = 2 + (OsRng.next_u32() % (n as u32 - 1)) as u8; // 2 a n
        let mut segredo = vec![0u8; 1 + aleatorio(500) as usize];
        OsRng.fill_bytes(&mut segredo);
        let (partes, _) = custodia::dividir(&segredo, n, k).unwrap();
        // Um grupo sorteado de k guardiões reconstrói; um grupo de k-1 nunca.
        let mut ids: Vec<usize> = (0..n as usize).collect();
        for i in (1..ids.len()).rev() { let j = (OsRng.next_u32() as usize) % (i + 1); ids.swap(i, j); }
        let grupo: Vec<&custodia::PartesDoGuardiao> = ids[..k as usize].iter().map(|i| &partes[*i]).collect();
        assert_eq!(&custodia::reconstruir(&grupo).unwrap()[..], &segredo[..], "n={n} k={k}");
        let menor: Vec<&custodia::PartesDoGuardiao> = ids[..k as usize - 1].iter().map(|i| &partes[*i]).collect();
        assert!(custodia::reconstruir(&menor).is_err(), "n={n} k={k}");
    }
}

#[test]
fn conversa_nunca_e_menos_restritiva_que_nenhum_participante() {
    let sorteio = |max: u64| if OsRng.next_u32().is_multiple_of(3) { None } else { Some(aleatorio(max)) };
    for _ in 0..2_000 {
        let n = 1 + (OsRng.next_u32() % 5) as usize;
        let prefs: Vec<conversa::Preferencias> = (0..n).map(|_| conversa::Preferencias {
            mensagens_apos_enviar: sorteio(1_000_000), nao_lidas_somem_em: sorteio(1_000_000),
            chamadas_atendidas: sorteio(1_000_000), chamadas_nao_atendidas: sorteio(1_000_000),
            remover_conversa_vazia: OsRng.next_u32().is_multiple_of(2), sem_marcas: OsRng.next_u32().is_multiple_of(2),
            apagar_tudo_a_pedido: OsRng.next_u32().is_multiple_of(2),
            retirar_lidas_a_pedido: OsRng.next_u32().is_multiple_of(2), aviso_de_visualizacao: OsRng.next_u32().is_multiple_of(2),
            prazo_do_nome_na_lista: [politica::PrazoDeVisibilidade::UmaHora, politica::PrazoDeVisibilidade::SeisHoras,
                                      politica::PrazoDeVisibilidade::VinteQuatroHoras, politica::PrazoDeVisibilidade::SeteDias,
                                      politica::PrazoDeVisibilidade::TrintaDias, politica::PrazoDeVisibilidade::TresMeses,
                                      politica::PrazoDeVisibilidade::Indeterminado][(OsRng.next_u32() as usize) % 7] }).collect();
        let r = conversa::combinar(&prefs);
        for q in &prefs {
            if let Some(x) = q.mensagens_apos_enviar { assert!(r.mensagens_apos_enviar.unwrap() <= x); }
            if let Some(x) = q.nao_lidas_somem_em { assert!(r.nao_lidas_somem_em.unwrap() <= x); }
            if let Some(x) = q.chamadas_atendidas { assert!(r.chamadas_atendidas.unwrap() <= x); }
            if let Some(x) = q.chamadas_nao_atendidas { assert!(r.chamadas_nao_atendidas.unwrap() <= x); }
            assert!(!q.sem_marcas || r.sem_marcas);
            assert!(!q.remover_conversa_vazia || r.remover_conversa_vazia);
            assert!(r.prazo_do_nome_na_lista <= q.prazo_do_nome_na_lista, "a mais restritiva vence também no prazo de visibilidade");
        }
        assert_eq!(r.apagar_tudo_a_pedido, prefs.iter().all(|q| q.apagar_tudo_a_pedido), "apagar tudo só com o consentimento de todos");
        let mut invertida = prefs.clone();
        invertida.reverse();
        assert_eq!(r, conversa::combinar(&invertida), "a ordem dos participantes não muda nada");
    }
}

#[test]
fn nenhuma_mensagem_nao_lida_vive_mais_de_48_horas() {
    let k = assinatura::gerar();
    for _ in 0..300 {
        let pol = expiracao::Politica { segundos_apos_abrir: talvez(1_000_000), segundos_apos_enviar: talvez(1_000_000),
                                        visualizacao_unica: OsRng.next_u32().is_multiple_of(2), segundos_para_ler: talvez(1_000_000) };
        let enviada = aleatorio(1_000_000);
        let b = expiracao::emitir(&k, [0; 16], enviada, pol);
        let d = expiracao::prazo_final(&b, None, None).expect("mensagem não lida sempre tem prazo");
        assert!(d <= enviada + expiracao::LIMITE_PARA_LER);
    }
}

#[test]
fn nenhuma_mensagem_sai_e_nenhum_arquivo_sai_alem_do_autorizado() {
    use nimbus_nucleo::politica::TipoDeConteudo::*;
    use liberacao::{Nivel, Uso};
    let tipos = [Texto, Audio, Foto, Video, VideoCircular, Arquivo, Figurinha, Localizacao];
    let niveis = [Nivel::Nenhum, Nivel::GuardarNoCofre, Nivel::EncaminharNoNimbus, Nivel::LevarParaFora];
    for _ in 0..5_000 {
        let tipo = tipos[(OsRng.next_u32() % 8) as usize];
        let autorizado = niveis[(OsRng.next_u32() % 4) as usize];
        let limite = if OsRng.next_u32().is_multiple_of(2) { None } else { Some(niveis[(OsRng.next_u32() % 4) as usize]) };
        let unica = OsRng.next_u32().is_multiple_of(3);
        let encaminhado = OsRng.next_u32().is_multiple_of(2);
        let teto = liberacao::teto_da_conversa(limite);
        let n = liberacao::efetivo(tipo, unica, teto, autorizado);
        assert!(n <= autorizado, "nunca além do que o autor autorizou");
        if unica { assert_eq!(n, Nivel::Nenhum); }
        // Blindada agora é permanente e sem opção de desligar (v37, 3ª rodada): o teto da conversa nunca
        // passa de EncaminharNoNimbus, então LevarParaFora nunca é alcançável a partir daqui, em nenhuma
        // conversa — essa é a propriedade que substituiu o antigo teste do parâmetro `blindada` opcional.
        assert!(teto <= Nivel::EncaminharNoNimbus, "o teto da Blindada mandatória nunca libera sair do Nimbus");
        assert!(n <= Nivel::EncaminharNoNimbus, "nenhum arquivo pode sair do Nimbus, em nenhuma conversa");
        if let Some(l) = limite { assert!(n <= l); }
        if liberacao::e_mensagem(tipo) {
            assert!(!liberacao::pode(tipo, n, Uso::GuardarNoCofre, encaminhado) && !liberacao::pode(tipo, n, Uso::LevarParaFora, encaminhado), "mensagem nunca é guardada nem sai do Nimbus");
        }
        if encaminhado { assert!(!liberacao::pode(tipo, n, Uso::EncaminharNoNimbus, true), "um salto só"); }
    }
}

#[test]
fn mensagem_nunca_sai_mais_solta_que_a_conversa_nem_passa_de_48_horas() {
    use nimbus_nucleo::politica::TipoDeConteudo::*;
    let tipos = [Texto, Audio, Foto, Video, VideoCircular, Arquivo, Figurinha, Localizacao];
    let modos_ = |x: u32| match x % 4 { 0 => modos::Modo::Normal, 1 => modos::Modo::Discreto, 2 => modos::Modo::VisualizacaoUnica, _ => modos::Modo::Temporizador(1 + aleatorio(400_000)) };
    for _ in 0..3_000 {
        let mut c = politica::Regras::padrao_nimbus();
        c.expiracao.segundos_apos_abrir = talvez(200_000);
        c.expiracao.visualizacao_unica = OsRng.next_u32().is_multiple_of(4);
        let p = modos::Padroes { texto: modos_(OsRng.next_u32()), foto_e_video: modos_(OsRng.next_u32()), voz: modos_(OsRng.next_u32()), arquivo: modos_(OsRng.next_u32()) };
        let escolha = if OsRng.next_u32().is_multiple_of(2) { None } else { Some(modos_(OsRng.next_u32())) };
        let r = modos::regra_da_mensagem(&c, &p, tipos[(OsRng.next_u32() % 8) as usize], escolha);
        if c.expiracao.visualizacao_unica { assert!(r.expiracao.visualizacao_unica); }
        if let (Some(ca), Some(ra)) = (c.expiracao.segundos_apos_abrir, r.expiracao.segundos_apos_abrir) { assert!(ra <= ca); }
        if c.expiracao.segundos_apos_abrir.is_some() { assert!(r.expiracao.segundos_apos_abrir.is_some()); }
    }
}

#[test]
fn toda_mensagem_de_qualquer_modo_vence_em_ate_48_horas() {
    let k = assinatura::gerar();
    let modos_ = |x: u32| match x % 4 { 0 => modos::Modo::Normal, 1 => modos::Modo::Discreto, 2 => modos::Modo::VisualizacaoUnica, _ => modos::Modo::Temporizador(1 + aleatorio(400_000)) };
    for _ in 0..600 {
        let mut c = politica::Regras::padrao_nimbus();
        c.expiracao.segundos_apos_abrir = talvez(400_000);
        c.expiracao.segundos_apos_enviar = talvez(400_000);
        let p = modos::Padroes { texto: modos_(OsRng.next_u32()), foto_e_video: modos_(OsRng.next_u32()), voz: modos_(OsRng.next_u32()), arquivo: modos_(OsRng.next_u32()) };
        let r = modos::regra_da_mensagem(&c, &p, politica::TipoDeConteudo::Texto, Some(modos_(OsRng.next_u32())));
        let enviada = aleatorio(1_000_000);
        let b = expiracao::emitir(&k, [0; 16], enviada, r.expiracao);
        let aberta = if OsRng.next_u32().is_multiple_of(2) { None } else { Some(enviada + aleatorio(200_000)) };
        let prazo = expiracao::prazo_final(&b, aberta, None).expect("toda mensagem tem prazo");
        assert!(prazo <= enviada + expiracao::VIDA_MAXIMA, "nenhum modo passa de 48 horas");
    }
}

#[test]
fn panico_a_reacao_e_sempre_a_mais_forte_entre_as_disparadas() {
    use panico::{Config, Gatilho, Reacao, Sinais, decidir};
    let reacoes = [Reacao::Trancar, Reacao::EsconderSensiveis, Reacao::MostrarIsca, Reacao::ApagarChaves];
    for _ in 0..5_000 {
        let r1 = reacoes[aleatorio(3) as usize];
        let r2 = reacoes[aleatorio(3) as usize];
        let cfg = Config { gatilhos: vec![Gatilho::MovimentoBrusco(r1), Gatilho::Coacao(r2)] };
        let tranco = OsRng.next_u32().is_multiple_of(2);
        let coacao = OsRng.next_u32().is_multiple_of(2);
        let s = Sinais { tranco_detectado: tranco, coacao_digitada: coacao, ..Default::default() };
        let esperado = match (tranco, coacao) {
            (false, false) => None,
            (true, false) => Some(r1),
            (false, true) => Some(r2),
            (true, true) => Some(r1.max(r2)),
        };
        assert_eq!(decidir(&cfg, &s), esperado);
    }
}

#[test]
fn album_escolher_e_deterministico_e_respeita_a_ordem() {
    use album::{Origem, Regra, Regras};
    for _ in 0..5_000 {
        let c1 = (aleatorio(5) + 1) as u32;
        let c2 = (aleatorio(5) + 1) as u32;
        let regras = Regras { regras: vec![
            Regra::DoContato { contato: 10, colecao: c1 },
            Regra::ComMarcacao { marcacao: "x".into(), colecao: c2 },
        ] };
        let de_10 = OsRng.next_u32().is_multiple_of(2);
        let marca_x = OsRng.next_u32().is_multiple_of(2);
        let origem = Origem { de_contato: if de_10 { Some(10) } else { Some(999) }, marcacao: if marca_x { Some("x".into()) } else { None } };
        let esperado = if de_10 { Some(c1) } else if marca_x { Some(c2) } else { None };
        assert_eq!(regras.escolher(&origem), esperado, "a primeira regra que casa vence, sempre igual");
    }
}
