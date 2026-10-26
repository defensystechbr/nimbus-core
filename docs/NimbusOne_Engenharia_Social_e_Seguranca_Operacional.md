# Engenharia social e segurança operacional — Nimbus

**Versão 37 · Confidencial**

A criptografia do Nimbus pode ser perfeita e mesmo assim a conta de alguém ser tomada: basta convencer a pessoa — ou um amigo dela, um administrador da empresa, um funcionário da Nimbus — a fazer a coisa errada na hora errada. Esta revisão olhou o produto pelos olhos de quem engana gente, não de quem quebra código: cada tela em que uma pessoa decide algo, cada conversa entre pessoas que o produto pressupõe, cada conta e cada processo da empresa que opera o serviço.

## 1. Princípio que guiou todas as correções

**Nenhuma proteção importante pode depender de alguém resistir à pressão no momento.** O golpista sempre cria pressa, medo ou autoridade. Por isso, onde dá, o produto troca a decisão humana por três mecanismos que não se convencem:

1. **Confirmação que não se pula** — o app pede uma ação que só quem fez a verificação de verdade consegue completar.
2. **Espera com aviso** — operações irreversíveis esperam, e quem pode ser vítima é avisado e cancela com um toque.
3. **Origem única** — operações sensíveis só começam numa tela do próprio app; nada que chega de fora (link, QR, mensagem) consegue iniciá-las.

## 2. Como a revisão foi feita

- As 54 telas do protótipo foram percorridas uma a uma, perguntando em cada decisão: "como um golpista faria a pessoa errar aqui?".
- Os fluxos que envolvem outras pessoas (convite, vínculo de aparelho, amigos de confiança, guardiões e administradores da empresa, pagamentos) foram simulados com o papel de golpista em cada ponta.
- As operações da empresa Nimbus (contas, domínio, lojas, publicação, atendimento, pedidos de autoridade) foram tratadas como alvo.
- Onde a correção cabia no núcleo, ela virou código com teste e defeito plantado; onde depende do app, do servidor ou de processo, virou requisito, regra da ponte ou procedimento.

## 3. O que foi corrigido no código do núcleo

| ID | Golpe | Correção | Função | Prova |
|---|---|---|---|---|
| EO1 | "É só tocar em *são iguais*": o golpista convence a pessoa a pular a comparação das palavras do convite | Não existe mais botão "são iguais". O app mostra quatro palavras e pede que a pessoa toque na que o outro **leu em voz alta**, numa posição sorteada | `convites::opcoes_para_confirmar`, `convites::confirmacao_correta` | C97 |
| EO2 | QR enviado "para verificar sua conta" ou "entrar no grupo": a vítima escaneia e vincula o computador do golpista (golpe usado por grupos ligados à Rússia contra usuários do Signal, relatado pelo Google Threat Intelligence Group em fevereiro de 2025) | Vínculo só começa na tela "Vincular aparelho" do próprio app; link, câmera do sistema ou QR recebido são recusados; exige celular verde, biometria e palavras iguais nos dois aparelhos, lado a lado | `sessoes::pode_vincular`, `sessoes::palavras_do_vinculo` | C98 |
| EO2b | O controle de sessão única tratava o computador vinculado como clone e derrubava tudo | Uma sessão por aparelho registrado; clone continua sendo o mesmo aparelho em paralelo ou aparelho não registrado | `sessoes::Registro::conectar` | C98 |
| EO3 | Aparecer como "Suporte Nimbus", "Segurança", "N1mbus"; nome quase igual ao de outro contato; trocar de nome em silêncio para se passar por outra pessoa | Nomes reservados e caracteres enganosos recusados (também em cartão forjado por app adulterado); nome parecido com outro contato ganha aviso; mudança de nome nunca é silenciosa; com nome travado, o nome que **você** salvou aparece sempre embaixo | `apresentacao::nome_permitido`, `apresentacao::parecido_com`, `apresentacao::mudanca_de_nome`, `apresentacao::subtitulo` | C99 |
| EO4 | O golpista pede aos amigos de confiança, por mensagem, "me manda a sua parte, perdi o celular" | O amigo só libera a parte depois de conferir **ao vivo** as palavras do aparelho novo; a recuperação espera **3 dias**, com aviso em todos os aparelhos antigos, e qualquer um deles cancela | `recuperacao::abrir_para_devolver`, `recuperacao::palavras_do_pedido`, `recuperacao::pode_concluir` | C100 |
| EO5 | Golpe do diretor: "sou o presidente, recupere agora os dados da conta do Fulano" | A ordem de recuperação de dados espera **24 horas**, visível a todos os administradores, que podem cancelar | `corporativo::autorizar_recuperacao` | C101 |

