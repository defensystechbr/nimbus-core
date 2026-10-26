//! # Exemplo executável: o caminho completo de uma mensagem no NimbusChat
//!
//! Rode com:  `cargo run --example fluxo_completo`
//!
//! Mostra, em ordem, cada módulo do núcleo sendo usado como o app real usará.
//! Serve de guia para o time: cada passo cita o requisito do Manual Técnico v28.

use nimbus_nucleo::{modos, recuperacao, apresentacao, direto, inspecao, liberacao, conversa, corporativo, custodia, assinatura, cofre, contatos, convites, notificacao, rede, acesso, pacote, aprovacao, registro, coacao, envelope, escudo, expiracao, higiene, historico, integridade, memoria, politica, sessoes, trafego, validade, voz};
use std::collections::HashMap;

fn passo(n: u32, t: &str) { println!("\n[{n}] {t}"); }

fn main() {
    println!("NimbusChat — fluxo completo de ponta a ponta (núcleo de segurança)");

    // 1. Verificação do aparelho ao abrir o app (DV2, DV3)
    passo(1, "Verificação do aparelho ao abrir");
    let sinais = integridade::Sinais { app_atestado: true, aparelho_integro: true, ..Default::default() };
    let v = integridade::avaliar(integridade::Plataforma::Ios, &sinais);
    let perm = integridade::permissoes(v.nivel);
    println!("    nível = {:?}; pode abrir conteúdo = {}", v.nivel, perm.abrir_conteudo);
    assert!(perm.abrir_conteudo);

    // 2. Cadastro sem telefone: só um par de chaves gerado no aparelho (HY7, ID1)
    passo(2, "Cadastro sem telefone, e-mail ou documento");
    let (pub_ana, sec_ana) = envelope::gerar_par_de_chaves();
    println!("    chave pública da Ana: {} bytes (é a única “identidade” que existe)", pub_ana.para_bytes().len());

    // 3. PIN real e PIN de coação (ANO7)
    passo(3, "PIN real e PIN de coação");
    let pin = coacao::Verificador::novo([7; 16], b"481516", b"000000").unwrap();
    println!("    PIN real → {:?}", pin.avaliar(b"481516"));
    println!("    PIN de coação → {:?} (abre cofre-isca, nada muda na tela)", pin.avaliar(b"000000"));

    // 4. Sessão única e anti-clone no servidor (ID2, CG1)
    passo(4, "Conexão ao servidor com sessão única");
    let mut reg = sessoes::Registro::default();
    let (conta, aparelho) = ([1u8; 32], [2u8; 32]);
    reg.registrar_aparelho(conta, aparelho);
    println!("    aparelho da Ana conecta → {:?}", reg.conectar(conta, aparelho, true, true, 1));
    println!("    alguém sem credencial tenta 100 vezes → {:?}; a Ana continua conectada: {}",
        (0..100).map(|_| reg.conectar(conta, [8u8; 32], false, true, 1)).last().unwrap(), reg.tem_sessao_ativa(&conta));
    println!("    clone com a credencial roubada tenta conectar → todas as sessões derrubadas: {}",
        matches!(reg.conectar(conta, [9u8; 32], true, true, 1), sessoes::Resultado::TodasDerrubadas { .. }));
    println!("    aparelhos ativos agora: {}; conexões aceitas hoje: {}", reg.aparelhos_ativos_agora(&conta).len(), reg.conexoes_hoje(&conta, 1));

    // 5. Regra da conversa + regra da mensagem (CH2, NC13)
    passo(5, "Temporizador por conversa e por mensagem");
    let regra_conversa = politica::Regras { expiracao: expiracao::Politica { segundos_apos_abrir: None, segundos_apos_enviar: Some(86_400), visualizacao_unica: false, segundos_para_ler: None }, ..politica::Regras::padrao_nimbus() };
    let regra_foto = politica::Regras { expiracao: expiracao::Politica { segundos_apos_abrir: Some(10), segundos_apos_enviar: None, visualizacao_unica: true, segundos_para_ler: None }, ..politica::Regras::padrao_nimbus() };
    let efetiva = politica::efetiva(&regra_conversa, Some(&regra_foto));
    println!("    foto: visualização única = {}, some 10 s após abrir, e no máximo 24 h após enviar", efetiva.expiracao.visualizacao_unica);
    println!("    pode citar? {}; abrir de novo depois de vista? {}", politica::pode(&efetiva, politica::Acao::Citar, false), politica::pode(&efetiva, politica::Acao::AbrirNovamente, true));

    // 6. Bilhete de expiração assinado pelo servidor (CH2)
    passo(6, "Servidor assina o prazo (mudar o relógio do celular não adianta)");
    let chave_servidor = assinatura::gerar();
    let bilhete = expiracao::emitir(&chave_servidor, [5; 16], 1_000, efetiva.expiracao);
    println!("    assinatura híbrida Ed25519 + ML-DSA-87 confere = {}", expiracao::conferir(&chave_servidor.verificacao(), &bilhete));

    // 7. Escudo de Conversa antes de mostrar (EC4, EC5)
    passo(7, "Escudo de Conversa: todos os participantes estão seguros?");
    let (k_ana, k_bia) = (assinatura::gerar(), assinatura::gerar());
    let (id_ana, id_bia, conv) = ([1u8; 32], [2u8; 32], [3u8; 16]);
    let mut participantes = HashMap::new();
    let _ = participantes.insert(id_ana, k_ana.verificacao());
    let _ = participantes.insert(id_bia, k_bia.verificacao());
    let mut esc = escudo::Escudo::novo(conv, participantes);
    esc.receber(&escudo::assinar_estado(&k_ana, id_ana, conv, 1, 100, integridade::Nivel::Verde, false), 100);
    esc.receber(&escudo::assinar_estado(&k_bia, id_bia, conv, 1, 100, integridade::Nivel::Verde, true), 100);
    match esc.decidir(101) {
        escudo::Acao::Mostrar => println!("    conversa visível"),
        escudo::Acao::Borrar(motivos) => for (quem, motivo) in motivos {
            let nome = if quem == id_bia { "Bia" } else { "Ana" };
            println!("    conversa BORRADA em todos os aparelhos — {nome}: {motivo:?}");
        },
    }

    // 8. Histórico no servidor cego + apagamento criptográfico (EF5, SV2)
    passo(8, "Guardar no servidor cego e abrir só no aparelho");
    let mut servidor = historico::ServidorCego::default();
    let mut chaves = historico::ChavesDoDono::novas(3_600);
    let id = historico::guardar(&mut servidor, &mut chaves, b"contrato assinado", Some(5_000)).unwrap();
    println!("    lida no aparelho: {:?}", String::from_utf8_lossy(&historico::ler(&servidor, &chaves, &id).unwrap()));
    let _ = chaves.destruir_janelas_vencidas(7_200);
    println!("    depois do prazo: {:?} (mesmo com cópia no servidor)", historico::ler(&servidor, &chaves, &id).err());

    // 9. Envelope direto para um contato (CRY1)
    passo(9, "Envelope pós-quântico para um contato");
    let env = envelope::selar(&pub_ana, b"oi, Ana", b"conversa-3").unwrap();
    println!("    envelope: {} bytes; aberto: {:?}", env.len(), String::from_utf8_lossy(&envelope::abrir(&sec_ana, &pub_ana, &env, b"conversa-3").unwrap()));

    // 10. Tráfego de cobertura (ANO3)
    passo(10, "Tráfego de cobertura: silêncio e conversa parecem iguais");
    let mut batida = trafego::Batida::nova(0);
    batida.enfileirar();
    let pacotes: Vec<_> = (0..3).map(|i| batida.na_hora(i * trafego::INTERVALO_MS).unwrap()).collect();
    println!("    pacotes enviados: {:?}", pacotes);

    // 11. Modulador de voz antes de cifrar (VM1, VM5)
    passo(11, "Modulador de voz no aparelho, antes de cifrar");
    let mut modulador = voz::DeslocadorDeTom::para_chamada(voz::Predefinicao::Grave, 1024, 1.0);
    let mut audio: Vec<f32> = (0..480).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
    modulador.processar(&mut audio);
    println!("    fator de tom sorteado nesta chamada: {:.3}", modulador.fator());
    let cfg = voz::ConfigNeural::novo(voz::VozAlvo::Neutra, 48_000);
    let lat = voz::OrcamentoLatencia { bloco_ms: 20, inferencia_ms: 25, rede_ms: 60 };
    println!("    Nível 2 neural: bloco de {} amostras; latência total {} ms; serve para chamada = {}", cfg.bloco, lat.total_ida_ms(), lat.aceitavel_para_chamada());

    // 12. Agenda que se apaga sozinha (VAL1–VAL8)
    passo(12, "Compromisso que se apaga 1 hora depois de acontecer");
    let mut serv = historico::ServidorCego::default();
    let mut ch = historico::ChavesDoDono::novas(validade::HORA);
    let fim_da_reuniao = 20_000;
    let id_ag = validade::guardar_com_validade(&mut serv, &mut ch, b"Reuniao reservada - sala 4", Some(&validade::Validade::apos_o_compromisso(validade::HORA)), 1_000, Some(fim_da_reuniao)).unwrap();
    println!("    antes: {:?}", String::from_utf8_lossy(&historico::ler(&serv, &ch, &id_ag).unwrap()));
    let _ = ch.destruir_janelas_vencidas(fim_da_reuniao + 2 * validade::HORA);
    println!("    depois: {:?} — nem cópia guardada abre", historico::ler(&serv, &ch, &id_ag).err());

    // 13. Foto sai sem GPS, modelo do celular ou autor (HIG1–HIG6)
    passo(13, "Higiene do arquivo antes de enviar");
    let foto = [&[0xFFu8, 0xD8][..], &[0xFF, 0xE1, 0x00, 0x16], b"Exif\0\0GPS-23.5,-46.6", &[0xFF, 0xDA, 0x00, 0x04, 0x01, 0x00, 0xAB, 0xFF, 0xD9]].concat();
    let limpa = higiene::limpar(&foto).unwrap();
    println!("    antes tinha GPS: {}; depois tem GPS: {}; nome enviado: {}",
        foto.windows(3).any(|w| w == b"GPS"), limpa.windows(3).any(|w| w == b"GPS"), higiene::nome_neutro("jpg"));

    // 15. Memória blindada: segredo nunca aparece em registro ou relatório de falha (RZ11)
    passo(14, "Memória blindada");
    let segredo = memoria::Segredo::de(b"texto aberto da mensagem");
    println!("    como aparece se alguém tentar registrar: {segredo:?}");

    // 16. Registro público: aparelho fantasma é denunciado (BL1, BC2)
    passo(15, "Registro público de chaves (a parte útil do blockchain)");
    let mut reg_chaves = registro::Registro::default();
    for i in 0..8 { let _ = reg_chaves.acrescentar(format!("aparelho-{i}").as_bytes()); }
    let prova = reg_chaves.prova_de_inclusao(3).unwrap();
    let raiz = reg_chaves.raiz();
    println!("    aparelho-3 está no registro? {} (prova com {} hashes)", registro::verificar_inclusao(b"aparelho-3", 3, 8, &prova, &raiz), prova.len());
    println!("    aparelho fantasma passa? {}", registro::verificar_inclusao(b"aparelho-FANTASMA", 3, 8, &prova, &raiz));

    // 17. Pagamento aprovado por 2 de 3 executivos, com vinculação dinâmica (ASS1–ASS12)
    passo(16, "Aprovação de pagamento: 2 de 3 executivos, valor e conta amarrados");
    let execs: Vec<assinatura::ChaveDeAssinatura> = (0..3).map(|_| assinatura::gerar()).collect();
    let publicas: Vec<_> = execs.iter().map(|k| k.verificacao()).collect();
    let ordem = aprovacao::Pedido { tipo: aprovacao::TipoDePedido::Pagamento, id: [4; 16], hash_do_conteudo: aprovacao::hash_do_conteudo(b"ordem 881"),
        valor_centavos: Some(12_500_000), beneficiario: Some("Fornecedor Alfa · conta 0001-9".into()), criado_em: 0, expira_em: 3_600, minimo_de_aprovacoes: 2 };
    let ap = vec![aprovacao::aprovar(&execs[0], &ordem), aprovacao::aprovar(&execs[1], &ordem)];
    println!("    ordem original: {:?}", aprovacao::avaliar(&ordem, &ap, &publicas, 10));
    let golpe = aprovacao::Pedido { beneficiario: Some("Conta do golpista".into()), ..ordem.clone() };
    println!("    mesma aprovação com a conta trocada: {:?}", aprovacao::avaliar(&golpe, &ap, &publicas, 10));
    println!("    carência para beneficiário novo: {} h", aprovacao::carencia_em_segundos(false, 12_500_000, 5_000_000, 86_400) / 3_600);

    // 18. Até o nome da foto é segredo (MTA1–MTA3)
    passo(17, "Nome, tipo e pasta cifrados junto com o conteúdo");
    let (pub_cofre, sec_cofre) = envelope::gerar_par_de_chaves();
    let meta = pacote::Metadados { nome: "Passaporte_2026.jpg".into(), tipo: "image/jpeg".into(), criado_em: 1_790_000_000, pasta: "Documentos pessoais".into() };
    let bloco_item = pacote::selar_item(&pub_cofre, &meta, b"bytes da foto").unwrap();
    println!("    o servidor vê o nome? {}", bloco_item.windows(10).any(|w| w == b"Passaporte"));
    let (m2, _) = pacote::abrir_item(&sec_cofre, &pub_cofre, &bloco_item).unwrap();
    println!("    no aparelho do dono, na memória: {} (pasta: {})", m2.nome, m2.pasta);

    // 19. Pedidos que o servidor não consegue ligar entre si (ANO9–ANO14)
    passo(18, "Lotes de tamanho fixo, endereços derivados e cobertura");
    let semente = acesso::Semente::nova();
    let meus: Vec<_> = (0..3).map(|i| acesso::endereco(&semente, "agenda", i)).collect();
    let lotes = acesso::montar_lotes(&meus, 16).unwrap();
    println!("    3 blocos reais viraram {} lote(s) de {} pedidos, embaralhados com cobertura", lotes.len(), lotes[0].enderecos.len());
    println!("    o aparelho não guarda a lista: recalcula o endereço 2 = {}", acesso::endereco(&semente, "agenda", 2) == meus[2]);

    // 20. Tor quando importa, velocidade no resto (NET1–NET14)
    passo(19, "Rota escolhida para cada situação");
    use rede::{EstadoDaRede::*, Operacao::*, Preferencia::*};
    for (pref, op, estado, descr) in [
        (Automatico, Mensagem, Normal, "Automático · mensagem (Blindada não força mais Tor aqui, v37 5ª rodada)"),
        (Automatico, NovoContato, Normal, "Automático · adicionar contato"),
        (Automatico, Mensagem, RotaRapidaBloqueada, "Automático · rede bloqueando"),
        (SempreTor, Mensagem, Normal, "Sempre Tor · mensagem"),
        (Rapido, NovoContato, Normal, "Rápido (sem Tor) · adicionar contato"),
        (Automatico, Chamada, Normal, "Qualquer modo · chamada"),
    ] {
        println!("    {descr}: {:?}", rede::decidir(pref, op, estado));
    }
    println!("    Tor falhou no modo Sempre Tor: {:?}", rede::ao_falhar(SempreTor, rede::Rota::Tor));

    // 21. Notificações em cadência fixa (NOT1–NOT7)
    passo(20, "Uma notificação por aparelho a cada janela, real ou de cobertura");
    let (pub_ap, sec_ap) = envelope::gerar_par_de_chaves();
    let rodada = notificacao::montar_rodada(&[pub_ap.clone(), pub_ap.clone()], &[true, false]);
    println!("    tamanhos vistos por quem entrega: {} e {} bytes", rodada[0].len(), rodada[1].len());
    println!("    no aparelho: {:?} / {:?}", notificacao::ao_receber(&sec_ap, &pub_ap, &rodada[0]), notificacao::ao_receber(&sec_ap, &pub_ap, &rodada[1]));

    // 22. O nome que você salva no contato é segredo (CTT1–CTT8)
    passo(21, "Contato salvo como “Mãe”: o que o servidor recebe");
    let (dono_pub, dono_sec) = envelope::gerar_par_de_chaves();
    let (mae, _) = envelope::gerar_par_de_chaves();
    let c = contatos::Contato { apelido: "Mãe".into(), chave_da_pessoa: mae.para_bytes(), notas: "ligar domingo".into(), etiqueta: "Família".into() };
    let bloco_contato = contatos::selar_contato(&dono_pub, &c).unwrap();
    println!("    o servidor vê “Mãe”? {}; tamanho fixo: {} bytes", bloco_contato.windows(4).any(|w| w == "Mãe".as_bytes()), bloco_contato.len());
    println!("    no aparelho do dono: {}", contatos::abrir_contato(&dono_sec, &dono_pub, &bloco_contato).unwrap().apelido);

    // 23. Sem telefone e sem spam: só com convite (SPAM1–SPAM6)
    passo(22, "Convite de uso único");
    let mut servidor_convites = convites::Convites::default();
    let convite = servidor_convites.criar(0, 86_400);
    println!("    1º uso: {:?}; 2º uso: {:?}", servidor_convites.resgatar(&convite, 10), servidor_convites.resgatar(&convite, 11));

    // 24. Cofre universal: o servidor não sabe nem o tipo do item (COF1–COF8)
    passo(23, "Nome do álbum, senha e compromisso: por fora, todos iguais");
    let (cofre_pub, _) = envelope::gerar_par_de_chaves();
    let album = cofre::selar(&cofre_pub, &cofre::Item { tipo: cofre::Tipo::Album, campos: vec![("nome".into(), "Viagem Espanha".into())], conteudo: vec![] }).unwrap();
    let senha = cofre::selar(&cofre_pub, &cofre::Item { tipo: cofre::Tipo::Senha, campos: vec![("site".into(), "banco".into())], conteudo: b"S3nh@".to_vec() }).unwrap();
    println!("    álbum: {} bytes; senha: {} bytes — o servidor não distingue um do outro", album.len(), senha.len());

    // 25. Assinatura híbrida pós-quântica (PQS1–PQS6)
    passo(24, "Assinatura dupla Ed25519 + ML-DSA-87 (CNSA 2.0)");
    let k = assinatura::gerar();
    let a = k.assinar(b"demo", b"ordem 881").unwrap();
    println!("    {:?}; confere = {}", a, k.verificacao().verificar(b"demo", b"ordem 881", &a));

    // 26. Plano corporativo: suspender sem ler (COR1–COR24)
    passo(25, "Empresa suspende o Marcos: sem ler nada, e o aparelho dele apaga as chaves");
    let admins: Vec<assinatura::ChaveDeAssinatura> = (0..3).map(|_| assinatura::gerar()).collect();
    let (mestra_pub, mestra_sec) = envelope::gerar_par_de_chaves();
    let empresa = corporativo::Empresa { id: [1; 16], administradores: admins.iter().map(|a| a.verificacao()).collect(),
        politica: corporativo::Politica::padrao(), chave_mestra_publica: Some(mestra_pub), assentos_contratados: 10 };
    let (ana_c, marcos_c) = ([1u8; 32], [2u8; 32]);
    let mut l1 = corporativo::Lista { empresa: [1; 16], numero: 1, emitida_em: 0, valida_ate: 900,
        ativas: [ana_c, marcos_c].into_iter().collect(), suspensas: Default::default(), minimo_declarado: 1, assinaturas: vec![] };
    l1.assinaturas = vec![corporativo::assinar_lista(&admins[0], &l1)];
    let mut l2 = corporativo::Lista { numero: 2, emitida_em: 60, valida_ate: 960, ativas: [ana_c].into_iter().collect(),
        suspensas: [marcos_c].into_iter().collect(), assinaturas: vec![], ..l1.clone() };
    l2.assinaturas = vec![corporativo::assinar_lista(&admins[0], &l2)];
    println!("    aparelho do Marcos: {:?}", corporativo::decidir_no_aparelho(&empresa, Some(&l1), Some(&l2), &marcos_c, 70).0);
    println!("    aparelho da Ana: {:?}", corporativo::decidir_no_aparelho(&empresa, Some(&l1), Some(&l2), &ana_c, 70).0);

    // 27. Recuperação só com a chave mestra: 2 de 3 administradores + 2 de 3 guardiões
    passo(26, "Recuperar os dados do Marcos para a Ana");
    let pacote = corporativo::selar_pacote_de_recuperacao(empresa.chave_mestra_publica.as_ref().unwrap(), &marcos_c, b"chaves da conta do Marcos").unwrap();
    let (partes, _) = custodia::dividir(&mestra_sec.para_bytes(), 3, 2).unwrap();
    let mut ordem = corporativo::OrdemDeRecuperacao { empresa: [1; 16], numero: 1, conta: marcos_c, destino: ana_c,
        emitida_em: 70, valida_ate: 70 + corporativo::CARENCIA_DA_RECUPERACAO + 600, minimo_declarado: 2, assinaturas: vec![] };
    ordem.assinaturas = vec![corporativo::assinar_ordem(&admins[0], &ordem)];
    println!("    com 1 administrador: {:?}", corporativo::autorizar_recuperacao(&empresa, &l2, &ordem, 70 + corporativo::CARENCIA_DA_RECUPERACAO + 10));
    ordem.assinaturas.push(corporativo::assinar_ordem(&admins[1], &ordem));
    println!("    com 1 guardião: {:?}", custodia::reconstruir(&[&partes[0]]).err());
    let mestra = custodia::reconstruir(&[&partes[0], &partes[2]]).unwrap();
    let aberto = corporativo::abrir_pacote_de_recuperacao(&empresa, &l2, &ordem, &mestra, &pacote, 70 + corporativo::CARENCIA_DA_RECUPERACAO + 10).unwrap();
    println!("    com 2 administradores e 2 guardiões: {:?}", String::from_utf8_lossy(&aberto));

    // 28. Histórico que some para os dois (HIS1–HIS13)
    passo(27, "Eu: mensagens em 1 h; o contato usa o padrão");
    let eu = conversa::Preferencias { mensagens_apos_enviar: Some(conversa::HORA), ..conversa::Preferencias::padrao() };
    let contato = conversa::Preferencias::padrao();
    let regras = conversa::combinar(&[eu, contato]);
    println!("    mensagens somem em {} h, para os dois; não lidas, em no máximo 48 h", regras.mensagens_apos_enviar.unwrap() / conversa::HORA);
    println!("    chamada atendida — registro no meu aparelho: {:?}; no de quem ligou: {:?}",
        conversa::registrar_chamada(&regras, &eu, [1; 16], 0, 300, conversa::Direcao::Recebida, conversa::Midia::Voz),
        conversa::registrar_chamada(&regras, &contato, [1; 16], 0, 300, conversa::Direcao::Feita, conversa::Midia::Voz));
    let perdida = conversa::registrar_chamada(&regras, &eu, [1; 16], 0, 25, conversa::Direcao::Perdida, conversa::Midia::Voz).unwrap();
    println!("    chamada que não atendi: fica na minha lista até {} s e depois some", perdida.apagar_em.unwrap());
    println!("    marca de chamada dentro da conversa: {:?}", conversa::onde_mostrar(&regras, conversa::Evento::ChamadaTerminada));
    println!("    última mensagem expirou → a conversa some da lista dos dois: {}", conversa::conversa_deve_sumir(&regras, 0));
    println!("    contato trocou de aparelho → {:?} (nunca silenciado)", conversa::onde_mostrar(&regras, conversa::Evento::NovoAparelhoDoContato));

    // 29. Nada sai sem autorização de quem enviou (LIB1–LIB16)
    passo(28, "Mensagem não sai; arquivo e cartão encaminhável só com liberação assinada e com prazo");
    let autor = assinatura::gerar();
    let v = autor.verificacao();
    let foto = [7u8; 32];
    let lib_foto = liberacao::autorizar(&autor, foto, liberacao::Nivel::GuardarNoCofre, 1, Some(120), false);
    let n = liberacao::efetivo(politica::TipoDeConteudo::Foto, false, liberacao::teto_da_conversa(None),
        liberacao::nivel_autorizado(&v, std::slice::from_ref(&lib_foto), &foto, politica::TipoDeConteudo::Foto, Some(0), 60));
    println!("    foto liberada para o cofre por 2 min: guardar {} · levar para fora {}",
        liberacao::pode(politica::TipoDeConteudo::Foto, n, liberacao::Uso::GuardarNoCofre, false), liberacao::pode(politica::TipoDeConteudo::Foto, n, liberacao::Uso::LevarParaFora, false));
    println!("    3 minutos depois: {:?}", liberacao::nivel_autorizado(&v, &[lib_foto], &foto, politica::TipoDeConteudo::Foto, Some(0), 180));
    let endereco = [8u8; 32];
    let cartao = liberacao::autorizar(&autor, endereco, liberacao::Nivel::EncaminharNoNimbus, 1, Some(300), true);
    let nc = liberacao::nivel_autorizado(&v, std::slice::from_ref(&cartao), &endereco, politica::TipoDeConteudo::Localizacao, Some(0), 10);
    println!("    endereço marcado como encaminhável por 5 min: encaminhar no Nimbus {} · levar para fora {} · some ao fim {}",
        liberacao::pode(politica::TipoDeConteudo::Localizacao, nc, liberacao::Uso::EncaminharNoNimbus, false),
        liberacao::pode(politica::TipoDeConteudo::Localizacao, nc, liberacao::Uso::LevarParaFora, false), liberacao::item_deve_sumir(&cartao, Some(0), 300));
    println!("    mensagem de texto comum, sem cartão: {:?}", liberacao::nivel_autorizado(&v, &[], &[9u8; 32], politica::TipoDeConteudo::Texto, Some(0), 1));

    // 30. Como você aparece para cada contato (APR1–APR8)
    passo(29, "Eu escolho o nome com que apareço para o Dr. Almeida, travado");
    let minha = assinatura::gerar();
    let cartao = apresentacao::emitir(&minha, "G.", true, 1).unwrap();
    println!("    na agenda dele aparece: {:?} · ele pode editar: {}", apresentacao::nome_exibido(Some(&cartao), Some("Gabriel Mazon Toffoli")), apresentacao::pode_editar_nome(Some(&cartao)));

    // 31. Inspeção e envio direto (INS1–INS9, DIR1–DIR7)
    passo(30, "Arquivo conferido antes de abrir; envio direto sem conversa e sem marca");
    println!("    'foto.jpg' que na verdade é um programa: {:?}", inspecao::inspecionar("foto.jpg", b"MZ\x90\x00").decisao);
    println!("    PDF: {:?} (o código do PDF nunca roda)", inspecao::inspecionar("contrato.pdf", b"%PDF-1.7").decisao);
    let (pd, sd) = envelope::gerar_par_de_chaves();
    let mut caixa = direto::Caixa::default();
    caixa.depositar(direto::preparar(&pd, [7; 16], "nota.txt", b"combinado", 0, 600).unwrap());
    let r = caixa.retirar(&[7; 16], 5).unwrap();
    println!("    recebido: {:?} · segunda retirada: {:?}", direto::abrir(&sd, &pd, &r, 5).unwrap().nome, caixa.retirar(&[7; 16], 6).err());

    // 32. Seu padrão de envio e a escolha na hora (MOD1–MOD8)
    passo(31, "Foto sai em visualização única sem eu fazer nada; nesta aqui escolho 5 minutos");
    let padroes = modos::Padroes::recomendados();
    let conversa_normal = politica::Regras::padrao_nimbus();
    println!("    botão acima do teclado: {}", modos::rotulo(padroes.para(politica::TipoDeConteudo::Foto)));
    let r = modos::regra_da_mensagem(&conversa_normal, &padroes, politica::TipoDeConteudo::Foto, Some(modos::Modo::Temporizador(300)));
    println!("    esta foto: visualização única {} · some {} s depois de aberta", r.expiracao.visualizacao_unica, r.expiracao.segundos_apos_abrir.unwrap());

    // 33. Aparelho corrompido não entra (DV2, DV3)
    passo(32, "Atestação de hardware e desafio de uso único");
    let sinais = integridade::Sinais { app_atestado: true, aparelho_integro: true, inicio: Some(integridade::Inicio::NaoVerificado), ..Default::default() };
    let v = integridade::avaliar(integridade::Plataforma::Android, &sinais);
    println!("    bootloader destravado: {:?} · sessão permitida: {}", v.nivel, integridade::sessao_permitida(&v));
    let mut desafios = integridade::Desafios::default();
    let d = desafios.emitir(0);
    println!("    desafio usado: {:?} · reenviado: {:?}", desafios.consumir(&d, 5), desafios.consumir(&d, 6));

    // 34. Recuperar a conta: três trilhas independentes — guardiões, frase pessoal, pergunta importante —
    // e qualquer 2 das 3 devolvem a conta; nunca uma trilha só (REC1–REC6, redesenho definitivo da Rodada
    // 21: "ao menos 2 destas 3" foi decisão final do fundador, substituindo o antigo "qualquer UM caminho
    // já bastava" — ver o comentário do módulo `recuperacao` para a correção de raciocínio feita antes de
    // implementar, sobre por que "palavras mágicas" não entra como uma 4ª trilha).
    passo(33, "3 trilhas seladas (guardiões / frase pessoal / pergunta importante); 2 das 3 devolvem a conta, nunca 1 só");
    let segredo_da_conta = [5u8; 32];
    let tres_trilhas = recuperacao::preparar_tres_trilhas(&segredo_da_conta).unwrap();
    let amigos: Vec<(envelope::ChavePublica, envelope::ChaveSecreta)> = (0..3).map(|_| envelope::gerar_par_de_chaves()).collect();
    let publicas: Vec<envelope::ChavePublica> = amigos.iter().map(|(p, _)| p.clone()).collect();
    let pacotes = recuperacao::preparar(&tres_trilhas.guardioes.para_bytes(), &publicas, 2).unwrap();
    let (chave_frase, bloco_frase) = recuperacao::preparar_frase_pessoal(&tres_trilhas.frase.para_bytes()).unwrap();

    // "Perdeu a pergunta importante (esqueceu a resposta), mas tem 2 dos 3 guardiões e a frase pessoal":
    // 2 das 3 trilhas bastam — a trilha dos guardiões, por sua vez, também exige 2 dos 3 amigos.
    let a = recuperacao::abrir_para_devolver(&amigos[0].1, &amigos[0].0, &pacotes[0], true).unwrap();
    let c = recuperacao::abrir_para_devolver(&amigos[2].1, &amigos[2].0, &pacotes[2], true).unwrap();
    let trilha_guardioes = recuperacao::Trilha::de_bytes(&recuperacao::reunir(&[&a, &c]).unwrap()).unwrap();
    let trilha_frase = recuperacao::Trilha::de_bytes(&recuperacao::abrir_frase_pessoal(&chave_frase, &bloco_frase).unwrap()).unwrap();
    let recuperado = recuperacao::concluir_com_duas_trilhas(&trilha_guardioes, &trilha_frase).unwrap();
    println!("    com 1 guardião só: {:?} · com 2 guardiões + frase pessoal (2 das 3 trilhas): recuperado = {}", recuperacao::reunir(&[&a]).err(), recuperado[..] == segredo_da_conta);

    println!("\nFim. Cada passo acima tem teste automatizado correspondente em `cargo test`.");
}
