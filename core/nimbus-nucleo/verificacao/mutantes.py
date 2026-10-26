#!/usr/bin/env python3
"""Teste por mutação do nimbus-nucleo.

Planta, um de cada vez, defeitos em pontos críticos de segurança e confere se algum teste percebe.
Mutante "morto" = um teste falhou (bom). Mutante "vivo" = nenhum teste percebeu (teste fraco: corrigir).
Uso: python3 verificacao/mutantes.py   (variável CARGO opcional, padrão "cargo")

Amostra (fluxo de desenvolvimento): MUTANTES_AMOSTRA=40 escolhe 40 defeitos determinísticos pelo hash do
commit (MUTANTES_SEMENTE, padrão `git rev-parse HEAD`). A escolha muda a cada commit, então o catálogo
inteiro é coberto ao longo do tempo; a campanha dos 356 continua no sentinela diário. O catálogo NUNCA
encolhe: `--conferir` confere os 356 trechos mesmo quando só uma amostra roda.
"""
import os, random, subprocess, sys

CARGO = os.environ.get("CARGO", "cargo")
MUTANTES = [
 ("src/sessoes.rs", "if !credencial_valida { self.contar(Gatilho::SemCredencial); return Resultado::Recusada; }", "if credencial_valida { self.contar(Gatilho::SemCredencial); return Resultado::Recusada; }", "sessão: inverter a checagem de credencial"),
 ("src/sessoes.rs", "if !atestado { return self.incidente(conta, Gatilho::AtestacaoFalhou); }", "if false { return self.incidente(conta, Gatilho::AtestacaoFalhou); }", "sessão: ignorar app adulterado"),
 ("src/sessoes.rs", "if self.ativas.get(&conta).is_some_and(|a| a.contains(&aparelho)) { return self.incidente(conta, Gatilho::SessaoParalela); }", "if false { return self.incidente(conta, Gatilho::SessaoParalela); }", "sessão: aceitar sessão paralela"),
 ("src/sessoes.rs", "if !self.aparelhos.get(&conta).is_some_and(|s| s.contains(&aparelho)) {", "if false {", "sessão: aceitar aparelho não registrado"),
 ("src/envelope.rs", "    if !segredo_x.was_contributory() { return Err(Erro::ChaveInvalida); }\n    let chave = derivar_chave(suite, segredo_kem.as_slice(), segredo_x.as_bytes(), ct_kem_bytes", "    let chave = derivar_chave(suite, segredo_kem.as_slice(), segredo_x.as_bytes(), ct_kem_bytes", "envelope: aceitar chave efêmera degenerada ao abrir"),
 ("src/envelope.rs", "if !XSecret::from([7u8; 32]).diffie_hellman(&x).was_contributory() { return Err(Erro::ChaveInvalida); }", "", "envelope: aceitar chave pública degenerada na leitura"),
 ("src/envelope.rs",
  "pub fn selar_com(suite: Suite, destinatario: &ChavePublica, texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {\n    if !tamanho_permitido(texto_claro.len()) { return Err(Erro::GrandeDemais); }\n",
  "pub fn selar_com(suite: Suite, destinatario: &ChavePublica, texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {\n",
  "envelope: remover o limite de tamanho (lado assimétrico) — (correção desta rodada, 11/10/2026: o trecho original passou a bater duas vezes no arquivo depois que a Rodada 19 acrescentou `selar_com_chave_simetrica` com a MESMA linha de guarda — achado da própria campanha de força bruta ao rodar `--conferir`; a intenção do mutante nunca mudou, só a âncora, agora amarrada à assinatura de `selar_com` para não se confundir com o lado simétrico, que tem o mutante equivalente próprio, abaixo)"),
 ("src/envelope.rs", "pub fn tamanho_permitido(n: usize) -> bool { n <= TAM_MAXIMO }", "pub fn tamanho_permitido(n: usize) -> bool { n < TAM_MAXIMO }", "envelope: limite fora por um"),
 ("src/envelope.rs", "    info.push(suite as u8);\n", "", "envelope: suíte fora da derivação da chave"),
 ("src/expiracao.rs", "prazo_final(b, aberta_em, fechada_em).is_some_and(|d| agora_servidor >= d)", "prazo_final(b, aberta_em, fechada_em).is_some_and(|d| agora_servidor > d)", "expiração: um segundo a mais de vida"),
 ("src/expiracao.rs", "    v.push(p.visualizacao_unica as u8);\n", "", "expiração: visualização única fora da assinatura"),
 ("src/politica.rs", "visualizacao_unica: conversa.expiracao.visualizacao_unica || m.expiracao.visualizacao_unica,", "visualizacao_unica: conversa.expiracao.visualizacao_unica && m.expiracao.visualizacao_unica,", "política: visualização única da conversa ignorada"),
 ("src/politica.rs", "Acao::AbrirNovamente => !(unica && ja_aberta),", "Acao::AbrirNovamente => true,", "política: visualização única reabre"),
 ("src/escudo.rs", "if s.sequencia <= *sequencia {", "if s.sequencia < *sequencia {", "escudo: aceitar repetição com a mesma sequência"),
 ("src/escudo.rs", "        if s.carimbo > agora.saturating_add(TOLERANCIA_DE_RELOGIO) {", "        if false {", "escudo: aceitar carimbo no futuro"),
 ("src/escudo.rs", "if agora.saturating_sub(*recebido_em) > VALIDADE_SEGUNDOS", "if agora.saturating_sub(*recebido_em) > VALIDADE_SEGUNDOS * 100", "escudo: estado vale 100 vezes mais"),
 ("src/escudo.rs", "                    else if *captura { saida.push((p, Motivo::CapturaAtiva)); }", "", "escudo: ignorar captura de tela"),
 ("src/historico.rs", "let vencidas: Vec<u64> = self.janelas.range(..=limite)", "let vencidas: Vec<u64> = self.janelas.range(..limite)", "histórico: chave da janela vive uma janela a mais"),
 ("src/validade.rs", "    if v.apos_segundos > VALIDADE_MAXIMA { return Err(ErroDeValidade::LongaDemais); }", "    if v.apos_segundos >= VALIDADE_MAXIMA { return Err(ErroDeValidade::LongaDemais); }", "validade: limite fora por um"),
 ("src/validade.rs", "Referencia::FimDoCompromisso => fim_do_compromisso.ok_or(ErroDeValidade::SemFimDeCompromisso)?,", "Referencia::FimDoCompromisso => fim_do_compromisso.unwrap_or(criado_em),", "validade: compromisso sem fim conta da criação"),
 ("src/coacao.rs", "if bool::from(segredo_real.ct_eq(segredo_de_coacao)) { return Err(ErroDeCoacao::SegredosIguais); }", "", "coação: aceitar segredos iguais"),
 ("src/coacao.rs", "        else if bate_coacao { Abertura::CofreIscaComProtecao }", "        else if bate_coacao { Abertura::CofreReal }", "coação: segredo de coação abre o cofre real"),
 ("src/aprovacao.rs", "        if chave.verificar(CONTEXTO, &msg, &a.assinatura) { let _ = validos.insert(a.aprovador); }", "        { let _ = validos.insert(a.aprovador); }", "aprovação: aceitar sem conferir a assinatura"),
 ("src/aprovacao.rs", "if minimo == 0 || minimo > lista.len() { return Resultado::RegraInvalida; }", "if minimo == 0 || minimo > autorizados.len() { return Resultado::RegraInvalida; }", "aprovação: lista repetida forja quórum"),
 ("src/aprovacao.rs", "match p.valor_centavos { Some(x) => { v.push(1); v.extend_from_slice(&x.to_be_bytes()); } None => v.push(0) }", "v.push(0);", "aprovação: valor fora da assinatura"),
 ("src/aprovacao.rs", "if !beneficiario_ja_usado || valor_centavos > limite_centavos { espera } else { 0 }", "if !beneficiario_ja_usado && valor_centavos > limite_centavos { espera } else { 0 }", "aprovação: carência só com as duas condições"),
 ("src/rede.rs", "            if rede == EstadoDaRede::RotaRapidaBloqueada { return Err(Falha::PerguntarAoUsuario); }", "", "rede: Rápido cai para rota bloqueada sem perguntar"),
 ("src/rede.rs", "Operacao::CriacaoDeConta | Operacao::RecuperacaoDeConta | Operacao::NovoContato", "Operacao::CriacaoDeConta | Operacao::RecuperacaoDeConta", "rede: adicionar contato sem Tor no Automático"),
 ("src/rede.rs", "(_, Rota::Tor) => Ok(Decisao { rota: Rota::TorComPonte, aviso: Some(Aviso::MaisLento) }),", "(_, Rota::Tor) => Ok(Decisao { rota: Rota::DuasEtapas, aviso: None }),", "rede: descer do Tor em silêncio"),
 ("src/acesso.rs", "while v.len() < tamanho {", "while v.len() + 1 < tamanho {", "acesso: lote um pedido menor"),
 ("src/notificacao.rs", "Ok(v) if v.as_slice() == [0] => Decisao::DescartarEmSilencio,", "Ok(v) if v.as_slice() == [0] => Decisao::MostrarGenerica(TEXTO_GENERICO),", "notificação: mostrar a de cobertura"),
 ("src/higiene.rs", "0xE1 | 0xE3..=0xED | 0xEF => false, // APP1 (EXIF/XMP) e demais APPn", "0xE1 | 0xE3..=0xED | 0xEF => true, // APP1 (EXIF/XMP) e demais APPn", "higiene: manter EXIF/GPS"),
 ("src/contatos.rs", "    claro.resize(TAM_FIXO_DO_CONTATO, 0);\n    envelope::selar(dono", "    envelope::selar(dono", "contatos: tamanho revela o nome"),
 ("src/convites.rs", "        if r.usado { return Err(RecusaDeConvite::JaUsado); }", "", "convites: convite reutilizável"),
 ("src/trafego.rs", "        if agora_ms < self.proximo_envio_ms { return None; }", "", "tráfego: envio antes da batida"),
 ("src/assinatura.rs", "        ok_ed && ok_pq", "        ok_ed || ok_pq", "assinatura: aceitar se só uma das duas conferir"),
 ("src/assinatura.rs", "    v.extend_from_slice(contexto);\n    v.extend_from_slice(mensagem);", "    v.extend_from_slice(mensagem);", "assinatura: parte clássica sem contexto"),
 ("src/senha.rs", "pub const MEMORIA_KIB: u32 = 64 * 1024;", "pub const MEMORIA_KIB: u32 = 8 * 1024;", "senha: Argon2id com pouca memória"),
 ("src/cofre.rs", "    claro.resize(classe, 0);", "", "cofre: tamanho revela o tipo"),
 ("src/cofre.rs", "    claro.push(item.tipo.codigo());", "    claro.push(0);", "cofre: tipo não guardado"),
 ("src/notificacao.rs", "Plataforma::Iphone => Canal::ApnsCifrado, _ => Canal::Proprio", "_ => Canal::ApnsCifrado", "notificação: Android passa pelo Google/Apple"),
 ("src/custodia.rs", "    if ids.len() < k as usize { return Err(ErroDeCustodia::Insuficiente); }", "    if ids.len() + 1 < k as usize { return Err(ErroDeCustodia::Insuficiente); }", "custódia: um guardião a menos basta"),
 ("src/custodia.rs", "    if !bool::from(compromisso_de(&segredo).ct_eq(&compromisso)) { return Err(ErroDeCustodia::PecaAdulterada); }", "", "custódia: aceitar peça adulterada"),
 ("src/custodia.rs", "    if segredo.is_empty() || k < 2 || k > n || n > MAX_GUARDIOES {", "    if segredo.is_empty() || k < 1 || k > n || n > MAX_GUARDIOES {", "custódia: aceitar um guardião só"),
 ("src/corporativo.rs", "    if nova.numero != anterior.numero.saturating_add(1) { return Err(Violacao::SaltoOuVolta); }", "", "corporativo: aceitar lista antiga ou com salto"),
 ("src/corporativo.rs", "    if anterior.ativas.iter().any(|c| !nova.ativas.contains(c) && !nova.suspensas.contains(c)) { return Err(Violacao::ContaSumiu); }", "", "corporativo: conta some sem registro"),
 ("src/corporativo.rs", "    if reativadas > 0 { exigido = exigido.max(p.minimo_para_reativar); }", "", "corporativo: reativar com um administrador só"),
 ("src/corporativo.rs", "    if suspensas_agora > p.limite_de_suspensao_simples { exigido = exigido.max(p.minimo_para_suspensao_em_massa); }", "    if suspensas_agora >= p.limite_de_suspensao_simples { exigido = exigido.max(p.minimo_para_suspensao_em_massa); }", "corporativo: limite de suspensão fora por um"),
 ("src/corporativo.rs", "    if nova.valida_ate > nova.emitida_em.saturating_add(e.politica.validade_da_lista) { return Err(Violacao::ValidadeLongaDemais); }", "", "corporativo: lista válida por meses"),
 ("src/corporativo.rs", "    if l.suspensas.contains(conta) { return (Decisao::Travar { motivo: Motivo::Suspensa, apagar_chaves: true }, Some(l)); }", "", "corporativo: suspenso continua usando"),
 ("src/corporativo.rs", "    if agora >= l.valida_ate { return (Decisao::Travar { motivo: Motivo::ListaVencida, apagar_chaves: false }, Some(l)); }", "", "corporativo: offline sem limite"),
 ("src/corporativo.rs", "    Some(relogio_do_aparelho.max(ultimo_horario_do_servidor.saturating_add(decorrido)))", "    Some(relogio_do_aparelho)", "corporativo: confiar no relógio do celular"),
 ("src/corporativo.rs", "    if !lista_vigente.suspensas.contains(&o.conta) { return Err(Violacao::ContaNaoSuspensa); }", "", "corporativo: ler escondido quem ainda trabalha"),
 ("src/corporativo.rs", "    if o.minimo_declarado < e.politica.minimo_para_recuperar { return Err(Violacao::AssinaturasInsuficientes); }", "", "corporativo: recuperar com um administrador só"),
 ("src/corporativo.rs", "    if nova.ativas.len() > e.assentos_contratados { return Err(Violacao::SemAssentos); }", "", "corporativo: mais usuários que licenças"),
 ("src/corporativo.rs", "    if minimo_declarado < e.politica.minimo_para_mudar_administradores { return Err(Violacao::AssinaturasInsuficientes); }", "", "corporativo: trocar administradores com um só"),
 ("src/conversa.rs", "        (Some(x), Some(y)) => Some(x.min(y)),", "        (Some(x), Some(y)) => Some(x.max(y)),", "conversa: vale o prazo mais longo"),
 ("src/conversa.rs", "        r.apagar_tudo_a_pedido &= p.apagar_tudo_a_pedido;", "        r.apagar_tudo_a_pedido |= p.apagar_tudo_a_pedido;", "conversa: apagar tudo sem o consentimento dos dois"),
 ("src/conversa.rs", "        r.sem_marcas |= p.sem_marcas;", "        r.sem_marcas &= p.sem_marcas;", "conversa: marcas voltam se um só pedir"),
 ("src/conversa.rs", "        Some(0) => None,\n", "", "conversa: chamada registrada mesmo com não registrar"),
 ("src/conversa.rs", "            if regras.sem_marcas { Exibicao::Nada } else { Exibicao::NaConversa },", "            Exibicao::NaConversa,", "conversa: marca de chamada aparece"),
 ("src/conversa.rs", "        Evento::NovoAparelhoDoContato => Exibicao::FaixaDeAlerta,", "        Evento::NovoAparelhoDoContato => if regras.sem_marcas { Exibicao::Nada } else { Exibicao::FaixaDeAlerta },", "conversa: silenciar alerta de aparelho novo"),
 ("src/conversa.rs", "    regras.remover_conversa_vazia && mensagens_restantes == 0", "    regras.remover_conversa_vazia && mensagens_restantes <= 1", "conversa: some com mensagem ainda nela"),
 ("src/conversa.rs", "    for x in [regras.mensagens_apos_enviar, regras.nao_lidas_somem_em].into_iter().flatten() { prazo = prazo.min(x); }", "", "conversa: caixa do servidor ignora prazos mais curtos"),
 ("src/conversa.rs", "    conversa.expiracao.segundos_apos_enviar = menor(conversa.expiracao.segundos_apos_enviar, regras.mensagens_apos_enviar);", "    conversa.expiracao.segundos_apos_enviar = regras.mensagens_apos_enviar;", "conversa: afrouxar a regra da conversa"),
 ("src/conversa.rs", "    registros.into_iter().filter(|r| r.apagar_em.is_none_or(|t| agora < t)).collect()", "    registros.into_iter().filter(|r| r.apagar_em.is_none_or(|t| agora <= t)).collect()", "conversa: registro vive um segundo a mais"),
 ("src/conversa.rs", "        Direcao::Perdida => minhas.chamadas_nao_atendidas,", "        Direcao::Perdida => regras.chamadas_nao_atendidas,", "conversa: quem ligou apaga o aviso de chamada perdida"),
 ("src/conversa.rs", "        Direcao::Feita | Direcao::Recebida => regras.chamadas_atendidas,", "        Direcao::Feita | Direcao::Recebida => regras.chamadas_nao_atendidas,", "conversa: chamada atendida fica registrada"),
 ("src/conversa.rs", "        r.nao_lidas_somem_em = menor(r.nao_lidas_somem_em, p.nao_lidas_somem_em);", "", "conversa: prazo de não lidas ignorado"),
 ("src/conversa.rs", "    conversa.expiracao.segundos_para_ler = menor(conversa.expiracao.segundos_para_ler, regras.nao_lidas_somem_em);", "", "conversa: prazo para ler não aplicado"),
 ("src/expiracao.rs", "    if aberta_em.is_none() {", "    if false {", "expiração: não lida vive para sempre"),
 ("src/expiracao.rs", "    if aberta_em.is_none() {", "    if true {", "expiração: prazo para ler vale mesmo depois de lida"),
 ("src/expiracao.rs", "    for o in [p.segundos_apos_abrir, p.segundos_apos_enviar, p.segundos_para_ler] {", "    for o in [p.segundos_apos_abrir, p.segundos_apos_enviar] {", "expiração: prazo para ler fora da assinatura"),
 ("src/politica.rs", "            segundos_para_ler: menor(conversa.expiracao.segundos_para_ler, m.expiracao.segundos_para_ler),", "            segundos_para_ler: m.expiracao.segundos_para_ler,", "política: mensagem alonga o prazo para ler"),
 ("src/politica.rs", "        permitir_citar: conversa.permitir_citar && m.permitir_citar,", "        permitir_citar: conversa.permitir_citar || m.permitir_citar,", "política: mensagem libera citar"),
 ("src/politica.rs", "        Acao::Citar => r.permitir_citar && !unica,", "        Acao::Citar => r.permitir_citar,", "política: citar visualização única"),
 ("src/liberacao.rs", "    matches!(t, TipoDeConteudo::Texto | TipoDeConteudo::Audio | TipoDeConteudo::VideoCircular | TipoDeConteudo::Figurinha | TipoDeConteudo::Localizacao | TipoDeConteudo::LocalizacaoAoVivo)", "    matches!(t, TipoDeConteudo::Texto)", "liberação: voz e figurinha tratadas como arquivo"),
 ("src/liberacao.rs", "pub fn pode_ser_cartao(t: TipoDeConteudo) -> bool { matches!(t, TipoDeConteudo::Texto | TipoDeConteudo::Localizacao | TipoDeConteudo::Figurinha) }", "pub fn pode_ser_cartao(t: TipoDeConteudo) -> bool { e_mensagem(t) }", "liberação: voz vira cartão"),
 ("src/liberacao.rs", "    if e_mensagem(tipo) { return matches!(uso, Uso::EncaminharNoNimbus | Uso::GuardarEmMinhaPasta) && nivel >= Nivel::EncaminharNoNimbus; }", "", "liberação: mensagem guardada ou levada para fora"),
 ("src/liberacao.rs", "    if uso == Uso::EncaminharNoNimbus && recebido_por_encaminhamento { return false; }", "", "liberação: encaminhar de novo o que chegou encaminhado"),
 ("src/liberacao.rs", "    if visualizacao_unica { return Nivel::Nenhum; }", "", "liberação: visualização única pode sair"),
 ("src/liberacao.rs", "    autorizado.min(teto)\n}", "    autorizado\n}", "liberação: ignorar o teto da conversa"),
 ("src/liberacao.rs", "    let t = Nivel::EncaminharNoNimbus;\n    match limite_da_empresa { Some(l) => t.min(l), None => t }", "    let t = Nivel::LevarParaFora;\n    match limite_da_empresa { Some(l) => t.min(l), None => t }", "liberação: Conversa Blindada mandatória deixa sair do Nimbus"),
 ("src/liberacao.rs", "    a.hash_do_conteudo == *hash && a.autor == chave_do_autor.impressao_digital()", "    a.autor == chave_do_autor.impressao_digital()", "liberação: autorização de outro conteúdo vale"),
 ("src/liberacao.rs", "        && chave_do_autor.verificar(CONTEXTO, &bytes_para_assinar(&a.hash_do_conteudo, a.nivel, a.versao, a.prazo_segundos, a.some_ao_fim), &a.assinatura)", "        && true", "liberação: aceitar sem conferir a assinatura"),
 ("src/liberacao.rs", "    match prazo { Some(p) => { v.push(1); v.extend_from_slice(&p.to_be_bytes()); } None => v.push(0) }", "    v.push(0);", "liberação: prazo fora da assinatura"),
 ("src/liberacao.rs", "    let Some(ultima) = validas.iter().max_by_key(|a| a.versao) else { return Nivel::Nenhum };", "    let Some(ultima) = validas.iter().min_by_key(|a| a.versao) else { return Nivel::Nenhum };", "liberação: revogação ignorada"),
 ("src/liberacao.rs", "        let Some(no_envio) = validas.iter().find(|a| a.versao == 1) else { return Nivel::Nenhum };", "        let Some(no_envio) = validas.iter().max_by_key(|a| a.versao) else { return Nivel::Nenhum };", "liberação: mensagem liberada depois do envio"),
 ("src/liberacao.rs", "        let Some(p) = no_envio.prazo_segundos.filter(|p| *p <= PRAZO_MAXIMO_DO_CARTAO) else { return Nivel::Nenhum };", "        let p = no_envio.prazo_segundos.unwrap_or(u64::MAX);", "liberação: cartão sem prazo ou longo demais"),
 ("src/liberacao.rs", "        (ultima.nivel.min(no_envio.nivel).min(Nivel::EncaminharNoNimbus), Some(p))", "        (ultima.nivel.min(Nivel::EncaminharNoNimbus), Some(p))", "liberação: mensagem aumentada depois do envio"),
 ("src/liberacao.rs", "            Some(o) if agora < o.saturating_add(p) => {}", "            Some(o) if agora <= o.saturating_add(p) => {}", "liberação: prazo vale um segundo a mais"),
 ("src/liberacao.rs", "            _ => return Nivel::Nenhum,\n        }\n    }\n    nivel", "            _ => {}\n        }\n    }\n    nivel", "liberação: prazo vencido ou sem abertura ignorado"),
 ("src/liberacao.rs", "        (true, Some(p), Some(o)) => agora >= o.saturating_add(p),", "        (_, Some(p), Some(o)) => agora >= o.saturating_add(p),", "liberação: item some sem o autor pedir"),
 ("src/liberacao.rs", "pub fn ao_encaminhar(recebido: Nivel, escolhido: Nivel) -> Nivel { recebido.min(escolhido) }", "pub fn ao_encaminhar(recebido: Nivel, escolhido: Nivel) -> Nivel { recebido.max(escolhido) }", "liberação: quem encaminha aumenta o nível"),
 ("src/liberacao.rs", "    if !pode(tipo, nivel, Uso::LevarParaFora, false) { return Err(Recusa::NaoAutorizado); }", "", "liberação: levar para fora sem autorização"),
 ("src/expiracao.rs", "    menor(b.enviada_em.saturating_add(VIDA_MAXIMA));\n", "", "expiração: mensagem lida vive mais de 48 horas"),
 ("src/apresentacao.rs", "        && chave_do_contato.verificar(CONTEXTO, &bytes_para_assinar(&c.nome, c.nome_travado, c.versao), &c.assinatura)", "        && true", "apresentação: aceitar cartão sem conferir assinatura"),
 ("src/apresentacao.rs", "        (Some(c), _) if c.nome_travado => Some(c.nome.as_str()),", "", "apresentação: nome travado ignorado"),
 ("src/apresentacao.rs", "pub fn pode_editar_nome(cartao: Option<&Cartao>) -> bool { !cartao.is_some_and(|c| c.nome_travado) }", "pub fn pode_editar_nome(cartao: Option<&Cartao>) -> bool { let _ = cartao; true }", "apresentação: editar nome travado"),
 ("src/apresentacao.rs", "        Some(a) if a.versao >= novo.versao => Some(a),", "        Some(a) if a.versao > novo.versao + 1 => Some(a),", "apresentação: versão antiga volta"),
 ("src/apresentacao.rs", "    if nome.trim().is_empty() || nome.len() > TAM_MAXIMO_DO_NOME || nome.chars().any(char::is_control) { return Err(ErroDeApresentacao::NomeInvalido); }", "    if nome.trim().is_empty() { return Err(ErroDeApresentacao::NomeInvalido); }", "apresentação: nome vazio ou longo aceito"),
 ("src/convites.rs", "    let (x, y) = if chave_a <= chave_b { (chave_a, chave_b) } else { (chave_b, chave_a) };", "    let (x, y) = (chave_a, chave_b);", "convites: palavras diferentes para os dois lados"),
 ("src/inspecao.rs", "    if c(b\"MZ\") || c(&[0x7F, b'E', b'L', b'F'])", "    if c(b\"ZZ\") || c(&[0x7F, b'E', b'L', b'F'])", "inspeção: programa do Windows passa"),
 ("src/inspecao.rs", "        if contem(d, b\"AndroidManifest.xml\") || contem(d, b\"classes.dex\") { return TipoReal::InstaladorAndroid; }", "", "inspeção: instalador Android passa"),
 ("src/inspecao.rs", "        TipoReal::Office if contem(dados, b\"vbaProject.bin\") => Decisao::Bloquear(Motivo::Macro),", "", "inspeção: macro passa"),
 ("src/inspecao.rs", "        _ if familia_da_extensao(nome).is_some_and(|f| !f.contains(&t)) => Decisao::Bloquear(Motivo::ExtensaoFalsa),", "", "inspeção: extensão mentirosa passa"),
 ("src/inspecao.rs", "        TipoReal::Pdf | TipoReal::Office => Decisao::AbrirComoImagem,", "        TipoReal::Pdf | TipoReal::Office => Decisao::Abrir,", "inspeção: PDF aberto com código"),
 ("src/inspecao.rs", "    if c(b\"#!\") { return TipoReal::Script; }", "", "inspeção: script passa"),
 ("src/direto.rs", "    if let inspecao::Decisao::Bloquear(m) = inspecao::inspecionar(nome, dados).decisao { return Err(ErroDireto::Recusado(m)); }", "", "direto: enviar programa"),
 ("src/direto.rs", "    let limpo = higiene::limpar(dados).unwrap_or_else(|_| dados.to_vec());", "    let limpo = dados.to_vec();", "direto: sair com metadados"),
 ("src/direto.rs", "        let r = self.remessas.remove(id).ok_or(ErroDireto::NaoEncontrada)?;", "        let r = self.remessas.get(id).cloned().ok_or(ErroDireto::NaoEncontrada)?;", "direto: entregar duas vezes"),
 ("src/direto.rs", "    let valida_ate = agora.saturating_add(prazo.min(PRAZO_MAXIMO));\n    let bloco = envelope::selar(destino, &claro, &aad_da_remessa", "    let valida_ate = agora.saturating_add(prazo);\n    let bloco = envelope::selar(destino, &claro, &aad_da_remessa", "direto: esperar mais de 48 horas"),
 ("src/direto.rs", "    aad.extend_from_slice(id);\n    aad.extend_from_slice(&valida_ate", "    aad.extend_from_slice(&valida_ate", "direto: id fora da cifra"),
 ("src/integridade.rs", "        else { vermelho.push(\"serviço de acessibilidade não confiável (risco de controle remoto)\"); }", "        else { amarelo.push(\"serviço de acessibilidade não confiável (risco de controle remoto)\"); }", "integridade: acessibilidade não contida só amarela"),
 ("src/integridade.rs", "        if s.acessibilidade_contida_pelo_sistema { amarelo.push(", "        if s.acessibilidade_contida_pelo_sistema { vermelho.push(", "integridade: acessibilidade contida tira o usuário do app"),
 ("src/integridade.rs", "    if s.inicio == Some(Inicio::NaoVerificado) { vermelho.push(\"sistema modificado ou bootloader destravado\"); }", "", "integridade: bootloader destravado passa"),
 ("src/integridade.rs", "    if s.origem_da_atestacao == Some(OrigemDaAtestacao::Software) { vermelho.push(\"atestação só em software (pode ser forjada)\"); }", "", "integridade: atestação em software passa"),
 ("src/integridade.rs", "    if s.atestacao_revogada { vermelho.push(\"certificado de atestação revogado\"); }", "", "integridade: atestação revogada passa"),
 ("src/integridade.rs", "    if s.sem_bloqueio_de_tela { vermelho.push(\"aparelho sem senha de bloqueio\"); }", "", "integridade: aparelho sem senha passa"),
 ("src/integridade.rs", "    if s.ganchos_detectados { vermelho.push(\"ferramenta de interceptação no app\"); }", "", "integridade: Frida passa"),
 ("src/integridade.rs", "    if s.emulador { vermelho.push(\"emulador\"); }", "", "integridade: emulador passa"),
 ("src/integridade.rs", "    if s.meses_sem_atualizacao.is_some_and(|m| m > MESES_TOLERADOS_SEM_ATUALIZACAO)", "    if s.meses_sem_atualizacao.is_some_and(|m| m >= MESES_TOLERADOS_SEM_ATUALIZACAO)", "integridade: 12 meses já amarela"),
 ("src/integridade.rs", "pub fn sessao_permitida(v: &Veredito) -> bool { v.nivel != Nivel::Vermelho }", "pub fn sessao_permitida(v: &Veredito) -> bool { let _ = v; true }", "integridade: aparelho vermelho recebe sessão"),
 ("src/integridade.rs", "pub fn precisa_reavaliar(avaliado_em: u64, agora: u64) -> bool { agora >= avaliado_em.saturating_add(VALIDADE_DO_VEREDITO) }", "pub fn precisa_reavaliar(avaliado_em: u64, agora: u64) -> bool { agora > avaliado_em.saturating_add(VALIDADE_DO_VEREDITO) }", "integridade: veredito vale um segundo a mais"),
 ("src/integridade.rs", "        let emitido = self.pendentes.remove(valor).ok_or(RecusaDeDesafio::Desconhecido)?;", "        let emitido = self.pendentes.get(valor).copied().ok_or(RecusaDeDesafio::Desconhecido)?;", "integridade: desafio reutilizável"),
 ("src/integridade.rs", "        if agora >= emitido.saturating_add(VALIDADE_DO_DESAFIO) { return Err(RecusaDeDesafio::Vencido); }", "", "integridade: desafio sem prazo"),
 ("src/modos.rs", "        Padroes { texto: Modo::Normal, foto_e_video: Modo::VisualizacaoUnica, voz: Modo::Discreto, arquivo: Modo::Normal }", "        Padroes { texto: Modo::Normal, foto_e_video: Modo::Normal, voz: Modo::Discreto, arquivo: Modo::Normal }", "modos: foto sem visualização única por padrão"),
 ("src/modos.rs", "    politica::efetiva(conversa, Some(&regra_do_modo(modo)))", "    let _ = conversa; regra_do_modo(modo)", "modos: mensagem ignora a regra da conversa"),
 ("src/modos.rs", "    let modo = escolha_na_hora.unwrap_or_else(|| padroes.para(tipo));", "    let _ = escolha_na_hora; let modo = padroes.para(tipo);", "modos: escolha na hora ignorada"),
 ("src/modos.rs", "        Modo::Temporizador(s) => r.expiracao.segundos_apos_abrir = Some(s.clamp(1, VIDA_MAXIMA)),", "        Modo::Temporizador(s) => r.expiracao.segundos_apos_abrir = Some(s),", "modos: temporizador acima de 48 horas"),
 ("src/modos.rs", " r.expiracao.segundos_para_ler = Some(24 * 3_600); }", " }", "modos: discreto sem prazo para ler"),
 ("src/recuperacao.rs", "    if !(3..=5).contains(&guardioes.len()) || k < 2 || k as usize > guardioes.len() { return Err(ErroDeRecuperacao::GuardioesInvalidos); }", "    if k < 2 || k as usize > guardioes.len() { return Err(ErroDeRecuperacao::GuardioesInvalidos); }", "recuperação: 2 ou 6 guardiões aceitos"),
 # 13/10/2026: a entrada original aqui usava só "ErroDeCustodia::Insuficiente => ErroDeRecuperacao::Insuficiente,"
 # como âncora — texto IDÊNTICO nos corpos de `reunir()` (reconstrução geral, Rodada 20) e de
 # `concluir_com_duas_trilhas()` (duas das três trilhas, Rodada 21), dois pontos de checagem distintos e
 # igualmente críticos. `--conferir` reportava "encontrado 2 vez(es)" (âncora ambígua): na prática isto
 # deixava UM dos dois caminhos sem nenhuma mutação testada de verdade, mascarado pelo outro caminho válido
 # (o código de produção dos dois foi lido e conferido correto — só faltava a precisão da lista aqui).
 # As duas linhas abaixo ancoram cada uma ao argumento da própria chamada (`&refs` vs `&[a, b]`, único em
 # cada função), preservando a mesma mutação original (apagar só a linha "Insuficiente => Insuficiente").
 ("src/recuperacao.rs", "custodia::reconstruir(&refs).map_err(|e| match e {\n        ErroDeCustodia::Insuficiente => ErroDeRecuperacao::Insuficiente,\n", "custodia::reconstruir(&refs).map_err(|e| match e {\n", "recuperação (reunir — reconstrução geral por guardiões): falta de guardiões vira adulteração"),
 ("src/recuperacao.rs", "custodia::reconstruir(&[a, b]).map_err(|e| match e {\n        ErroDeCustodia::Insuficiente => ErroDeRecuperacao::Insuficiente,\n", "custodia::reconstruir(&[a, b]).map_err(|e| match e {\n", "recuperação (concluir_com_duas_trilhas — Rodada 21, duas das três trilhas): falta de guardiões vira adulteração"),
 ("src/custodia.rs", "        if i != b.len() { return Err(ErroDeCustodia::PecaAdulterada); }", "", "custódia: bytes a mais aceitos"),
 ("src/custodia.rs", "            if b.len() < i + tam { return Err(ErroDeCustodia::PecaAdulterada); }", "", "custódia: parte cortada aceita"),
    # --- coacao: política de tentativas configurável (v32) ---
    ("src/coacao.rs", "erros_para_apagar: Some(10), teto_de_espera_segundos: None }", "erros_para_apagar: Some(100), teto_de_espera_segundos: None }", "coacao: equilibrada quase nunca apaga"),
    ("src/coacao.rs", "erros_para_apagar: Some(5), teto_de_espera_segundos: None }", "erros_para_apagar: None, teto_de_espera_segundos: None }", "coacao: rígida nunca apaga"),
    ("src/coacao.rs", "if self.erros <= pol.erros_sem_espera { return AposErro::TentarDeNovo; }", "if self.erros < pol.erros_sem_espera { return AposErro::TentarDeNovo; }", "coacao: um erro livre a mais"),
    ("src/coacao.rs", "let erros_sem_espera = self.erros_sem_espera.clamp(1, 20);", "let erros_sem_espera = self.erros_sem_espera;", "coacao: aceita 0 erros livres"),
    ("src/coacao.rs", "self.erros_para_apagar.map(|n| n.clamp(erros_sem_espera + 1, 100))", "self.erros_para_apagar.map(|n| n.clamp(1, 100))", "coacao: apagar pode cair nos erros livres"),
    # --- panico ---
    ("src/panico.rs", "cfg.gatilhos.iter().filter_map(|&g| reacao_se_disparou(g, s)).max()", "cfg.gatilhos.iter().filter_map(|&g| reacao_se_disparou(g, s)).min()", "panico: escolhe a reação mais fraca (correção desta rodada, 11/10/2026: o trecho original — de antes de `filter`+`map`+`disparou` virarem o `filter_map`+`reacao_se_disparou` de hoje — não existia mais no código; achado da própria campanha ao rodar `--conferir`, sem relação com qualquer mudança desta sessão. A intenção do mutante não mudou, só a sintaxe que ele planta)"),
    ("src/panico.rs", "Gatilho::SemMeMexer { segundos, .. } => s.segundos_sem_toque >= segundos,", "Gatilho::SemMeMexer { segundos, .. } => s.segundos_sem_toque > segundos,", "panico: sem-me-mexer atrasa um segundo"),
    ("src/panico.rs", "Gatilho::MovimentoBrusco(_) => s.tranco_detectado,", "Gatilho::MovimentoBrusco(_) => false,", "panico: movimento brusco nunca dispara"),
    ("src/panico.rs", "Gatilho::Coacao(_) => s.coacao_digitada,", "Gatilho::Coacao(_) => false,", "panico: coação nunca dispara"),
    # --- album ---
    ("src/album.rs", "Regra::Padrao { colecao } => return Some(*colecao),", "Regra::Padrao { colecao } => { let _ = colecao; }", "album: regra padrão ignorada"),
    ("src/album.rs", "Regra::DoContato { contato, colecao } if origem.de_contato == Some(*contato) => return Some(*colecao),", "Regra::DoContato { contato, colecao } if origem.de_contato != Some(*contato) => return Some(*colecao),", "album: regra de contato invertida"),

    # --- v37: computador, hora confirmada e NimbusDrop lacrado ---
    ("src/integridade.rs", "    if tem_atestacao_do_fabricante(p) {", "    if true {", "integridade: computador tratado como celular atestado"),
    ("src/integridade.rs", "matches!(p, Plataforma::Ios | Plataforma::Android)", "matches!(p, Plataforma::Ios | Plataforma::Android | Plataforma::Computador)", "integridade: computador chega ao verde"),
    ("src/integridade.rs", "if s.disco_cifrado != Some(true) {", "if s.disco_cifrado == Some(false) {", "integridade: disco de estado desconhecido aceito"),
    ("src/integridade.rs", "celular_virtual: true, aplicativos_bancarios: false, biometria_em_acao_sensivel: true, receber_por_proximidade: false", "celular_virtual: true, aplicativos_bancarios: false, biometria_em_acao_sensivel: true, receber_por_proximidade: true", "integridade: amarelo recebe por proximidade"),
    ("src/expiracao.rs", "    if decorrido > LIMITE_SEM_HORA_CONFIRMADA { return None; }", "    if decorrido > LIMITE_SEM_HORA_CONFIRMADA * 100 { return None; }", "hora: vale 100 vezes mais sem confirmar"),
    ("src/expiracao.rs", "    if decorrido > LIMITE_SEM_HORA_CONFIRMADA { return None; }", "    if decorrido >= LIMITE_SEM_HORA_CONFIRMADA { return None; }", "hora: limite fora por um"),
    ("src/expiracao.rs", "    Some(relogio_do_aparelho.max(ultimo_horario_do_servidor.saturating_add(decorrido)))", "    Some(relogio_do_aparelho)", "hora: confiar no relógio do celular"),
    ("src/expiracao.rs", "hora.is_some_and(|h| !expirou(b, h, aberta_em, fechada_em))", "hora.map_or(true, |h| !expirou(b, h, aberta_em, fechada_em))", "hora: exibir sem hora confirmada"),
    ("src/direto.rs", "    if !chave_chegou { return Err(MotivoDoLacre::ChaveAindaNaoChegou); }", "", "lacre: liberar antes da chave chegar"),
    ("src/direto.rs", "    if hora >= l.valida_ate { return Err(MotivoDoLacre::Vencido); }", "    if hora > l.valida_ate { return Err(MotivoDoLacre::Vencido); }", "lacre: um segundo a mais de prazo"),
    ("src/direto.rs", "    if !integridade::permissoes(nivel).receber_por_proximidade { return Err(MotivoDoLacre::AparelhoNaoVerde); }", "", "lacre: amarelo e vermelho recebem por proximidade"),
    ("src/direto.rs", "    let hora = hora_confiavel.ok_or(MotivoDoLacre::SemHoraConfirmada)?;", "    let hora = hora_confiavel.unwrap_or(0);", "lacre: abrir sem hora confirmada"),

    # --- v37, segunda rodada: hora assinada com desafio e prazo do lacre autenticado ---
    ("src/expiracao.rs", "    if &h.desafio != desafio_enviado { return Err(RecusaDeHora::DesafioErrado); }", "", "hora: aceitar resposta de outro desafio (repetição)"),
    ("src/expiracao.rs", "    if !chave_publica_servidor.verificar(CONTEXTO_HORA, &codificar_hora(&h.desafio, h.segundos), &h.assinatura) {", "    if false {", "hora: aceitar hora sem assinatura válida"),
    ("src/expiracao.rs", "    if ultima_aceita.is_some_and(|u| h.segundos < u) { return Err(RecusaDeHora::VoltouNoTempo); }", "", "hora: deixar o relógio do app voltar"),
    ("src/expiracao.rs", "ultima_aceita.is_some_and(|u| h.segundos < u)", "ultima_aceita.is_some_and(|u| h.segundos <= u)", "hora: recusar a mesma hora (fora por um)"),

    # --- relógio para destruir mesmo offline (achado do dono em 09/10/2026: apagar a chave de uma janela
    # vencida não pode depender de contato com o servidor, senão isolar o aparelho da rede impede a
    # destruição para sempre — o oposto do que historico/prechaves/metadados existem para garantir) ---
    ("src/expiracao.rs", "ultimo_horario_do_servidor.saturating_add(decorrido_monotonico)", "ultimo_horario_do_servidor.saturating_add(decorrido_monotonico.min(LIMITE_SEM_HORA_CONFIRMADA))", "relógio de destruir mesmo offline: capado no mesmo teto de 15 min que hora_confiavel usa — reintroduziria exatamente o bug que esta rodada corrigiu (isolar o aparelho da rede impediria a destruição para sempre)"),
    ("src/expiracao.rs", "ultimo_horario_do_servidor.saturating_add(decorrido_monotonico)", "decorrido_monotonico", "relógio de destruir mesmo offline: ignora a última hora do servidor, conta só o monotônico a partir de zero"),
    ("src/direto.rs", "    a.extend_from_slice(&valida_ate.to_be_bytes());\n", "", "lacre: prazo fora do dado autenticado"),

    # --- v37, terceira rodada: espera do PIN só com tempo real e prazo da remessa autenticado ---
    ("src/coacao.rs", "        if inicializacao == self.inicializacao && monotonico >= self.ultimo_monotonico {", "        if monotonico >= self.ultimo_monotonico {", "pin: reinício do aparelho conta como tempo de espera"),
    ("src/coacao.rs", "            self.acumulado = self.acumulado.saturating_add(monotonico - self.ultimo_monotonico);", "            self.acumulado = self.acumulado.saturating_add((monotonico - self.ultimo_monotonico) * 1_000);", "pin: espera acelerada"),
    ("src/direto.rs", "    aad.extend_from_slice(&valida_ate.to_be_bytes());\n", "", "direto: prazo da remessa fora do dado autenticado"),

    # --- v37, quarta rodada: chave ML-KEM validada, remetente autenticado, consistência do registro ---
    ("src/envelope.rs", "        if kem.as_bytes().as_slice() != &b[TAM_X25519..] { return Err(Erro::ChaveInvalida); }", "", "envelope: aceitar chave ML-KEM fora do intervalo"),
    ("src/direto.rs", "    if !vk.verificar(CONTEXTO_REMETENTE, &mensagem_do_remetente(&r.id, r.valida_ate, publ, claro), &a) {", "    if false {", "direto: abrir sem conferir o remetente"),
    ("src/direto.rs", "    m.extend_from_slice(&Sha512::digest(destino.para_bytes()));", "", "direto: assinatura não cobre o destinatário"),
    ("src/direto.rs", "    if pegar(&mut i, 4)? != MAGICO_ASSINADA { return Err(ErroDireto::Ilegivel); }", "    let _ = pegar(&mut i, 4)?;", "direto: aceitar remessa sem marca de assinada"),
    ("src/registro.rs", "    if anterior.is_power_of_two() { caminho.push(*raiz_anterior); }", "", "registro: consistência sem a raiz anterior"),
    ("src/registro.rs", "    &fr == raiz_anterior && &sr == raiz_atual && sn == 0", "    &sr == raiz_atual && sn == 0", "registro: consistência sem conferir o passado"),
    ("src/registro.rs", "            sr = hash_de_no(&sr, c);", "            sr = hash_de_no(c, &sr);", "registro: consistência com lado trocado"),
    ("src/registro.rs", "    if anterior == atual { return prova.is_empty() && raiz_anterior == raiz_atual; }", "    if anterior == atual { return prova.is_empty(); }", "registro: tamanhos iguais com raízes diferentes"),

    # --- v37, quinta rodada: engenharia social e segurança operacional ---
    ("src/sessoes.rs", "    if origem != OrigemDoVinculo::TelaVincularDoApp { return Err(RecusaDeVinculo::OrigemNaoPermitida); }", "", "vínculo: aceitar QR recebido por mensagem"),
    ("src/sessoes.rs", "    if !palavras_conferidas { return Err(RecusaDeVinculo::PalavrasNaoConferidas); }", "", "vínculo: sem conferir as palavras"),
    ("src/sessoes.rs", "    if nivel_do_celular != crate::integridade::Nivel::Verde { return Err(RecusaDeVinculo::CelularNaoVerde); }", "", "vínculo: celular amarelo vincula"),
    ("src/sessoes.rs", "is_some_and(|a| a.contains(&aparelho))", "is_some_and(|a| !a.is_empty())", "sessão: computador vinculado vira clone"),
    ("src/convites.rs", "    palavras[posicao % 6] == escolhida", "    !escolhida.is_empty()", "convite: qualquer palavra confirma"),
    ("src/apresentacao.rs", "    if TERMOS_RESERVADOS.iter().any(|t| f.contains(&forma_de_comparacao(t))) { return Err(ErroDeApresentacao::NomeReservado); }", "", "nome: aceitar Suporte Nimbus"),
    ("src/apresentacao.rs", "    if !crate::inspecao::texto_seguro_para_exibir(nome) { return Err(ErroDeApresentacao::CaractereEnganoso); }", "", "nome: aceitar caractere enganoso"),
    ("src/apresentacao.rs", "    c.autor == chave_do_contato.impressao_digital() && nome_permitido(&c.nome).is_ok()", "    c.autor == chave_do_contato.impressao_digital() && !c.nome.is_empty()", "nome: cartão recebido sem conferir o nome"),
    ("src/apresentacao.rs", "g == f || (f.chars().count() >= 4 && distancia_de_uma_letra(&f, &g))", "g == f", "nome: não pegar parecido por uma letra"),
    ("src/apresentacao.rs", "    base.replace(\"rn\", \"m\").replace(\"vv\", \"w\")", "    base", "nome: rn não parece m"),
    ("src/apresentacao.rs", "Some(a) if a.nome != novo.nome && novo.versao > a.versao => Some((a.nome.clone(), novo.nome.clone()))", "Some(a) if false => Some((a.nome.clone(), novo.nome.clone()))", "nome: mudança silenciosa"),
    ("src/recuperacao.rs", "    if !palavras_conferidas_ao_vivo { return Err(ErroDeRecuperacao::SemConfirmacaoAoVivo); }", "", "recuperação: amigo devolve sem conferir ao vivo"),
    ("src/recuperacao.rs", "    if agora < pedido_em.saturating_add(CARENCIA_SEGUNDOS) { return Err(ErroDeRecuperacao::EmEspera); }", "", "recuperação: sem espera"),
    ("src/recuperacao.rs", "    if cancelado_por_aparelho_antigo { return Err(ErroDeRecuperacao::Cancelado); }", "", "recuperação: aparelho antigo não cancela"),
    ("src/corporativo.rs", "    if agora < o.emitida_em.saturating_add(CARENCIA_DA_RECUPERACAO) { return Err(Violacao::EmCarencia); }", "", "empresa: recuperar dados sem espera"),

    # --- v37, sexta rodada: sigilo futuro fora da libsignal e assinatura hedged ---
    ("src/prechaves.rs", "    if !identidade.verificar(CONTEXTO, &bytes_para_assinar(&p.id, &p.publica, p.valida_ate), &p.assinatura) {", "    if false {", "prechave: aceitar chave plantada pelo servidor"),
    ("src/prechaves.rs", "    if agora >= p.valida_ate || p.valida_ate > agora.saturating_add(VALIDADE_MAXIMA) { return Err(ErroDePrechave::ForaDoPrazo); }", "    if agora > p.valida_ate { return Err(ErroDePrechave::ForaDoPrazo); }", "prechave: prazo frouxo"),
    ("src/prechaves.rs", "        self.chaves.retain(|k| agora < k.valida_ate.saturating_add(SOBREVIDA_DA_SECRETA));", "        self.chaves.retain(|k| agora < k.valida_ate.saturating_add(SOBREVIDA_DA_SECRETA * 10));", "prechave: secreta vive demais"),
    ("src/prechaves.rs", "    let valida_ate = agora.saturating_add(validade.min(VALIDADE_MAXIMA));", "    let valida_ate = agora.saturating_add(validade);", "prechave: validade sem teto"),
    ("src/prechaves.rs", "    v.extend_from_slice(aad_do_uso);", "", "prechave: uso fora do dado autenticado"),
    ("src/assinatura.rs", "        OsRng.fill_bytes(&mut rnd);\n        let prefixo", "        let prefixo", "assinatura: ML-DSA determinística (sem hedge)"),
    ("src/assinatura.rs", "        let prefixo = [0u8, contexto.len() as u8];", "        let prefixo = [1u8, contexto.len() as u8];", "assinatura: domínio errado na mensagem do ML-DSA"),

    # --- v37, sétima rodada: back-end (rotas, limites) ---
    ("src/autenticacao.rs", "    ROTAS.iter().find(|r| r.metodo == metodo && r.caminho == caminho)", "    ROTAS.iter().find(|r| r.metodo == metodo && caminho.starts_with(r.caminho))", "rota: aceitar prefixo"),
    ("src/autenticacao.rs", "        if corpo.len() > r.corpo_maximo { return Err(Recusa::CorpoGrandeDemais); }", "", "rota: sem limite de corpo"),
    ("src/autenticacao.rs", "        if *tipo != tipo_exigido { return Err(Recusa::TipoErrado); }", "", "rota: chave de caixa abre rota de aparelho"),
    ("src/autenticacao.rs", "        let janela_ok = !(p.carimbo.saturating_add(JANELA_SEGUNDOS) < agora || p.carimbo > agora.saturating_add(JANELA_SEGUNDOS));", "        let janela_ok = !(p.carimbo.saturating_add(JANELA_SEGUNDOS * 100) < agora);", "rota: janela frouxa"),
    ("src/autenticacao.rs", "        if self.vistos.contains_key(&(p.titular, p.numero)) { return Err(Recusa::Repetido); }", "", "rota: aceitar pedido repetido"),
    ("src/autenticacao.rs", "    v.extend_from_slice(&Sha512::digest(corpo));", "", "rota: assinatura não cobre o corpo"),
    ("src/autenticacao.rs", "            Exigencia::Ficha => return if ficha_valida { Ok(Autorizado::Ficha) } else { Err(Recusa::FichaInvalida) },", "            Exigencia::Ficha => return Ok(Autorizado::Ficha),", "rota: depositar sem ficha"),
    ("src/limites.rs", "        if b.usados >= maximo { return Err(Limite::EsperarAte(b.inicio_da_janela.saturating_add(janela))); }", "", "limite: sem cota"),
    ("src/limites.rs", "        let espera = inicial.saturating_mul(1u64 << passos).min(CASTIGO_MAXIMO);", "        let espera = inicial.min(CASTIGO_MAXIMO);", "limite: castigo não dobra"),
    ("src/limites.rs", "            if self.baldes.len() >= self.capacidade { return Err(Limite::Cheio); }", "", "limite: memória sem teto em permitir"),
    ("src/limites.rs", "            if self.baldes.len() >= self.capacidade { return agora; }", "", "limite: memória sem teto em falhas"),
    ("src/limites.rs", "        if agora < b.bloqueado_ate { return Err(Limite::EsperarAte(b.bloqueado_ate)); }", "", "limite: castigo ignorado"),

    # --- v37, oitava rodada: limite de uso obrigatório em toda rota ---
    ("src/autenticacao.rs", "            Err(Limite::EsperarAte(t)) => return Err(Recusa::LimiteDeUso(t)),", "            Err(Limite::EsperarAte(_)) => {}", "rota: limite de uso ignorado"),
    ("src/autenticacao.rs", "Recusa::ForaDaJanela) => { let _ = l.registrar_falha(chave_de_limite, agora); }", "Recusa::ForaDaJanela) => {}", "rota: falha de prova sem castigo"),
    ("src/autenticacao.rs", "        let por_minuto = r.por_minuto.clamp(1, LIMITE_MAXIMO_POR_MINUTO);", "        let por_minuto = r.por_minuto.saturating_mul(1_000);", "rota: limite mil vezes maior"),

    # --- v37, nona rodada: sentinela (cada invariante violada tem de travar) ---
    ("src/sentinela.rs", "        if a.por_minuto == 0 || a.por_minuto > v.teto_por_minuto || v.teto_por_minuto > 600 {", "        if false {", "sentinela: rota sem limite passa"),
    ("src/sentinela.rs", "a.caminho.chars().any(|c| c.is_ascii_uppercase() || c == '%') {", "false {", "sentinela: caminho torto passa"),
    ("src/sentinela.rs", "        if rotas[i + 1..].iter().any(|b| b.metodo == a.metodo && b.caminho == a.caminho) {", "        if false {", "sentinela: rota repetida passa"),
    ("src/sentinela.rs", "    if v.janela > 60 {", "    if v.janela > 6000 {", "sentinela: janela frouxa passa"),
    ("src/sentinela.rs", "    if v.vida_maxima > 48 * 3600 || v.prazo_direto > 48 * 3600 || v.limite_para_ler > 48 * 3600 {", "    if v.vida_maxima > 48 * 3600 {", "sentinela: prazo acima de 48 h passa"),
    ("src/sentinela.rs", "    if v.limite_sem_hora > 15 * 60 {", "    if false {", "sentinela: hora sem confirmação passa"),
    ("src/sentinela.rs", "    if v.prechave_validade > 7 * 24 * 3600 || v.prechave_sobrevida > 48 * 3600 {", "    if v.prechave_validade > 7 * 24 * 3600 {", "sentinela: chave de recebimento que não morre passa"),
    ("src/sentinela.rs", "    if v.carencia_recuperacao < 72 * 3600 {", "    if false {", "sentinela: recuperação sem espera passa"),
    ("src/sentinela.rs", "    if v.carencia_corporativa < 24 * 3600 {", "    if false {", "sentinela: recuperação corporativa sem espera passa"),
    ("src/sentinela.rs", "    if v.senha_memoria_kib < 19 * 1024 || v.senha_passagens < 2 || v.senha_faixas < 1 {", "    if v.senha_memoria_kib < 1 {", "sentinela: Argon2id fraco passa"),
    ("src/sentinela.rs", "    pub fn pode_funcionar(&self) -> bool { self.achados.is_empty() }", "    pub fn pode_funcionar(&self) -> bool { self.achados.len() < 2 }", "sentinela: um achado não trava"),

    # --- v37, décima rodada: selo de liberação (sem checklist completo, nada roda) ---
    ("src/selo.rs", "    if &s.resumo_do_artefato != artefato_rodando { return Err(RecusaDoSelo::ArtefatoDiferente); }", "", "selo: aceitar artefato trocado"),
    ("src/selo.rs", "    if s.versao < versao_minima { return Err(RecusaDoSelo::VersaoAntiga); }", "", "selo: aceitar versão antiga"),
    ("src/selo.rs", "            None => return Err(RecusaDoSelo::PortaoAusente(obrig)),", "            None => {}", "selo: aceitar portão ausente"),
    ("src/selo.rs", "            Some((_, false)) => return Err(RecusaDoSelo::PortaoReprovado((*obrig).to_string())),", "            Some((_, false)) => {}", "selo: aceitar portão reprovado"),
    ("src/selo.rs", "        return Err(RecusaDoSelo::PortaoDesconhecido(n.clone()));", "        let _ = n;", "selo: aceitar portão inventado"),
    ("src/selo.rs", "        if validas.contains(fp) { continue; }", "", "selo: mesma chave conta duas vezes"),
    ("src/selo.rs", "            if k.verificar(CONTEXTO, &m, a) { validas.push(*fp); }", "            { let _ = a; validas.push(*fp); }", "selo: assinatura não conferida"),
    ("src/selo.rs", "pub const MINIMO_DE_ASSINATURAS: usize = 3;", "pub const MINIMO_DE_ASSINATURAS: usize = 2;", "selo: dois de dentro liberam sem o auditor"),
    ("src/selo.rs", "    if s.emitido_em > agora.saturating_add(300) { return Err(RecusaDoSelo::EmitidoNoFuturo); }", "", "selo: aceitar data no futuro"),
    ("src/selo.rs", "    for par in portoes.windows(2) { if par[0].0 == par[1].0 { return Err(RecusaDoSelo::Malformado); } }", "", "selo: portão repetido"),
    ("src/sentinela.rs", "    if let Err(e) = selo::conferir(s, assinaturas, artefato_rodando, chaves_fixadas, versao_minima, agora) {", "    if let Err(e) = Ok::<(), selo::RecusaDoSelo>(()) {", "sentinela: ignorar o selo"),

    # --- v37, revisão final: metadados ---
    ("src/metadados.rs", "    h.expand(&epoca.to_be_bytes(), &mut e).expect(\"tamanho válido\");", "    h.expand(b\"fixo\", &mut e).expect(\"tamanho válido\");", "metadados: caixa não muda de época"),
    ("src/metadados.rs", "    let h = Hkdf::<Sha512>::new(Some(b\"NimbusOne/v1/indice-cego\"), chave);\n    let mut i = [0u8; 32];\n    h.expand(endereco, &mut i).expect(\"tamanho válido\");\n    i", "    let _ = chave; *endereco", "metadados: índice igual ao endereço"),
    ("src/metadados.rs", "agora < e.saturating_add(1).saturating_mul(SEGUNDOS_POR_EPOCA).saturating_add(VIDA_DA_CHAVE_DE_INDICE)", "agora < e.saturating_add(1).saturating_mul(SEGUNDOS_POR_EPOCA).saturating_add(VIDA_DA_CHAVE_DE_INDICE * 100)", "metadados: chave de índice vive demais"),
    ("src/metadados.rs", "    if ip.len() > TAM_DO_REGISTRO - 9 { return Err(ErroDeRegistro::GrandeDemais); }", "    if ip.len() > TAM_DO_REGISTRO { return Err(ErroDeRegistro::GrandeDemais); }", "metadados: registro maior que o tamanho fixo"),

    # --- v37, revisão de fechamento: respostas, origem, ambiente, disco, chave do dia ---
    ("src/autenticacao.rs", "            | Recusa::ForaDaJanela | Recusa::Repetido | Recusa::AssinaturaInvalida) => (401, \"nao autorizado\"),", "            | Recusa::ForaDaJanela | Recusa::Repetido) => (401, \"nao autorizado\"),\n        Err(Recusa::AssinaturaInvalida) => (403, \"assinatura invalida\"),", "resposta: conta o motivo da recusa"),
    ("src/autenticacao.rs", "    origem.is_none()", "    true", "rota: navegador volta a falar com a API"),
    ("src/autenticacao.rs", "        (\"x-frame-options\", \"DENY\"),\n        (\"content-security-policy\", \"default-src 'none'; frame-ancestors 'none'", "        (\"content-security-policy\", \"default-src 'none'; frame-ancestors *", "cabeçalhos: página da API pode ir para moldura"),
    ("src/ambiente.rs", "    if !s.bloqueio_de_tela {\n        rebaixar(Nivel::Vermelho);", "    if !s.bloqueio_de_tela {\n        rebaixar(Nivel::Amarelo);", "ambiente: sem bloqueio de tela só amarelo"),
    ("src/ambiente.rs", "    if s.dias_desde_o_patch > DIAS_MAXIMOS_SEM_PATCH {", "    if s.dias_desde_o_patch > DIAS_MAXIMOS_SEM_PATCH * 10 {", "ambiente: patch velho passa"),
    ("src/ambiente.rs", "    if (d.teto as u8) > (integridade as u8) { d.teto } else { integridade }", "    integridade", "ambiente: teto ignorado"),
    ("src/metadados.rs", "Payload { msg: &selado[24..], aad: &indice }", "Payload { msg: &selado[24..], aad: b\"\" }", "disco: selo sem amarrar ao índice"),
    ("src/chaves_do_servidor.rs", "    if agora.saturating_add(FOLGA) < inicio || agora >= inicio.saturating_add(DIA).saturating_add(FOLGA) {", "    if agora.saturating_add(FOLGA) < inicio {", "chave do dia: a de ontem continua valendo"),
    ("src/chaves_do_servidor.rs", "pub const MINIMO_DE_RAIZES: usize = 2;", "pub const MINIMO_DE_RAIZES: usize = 1;", "chave do dia: uma raiz só certifica"),
    ("src/chaves_do_servidor.rs", "        if validas.contains(fp) { continue; }", "", "chave do dia: mesma raiz conta duas vezes"),

    # --- v37, ataque vermelho: nomes com outros alfabetos e imitações ---
    ("src/apresentacao.rs", "    if nome.chars().any(imitacao_de_letra) { return Err(ErroDeApresentacao::CaractereEnganoso); }", "", "nome: aceitar letras de imitação"),
    ("src/apresentacao.rs", "    if (tem(1) && (tem(2) || tem(3))) || (tem(2) && tem(3)) { return Err(ErroDeApresentacao::CaractereEnganoso); }", "", "nome: aceitar mistura de alfabetos"),
    ("src/apresentacao.rs", "    let base: String = nome.chars().map(latina_parecida).flat_map(char::to_lowercase)", "    let base: String = nome.chars().flat_map(char::to_lowercase)", "nome: grego e cirílico não comparados como latinos"),

    # vigia_de_sensores (VSE1–VSE8)
    ("src/vigia_de_sensores.rs", "    fn arriscado(self) -> bool { matches!(self, Sinal::Sim | Sinal::Desconhecido) }", "    fn arriscado(self) -> bool { matches!(self, Sinal::Sim) }", "sensores: não sei vira tudo bem"),
    ("src/vigia_de_sensores.rs", "    if l.carimbo.saturating_add(VALIDADE_DA_LEITURA_SEGUNDOS) < agora { return tudo_fechado(\"leitura dos sensores vencida\"); }", "", "sensores: leitura vencida aceita"),
    ("src/vigia_de_sensores.rs", "    if l.carimbo > agora.saturating_add(TOLERANCIA_DE_RELOGIO) { return tudo_fechado(\"leitura dos sensores com relógio adiantado\"); }", "", "sensores: leitura do futuro aceita"),
    ("src/vigia_de_sensores.rs", "    if !l.app_em_primeiro_plano { return tudo_fechado(\"app fora do primeiro plano\"); }", "", "sensores: segundo plano mostra e grava"),
    ("src/vigia_de_sensores.rs", "    if l.tela_capturada.arriscado() { d.mostrar_conteudo = false; d.avisar_participantes = true;", "    if l.tela_capturada.arriscado() { d.avisar_participantes = true;", "sensores: tela capturada mostra conteúdo"),
    ("src/vigia_de_sensores.rs", "    if l.tela_externa_conectada.arriscado() { d.mostrar_conteudo = false;", "    if l.tela_externa_conectada.arriscado() {", "sensores: tela externa mostra conteúdo"),
    ("src/vigia_de_sensores.rs", "    if l.sobreposicao.arriscado() { d.mostrar_conteudo = false; d.bloquear_sessao = true;", "    if l.sobreposicao.arriscado() { d.mostrar_conteudo = false;", "sensores: sobreposição sem bloquear sessão"),
    ("src/vigia_de_sensores.rs", "    if l.teclado_de_terceiros.arriscado() { d.teclado_permitido = false;", "    if l.teclado_de_terceiros.arriscado() {", "sensores: teclado de terceiros liberado"),
    ("src/vigia_de_sensores.rs", "        if l.microfone_de_outro_app.arriscado() {", "        if false {", "sensores: microfone junto com outro app"),
    ("src/vigia_de_sensores.rs", "        if l.camera_de_outro_app.arriscado() { d.motivos.push(\"outro app com a câmera\"); d.avisar_participantes = true; }", "        if false { d.motivos.push(\"outro app com a câmera\"); d.avisar_participantes = true; }", "sensores: câmera junto com outro app"),
    ("src/vigia_de_sensores.rs", "    if d.bloquear_sessao { d.mostrar_conteudo = false; d.microfone = false; d.camera = false; d.motivos.push(\"sessão bloqueada: tudo fechado\"); }", "", "sensores: sessão bloqueada com sensores abertos"),
    ("src/vigia_de_sensores.rs", "    if matches!(uso, Uso::Chamada | Uso::GravandoVoz) {", "    if true {", "sensores: microfone ligado fora de chamada"),
    # autenticidade (AUT1–AUT3)
    ("src/autenticidade.rs", "    bool::from(algum) && !fixados.is_empty()", "    true", "autenticidade: qualquer assinatura de app aceita"),
    ("src/autenticidade.rs", "        if d == *o || d.ends_with(&format!(\".{o}\")) { return None; }", "        if d.contains(*o) { return None; }", "autenticidade: oficial dentro de outro domínio vira confiável"),
    ("src/autenticidade.rs", "    if d.split('.').any(|r| r.starts_with(\"xn--\")) { return Some(Motivo::CodificacaoEstrangeira); }", "", "autenticidade: codificação estrangeira passa"),
    ("src/autenticidade.rs", "        if base_limpa == *o || distancia(&base_limpa, o, 2) <= 2 { return Some(Motivo::Parecido); }", "        if base_limpa == *o { return Some(Motivo::Parecido); }", "autenticidade: letra a mais passa (caso Privnote)"),
    ("src/autenticidade.rs", "            'а' => 'a', 'е' => 'e', 'о' => 'o', 'р' => 'p', 'с' => 'c',", "            'е' => 'e', 'о' => 'o', 'р' => 'p', 'с' => 'c',", "autenticidade: a cirílico passa"),
    ("src/autenticidade.rs", "        if usuario { suspeitos.push((d, Motivo::UsuarioFalso)); continue; }", "", "autenticidade: usuário falso antes de @ passa"),
    ("src/autenticidade.rs", "        if achados.len() >= MAXIMO_DE_ENDERECOS { break; }", "", "autenticidade: extração sem limite"),

    # protecao_do_usuario (BLQ, DEN, APG)
    ("src/protecao_do_usuario.rs", "        if self.esta_bloqueado(remetente) { Destino::DescartarEmSilencio } else { Destino::Entregar }", "        Destino::Entregar", "bloqueio: bloqueado entregue"),
    ("src/protecao_do_usuario.rs", "        if !self.bloqueados.contains(&d) && self.bloqueados.len() >= MAXIMO_DE_BLOQUEADOS { return Err(ErroDeBloqueio::ListaCheia); }", "", "bloqueio: lista sem limite"),
    ("src/protecao_do_usuario.rs", "        v.resize(4 + MAXIMO_DE_BLOQUEADOS * 32, 0);", "", "bloqueio: tamanho revela quantos"),
    ("src/protecao_do_usuario.rs", "            if !l.bloqueados.insert(d) { return None; } // repetido = adulterado", "            l.bloqueados.insert(d);", "bloqueio: repetido aceito"),
    ("src/protecao_do_usuario.rs", "        if b[4 + n * 32..].iter().any(|&x| x != 0) { return None; } // sobra que não é zero = adulterado", "", "bloqueio: lixo na sobra aceito"),
    ("src/protecao_do_usuario.rs", "        if n > MAXIMO_DE_BLOQUEADOS { return None; }", "", "bloqueio: contagem impossível aceita"),
    ("src/protecao_do_usuario.rs", "    if !consentimento_explicito { return Err(ErroDeDenuncia::SemConsentimento); }", "", "denúncia: sem consentimento"),
    ("src/protecao_do_usuario.rs", "    claro.resize(TAMANHO_FIXO_DA_DENUNCIA, 0);", "", "denúncia: tamanho revela o conteúdo"),
    ("src/protecao_do_usuario.rs", "    claro.push(motivo as u8);", "    claro.push(motivo as u8); claro.extend_from_slice(b\"conteudo\");", "denúncia: conteúdo vaza no pacote"),
    ("src/protecao_do_usuario.rs", "    if em_coacao { return Err(RecusaDeApagar::FalhaDeRede); }", "", "apagar: coação apaga a conta"),
    ("src/protecao_do_usuario.rs", "    if !pin_real_confirmado { return Err(RecusaDeApagar::PinNaoConfirmado); }", "", "apagar: sem PIN real"),
    ("src/protecao_do_usuario.rs", "    if p.emitido_em > agora.saturating_add(30) { return Err(RecusaDeApagar::DoFuturo); }", "", "apagar: pedido do futuro aceito"),
    ("src/protecao_do_usuario.rs", "    if agora > p.emitido_em.saturating_add(VALIDADE_DO_PEDIDO_DE_APAGAR) { return Err(RecusaDeApagar::Vencido); }", "", "apagar: pedido vencido aceito"),
    ("src/protecao_do_usuario.rs", "    if !nonces_ja_usados.insert(p.nonce) { return Err(RecusaDeApagar::Repetido); }", "", "apagar: pedido repetido aceito"),

    # conversa: retirar mensagem e avisos com reciprocidade
    ("src/conversa.rs", "    autor_da_original == quem_pede && (!ja_lida || regras.retirar_lidas_a_pedido)", "    !ja_lida || regras.retirar_lidas_a_pedido", "conversa: qualquer um retira mensagem de outro"),
    ("src/conversa.rs", "    autor_da_original == quem_pede && (!ja_lida || regras.retirar_lidas_a_pedido)", "    autor_da_original == quem_pede", "conversa: retira lida mesmo com proteção ligada"),
    ("src/conversa.rs", "        r.retirar_lidas_a_pedido &= p.retirar_lidas_a_pedido;", "", "conversa: um só não protege o que foi lido"),
    ("src/conversa.rs", "        r.aviso_de_visualizacao &= p.aviso_de_visualizacao;", "", "conversa: visualizada sem reciprocidade"),
    ("src/conversa.rs", "                       retirar_lidas_a_pedido: true, aviso_de_visualizacao: false,", "                       retirar_lidas_a_pedido: true, aviso_de_visualizacao: true,", "conversa: visualizada ligada por padrão"),

    # --- coacao: teto diurno do PIN e piso de reinício noturno (v37, 3ª rodada) ---
    ("src/coacao.rs", "self.liberado_a_partir_de = self.liberado_a_partir_de.max(agora.saturating_add(PISO_NOTURNO_SEGUNDOS));", "self.liberado_a_partir_de = agora.saturating_add(PISO_NOTURNO_SEGUNDOS);", "coacao: reinício noturno vira teto e encurta espera real pendente"),
    ("src/coacao.rs", "if let Some(teto) = pol.teto_de_espera_segundos { espera = espera.min(teto); }", "", "coacao: teto diurno do PIN ignorado"),
    ("src/coacao.rs", "pub fn de_horas(horas: u8) -> Self { TetoDiurno(horas.clamp(TETO_DIURNO_MINIMO_HORAS, TETO_DIURNO_MAXIMO_HORAS)) }", "pub fn de_horas(horas: u8) -> Self { TetoDiurno(horas) }", "coacao: teto diurno aceita 0h ou mais de 12h"),

    # --- grupo: conversa em grupo (v37, 3ª rodada; admin múltiplo na 5ª rodada) ---
    ("src/grupo.rs", "if !self.e_admin(solicitante) { return Err(ErroDeGrupo::SemPermissao); }", "", "grupo: qualquer um (não admin) adiciona, remove, silencia ou define teto"),
    ("src/grupo.rs", "if self.membros.len() >= MAX_MEMBROS { return Err(ErroDeGrupo::GrupoCheio); }", "", "grupo: teto de membros ignorado"),
    ("src/grupo.rs", "if alvo == self.criador { return Err(ErroDeGrupo::NaoPodeMexerNoCriador); }\n            if self.admins.contains(&alvo) && *solicitante != self.criador { return Err(ErroDeGrupo::SemPermissao); }", "if alvo == self.criador { return Err(ErroDeGrupo::NaoPodeMexerNoCriador); }", "grupo: admin comum remove outro admin"),
    ("src/grupo.rs", "if alvo == self.criador { return Err(ErroDeGrupo::NaoPodeMexerNoCriador); }\n        if *solicitante != alvo && *solicitante != self.criador { return Err(ErroDeGrupo::SemPermissao); }", "if alvo == self.criador { return Err(ErroDeGrupo::NaoPodeMexerNoCriador); }", "grupo: admin comum rebaixa outro admin"),
    ("src/grupo.rs", "if self.admins.contains(&alvo) { return Err(ErroDeGrupo::SemPermissao); }\n        self.silenciados.insert(alvo);", "self.silenciados.insert(alvo);", "grupo: admin consegue silenciar outro admin"),
    ("src/grupo.rs", "self.membros.contains_key(quem) && !self.silenciados.contains(quem)", "self.membros.contains_key(quem)", "grupo: silenciado continua podendo enviar"),
    ("src/grupo.rs", "conversa::aceitar_retirada(regras, autor_da_original, quem_pede, ja_lida) || self.e_admin(quem_pede)", "self.e_admin(quem_pede)", "grupo: admin apagando mensagem ignora a regra normal de retirada"),

    # --- chamada_em_grupo: malha de chamadas 1:1 (v37, 3ª rodada) ---
    ("src/chamada_em_grupo.rs", "if self.pernas.len() >= MAX_PARTICIPANTES { return Err(ErroDeChamadaEmGrupo::GrupoCheio); }", "", "chamada em grupo: teto de participantes ignorado"),

    # --- liberacao: teto da conversa mandatório (v37, 3ª rodada) ---
    ("src/liberacao.rs", "    match limite_da_empresa { Some(l) => t.min(l), None => t }", "    match limite_da_empresa { Some(l) => t.max(l), None => t }", "liberação: limite da empresa pode afrouxar o teto"),

    # --- sessoes: teto de duração da sessão de computador (v37, 5ª rodada) ---
    ("src/sessoes.rs", "TetoDeSessaoDoComputador(horas.clamp(TETO_SESSAO_COMPUTADOR_MINIMO_HORAS, TETO_SESSAO_COMPUTADOR_MAXIMO_HORAS))", "TetoDeSessaoDoComputador(horas)", "sessões: teto de sessão do computador aceita 0h ou mais de 12h"),
    ("src/sessoes.rs", ".is_some_and(|(conectado_em, teto_segundos)| agora >= conectado_em.saturating_add(*teto_segundos))", ".is_some_and(|(conectado_em, teto_segundos)| agora >= conectado_em.saturating_add(teto_segundos.saturating_mul(2)))", "sessões: sessão de computador dura o dobro do teto escolhido"),

    # --- sessões: revogar TODOS os aparelhos ao concluir recuperação por guardiões (achado do dono em
    # 09/10/2026 — "ao menos a sessão no outro aparelho perdido é encerrada, não?" — resposta: não era,
    # antes desta rodada; a conclusão da recuperação nunca tocava sessoes.rs) ---
    ("src/sessoes.rs", "let _ = self.aparelhos.remove(&conta);", "", "revogar todos os aparelhos ao concluir recuperação: esquece de tirar o aparelho antigo do REGISTRO — ele continuaria podendo reconectar normalmente depois da recuperação concluída (o próprio achado desta rodada)"),
    ("src/sessoes.rs", "let _ = self.aparelhos.remove(&conta);\n        let _ = self.ativas.remove(&conta);", "let _ = self.aparelhos.remove(&conta);", "revogar todos os aparelhos ao concluir recuperação: esquece de limpar as sessões ATIVAS — o aparelho antigo continuaria aparecendo como conectado agora até a próxima desconexão espontânea"),
    ("src/sessoes.rs", "self.sessoes_de_computador.retain(|(c, _), _| *c != conta);", "", "revogar todos os aparelhos ao concluir recuperação: esquece de limpar o teto de sessão de computador da conta — uma sessão de computador do aparelho perdido continuaria com o teto de horas sendo contado"),

    # --- rede: Conversa Blindada não força mais Tor (v37, 5ª rodada) ---
    ("src/rede.rs", "let precisa_tor = op.e_momento_de_identidade() || rede == EstadoDaRede::RotaRapidaBloqueada;", "let precisa_tor = rede == EstadoDaRede::RotaRapidaBloqueada;", "rede: momento de identidade deixa de forçar Tor"),

    # --- cofre: retenção por prazo reaproveitando o historico, nunca apagamento próprio (v37, 6ª rodada) ---
    # Nota (rodada atual): o teto mudou de 365 para 182 dias; a descrição abaixo foi corrigida para não
    # ficar desatualizada, mas o trecho plantado usa os NOMES das constantes, não o número — continua
    # válido sem mudança nenhuma de código.
    ("src/cofre.rs", "PrazoDeRetencaoDoCofre(dias.clamp(PRAZO_MINIMO_DIAS, PRAZO_MAXIMO_DIAS))", "PrazoDeRetencaoDoCofre(dias)", "cofre: prazo de retenção aceita 0 dias ou mais de 182"),
    ("src/cofre.rs", "agora < expira_em && expira_em - agora <= duracao_do_aviso", "agora < expira_em && expira_em - agora < duracao_do_aviso", "cofre: período de aviso perde a borda exata do início"),
    ("src/cofre.rs", "historico::guardar(servidor, chaves, &bloco, Some(prazo.em_segundos())).map_err(ErroDeRetencao::AoGuardar)", "historico::guardar(servidor, chaves, &bloco, None).map_err(ErroDeRetencao::AoGuardar)", "cofre: guardar com prazo vira guardar para sempre"),

    # --- checkup: disco cifrado no computador vinculado (v37, 6ª rodada) ---
    ("src/checkup.rs", "if c.computador_vinculado && !c.disco_do_computador_cifrado { itens.push(Item::ComputadorSemDiscoCifrado); }", "if c.computador_vinculado || !c.disco_do_computador_cifrado { itens.push(Item::ComputadorSemDiscoCifrado); }", "checkup: disco sem cifra acusado mesmo sem computador vinculado"),
    ("src/checkup.rs",
     "            SemPin | PoliticaDeTentativasAusente | ComputadorSemDiscoCifrado => Severidade::Critico,\n            SemPinDeCoacao | PoucosGuardioesDeRecuperacao | SemNotificacaoDeAparelhoNovo => Severidade::Recomendado,",
     "            SemPin | PoliticaDeTentativasAusente => Severidade::Critico,\n            SemPinDeCoacao | PoucosGuardioesDeRecuperacao | SemNotificacaoDeAparelhoNovo | ComputadorSemDiscoCifrado => Severidade::Recomendado,",
     "checkup: disco sem cifra no computador cai pra 'recomendado', não crítico"),

    # --- custodia_mutua: aba trancada, par de chaves descartável por item, apagar para os dois (rodada atual) ---
    ("src/custodia_mutua.rs", "if agora >= referencia.valida_ate {", "if agora > referencia.valida_ate {", "custódia mútua: referência expira um instante depois do prazo, não na borda exata"),
    ("src/custodia_mutua.rs", "if agora >= *valida_ate_no_servidor {", "if false {", "custódia mútua: servidor nunca recusa bloco vencido (segunda checagem desligada)"),
    ("src/custodia_mutua.rs", "servidor.blocos.remove(&referencia.id).is_some()", "servidor.blocos.get(&referencia.id).is_some()", "custódia mútua: revogar finge que apagou mas não remove o bloco"),
    ("src/custodia_mutua.rs", "servidor.blocos.retain(|_, (_, valida_ate)| agora < *valida_ate);", "servidor.blocos.retain(|_, (_, valida_ate)| agora <= *valida_ate);", "custódia mútua: faxina deixa passar item vencido no instante exato do prazo"),
    ("src/custodia_mutua.rs", "let valida_ate = agora.saturating_add(prazo.em_segundos());", "let valida_ate = agora;", "custódia mútua: todo item já nasce vencido"),
    ("src/custodia_mutua.rs", "Self(dias.clamp(PRAZO_MINIMO_DIAS, PRAZO_MAXIMO_DIAS))", "Self(dias)", "custódia mútua: prazo aceita 0 dias ou mais de 182"),
    ("src/custodia_mutua.rs", "if b.len() < TAM_FIXO + envelope::TAM_CHAVE_SECRETA {", "if false {", "custódia mútua: aceita referência serializada curta demais sem checar"),
    ("src/custodia_mutua.rs", "servidor.blocos.insert(id, (bloco, arredondar_para_o_servidor(valida_ate)));", "servidor.blocos.insert(id, (bloco, valida_ate));", "custódia mútua: servidor volta a receber o prazo exato sem arredondar (vaza o segundo exato do dia)"),
    ("src/custodia_mutua.rs", "valida_ate.div_ceil(86_400).saturating_mul(86_400)", "(valida_ate / 86_400).saturating_mul(86_400)", "custódia mútua: arredondamento para o servidor arredonda para baixo (podia negar acesso antes do prazo real)"),

    # --- custódia mútua: transferência na troca de aparelho (regra 35, achado de 09/10/2026 — a função
    # que a regra descrevia como arquitetura ainda não tinha sido escrita; escrita e testada nesta rodada) ---
    ("src/custodia_mutua.rs", "let validas: Vec<&Referencia> = referencias.iter().filter(|r| agora < r.valida_ate).collect();", "let validas: Vec<&Referencia> = referencias.iter().filter(|_| true).collect();", "custódia mútua: transferência de aparelho para de descartar Referencia já vencida antes de levar ao aparelho novo"),
    ("src/custodia_mutua.rs", "if i + tam > claro.len() { return Err(Erro::Malformado); }", "if false { return Err(Erro::Malformado); }", "custódia mútua: abrir pacote de transferência para de conferir se o tamanho declarado de uma Referencia realmente sobra no pacote (risco de pânico por indexar fora dos limites, não só de aceitar lixo)"),

    # --- cofre: trava da aba trancada — inatividade e retrava por integridade (rodada atual) ---
    ("src/cofre.rs", "if nivel != crate::integridade::Nivel::Verde {", "if nivel == crate::integridade::Nivel::Vermelho {", "cofre: trava só no Vermelho, não no Amarelo (mais fraco que integridade::permissoes)"),
    ("src/cofre.rs", "agora >= ultimo_uso_em.saturating_add(politica.teto_de_inatividade.em_segundos() as u64)", "agora > ultimo_uso_em.saturating_add(politica.teto_de_inatividade.em_segundos() as u64)", "cofre: trava por inatividade um segundo depois do teto, não na borda exata"),
    ("src/cofre.rs", "TetoDeInatividadeDoCofre(s.clamp(TETO_DE_INATIVIDADE_MINIMO_SEGUNDOS, TETO_DE_INATIVIDADE_MAXIMO_SEGUNDOS))", "TetoDeInatividadeDoCofre(s)", "cofre: teto de inatividade aceita 0s ou mais de 900s"),
    ("src/cofre.rs", "PoliticaDeTrancaDoCofre { teto_de_inatividade: TetoDeInatividadeDoCofre::default(), reautenticar_a_cada_abertura: true }", "PoliticaDeTrancaDoCofre { teto_de_inatividade: TetoDeInatividadeDoCofre::default(), reautenticar_a_cada_abertura: false }", "cofre: padrão da trava deixa de exigir biometria a cada abertura"),

    # --- Cláusula 6b: aceleração do relógio de destruição por tentativas malsucedidas (achado de
    # 10/10/2026 — vínculo entre erros de PIN e a destruição por prazo) ---
    ("src/coacao.rs", "pub fn erros_consecutivos(&self) -> u32 { self.erros }", "pub fn erros_consecutivos(&self) -> u32 { 0 }", "coação: contador de erros consecutivos sempre zero — a Cláusula 6b nunca é acionada por nenhum padrão de ataque"),
    ("src/expiracao.rs", "let alem_do_limiar = erros_consecutivos.saturating_sub(self.limiar) as u64;", "let alem_do_limiar = erros_consecutivos as u64;", "aceleração por tentativas: conta TODOS os erros, não só os além do limiar — poucos erros de descuido já acelerariam o relógio de destruição"),
    ("src/expiracao.rs", "        alem_do_limiar.saturating_mul(self.segundos_extras_por_erro).min(self.teto_de_segundos_extras)", "        alem_do_limiar.saturating_mul(self.segundos_extras_por_erro)", "aceleração por tentativas: sem teto — um contador de erros corrompido ou um ataque automatizado longo jogaria o relógio para um valor absurdo de uma vez"),
    ("src/expiracao.rs", "        .saturating_add(aceleracao.segundos_extras(erros_consecutivos))\n", "", "aceleração por tentativas: a função ignora a aceleração por completo, devolvendo só a Cláusula 6a sem nenhum acréscimo por tentativas malsucedidas"),
    ("src/expiracao.rs", "            limiar: crate::coacao::ERROS_SEM_ESPERA,", "            limiar: 1,", "aceleração por tentativas: limiar padrão deixa de ser sincronizado com coacao::ERROS_SEM_ESPERA (poderia divergir silenciosamente se a regra 13 mudar o próprio limiar)"),

    # --- metadado/GPS impossível de burlar (HEIC nomeado + selar_midia) e mitigação do risco de perda
    # acidental da Cláusula 6b (pedido do dono em 10/10/2026, mesma rodada) ---
    ("src/higiene.rs", "Formato::Heic => Err(ErroDeHigiene::PrecisaConverter)", "Formato::Heic => Ok(dados.to_vec())", "higiene: HEIC passa a ser \"limpo\" devolvendo os bytes originais sem tocar — o buraco mais grave possível, GPS sairia intacto com etiqueta de limpo"),
    ("src/higiene.rs", "matches!(&d[8..12], b\"heic\" | b\"heix\" | b\"heim\" | b\"heis\" | b\"hevc\" | b\"hevx\" | b\"mif1\" | b\"msf1\")", "false", "higiene: HEIC deixa de ser reconhecido pelo nome — cairia em Desconhecido (ainda recusado, mas com o erro genérico errado, não o específico de converter)"),
    ("src/pacote.rs", "let limpo = higiene::limpar(bytes_originais).map_err(ErroDeSelarMidia::Higiene)?;", "let limpo = bytes_originais.to_vec();", "pacote: selar_midia deixa de higienizar e sela os bytes originais direto — exatamente o buraco que esta função existe para fechar"),
    ("src/expiracao.rs", "AceleracaoPorTentativas { limiar: pol.erros_sem_espera, ..Self::padrao() }", "AceleracaoPorTentativas { limiar: crate::coacao::ERROS_SEM_ESPERA, ..Self::padrao() }", "aceleração por tentativas: a_partir_da_politica ignora a política escolhida e volta ao limiar global fixo em silêncio — a correção do risco de perda acidental para quem escolheu tolerante() deixaria de valer"),
    ("src/expiracao.rs", "self.limiar.saturating_add(1).saturating_sub(erros_consecutivos)", "self.limiar.saturating_sub(erros_consecutivos)", "aceleração por tentativas: tentativas_restantes_antes_de_acelerar perde o +1 (erro de um a menos) — avisaria \"já está acelerando\" uma tentativa antes da hora certa"),

    ("src/custodia_do_cofre.rs", "pub const K: u8 = 2;", "pub const K: u8 = 1;", "custódia do cofre: limiar K cai para 1 — UMA peça sozinha (aparelho OU servidor) passaria a bastar para reconstruir a chave, exatamente a garantia que este módulo existe para dar"),
    ("src/custodia_do_cofre.rs", "pub const N: u8 = 2;", "pub const N: u8 = 3;", "custódia do cofre: número de participantes vira 3 — deixaria de ser a dupla fechada aparelho+servidor"),
    ("src/custodia_do_cofre.rs", "    let peca_servidor = partes.pop().ok_or(ErroDeCustodiaDoCofre::FormatoInesperado)?;\n    let peca_aparelho = partes.pop().ok_or(ErroDeCustodiaDoCofre::FormatoInesperado)?;", "    let peca_aparelho = partes.pop().ok_or(ErroDeCustodiaDoCofre::FormatoInesperado)?;\n    let peca_servidor = partes.pop().ok_or(ErroDeCustodiaDoCofre::FormatoInesperado)?;", "custódia do cofre: troca qual peça vai para o aparelho e qual vai para o servidor"),
    ("src/custodia_do_cofre.rs", "custodia::reconstruir(&[peca_aparelho, peca_servidor]).map_err(ErroDeCustodiaDoCofre::Custodia)", "custodia::reconstruir(&[peca_aparelho]).map_err(ErroDeCustodiaDoCofre::Custodia)", "custódia do cofre: reconstruir esquece de usar a peça do servidor — tentaria reconstruir só com a do aparelho"),

    # --- Rodada 20 (11/10/2026): "ataque com a maior força bruta até agora", pedido explícito do dono —
    # os módulos novos da Rodada 19 (comando_remoto, pedido_de_guarda, a frase de recuperação pessoal e o
    # envelope simétrico que ela usa) ainda não tinham NENHUM defeito plantado aqui, apesar de a maioria
    # já ter teste direto cobrindo a propriedade (achado honesto: isto é uma lacuna do PROCESSO desta
    # lista curada, não necessariamente de cobertura de teste — ver PENDENCIAS_E_DECISOES.md, Rodada 20,
    # para a distinção, feita mutante a mutante). Dois mutantes abaixo (marcados "achado real") expuseram
    # falta de teste de verdade, não só de entrada nesta lista — os dois já ganharam teste dedicado antes
    # desta lista existir (ver recuperacao.rs e pedido_de_guarda.rs), então já morrem aqui também.
    ("src/comando_remoto.rs", "v.push(reacao as u8);\n", "", "comando remoto: reação fora da assinatura — trocar Trancar por ApagarChaves depois de assinado deixaria de ser detectado"),
    # 13/10/2026: as duas entradas originais abaixo ancoravam só no início da condição ("if cmd.aparelho_alvo
    # != *aparelho_deste_dispositivo {" / "if cmd.contador <= ultimo_contador_aceito {") — texto que é também
    # um PREFIXO literal da forma de uma linha só usada em `validar_por_guardioes` (Rodada 21, quórum de
    # guardiões), então `--conferir` reportava "encontrado 2 vez(es)" para as duas. Isto mascarava que só UM
    # dos dois caminhos independentes — `validar` (emissor único) OU `validar_por_guardioes` (quórum) — tinha
    # mutação testada de verdade para cada checagem; o código de produção dos dois foi lido e conferido
    # correto. As quatro linhas abaixo (duas checagens × dois caminhos) incluem o corpo inteiro do `if`,
    # usando o nome do erro (`Recusa::*` vs `RecusaDeGuardioes::*`) e a forma (várias linhas vs uma linha só)
    # para ancorar cada uma ao seu único caminho, preservando a mesma mutação original em cada caso.
    ("src/comando_remoto.rs", "if cmd.aparelho_alvo != *aparelho_deste_dispositivo {\n        return Err(Recusa::AparelhoErrado);\n    }", "if false {\n        return Err(Recusa::AparelhoErrado);\n    }", "comando remoto (validar — emissor único): qualquer aparelho aceita um comando endereçado a outro"),
    ("src/comando_remoto.rs", "if cmd.aparelho_alvo != *aparelho_deste_dispositivo { return Err(RecusaDeGuardioes::AparelhoErrado); }", "if false { return Err(RecusaDeGuardioes::AparelhoErrado); }", "comando remoto (validar_por_guardioes — quórum, Rodada 21): qualquer aparelho aceita um comando endereçado a outro"),
    ("src/comando_remoto.rs", "if cmd.contador <= ultimo_contador_aceito {\n        return Err(Recusa::ComandoRepetido);\n    }", "if cmd.contador < ultimo_contador_aceito {\n        return Err(Recusa::ComandoRepetido);\n    }", "comando remoto (validar — emissor único): comando repetido (mesmo contador do último já aceito) passa a ser aceito de novo"),
    ("src/comando_remoto.rs", "if cmd.contador <= ultimo_contador_aceito { return Err(RecusaDeGuardioes::ComandoRepetido); }", "if cmd.contador < ultimo_contador_aceito { return Err(RecusaDeGuardioes::ComandoRepetido); }", "comando remoto (validar_por_guardioes — quórum, Rodada 21): comando repetido (mesmo contador do último já aceito) passa a ser aceito de novo"),
    ("src/pedido_de_guarda.rs", "if p.solicitante != chave_do_solicitante.impressao_digital() || !chave_do_solicitante.verificar(CONTEXTO, &bytes, &p.assinatura) {", "if !chave_do_solicitante.verificar(CONTEXTO, &bytes, &p.assinatura) {", "pedido de guarda (ACHADO REAL desta campanha): campo `solicitante` deixa de ser conferido contra a chave — forjável mesmo com a chave que realmente assinou, mesmo sem nenhuma assinatura inválida; teste dedicado escrito nesta mesma rodada antes deste mutante"),
    ("src/pedido_de_guarda.rs", "    if !aprovar {\n        return Ok(Resposta::Negado);\n    }", "", "pedido de guarda: negar o cofre pessoal (ou os outros usos genéricos) passa a aprovar sozinho"),
    ("src/pedido_de_guarda.rs", "    if !aprovar {\n        return Ok(RespostaDeAbaTrancada::Negado);\n    }", "", "pedido de guarda: negar a entrada na aba trancada passa a aprovar sozinho (regra 34 revista desta rodada)"),
    ("src/recuperacao.rs", "    OsRng.fill_bytes(chave.as_mut());\n", "", "recuperação: chave da frase pessoal deixa de ser aleatória — sempre a mesma (zero), previsível por qualquer um"),
    ("src/recuperacao.rs", "envelope::abrir_com_chave_simetrica(chave, bloco, CONTEXTO_FRASE)", "envelope::abrir_com_chave_simetrica(chave, bloco, b\"outro-contexto-qualquer\")", "recuperação: abrir a frase pessoal com um contexto diferente do usado para selar — tem que falhar, nunca abrir por coincidência"),
    ("src/envelope.rs",
     "pub fn selar_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {\n    if !tamanho_permitido(texto_claro.len()) { return Err(Erro::GrandeDemais); }\n",
     "pub fn selar_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {\n",
     "envelope simétrico: remover o limite de tamanho (lado simétrico — o mutante acima cobre só o lado assimétrico; os dois limites são checados por funções separadas, por isso precisam de um mutante cada)"),
    ("src/envelope.rs",
     "pub fn selar_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {\n    if !tamanho_permitido(texto_claro.len()) { return Err(Erro::GrandeDemais); }\n    let mut nonce = [0u8; TAM_NONCE];\n    OsRng.fill_bytes(&mut nonce);",
     "pub fn selar_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro> {\n    if !tamanho_permitido(texto_claro.len()) { return Err(Erro::GrandeDemais); }\n    let mut nonce = [0u8; TAM_NONCE];",
     "envelope simétrico: nonce deixa de ser aleatório ao selar — sempre zero, mesma chave+nonce reaproveitado entre mensagens diferentes (quebra a garantia que a frase pessoal e qualquer outro uso futuro deste envelope dependem)"),
    ("src/envelope.rs",
     "pub fn abrir_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], envelope: &[u8], aad: &[u8]) -> Result<Zeroizing<Vec<u8>>, Erro> {\n    if envelope.len() < TAM_CABECALHO_SIMETRICO + 16 { return Err(Erro::MalFormado); }\n    if envelope[0] != VERSAO { return Err(Erro::VersaoInvalida); }\n    let suite = Suite::de_byte(envelope[1])?;\n    let nonce = &envelope[2..TAM_CABECALHO_SIMETRICO];\n    let aad_completo = {\n        let mut v = envelope[..TAM_CABECALHO_SIMETRICO].to_vec();\n        v.extend_from_slice(aad);",
     "pub fn abrir_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], envelope: &[u8], aad: &[u8]) -> Result<Zeroizing<Vec<u8>>, Erro> {\n    if envelope.len() < TAM_CABECALHO_SIMETRICO + 16 { return Err(Erro::MalFormado); }\n    if envelope[0] != VERSAO { return Err(Erro::VersaoInvalida); }\n    let suite = Suite::de_byte(envelope[1])?;\n    let nonce = &envelope[2..TAM_CABECALHO_SIMETRICO];\n    let aad_completo = {\n        let mut v = envelope[..TAM_CABECALHO_SIMETRICO].to_vec();",
     "envelope simétrico: `aad` deixa de ser conferido ao abrir — um bloco selado para um contexto (ex.: a frase pessoal de UMA conta) passaria a abrir sob outro contexto (ex.: de outra conta ou outro uso), quebrando a separação de domínio — achado em 13/10/2026: o texto-âncora ainda citava a assinatura de antes da Rodada 22 (`Result<Vec<u8>, Erro>`); a função já retorna `Result<Zeroizing<Vec<u8>>, Erro>` desde então (ver comentário em `abrir()`, logo abaixo da função simétrica), e o trecho real que conta — `v.extend_from_slice(aad);` — nunca saiu do código; era o âncora do teste que tinha ficado para trás, não a proteção"),

]

