//! # nimbus-nucleo — núcleo de segurança do NimbusOne
//!
//! Referência: Manual Técnico NimbusOne v30.0. **Protótipo para auditoria (prova C1).**
//! Todo o código, os comentários e os testes estão em português.
//!
//! | Módulo | Requisito | Para que serve |
//! |---|---|---|
//! | [`album`] | ALB1–ALB8 | Álbuns e coleções automáticas: cada foto entra sozinha na coleção certa por regra (contato de origem, marcação ou padrão); tudo cifrado no cofre, inclusive o nome da coleção, a legenda e a data |
//! | [`direto`] | DIR1–DIR8, ND2–ND5 | Arquivo de Nimbus para Nimbus fora da conversa: inspecionado, sem metadados, cifrado para quem recebe, entregue uma vez pelos retransmissores (nunca direto entre aparelhos), prazo de até 48 h; NimbusDrop lacrado: por proximidade, só abre com a chave entregue pelo servidor (ND2–ND5) |
//! | [`envelope`] | CRY1, CRY3, CRY9, ARQ4, FP1, FP2 | Cifra híbrida pós-quântica (suíte padrão e suíte FIPS), preenchimento de tamanho, zeragem de memória |
//! | [`expiracao`] | CH2, OFF2 | Mensagens temporárias Modo A e Modo B, com prazo assinado pelo servidor; hora confirmada, assinada pelo servidor com desafio de uso único (sem confirmação há mais de 15 min, nada com prazo aparece) |
//! | [`inspecao`] | INS1–INS9 | Tipo real pelos primeiros bytes; bloqueia programas, instaladores, scripts, macros e extensão mentirosa; PDF e Office só como imagem |
//! | [`integridade`] | DV2, DV3, PC1–PC5 | Verificação do aparelho ao abrir o app: verde, amarelo, vermelho; computador nunca passa do amarelo; não existe cliente web |
//! | [`panico`] | PAN1–PAN9 | Travas de emergência decididas dentro do aparelho: palavra mágica, toque de pânico, longe de casa, movimento brusco, coação e sem me mexer, cada uma com a reação escolhida; mais um gatilho de origem remota (`comando_remoto`, nova, 10/10/2026), a única exceção deliberada a "nada sai para o servidor" |
//! | [`comando_remoto`] | nova, sem ID de requisito ainda (10/10/2026; quórum de guardiões acrescentado 11/10/2026, Rodada 21) | Comando de bloquear/apagar, por dois caminhos independentes, para alimentar `panico::Gatilho::ComandoRemoto`: (a) assinado por outro aparelho já confiado na conta — exige um segundo aparelho/computador já vinculado; (b) assinado por `k` de `n` guardiões (mesmo quórum de `recuperacao::preparar_tres_trilhas`), nunca 1 sozinho — cobre o caso sem nenhum segundo aparelho à mão. Os dois vinculados ao aparelho-alvo certo e protegidos contra repetição por contador crescente — zero criptografia nova, mesma assinatura híbrida de sempre, contextos de domínio separados entre os dois caminhos e entre isto e a recuperação de conta |
//! | [`sessoes`] | ID2, CG1 | Sessão única, anti-clonagem, derrubada total, contagens anônimas |
//! | [`escudo`] | EC1–EC9 | Escudo de Conversa: borra a conversa em todos os aparelhos se alguém captura a tela |
//! | [`voz`] | VZ4, VC4, VN1, VN2 | Modulador de voz: Nível 1 deslocador de tom em tempo real; Nível 2 conversão neural para voz-alvo fixa (orquestração no aparelho, inferência nativa). Sempre antes de cifrar |
//! | [`modos`] | MOD1–MOD8 | Seu padrão de envio por tipo (texto, foto e vídeo, voz, arquivo) e a escolha na hora; a conversa continua sendo o teto |
//! | [`politica`] | CH2, CH3, AR2 | Regras por conversa e por mensagem: vence a mais restritiva; visualização única para toda mídia; bloqueios de encaminhar, copiar, salvar |
//! | [`historico`] | EF5, EF6, CH4 | Histórico guardado cifrado no servidor e aberto só no aparelho; apagamento criptográfico por janela de tempo |
//! | [`coacao`] | AL2, AR6 | PIN de coação, cofre-isca e negação plausível, em tempo constante |
//! | [`validade`] | VAL1–VAL8 | Apagar sozinho: agenda (após o fim do compromisso), notas, contatos, arquivos, fotos — com apagamento criptográfico |
//! | [`higiene`] | HIG1–HIG6 | Arquivo sai sem rastro: remove EXIF/GPS/autor de JPEG e PNG, nome neutro; exige achatar PDF e Office |
//! | [`registro`] | BL1, BL6, CO7, BC2 | Registro público só-de-acréscimo (árvore de Merkle, RFC 9162): chaves, imagens e versões; aparelho fantasma e versão adulterada são denunciados |
//! | [`assinatura`] | PQS1–PQS6 | Assinatura híbrida Ed25519 + ML-DSA-87 (FIPS 204, CNSA 2.0): as duas precisam conferir; contexto por uso |
//! | [`senha`] | SEG5, ENV3, BA3 | Argon2id (RFC 9106: 64 MiB, 3 passagens, 4 faixas) para PIN de coação e senha de link |
//! | [`cofre`] | COF1–COF8 | Álbuns, notas, compromissos, senhas e ajustes cifrados com o tipo dentro da cifra e classes de tamanho |
//! | [`conversa`] | HIS1–HIS18 | Regras da conversa para os dois lados (a mais restritiva vence): mensagens e não lidas em até 48 h; chamada atendida não fica registrada; chamada não atendida fica 48 h para quem recebeu (quem ligou não apaga esse aviso); sem marcas; conversa vazia some; apagar tudo só com consentimento; caixa do servidor em até 48 h |
//! | [`liberacao`] | LIB1–LIB16 | Mensagem não sai; exceção estreita: cartão encaminhável (texto ou localização, marcado no envio, prazo de até 4 h, só dentro do Nimbus, um salto). Arquivo só com liberação assinada pelo autor, com prazo opcional; revogação; aviso ao autor; saída única com metadados limpos |
//! | [`corporativo`] | COR1–COR24 | Plano corporativo: suspender sem ler (lista de contas assinada pela empresa, validade curta, relógio confiável), recuperar só conta suspensa com a chave mestra, k de n para reativar, suspender em massa e trocar administradores |
//! | [`custodia`] | GUA1–GUA8 | Chave mestra dividida entre guardiões, k de n (partilha replicada por XOR), com compromisso contra peças adulteradas |
//! | [`custodia_mutua`] | CTM1–CTM4 (nova, rodada atual) | Aba trancada por contato ("cofre da conversa"): item sob guarda dos dois lados ao mesmo tempo; qualquer um apaga para os dois a qualquer momento (par de chaves descartável destruído, não só removido de uma lista); sem metadado de participantes no servidor |
//! | [`custodia_do_cofre`] | proposta nova, sem ID de requisito ainda (10/10/2026; cache local com prazo fechado 11/10/2026, Rodada 21, resposta ao pedido do fundador contra extração forense) | Chave do cofre pessoal dividida 2 de 2 entre aparelho e servidor: nenhum dos dois sozinho reconstrói nada, nem o servidor sob ordem judicial — reaproveita `custodia` sem criptografia nova; cache local da chave reconstruída agora tem prazo (`verificar_cache`, 72h, mesmo padrão offline da Cláusula 6a) — um aparelho isolado da rede não reabre o cofre sozinho para sempre; falta transporte de rede e rota de servidor, ver o próprio módulo |
//! | [`custodia_por_categoria`] | nova, sem ID de requisito ainda (10/10/2026) | Política de continuidade em servidor por `cofre::Tipo` (granularidade por categoria): padrão de fábrica é só o aparelho para as nove categorias, sem exceção — o dono liga o complemento de `custodia_do_cofre` categoria por categoria, nunca todas de uma vez |
//! | [`pedido_de_guarda`] | nova, sem ID de requisito ainda (10/10/2026) | Guardar no cofre pessoal de quem recebe é pedido item por item pelo destinatário e aprovado ou negado pelo autor — nunca uma permissão genérica dada de antemão, e nunca confundir com a aba trancada (`custodia_mutua`, intocada); zero criptografia nova: aprovar produz a `liberacao::Autorizacao` normal (`Nivel::GuardarNoCofre`) que o resto do núcleo já valida; só o hash do conteúdo viaja, nunca um dado do remetente |
//! | [`apresentacao`] | APR1–APR8 | Como você aparece para cada contato: nome sugerido ou travado, assinado; mudança chega a todos; personas com chaves separadas, que ninguém liga entre si |
//! | [`aprovacao`] | ASS1–ASS12, BL4b | Assinaturas e aprovações k de n com vinculação dinâmica: documentos, pagamentos e publicação de versões (nenhum programador publica sozinho) |
//! | [`pacote`] | MTA1–MTA3 | Nome do arquivo, tipo, data e pasta cifrados junto com o conteúdo, num bloco de tamanho padronizado |
//! | [`acesso`] | ANO9–ANO14 | Endereços derivados e lotes de tamanho fixo com cobertura embaralhada: o servidor não liga pedidos entre si |
//! | [`recuperacao`] | REC1–REC6 | Recuperar a conta: 3 trilhas independentes (guardiões k de n, frase pessoal, pergunta importante — Rodada 21), qualquer 2 das 3 reconstroem o segredo; 72h de carência cancelável pelo aparelho antigo; sem a trilha dos guardiões, exige testemunha viva de um guardião contra coação |
//! | [`codec_palavras`] | nova, sem ID de requisito ainda (Rodada 21, 11/10/2026) | BIP-39 para a frase de recuperação pessoal: 32 bytes ↔ 24 palavras em português, wordlist oficial do padrão (não fabricada), checksum de 8 bits que detecta erro de dedo; aditivo, nunca substitui hex/QR — zero criptografia nova, só codificação determinística |
//! | [`rede`] | NET1–NET14 | Automático (Tor nos momentos de identidade, nas conversas blindadas e quando a rede bloqueia), Sempre Tor ou Rápido (sem Tor); nunca rebaixar em silêncio; vários operadores sorteados; cabeçalhos uniformes; Bluetooth sem identificador fixo |
//! | [`notificacao`] | NOT1–NOT7 | Cadência fixa: uma notificação por aparelho por janela, real ou de cobertura, do mesmo tamanho; texto genérico |
//! | [`contatos`] | CTT1–CTT8 | O nome que você salva, as chaves e as notas do contato num bloco cifrado de tamanho fixo; nome de perfil cifrado com chave entregue só a contatos |
//! | [`convites`] | SPAM1–SPAM6 | Sem telefone e sem spam: conversa só com convite de uso único e prazo; o servidor guarda só o hash |
//! | [`memoria`] | RZ3, RZ11, RZ12 | Segredo em memória blindada: zerado, travado na RAM, nunca impresso em registro ou relatório de falha |
//! | [`chamada`] | C88 | Estados da chamada: nenhuma mensagem que chega pela rede liga o microfone; só a ação local de atender (prova C88) |
//! | [`chamada_em_grupo`] | C88, GRP-C1–GRP-C3 | Chamada em grupo (v37, terceira rodada) como malha de chamadas 1:1 independentes, reaproveitando `chamada` sem alterá-lo: cada perna exige o próprio atendimento local, nenhuma ativa as outras; teto de participantes pela banda da malha |
//! | [`grupo`] | GRP-M1–GRP-M3 | Conversa em grupo (v37, terceira rodada): só a camada de membros e a regra efetiva (combinação mais restritiva de sempre); o protocolo de mensagens em grupo continua sendo o MLS, fora deste núcleo |
//! | [`metadados`] | MET1–MET5 | Caixa que muda todo dia, índice cego no disco com chave destruída depois de 48 h, registros legais selados para chave que não fica no servidor |
//! | [`ambiente`] | AMB1–AMB4 | Diagnóstico local do aparelho (bloqueio de tela, patch, teclado, opções de desenvolvedor, fontes desconhecidas); nada sai do aparelho |
//! | [`chaves_do_servidor`] | ROT1–ROT4 | Chave de assinatura do servidor diferente a cada dia, certificada por 2 de 3 raízes offline; a de ontem já não vale |
//! | [`prechaves`] | FS1–FS5 | Chaves de recebimento de vida curta, assinadas pela identidade e destruídas depois do prazo: envelopes antigos ficam ilegíveis para sempre, mesmo com o aparelho tomado |
//! | [`autenticacao`] | SRV1–SRV6 | Servidor: rotas negadas por padrão, prova por recurso (não por pessoa), assinatura que cobre método, caminho, corpo e horário, número de uso único |
//! | [`limites`] | SRV7–SRV9 | Servidor: cota por janela, castigo que dobra a cada falha até 24 h, memória com teto e falha fechada |
//! | [`selo`] | SLO1–SLO7 | Selo de liberação: artefato exato, todos os portões de segurança aprovados, assinado por três chaves diferentes fixadas no código (CI, responsável de segurança e auditor independente); sem ele, servidor e apps não funcionam |
//! | [`sentinela`] | SEN1–SEN5 | Autoverificação na partida e periódica: invariantes de segurança e autotestes criptográficos; qualquer achado trava (falha fechada) e o relatório diz onde e como corrigir |
//! | [`trafego`] | ANO1–ANO3 | Batida constante de envio: silêncio e conversa ficam iguais para quem observa a rede |
//! | [`vigia_de_sensores`] | VSE1–VSE8 | Vigia contínuo durante o uso: microfone ou câmera de outro app, tela capturada ou externa, sobreposição, acessibilidade, teclado de terceiros; na dúvida, fecha |
//! | [`autenticidade`] | AUT1–AUT5 | Nunca ser imitado: certificado do app conferido em tempo constante; endereços parecidos com os oficiais (caso Privnote) acusados antes de abrir |
//! | [`protecao_do_usuario`] | BLQ1–BLQ4, DEN1–DEN5, APG1–APG4 | Bloquear (lista só no aparelho, descarte em silêncio), denunciar só por iniciativa do usuário (cifrado para a moderação, tamanho fixo) e apagar a conta (pedido assinado, PIN real, uso único) |
//!
//! As conversas 1:1 e em grupo do NimbusChat **não** usam este núcleo para o protocolo de mensagens:
//! usam a libsignal (PQXDH + Triple Ratchet) e o MLS (RFC 9420), conforme o Manual.

