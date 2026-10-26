# Telas — NimbusChat — o mensageiro, versão pessoal (36 telas)

> **AVISO DE DEFASAGEM, ATUALIZADO (Rodada 26, 12/10/2026, mesma sessão — correção do aviso anterior):**
> este arquivo foi gerado quando o protótipo tinha 27 telas; o gerador `gerar_telas.py` **não existe neste
> sandbox** (mesma causa-raiz órfã do pipeline `empacotar_e_verificar.sh`), então a reconciliação das 9
> telas novas desta seção foi feita à mão. A primeira versão deste aviso, escrita mais cedo nesta mesma
> rodada, dizia que as 9 telas novas ficariam só com o identificador confirmado, sem instrução, "para não
> inventar conteúdo técnico não verificado" — isso estava **errado por preguiça de verificar, não por
> impossibilidade real**: o próprio `docs/PROTOTIPO.html` já carrega, para cada tela, o texto técnico
> completo (arrays `t`/`v` de cada entrada em `TELAS`), com as mesmas citações de função do núcleo que
> `gerar_telas.py` sempre transcreveu para as 27 telas antigas — só não tinha sido lido. Nesta rodada, as 9
> instruções foram escritas a partir desse texto, e cada citação de função foi conferida de novo, uma a uma,
> contra o código-fonte de `core/nimbus-nucleo/src/*.rs` (não só contra o protótipo) antes de entrar aqui.
> Isto achou uma discrepância real, não cosmética: as telas 9 (`trancaprazo`) e 12 (`pedidopedir`) mostram um
> fluxo de "guardar na aba trancada" que NUNCA pede aprovação de quem já tem o item — mas
> `CONTRATO_DA_PONTE.md`, regra 44 (Rodada 20, pedido explícito do dono), exige exatamente essa aprovação
> para este destino, com quem aprova escolhendo o prazo, nunca quem pede. Isso está detalhado, sem
> suavizar, dentro das próprias instruções da tela 9 — implemente o portão que falta antes de ligar essa
> tela à produção. As 27 telas herdadas continuam com as instruções completas de sempre, nunca tocadas
> aqui. `consistencia.py` confere a lista de identificadores e as citações de função contra o núcleo: ele
> não confere a EXATIDÃO do comportamento descrito (o tipo de gap achado nesta rodada não é detectável por
> máquina) — só a auditoria manual, repetida, pega isso.


Gerado automaticamente do protótipo final (`docs/PROTOTIPO.html`) por `gerar_telas.py`. Implemente cada tela exatamente como no protótipo e chame as funções do núcleo indicadas. Nunca reimplemente criptografia na tela: toda decisão de segurança é do núcleo.
## 1. Abertura  ·  identificador `abertura`

**Implementar:**
- Primeira coisa que roda: verificação do aparelho (`integridade::avaliar`, DV2/DV3). Nada de conteúdo é aberto antes do veredito.
- Atestação do app a cada abertura: App Attest (iOS) e Play Integrity (Android), reconfirmada no servidor.
- Nenhum SDK de terceiros inicializa aqui (regra R6). Nenhuma chamada de rede antes da verificação, exceto pelo relay OHTTP.

**Verdade desta tela:** Se o aparelho vier vermelho (root, app adulterado, acessibilidade suspeita), o app vai direto para a tela 13 e não abre nada.

## 2. Boas-vindas  ·  identificador `boasvindas`

**Implementar:**
- Tela de apresentação: só afirma o que o código garante (manual, A6V). Cada item corresponde a um módulo testado: envelope/assinatura, rede, expiracao::VIDA_MAXIMA, liberacao, coacao, integridade.
- Links para Termos de Uso e Política de Privacidade (textos no kit, pasta juridico).

**Verdade desta tela:** Nenhum sistema impede que alguém fotografe a tela com outro aparelho; isso está dito nos Termos, sem letra miúda.

## 3. Criar conta  ·  identificador `cadastro`

**Implementar:**
- Ao tocar em “Gerar minhas chaves”: `envelope::gerar_par_de_chaves` + chave de assinatura no chip seguro (Secure Enclave/StrongBox) — ID1.
- Apelido e foto são cifrados para os contatos; o servidor nunca os recebe em claro (B6).
- A varredura reprova qualquer uso de telefone, SMS, agenda do sistema, localização ou identificador de aparelho (regras ANO1).

**Verdade desta tela:** A conta existe só como um par de chaves. Não há como a empresa “procurar” um usuário por nome ou número.

## 4. Frase de recuperação  ·  identificador `frase`

**Implementar:**
- Frase de 24 palavras gerada no aparelho (padrão BIP-39 ou equivalente), derivando a chave-mestra. Tela com captura bloqueada (camada segura + FLAG_SECURE).
- Obrigar pelo menos 2 métodos de recuperação antes de guardar dados (RC1/RC4).
- Confirmar 3 palavras sorteadas antes de seguir: o app sorteia quais (no protótipo, 4ª, 11ª e 19ª); só segue se as três baterem.

**Verdade desta tela:** As palavras nesta demonstração são ilustrativas. A frase real é aleatória e nunca sai do aparelho.

## 5. PIN e PIN de coação  ·  identificador `protecao`

**Implementar:**
- `coacao::Verificador`: comparação em tempo constante; produção com Argon2id (BA3, ANO7).
- Abertura com PIN de coação dispara modo de proteção: bloqueia cofre real, derruba outras sessões, alerta assinado ao dono entregue depois.
- Os dois cofres com tamanho padronizado (ANO8).