def restaurar_copias():
    """Se uma rodada anterior foi interrompida no meio, devolve os arquivos originais antes de tudo."""
    restaurados = 0
    for raiz, _, arquivos in os.walk("src"):
        for nome in arquivos:
            if nome.endswith(".original"):
                caminho = os.path.join(raiz, nome)
                os.replace(caminho, caminho[: -len(".original")])
                os.utime(caminho[: -len(".original")], None)
                restaurados += 1
    if restaurados:
        print(f"Atenção: {restaurados} arquivo(s) restaurado(s) de uma rodada interrompida.")

def compila():
    r = subprocess.run([CARGO, "test", "--lib", "--locked", "-q", "--no-run"], capture_output=True, text=True)
    return r.returncode == 0

def testes_passam(limite=900):
    """True se os testes passam. Teste que trava além do limite conta como reprovado (o defeito foi notado)."""
    try:
        r = subprocess.run([CARGO, "test", "--lib", "--locked", "-q"], capture_output=True, text=True, timeout=limite)
    except subprocess.TimeoutExpired:
        return False
    return r.returncode == 0

def trechos_quebrados():
    """Defeito plantado cujo trecho não existe (ou existe mais de uma vez) no código não testa nada: o código
    mudou e o defeito ficou velho. Conferência estática, sem compilar."""
    ruins = []
    for i, (arq, de, _para, descricao) in enumerate(MUTANTES, 1):
        n = open(arq, encoding="utf-8").read().count(de)
        if n != 1: ruins.append(f"[{i:02}] {descricao} — trecho encontrado {n} vez(es) em {arq}")
    return ruins