// Nenhum `unsafe` no núcleo, exceto o mlock/munlock de `memoria` (marcado um a um). Código novo com
// `unsafe` não compila; liberar exige mudar esta linha, e a catraca (verificacao/catraca.py) reprova.
#![deny(unsafe_code)]
pub mod acesso;
pub mod aprovacao;
pub mod album;
pub mod ambiente;
pub mod apresentacao;
pub mod assinatura;
pub mod autenticacao;
pub mod cobranca;
pub mod chamada;
pub mod chamada_em_grupo;
pub mod chaves_do_servidor;
pub mod checkup;
pub mod coacao;
pub mod codec_palavras;
pub mod cofre;
pub mod comando_remoto;
pub mod contatos;
pub mod corporativo;
pub mod conversa;
pub mod convites;
pub mod custodia;
pub mod custodia_mutua;
pub mod custodia_do_cofre;
pub mod custodia_por_categoria;
pub mod pedido_de_guarda;
pub mod direto;
pub mod envelope;
pub mod escudo;
pub mod expiracao;
pub mod grupo;
pub mod higiene;
pub mod historico;
pub mod inspecao;
pub mod integridade;
pub mod liberacao;
pub mod limites;
pub mod metadados;
pub mod modos;
pub mod politica;
pub mod prechaves;
pub mod recuperacao;
pub mod rede;
pub mod registro;
pub mod retirada_de_aparelho;
pub mod selo;
pub mod senha;
pub mod sentinela;
pub mod sessoes;
pub mod memoria;
pub mod notificacao;
pub mod panico;
pub mod pacote;
pub mod trafego;
pub mod vigia_de_sensores;
pub mod autenticidade;
pub mod protecao_do_usuario;
pub mod validade;
pub mod voz;