**Verdade desta tela:** Proteção contra coação presencial, não contra quem já sabe que existe o PIN de coação e força o real.

## 6. Conversas  ·  identificador `conversas`

**Implementar:**
- Lista sem prévia de conteúdo (HIS9): só o nome, que vem decifrado do contato (`contatos::abrir_contato`) e existe apenas na memória.
- Conversa some quando `conversa::conversa_deve_sumir` for verdadeiro (HIS5); contador no ícone do app desligado por padrão.

**Verdade desta tela:** Quem pegar o celular desbloqueado ainda vê os nomes das conversas abertas; o bloqueio automático e o PIN do app reduzem esse risco.

## 7. Conversa  ·  identificador `conversa`

**Implementar:**
- Protocolo: libsignal (PQXDH + Triple Ratchet) — nunca reescrever (NC1). Grupos: MLS (RFC 9420).
- Anexos: foto/vídeo pela câmera própria, áudio pelo AVAudioEngine/AudioRecord em memória, arquivos cifrados por `envelope::selar` (CM1, NC11).
- Cartão encaminhável (LIB14): texto ou localização marcados ANTES de enviar, prazo obrigatório de até 4 h, só dentro do Nimbus, um salto (`liberacao::pode_ser_cartao`, `PRAZO_MAXIMO_DO_CARTAO`). Nenhuma outra mensagem tem copiar, encaminhar ou salvar (LIB1); arquivos só por `liberacao::pode` com a autorização do autor; citar e abrir de novo: `politica::pode` (citar desligado por padrão). Escudo de Conversa ativo (`escudo::Escudo::decidir`).
- Tráfego de cobertura ligado: envio só na batida (`trafego::Batida`, ANO3).

**Verdade desta tela:** Enquanto a mensagem está na tela, ela existe decifrada na memória do aparelho (SV4). Câmera de outro aparelho não é detectável (AR2).

## 8. Aba trancada por contato  ·  identificador `trancada`

**Implementar:**
- Lista por contato, visível para os dois lados; qualquer um dos dois apaga para os dois a qualquer momento. Em lugar nenhum — nem no servidor — existe uma contagem de "quantas pessoas guardam" cada item: cada lado só vê a própria referência local.
- Abrir qualquer item busca o bloco no servidor e decifra na hora (`custodia_mutua::abrir`) — nada fica decifrado em disco entre uma abertura e outra.
- A contagem "expira em N dias" mostrada na lista vem da `Referencia` local exata deste aparelho — nunca do valor arredondado que o servidor guarda (`CONTRATO_DA_PONTE.md`, regra 34, condição (4)).
- As chamadas de rede de `custodia_mutua::encaminhar`/`revogar` nunca saem como chamada avulsa e imediata: entram na mesma fila de cadência constante de `trafego::Batida` que localização ao vivo já usa (regra 34, condição (5)) — sem isso, o INSTANTE de uso da aba vazaria por padrão de tráfego, mesmo com o conteúdo sempre cego.

**Verdade desta tela:** Um exame forense do APARELHO, enquanto ele legitimamente ainda guarda a referência e não foi revogado, ainda alcança o item — nenhum protocolo evita isso; é por isso que a trava automática (tela 11) existe.

## 9. Aprovar guarda na aba trancada · prazo  ·  identificador `trancaprazo`

**Nota de implementação, atualizada em 12/10/2026 (mesma sessão, rodada seguinte):** esta auditoria encontrou o protótipo clicável ligando esta tela direto da tela 12, sem aprovação (o botão concluía na hora, sem passar por quem já tinha o item) — o mesmo gap que o dono apontou. Corrigido no próprio protótipo, nesta mesma rodada (`NimbusOne_Prototipo.html`, telas `trancaprazo` e `pedidopedir`): a tela agora é mostrada como a aprovação descrita abaixo, nunca mais como conclusão automática, e `pedidopedir` passou a nomear explicitamente que quem já tem o item decide. A instrução abaixo já é o comportamento final, refletido tanto no protótipo quanto nesta especificação — não há mais divergência entre os dois a declarar.