**Limite honesto do EO4:** as palavras provam *para qual aparelho* a parte vai — não provam *quem* está do outro lado. Um golpista com voz ou vídeo falsificado lê as palavras do próprio aparelho e elas batem. O que segura esse caso é: o amigo precisa **reconhecer** a pessoa (a tela pede uma pergunta que só ela sabe responder), precisam ser pelo menos dois amigos que não se conhecem, e o dono tem 3 dias para cancelar em qualquer aparelho antigo. Quem perdeu **todos** os aparelhos não tem quem cancele: para essa pessoa, a frase de recuperação guardada em papel continua sendo o caminho mais seguro.

## 4. Catálogo completo de vetores

Cada vetor tem a defesa do produto, a defesa de processo e o que sobra. "Sobra" não é desculpa: é o que precisa estar escrito para ninguém prometer o contrário.

| ID | Vetor | Defesa no produto | Defesa no processo | O que sobra |
|---|---|---|---|---|
| EO6 | **Falso suporte**: "Aqui é a Nimbus, precisamos da sua frase / PIN / palavras para não apagar a conta" | Não existe conta de atendimento dentro do app; nomes reservados bloqueados (EO3); a tela da frase e a de Ajustes dizem: "A Nimbus nunca pede frase, PIN, palavras ou código, nem por aqui, nem por e-mail, nem por telefone" | Atendimento só pelo site oficial; o roteiro do atendimento proíbe pedir qualquer segredo; quem pedir é desligado | A pessoa ainda pode entregar a frase a um golpista fora do app |
| EO7 | **Página ou app falso que imita o Nimbus** (o caso Privnote) | Não existe página web nem link do Nimbus: qualquer página que diga ser do Nimbus é falsa. O app confere o próprio certificado (AUT1) e acusa endereços parecidos com os oficiais antes de abrir (AUT3) | `autenticidade` |
| EO8 | **Página do Envio adulterada no próprio servidor** (quem toma a hospedagem troca o JavaScript e passa a ler a chave do link) | Página estática, sem script de terceiros, com política de segurança de conteúdo estrita; a impressão digital de cada versão publicada no registro público | Publicação da página com duas pessoas (mesma regra do app); hospedagem separada do resto; alerta se o arquivo servido mudar | Limite de qualquer criptografia no navegador: o navegador confia no servidor que entrega a página. Para o que não pode vazar: app ou Sala Blindada no nível máximo |
| EO9 | **App falso** nas lojas ou APK "versão premium" | Ajustes mostra a impressão digital do app; versões no registro público (`registro`); o app recusa rodar fora da loja oficial (atestação) | Monitorar lojas e sites; processo de derrubada; site oficial lista só os links das lojas | Quem instala de fora da loja pode ser enganado antes de abrir qualquer app verdadeiro |
| EO10 | **Coação física** ("desbloqueie agora") | PIN de coação abre o cofre-isca; o app nunca mostra se há PIN de coação configurado; travas de emergência | Não divulgar em material público quem usa a função | Quem coage e já sabe da função pode exigir os dois PINs |
| EO11 | **Olhar por cima do ombro** (PIN, frase) | Tela da frase com captura bloqueada; teclado do PIN com opção de embaralhar as teclas; frase confirmada por 3 palavras sorteadas | — | Quem filma a digitação de longe |
| EO12 | **Troca de chip (SIM swap)** | Não se aplica: não existe número de telefone na conta | — | — |
| EO13 | **Voz ou vídeo falsificados** pedindo aprovação de pagamento ou documento | Nenhuma aprovação vale por voz, vídeo, e-mail ou telefone: só assinada no app (ASS10); beneficiário novo espera 24 h | Treinar aprovadores: "pedido por telefone nunca vale" | — |
| EO14 | **Guardião corporativo coagido ou subornado** | 2 de 3 guardiões + 2 de 3 administradores + 24 h de espera (EO5) + registro de auditoria visível a todos | Guardiões de áreas diferentes, que não se reportam entre si; troca de guardião com a mesma exigência | Conluio entre o número mínimo de pessoas |
| EO15 | **Funcionário da Nimbus subornado ou coagido** | Por arquitetura, ninguém na Nimbus tem as chaves: não há conteúdo a entregar | Dupla aprovação de código e publicação; acesso à infraestrutura só com chave física; registros de acesso; separação de funções | Metadados mínimos que o servidor vê (declarados no Dossiê) |
| EO16 | **Tomada das contas da empresa**: registrador do domínio, DNS, App Store Connect, Google Play Console, repositório, e-mail corporativo, nuvem | — | Chave física (FIDO2) obrigatória em todas, sem SMS; bloqueio de registro no domínio; contas de administração separadas das de uso diário; duas pessoas para mudanças; monitoramento dos certificados emitidos para o domínio (registros públicos de certificados) | Falha do próprio fornecedor |
| EO17 | **Falsa ordem judicial** ou "autoridade" exigindo dados por telefone ou e-mail | Não há conteúdo a entregar; os registros de acesso do Marco Civil (art. 15) são cifrados e mínimos | Só se atende ordem que chega por canal oficial e é conferida pelo advogado; nunca por telefone; registro de cada pedido | — |
| EO18 | **Golpe da cobrança**: "seu pagamento falhou, informe o cartão" | Pagamento só na loja do celular ou no site oficial, com ficha anônima; o app nunca pede cartão em conversa | Comunicação oficial só pelo site | — |
| EO19 | **Aparelho do contato roubado e desbloqueado** | Troca de aparelho do contato sempre avisa (faixa que não se silencia); Escudo; contato vermelho não conversa | — | Mensagens enviadas antes do dono perceber o roubo |
| EO20 | **Cansaço de alertas** (de tanto aviso, ninguém lê) | Alertas de segurança são raros, específicos e não se desligam; nada de aviso genérico | Teste de usabilidade mede se as pessoas leem os alertas | — |
| EO21 | **Mensagem "do sistema" dentro da conversa** ("sua conta será apagada, toque aqui") | Não existem mensagens do sistema nas conversas: avisos de segurança só numa tela própria do app | — | — |
| EO22 | **"Reinstale para corrigir"** (a pessoa apaga o app e perde a conta, ou instala um falso) | Recuperação só pela frase ou pelos amigos de confiança; nenhum backup em nuvem a ser "restaurado" por terceiros | — | — |
| EO23 | **Funcionário desligado que ainda tem acesso** | Suspensão imediata pelo console; sem internet, o app trava em 15 minutos | Desligar no ato da demissão (a tela diz isso) | O que ele já viu antes |
| EO24 | **Pressão para "liberar o arquivo para fora"** | Só o autor libera, arquivo por arquivo; quem recebe não aumenta a liberação | — | Foto da tela por outro aparelho |
| EO25 | **Atendimento ao cliente usado para mapear usuários** ("qual é a conta do fulano?") | A Nimbus não sabe quem é quem: não há nome, telefone nem e-mail na conta | O roteiro do atendimento não tem como consultar conta por pessoa | — |
| EO26 | **Grupo com convidado infiltrado** | Entrada em grupo só por convite de quem já está; troca de chave ao entrar e ao sair (MLS); lista de participantes visível | — | Quem foi convidado de propósito por um participante |
| EO27 | **O próprio dono se engana** porque não entendeu a tela | Cada tela tem a "verdade desta tela"; linguagem simples | Teste com 5 a 8 pessoas reais antes de publicar; repetir a cada mudança de fluxo sensível | Só a observação de gente mostra |