def _semente():
    """Hash do commit em verificação. Sem git (ex.: cópia avulsa), cai num valor fixo: ainda determinístico."""
    try:
        r = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, timeout=10)
        if r.returncode == 0 and r.stdout.strip():
            return r.stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        pass
    return "local"

def main():
    # Pasta de compilação compartilhada com outra cópia do núcleo: o Cargo pode reaproveitar o binário da outra
    # cópia e dar um defeito plantado como pego (ou como vivo) sem ter compilado este. Reprova antes de começar.
    alvo = os.environ.get("CARGO_TARGET_DIR")
    if alvo and not os.path.abspath(alvo).startswith(os.path.abspath(".") + os.sep):
        print(f"ERRO: CARGO_TARGET_DIR={alvo} fica fora desta cópia do núcleo; rode sem ele."); sys.exit(2)
    restaurar_copias()
    ruins = trechos_quebrados()
    if ruins:
        print("ERRO: defeitos plantados velhos (o código mudou e o trecho não existe mais exatamente uma vez):")
        for r in ruins: print("  " + r)
        sys.exit(2)
    if len(sys.argv) > 1 and sys.argv[1] == "--conferir":
        print(f"Defeitos plantados: os {len(MUTANTES)} trechos existem exatamente uma vez no código."); return
    # Faixa opcional: python3 verificacao/mutantes.py 1 20  (roda só os mutantes 1 a 20)
    de = int(sys.argv[1]) if len(sys.argv) > 1 else 1
    ate = int(sys.argv[2]) if len(sys.argv) > 2 else len(MUTANTES)
    # Amostra determinística pelo hash do commit: o fluxo de desenvolvimento não espera a campanha inteira,
    # e a escolha muda a cada commit, então o catálogo é coberto ao longo do tempo. A campanha dos 356
    # continua obrigatória no sentinela diário.
    amostra = int(os.environ.get("MUTANTES_AMOSTRA", "0") or 0)
    escolhidos = None
    semente = ""
    if amostra:
        n = len(MUTANTES)
        if not 1 <= amostra <= n:
            print(f"ERRO: MUTANTES_AMOSTRA={amostra} fora de 1..{n}"); sys.exit(2)
        semente = os.environ.get("MUTANTES_SEMENTE") or _semente()
        escolhidos = set(random.Random(semente).sample(range(1, n + 1), amostra))
        print(f"Amostra: {amostra} de {n} defeitos plantados, semente {semente[:12]}")
    if not testes_passam():
        print("Os testes precisam passar antes da mutação."); sys.exit(2)
    vivos = []
    rodados = 0
    for i, (arq, original, mutado, descricao) in enumerate(MUTANTES, 1):
        if escolhidos is not None and i not in escolhidos:
            continue
        if i < de or i > ate:
            continue
        rodados += 1
        texto = open(arq, encoding="utf-8").read()
        if texto.count(original) != 1:
            print(f"[{i:02}] ERRO: trecho não encontrado exatamente uma vez em {arq}: {descricao}"); vivos.append(descricao); continue
        # Cópia de segurança em disco ANTES de plantar o defeito: se a rodada for interrompida,
        # a próxima execução restaura o original automaticamente.
        open(arq + ".original", "w", encoding="utf-8").write(texto)
        open(arq, "w", encoding="utf-8").write(texto.replace(original, mutado))
        try:
            # Defeito que nem compila não prova nada sobre os testes: conta como mutante inválido (falha da suíte).
            if not compila():
                pego = None
            else:
                pego = not testes_passam()
        finally:
            os.replace(arq + ".original", arq)
            # Data de agora no arquivo devolvido: sem isso, o compilador acha que nada mudou e mantém
            # o programa compilado COM o defeito plantado.
            os.utime(arq, None)
        if pego is None:
            print(f"[{i:02}] ERRO: o defeito plantado não compila (não testa nada) · {descricao}"); vivos.append(descricao); continue
        print(f"[{i:02}] {'morto' if pego else 'VIVO '} · {descricao}")
        if not pego: vivos.append(descricao)
    if escolhidos is not None:
        print(f"\nResultado: {rodados - len(vivos)} de {rodados} defeitos plantados da amostra foram pegos (semente {semente[:12]}, catálogo {len(MUTANTES)}).")
    else:
        print(f"\nResultado: {rodados - len(vivos)} de {rodados} defeitos plantados foram pegos pelos testes (mutantes {de} a {min(ate, len(MUTANTES))} de {len(MUTANTES)}).")
    if vivos:
        print("Vivos (testes a reforçar):"); [print("  -", v) for v in vivos]; sys.exit(1)

if __name__ == "__main__":
    main()