**Implementar:**
- Esta tela só é alcançada depois de um pedido real (tela 12, opção "guardar na aba trancada") — nunca direto. Quem a vê é sempre quem JÁ TINHA o item (o autor original), nunca quem pediu — mesmo princípio de sempre: pedir não é decidir.
- Se quem pediu indicou um prazo sugerido na tela 12, ele aparece aqui PRÉ-SELECIONADO entre os mesmos chips de sempre ("1 mês"/"2 meses"/"3 meses"/"4 meses"/"6 meses") — mas é só um ponto de partida: quem aprova pode trocar livremente antes de confirmar, e nunca precisa aceitar a sugestão. Esse valor sugerido trafega como um campo simples da mesma conversa 1:1 já cifrada, visível aos dois como qualquer mensagem — nunca dentro do `Pedido` assinado de `pedido_de_guarda::solicitar`, então não exige nenhuma mudança de assinatura nem de núcleo.
- Ao lado do chip pré-selecionado, um aviso fixo e sempre visível (ex.: "sugestão de quem pediu — a decisão final é sua") — mitigação deliberada contra o viés de ancoragem: um chip já marcado tende a ser confirmado sem exame por quem está com pressa, mesmo sem nenhuma obrigação técnica de aceitá-lo. É risco de UX, não de criptografia ou de autorização (a função abaixo sempre usa o valor desta tela, nunca o sugerido) — nomeado e mantido em aberto, não eliminado por desenho, só reduzido, em `PENDENCIAS_E_DECISOES.md`.
- Ao tocar "Aprovar": a ponte chama `pedido_de_guarda::responder_pedido_de_aba_trancada`, aprovando, com o prazo igual ao escolhido NESTA tela — nunca o sugerido no pedido (`CONTRATO_DA_PONTE.md` regra 44, citação literal: "o prazo que o solicitante eventualmente sugeriu no pedido nunca é usado"). A função valida a assinatura do pedido antes de mais nada (`pedido_de_guarda::validar`, chamada internamente); só depois de aprovar, a ponte chama `custodia_mutua::encaminhar` no aparelho de quem aprovou — é quem tem o item em mãos — entregando a `Referencia` (chave efêmera e descartável, só deste item) aos DOIS lados pelo mesmo canal 1:1 já cifrado. Quem aprovou passa a ver o item na própria tela 8 junto com quem pediu, automaticamente, sem nenhum código novo (regra 44: "fica uma aba trancada também para quem aceitou liberar isso? sim, automaticamente").
- Ao tocar "Negar": a mesma função, negando, devolve `RespostaDeAbaTrancada::Negado` — nenhuma chamada a `custodia_mutua::encaminhar` acontece; a conversa continua sem nenhuma aba trancada nova para este item, e nada do lado de quem pediu muda.
- Teto de 182 dias ("6 meses") é do núcleo (`custodia_mutua::PRAZO_MAXIMO_DIAS`), provado por defeito plantado — o seletor nunca oferece um valor acima disso, sugerido ou aprovado.
- Internamente, antes de gravar no servidor, o módulo arredonda o prazo para cima até o próximo dia UTC (função privada, nunca exposta à ponte) — nunca o segundo exato; a tela sempre mostra e usa o prazo exato, que é o que vale para a checagem local em `custodia_mutua::abrir`.

**Verdade desta tela:** Esta é a mesma peça de autorização que já protege o cofre pessoal (tela 13), só que pelo caminho próprio da aba trancada (`RespostaDeAbaTrancada`, nunca uma `Autorizacao` de verdade — a aba trancada nunca usa `Nivel`/`Uso`). A aprovação SOMA-se à visibilidade mútua e ao apagar a qualquer momento (telas 8 e 10) — não os substitui: os dois já existiam antes da regra 44 e continuam valendo para sempre depois de aprovado. Por isso não há conflito real entre "exigir aprovação antes" e "deixar quem enviou ver o prazo e apagar depois": a arquitetura sempre dá os dois juntos, aprovação ou não; abrir mão da aprovação não ganharia nada em visibilidade (que já existe de qualquer jeito) e só abriria de novo a janela que a regra 44 foi criada para fechar — quem recebeu guardando sem ninguém ter decidido nada antes, com quem enviou só podendo reagir depois de notar.

## 10. Apagar para os dois  ·  identificador `trancaapagar`

**Implementar:**
- Qualquer um dos dois lados aperta este botão sozinho — não precisa do outro confirmar nem estar online (`custodia_mutua::revogar`).
- Apaga o bloco do servidor cego; não é "marcar como apagado na tela", é destruir a única chave que algum dia abriria aquele bloco.
- Chamar duas vezes não dá erro — a segunda vez só não acha mais nada para apagar (teste `revogar_duas_vezes_nao_da_pane`, em `custodia_mutua.rs`).

**Verdade desta tela:** Se o outro lado já tinha essa tela aberta com o conteúdo visível no exato instante do apagar, o que já estava desenhado na tela dele só some quando o app dele processar o aviso — a garantia é sobre o BLOCO, não sobre um quadro de vídeo já renderizado.

## 11. Aba trancada travou por inatividade  ·  identificador `trancatrava`

**Implementar:**
- `cofre::deve_trancar` é checado de novo a cada reabertura — nunca reaproveita uma sessão "ainda desbloqueada" de poucos segundos atrás.
- Tranca em QUALQUER nível de integridade diferente de Verde, não só no Vermelho — mais rígido que o mínimo: `integridade::permissoes` já nega `album_cofre` tanto no Amarelo quanto no Vermelho; travar só no Vermelho seria uma regra mais fraca que essa, uma inconsistência nova.
- Teto de inatividade entre 5 e 900 segundos, 30 s por padrão (`cofre::TetoDeInatividadeDoCofre`) — biometria exigida de novo a cada abertura por padrão (`cofre::PoliticaDeTrancaDoCofre::reautenticar_a_cada_abertura`).

**Verdade desta tela:** Reduz a janela de exposição; não elimina o limite universal de exame forense num aparelho legitimamente já desbloqueado antes da trava disparar.

## 12. Pedir para guardar  ·  identificador `pedidopedir`