**Golpes com IA (EO28 a EO36)** — voz e vídeo falsificados em massa, golpe escrito por IA, robô de atendimento falso, injeção de instruções no agente de IA do time, pacote com nome parecido, falso pesquisador, falso jornalista ou investidor, recrutamento de funcionário e QR físico adesivado — estão em `NimbusOne_Ataques_com_IA_e_Defesa_em_Profundidade.md`, seção 7.

## 5. Procedimentos operacionais que passam a ser obrigatórios

1. **Política do que a Nimbus nunca pede**, publicada no site e repetida no app: frase, PIN, palavras de confirmação, código de convite, cartão, nem pede para instalar nada fora da loja.
2. **Endurecimento das contas da empresa (EO16)**: inventário de todas as contas críticas; chave física obrigatória em cada uma; nenhuma recuperação por SMS; bloqueio de registro do domínio; revisão trimestral de quem tem acesso; procedimento de emergência guardado com duas pessoas.
3. **Atendimento (EO6, EO25)**: roteiro que proíbe pedir segredo e que não tem consulta de conta por pessoa; atendimento só por canal oficial; registro de tentativas de golpe para alertar usuários.
4. **Pedidos de autoridade (EO17)**: canal oficial único; conferência pelo advogado; resposta só com o que existe; registro.
5. **Marca e derrubada (EO7, EO9)**: monitoramento de apps e domínios parecidos; procedimento de derrubada junto às lojas e registradores.
6. **Treino e exercício**: simulação autorizada de golpe contra a equipe (telefone, e-mail, mensagem) duas vezes por ano; resultados viram correção, não punição.

## 6. Testes de engenharia social no pentest

O Plano de Pentest passa a ter engenharia social **autorizada por escrito** no escopo (PT-26 a PT-31): convencer alguém a pular a confirmação do convite; fazer vincular um computador por QR enviado; conseguir partes de recuperação com amigos por mensagem; se passar por suporte; pedir recuperação de dados corporativos com urgência; publicar página falsa do Envio.

## 7. Verificar antes de usar

1. A opção de embaralhar o teclado do PIN (EO11) e a pergunta de reconhecimento na tela do amigo (EO4) são requisitos de app: conferir nas Tarefas T5 e T8.
2. Que nenhuma página web do Nimbus existe no ar no lançamento (qualquer uma seria falsa) e que o monitoramento de domínios parecidos está ativo.
3. O relatório do Google Threat Intelligence Group (fevereiro de 2025) sobre o abuso de "aparelhos vinculados" no Signal, citado no EO2 — conferir a publicação original no blog do Google Cloud antes de citar externamente.
4. Os prazos de espera (3 dias na recuperação social, 24 h na recuperação de dados corporativos) são decisões de produto: confirmar com o fundador e com os primeiros clientes corporativos.