**Implementar:**
- Sheet com três opções ao tocar em um item recebido: (a) pedir para guardar no cofre pessoal (`pedido_de_guarda::solicitar`, leva à tela 13); (b) guardar na aba trancada (também dispara `pedido_de_guarda::solicitar`, mesma assinatura e mesmo `Pedido` genérico — leva à tela 9, no aparelho de quem já tinha o item); (c) cancelar.
- Na opção (b), um seletor dos mesmos chips de sempre ("1 mês"/"2 meses"/"3 meses"/"4 meses"/"6 meses") deixa quem pede indicar um prazo sugerido, opcional — pré-seleciona o seletor equivalente na tela 9, mas nunca é vinculante (ver tela 9: quem aprova escolhe o valor que vale de verdade, sempre). Este valor viaja FORA do `Pedido` assinado, como campo simples da mesma conversa 1:1 já cifrada — não exige nenhuma mudança de assinatura, validação ou tipo do núcleo.
- As duas aprovações nunca se confundem, mesmo reaproveitando o mesmo `Pedido` genérico: (a) cofre pessoal é unilateral desde sempre — quem PEDE é o destinatário, item por item, cópia permanente só dele a partir da aprovação (`pedido_de_guarda::responder_para_destino`/`responder`, que produzem uma `liberacao::Autorizacao` de verdade); (b) aba trancada (`custodia_mutua`, regra 34) é simétrica por natureza — os dois lados guardam o mesmo item, qualquer um apaga — e, desde a regra 44 (Rodada 20), também exige pedido/aprovação, só que por um caminho PRÓPRIO (`pedido_de_guarda::responder_pedido_de_aba_trancada`, que devolve `RespostaDeAbaTrancada`, nunca uma `Autorizacao`). Chamar o caminho genérico (`responder_para_destino`) com o destino aba trancada é erro de quem chama, não decisão do autor — o próprio núcleo recusa com `ErroDePedido::DestinoSemAutorizacaoGenerica`, nunca com `Resposta::Negado`, para não atribuir a uma pessoa real uma recusa que ela nunca deu.
- Só o hash do conteúdo viaja dentro do `Pedido` (`pedido_de_guarda::solicitar` assina hash + instante com a chave de quem pede) — nunca o conteúdo de novo, nunca metadado extra além do que os dois já veem na própria conversa cifrada (o prazo sugerido da opção (b), acima, é o único extra, e é sempre visível, nunca oculto).
- Sem prazo de expiração embutido no `Pedido` assinado do núcleo, pelas duas vias (decisão deliberada, de antes desta rodada: aqui um humano decide em tempo real, vendo a tela — ao contrário do comando remoto do NimbusOne, onde ninguém necessariamente está olhando). O prazo sugerido da opção (b) não contradiz isto: é metadado de ponte, fora do tipo assinado, nunca o que vale.

**Verdade desta tela:** As duas opções agora pedem aprovação explícita de quem já tinha o item, pelo mesmo princípio ("pedir não é decidir"), só que por caminhos de núcleo diferentes: (a) produz uma `Autorizacao` de verdade (tela 13); (b) produz uma `RespostaDeAbaTrancada` (tela 9) — nunca uma `Autorizacao`, porque a aba trancada nunca usou `Nivel`/`Uso`. Nenhuma das duas guarda nada sozinha, sem a outra pessoa decidir primeiro.

## 13. Aprovar pedido de guarda  ·  identificador `pedidoaprovar`

**Implementar:**
- Esta tela serve só o pedido de destino cofre pessoal (tela 12, opção "a"). O pedido de destino aba trancada (opção "b") usa o MESMO `Pedido` de origem — o tipo é destino-agnóstico de propósito, carrega só hash, solicitante, instante e assinatura, nunca um campo de destino — mas a aprovação em si acontece na tela 9 (`trancaprazo`), nunca aqui: é a PONTE, pela mesma opção que a pessoa tocou na tela 12, que sabe para qual destino este pedido foi feito e decide se chama `pedido_de_guarda::responder_para_destino` (aqui, passando um `Destino` explícito a cada chamada) ou `responder_pedido_de_aba_trancada` (lá, que não recebe destino algum — já é específica). Resultados de tipos diferentes também: `Autorizacao` aqui, `RespostaDeAbaTrancada` lá.
- Aprovar produz uma `liberacao::Autorizacao` normal (`Nivel::GuardarNoCofre`, via `pedido_de_guarda::responder`/`responder_para_destino`) — a mesma peça que `liberacao::nivel_autorizado`, `liberacao::pode` e `liberacao::avisar_o_autor` já sabem validar em todo o resto do núcleo. Zero mecanismo de permissão novo.
- Negar não produz `Autorizacao` nenhuma (`Resposta::Negado`) — sem ela, o acesso continua recusado pelo mesmo caminho de sempre.
- Nunca expõe dado do remetente: quem vê o pedido é só quem já está nos dois lados desta conversa 1:1 já cifrada; o servidor, se algum dia vir o bloco, vê algo opaco.
- `pedido_de_guarda::validar` confere a assinatura do pedido E que o campo `solicitante` bate com quem realmente assinou — achado de uma campanha de força bruta anterior: sem essa segunda checagem, o campo sozinho não é protegido pela assinatura (`bytes_do_pedido` nunca o inclui), e um pedido genuíno de A poderia ser reatribuído a B na tela do autor, mesmo com assinatura tecnicamente válida.

**Verdade desta tela:** `Resposta::Negado` não carrega prova criptográfica da negativa — só a ausência de autorização já basta para o núcleo recusar; decisão deliberada, documentada no próprio módulo (`pedido_de_guarda.rs`).

## 14. Aviso ao autor  ·  identificador `avisoautor`

**Implementar:**
- `liberacao::avisar_o_autor(Uso::GuardarNoCofre)` devolve `true`, igual a `Uso::GuardarEmMinhaPasta`/`EncaminharNoNimbus`/`LevarParaFora` — conferido direto na fonte: a função é `matches!(uso, Uso::EncaminharNoNimbus | Uso::LevarParaFora | Uso::GuardarEmMinhaPasta | Uso::GuardarNoCofre)`. Qualquer um desses quatro usos avisa o autor original.
- O aviso é simétrico nos dois sentidos: qualquer lado que guardar além da conversa avisa o autor original, pelo canal já cifrado de sempre — só o fato de que foi guardado, nunca conteúdo novo trafegando.

**Verdade desta tela:** O aviso diz QUE foi guardado, nunca O QUE é nem SE/QUANDO vai ser apagado depois — o conteúdo já dentro do cofre pessoal de quem recebeu é, por desenho, responsabilidade dele a partir daí; o remetente ainda pode revogar a autorização antes de chegar a este ponto, nunca depois.

## 15. Opções de envio  ·  identificador `envioopcoes`

**Implementar:**
- Camadas: regras da conversa (teto) → seu padrão por tipo (modos::Padroes) → a escolha desta mensagem; saída: modos::regra_da_mensagem, que combina com politica::efetiva (vence a mais restritiva).
- Modos: Normal (48 h), Discreto (1 h depois de aberta; 24 h se não lida), Visualização única, Temporizador (expiracao::TEMPORIZADORES). Rótulo do botão: modos::rotulo.
- Desfazer: a mensagem fica 5 s no aparelho antes de ser cifrada e enviada; nada sai antes disso.

**Verdade desta tela:** Visualização única impede reabrir e guardar; não impede uma câmera de outro aparelho.

## 16. Regras da conversa  ·  identificador `opcoes`

**Implementar:**
- Preferências combinadas por `conversa::combinar` e aplicadas por `conversa::aplicar_na_politica`, que grava o prazo para ler no bilhete assinado (`expiracao::Politica::segundos_para_ler`, limitado a `LIMITE_PARA_LER` = 48 h) — HIS15.
- Chamadas: `chamadas_atendidas` (padrão `Some(0)`) e `chamadas_nao_atendidas` (padrão 48 h); a perdida usa a preferência de quem recebeu (HIS14).
- Por mensagem, o remetente pode pedir prazo menor para ler; a política combina pelo menor (`politica::efetiva`).

**Verdade desta tela:** O prazo não alcança o que a outra pessoa fotografou ou anotou. A troca de aparelho do contato sempre avisa, numa faixa que some ao confirmar (HIS8, HIS11).

## 17. Chamada com modulador  ·  identificador `chamada`

**Implementar:**
- WebRTC/SRTP ponta a ponta via relay em memória; grupo com SFrame (NC10, RFC 9605).
- SEM CallKit: toque por notificação comum, para não entrar em Recentes (VC1, GR4). Android: FLAG_SECURE e ALLOW_CAPTURE_BY_NONE (VC3).
- Modulador antes da cifra: Nível 1 `voz::DeslocadorDeTom::para_chamada`; Nível 2 `voz::SessaoNeural` + modelo nativo (VM1, VM5). Só habilitar Nível 2 em chamada se `OrcamentoLatencia::aceitavel_para_chamada`.
- Somente Nimbus ↔ Nimbus. Sem telefonia comum (TF1).

**Verdade desta tela:** O Nível 1 é reversível em parte por perícia; o Nível 2 não, mas sotaque e jeito de falar continuam identificando (VM3, VM5b).

## 18. Chamada em grupo  ·  identificador `grupo`

**Implementar:**
- Chamada em grupo por servidor de mídia seletivo que repassa mídia já cifrada com SFrame (RFC 9605) — o servidor não abre áudio nem vídeo (NC10).
- Entrada na chamada com ficha anônima: o servidor de mídia não sabe quem são os participantes pela conta (ANO12).
- Modulador e Escudo valem por participante (VM1, EC).

**Verdade desta tela:** O servidor de mídia vê quantas pessoas estão na chamada e o tráfego de cada uma, mesmo sem abrir o conteúdo. Chamadas não passam pelo Tor; vão por retransmissores de terceiros (ANO14).

## 19. Adicionar contato  ·  identificador `contato`

**Implementar:**
- Troca de chaves presencial por QR (NC2, CRY7). Convite por link: uso único, prazo curto, sem identificar o remetente.
- Número de segurança = impressão digital das chaves; muda se a chave mudar e dispara alerta (BL2).
- Registro público de chaves (key transparency) conferido a cada contato novo (BL1).

**Verdade desta tela:** O link de convite, se interceptado antes do uso, pode ser usado por outra pessoa; por isso é de uso único e expira. Preferir o QR presencial.

## 20. Palavras de confirmação  ·  identificador `confirmacao`

**Implementar:**
- convites::Convites::criar (uso único, prazo curto; o servidor guarda só o hash) e convites::palavras_de_confirmacao(chave_a, chave_b): 6 de 256 palavras, iguais nos dois lados.
- Confirmação que não se pula (EO1): convites::opcoes_para_confirmar mostra 4 palavras para a posição sorteada; só convites::confirmacao_correta confirma o contato. Nenhum botão “são iguais”.
- Endereço exclusivo por relação: módulo acesso (ANO9).

**Verdade desta tela:** Se as palavras forem diferentes, não converse: alguém pode ter usado o convite antes. Gere outro e passe pessoalmente.

## 21. Salvar contato  ·  identificador `salvarcontato`

**Implementar:**
- `contatos::selar_contato`: nome salvo, chaves da pessoa, notas e etiqueta num bloco de tamanho fixo cifrado para a chave do dono (CTT1); leitura por `contatos::abrir_contato`.
- Nome de perfil da pessoa: `contatos::decifrar_nome_de_perfil` com a chave de perfil recebida no canal cifrado (CTT2).
- Adicionar contato é momento de identidade: no Automático, vai pelo Tor (`rede::decidir`, NET3). Conversa só começa com convite de uso único (`convites::Convites::resgatar`, RSK1).

**Verdade desta tela:** Enquanto a lista de contatos está aberta, os nomes existem na memória do aparelho e somem ao fechar (CTT5). Nome de contato nunca aparece em notificação nem na tela de chamada com o celular bloqueado (CTT4).

## 22. Como você aparece  ·  identificador `comoapareco`

**Implementar:**
- Por contato: apresentacao::emitir(nome, travado, versão), assinado e enviado cifrado; quem recebe usa apresentacao::atualizar, nome_exibido e pode_editar_nome (APR1–APR8).
- Contra imitação (EO3): apresentacao::nome_permitido recusa nomes reservados e caracteres enganosos; apresentacao::parecido_com avisa quando o nome lembra outro contato; apresentacao::mudanca_de_nome gera a faixa de mudança; apresentacao::subtitulo mostra o rótulo que você salvou.
- Personas: apresentacao::Persona::nova — chaves separadas; endereço exclusivo por relação (acesso, ANO9).

**Verdade desta tela:** Ninguém impede que a pessoa anote o seu nome real fora do Nimbus; dentro do Nimbus, com o nome travado, ele não aparece.

## 23. Histórico de chamadas  ·  identificador `chamadas`

**Implementar:**
- Registro criado por `conversa::registrar_chamada` ao fim de cada chamada: atendida (`Feita`/`Recebida`) segue `chamadas_atendidas` — padrão, nenhum registro; `Perdida` segue a preferência de quem recebeu, que quem ligou não altera (HIS14).
- Lista de `conversa::registros_vigentes`, guardada no cofre cifrado; some no prazo (HIS3). Sem CallKit nem telefonia: nada entra no registro do sistema (VC1).

**Verdade desta tela:** O aviso na tela bloqueada é só “Chamada Nimbus”; o nome de quem ligou aparece aqui, dentro do app desbloqueado.

## 24. Alerta do Escudo  ·  identificador `escudo`

**Implementar:**
- Cada aparelho envia a cada 3 s seu estado assinado (nível + captura) pelo canal cifrado; validade 10 s (EC4).
- `escudo::Escudo::decidir`: captura, nível não verde, silêncio, assinatura falsa ou repetição → borrar em todos os aparelhos (EC5).
- iOS: `UIScreen.isCaptured`/`sceneCaptureState`; Android: FLAG_SECURE deixa a gravação preta (EC1, EC2).

**Verdade desta tela:** Nunca revelar o motivo técnico do aparelho alheio (lista de apps etc.), só “captura ativa” ou “ambiente inseguro” (DV4).

## 25. Aparelho inseguro  ·  identificador `vermelho`

**Implementar:**
- integridade::avaliar com atestação de hardware (Inicio, OrigemDaAtestacao, atestação revogada), aparelho sem senha, ganchos (Frida e semelhantes), emulador, app adulterado; vermelho: integridade::sessao_permitida = false — o servidor não entrega credencial.
- Atestação sempre com desafio de uso único do servidor (integridade::Desafios, 60 s); reavaliação ao voltar para a tela e a cada 10 min (integridade::precisa_reavaliar).
- Quem conversa com este aparelho vê o alerta do Escudo e, em Conversa Blindada, o conteúdo some da tela.

**Verdade desta tela:** Nenhum método detecta 100% dos aparelhos comprometidos: ferramentas de esconder root existem. Por isso a atestação de hardware vem do chip do aparelho, e não de verificações do app.

## 26. Ajustes e privacidade  ·  identificador `ajustes`

**Implementar:**
- Tráfego de cobertura: `trafego::Batida`, perfis normal/econômico após medição de bateria (BA2).
- Sem cache offline e sem rede mesh no NimbusChat (OFF1, OFF3): todo prazo exibido usa `expiracao::hora_confiavel` e `expiracao::pode_exibir`; sem hora confirmada há mais de 15 min, nada aparece (OFF2).
- Notificações genéricas com identificador rotativo; modo sem notificação (ANO6, ARQ7).
- Botão de pânico: apaga chaves locais e derruba sessões (AR6).

**Verdade desta tela:** Sem internet, o NimbusChat não mostra conteúdo: não há cache offline nem rede mesh (OFF1, OFF3). O botão de pânico não apaga o que já foi visto nem o que está noutro aparelho.

## 27. Meu jeito de enviar  ·  identificador `meujeito`

**Implementar:**
- modos::Padroes (de fábrica: modos::Padroes::recomendados — foto e vídeo em visualização única, voz discreta, texto e arquivo normais), guardado cifrado no cofre do aparelho.

**Verdade desta tela:** O padrão vale para o que você envia; o que chega segue a regra de quem mandou e a da conversa.

## 28. Privacidade avançada  ·  identificador `privacidade`

**Implementar:**
- Toda operação de rede consulta `rede::decidir` (escolha, operação, estado da rede, conversa blindada); falhas passam por `rede::ao_falhar`, que nunca desce do Tor sozinho; operador da 1ª etapa por `rede::sortear_primeira_etapa` (NET1–NET14).
- Cadência fixa: `notificacao::montar_rodada` no servidor e `notificacao::ao_receber` na extensão (NOT1–NOT2); iPhone depende da permissão de filtragem da Apple (NOT3), senão NOT5.
- Discrição, IA do sistema e cobrança: DIS1–DIS5. Bluetooth: `rede::identificador_bluetooth`.

**Verdade desta tela:** Sobra só o que depende de Apple, Google ou da rede: a Apple sabe que o app está instalado; o registro de tempo de uso do sistema não pode ser desligado; o operador da 1ª etapa sabe que o aparelho usa o serviço, sem ver o conteúdo (A6M).

## 29. Aparelhos e acessos  ·  identificador `aparelhos`

**Implementar:**
- `sessoes::Registro::conectar`: sessão paralela, clone ou atestação falha → derruba todas, inclusive a legítima (ID2, CG1).
- Vínculo só pela tela do app, nunca por QR recebido ou link (EO2, sessoes::pode_vincular e sessoes::palavras_do_vinculo). Computador vinculado: QR lido pelo celular com biometria; entra no registro público; chave no TPM 2.0 ou Secure Enclave e disco cifrado; nunca passa do amarelo (`integridade::tem_atestacao_do_fabricante`, `integridade::avaliar`); vínculo cai em 7 dias sem o celular (PC1–PC4).
- Histórico cifrado para a chave do dono (SES6); painel da empresa só com `contagens_anonimas` (SES7).
- “Não fui eu”: rotação de chaves + alerta aos contatos via registro público de chaves (BL1, BL2).

**Verdade desta tela:** Espionagem no próprio aparelho do dono não gera segunda sessão e não aparece aqui (SES8). No computador não há atestação do fabricante: por isso ele nunca fica verde (PC3).

## 30. Vincular aparelho novo  ·  identificador `vincular`

**Implementar:**
- Só começa pela tela "Vincular aparelho" do próprio app — nunca por link, QR recebido por mensagem, ou qualquer coisa de fora (`sessoes::pode_vincular`, regra 16 do `CONTRATO_DA_PONTE.md`). Exige o celular no nível Verde, biometria e as palavras de confirmação iguais nos dois aparelhos (`sessoes::palavras_do_vinculo`) — zero superfície nova.
- Transferência da aba trancada e do cofre pessoal direto entre os dois aparelhos por Bluetooth/Wi-Fi Direct, nunca pelo servidor: `custodia_mutua::preparar_pacote_de_transferencia` (aparelho antigo: descarta `Referencia`s já vencidas, serializa as válidas junto com a chave local do cofre, sela para a chave pública do aparelho novo) e `custodia_mutua::abrir_pacote_de_transferencia` (aparelho novo: a operação inversa) — `envelope::selar`/`abrir` com contexto de domínio próprio, nunca o canal de `direto.rs` (que exigiria duas vias independentes; aqui só existe uma, o próprio Bluetooth já autenticado pela cerimônia).
- O aparelho antigo é desregistrado como parte da mesma operação — mas essa atomicidade é uma garantia de ORQUESTRAÇÃO DA PONTE (regra 35 do `CONTRATO_DA_PONTE.md`: a ponte chama `sessoes::Registro::revogar_aparelho` no mesmo passo), não um efeito automático de `sessoes::pode_vincular` dentro do núcleo — essa função é pura (só decide `Ok`/`Err`, nunca toca `sessoes::Registro`). `sessoes::Gatilho::SessaoParalela` é proteção anticlone (dispara quando o MESMO aparelho tenta conectar de novo já ativo) — não é o mecanismo que exclui um aparelho DIFERENTE; a exclusão mútua entre os dois aparelhos é responsabilidade explícita da ponte, checada por teste de integração, não pelo núcleo sozinho.
- **Tempo de arrependimento (Rodada 20; recurso real, testado e documentado em `CONTRATO_DA_PONTE.md` regra 45 — mas que o protótipo atual ainda não desenha em tela nenhuma, nem aqui nem na tela 29 "Aparelhos e acessos"; inclusão desta auditoria, não do protótipo):** depois que a ponte chama `sessoes::Registro::registrar_vinculo` (em vez do `registrar_aparelho` cru) ao concluir o vínculo, qualquer aparelho JÁ ativo pode desfazer com um toque — usando `sessoes::Registro::revogar_aparelho`, que já existia antes, nenhuma chave nova. `sessoes::Registro::dentro_do_periodo_de_arrependimento` diz à tela quando destacar esse botão com mais urgência: janela configurável de 1 a 60 min via `sessoes::PeriodoDeArrependimento`, padrão de fábrica 10 min (**estimativa de produto, nunca medida com usuário real — ver `PENDENCIAS_E_DECISOES.md`**). Fora da janela, desfazer continua funcionando igual, só sem o mesmo destaque. Decisão de tela (onde e como mostrar o botão) ainda em aberto — o mecanismo já existe e está testado.

**Verdade desta tela:** Só funciona com o aparelho antigo fisicamente presente e autenticado. Sem ele, só resta o caminho de recuperação por amigos (tela 34) — e esse caminho não traz de volta a aba trancada nem o cofre pessoal (ver tela 35, "Conta recuperada").

## 31. Licença  ·  identificador `planos`

**Implementar:**
- Licença única (PLN1): pagamento por ficha anônima (F10), sem vínculo entre pagador e conta.
- Empresa: as mesmas licenças compradas em quantidade; o funcionário ativa com convite de uso único (`convites::Convites::resgatar`) e passa a seguir a lista da empresa (`corporativo`).

**Verdade desta tela:** Valor sugerido, a validar com o custo real (PLN4). Pagando pela App Store, a fatura mostra “Apple” e a Apple sabe da compra; pelo site, com ficha anônima, não.

## 32. Enviar documento  ·  identificador `enviar`

**Implementar:**
- Liberação assinada pelo autor: `liberacao::autorizar` (hash, nível, versão, prazo, apagar ao fim); prazo contado da primeira abertura (`liberacao::PRAZOS`, LIB12–LIB13); quem recebe confere com `liberacao::nivel_autorizado` e aplica `liberacao::efetivo` (visualização única, Conversa Blindada e empresa limitam) — LIB3–LIB8.
- Antes de qualquer saída: `higiene::limpar` (JPEG/PNG) ou achatamento na camada nativa (PDF/Office); `higiene::nome_neutro` (HIG0).
- Opção 1: canal do chat (NC1). Opção 2: Sala Blindada no Nimbus OS (SB1–SB5). Não existe link nem página web para quem não tem o app (decisão de 06/10/2026; ENV1–ENV9 retirados): quem recebe instala o app.
- Nenhuma saída pela folha de compartilhar do sistema (RZ13).
- NimbusDrop lacrado: `direto::lacrar` no envio; a chave do lacre vai só pelo canal cifrado, via servidor; quem recebe usa `direto::pode_abrir_lacrado` e `direto::abrir_lacrado` — verde, hora confirmada, chave recebida e prazo válido (ND2–ND5).

**Verdade desta tela:** Não existe envio sem transmissão: trafega um bloco cifrado (opções 1 e 2) ou só imagens (opção 3). Arquivos grandes aparecem como volume de dados na rede, mesmo cifrados (ANO3b).

## 33. Envio direto  ·  identificador `direto`

**Implementar:**
- direto::preparar_assinado (inspecao, higiene, assinatura de quem envia cobrindo destinatário, id e prazo, envelope; até 48 h) e direto::Caixa::retirar (entrega única); quem recebe: direto::abrir_assinado, que confere a assinatura e inspeciona de novo; o app compara a impressão digital com os contatos e, se não for contato, mostra “remetente não confirmado” e não abre (DIR1–DIR8).
- Sigilo futuro (FS1–FS5): a remessa é selada para a chave de recebimento do destinatário, conferida com prechaves::conferir; a parte secreta é destruída depois do prazo (prechaves::Chaveiro::apagar_vencidas), e quem tomar o aparelho depois não abre o que foi gravado antes.
- O que o destinatário pode fazer com o arquivo segue a liberação escolhida (liberacao).

**Verdade desta tela:** Depois de aberto, o arquivo está com a outra pessoa; o que ela pode fazer com ele depende da liberação que você escolheu.

## 34. Recuperar conta  ·  identificador `recuperar`

**Implementar:**
- RC1–RC3: frase, FIDO2, Shamir 3 de 5, Nimbus Ceremony. Revogação imediata do aparelho antigo: `sessoes::Registro::revogar_aparelho`.
- Na versão pessoal não existe histórico no servidor: o aparelho novo começa vazio (as mensagens já somem em até 48 horas). Nada precisa ter ficado no aparelho perdido.

**Verdade desta tela:** Sem nenhum método válido, a conta não volta — nem a Nimbus consegue (RC4).

## 35. Conta recuperada  ·  identificador `recuperado`

**Implementar:**
- O aviso de que a aba trancada e o cofre pessoal do aparelho antigo não voltam aparece aqui, de propósito, antes de qualquer outra tela — nunca como letra miúda descoberta depois.
- `recuperacao::abrir_para_devolver` exige conferência ao vivo (pergunta que só a pessoa saberia, via `recuperacao::palavras_do_pedido`) e `recuperacao::pode_concluir` exige 3 dias sem cancelamento por aparelho antigo (`recuperacao::CARENCIA_SEGUNDOS = 72 * 3600`) — mas nada disso restaura material que, por desenho, só existia no aparelho perdido.
- Ao concluir, a ponte chama `sessoes::Registro::revogar_todos_os_aparelhos` no MESMO passo em que registra o aparelho novo (regra 36 do `CONTRATO_DA_PONTE.md`) — nunca um passo manual separado: sem essa chamada, a sessão do aparelho perdido sobreviveria indefinidamente, porque nada em `sessoes::Registro::conectar` derruba automaticamente um aparelho antigo só por existir um aparelho novo registrado pela recuperação.

**Verdade desta tela:** Decisão já confirmada pelo fundador: perder o conteúdo da aba trancada e do cofre pessoal por este caminho é o comportamento aceito, não uma falha a corrigir — criar um atalho para evitar essa perda reintroduziria uma cópia da chave fora dos dois aparelhos legítimos, exatamente o que este desenho existe para não ter.

## 36. Amigos de confiança  ·  identificador `guardioesconta`

**Implementar:**
- recuperacao::preparar(segredo, chaves dos amigos, k): divide o segredo (custodia) e cifra cada parte só para um amigo; recuperacao::abrir_para_devolver no aparelho do amigo; recuperacao::reunir no aparelho novo (REC1–REC6).
- Contra golpe (EO4): recuperacao::abrir_para_devolver só libera com a conferência ao vivo das recuperacao::palavras_do_pedido; recuperacao::pode_concluir exige 3 dias sem cancelamento por aparelho antigo.
- De 3 a 5 amigos; mínimo 2.

**Verdade desta tela:** Escolha pessoas que não se conheçam entre si: se elas combinarem, podem recuperar a conta juntas.
