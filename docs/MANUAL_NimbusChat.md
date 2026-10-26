# Manual do NimbusChat

Gerado por máquina do Manual Técnico v37 completo (`docs/referencia/MANUAL_TECNICO_COMPLETO.docx`) por `gerar_manuais_por_produto.py`. Seções só do outro produto foram omitidas; comuns aos dois foram mantidas. Em caso de dúvida, vale o manual completo.

**NimbusOne**

Manual Técnico Completo v37.0 — NimbusChat e NimbusOne

Tudo em português · Arquitetura, código entregue, instruções ao time, plano de trabalho e validação · 27/09/2026, revisado em 03/10/2026 · Confidencial · substitui todas as versões anteriores

*Etiquetas: \[SIM\] implementável · \[PARCIAL\] implementável com limite declarado · \[NÃO\] impossível por fator externo, com alternativa. “(validar)” = conferir antes de implementar. Todo requisito tem código e toda prova tem critério de aprovação.*

**PARTE A — A SOLUÇÃO (para o fundador)**

## A0. Estrutura final dos produtos (vale sobre qualquer trecho anterior)

Dois aplicativos, uma conta. O NimbusChat é só comunicação; o NimbusOne é o celular blindado. Onde este manual citar agenda, notas, álbuns, arquivos ou senhas como parte do NimbusChat, leia NimbusOne (Cofre Pessoal): os requisitos técnicos não mudam, só o app que os contém.

| **Aplicativo** | **O que tem**                                                                                                                                                                                                                                                             | **Onde roda**                                                                                                               |
|----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------|
| **NimbusChat** | Conversas 1:1 e em grupo · chamadas de voz e vídeo 1:1 e em grupo · mensagens de voz · fotos, vídeos e arquivos na conversa · temporárias por conversa e por mensagem · visualização única · Escudo de Conversa · modulador de voz · Nimbus Envio para quem não tem o app | No aparelho, modo efêmero (proteção matemática)                                                                             |
| **NimbusOne**  | Cofre Pessoal: agenda, notas, álbuns, arquivos e senhas, tudo com validade · celular virtual (Nimbus OS) para WhatsApp, Signal, Instagram e outros · Sala Blindada · assinatura de documentos e aprovação de pagamentos                                                   | Cofre Pessoal no aparelho (matemática); celular virtual, Sala e aprovações de alto valor na máquina confidencial (hardware) |

- **PRD5 \[SIM\]** Um abre o outro: no NimbusOne, o ícone “Conversas” abre o NimbusChat; no NimbusChat, “Cofre” abre o NimbusOne. Conteúdo passa de um app ao outro só pelo cofre cifrado da conta, nunca pela folha de compartilhar do sistema.

- **PRD6 \[SIM\]** Por que não um app só: o NimbusChat é gratuito ou barato, entra nas lojas sem restrição e cresce sozinho; o NimbusOne é premium, e no iPhone o celular virtual precisa abrir pelo navegador (regra 4.2.7 da App Store). Juntos, um travaria o outro.

- **NB1 \[NÃO\]** Não haverá NimbusBank. Um produto com “Bank” no nome sugere instituição financeira e atrai exigências de autorização e supervisão (validar com jurídico). Bancos entram pelo programa Nimbus Trusted Banking e pelas aprovações do NimbusOne — sem a NimbusOne virar banco.

## A0B. NimbusChat comparado ao WhatsApp e ao Signal

Correção de premissa, com verdade: o Signal passou a testar, em setembro de 2026, o cadastro sem número de telefone (Signal Login), com pagamento único antiespam (validar no blog oficial do Signal). Portanto “sem telefone” deixa de ser exclusividade nossa. Os diferenciais reais estão na última coluna.

| **Recurso**                                                    | **WhatsApp** | **Signal**               | **NimbusChat**                                                                                                  |
|----------------------------------------------------------------|--------------|--------------------------|-----------------------------------------------------------------------------------------------------------------|
| **Conversas, grupos, voz, fotos, arquivos, reações, enquetes** | Sim          | Sim                      | Sim                                                                                                             |
| **Chamadas de voz e vídeo 1:1 e em grupo**                     | Sim          | Sim                      | Sim — mídia cifrada ponta a ponta (SFrame em grupo)                                                             |
| **Temporárias e visualização única**                           | Sim          | Sim                      | Sim — por conversa e por mensagem, para toda mídia, com prazo assinado pelo servidor e apagamento criptográfico |
| **Cadastro sem telefone**                                      | Não          | Em teste (validar)       | Sim, desde o início, sem depender de loja de apps para o pagamento                                              |
| **Servidor não vê seu IP**                                     | Não          | Não por padrão (validar) | Sim — em todas as escolhas; Tor automático nos momentos críticos                                                |
| **Esconder quando você conversa (tráfego de cobertura)**       | Não          | Não                      | Sim                                                                                                             |
| **Nada gravado no aparelho**                                   | Não          | Não (histórico local)    | Sim — modo efêmero; histórico no servidor cego                                                                  |
| **Borrar a conversa se o outro grava a tela**                  | Não          | Não                      | Sim — Escudo de Conversa                                                                                        |
| **Modulador de voz no aparelho**                               | Não          | Não                      | Sim — dois níveis                                                                                               |
| **PIN de coação com cofre-isca**                               | Não          | Não                      | Sim                                                                                                             |
| **Nomes de arquivos cifrados junto com o conteúdo**            | —            | —                        | Sim                                                                                                             |
| **Envio para quem não tem o app, com prazo e revogação**       | Não          | Não                      | Sim — Nimbus Envio e Sala Blindada                                                                              |
| **Status/histórias, canais, comunidades, pagamentos**          | Sim          | Parcial                  | Não, por decisão: ampliam metadados. Reavaliar depois do lançamento                                             |
| **Vários aparelhos na mesma conta**                            | Sim          | Sim                      | Sim, só com cerimônia presencial e alerta no registro público                                                   |

- **NC14 \[PARCIAL\]** Qualquer comparação pública com WhatsApp ou Signal passa por verificação na data da publicação: os dois evoluem rápido, como mostrou o cadastro sem telefone do Signal.

## A1. A ideia central: cada app roda onde é mais seguro — o cliente vê um só celular

Colocar tudo numa máquina virtual (VPS) resolve portabilidade, mas cria uma perda: o conteúdo passa a existir decifrado no servidor. A solução fora da caixa elimina essa perda sem abrir mão de nada que você pediu. O cliente usa dois apps da mesma conta (A0): o NimbusChat para comunicação e o NimbusOne para o cofre pessoal e o celular virtual. Por trás, cada app roda no lugar que dá a garantia máxima:

| **Tipo de app**                                                                                             | **Onde roda**                                                                                                     | **Garantia**                                                                                                                            |
|-------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| **Apps Nimbus: Chat, chamadas Nimbus, contatos, agenda, notas, Álbum Cofre, Álbum Comum, arquivos, senhas** | No aparelho, em modo efêmero: nada gravado; dados chegam cifrados, são abertos só na memória e apagados ao fechar | Máxima (matemática): nem o aparelho guarda nada, nem o servidor consegue ler. Melhor do que rodar numa VPS                              |
| **Apps de terceiros: WhatsApp, Signal, Telegram, Instagram, assinatura de documentos**                      | No Cloud Phone: Android virtual confidencial exclusivo do cliente, exibido dentro do Nimbus OS                    | Alta (hardware + chave dividida): a empresa não consegue ligar nem ler a máquina do cliente (A3). Melhor do que rodar no celular físico |

Resultado para o cliente, idêntico ao que você descreveu: um app só; nada fica no celular; trocou de aparelho, entra e a vida inteira está lá — inclusive WhatsApp e Signal. Resultado técnico: os apps Nimbus não perdem nenhuma garantia, e os apps de terceiros ganham proteção que não teriam no celular comum.

## A1B. Dois produtos, um ecossistema — decisão confirmada

Confirmado: são dois aplicativos, com públicos, preços e ciclos de desenvolvimento diferentes. Fazer um só atrasaria o lançamento do chat e misturaria promessas distintas.

| **Produto**    | **O que é**                                                                                                                       | **Por que separado**                                                                                                              |
|----------------|-----------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------|
| **NimbusChat** | Mensageiro, chamadas de voz e vídeo, figurinhas, temporárias, modulador de voz. Sem número de telefone, sem e-mail, sem documento | Vai à App Store e ao Google Play sem restrição; lança primeiro; é a marca e a porta de entrada                                    |
| **Nimbus OS**  | O celular virtual: agenda, contatos, notas, álbuns, arquivos, senhas e, no Cloud Phone, apps de terceiros                         | Depende de infraestrutura confidencial e custo por cliente; no iPhone entra pelo navegador; lança depois, como assinatura premium |

- **DP1 \[SIM\]** Conta única: quem já usa o NimbusChat entra no Nimbus OS com a mesma identidade, sem novo cadastro.

- **DP2 \[SIM\]** O NimbusChat NUNCA roda dentro do Cloud Phone, nem como app instalado na máquina virtual. Dentro do Nimbus OS existe apenas um atalho, que sai da máquina virtual e abre o app nativo do NimbusChat no próprio aparelho. Motivo em A2B.

- **DP4 \[SIM\]** Uma sessão por conta continua valendo entre os dois produtos: o NimbusChat autentica no aparelho físico; o Nimbus OS autentica a máquina virtual do cliente. São papéis distintos, nunca duas sessões do mesmo app.

- **DP5 \[SIM\]** Quem usa só o NimbusChat tem um mensageiro completo. O Cofre Pessoal e o celular virtual ficam no NimbusOne; o mesmo login abre os dois.

- **DP3 \[SIM\]** Ordem de lançamento: NimbusChat primeiro (receita e base de usuários), Nimbus OS depois.

## A0C. Plano corporativo: suspender sem ler, recuperar só com a chave mestra

Para empresas e governos, nos dois apps. Dois poderes separados de propósito: tirar o acesso de alguém é imediato e não lê nada; ler o conteúdo exige a chave mestra, dividida entre guardiões, e só vale para conta já suspensa.

| **Ação**                                  | **Quem pode**                             | **Precisa de**                             | **Lê o conteúdo?**  |
|-------------------------------------------|-------------------------------------------|--------------------------------------------|---------------------|
| **Convidar funcionário**                  | 1 administrador                           | Convite de uso único                       | Não                 |
| **Suspender acesso**                      | 1 administrador, até 5 contas por ordem   | Assinatura com chave física e biometria    | Não                 |
| **Suspender mais de 5 contas de uma vez** | 2 de 3 administradores                    | Assinaturas                                | Não                 |
| **Reativar conta suspensa**               | 2 de 3 administradores                    | Assinaturas                                | Não                 |
| **Recuperar conteúdo de conta suspensa**  | 2 de 3 administradores e 2 de 3 guardiões | Ordem assinada e chave mestra reconstruída | Sim, só dessa conta |
| **Trocar administradores**                | 2 de 3 administradores atuais             | Assinaturas                                | Não                 |
| **NimbusOne (fornecedor)**                | Ninguém                                   | —                                          | Nunca               |

- **COR1 \[SIM\]** Lista de contas autorizadas assinada pelos administradores da empresa (assinatura híbrida pós-quântica). O servidor só entrega; nunca assina (código pronto, módulo corporativo).

- **COR2 \[SIM\]** O aparelho só abre a conta corporativa se ela estiver na lista mais recente e dentro da validade (padrão 15 minutos). Conta suspensa: o aparelho apaga as chaves na hora.

- **COR3 \[SIM\]** Sem internet: quando a lista vence, o app trava sozinho. A hora usada é confiável: voltar o relógio do celular não estende o acesso, e reinício sem internet exige conectar.

- **COR4 \[SIM\]** O aparelho recusa lista forjada, mais antiga ou com número pulado; nenhuma conta ativa some sem ficar registrada como suspensa; o servidor não publica mais contas que as licenças contratadas.

- **COR5 \[SIM\]** Chave mestra: par de chaves pós-quântico gerado em cerimônia; a parte secreta é dividida na hora entre até 7 guardiões, k de n (módulo custodia), e nunca existe inteira — nem no servidor, nem com a NimbusOne, nem com um administrador.

- **COR6 \[SIM\]** Ao entrar na empresa, o aparelho do funcionário sela as chaves da conta corporativa para a chave mestra; o pacote fica no servidor, ilegível para todos, e só abre para a conta a que pertence.

- **COR7 \[SIM\]** Recuperação só de conta já suspensa, com 2 de 3 administradores e 2 de 3 guardiões: a empresa não lê escondido quem ainda trabalha. O conteúdo vai para outra conta ativa; a identidade do ex-funcionário fica encerrada para sempre — ninguém conversa “como se fosse ele”.

- **COR8 \[SIM\]** Sem chave mestra (opção da empresa na contratação): suspender equivale a apagar para sempre; nem a empresa recupera.

- **COR9 \[SIM\]** O funcionário não é administrador: não tem chave de administrador, não reativa a própria conta, não muda política nem administradores (testado).

- **COR10 \[SIM\]** Aviso permanente na conta corporativa e aceite no convite: a empresa pode suspender o acesso e, se habilitado, recuperar o conteúdo depois da suspensão. Quem conversa com uma conta corporativa vê “conta corporativa gerenciada”.

- **COR11 \[SIM\]** Conta pessoal separada da corporativa no mesmo app: a lista da empresa vale só para a conta corporativa; a suspensão nunca toca a pessoal.

- **COR12 \[PARCIAL\]** Grupos: ao suspender, o funcionário sai de todos os grupos corporativos com troca de chave (MLS), e mensagens futuras ficam ilegíveis para ele. A construir na camada de grupos.

- **COR13 \[SIM\]** A política da empresa pode desligar mensagens temporárias nas contas corporativas (obrigação de guarda). O que já expirou não se recupera nem com a chave mestra.

- **COR14 \[SIM\]** Registro de auditoria: toda ordem (suspender, reativar, recuperar, trocar administradores) entra no registro público da empresa (árvore de Merkle) e aparece para todos os administradores.

- **COR15 \[SIM\]** Administradores agem só dentro do console, com chave física FIDO2 e biometria. Nenhuma ordem por e-mail, telefone ou link; a NimbusOne não suspende contas de clientes.

- **COR16 \[PARCIAL\]** Limite: o que o funcionário viu ou fotografou antes da suspensão não pode ser recolhido. Por isso: suspender no ato do desligamento; visualização única, Escudo e bloqueio de exportação reduzem o risco.

- **COR17 \[SIM\]** O nome da organização e os nomes dos administradores nunca aparecem para o funcionário nem para quem conversa com ele, e não existem no servidor: a lista corporativa só tem identificadores. O funcionário vê “conta corporativa gerenciada” e, com clareza, o que a organização pode e não pode fazer.

## A0D. Análise de riscos do plano corporativo

| **Ameaça**                                                   | **Decisão**                                                                                                                          | **O que sobra**                                                                  |
|--------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
| **Administrador desonesto ou invadido suspende todo mundo**  | Mais de 5 contas por ordem exige 2 de 3; tudo no registro; alerta a todos os administradores                                         | Um administrador suspende até 5 contas antes de ser detido                       |
| **Administrador reativa um cúmplice**                        | Reativação exige 2 de 3                                                                                                              | Conluio de 2 administradores                                                     |
| **Empresa lê escondido quem ainda trabalha**                 | Recuperação só de conta suspensa — a suspensão é visível                                                                             | Nada                                                                             |
| **Empresa se passa pelo ex-funcionário**                     | Identidade encerrada; dados vão para outra conta                                                                                     | Nada                                                                             |
| **Funcionário offline continua usando**                      | Lista vence em 15 minutos; relógio confiável; reinício sem internet exige conectar                                                   | Até 15 minutos de uso offline (ajustável)                                        |
| **Servidor ou pessoa da NimbusOne mantém ou reativa alguém** | O servidor não assina listas; o aparelho recusa lista forjada, antiga ou pulada                                                      | O servidor pode atrasar a lista nova, no máximo até a validade                   |
| **Guardiões combinados entre si**                            | Precisam também de 2 administradores; tudo no registro                                                                               | Conluio de 2 guardiões e 2 administradores                                       |
| **Guardiões perdem suas peças**                              | k de n tolera perdas; nova divisão com k de n                                                                                        | Se sobrarem menos de k, os dados de contas suspensas ficam ilegíveis para sempre |
| **App adulterado ignora a ordem de apagar**                  | Chaves presas no chip ao app original; o servidor recusa app sem atestação                                                           | Validar por plataforma                                                           |
| **Engenharia social contra o suporte**                       | A NimbusOne não suspende contas de clientes                                                                                          | Nada                                                                             |
| **Pessoas de fora conversando com conta corporativa**        | Aviso visível na conversa                                                                                                            | A pessoa de fora decide o que envia                                              |
| **Lei trabalhista e de proteção de dados**                   | Aviso e aceite no convite; política de uso da empresa                                                                                | Validar com jurídico em cada país                                                |
| **Falta de pagamento**                                       | Carência contratual; depois, o servidor deixa de entregar listas e os apps travam; dados seguem cifrados até o apagamento contratual | Definir prazos em contrato                                                       |

## A0E. Planos e contratação (substituem tabelas de preço anteriores)

Valores sugeridos, a validar com o custo real medido no piloto. Pagamento sem identificação pelo site (ficha anônima) ou fatura corporativa com descritor neutro.

| **Plano**                      | **Para quem**                                       | **O que inclui**                                                                                                                                                  | **Valor sugerido**                                      |
|--------------------------------|-----------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------|
| **Licença NimbusChat**         | Qualquer pessoa                                     | O mensageiro completo; um plano só                                                                                                                                | US\$ 4,99 por pessoa/mês                                |
| **NimbusChat para empresas**   | Empresas                                            | As mesmas licenças, compradas em quantidade, com o console: convidar, suspender, reativar, chave mestra opcional e auditoria                                      | US\$ 4,99 por licença/mês; mínimo de 5; console incluso |
| **NimbusOne Cofre**            | Pessoa física                                       | Cofre pessoal com validade; Nimbus Envio                                                                                                                          | US\$ 6,99/mês                                           |
| **NimbusOne Completo**         | Pessoa física e executivo                           | Tudo do Cofre, celular virtual confidencial, Sala Blindada, aprovações pessoais e Licença NimbusChat                                                              | US\$ 29/mês                                             |
| **NimbusOne Empresas**         | Empresas                                            | Tudo do Completo para cada usuário, console corporativo, chave mestra com guardiões, aprovações da empresa (2 de 3) e auditoria                                   | US\$ 39 por usuário/mês; mínimo de 5                    |
| **NimbusOne Governo e Defesa** | Governos, segurança pública, infraestrutura crítica | Edição FIPS com algoritmos CNSA 2.0, infraestrutura dedicada ou no país do cliente, custódia com o próprio cliente, auditoria dedicada, implantação pela Defensys | Sob consulta                                            |

- **PLN1 \[SIM\]** NimbusChat tem um plano só (a licença). A empresa compra várias licenças e ganha o console sem custo extra.

- **PLN2 \[SIM\]** NimbusOne em escada simples: guardar (Cofre), usar tudo (Completo), gerir equipe (Empresas), exigência de Estado (Governo e Defesa).

- **PLN3 \[SIM\]** Contratação dentro do app: plano, número de usuários, recuperação com ou sem chave mestra, forma de pagamento e confirmação. Área do cliente: plano, licenças em uso, renovação e faturas.

- **PLN4 \[PARCIAL\]** O preço do Completo e do Empresas depende do custo da máquina virtual confidencial por usuário: medir no piloto antes de publicar preços.

## A0F. Histórico que some para os dois: mensagens, chamadas e a própria conversa

Cada pessoa escolhe as próprias preferências; a conversa usa sempre a combinação mais restritiva entre os participantes. O que você define vale também no aparelho de quem liga ou escreve para você. Estes padrões substituem qualquer padrão anterior deste manual.

| **Preferência**                                                                        | **Padrão**                                                          | **Como combina entre os participantes**                                                                                      |
|----------------------------------------------------------------------------------------|---------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------|
| **Mensagens somem depois de enviadas**                                                 | 48 horas, que é também o máximo para qualquer mensagem, lida ou não | Vale o prazo mais curto entre todos; nada passa de 48 horas                                                                  |
| **Mensagem não lida**                                                                  | Some em até 48 horas; quem envia pode pedir menos, nunca mais       | Vale o prazo mais curto; o limite de 48 horas vai assinado em cada mensagem                                                  |
| **Chamada atendida**                                                                   | Não fica registrada                                                 | Vale a mais restritiva entre os dois                                                                                         |
| **Chamada não atendida**                                                               | Fica 48 horas na lista de chamadas de quem recebeu, depois some     | No aparelho de quem recebeu, vale a escolha dele — quem ligou não apaga esse aviso; no de quem ligou, vale a mais restritiva |
| **A conversa some da lista quando não restar mensagem**                                | Ligado                                                              | Basta um pedir                                                                                                               |
| **Nenhuma marca na conversa (chamada, chamada perdida, mensagem expirada ou apagada)** | Ligado                                                              | Basta um pedir                                                                                                               |
| **Qualquer um pode apagar tudo, na hora, dos aparelhos de todos**                      | Desligado                                                           | Só vale se todos ligarem                                                                                                     |

- **HIS1 \[SIM\]** Regras combinadas pela mais restritiva entre os participantes, inclusive em grupo (código pronto, módulo conversa). As preferências viajam pelo canal cifrado ponta a ponta, que já autentica quem as enviou.

- **HIS2 \[SIM\]** Chamada atendida não deixa registro em nenhum dos dois aparelhos (padrão). Chamadas Nimbus nunca entram no registro de chamadas do sistema (sem CallKit nem telefonia, VC1).

- **HIS3 \[SIM\]** Registro de chamadas, enquanto existir, fica no cofre cifrado e some no prazo por apagamento criptográfico.

- **HIS4 \[SIM\]** Nenhuma marca dentro da conversa: não aparece “chamada de voz”, “chamada perdida”, “mensagem apagada” nem “mensagem expirada”.

- **HIS5 \[SIM\]** A própria conversa some da lista dos dois quando não resta mensagem: não fica o registro de que vocês conversaram. O contato continua salvo nos contatos.

- **HIS6 \[SIM\]** Apagar tudo, na hora, dos aparelhos de todos: só se todos tiverem ligado essa opção. Sem isso, cada um apaga o próprio aparelho, e o prazo combinado apaga o resto.

- **HIS7 \[SIM\]** Caixa do servidor: mensagem não entregue ou não lida espera no máximo o prazo da conversa ou 48 horas, o que for menor — e seria ilegível de qualquer forma.

- **HIS8 \[SIM\]** Exceção de segurança: o alerta de que o contato trocou de aparelho (proteção contra aparelho fantasma) nunca é silenciado. Aparece como faixa temporária no topo, some ao confirmar e não vira histórico. Mudança de regra da conversa aparece do mesmo jeito, para ninguém ser surpreendido.

- **HIS9 \[SIM\]** Rastros fora da conversa desligados por padrão: prévia das mensagens na lista, contador no ícone do app, confirmação de leitura e “digitando…”. Não existe “online” nem “visto por último”.

- **HIS10 \[SIM\]** Cabeçalho da conversa mostra o prazo vigente (ex.: ⏱ 48 h): é regra, não histórico, e permite a cada um saber o que vai sumir.

- **HIS11 \[PARCIAL\]** Limites: com “não registrar”, uma chamada perdida só é conhecida pelo aviso genérico na hora; o prazo não alcança o que a outra pessoa fotografou, anotou ou lembrou; um app adulterado do outro lado poderia desobedecer — o servidor recusa app sem atestação (DV).

- **HIS12 \[PARCIAL\]** Contrapartida: quem sofre assédio perde a prova quando a conversa some. Decisões: bloquear é sempre possível e imediato; apagar tudo à força exige o consentimento dos dois.

- **HIS13 \[SIM\]** Contas corporativas: se a empresa exige guarda (COR13), a conta corporativa não aceita prazo menor imposto de fora; a outra pessoa vê “conta corporativa gerenciada” e decide se continua.

- **HIS14 \[SIM\]** Chamada não atendida: quem recebeu vê na lista de chamadas quem ligou, por 48 horas (padrão, ajustável), e depois o aviso some. É a única exceção à regra “a mais restritiva vence”: quem ligou não consegue apagar o aviso de que ligou. No aparelho de quem ligou, o registro da tentativa segue a regra mais restritiva entre os dois.

- **HIS15 \[SIM\]** Mensagem enviada e não lida some em até 48 horas, qualquer que seja o prazo da conversa; quem envia pode pedir um prazo menor para aquela mensagem. O prazo vai assinado no bilhete da mensagem (módulo expiracao) — alterá-lo invalida a assinatura. Depois de lida, vale o prazo normal da conversa.

## A0G. Nada sai da conversa sem a sua autorização: mensagem não sai; arquivo e cartão encaminhável só se você liberar, com prazo

Regra de ouro, sem exceção: mensagem nunca sai. Texto, mensagem de voz, vídeo circular, figurinha e localização não se copiam, não se encaminham, não se guardam e não se levam para fora — nem com autorização. Print e gravação de tela não existem no NimbusChat, em nenhum caso. Arquivo (foto, vídeo, documento) só sai se quem enviou liberar, arquivo por arquivo.

| **Liberação escolhida por quem envia** | **O que quem recebe pode fazer**                                          | **Observação**                                                                     |
|----------------------------------------|---------------------------------------------------------------------------|------------------------------------------------------------------------------------|
| **Nada (padrão)**                      | Só ver dentro da conversa; o arquivo some com ela                         | —                                                                                  |
| **Guardar no cofre**                   | Guardar no cofre do NimbusOne, onde continua cifrado                      | Não sai do Nimbus                                                                  |
| **Encaminhar no Nimbus**               | Também encaminhar a outro contato Nimbus, que recebe com as mesmas regras | Continua protegido                                                                 |
| **Levar para fora**                    | Também salvar no celular, em arquivos ou mandar por outro mensageiro      | Fora do Nimbus, o arquivo deixa de ser protegido — quem envia é avisado ao liberar |

- **LIB1 \[SIM\]** Mensagem nunca sai, em nenhum caso, nem com autorização (código pronto, módulo liberacao; testado para todos os tipos de mensagem). Não existe botão de copiar, encaminhar ou salvar mensagem.

- **LIB2 \[SIM\]** Print e gravação de tela bloqueados sempre (camada segura no iPhone, proteção de tela no Android, Escudo de Conversa). Não há opção para liberar.

- **LIB3 \[SIM\]** Arquivo só sai com autorização de quem enviou, arquivo por arquivo; o padrão é nada. Cada nível inclui os anteriores.

- **LIB4 \[SIM\]** A autorização é assinada pelo autor (assinatura híbrida pós-quântica) e viaja cifrada com o arquivo: quem recebe não consegue aumentar o nível, e autorização de outra pessoa ou de outro arquivo não vale.

- **LIB5 \[SIM\]** Quem encaminha dentro do Nimbus pode reduzir o nível para o próximo destinatário, nunca aumentar.

- **LIB6 \[SIM\]** O autor pode revogar a liberação antes do uso, com uma autorização nova (versão maior); vale sempre a mais recente.

- **LIB7 \[SIM\]** O autor é avisado, pelo canal cifrado, quando o arquivo é encaminhado ou levado para fora.

- **LIB8 \[SIM\]** Limites que vencem a liberação: visualização única (nunca sai), Conversa Blindada (no máximo o cofre) e política da empresa.

- **LIB9 \[SIM\]** Saída para fora só pelo componente ExportacaoAutorizada; em qualquer outro lugar do código a varredura reprova. Fotos saem sem GPS, modelo do aparelho ou autor; quando o formato não permite limpar (PDF, Office), o app avisa antes de sair.

- **LIB10 \[SIM\]** Responder citando fica desligado por padrão; quando a conversa permite, a citação é uma referência ao original, que some junto com ele — nunca uma cópia.

- **LIB11 \[PARCIAL\]** Limites: depois de levado para fora, o arquivo está fora do controle do Nimbus; a revogação não alcança o que já saiu; uma câmera de outro aparelho apontada para a tela não pode ser impedida.

- **LIB12 \[SIM\]** Liberação com prazo: 30 segundos, 1 minuto, 2 minutos, 1 hora, 2 horas, 4 horas ou sem prazo. O prazo conta a partir de quando quem recebe abre o conteúdo; ao fim, a liberação volta a nada. O prazo vai assinado — tirá-lo invalida a liberação.

- **LIB13 \[SIM\]** O autor pode pedir que o próprio item suma da conversa quando o prazo da liberação acabar.

- **LIB14 \[SIM\]** Cartão encaminhável (única exceção para mensagens): um texto ou uma localização — por exemplo, um endereço — marcado ANTES de enviar como encaminhável dentro do Nimbus, com prazo obrigatório de no máximo 4 horas. Nunca depois de enviado, nunca para fora do Nimbus, nunca guardado; voz, vídeo circular e figurinha nunca viram cartão.

- **LIB15 \[SIM\]** Um salto só: o que chega por encaminhamento (autor diferente de quem enviou) não pode ser encaminhado de novo — garantido pela assinatura do autor, que ninguém mais consegue produzir.

- **LIB16 \[SIM\]** Sem marca d’água em conteúdo de conversa: nada no arquivo identifica quem o recebeu ou de onde veio.

## A0H. Auditorias independentes (v25 e v26): o que foi encontrado sem ninguém pedir

- **AUD1 \[SIM\]** v25: Contradição grave: a política antiga permitia copiar, encaminhar e salvar MENSAGENS quando a conversa permitisse, e citar vinha ligado por padrão. Corrigido: a política não decide mais nada sobre saída; o módulo liberacao decide, e mensagem nunca sai.

- **AUD2 \[SIM\]** v25: Remetente malicioso: um app adulterado pode cifrar conteúdo fora do padrão. Teste novo prova que o app recusa sem travar, em todos os formatos inválidos.

- **AUD3 \[SIM\]** v25: Código de produção sem nenhuma conversão que possa travar o app (as quatro que existiam estavam protegidas, e foram trocadas por conversões que não têm como falhar).

- **AUD4 \[SIM\]** v25: Manual: quatro trechos ainda descreviam a regra antiga (salvamento em três níveis, saída só pelo link, bloqueios configuráveis na política, visualização única). Todos reescritos.

- **AUD5 \[SIM\]** v25: Mutação completa, em todos os módulos (não só nos alterados): 92 de 92 defeitos plantados pegos.

- **AUD6 \[SIM\]** v26–v27: a marca d’água (inclusive uma “invisível” em toda conversa) contrariava o princípio de anonimato — seria um rastro de quem recebeu. Removida de todo o material; na v27, também a opção que restava na Sala Blindada.

- **AUD7 \[SIM\]** v26: faltava limite de tentativas de PIN no aparelho. Agora: 3 erros livres, espera que dobra, e no 10º erro as chaves do aparelho são apagadas (a conta volta pela frase de recuperação).

- **AUD8 \[SIM\]** v26: a ferramenta de mutação foi cortada por tempo no meio de uma rodada e deixou um defeito plantado num arquivo; a cópia de segurança o restaurou sozinha, como projetado, e a integridade foi conferida antes de continuar.

- **AUD9 \[SIM\]** v27: a contagem de módulos estava errada na v25 e na v26 (diziam 30; eram 29 — a troca de salvamento por liberacao foi só de nome). Corrigida: agora são 32.

- **AUD10 \[SIM\]** v27: o nome da empresa aparecia na conta corporativa e na tela de acesso suspenso. Removido; o nome da organização e dos administradores não existe no servidor (COR17).

- **AUD11 \[SIM\]** v27: o teto de 48 horas para toda mensagem tornou redundante a trava antiga de não lidas; o defeito plantado que a testava virou equivalente e saiu, junto com a redundância no código.

- **AUD12 \[SIM\]** v28: a integridade tirava do app qualquer pessoa com serviço de acessibilidade de terceiros — inclusive quem usa gerenciador de senhas. Agora, quando o sistema esconde o conteúdo desses serviços (Android 14+), o aparelho fica amarelo, e os leitores de tela oficiais continuam funcionando.

- **AUD13 \[SIM\]** v28: a lei que eu mesmo escrevi num teste de propriedade estava errada (exigia o teto de 48 horas num campo, e não no prazo final assinado). O código estava certo; o teste foi corrigido para conferir o prazo real.

- **AUD14 \[SIM\]** v28: o botão do modo de envio ficava escondido atrás da barra de digitação no protótipo. Agora o próprio campo de digitação mostra o modo, e o ícone ⏱ abre as opções.

- **AUD15 \[SIM\]** v29: ninguém tinha auditado as dependências do núcleo contra falhas conhecidas. Auditadas contra a base RustSec (71 pacotes): nenhuma vulnerabilidade em aberto, e a auditoria entrou no verificar.sh.

- **AUD16 \[SIM\]** v29: os três “verificar em aparelho” eram pendências passadas ao usuário. Viraram testes prontos no código (tests/aparelho_real.rs e servidor_atestacao.rs), que o programador roda com um comando.

- **AUD17 \[SIM\]** v29: a regra de varredura que proíbe log de conteúdo também barrava os prints dos arquivos de teste (que imprimem só o nome do cenário e não vão para produção). Ajustada para excluir tests/, examples/ e verificacao/, mantendo o bloqueio no código do app — conferido com um exemplo.

- **AUD18 \[SIM\]** v30 (auditoria adversarial): varredura de todo o código de produção atrás de pânicos, estouro de índice, comparação fora de tempo constante, aleatoriedade fraca e segredo sem apagar. O núcleo já estava sólido (todo leitor de dados externos checa limites antes de fatiar; PIN e senhas usam ct_eq; toda aleatoriedade é OsRng; todo segredo é Zeroizing). Faltavam provas — agora existem: fuzzing (C79) e prova de tempo constante do PIN.

- **AUD19 \[SIM\]** v31: o teste de tempo constante fazia 9.500 verificações de PIN com Argon2id de 64 MiB e não estava marcado como pesado. O cargo test — primeiro passo do verificar.sh e da integração contínua — o rodava em modo de depuração, e o comando do técnico parecia travado por horas. Agora: um único teste C80 com os quatro casos e 15 medições cada (cerca de 6 segundos), fora do cargo test do dia a dia, como o fuzzing.

- **AUD20 \[SIM\]** v31: na tabela de fases, a lista de provas da fase 2 tinha sido trocada por um resto de código de uma substituição automática malfeita; quem lesse o critério de aprovação não via nada. Restaurada, com C79 e C80.

- **AUD21 \[SIM\]** v31: números que não batiam com o código — 338 testes e 1 milhão de entradas de fuzzing. O real: 334 testes rápidos, fuzzing de 50 mil entradas por alvo (2 milhões no modo completo) e a prova de tempo. Corrigidos na origem.

- **AUD22 \[SIM\]** v31: a ficha do módulo coacao mandava trocar a derivação por Argon2id em produção, mas o código já usa Argon2id de 64 MiB. E a versão do Rust fixada no projeto (1.91.0) não batia com a da integração contínua (1.91.1). Ambos corrigidos.

- **AUD23 \[SIM\]** v32: ataque de força ao núcleo com 27 milhões de entradas aleatórias e malformadas contra os 18 leitores que recebem bytes de fora (envelope, chaves, assinatura, cofre, contatos, pacote, custódia, recuperação, inspeção, higiene, Nimbus Envio, envio direto, registro público e notificação): nenhum pânico. É o mesmo princípio do cargo-fuzz da indústria, rodando em Rust estável e reproduzível.

- **AUD24 \[SIM\]** v32: prova de sigilo de metadados (C81): dois contatos com nomes e notas de tamanhos muito diferentes viram cifras do mesmo tamanho, e o mesmo contato cifrado duas vezes dá cifras diferentes. O servidor não distingue “Mãe” de um nome longo pelo comprimento, nem sabe que dois blocos são o mesmo contato.

- **AUD25 \[SIM\]** v32: prova de ponta a ponta do limite de senha configurável (C82): 3 erros livres, espera que dobra a cada erro, e apagamento das chaves no 10º — além da política tolerante, que nunca apaga. Confirma o que já era possível ajustar em código.

- **AUD26 \[SIM\]** v33: ataque de força bruta máximo — 45 milhões de entradas aleatórias e malformadas em uma só passada contra os 18 leitores que recebem bytes de fora, direto no binário compilado. Nenhum pânico. A integração contínua repete a cada versão com 54 milhões (3 milhões por alvo).

- **AUD27 \[SIM\]** v33: automação da entrega — o script verificar_pacote.py extrai o pacote e confere, sem ninguém à mão, que o núcleo dentro de cada kit é idêntico byte a byte ao núcleo avulso, que o manual e o protótipo são os mesmos arquivos em toda parte, que cada kit passa na própria consistência, que nenhum artefato pesado foi empacotado e que não sobrou termo de versão antiga. O script empacotar_e_verificar.sh gera os ZIPs e roda essa verificação de uma vez.

- **AUD28 \[SIM\]** v34: os números subiram e os materiais não acompanharam — a mutação passou de 143 para 151 defeitos plantados e o núcleo de 34 para 36 módulos, mas decks, manual e leia-mes ainda diziam 143 e 34. Os 151 foram rodados e todos pegos antes de o número entrar nos materiais. O verificar_numeros.py passou a conferir módulos e provas, e a achar sozinho a versão mais nova do manual.

- **AUD29 \[SIM\]** v34: as tabelas de módulos do código e do LEIAME do núcleo listavam 34 dos 36 — faltavam album e panico, justamente os álbuns automáticos e as travas de emergência. Incluídos nas duas e no manual.

- **AUD30 \[SIM\]** v34: uma mutação interrompida com o disco cheio corrompeu a compilação e fez um teste falhar sem motivo, embora o código-fonte estivesse idêntico, byte a byte, ao aprovado. O verificar.sh agora restaura sozinho arquivos deixados por mutação interrompida e avisa quando há menos de 3 GB livres; o remédio, se acontecer, é apagar a pasta target e rodar de novo.

- **AUD31 \[SIM\]** v35: faltava o ataque que quebra protocolos reais — um adversário que controla a rede e pode guardar, repetir, reordenar e devolver mensagens legítimas sem quebrar a cifra (modelo Dolev-Yao). Escrito e provado (C86, tests/adversario_ativo.rs): mensagem repetida, fora de ordem, de outra conversa ou adulterada é rejeitada, porque a identidade da conversa e o número de sequência vão presos no dado autenticado (AAD) e o receptor só aceita o próximo número esperado. Os 5 testes passam; o número de testes rápidos subiu para 360.

- **AUD32 \[SIM\]** v36: estudo dos ataques públicos a WhatsApp, Signal, Telegram, Confide, Wickr e semelhantes (14 famílias, seção A0X5), verificado porta por porta no código. Duas portas estavam abertas no núcleo e foram fechadas: (1) texto com reordenação bidirecional (RTLO), que fazia um .exe parecer .jpg e um link parecer outro site — o CVE-2022-28345 do Signal —, agora saneado antes de exibir (C87); (2) chamada forçada a atender sem toque do usuário — o CVE-2019-17191 do Signal —, agora impossível, porque só a ação local de atender liga o microfone (C88). As outras 12 já estavam cobertas. 368 testes; 37 módulos.

- **AUD33 \[SIM\]** v37: a revisão encontrou dois avisos de compilação nos testes do núcleo (nome de variante fora do padrão, hoje EuDesligo, e uma importação sem uso), que reprovariam o portão de zero avisos do CI; ambos corrigidos. O Plano de CI citava 11 regras de varredura quando eram 32; hoje são 33, com a regra nimbus-sem-rede-mesh. 385 testes; 163 de 163 defeitos plantados pegos.

- **AUD34 \[SIM\]** v37, segunda rodada de revisão: duas vulnerabilidades fechadas no núcleo. (1) A hora do servidor chegava ao núcleo sem autenticação: quem controlasse a rede poderia repetir uma hora antiga e esticar todos os prazos — agora a hora é assinada junto com um desafio de uso único do aparelho e não pode voltar no tempo (expiracao::emitir_hora, expiracao::conferir_hora). (2) O prazo do NimbusDrop viajava fora do dado autenticado: quem recebia podia esticá-lo — agora alterar o prazo invalida o lacre. 387 testes; 168 de 168 defeitos plantados pegos.

- **AUD35 \[SIM\]** v37, terceira rodada (revisão sistemática de todos os campos que viajam pela rede e de todas as funções que recebem a hora): mais duas falhas fechadas. (1) As esperas entre erros de PIN usavam a hora entregue pela ponte: quem pegasse o aparelho adiantaria o relógio do sistema e pularia todas as esperas — no modo Tolerante, que nunca apaga, isso permitiria tentar todos os PINs. Agora as esperas só contam tempo real decorrido (coacao::RelogioDeEspera); reiniciar o aparelho não encurta nada. (2) O prazo da remessa direta (envio de Nimbus para Nimbus) viajava fora do dado autenticado, a mesma classe de falha do NimbusDrop; agora está dentro. 389 testes; 171 de 171 defeitos plantados pegos.

- **AUD36 \[SIM\]** v37, quarta rodada (leitura linha a linha do núcleo de criptografia, do envio direto e do registro; clippy com regras de segurança; varredura de todas as conversões de tamanho): três falhas fechadas e um endurecimento. (1) O envio direto não autenticava quem enviou: qualquer pessoa com a chave pública do destinatário podia mandar um arquivo que pareceria vir de um contato — agora é assinado, e a assinatura cobre o destinatário (DIR8). (2) O registro público não tinha prova de consistência: o servidor podia reescrever o passado sem ser pego (BL6). (3) Chaves ML-KEM recebidas de fora não passavam pela verificação da FIPS 203 (CRY9). (4) O perfil de produção agora interrompe em estouro aritmético em vez de dar a volta em silêncio. 399 testes; 179 de 179 defeitos plantados pegos — dois deles, na primeira passada, revelaram testes fracos que foram reforçados.

- **AUD37 \[SIM\]** v37, quinta rodada (engenharia social e segurança operacional: as 58 telas, os fluxos que envolvem outras pessoas e as operações da empresa, com o papel de golpista em cada ponta). Cinco correções no núcleo (EO1 a EO5), com 15 defeitos plantados novos, dois dos quais revelaram testes fracos, reforçados. Encontrado no caminho: o controle de sessão única derrubava tudo quando o computador vinculado conectava junto com o celular — contradição com o PC1, corrigida com uma sessão por aparelho registrado. 405 testes; 194 de 194 defeitos plantados pegos. Catálogo completo de 27 vetores em NimbusOne_Engenharia_Social_e_Seguranca_Operacional.md.

- **AUD38 \[SIM\]** v37, sexta rodada (atacante com IA e defesa em profundidade: o que o atacante leva em cada camada se passar). Duas melhorias no núcleo: sigilo futuro fora da libsignal com chaves de recebimento que se destroem (módulo prechaves, FS1–FS5) e ML-DSA na variante hedged (PQS7). Avaliado e não alterado, com motivo: compromisso de chave na cifra. 410 testes; 38 módulos; 201 de 201 defeitos plantados pegos. Catálogo completo em NimbusOne_Ataques_com_IA_e_Defesa_em_Profundidade.md.

- **AUD39 \[SIM\]** v37, sétima rodada (back-end: as cinco perguntas clássicas — rotas autenticadas, senhas, tentativas, RLS e dados cifrados — e a pergunta "se a porta abrir, o que sai?"). Dois módulos novos no núcleo (autenticacao e limites) e o banco do servidor com RLS, testado num PostgreSQL 16 real e com 12 defeitos plantados no esquema. Correção conceitual registrada: senha não se criptografa, guarda-se resumo lento com sal; no Nimbus, não há senha no servidor. 418 testes; 40 módulos; 213 de 213 defeitos plantados pegos no núcleo.

- **AUD40 \[SIM\]** v37, oitava rodada (segredos e variáveis de ambiente, entradas, APIs de IA, senhas, rate limit em toda rota, permissões mínimas, plataforma de revisão, manutenção que não piora, cobertura frente a OWASP Top 10:2025, ASVS 5.0.0, SSDF, SLSA, CIS, CISA e NIST 800-63B). Limite de uso obrigatório em toda rota, antes da verificação de assinatura; 5 regras de varredura novas (38 no total), testadas com exemplos; gitleaks testado com segredo plantado; catraca de segurança; unsafe proibido no núcleo; ações do CI fixadas por hash. Registro permanente de todas as solicitações em REGISTRO_E_POLITICA_PERMANENTE_DE_SEGURANCA.md. 419 testes; 216 de 216 defeitos plantados.

- **AUD41 \[SIM\]** v37, nona rodada (sistema que trava, acusa e diz como corrigir; duas pessoas para toda mudança; o que um usuário não autenticado acessa; perguntas de teste; itens em aberto). Módulo sentinela no núcleo e sentinela do banco, com defeitos plantados (227 no núcleo, 14 no banco); proteção do repositório como código com vigia diário testado; certificado de procedência no CI; mapa dos 345 requisitos do ASVS 5.0.0; inventário do acesso sem autenticação gerado do código; 91 perguntas de teste. Conferidos na fonte: mínimo OWASP do Argon2id (19 MiB, 2, 1), o comando gitleaks git e \[extend\] useDefault da versão 8.30.1. 423 testes; 41 módulos.

- **AUD42 \[SIM\]** v37, décima rodada (sem checklist completo, nada roda; impossível uma pessoa ou IA sozinha deixar brecha; vigia que não pode ser ignorado). Selo de liberação com 16 portões e duas chaves diferentes, conferido pela sentinela na partida; portões de governança conferidos na API do GitHub; CI que só emite selo com tudo aprovado e com aprovação humana; ferramenta de linha de comando do selo testada de ponta a ponta; guia de liberação. Limite declarado: conluio de todas as pessoas com chave e aprovação. 428 testes; 42 módulos; 238 de 238 defeitos plantados.

- **AUD43 \[SIM\]** v37, décima primeira rodada (camadas por mudança, ninguém sozinho deixa brecha, administração do GitHub, atualizações, Termos de Uso, quem consegue burlar). Selo passa a exigir três chaves (uma de fora da empresa); portão de administração dupla do GitHub; atualizações com espera e revisão; sentinela diária; Termos de Uso refeitos com notas jurídicas, base legal conferida (CDC art. 51, I; LGPD arts. 42 e 43; Marco Civil art. 15); tabela de quantas pessoas precisariam combinar para cada objetivo. 428 testes; 238 de 238.

- **AUD44 \[SIM\]** v37, revisão final (metadados e verificação completa de tudo de uma vez). Módulo metadados: caixa que muda todo dia, índice cego no disco com chave destruída em 48 h, registros legais selados para chave fora do servidor; o que não dá para zerar (o instante da entrega) declarado com os próximos passos e custos. Mutação completa de todos os defeitos plantados rodada de uma vez, além de verificação, banco, governança, catraca, varredura, segredos e consistência. 431 testes; 43 módulos.

- **AUD45 \[SIM\]** v37, revisão de fechamento (testes clássicos de API, diagnóstico do aparelho, rotação de todas as chaves, apagamento criptográfico, PRD, TRD, fluxo, UI/UX, esquema do back-end e plano de implementação). Três módulos novos (ambiente, chaves_do_servidor e a segunda camada de disco em metadados) e respostas públicas, origem e cabeçalhos em autenticacao; a rota de abrir link do Envio, que faltava no inventário de rotas, entrou com análise e limite. 438 testes; 45 módulos; 252 defeitos plantados.

- **AUD46 \[SIM\]** v37, ataque vermelho contra o próprio Nimbus: 7 ataques executados; 4 brechas reais corrigidas (nomes com letras de imitação, catraca enganada por comentário, estouro aritmético na faxina de chaves, instrução de SQL montado com texto) e 1 limite de desenho declarado (acesso de administrador à máquina de produção), com a tabela de "quem consegue" corrigida. 448 testes; 255 defeitos plantados.

Nota acrescentada na Rodada 26 (12/10/2026), sem renumerar o log acima: este log de auditorias (AUD1–AUD46) registra o trabalho até a entrada anterior (v37, ataque vermelho). O desenvolvimento continuou depois disso sob um sistema de rodadas diferente, registrado em PENDENCIAS_E_DECISOES.md (fora deste manual) — inclui, entre outros, o módulo de cobrança (RFC 9474), duas cadências de plano e dois bugs reais corrigidos (janela de renovação anual; cross-check de plano na renovação), e uma campanha de mutação isolada em envelope.rs/autenticacao.rs. Números correntes nesta rodada, medidos ao vivo (não herdados): 725 testes; 58 módulos; mínimo de 353 defeitos plantados (ratchet confirmado; não remedido nesta rodada). Este manual não foi estendido linha a linha com o mesmo nível de detalhe de cada AUD individual — isso exigiria reabrir cada rodada posterior com o mesmo rigor de auditoria desta série, trabalho explicitamente não executado agora, por proporcionalidade, e não escondido.

## A0I. Sugestões para as próximas versões (propostas, não construídas)

- **SUG1 \[NÃO\]** Marca d’água invisível — DESCARTADA: identificaria quem recebeu, contra o princípio de anonimato.

- **SUG2 \[SIM\]** Liberação com prazo — CONSTRUÍDA na v26 (LIB12, LIB13).

- **SUG3 \[SIM\]** Limite de encaminhamentos — CONSTRUÍDO na v26 como um salto só (LIB15).

- **SUG4 \[NÃO\]** Testes automáticos em aparelhos reais (fazenda de dispositivos) para print, gravação, notificações e registro de chamadas a cada versão.

- **SUG5 \[NÃO\]** Verificação formal do protocolo de liberação e do plano corporativo, além dos protocolos de mensagem.

- **SUG6 \[NÃO\]** Prioridade alta — decodificar fotos, vídeos e PDFs recebidos num processo isolado, com decodificadores em Rust, antes de mostrar. É a defesa contra ataques “sem clique”, que exploram falhas na leitura de imagens.

- **SUG7 \[NÃO\]** Conversa com data de fim: a conversa inteira some numa data escolhida pelos dois.

- **SUG8 \[NÃO\]** Relatório de transparência semestral (pedidos de autoridades recebidos e o que foi possível entregar: nada de conteúdo). Validar com jurídico.

- **SUG9 \[NÃO\]** Modo viagem: tirar do aparelho as conversas escolhidas antes de cruzar uma fronteira e trazê-las de volta depois, pelo cofre.

## A0J. Pagamentos e compra: revisão

Detalhes para o time em docs/PAGAMENTOS.md do kit. A ficha cega (padrão Privacy Pass) garante que nem a NimbusOne consegue ligar um pagamento à conta que o usa.

| **Caminho**                              | **Quem vê o pagamento**                      | **O que a NimbusOne sabe**                                                    |
|------------------------------------------|----------------------------------------------|-------------------------------------------------------------------------------|
| **Assinatura na loja do celular**        | A loja sabe que aquela conta de loja comprou | O recibo é trocado por ficha cega: o servidor não liga a ficha à conta Nimbus |
| **Site da NimbusOne, com ficha anônima** | O processador de pagamento                   | Que houve uma compra; nunca qual conta usa a ficha                            |
| **Fatura corporativa**                   | O financeiro das duas empresas               | Quantas licenças; nunca o conteúdo                                            |

- **PAG1 \[SIM\]** iPhone, loja dos EUA: desde maio de 2025 (regra 3.1.1(a)), o app pode ter link para comprar no site sem autorização especial; a comissão sobre essas compras segue em disputa judicial (Epic × Apple). Conferido em setembro de 2026.

- **PAG2 \[SIM\]** iPhone, demais países: funções digitais pagas pela compra dentro do app, salvo autorização regional da Apple; o recibo vira ficha cega.

- **PAG3 \[PARCIAL\]** Resgatar no app uma ficha comprada no site pode ser lido pela Apple como “chave de licença” (vedada pela regra 3.1.1) fora dos EUA — risco de revisão a validar antes de publicar.

- **PAG4 \[PARCIAL\]** Android: faturamento do Google Play para bens digitais, com programas alternativos em alguns países — validar por país.

- **PAG5 \[SIM\]** Descritor neutro nas faturas; reembolso sem ligação com a conta; preço por país e impostos a definir com financeiro e contador.

## A0K. Um app ou dois? Decisão

Decisão: dois apps, NimbusChat e NimbusOne, com uma conta só. Motivos: (1) cada app fica numa área isolada do sistema — se um for comprometido, o outro não é; (2) cada app tem menos código, logo menos pontos de ataque; (3) o celular virtual tem regras próprias nas lojas (regra 4.2.7 da App Store), e separá-lo protege a publicação do mensageiro; (4) quem só quer conversar instala só o NimbusChat.

- **DUO1 \[SIM\]** Passar uma foto de um app para o outro é possível e seguro: pelo cofre cifrado da conta, nunca pela folha de compartilhar do sistema. Vale a regra de liberação: o que você mesmo criou ou enviou passa livremente; o que recebeu só passa se o autor liberou pelo menos “guardar no cofre”.

- **DUO2 \[SIM\]** Um app abre o outro (ícone Conversas no NimbusOne; Cofre no NimbusChat), sem nenhum conteúdo passar pelo sistema.

## A0L. Kits para o Claude Code

Dois kits, um por produto (seção A0U): instruções que o Claude Code lê ao abrir o projeto, permissões que o impedem de ler segredos, baixar da internet por conta própria e publicar código, as telas do produto com o que implementar, as tarefas em ordem com critério de aceite, pagamentos, documentos jurídicos, prompts por fase e o núcleo completo. Cada kit se verifica inteiro com um comando: ./verificar.sh (verificação completa em falha fechada, cerca de 25 minutos numa máquina de 8 núcleos) e ./verificar.sh --rapido (só as etapas rápidas; termina em PARCIAL e não vale como verificação). A versão do Rust é baixada sozinha pelo arquivo rust-toolchain.toml.

- **KIT1 \[PARCIAL\]** Expectativa honesta: o kit faz o Claude Code construir os apps com as regras certas e verificação a cada passo, mas integrar libsignal, chamadas, Tor, máquinas confidenciais, publicar nas lojas e passar pela auditoria exigem o time e decisões humanas. Não existe “rodar uma vez e sair com o app pronto” para um produto deste nível.

## A0M. No momento do envio: tudo escolhido na hora, por mensagem

- **OPE1 \[SIM\]** Visualização única para qualquer mensagem: abre uma vez e some; não reabre, não se cita, não se guarda.

- **OPE2 \[SIM\]** Temporizador por mensagem, contado da abertura: 1, 2, 3, 4, 5, 10, 15 e 30 minutos; 1, 2, 4, 8, 12, 24 e 48 horas (expiracao::TEMPORIZADORES).

- **OPE3 \[SIM\]** Se não for lida: some em 1, 6, 24 ou 48 horas. Nenhuma mensagem vive mais de 48 horas, lida ou não — teto assinado em cada mensagem (expiracao::VIDA_MAXIMA); nenhuma regra o alonga.

- **OPE4 \[SIM\]** Mensagem de voz com modulador (módulo voz), escolhido na hora do envio; cartão encaminhável para texto ou localização (LIB14).

## A0N. Como você aparece para cada contato

- **APR1 \[SIM\]** Você escolhe, contato por contato, o nome com que aparece. Sugerido: vem preenchido e a pessoa pode trocar. Travado: a pessoa não consegue trocar o seu nome na agenda dela — o seu nome real nunca aparece ali (módulo apresentacao).

- **APR2 \[SIM\]** O cartão é assinado por você e chega cifrado; ninguém altera o nome no caminho. Quando você muda, a versão nova substitui a antiga no aparelho de todos, sem aviso na conversa; uma versão antiga reenviada não desfaz a mudança.

- **APR3 \[SIM\]** Personas: identidades separadas (ex.: Pessoal, Trabalho), cada uma com chaves próprias. Quem conversa com uma não consegue ligá-la à outra.

- **APR4 \[PARCIAL\]** Limite: ninguém impede que a pessoa anote o seu nome real fora do Nimbus.

## A0O. Convite sem rastro e palavras de confirmação

- **CNV1 \[SIM\]** O jeito mais seguro de adicionar alguém: QR pessoalmente. Por outro aplicativo, só o código de uso único, que vence em minutos, queima ao ser usado e vira um endereço exclusivo daquela relação: o código que ficou no outro app não leva a você.

- **CNV2 \[SIM\]** Palavras de confirmação: depois do convite, os dois leem seis palavras em voz alta (chamada Nimbus ou pessoalmente). Iguais: ninguém se pôs no meio. Diferentes: não conversar e refazer pessoalmente (convites::palavras_de_confirmacao; 6 de 256 palavras; para enganar, seria preciso gerar em massa chaves pós-quânticas completas até acertar as seis).

## A0P. Arquivos: inspeção, trazer do celular e envio direto

- **INS1 \[SIM\]** Todo arquivo — recebido, importado ou enviado — é conferido antes de abrir: tipo real pelos primeiros bytes, nunca pelo nome; programas, instaladores, scripts e documentos com macro são bloqueados; extensão mentirosa é bloqueada; PDF e Office só aparecem como imagem, então o código deles nunca roda (módulo inspecao).

- **INS2 \[SIM\]** Não é antivírus de assinaturas, que só reconhece ataques já catalogados: é mais forte contra ataques novos. Arquivo corrompido é recusado pela própria cifra, que não abre nada com um byte alterado.

- **INS3 \[NÃO\]** A construir, prioridade alta: decodificar fotos, vídeos e PDFs num processo isolado, com decodificadores em Rust (SUG6).

- **IMP1 \[SIM\]** Trazer do celular para o NimbusOne: contatos, fotos, vídeos e arquivos, sempre pelos seletores do sistema, com acesso só ao que a pessoa escolher. Cada item é inspecionado, limpo de metadados e cifrado no cofre; a cópia temporária do sistema é apagada na hora.

- **IMP2 \[PARCIAL\]** Apagar do celular depois de trazer é opcional e o sistema pede confirmação. Cópias em backup do celular (iCloud, Google) não são apagadas pelo Nimbus.

- **DIR1 \[SIM\]** Envio direto de Nimbus para Nimbus, sem conversa e sem marca: arquivo inspecionado, sem metadados, cifrado para quem recebe e entregue uma vez só pelos retransmissores — nunca direto entre os aparelhos, para nenhum dos dois ver o IP do outro. Até 48 horas no servidor (módulo direto).

## A0Q. Link para quem não tem o Nimbus: prazo, download e o que fica no navegador

- **LNK1 \[SIM\]** Validade escolhida por quem envia: 5 min, 30 min, 1 h, 6 h, 24 h ou 48 h (máximo). Download só se quem enviou permitir; sem isso, a pessoa só vê. Opção de abrir uma vez só e de senha.

- **LNK2 \[SIM\]** No navegador: nada vai para o cache (no-store, que também impede a cópia de voltar-página); a chave sai da barra de endereços na hora; o conteúdo só aparece depois do toque; ao sair ou trocar de app, a página apaga o conteúdo da tela e zera os dados decifrados; ao encerrar, o servidor pede ao navegador que apague cache e armazenamento do site (Clear-Site-Data).

- **LNK3 \[PARCIAL\]** O seu técnico tem razão em parte: um navegador comum não garante apagar tudo da memória do aparelho, e o endereço do link fica na mensagem em que foi recebido. Por isso a proteção decisiva é o link queimar depois de aberto e vencer em até 48 horas: o endereço que sobra não abre mais nada. Print em navegador não pode ser bloqueado; para isso, Sala Blindada no nível máximo, que exige o app.

## A0R. IP nunca exposto — e o Marco Civil da Internet

- **IPN1 \[SIM\]** Padrão, sem opção de desligar: mensagens passam por dois retransmissores independentes ou pelo Tor; chamadas, por retransmissor. Nenhuma rota do código conecta direto (módulo rede). Nem quem conversa com você nem o servidor do Operador veem o IP do seu aparelho.

- **MCI1 \[SIM\]** Lei nº 12.965/2014 (Marco Civil), art. 15 — conferido no texto compilado do Planalto em setembro de 2026, sem alteração no dispositivo: o provedor de aplicações constituído como pessoa jurídica, com fins econômicos, deve guardar os registros de acesso a aplicações (data, hora e endereço IP, art. 5º, VIII) por 6 meses, sob sigilo; entrega só por ordem judicial (art. 15, § 3º; art. 10, § 1º). Vale também para empresa sediada no exterior que atende o público brasileiro (art. 11, § 2º — a MP 1.068/2021, que o revogava, foi rejeitada).

- **MCI2 \[PARCIAL\]** Como cumprir sem quebrar a promessa: o servidor guarda, cifrados e por 6 meses, data, hora e o IP que chega a ele — que, pela arquitetura, é o do retransmissor, não o do aparelho. A suficiência disso perante a lei é questão jurídica a validar. Alternativa a estudar com advogado: o art. 15 alcança só quem tem fins econômicos; um operador sem fins lucrativos da rede (como faz o Signal nos EUA) mudaria o enquadramento — desde que seja uma estrutura real, não de fachada.

## A0S. IA dentro do aparelho: decisão

Decisão: não incluir agora. Uma IA no aparelho cria riscos que contrariam o produto: (1) memória do que ela aprende vira um novo acervo de dados no aparelho, que pode ser extraído numa perícia; (2) mensagens recebidas podem conter instruções escondidas para a IA (injeção de instruções); (3) usar os modelos do próprio sistema significa código de terceiros lendo o conteúdo. Como a regra é “se trouxer qualquer risco, não pode ter”, a IA fica fora. Só seria reavaliada numa forma sem memória, sem acesso à rede e sem poder agir, depois de auditoria.

## A0T. Um programador com o Claude Code: é possível?

Com honestidade: um programador experiente, com o Claude Code e os kits, consegue construir o NimbusChat funcional (Android e iOS) e o servidor, porque o núcleo de segurança — a parte mais difícil e arriscada — já está pronto e testado. O projeto inteiro (os dois apps, chamadas, Tor, máquinas confidenciais, publicação nas lojas) exige, além dele: auditoria externa, parecer jurídico, fornecedor de infraestrutura e tempo. A ordem recomendada é NimbusChat primeiro, NimbusOne depois.

## A0U. Separação NimbusChat × NimbusOne

Dois kits separados, um por produto: NimbusChat_Kit_ClaudeCode.zip e NimbusOne_Kit_ClaudeCode.zip. Cada um tem as próprias instruções para o Claude Code, as próprias telas, as próprias tarefas e o próprio servidor. O núcleo de segurança é o mesmo nos dois, na mesma versão, como biblioteca compartilhada — nunca copiado e alterado de um lado só. Dois manuais por produto acompanham este manual completo.

## A0V. Termos de Uso e Política de Privacidade

Minutas completas em Termos_de_Uso.md e Politica_de_Privacidade.md: maiores de 18 anos; o que nunca coletamos; o que guardamos por obrigação legal (art. 15 do Marco Civil); entrega de dados só por ordem judicial (art. 10); responsabilidade por conteúdo de terceiros nos termos do art. 19; foro brasileiro (art. 8º, parágrafo único, II). Exigem revisão de advogado e a definição da pessoa jurídica operadora e do encarregado antes de publicar.

## A0W. Usabilidade: decisões tomadas

Princípio: seguro por padrão, sem configurar nada. Os ajustes servem para escolher, nunca para ficar seguro.

- **UX1 \[SIM\]** Três camadas de envio, nesta ordem: (1) as regras da conversa, combinadas entre os dois — o teto; (2) o seu padrão por tipo de conteúdo — Meu jeito de enviar; (3) a escolha desta mensagem, antes de enviar. A mensagem sai com o mais restritivo entre a conversa e a escolha (módulo modos).

- **UX2 \[SIM\]** Padrão de fábrica: fotos e vídeos em visualização única (o que mais vaza por print), mensagem de voz discreta (a voz identifica a pessoa), textos e arquivos normais (48 horas).

- **UX3 \[SIM\]** Três modos com nomes simples — Normal, Discreto, Visualização única — e, se quiser, o tempo exato de 1 minuto a 48 horas. O campo de digitação mostra o modo desta mensagem; o ícone ⏱ ao lado abre as opções.

- **UX4 \[SIM\]** Desfazer envio: depois de tocar em Enviar, 5 segundos para desfazer. A mensagem só é cifrada e enviada depois disso; nada sai antes.

- **UX5 \[SIM\]** Adicionar contato: o caminho principal é o QR pessoalmente; o código de uso único por outro aplicativo é a alternativa, e só nesse caso aparecem as palavras de confirmação.

- **UX6 \[SIM\]** Começo em três passos: criar a conta sem nenhum dado, guardar a frase (confirmando 3 palavras, não todas) e criar o PIN. PIN de coação e amigos de confiança são oferecidos depois, sem travar o começo.

- **UX7 \[SIM\]** Um único sinal de segurança, verde, amarelo ou vermelho, com a explicação ao tocar. Amarelo só aparece quando há algo que a pessoa possa fazer.

- **UX8 \[SIM\]** Mensagens de erro humanas, sempre com o próximo passo: o que aconteceu, se algo foi exposto e o que fazer.

- **UX9 \[SIM\]** Acessibilidade: leitores de tela oficiais funcionam; no Android 14 ou mais novo, todo conteúdo é marcado como sensível, e serviços de terceiros não o leem. Assim, quem usa gerenciador de senhas continua usando o app.

- **UX10 \[SIM\]** Trava ao sair do app e depois de 1 minuto parado; desbloqueio por biometria, com o PIN como alternativa.

- **UX11 \[SIM\]** Busca só na memória, com o app aberto; nada de índice gravado. Notificação sempre genérica; o contador por conversa fica só dentro do app.

- **UX12 \[SIM\]** Metas de desempenho que o time deve medir: abrir o app em até 1 segundo; mensagem aparece como enviada em até 300 ms; chamada conecta em até 3 segundos.

- **UX13 \[SIM\]** Vocabulário único em todas as telas: “some em”, “liberar”, “visualização única”, “amigos de confiança”. Dicas de primeiro uso aparecem uma vez só.

## A0X. Aparelho corrompido: a proteção mais forte possível

- **INT1 \[SIM\]** Atestação de hardware a cada abertura, sempre com desafio de uso único do servidor (60 s): uma atestação antiga de um aparelho limpo não serve para um aparelho comprometido (integridade::Desafios).

- **INT2 \[SIM\]** Vermelho — conversas fechadas e sem credencial no servidor (integridade::sessao_permitida): app adulterado ou não atestado, root ou jailbreak, sistema modificado ou bootloader destravado, atestação só em software, certificado de atestação revogado, aparelho sem senha de bloqueio, depurador, ferramentas de interceptação (Frida e semelhantes), emulador, sobreposição de tela, perfil desconhecido, acessibilidade de terceiros sem contenção do sistema.

- **INT3 \[SIM\]** Amarelo — funciona com restrições: tela sendo gravada ou espelhada, depuração USB, mais de 12 meses sem atualização de segurança, instalação fora da loja, acessibilidade de terceiros contida pelo sistema.

- **INT4 \[SIM\]** Sistema alternativo com bootloader travado e chave de verificação na lista permitida (ex.: GrapheneOS) é verde: é mais seguro que muitos sistemas originais.

- **INT5 \[SIM\]** Reavaliação ao voltar para a tela e a cada 10 minutos. Quem conversa com um aparelho vermelho vê o alerta do Escudo; em Conversa Blindada, o conteúdo some da tela dele.

- **INT6 \[PARCIAL\]** Limite honesto: nenhum método detecta 100% dos aparelhos comprometidos. Existem ferramentas que escondem root e chaves de atestação vazadas. Por isso a decisão se apoia na atestação que vem do chip do aparelho, com lista de revogação, e não em verificações feitas pelo próprio app.

- **INT7 \[NÃO\]** A fazer antes de lançar: bateria de testes em aparelhos reais — root escondido (Magisk com DenyList e módulos de ocultação), Frida, emuladores, chaves de atestação vazadas, jailbreak — a cada versão (prova C76).

## A0X2. Como o programador verifica tudo — em um comando

Cada kit traz o comando ./verificar.sh. Ele confere, sozinho, tudo o que já foi feito, imprime o resultado de cada etapa e reprova se qualquer etapa falhar ou deixar de rodar.

- **VRF1 \[SIM\]** ./verificar.sh roda, em falha fechada (ferramenta ausente, etapa pulada ou etapa que não rodou reprova): os 725 testes do núcleo; a catraca e os defeitos plantados contra a própria catraca, contra o auditor de dependências e contra o próprio verificar.sh; o exemplo de ponta a ponta (34 passos reais); o fuzzing dos leitores (50 mil entradas malformadas por alvo, prova C79); a varredura de segurança (semgrep, 47 regras); a auditoria de todas as versões do Cargo.lock contra a base RustSec do dia; e, no kit, a integridade e a consistência dos documentos com o protótipo e o núcleo. ./verificar.sh --rapido roda só essas etapas e termina com PARCIAL (código 3): nunca vale como verificação.

- **VRF2 \[SIM\]** O mesmo ./verificar.sh (igual a ./verificar.sh --completo) roda ainda: a prova de tempo constante (C80), o tempo de recusa, o fuzzing profundo (2 milhões de entradas por alvo) e a mutação completa (353 defeitos plantados; todos precisam ser pegos, e defeito plantado que não compila conta como falha). "Tudo certo" só aparece quando todas as etapas rodaram e passaram.

- **VRF3 \[SIM\]** Auditoria de dependências sem depender de rede: verificacao/auditar_dependencias.py compara cada versão do Cargo.lock — inclusive as duplicadas da mesma biblioteca — com a base de avisos RustSec, lê o campo withdrawn de verdade, exige base com no máximo 7 dias e reprova se alguma dependência estiver vulnerável (código 1) ou se a auditoria não rodar (código 2). Hoje: 84 versões de 71 bibliotecas, nenhuma vulnerabilidade em aberto. Os defeitos plantados contra o próprio auditor ficam em verificacao/testar_auditoria.py.

- **VRF4 \[SIM\]** Autoverificação da criptografia (tests/criptografia.rs, prova C1): trocar um bit do envelope impede a leitura; trocar o contexto invalida; a chave de outra pessoa não abre; duas mensagens iguais geram cifras diferentes; a assinatura híbrida exige as duas partes.

- **VRF5 \[SIM\]** Testes de aparelho real prontos (tests/aparelho_real.rs, provas C76/C77): ignorados no dia a dia, rodam com um comando no aparelho preparado; passo a passo em docs/roteiro-aparelho-real.md. O desafio de atestação contra reenvio (tests/servidor_atestacao.rs) roda sozinho.

- **VRF8 \[SIM\]** Fuzzing dos leitores (tests/fuzzing.rs, prova C79): entradas aleatórias e malformadas em 18 pontos que recebem bytes de fora — envelope, chaves, assinatura, cofre, contatos, pacote, custódia, recuperação, inspeção de arquivos, limpeza de imagens, Nimbus Envio, envio direto, registro público e notificação. Roda no perfil verificacao (otimizado, com verificação de estouro aritmético e asserções ligadas): 50 mil entradas por alvo e depois 2 milhões no fuzzing profundo, ambos dentro do ./verificar.sh. Resultado nesta versão: nenhum pânico.

- **VRF9 \[SIM\]** Tempo constante (tests/timing.rs, prova C80): PIN real, PIN de coação e PIN errado — no primeiro e no último dígito — levam o mesmo tempo para avaliar. Quem mede o tempo não descobre qual foi digitado, nem se existe PIN de coação. Nesta versão, a maior diferença entre os quatro casos foi de 5% (limite: 25%).

- **VRF10 \[SIM\]** Conferidores conferidos: verificacao/testar_catraca.py (29 afrouxamentos plantados, inclusive cadeia de suprimentos, inclusive texto invisível que inverte a leitura e gatilhos perigosos no CI, inclusive as quatro formas de liberar unsafe que a catraca deixava passar e a retirada das regras de branch e de tags), verificacao/testar_auditoria.py (7 casos), verificacao/testar_verificar.py (9 situações, inclusive código alterado no meio da verificação, inclusive pasta de compilação compartilhada com outra cópia), portoes_de_governanca.py --teste (20 casos, inclusive regra apontando para outra branch e aprovação de robô) e testar_conferidores.py (22 ataques aos conferidores de números, pacote, consistência, à catraca dentro do kit e ao medidor de provas, que recusa registro sem a identidade do código medido) rodam dentro do ./verificar.sh e no CI; a entrega só é declarada pelo entregar.py (16 ataques ao próprio portão; a declaração só roda no CI, com 4+ núcleos, 8+ GB de memória e 20+ GB livres, e o ./verificar.sh completo dentro de cada kit extraído), com o pacote final extraído e a identidade do código igual à das provas; assim como mutantes.py --conferir (todo defeito plantado aponta para um trecho que existe exatamente uma vez no código); cada número de prova é registrado com data e comando em PROVAS_DA_RODADA.md; a catraca reprova se algum deles sumir ou deixar de ser chamado.

- **VRF6 \[SIM\]** A versão do Rust (1.91.1) fica fixada no rust-toolchain.toml, a mesma da integração contínua; com rustup instalado, o verificar.sh a instala sozinho.

- **VRF7 \[PARCIAL\]** O que o comando NÃO substitui: teste com usuários reais (usabilidade) e teste de invasão independente sobre o app compilado e a infraestrutura. O código passar prova que cada promessa tem uma verificação automática — não que o produto pronto resiste a um atacante humano.

## A0X3. Automação: a entrega se verifica sozinha

Duas camadas de automação, para o técnico ter o menor trabalho possível e nenhuma conferência manual.

- **AT1 \[SIM\]** Dentro de cada kit: ./verificar.sh roda tudo do núcleo (725 testes, catraca, exemplo, fuzzing, varredura, auditoria de dependências, consistência, mutação, tempo constante e fuzzing profundo) em falha fechada: reprova se algum teste sumir, se uma dependência ficar vulnerável, se uma ferramenta faltar ou se qualquer etapa deixar de rodar.

- **AT2 \[SIM\]** No repositório do projeto: verificar_pacote.py compara byte a byte o núcleo, o manual e o protótipo entre o pacote e os dois kits, confere a consistência de cada kit e barra artefatos pesados e termos de versão antiga. empacotar_e_verificar.sh gera os ZIPs e roda essa verificação num comando só. E verificar_numeros.py confere os números e fatos citados em todos os materiais — documentos de entrega, kits, manual com as tabelas, decks, leia-mes e protótipo — contra a fonte única de verdade (NUMEROS.json), apurada do próprio código.

- **AT3 \[SIM\]** Na nuvem (GitHub Actions, .github/workflows/nimbus-ci.yml): a cada envio, roda os testes, o fuzzing (200 mil por alvo no teste rápido e 3 milhões no profundo), a prova de tempo constante, a mutação (353 defeitos), a varredura semgrep, a auditoria cargo-audit e cargo-deny, e gera a lista de materiais (SBOM). Nada é publicado sem passar.

- **AT4 \[PARCIAL\]** O que a automação NÃO cobre, e continua sendo trabalho humano: o teste em aparelho real (C76, precisa dos aparelhos), o teste de invasão independente e o teste com usuários reais.

## A0X4. Ataques e contra-ataques: cada defesa com a sua prova

Como um atacante profissional pensaria, e o que responde a cada ataque. A coluna da direita diz como a defesa é provada. Onde nenhum teste automático é possível, isso está dito.

| **Ataque**                                                                               | **Defesa no Nimbus**                                                                                           | **Como se prova**                                                                                              |
|------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|
| **Tomar ou invadir o servidor**                                                          | Só há blocos cifrados; as chaves ficam só nos aparelhos                                                        | Autoverificação da criptografia (C1): chave errada não abre, bit trocado não abre                              |
| **Adulterar ou trocar mensagem no caminho**                                              | Cifra autenticada e assinatura híbrida (clássica e pós-quântica)                                               | C1: bit trocado, contexto trocado e assinatura trocada são recusados                                           |
| **Repetir, reordenar ou replantar uma mensagem legítima**                                | Identidade da conversa e número de sequência presos no dado autenticado; o receptor só aceita o próximo número | C86: adversário ativo (Dolev-Yao) — repetida, fora de ordem, de outra conversa ou adulterada, todas rejeitadas |
| **Descobrir se duas mensagens são iguais**                                               | Cada envelope é diferente, mesmo com o mesmo texto                                                             | C1: mesmo texto gera envelopes diferentes                                                                      |
| **Mandar dado malformado para derrubar ou invadir o app**                                | Todos os leitores de bytes de fora recusam o que não é válido, sem cair                                        | Fuzzing (C79/C83): 45 milhões de entradas numa passada, nenhum pânico; 54 milhões no CI                        |
| **Cronometrar o PIN ou descobrir o PIN de coação**                                       | Argon2id de 64 MiB e comparação em tempo constante                                                             | Tempo constante (C80): os quatro casos levam o mesmo tempo                                                     |
| **Usar celular com root escondido, Frida, emulador, jailbreak ou bootloader destravado** | Atestação de hardware; vermelho não recebe acesso no servidor                                                  | Lógica: testes automáticos; aparelho real: tests/aparelho_real.rs (C76), com roteiro                           |
| **Reaproveitar a atestação de um celular limpo**                                         | Desafio de uso único, válido por 60 segundos                                                                   | tests/servidor_atestacao.rs (C77)                                                                              |
| **Se passar pelo dono na recuperação da conta**                                          | 2 de 3 amigos, cada parte cifrada só para o seu dono; parte adulterada é recusada                              | Testes do módulo recuperacao (C78); o contato ao vivo com os amigos é procedimento, não automatizável          |
| **Fazer uma mensagem durar mais que o combinado**                                        | Toda mensagem vence em até 48 horas; a conversa é o teto                                                       | Testes de propriedades: milhares de combinações sorteadas                                                      |
| **Explorar falha conhecida numa biblioteca**                                             | Auditoria contra a base RustSec a cada verificação                                                             | auditar_dependencias.py: 71 pacotes, nenhuma vulnerabilidade em aberto                                         |
| **Descobrir conteúdo pelos registros do app**                                            | Regra que proíbe gravar conteúdo em log                                                                        | Varredura semgrep (47 regras), zero ocorrências                                                                |
| **Introduzir um defeito sem ninguém perceber**                                           | Todo ponto crítico tem teste que falha se o código mudar                                                       | Mutação: 353 defeitos plantados, todos exigidos pegos                                                          |
| **Ler a tela de um celular espionado durante o uso**                                     | Trava ao sair do app e após 1 minuto parado; Escudo avisa o outro lado                                         | Limite de qualquer sistema: não há teste que elimine esse risco                                                |
| **Fotografar a tela com outro aparelho**                                                 | Visualização única e prazo curto reduzem a exposição                                                           | Limite físico: nenhuma tecnologia impede                                                                       |

## A0X5. Ataques reais aos concorrentes: como o Nimbus fecha cada porta

Estudo dos ataques públicos e documentados a WhatsApp, Signal, Telegram, Confide, Wickr e semelhantes, e onde cada porta está fechada no Nimbus. Onde a defesa depende da implementação nativa (Tarefa T1) ou do servidor, está dito.

| **Ataque documentado (alvo, referência)**                                              | **Como funcionava**                                                                         | **Porta no Nimbus**                                                                                                                                                                                                               |
|----------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Chave falsa / troca de chave sem aviso (Confide, Quarkslab 2017)**                   | O servidor entrega uma chave pública falsa e lê no meio; o app não avisa chave nova         | Palavras de confirmação ao adicionar contato e impressão digital da chave assinada e versionada (convites, apresentacao). Prova C86 impede o meio de campo ativo                                                                  |
| **Membro-fantasma em grupo (WhatsApp/Signal/Threema, Ruhr 2018)**                      | Quem controla o servidor insere um membro em grupo privado sem o admin autorizar            | Ações que valem exigem assinatura de aprovadores autorizados e distintos, k de n (aprovacao); o servidor não forja autorização                                                                                                    |
| **Reordenação e replay de mensagem (Telegram MTProto, Royal Holloway 2021)**           | A rede reordena ou repete mensagens; o cliente aceita                                       | Identidade da conversa e número de sequência presos no dado autenticado; só o próximo número é aceito. Prova C86 (adversário ativo, Dolev-Yao)                                                                                    |
| **Arquivo disfarçado de vídeo (Telegram EvilVideo, ESET 2024)**                        | Um APK é entregue parecendo um vídeo de 30 segundos                                         | Tipo real pelos primeiros bytes, nunca pelo nome; instalador Android, programa, script e macro são bloqueados (inspecao). Prova C40                                                                                               |
| **Link/nome com reordenação de texto (Signal iOS, CVE-2022-28345)**                    | Caractere RTLO inverte o texto e um .exe parece .jpg; um link parece outro site             | Caracteres bidirecionais e invisíveis removidos antes de exibir; o texto real aparece (inspecao::sanear_texto_exibido). Prova C87                                                                                                 |
| **Chamada forçada a atender (Signal Android, CVE-2019-17191)**                         | Uma mensagem de conexão forjada liga o áudio sem o usuário tocar em nada                    | O microfone só liga no estado Ativa, e só a ação local de atender chega a Ativa; nenhuma mensagem da rede pula esse passo (chamada). Prova C88                                                                                    |
| **Anexo em cache lido ou trocado no disco (Signal Desktop, CVE-2023-24068/24069)**     | Anexos ficam em pasta previsível, recuperáveis ou adulteráveis por quem tem acesso ao disco | Conteúdo vive só na memória e é zerado após o uso (memoria, higiene); nada de índice ou cache em claro. Em computador, exigir disco cifrado do sistema                                                                            |
| **Vazamento de memória por endpoint de diagnóstico (TeleMessage, CVE-2025-48927)**     | Um endpoint de diagnóstico do servidor devolvia a memória em claro, com senhas              | O servidor do Nimbus guarda só blocos cifrados e não tem as chaves; não há o que um dump de memória do servidor revele em claro. A garantir na configuração do servidor (Fase 1)                                                  |
| **Travessia de caminho / Trojan por nome de arquivo (Signal Desktop, CVE-2019-19954)** | Um nome de arquivo com ../ escreve fora da pasta e planta um executável                     | Nome de arquivo trocado por um neutro, sem o nome original nem caminho (higiene::nome_neutro)                                                                                                                                     |
| **Downgrade forçado de cifra**                                                         | Forçar o app a usar um algoritmo fraco                                                      | As duas únicas suítes são fortes (XChaCha20-Poly1305 e AES-256-GCM); um identificador desconhecido é recusado (envelope). Cifra pós-quântica ML-KEM-1024 e ML-DSA-87                                                              |
| **Homem no meio por chave degenerada (small subgroup)**                                | Uma chave pública de ordem baixa zera parte do segredo e abre o meio de campo               | Chave X25519 de ordem baixa é recusada na leitura (envelope); o segredo precisa ser contributivo. Provas C1 e a auditoria de dependências                                                                                         |
| **Espião no aparelho (Pegasus e zero-click, modelo)**                                  | Um exploit sem toque compromete o aparelho e lê tudo                                        | Aparelho comprometido fica vermelho e não recebe credencial no servidor; atestação de hardware com desafio de uso único (integridade). Limite honesto: a tela vista durante o uso de um aparelho já dominado não há como proteger |
| **Metadados: quem fala com quem (sealed sender, análise de tráfego)**                  | Mesmo cifrado, o servidor sabe quem conversa com quem e quando                              | Contatos e nomes cifrados em bloco de tamanho fixo; IP nunca exposto (dois retransmissores ou Tor); nada dura mais de 48 horas. Limite: um servidor não substitui uma rede de anonimato dedicada                                  |
| **Derrubar o app com mensagem malformada (WhatsApp, Check Point 2019)**                | Uma mensagem inválida trava o app de todos num laço                                         | Todos os leitores de bytes de fora recusam o inválido sem cair: 45 milhões de entradas malformadas, nenhum pânico (fuzzing). Provas C79 e C83                                                                                     |

Limite honesto e igual para todos: este estudo cobre os ataques PÚBLICOS e conhecidos. Nenhum estudo prova a ausência de falhas novas — por isso o teste de invasão independente e a atualização contínua da lista de vulnerabilidades continuam obrigatórios. Fechar as portas conhecidas é o dever de casa; não é garantia de invulnerabilidade, e ninguém sério promete isso.

## A0Y. O que existe em cada lugar se alguém tomar ou invadir

| **Se alguém tomar ou invadir**                   | **O que encontra**                                                                                                  | **Por quê**                                                                         |
|--------------------------------------------------|---------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| **O servidor do NimbusOne**                      | Blocos cifrados, de tamanhos padronizados, com prazo; chaves públicas; fichas de licença que não se ligam a ninguém | As chaves ficam só nos aparelhos; o servidor não tem como abrir                     |
| **O retransmissor da primeira etapa**            | O IP de quem conecta, sem saber para onde vai nem o conteúdo                                                        | É de outra empresa, em outra jurisdição; a segunda etapa não vê o IP                |
| **O celular bloqueado**                          | Nada legível                                                                                                        | Chaves no chip; 10 PINs errados apagam as chaves; PIN de coação abre um cofre falso |
| **O celular aberto, com o app aberto**           | O que está na tela naquele momento                                                                                  | Limite de qualquer sistema; por isso o app trava ao sair e após 1 minuto parado     |
| **O pagamento**                                  | Que alguém comprou uma licença                                                                                      | A ficha cega não se liga à conta que a usa                                          |
| **Os registros de acesso (se exigidos por lei)** | Data, hora e o IP do retransmissor                                                                                  | O servidor nunca recebe o IP do aparelho                                            |

## A0Z. Amigos de confiança

- **REC1 \[SIM\]** Além da frase de recuperação, a pessoa pode escolher de 3 a 5 contatos Nimbus como amigos de confiança; o segredo de recuperação é dividido entre eles e cada parte vai cifrada só para o seu dono (módulo recuperacao).

- **REC2 \[SIM\]** Nenhum amigo sozinho — nem o NimbusOne — consegue nada; um não sabe quem são os outros; parte adulterada é recusada.

- **REC3 \[PARCIAL\]** Na hora de recuperar, os amigos devem falar com a pessoa ao vivo antes de devolver a parte: é o que impede alguém de se passar por ela. Recomendação na tela: escolher amigos que não se conheçam entre si.

## A1C. Quantos aplicativos: dois — e o destinatário não precisa de nenhum

Decisão final, pensando em segurança e em sucesso comercial: dois aplicativos, nenhum terceiro.

- **APP1 \[SIM\]** NimbusChat: só comunicação — conversas, grupos, chamadas 1:1 e em grupo, mensagens de voz, arquivos na conversa e o Nimbus Envio. É o aplicativo que todo mundo baixa.

- **APP2 \[SIM\]** NimbusOne: o celular blindado — Cofre Pessoal (agenda, notas, álbuns, arquivos, senhas), celular virtual para apps de terceiros, Sala Blindada e aprovações. Plano premium.

- **APP3 \[SIM\]** Quem recebe um documento não precisa instalar nada: abre o link no navegador (Nimbus Envio) ou vê o documento pela Sala Blindada, também no navegador. Cada documento enviado apresenta a marca a quem ainda não é cliente — crescimento sem custo de aquisição.

- **APP4 \[SIM\]** Não criar um terceiro aplicativo de arquivos: mais um download reduz adesão e duplica a superfície de ataque. Tudo cabe nos dois.

## A2. Por que o NimbusChat é um aplicativo separado, e não um app dentro do Nimbus OS

Sua conclusão está certa, e por três motivos que se somam. Esta é a decisão final do projeto.

- **SEP1 \[SIM\]** Segurança. Dentro do Cloud Phone, as mensagens do NimbusChat seriam decifradas no servidor enquanto o cliente conversa, protegidas por hardware. Como app nativo, elas só são decifradas na memória do aparelho do próprio cliente, por segundos. A promessa “ponta a ponta” só é verdadeira na segunda forma: a ponta precisa ser o aparelho da pessoa, não um servidor nosso.

- **SEP2 \[SIM\]** Alcance. O app do NimbusChat entra na App Store e no Google Play sem esbarrar na regra 4.2.7 da Apple, que barra clientes de apps que rodam na nuvem. O Nimbus OS, no iPhone, precisa do navegador. Separados, o chat chega a todo mundo desde o primeiro dia.

- **SEP3 \[SIM\]** Negócio. O NimbusChat pode ser gratuito ou barato, escalar sem custo de servidor por cliente e criar a base de usuários. O Nimbus OS custa uma máquina virtual por cliente e é plano premium. Juntos num app só, o custo do segundo contaminaria o preço do primeiro.

- **SEP4 \[SIM\]** Funcionamento na prática: quem tem os dois abre o NimbusChat direto no celular; dentro do Nimbus OS, o ícone do NimbusChat é um atalho que traz a pessoa de volta ao app nativo. Nunca existem duas cópias do mesmo chat.

## A2B. Por que o NimbusChat roda no aparelho, e não na VPS

Porque seria pior, não melhor. Na VPS, as mensagens do NimbusChat existiriam decifradas no servidor enquanto o cliente usa. No modo efêmero, elas só existem decifradas na memória do aparelho do próprio cliente, por segundos, e somem ao fechar. O objetivo do “tudo virtual” — nada no celular, continuidade ao trocar de aparelho, um só app — é entregue integralmente pelo modo efêmero, sem a perda.

## A2C. Guardar tudo no servidor e abrir só no celular do dono — sim, dá, e melhora a proteção

Sua intuição está certa e agora está no código (módulo historico). O servidor guarda o histórico do NimbusChat como blocos cifrados com identificadores aleatórios: não sabe de quem são, de que conversa são, nem o que contêm, e todos têm o mesmo tamanho aparente. A chave fica só com o dono, protegida pelo chip de segurança do aparelho. O celular baixa o bloco, abre na memória, mostra e apaga — nada escrito no disco. Isso reduz o rastro no aparelho (quase nada mora nele) sem entregar nada ao servidor.

- **SV1 \[SIM\]** Só o dono abre: cada bloco é cifrado para a chave do dono; sem essa chave, o bloco é ruído. Trocar de aparelho recupera o histórico; perder todas as chaves e métodos de recuperação torna o histórico irrecuperável (é a prova de que a empresa não tem a chave).

- **SV2 \[SIM\]** Apagamento criptográfico por janela de tempo (a resposta forte às temporárias): cada mensagem é cifrada com a chave da janela em que expira. Quando a janela vence, o aparelho destrói essa chave. A partir daí, qualquer cópia do bloco — no nosso servidor, num backup de terceiro, num grampo de rede — fica ilegível para sempre. Não dependemos de acreditar que o servidor apagou.

- **SV3 \[SIM\]** O servidor nunca liga um bloco a outro: identificadores aleatórios e tamanho padronizado. Nem a quantidade de mensagens de uma conversa é visível.

- **SV4 \[PARCIAL\]** Limite honesto: enquanto o bloco está aberto na tela, ele existe decifrado na memória do aparelho — é inevitável para poder mostrar. Some ao fechar. Por isso as defesas de aparelho (modo efêmero, anti-captura, verificação de risco) continuam necessárias mesmo com o histórico no servidor.

## A3. Como a empresa fica sem acesso até ao Cloud Phone

- **KS1 \[SIM\]** Chave dividida em duas metades (esquema 2 de 2): uma fica no chip de segurança do aparelho do cliente; a outra fica com um custodiante independente, em outra jurisdição, que só a entrega a uma máquina virtual que prove, por atestação de hardware, que roda exatamente o sistema publicado e auditado.

- **KS2 \[SIM\]** Consequência: a Nimbus sozinha não abre o disco; o custodiante sozinho não abre; um ladrão com o celular sem o desbloqueio não abre; uma máquina adulterada não recebe a chave.

- **KS3 \[SIM\]** A máquina do cliente só existe ligada durante a sessão. Ao sair, a memória é destruída e o disco volta a ser um bloco cifrado sem chave em lugar nenhum da empresa.

- **KS4 \[SIM\]** Quatro partes independentes, nenhuma capaz de violar o cliente sozinha: Nimbus (servidores), operador do relay que oculta o IP, custodiante da meia-chave e auditor. Para violar, as quatro precisariam agir juntas e em segredo — e o registro público de chaves denunciaria.

## A4. O que nenhuma arquitetura remove — a verdade inteira

Estes limites vêm de terceiros (Google, Apple, Meta, bancos) ou da física. Não são falhas do desenho e não somem com engenharia. Prometer o contrário seria a primeira brecha — a de credibilidade.

- **LM1 \[PARCIAL\]** Bancos: os apps bancários exigem atestado do Google de aparelho físico certificado; em ambiente virtual o atestado vem vazio. Rota: programa Nimbus Trusted Banking (dupla atestação + biometria; documento próprio), que depende de o banco adaptar o app. Até lá: bancos no celular físico e Navegador Isolado Nimbus.

- **LM2 \[NÃO\]** iPhone: a regra 4.2.7 da App Store proíbe clientes de apps na nuvem. O app Nimbus OS da App Store traz os apps Nimbus; no iPhone, o Cloud Phone abre pelo navegador (ícone na tela inicial). Android e computador: tudo num app só.

- **LM3 \[PARCIAL\]** WhatsApp/Signal: um único aparelho principal por número; o Cloud Phone vira o principal e o app sai do celular físico. A Meta pode bloquear contas em servidor — piloto obrigatório (C-WA).

- **LM4 \[PARCIAL\]** Sem internet: o NimbusChat não mostra conteúdo (sem cache offline e sem rede mesh na v1 — OFF1 a OFF3); o NimbusOne mostra, do cache opcional do cofre, só itens sem validade (OFF4); o Cloud Phone não funciona offline. Chamada de emergência continua pelo discador do aparelho.

- **LM5 \[NÃO\]** Espionagem instalada no aparelho do cliente vê a tela como o dono vê. Reduzida pelo modo efêmero e pelo Cloud Phone, não eliminada.

- **LM7 \[NÃO\]** Sistema operacional comprometido: o sistema é quem desenha a tela, então tudo o que aparece passa por ele. Se o sistema estiver tomado por espionagem de alto nível, ele vê o que o usuário vê. Nenhum aplicativo impede isso. O que fazemos: detectar sinais de sistema adulterado e bloquear (DV), manter o conteúdo aberto o menor tempo possível (RZ3) e, no Nimbus OS, mandar ao aparelho só a imagem da tela.

- **LM8 \[NÃO\]** O Nimbus OS, sozinho, não resolve os aplicativos de banco. O bloqueio está dentro do app do banco, que exige do Google a confirmação de aparelho físico certificado; num celular virtual essa confirmação vem vazia, e falsificá-la seria fraude. O único caminho é o banco aderir ao Nimbus Trusted Banking e adaptar o app dele.

- **LM6 \[NÃO\]** “Zero brechas” não existe em nenhum sistema do mundo. O que existe é defesa em camadas, verificada por terceiros, com correção rápida. É isso que este blueprint entrega e prova.

## A5. O que eu (Claude) já entreguei e o que posso desenvolver

| **Entrega**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | **Situação**                                                                                                                                                                                                                                                    | **Quem valida**                   |
|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------|
| **nimbus-nucleo (Rust, todo em português): cifra pós-quântica (padrão e FIPS), temporárias, regras por conversa e por mensagem, histórico no servidor com apagamento criptográfico, verificação do aparelho, sessões, Escudo de Conversa, modulador de voz e tráfego de cobertura — 58 módulos; 725 testes rápidos, fuzzing (50 mil entradas por alvo; 2 milhões no modo completo) e prova de tempo constante; 353 defeitos plantados, todos exigidos pegos; auditoria adversarial completa** | PRONTO e testado nesta conversa (725 de 725 aprovados no cargo test; fuzzing de 2 milhões de entradas por alvo e prova de tempo constante aprovados; 353 defeitos plantados, todos exigidos pegos; dependências auditadas; zero avisos; Rust 1.91.1). Protótipo | Time técnico + auditor (C1)       |
| **nimbus-regras.yml: 47 regras de varredura automática que reprovam código proibido**                                                                                                                                                                                                                                                                                                                                                                                                         | PRONTO e testado                                                                                                                                                                                                                                                | Time roda a cada entrega (C7)     |
| **Nimbus Trusted Banking: proposta técnica para bancos**                                                                                                                                                                                                                                                                                                                                                                                                                                      | PRONTO                                                                                                                                                                                                                                                          | Fundador leva ao banco piloto     |
| **Modelo de ameaças, especificação do protocolo de chave dividida, contratos de API do servidor**                                                                                                                                                                                                                                                                                                                                                                                             | Posso escrever na próxima etapa                                                                                                                                                                                                                                 | Time + auditor                    |
| **Relay de mensagens em memória (Rust), serviço de custódia da meia-chave, pipeline de integração com varredura e inventário de componentes**                                                                                                                                                                                                                                                                                                                                                 | Posso escrever protótipos testáveis                                                                                                                                                                                                                             | Time implanta; auditor verifica   |
| **Scripts das provas C2, C8, C14 e C15 (servidor entregue, tráfego, perícia efêmera, chave dividida)**                                                                                                                                                                                                                                                                                                                                                                                        | Posso escrever                                                                                                                                                                                                                                                  | Auditor executa                   |
| **Revisão de cada entrega do time (código enviado em .zip)**                                                                                                                                                                                                                                                                                                                                                                                                                                  | Contínuo                                                                                                                                                                                                                                                        | Primeira camada; não substitui C1 |

*Por que não entrego “o aplicativo inteiro pronto”: um mensageiro deste nível é trabalho de equipe por longo período (o Signal é desenvolvido há mais de uma década). Código de segurança entregue de uma vez, sem revisão e auditoria, é o oposto de seguro — seria a maior brecha do projeto. O caminho seguro é o deste manual: núcleo crítico pronto e testado, especificação completa, regras automáticas e provas; o time constrói por fases e eu reviso cada entrega.*

*Limites do meu papel: não compilo nem testo em iPhone real, não publico em loja, não opero servidores e não assino versões. Todo código meu passa pela mesma revisão em dupla e auditoria que o código humano.*

## A6. Verificação do aparelho ao abrir o app

- **DV1 \[NÃO\]** Passar antivírus da VPS (ou do app) no aparelho do cliente é impossível: o iOS proíbe qualquer app de examinar outros apps e arquivos, e o Android isola cada app. Nenhum app de loja faz isso de verdade — quem promete, exagera.

- **DV2 \[SIM\]** O que é possível, a cada abertura e periodicamente: coletar sinais oficiais e locais — atestação do app (App Attest/Play Integrity), integridade do aparelho, root/jailbreak, depurador, gravação/espelhamento de tela, sobreposição de tela, serviços de acessibilidade suspeitos (vetor do controle remoto), depuração USB, sistema desatualizado. No Android, o Play Integrity oferece ainda sinais opcionais de status do Play Protect e de risco de apps com acesso à tela. No computador e na web não há atestação do fabricante: teto amarelo (PC3).

- **DV3 \[SIM\]** Resposta graduada (código pronto, módulo integridade): VERDE uso normal; AMARELO fecha Álbum Cofre, visualização única e bancos, exige biometria em ações sensíveis; VERMELHO não abre conteúdo — só permite sair, revogar aparelhos e ver orientação. O servidor reconfirma com a atestação: o aparelho sozinho não decide.

- **DV4 \[SIM\]** Privacidade: a lista de apps do cliente nunca sai do aparelho; o servidor recebe só o nível e os motivos.

- **DV5 \[SIM\]** Nimbus Check (opcional, iniciado pelo cliente): verificação profunda contra spyware conhecido com o MVT, ferramenta aberta da Anistia Internacional, rodando no computador do próprio cliente. Resultado fica com ele.

- **DV6 \[NÃO\]** Espionagem de nível estatal pode não deixar sinal detectável. Mitigação: modo efêmero (pouco a roubar), Cloud Phone e Modo de Isolamento da Apple.

## A6B. Gravação de chamadas e modulador de voz

- **GR1 \[SIM\]** A gravação de chamadas embutida no iPhone atua no app Telefone e em chamadas FaceTime; chamadas de apps próprios como o NimbusChat não entram nesse recurso (validar a cada versão do iOS). O NimbusChat usa áudio próprio, portanto fora desse caminho.

- **GR2 \[SIM\]** Contra gravação de tela durante a chamada: o Escudo de Conversa (A7) pausa o áudio, borra a imagem e avisa todos os participantes.

- **GR3 \[SIM\]** Android: o áudio do NimbusChat é marcado como não capturável por outros apps; gravadores de tela e de chamada recebem silêncio.

- **GR4 \[SIM\]** Sem histórico no sistema: a chamada não usa o CallKit e por isso não entra em Recentes do iPhone (requisito técnico VC1).

- **VM1 \[SIM\]** Modulador de voz Nível 1 (código pronto, módulo voz): desloca o tom da voz em tempo real, no aparelho, ANTES de cifrar — a voz original nunca sai do celular. Sorteia um desvio de tom a cada chamada, para o valor exato não ser conhecido e revertido.

- **VM2 \[SIM\]** Perfis: Desligado, Mais grave, Mais agudo e Neutro; mantém a duração e a inteligibilidade da fala.

- **VM3 \[PARCIAL\]** Verdade sobre o Nível 1: deslocar o tom é em parte reversível por perícia (desloca-se de volta) e o deslocamento de formante ainda está pendente (o código declara o parâmetro). Serve contra reconhecimento casual e contra quem apenas gravou a chamada, não contra laboratório forense.

- **VM5 \[SIM\]** Modulador de voz Nível 2 (orquestração pronta, módulo voz): conversão neural para uma voz-alvo sintética fixa, escolhida pelo usuário. Não é reversível ao timbre original, ao contrário do Nível 1. Roda no aparelho: a voz real nunca sai dele. A inferência da rede é nativa (Core ML no iPhone, LiteRT/NNAPI no Android); o núcleo cuida da divisão em blocos, do orçamento de latência e de zerar a voz real da memória.

- **VM5b \[PARCIAL\]** Verdade sobre o Nível 2: esconde o timbre, não o conteúdo. Sotaque, escolha de palavras e ritmo de fala continuam identificando a pessoa. Medir a latência por aparelho antes de habilitar em chamada; se passar do orçamento, oferecer só em mensagem de voz gravada.

- **VM4 \[SIM\]** Meta de latência: menos de 20 ms por bloco no aparelho; medir antes de liberar em chamadas ao vivo.

- **VM7 \[PARCIAL\]** Aviso obrigatório na interface: o modulador descaracteriza a voz, mas não é anonimato provado — quem conhece a pessoa pode reconhecer o ritmo da fala, e análise forense pode tentar reverter. Nunca anunciar como “anonimato de voz garantido”.

- **ID0 \[SIM\]** Sem número de telefone, sem e-mail, sem documento e sem acesso à agenda do sistema: contato só por convite, QR ou Nimbus Ceremony.

## A6C. Decisão: sem número de telefone comum (VoIP para a rede telefônica)

Revisão final: o número VoIP para ligar e receber de telefones comuns sai do produto. Ele seria a maior brecha de toda a arquitetura, e nenhum ajuste técnico a fecha. O NimbusChat fica com chamadas Nimbus para Nimbus, que são cifradas de ponta a ponta e anônimas.

- **TF1 \[SIM\]** Decisão: nenhuma ponte com a rede telefônica comum, em nenhum dos dois produtos. Chamadas só entre usuários Nimbus.

- **TF2 \[NÃO\]** Por que sai (1) — cifra: no trecho da rede telefônica, a voz trafega sem cifra ponta a ponta; a operadora e quem tiver ordem judicial sobre ela podem ouvir.

- **TF3 \[NÃO\]** Por que sai (2) — metadados: a operadora registra quem liga para quem, quando e por quanto tempo. Quem liga para os números reais da família e do trabalho revela a própria identidade só pelo padrão das ligações, mesmo sem nome na conta.

- **TF4 \[NÃO\]** Por que sai (3) — lei: na maior parte dos países, número de telefone exige cadastro do titular; e oferecer interligação com a rede telefônica transforma a empresa em prestadora regulada, com obrigações de interceptação legal. Isso contraria o princípio “nada a entregar”.

- **TF5 \[NÃO\]** Por que sai (4) — abuso: números sem identificação atraem golpe e spam, com risco jurídico e de reputação para a marca (lição do Wickr Me, encerrado em 2023).

- **TF6 \[SIM\]** O que o usuário faz para falar com quem não tem Nimbus: convida a pessoa por link ou QR. Emergências continuam pelo telefone do próprio aparelho, fora do app. A varredura de código reprova qualquer biblioteca de telefonia comum (SIP/PSTN).

## A6E. Envio de documentos: três níveis, do seguro ao extremo

Não existe transmissão sem transmissão: para um documento chegar a alguém, algo trafega. O que garantimos é que só trafega o que não serve a ninguém — bloco cifrado ou imagem da tela — e que o servidor nunca tem a chave.

| **Nível**                        | **Como funciona**                                                                                                                                                                                                | **Para quê**                                                                            |
|----------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
| **1. Dentro do NimbusChat**      | Entre usuários Nimbus: cifra ponta a ponta, visualização única, validade; mensagens nunca saem e arquivos só saem com a autorização de quem enviou                                                               | O dia a dia entre clientes Nimbus                                                       |
| **2. Nimbus Envio (link)**       | Para quem não tem o app: o arquivo é cifrado no aparelho; a chave vai depois do \# do link, parte que o navegador nunca envia ao servidor; senha opcional; expira por prazo e por número de downloads; revogável | Mandar um contrato ao advogado, a um cliente, a quem for, sem ele instalar nada         |
| **3. Sala Blindada (Nimbus OS)** | O arquivo nunca sai: fica na máquina virtual confidencial e o destinatário só vê a imagem das páginas no navegador, sem marca d’água; o remetente revoga o acesso na hora                                        | Documento que não pode vazar como arquivo: due diligence, contrato em negociação, prova |

- **ENV0 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **ENV0b \[SIM\]** Lição do mercado: serviços de envio anônimo já foram usados para espalhar vírus, e o Firefox Send foi encerrado em 2020 por isso (validar). Só contas NimbusOne criam links, com limites de tamanho e quantidade, e o destinatário é avisado de que o arquivo vem de fora.

## A6F. Sem rastro também no arquivo, na falha e na memória

- **HIG0 \[SIM\]** Toda foto e todo arquivo saem limpos: sem GPS, modelo do celular, autor, data ou comentário escondidos, e com nome neutro (código pronto, módulo higiene). PDF, Word e Excel são “achatados” antes de sair — viram páginas-imagem, porque os metadados deles se escondem em muitos lugares.

- **RZ0 \[SIM\]** Relatório de falha (crash) nunca leva conteúdo: segredos e textos abertos ficam num tipo que só mostra o tamanho, nunca o conteúdo (código pronto, módulo memoria); mensagens de erro são proibidas de conter conteúdo (varredura).

- **RZ0b \[SIM\]** Notificações agendadas pelo app (lembretes da agenda) ficam guardadas pelo sistema até tocar: por isso têm só texto genérico — “Você tem um compromisso” — e são canceladas quando o item é apagado.

- **RZ0c \[PARCIAL\]** Memória RAM: o conteúdo precisa existir aberto na memória enquanto está na tela; fica o menor tempo possível, travado na RAM para não ir ao disco e zerado ao fechar. No computador, a hibernação grava a RAM em disco: a edição de computador exige disco cifrado do sistema.

## A6G. Blockchain: por que não para os dados — e o que aproveitamos dele

Pergunta natural, resposta contraintuitiva: uma blockchain como a do Bitcoin é o contrário do que queremos para os dados. Ela é um registro público, copiado em milhares de computadores e impossível de apagar.

- **BC1 \[NÃO\]** Guardar mensagens, arquivos ou metadados numa blockchain tornaria tudo permanente e público: impossível apagar (quebra o apagamento criptográfico), metadados expostos para sempre e blocos cifrados guardados eternamente — à espera de um computador capaz de quebrá-los no futuro.

- **BC2 \[SIM\]** Aproveitamos a parte útil da tecnologia: a árvore de Merkle, um registro só-de-acréscimo que ninguém consegue alterar sem ser pego — nem nós. Usamos para as chaves dos aparelhos, as imagens do Nimbus OS e as versões do app (código pronto, módulo registro, conforme a RFC 9162).

- **BC3 \[PARCIAL\]** “Ninguém sabe onde o servidor roda”: dá para esconder a localização do servidor de usuários e da rede, publicando-o como serviço onion da rede Tor, e espalhar os retransmissores entre operadores independentes. Mas quem implanta o servidor sabe onde implantou. A proteção real não é esconder o servidor — é ele não guardar nada legível: mesmo quem o encontrar, invadir ou apreender não lê nada.

- **BC4 \[SIM\]** Descentralizar a confiança, que é o que a blockchain promete: quatro partes independentes (servidores, relay que oculta o IP, custodiante da meia-chave, auditor) e publicação de versões só com várias assinaturas (A6H).

## A6H. Nenhum programador — nosso ou de fora — acessa os dados

- **PG1 \[SIM\]** Servidor: nada legível. Nenhum programador lê o que não existe em claro no servidor, com ou sem senha de administrador.

- **PG2 \[SIM\]** O ponto de ataque verdadeiro é o app: quem escreve o app poderia escrever uma versão espiã. Defesa: publicar uma versão exige assinaturas de várias pessoas diferentes (por exemplo, 3 de 5, incluindo o auditor externo) — código pronto, módulo aprovacao. O app recusa atualização que não esteja no registro público de versões.

- **PG3 \[SIM\]** A versão publicada é verificável: código aberto para auditoria, compilação reproduzível e registro público. Uma versão espiã não passaria escondida.

- **PG4 \[PARCIAL\]** Verdade: um programador pode escrever código malicioso na máquina dele. O que o desenho garante é que ele não consegue publicá-lo sozinho nem escondê-lo.

## A6I. Quem não tem o Nimbus: recebe — e isso abre espaço, sim

Sua intuição está correta. Quando o destinatário baixa o arquivo pelo Nimbus Envio, a cópia passa a viver no aparelho dele, fora do nosso controle: pode ser encaminhada, copiada, ou estar num aparelho infectado. O apagamento criptográfico não alcança essa cópia.

- **EX1 \[SIM\]** Documento sensível vai pela Sala Blindada: o arquivo nunca sai, a pessoa só vê a imagem das páginas e o acesso é encerrado quando você quiser. Sem marca d’água.

- **EX2 \[SIM\]** Nimbus Envio no modo “só visualizar”: o documento abre no navegador sem botão de download. Mais fraco que a Sala Blindada (o conteúdo chega decifrado ao navegador), mais forte que o download livre.

- **EX3 \[SIM\]** Padrão seguro: 1 download, 24 horas, senha por outro caminho. Empresas podem desligar o Nimbus Envio por política e exigir Sala Blindada.

## A6K. Código malicioso — de gente, de IA ou de fornecedor

“Nunca ser hackeado” ninguém consegue prometer. O que dá para garantir é que não exista ponto único: nenhuma pessoa, ferramenta ou fornecedor consegue sozinho colocar algo malicioso no app sem ser pego.

- **MAL1 \[SIM\]** Revisão de duas pessoas em toda alteração, ramo principal protegido, commits assinados e chave física para quem acessa o repositório.

- **MAL2 \[SIM\]** Publicação só com 3 de 5 assinaturas, incluindo o auditor externo (módulo aprovacao); o app só instala versão presente no registro público (módulo registro).

- **MAL3 \[SIM\]** Dependências travadas por hash, verificadas por cargo-audit e cargo-deny a cada alteração, inventário SBOM a cada versão; compilação de release sem acesso à internet (pipeline pronto no pacote do código).

- **MAL4 \[SIM\]** IA no desenvolvimento como assistente, nunca como autoridade: todo código de IA passa pela mesma revisão, testes e varredura; IA nunca recebe segredos nem dados de usuário; agentes de IA sem permissão de escrita no ramo principal nem acesso à produção.

- **MAL5 \[SIM\]** Contra truques de IA: instruções escondidas em arquivos, tickets e páginas lidas por agentes são tratadas como dados, nunca como ordens; biblioteca sugerida por IA é conferida pelo nome exato, porque atacantes registram nomes parecidos ou inventados por IA.

- **MAL6 \[SIM\]** No aparelho: atestação do app a cada conexão, detecção de ferramentas de injeção e depuração, e o servidor recusa cliente adulterado (DV, ID).

- **MAL7 \[PARCIAL\]** Verdade: um programador pode escrever código malicioso na máquina dele; o desenho garante que ele não consegue publicá-lo sozinho nem escondê-lo.

## A6L. Memória flash: por que destruir a chave é o método certo — e rasurar não ajuda

- **FL1 \[SIM\]** Não gravamos conteúdo em claro no armazenamento do aparelho. Não existe o que reconstruir.

- **FL2 \[SIM\]** O pouco que é gravado (cache offline opcional, chaves de janela) é cifrado; ao apagar, destruímos a chave. É o mesmo princípio que o próprio iPhone usa para apagar arquivos: sem a chave, os dados restantes são ruído. Com cifra de 256 bits, nenhuma técnica conhecida — nem computador quântico — recupera o conteúdo sem a chave.

- **FL3 \[NÃO\]** Rasurar (sobrescrever) não é recomendado em memória flash: o chip grava em células novas para distribuir o desgaste, então a sobrescrita pode nem atingir as células antigas, desgasta o aparelho e dá falsa segurança.

- **FL4 \[SIM\]** O que resta de risco real não está no armazenamento, e sim na memória RAM enquanto o conteúdo está na tela (zerada ao fechar) e nos buffers internos do sistema (fora do controle de qualquer app).

## A6M. Cada exposição, resolvida — e o pouco que sobra, com o controle nas mãos do usuário

Nenhuma pendência: cada item tem solução implementável. Onde algo sobra, é porque depende de Apple, Google ou da rede telefônica, e está declarado — com a alternativa que o usuário pode escolher.

| **Exposição**                                                 | **Solução**                                                                                                                                                                                                                                                                           | **O que sobra**                                                                                                                                            | **Controle do usuário**            |
|---------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------|
| **Consulta de DNS e destino da conexão**                      | Modo rápido: rota em duas etapas (Oblivious HTTP), nome do servidor cifrado na conexão (ECH, RFC 9849) e DNS cifrado. Nenhuma consulta em aberto; quem observa a rede vê só a conexão com o retransmissor                                                                             | O operador da 1ª etapa sabe que o aparelho usa o serviço, sem ver conteúdo nem destino final                                                               | Automático ou Sempre Tor           |
| **Horário das notificações**                                  | Cadência fixa: uma notificação por aparelho a cada janela, real ou de cobertura, do mesmo tamanho (código pronto). No Android, sem Google: conexão própria                                                                                                                            | iPhone: descartar a de cobertura exige permissão da Apple, concedida caso a caso. Sem ela, entrega em janelas de 15 minutos                                | Modo sem notificação               |
| **Nome do app na fatura**                                     | Descritor neutro: nome jurídico da empresa, sem “Nimbus” ou “chat”; vale-assinatura comprado em dinheiro; ficha anônima separa pagamento de conta. Pela loja, a fatura mostra Apple ou Google                                                                                         | O processador sabe que alguém pagou, não quem é a conta                                                                                                    | Escolher vale em dinheiro          |
| **Ícone e nome na tela**                                      | iPhone: ícone alternativo, nome curto neutro (“Nimbus”) e o recurso do sistema de exigir Face ID ou ocultar o app. Android: ícone e nome trocáveis (ex.: “Notas”), fachada funcional com código e Espaço Privado                                                                      | iPhone não permite trocar o nome em uso nem fachada escondida (regra da App Store); app oculto fica sem notificações e chamadas e ainda aparece em Ajustes | Modo discreto                      |
| **IA do sistema lendo a tela**                                | Android: proteção de tela (FLAG_SECURE), exclusão da captura de conteúdo e do preenchimento automático. iPhone: camada segura (capturas saem pretas), ferramentas de escrita desligadas, nada exposto a Siri ou atalhos, notificações genéricas (resumos de IA não têm o que resumir) | Recurso novo do sistema ou sistema comprometido: reteste a cada versão (prova C41)                                                                         | —                                  |
| **Widgets, atividades ao vivo, impressão, arrastar e soltar** | Não existem no app (varredura reprova)                                                                                                                                                                                                                                                | Nada                                                                                                                                                       | —                                  |
| **Bluetooth**                                                 | Cerimônia presencial por QR e rede, sem Bluetooth. Transferência por proximidade só quando o usuário inicia, janela de 60 segundos, identificador que muda e nada fixo que diga “Nimbus” (código pronto)                                                                              | Quem escuta vê um aparelho anunciando um número aleatório por 60 segundos                                                                                  | Bluetooth desligado por padrão     |
| **Loja de apps sabe que instalou**                            | Android: instalação direta verificada, sem loja                                                                                                                                                                                                                                       | Inevitável na App Store                                                                                                                                    | Android direto                     |
| **Tempo de uso do sistema**                                   | —                                                                                                                                                                                                                                                                                     | O sistema registra quanto o app é usado; não há como impedir                                                                                               | Ocultar o app reduz a visibilidade |
| **Idioma, fuso e modelo nos pedidos**                         | Cabeçalhos iguais para todos (código pronto)                                                                                                                                                                                                                                          | Nada                                                                                                                                                       | —                                  |
| **Figurinhas e nomes de arquivos**                            | Pacotes baixados inteiros; nomes cifrados junto com o conteúdo                                                                                                                                                                                                                        | Nada                                                                                                                                                       | —                                  |

## A6N. Rede: Tor quando importa, velocidade no resto — e a escolha é do usuário

Um só controle, com três escolhas. O padrão, Automático, liga o Tor sozinho nos momentos em que ele faz diferença e usa a rota rápida no resto. Quem quiser decide manualmente.

| **Escolha**                          | **Como funciona**                                                                                                                                                                                                                                                                                                                | **Para quem**                                                         |
|--------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
| **Automático (padrão, recomendado)** | Dia a dia pela rota rápida em duas etapas. O app liga o Tor sozinho: ao criar ou recuperar conta, adicionar contato, cadastrar aparelho, aprovar ou publicar chaves; nas conversas blindadas; e quando a rede bloqueia a rota rápida (Tor com ponte, que parece tráfego comum). Um pequeno ícone mostra quando o Tor está em uso | Todos: sente como um app comum e fica protegido nos momentos críticos |
| **Sempre Tor (máximo)**              | Tudo pelo Tor, exceto chamadas; mais lento, com aviso                                                                                                                                                                                                                                                                            | Quem corre risco alto                                                 |
| **Rápido (sem Tor)**                 | Nunca usa Tor; a rota em duas etapas parece uma conexão comum com um grande provedor                                                                                                                                                                                                                                             | Quem está em país onde usar Tor chama atenção ou é bloqueado          |

“Sem Tor fico protegido da mesma maneira?” Para quase tudo, sim. A tabela mostra exatamente onde está a diferença:

| **Proteção**                                                                   | **Rápido (sem Tor)**                            | **Automático**                                          | **Sempre Tor**                                             |
|--------------------------------------------------------------------------------|-------------------------------------------------|---------------------------------------------------------|------------------------------------------------------------|
| **Conteúdo das mensagens, fotos e arquivos**                                   | Protegido                                       | Protegido                                               | Protegido                                                  |
| **Quem você é (conta sem telefone, e-mail ou documento)**                      | Protegido                                       | Protegido                                               | Protegido                                                  |
| **Com quem você fala (remetente oculto, pedidos não ligáveis)**                | Protegido                                       | Protegido                                               | Protegido                                                  |
| **Seu IP diante da Nimbus**                                                    | Protegido                                       | Protegido                                               | Protegido                                                  |
| **Que você usa o Nimbus, diante do seu provedor de internet**                  | Protegido (vê só a conexão com o retransmissor) | Protegido                                               | Protegido; o provedor vê que você usa Tor, salvo com ponte |
| **Seu IP nos momentos de identidade, mesmo se a infraestrutura conspirar**     | Não coberto                                     | Protegido                                               | Protegido                                                  |
| **Horário da sua atividade, se o operador da 1ª etapa e a Nimbus conspirarem** | Não coberto                                     | Coberto nos momentos críticos e nas conversas blindadas | Protegido                                                  |

- **NET1 \[SIM\]** Rota rápida em duas etapas: Oblivious HTTP (RFC 9458) por retransmissor de terceiro, nome do servidor cifrado (ECH, RFC 9849) e DNS cifrado — o mesmo princípio de duas etapas do iCloud Private Relay (validar).

- **NET2 \[SIM\]** Controle com três escolhas: Automático (padrão), Sempre Tor e Rápido (sem Tor). Toda operação de rede consulta rede::decidir; a interface nunca decide a rota (código pronto).

- **NET3 \[SIM\]** Automático liga o Tor nos momentos de identidade: criar e recuperar conta, adicionar contato, cadastrar aparelho, aprovar, publicar chaves.

- **NET4 \[SIM\]** Automático liga o Tor nas conversas blindadas: a escolha vale para os dois lados da conversa, que buscam e enviam as mensagens dela pelo Tor.

- **NET5 \[SIM\]** Rede bloqueando a rota rápida: o Automático sobe para Tor com ponte disfarçada; o app continua funcionando onde outros param.

- **NET6 \[SIM\]** Nunca descer do Tor para a rota rápida sozinho: se o Tor falhar, tenta ponte; se a ponte falhar, pergunta ao usuário.

- **NET7 \[SIM\]** “Rápido (sem Tor)” é respeitado sempre: nem o Automático sobe para o Tor. Nos momentos de identidade, o app mostra que seguiu sem Tor por escolha do usuário.

- **NET8 \[SIM\]** Chamadas nunca pelo Tor (tempo real): retransmissores de operadores independentes, em todas as escolhas.

- **NET9 \[SIM\]** Primeira etapa da rota rápida sorteada, por sessão, entre vários operadores independentes, de empresas e países diferentes, sem vínculo societário com a Nimbus.

- **NET10 \[SIM\]** Circuito Tor mantido aquecido em segundo plano enquanto o app está aberto, para a troca automática levar poucos segundos.

- **NET11 \[SIM\]** Tráfego de cobertura leve na rota rápida (economiza bateria) e completo no Sempre Tor.

- **NET12 \[SIM\]** Indicador discreto de rota: o usuário sempre sabe quando o Tor está em uso.

- **NET13 \[PARCIAL\]** Limite honesto do Rápido: se o operador sorteado da 1ª etapa e a Nimbus conspirassem, poderiam saber que um IP usa o Nimbus e quando — nunca o conteúdo, a conta ou os contatos. Por isso existem o Automático e o Sempre Tor.

- **NET14 \[SIM\]** Metas de velocidade a medir desde a Fase 1 (não são resultados): mensagem entregue em até 1 segundo em 95% dos envios pela rota rápida; app aberto até a lista de conversas em até 2 segundos; troca para o Tor em poucos segundos com circuito aquecido.

## A6O. O nome que você salva no contato — e todo nome, em todo lugar

Sim: o nome que você dá a um contato é cifrado no seu aparelho antes de ir a qualquer lugar. A revisão encontrou que isso dependia da camada do app; agora está no núcleo, com teste (módulo contatos).

- **CTT1 \[SIM\]** Contato salvo (nome que você deu, chaves da pessoa, número de segurança, notas, etiqueta) vira um único bloco cifrado para a sua chave, de tamanho fixo: o servidor não sabe quantos contatos você tem, como se chamam nem se o nome é curto ou longo.

- **CTT2 \[SIM\]** O nome que a própria pessoa escolheu (perfil) é cifrado com uma chave de perfil que ela entrega só a quem aceitou como contato; o servidor guarda só ruído de tamanho fixo.

- **CTT3 \[SIM\]** Na tela, o nome só existe decifrado na memória, pelo tempo em que aparece; busca de contatos feita em memória, sem índice gravado.

- **CTT4 \[SIM\]** Nome de contato nunca aparece em notificação, tela de chamada recebida com o aparelho bloqueado, relatório de falha ou registro (varredura e regras RZ9, RZ11).

- **CTT5 \[PARCIAL\]** Limite: enquanto a lista de contatos está aberta na tela, os nomes existem na memória do aparelho (SV4); somem ao fechar.

## A6P. Riscos que ninguém tinha levantado — e a decisão para cada um

- **RSK1 \[SIM\]** Spam sem telefone: conta anônima é fácil de criar em massa. Decisão: ninguém fala com você sem convite de uso único e prazo; não existe busca por nome ou número; o servidor guarda só o hash do convite (código pronto, módulo convites). Criação de conta com ficha anônima de custo simbólico ou prova de trabalho (validar).

- **RSK2 \[PARCIAL\]** Quem participa de cada grupo: se o servidor souber os membros, conhece o círculo de relações. Decisão: membros guardados cifrados com credenciais anônimas de grupo (modelo do sistema de grupos privados do Signal, validar) e entrega por caixas que o servidor não liga. A construir na Fase 2.

- **RSK3 \[PARCIAL\]** Página do navegador (Nimbus Envio e Sala Blindada): quem entrega a página poderia entregar código espião que lê a chave. Decisão: a página tem seu hash publicado no registro público; política de segurança de conteúdo rígida, sem scripts de terceiros; e “abrir no app Nimbus” como caminho mais seguro, sempre oferecido.

- **RSK4 \[SIM\]** Chamada recebida com o celular bloqueado: a tela mostraria o nome de quem liga. Decisão: tela genérica “Chamada Nimbus”; o nome só aparece com o app desbloqueado.

- **RSK5 \[SIM\]** Apagar a conta: o que acontece com o que ficou no servidor? Decisão: apagar a conta destrói todas as chaves; todo bloco guardado, e qualquer cópia, vira ruído para sempre (código pronto).

- **RSK6 \[PARCIAL\]** Guardiões de recuperação combinados entre si poderiam reconstruir a chave. Decisão: cada guardião tem só uma parte, que sozinha não revela nada; o app orienta a escolher guardiões que não se conhecem.

- **RSK7 \[SIM\]** Assinaturas pós-quânticas: RESOLVIDO na v21. Toda assinatura do núcleo é dupla, Ed25519 + ML-DSA-87 (FIPS 204); as duas precisam conferir (módulo assinatura).

- **RSK8 \[SIM\]** Ambientes de teste: dados reais de usuários são proibidos fora de produção; as mesmas regras de registro e varredura valem em homologação.

## A6Q. Como a revisão passou a não depender de atenção humana

Pergunta justa: por que cada revisão achava um erro novo? Porque revisão por leitura, minha ou de qualquer pessoa, depende de atenção. A resposta profissional é medir a força dos próprios testes. Ninguém honesto promete zero defeitos em software; o que existe é um método que torna defeitos raros e faz o sistema pegá-los sozinho.

- **VER1 \[SIM\]** Testes que rodam sozinhos: 449 no cargo test — cada regra com exemplos, autoverificação da criptografia (bit trocado, chave errada, assinatura híbrida) e o desafio de atestação contra reenvio — mais o fuzzing (C79) e o tempo constante (C80) no verificar.sh.

- **VER2 \[SIM\]** Testes de propriedades: leis que valem para qualquer entrada, com centenas de casos sorteados (15).

- **VER3 \[SIM\]** Testes de robustez: milhares de entradas aleatórias e adulteradas; nada pode derrubar o app (8).

- **VER4 \[SIM\]** Teste por mutação: 353 defeitos plantados em pontos críticos de todos os módulos; o pipeline exige que todos sejam pegos (353 de 353 exigidos). Tudo verificável com um comando: ./verificar.sh (a mutação faz parte dele).

- **VER5 \[SIM\]** Compilação com os avisos mais rigorosos: zero avisos, inclusive no código de teste.

- **VER6 \[SIM\]** Varredura com 47 regras: zero ocorrências.

- **VER7 \[SIM\]** Regra para o time: toda regra de segurança nova entra com teste de exemplo, propriedade e mutante; mutante vivo reprova a entrega.

- **VER8 \[SIM\]** Camadas humanas: revisão em dupla, verificação formal dos protocolos, auditoria independente e programa de recompensa por falhas — a última palavra continua sendo de terceiros.

## A6R. Se houver invasão: o que continua cifrado — e a única janela que existe

Pergunta decisiva, resposta exata. A garantia depende de onde e quando o ataque acontece:

| **Situação do ataque**                                                        | **O que o invasor obtém**                | **Por quê**                                                                                                                   |
|-------------------------------------------------------------------------------|------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------|
| **Servidor invadido, copiado ou apreendido**                                  | Nada legível                             | Só blocos cifrados, de tamanho padronizado, sem nomes, tipos ou donos; as chaves nunca estão lá                               |
| **Cópia de tudo o que passou pela rede**                                      | Nada legível, hoje ou no futuro quântico | Troca de chaves híbrida com ML-KEM-1024; conteúdo com cifra de 256 bits                                                       |
| **Celular roubado, desligado ou bloqueado**                                   | Nada legível                             | Nada gravado em claro; chaves no chip de segurança; memória zerada ao bloquear                                                |
| **Backup antigo ou cópia vazada de um item**                                  | Nada legível após o prazo                | Apagamento criptográfico: a chave da janela foi destruída                                                                     |
| **Celular invadido por espionagem de alto nível enquanto a pessoa usa o app** | Só o que está na tela naquele instante   | Para aparecer na tela, o conteúdo precisa estar decifrado na memória — em qualquer sistema do mundo; o resto continua cifrado |

- **INV1 \[SIM\]** Cada item tem chave própria (envelope novo a cada item): quebrar um não abre os outros.

- **INV2 \[SIM\]** Mensagens com sigilo futuro e recuperação (protocolo do Signal): roubar a chave de hoje não abre o passado nem o futuro.

- **INV3 \[NÃO\]** Ninguém pode garantir que um aparelho tomado por espionagem durante o uso não veja a própria tela. A defesa é encolher essa janela: pouco tempo aberto, bloqueio automático, Escudo, verificação do aparelho, e o celular virtual para o que exige o máximo.

## A6S. Notificações: o que é possível sem Apple e sem Google

- **NOT8 \[SIM\]** Android e computador: canal próprio do Nimbus. Google e terceiros não participam da entrega (código pronto, notificacao::canal_para).

- **NOT9 \[NÃO\]** iPhone: o sistema não permite acordar um app em segundo plano sem o serviço da Apple. Não existe caminho técnico permitido para contornar isso.

- **NOT10 \[SIM\]** O que a Apple recebe no iPhone: só um bloco cifrado de tamanho fixo, na cadência fixa, sem remetente, sem conteúdo e sem dizer se é mensagem ou chamada. Ela não sabe quem manda, quem liga nem o quê.

- **NOT11 \[SIM\]** Chamada recebida com o celular bloqueado mostra só “Chamada Nimbus”; notificação mostra só “Nova atividade”. O histórico de notificações do Android guarda apenas esse texto genérico.

- **NOT12 \[SIM\]** Quem quiser zero participação da Apple no iPhone: modo sem notificação (as mensagens chegam quando o app é aberto).

## A6T. Tudo cifrado: a lista completa

| **O que é**                                                            | **Como fica**                                                                                               | **Módulo**                     |
|------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------|--------------------------------|
| **Mensagens, áudios, fotos, vídeos (inclusive de visualização única)** | Ponta a ponta com troca de chaves pós-quântica; visualização única para texto, foto, vídeo, áudio e arquivo | Protocolo do Signal + politica |
| **Nome que você dá ao contato, notas, chaves da pessoa**               | Bloco de tamanho fixo cifrado para você                                                                     | contatos                       |
| **Nome que a própria pessoa escolheu (perfil)**                        | Cifrado com chave entregue só aos contatos                                                                  | contatos                       |
| **Nome, tipo, data e pasta de fotos e arquivos**                       | Dentro do mesmo bloco do conteúdo                                                                           | pacote                         |
| **Álbuns e nomes de álbuns, notas, compromissos, senhas, ajustes**     | Tipo escondido dentro da cifra; classes de tamanho                                                          | cofre                          |
| **Arquivo enviado por link, com o nome**                               | Nome dentro da cifra; página sem nenhum dado de quem enviou                                                 | envio                          |
| **Histórico no servidor**                                              | Blocos cegos; apagamento criptográfico por prazo                                                            | historico, validade            |
| **Notificações**                                                       | Bloco cifrado de tamanho fixo, cadência fixa                                                                | notificacao                    |
| **PIN de coação, senha de link**                                       | Argon2id (64 MiB por tentativa)                                                                             | senha, coacao                  |
| **Aprovações, prazos, Escudo**                                         | Assinatura híbrida pós-quântica                                                                             | assinatura                     |
| **Convites**                                                           | Servidor guarda só o hash                                                                                   | convites                       |

## A6U. Pós-quântico: o que está entregue hoje

| **Função**                | **Algoritmo**                                             | **Situação**    | **Referência**                                   |
|---------------------------|-----------------------------------------------------------|-----------------|--------------------------------------------------|
| **Troca de chaves**       | X25519 + ML-KEM-1024 (híbrido)                            | Entregue        | NIST FIPS 203                                    |
| **Assinatura**            | Ed25519 + ML-DSA-87 (híbrido)                             | Entregue na v21 | NIST FIPS 204                                    |
| **Cifra de conteúdo**     | XChaCha20-Poly1305 ou AES-256-GCM (edição FIPS), 256 bits | Entregue        | Resistente com margem a computador quântico      |
| **Derivação e resumo**    | HKDF-SHA-512; registro público com SHA-256 (RFC 9162)     | Entregue        | Edição militar: registro com SHA-384 (a ajustar) |
| **Senhas e PINs**         | Argon2id                                                  | Entregue na v21 | IETF RFC 9106                                    |
| **Protocolo de conversa** | PQXDH + Triple Ratchet (Signal)                           | Via libsignal   | Especificações públicas do Signal                |

- **PQ1 \[SIM\]** Alinhamento com a suíte CNSA 2.0 da NSA para sistemas de segurança nacional (ML-KEM-1024, ML-DSA-87, AES-256, SHA-384/512): a edição Enterprise/FIPS usa ML-KEM-1024, ML-DSA-87, AES-256-GCM e HKDF-SHA-512. Dois ajustes a validar com o órgão avaliador: se a combinação híbrida com X25519 e Ed25519 é aceita ou se a edição deve usar os algoritmos pós-quânticos puros; e a troca do SHA-256 do registro público por SHA-384.

- **PQ2 \[PARCIAL\]** Verdade: usar os algoritmos da CNSA 2.0 não é o mesmo que ser certificado para uso militar. Certificação exige módulo criptográfico validado (FIPS 140-3), avaliação formal (Common Criteria e programas nacionais) e aprovação do órgão contratante.

- **PQ3 \[PARCIAL\]** As bibliotecas pós-quânticas usadas (RustCrypto ml-kem e ml-dsa) ainda não têm auditoria formal publicada; a auditoria do NimbusOne precisa cobri-las ou trocá-las por módulo validado na edição FIPS.

## A6V. Posicionamento: como apresentar sem risco

- **POS1 \[NÃO\]** Não afirmar “a mais segura do mundo” nem “impossível de hackear”: nenhuma avaliação independente sustenta isso hoje, e a frase vira alvo — de concorrentes, jornalistas e pesquisadores.

- **POS2 \[SIM\]** Afirmações que se sustentam: “construída com os algoritmos exigidos pela NSA para sistemas de segurança nacional (CNSA 2.0)”; “o servidor não guarda nada legível — nem nomes, nem tipos, nem quem fala com quem”; “cada promessa verificável por auditoria independente”; “projetada para uso governamental e militar”.

- **POS3 \[SIM\]** Rota para uso militar: módulo FIPS 140-3 validado na edição Enterprise, avaliação Common Criteria, programa de aprovação do país contratante, auditoria independente publicada e programa de recompensa por falhas ativo.

## A6W. Detalhes que quase ninguém percebe no link de envio

- **WEB5 \[SIM\]** A chave sai da barra de endereços assim que é lida (history.replaceState): não fica no histórico do navegador.

- **WEB6 \[SIM\]** Robôs de pré-visualização (WhatsApp, e-mail, redes) e verificadores de links de e-mail abrem só a página estática: o download exige clique e, se houver, senha — nenhum robô consome o download único.

- **WEB7 \[SIM\]** Página sem cache, sem indexação, sem enviar de onde veio, sem scripts de fora e sem moldura alheia (cabeçalhos prontos em envio::cabecalhos_da_pagina).

- **WEB8 \[SIM\]** Título e descrição iguais para todos os links (“Nimbus Envio”, “Arquivo protegido”): nenhuma pré-visualização mostra quem enviou, o nome do arquivo ou a data.

- **WEB9 \[PARCIAL\]** Histórico da área de transferência (Windows, Android) guarda links copiados: o app recomenda enviar o link pelo NimbusChat ou por QR, e limpa a área de transferência onde o sistema permite.

## A6X. Revisão v37: computador, sem modo offline e NimbusDrop lacrado

A arquitetura vale para a ferramenta toda: servidor, celular (iOS e Android) e computador (macOS e Windows). O servidor nunca tem as chaves; em cada aparelho, tudo fica cifrado quando guardado. O conteúdo só existe aberto no instante em que aparece na tela de quem tem direito — limite físico de qualquer sistema. Esta revisão fecha o que o computador, a ausência de internet e a proximidade abriam.

- **PC1 \[SIM\]** O computador (macOS e Windows) é aparelho vinculado a um celular já ativo: vincula por QR mostrado no computador e lido pelo celular, com biometria no celular; entra no registro público de chaves (BL1) e aparece na lista de aparelhos do dono e dos contatos.

- **PC2 \[SIM\]** Chave no chip do computador (Secure Enclave no Mac, TPM 2.0 no Windows) e disco do sistema cifrado (FileVault ou BitLocker, RZ12). Sem um dos dois, o computador fica vermelho e não vincula (integridade::avaliar; prova C89).

- **PC3 \[PARCIAL\]** Teto amarelo: a Apple documenta que o App Attest não está disponível para apps no Mac, e o Windows não tem equivalente. Sem prova do fabricante, o computador nunca fica verde (integridade::tem_atestacao_do_fabricante), e Cofre, visualização única e NimbusDrop ficam só no celular. O mesmo teto vale para a web.

- **PC4 \[SIM\]** O vínculo cai sozinho após 7 dias sem o celular confirmar, e o celular derruba o computador a qualquer momento (servidor e ponte; prova C90).

- **PC5 \[PARCIAL\]** Janela do app excluída da captura do sistema (Windows: WDA_EXCLUDEFROMCAPTURE; macOS: janela não compartilhável). Ferramentas de captura mais novas podem não respeitar o pedido em todas as versões (validar, VF27).

- **OFF1 \[SIM\]** Sem rede mesh na v1, por decisão do fundador: nenhuma mensagem trafega por Bluetooth, Wi-Fi Direct ou aparelhos de terceiros; tudo passa pelo servidor. Motivo: offline, três garantias caem (prazo pela hora do servidor, derrubada de sessão paralela e reconfirmação do aparelho), e as análises públicas do Bridgefy (Royal Holloway, 2021 e 2022) mostraram rastreamento de usuários e montagem de grafos sociais em rede mesh. Varredura: regra nimbus-sem-rede-mesh (prova C92).

- **OFF2 \[SIM\]** Hora confirmada: todo prazo (mensagens, visualização única, NimbusDrop e validade do cofre) usa expiracao::hora_confiavel — a última hora assinada pelo servidor mais o relógio monotônico do aparelho, que não muda quando alguém altera a hora do sistema. Atrasar o relógio não estende nada; adiantar só faz sumir antes. Sem confirmação há mais de 15 minutos, nada com prazo aparece (expiracao::pode_exibir; prova C91). A hora só é aceita se responder ao desafio de uso único que o aparelho mandou, com assinatura do servidor e sem voltar no tempo (expiracao::conferir_hora): uma resposta antiga repetida por quem controla a rede é recusada.

- **OFF3 \[SIM\]** NimbusChat sem cache offline: a opção “Cache offline cifrado” sai dos Ajustes. Sem internet, o app mostra “Conectando” e nenhum conteúdo.

- **OFF4 \[PARCIAL\]** NimbusOne: o cache offline opcional do cofre continua (EF3), mas itens com validade só aparecem com hora confirmada (OFF2); itens sem validade seguem visíveis offline.

- **ND2 \[SIM\]** NimbusDrop lacrado: o arquivo pode chegar por proximidade sem internet, em duas camadas. A de fora só a chave do destinatário abre — quem captar o Bluetooth não lê nada; a de dentro usa uma chave de uso único (direto::lacrar). O identificador e o prazo vão no dado autenticado das duas camadas: alterar o prazo que viaja junto invalida o arquivo.

- **ND3 \[SIM\]** A chave do lacre só viaja pelo canal normal do NimbusChat, cifrado de ponta a ponta, via servidor — nunca pela proximidade. O servidor entrega o envelope sem conseguir ler a chave. Nenhuma criptografia, protocolo, biblioteca ou servidor novos.

- **ND4 \[SIM\]** Abrir exige, nesta ordem: aparelho verde, hora confirmada, chave recebida e prazo válido pela hora do servidor (direto::pode_abrir_lacrado e direto::abrir_lacrado; prova C93).

- **ND5 \[SIM\]** Tela de quem recebe sem conexão: “Recebido. Abre quando houver conexão.” Com internet, abre em segundos. Custo aceito: sem internet, o arquivo não pode ser visto na hora.

- **PQ4 \[PARCIAL\]** Combinador híbrido: o núcleo deriva a chave com HKDF sobre os dois segredos (X25519 e ML-KEM-1024) e a cifra do ML-KEM. O X-Wing (rascunho IETF) é definido com ML-KEM-768; o auditor decide entre manter o combinador próprio com 1024, documentado, ou adotar o X-Wing (VF28).

- **DIR8 \[SIM\]** O envio direto é sempre assinado por quem envia (direto::preparar_assinado e direto::abrir_assinado). O envelope é anônimo por construção — qualquer pessoa com a chave pública de quem recebe consegue montar um —, então o app só abre o arquivo e só mostra “enviado por” quando a assinatura confere e a impressão digital da chave pertence a um contato. A assinatura cobre o destinatário, o identificador e o prazo: um arquivo assinado para uma pessoa não serve para outra. Prova C94.

- **BL6 \[SIM\]** Prova de consistência do registro público (RFC 9162, 2.1.4): o app guarda a última raiz aceita e só aceita uma raiz nova com prova de que o registro anterior está contido nela, sem alteração (registro::verificar_consistencia). Sem isso, o servidor poderia mostrar registros diferentes a pessoas diferentes ou reescrever o passado — e esconder um aparelho fantasma. Para pegar registros bifurcados, os aparelhos também comparam a raiz entre si nas conversas. Prova C95.

- **CRY9 \[SIM\]** Chave pública ML-KEM recebida de fora passa pela verificação da FIPS 203 (7.2): decodificar e codificar de novo tem de devolver os mesmos bytes; coeficiente fora do intervalo é recusado (envelope::ChavePublica::de_bytes). Prova C96.

## A6Y. Engenharia social e segurança operacional

- **EO1 \[SIM\]** Convite sem botão "são iguais": o app mostra quatro palavras e a pessoa toca na que o outro leu em voz alta, na posição sorteada (convites::opcoes_para_confirmar, convites::confirmacao_correta). Fecha o golpe "é só tocar em iguais". Prova C97.

- **EO2 \[SIM\]** Vínculo de aparelho só a partir da tela "Vincular aparelho" do próprio app; QR recebido, link ou câmera do sistema são recusados; exige celular verde, biometria e palavras iguais nos dois aparelhos (sessoes::pode_vincular, sessoes::palavras_do_vinculo). Uma sessão por aparelho registrado: celular e computador vinculado convivem (sessoes::Registro::conectar). Fecha o golpe do QR de "aparelho vinculado" relatado pelo Google Threat Intelligence Group em fevereiro de 2025. Prova C98.

- **EO3 \[SIM\]** Nomes: termos reservados (Nimbus, suporte, segurança, oficial, administrador e afins, mesmo disfarçados) e caracteres enganosos recusados, também em cartão forjado; nome parecido com outro contato ganha aviso; mudança de nome nunca é silenciosa; com nome travado, o nome que você salvou aparece sempre embaixo (apresentacao::nome_permitido, apresentacao::parecido_com, apresentacao::mudanca_de_nome, apresentacao::subtitulo). Prova C99.

- **EO4 \[PARCIAL\]** Recuperação com amigos: cada amigo só libera a parte depois de conferir ao vivo as palavras do aparelho novo; espera de 3 dias com aviso em todos os aparelhos antigos, que cancelam com um toque (recuperacao::abrir_para_devolver, recuperacao::palavras_do_pedido, recuperacao::pode_concluir). Limite: as palavras provam para qual aparelho vai a parte, não quem pede; contra voz ou vídeo falsificados valem o reconhecimento pelo amigo, dois amigos que não se conhecem e a espera. Prova C100.

- **EO5 \[SIM\]** Recuperação de dados de conta corporativa suspensa espera 24 h, visível a todos os administradores, que podem cancelar (corporativo::autorizar_recuperacao). Fecha o golpe do "diretor com pressa". Prova C101.

- **EO6 \[SIM\]** Não existe conta de atendimento dentro do app. App e site dizem: a Nimbus nunca pede frase, PIN, palavras, código de convite ou cartão, por nenhum canal. Atendimento proibido de pedir segredos.

- **EO7 \[PARCIAL\]** Página do Nimbus Envio: domínio exclusivo e estático, sem script de terceiros, política de segurança de conteúdo estrita, impressão digital de cada versão no registro público, publicação com duas pessoas. Limite de qualquer criptografia no navegador: quem toma a hospedagem pode trocar a página.

- **EO8 \[SIM\]** Contas da empresa (domínio, DNS, lojas, repositório, e-mail, nuvem): chave física obrigatória, sem SMS, bloqueio de registro no domínio, contas de administração separadas, duas pessoas para mudanças, monitoramento de certificados emitidos.

- **EO9 \[SIM\]** Sem mensagens do sistema dentro das conversas: avisos de segurança só numa tela própria do app. Alertas de segurança são raros, específicos e não se desligam.

- **EO10 \[SIM\]** Pedidos de autoridade só por canal oficial, conferidos pelo advogado; nunca por telefone ou e-mail; resposta só com o que existe.

## A6Z. Atacante com IA e defesa em profundidade

- **FS1 \[SIM\]** Chaves de recebimento de vida curta (no máximo 7 dias), uma nova por dia, publicadas no servidor e assinadas pela identidade (prechaves::gerar). Prova C102.

- **FS2 \[SIM\]** Quem envia confere a assinatura da identidade e o prazo antes de usar a chave de recebimento: o servidor não consegue trocá-la (prechaves::conferir). Prova C102.

- **FS3 \[SIM\]** Envio direto, NimbusDrop e demais usos de envelope fora da libsignal selam para a chave de recebimento conferida; o identificador, o prazo e o uso vão no dado autenticado (prechaves::selar). Prova C102.

- **FS4 \[SIM\]** A parte secreta é destruída 48 h depois do fim da validade (prechaves::Chaveiro::apagar_vencidas): envelopes antigos ficam ilegíveis para sempre, mesmo para quem tomar o aparelho ou tiver gravado o tráfego. Prova C102.

- **FS5 \[SIM\]** A hora da destruição vem de expiracao::hora_confiavel, nunca do relógio do aparelho.

- **PQS7 \[SIM\]** Assinatura ML-DSA-87 na variante hedged da FIPS 204 (32 bytes aleatórios por assinatura), recomendada pela norma: mais resistente a injeção de falha e canal lateral (assinatura::ChaveDeAssinatura::assinar). Prova C103.

- **IA8 \[SIM\]** Agente de IA que constrói o app (Claude Code): instrução encontrada em arquivo, comentário, issue, página ou dependência nunca é ordem; nada que afrouxe regra de segurança entra sem revisão de uma segunda pessoa; varredura e consistência reprovam o afrouxamento.

## A6Z2. Back-end: rotas, tentativas e banco com RLS

- **SRV1 \[SIM\]** Rotas do servidor negadas por padrão: só existem as do inventário (autenticacao::ROTAS), comparadas por igualdade exata de método e caminho (autenticacao::rota). Prova C104.

- **SRV2 \[SIM\]** Prova por recurso, não por pessoa: a caixa é aberta com a chave da caixa; só as rotas de conta usam a chave do aparelho; depositar e criar caixa exigem ficha anônima de licença. Chave de um tipo não abre rota de outro (autenticacao::Porteiro::autorizar). Prova C104.

- **SRV3 \[SIM\]** A assinatura do pedido (híbrida) cobre método, caminho, resumo do corpo, titular, horário e número de uso único; janela de 60 s; pedido repetido recusado; só pedido com assinatura válida ocupa memória (autenticacao::provar). Prova C104.

- **SRV4 \[SIM\]** Limite de corpo por rota e revogação imediata do titular. Prova C104.

- **SRV5 \[SIM\]** Limites no servidor: cota por janela, castigo que dobra a cada falha até 24 h, memória com teto e falha fechada (limites::Limitador). A chave de limite é o titular da prova, a ficha ou o retransmissor — nunca a pessoa. Prova C105.

- **SRV6 \[SIM\]** Banco PostgreSQL com RLS ligada e forçada em todas as tabelas: sem a caixa autorizada pelo porteiro na transação, nenhuma linha aparece; depósito e criação de caixa só com ficha; prazo máximo de 48 h garantido pelo banco; bloco não se altera; registros de acesso só de escrita; papel do app sem posse das tabelas e sem ignorar RLS (servidor/banco/esquema.sql). Prova C106.

- **SRV7 \[SIM\]** O esquema do banco é testado num PostgreSQL real a cada mudança (teste_do_banco.sh) e com defeitos plantados no esquema (mutacao_do_banco.sh): 12 de 12 pegos. Prova C106.

- **SRV8 \[SIM\]** Faxina automática com papel próprio: apaga blocos vencidos e registros fora do prazo legal (nimbus.apagar_vencidos).

- **SRV9 \[SIM\]** Nenhuma senha no servidor; segredos do servidor só em módulo de hardware ou cofre da nuvem; respostas de erro genéricas; nenhuma rota de diagnóstico ou administração na API pública; o servidor nunca busca endereço vindo de fora.

- **SRV10 \[SIM\]** Disco do banco e cópias de segurança cifrados, com restauração testada; conexão do app com o banco por certificado, em rede privada.

## A6Z3. Segredos, entradas, IA, senhas, limites e manutenção

- **SEG1 \[SIM\]** Nenhum segredo em código, repositório, imagem, registro ou variável de ambiente de produção; segredos do servidor em módulo de hardware ou cofre da nuvem, sem exportação — o servidor pede a assinatura, não a chave; identidade do servidor no cofre pela plataforma, não por senha. Regras nimbus-segredo-no-codigo e nimbus-ambiente-fora-do-cofre; gitleaks no commit e no CI.

- **SEG2 \[SIM\]** O app não contém chave de API nenhuma: fala só com o servidor do Nimbus, provando posse de chaves nascidas no chip.

- **SEG3 \[SIM\]** Nenhuma entrada é confiável: mapa de fronteiras com tamanho máximo, formato fixo e leitor do núcleo em cada uma; SQL só com parâmetros (regra nimbus-sql-montado-com-texto).

- **SEG4 \[SIM\]** Nenhum serviço de IA de terceiros no app ou no servidor (regra nimbus-ia-externa).

- **SEG5 \[SIM\]** Senha nunca em texto: o servidor não guarda senha; PIN e senha de link só como resumo Argon2id (regra nimbus-senha-em-texto).

- **SEG6 \[SIM\]** Limite de uso obrigatório em toda rota, aplicado antes de verificar a assinatura; falha de prova castiga (autenticacao::Porteiro::autorizar). Prova C107.

- **SEG7 \[SIM\]** Credenciais com o mínimo necessário: cofre só assina; papel do banco dentro da RLS; token do CI só lê; publicação dividida entre duas pessoas; agente de IA sem acesso a segredos, rede ou publicação.

- **SEG8 \[SIM\]** Manutenção que nunca piora: catraca (verificacao/catraca.py) reprova proteção a menos; unsafe proibido no núcleo; CODEOWNERS e duas aprovações nos arquivos de segurança; ações do CI fixadas pelo hash do commit.

## A6Z4. Sentinela, duas pessoas para toda mudança e procedência

> **Nota de 26/10/2026.** Os itens GOV1 e GOV6 descrevem o regime **alvo** (duas aprovações). O regime **vigente** é
> aprovador único, o dono, por decisão registrada em `docs/DECISAO_APROVADOR_UNICO.md`: o que saiu foi a segunda
> aprovação humana; continuam exigidos pedido de mudança, commit assinado, histórico linear, sem força nem
> apagamento, verificações obrigatórias e nenhum passe livre. O portão `administracao_dupla` (segundo fator, dois
> administradores, equipe de segurança) continua exigido no selo de liberação e continua aberto na organização.

- **SEN1 \[SIM\]** Sentinela na partida e periódica (servidor a cada hora, app a cada abertura): invariantes de segurança (rotas, limites, prazos, esperas, parâmetros de senha) e autotestes criptográficos (cifra, adulteração, chave malformada, assinatura hedged, gerador de números aleatórios) (sentinela::autoverificar). Prova C108.

- **SEN2 \[SIM\]** Qualquer achado trava: o servidor não atende e o app não abre conteúdo (falha fechada); o relatório diz o código, o lugar e como corrigir, sem conteúdo nem segredo (sentinela::Relatorio::texto). Prova C108.

- **SEN3 \[SIM\]** Sentinela do banco na partida e a cada hora: RLS ligada e forçada em toda tabela, papel do app sem poder demais nem posse, sem UPDATE em bloco, sem leitura de registro, nenhuma política aberta, restrição de 48 h presente (servidor/banco/verificar_banco_na_partida.sql). 14 de 14 defeitos plantados no banco pegos. Prova C108.

- **SEN4 \[SIM\]** Conserto nunca automático em produção (seria uma porta: quem descobre o gatilho escolhe a configuração); conserto por gente, com teste e duas aprovações.

- **SEN5 \[SIM\]** Toda rota sem prova de titular tem análise registrada (devolve, expõe, riscos, defesa); rota nova sem análise reprova a catraca; inventário do acesso sem autenticação gerado do código (ACESSO_SEM_AUTENTICACAO.md).

- **GOV1 \[SIM\]** Proteção da branch principal como código (.github/regras/protecao-principal.json): 2 aprovações, dono do código, último envio aprovado por outra pessoa, commits assinados, histórico linear, sem força nem apagamento, verificações obrigatórias, ninguém com passe livre. Aplicada uma vez por um administrador (aplicar_protecao.sh). Prova C109.

- **GOV2 \[SIM\]** Vigia da proteção: fluxo diário compara a proteção ativa com a do código e reprova se alguém afrouxar (vigia-da-protecao.yml), com token só de leitura. Prova C109.

- **GOV3 \[PARCIAL\]** Certificado de procedência (SLSA, in-toto, Sigstore) do pacote-fonte gerado no CI (actions/attest) e conferível com gh attestation verify. Em repositório privado exige GitHub Enterprise Cloud.

- **GOV4 \[PARCIAL\]** Mapa dos 345 requisitos do OWASP ASVS 5.0.0 com situação proposta e motivo de cada linha (MAPA_ASVS_5.0.csv), para o auditor confirmar.

## A6Z5. Selo de liberação: sem checklist completo, nada roda

- **SLO1 \[SIM\]** Selo de liberação: artefato exato (SHA-512), commit, versão e os 17 portões de segurança, todos aprovados; portão ausente, reprovado ou inventado recusa (selo::conferir). Prova C110.

- **SLO2 \[SIM\]** O selo exige assinatura de três chaves diferentes fixadas no código (CI, responsável de segurança e auditor independente de fora da empresa; ver SLO7); a mesma chave não conta duas vezes. Prova C110.

- **SLO3 \[SIM\]** Servidor e apps conferem o selo na partida pela sentinela (sentinela::autoverificar_com_selo); sem selo válido não funcionam. Versão mínima impede voltar a versão com falha conhecida. Prova C110.

- **SLO4 \[SIM\]** Portões de governança conferidos na API do GitHub, não na palavra de ninguém: proteção da branch igual à do código, commit com assinatura verificada, duas aprovações finais de pessoas diferentes do autor (portoes_de_governanca.py, 20 casos testados). Prova C111.

- **SLO5 \[PARCIAL\]** O servidor recusa versão de app sem selo registrado; os apps só vão às lojas com selo completo (implementar no servidor, T3, e no processo de publicação, T13).

- **SLO6 \[SIM\]** Sementes dos assinantes geradas e guardadas offline, nunca em repositório, servidor, nuvem ou variável de ambiente; a do CI só existe no ambiente "liberacao", que exige aprovação humana para rodar (GUIA_DE_LIBERACAO.md).

## A6Z6. Três chaves, administração dupla, atualizações e Termos de Uso

- **SLO7 \[SIM\]** O selo exige três chaves diferentes: CI, responsável de segurança e auditor independente de fora da empresa; duas de três não bastam (selo::MINIMO_DE_ASSINATURAS). O responsável de segurança e o auditor refazem a governança por conta própria antes de assinar. Prova C112.

- **GOV5 \[PARCIAL\]** Administração dupla do GitHub conferida na API a cada liberação e todo dia: segundo fator obrigatório na organização, pelo menos 2 administradores, equipe de segurança com pelo menos 2 membros (portão administracao_dupla). Limite: a API não informa o tipo de segundo fator; chave física conferida por termo assinado e revisão trimestral. Prova C112.

- **GOV6 \[SIM\]** Tarefa T0, antes de qualquer outra: nenhum trabalho novo (T1 em diante) começa antes de a proteção da branch, a administração dupla e o responsável de segurança estarem ativos e conferidos pelos portões. Todo o código escrito antes disso entra no repositório protegido por um único pedido de importação, revisado integralmente por duas pessoas, módulo por módulo, com o termo de revisão assinado (REVISAO_INTEGRAL_T0.md).

- **ATU1 \[SIM\]** Atualizações de dependência nunca automáticas: Renovate sem fusão automática, espera de 14 dias após a publicação (30 dias para bibliotecas de criptografia), revisão de duas pessoas e todos os portões (renovate.json; catraca confere).

- **ATU2 \[SIM\]** Sentinela diária: verificação completa com a base de vulnerabilidades do dia, banco e governança; reprovação impede novas liberações até a correção (sentinela-diaria.yml; catraca confere).

- **JUR1 \[PARCIAL\]** Termos de Uso alinhados ao que o sistema trata de fato: natureza do Serviço informada, excludentes legais, limitação para consumidor pessoa jurídica (CDC art. 51, I) e para contratação empresarial (CC art. 421-A), papéis da LGPD no anexo corporativo, lista exata dos dados tratados, igual à Política e ao código. Revisão de advogado obrigatória (NOTAS_JURIDICAS_DOS_TERMOS.md).

## A6Z7. Metadados: o mínimo que o servidor vê, cada vez menor

- **MET1 \[SIM\]** A caixa de cada relação muda todo dia: endereço recalculado por época a partir do segredo da relação; os dois lados chegam ao mesmo endereço e um observador não liga o de hoje ao de ontem (metadados::caixa_da_epoca). Prova C113.

- **MET2 \[SIM\]** Índice cego no disco: o banco guarda cada bloco sob um índice derivado do endereço com chave do dia, mantida só em memória ou em módulo de hardware e destruída 48 h depois; quem levar o disco depois disso não confere se um endereço conhecido recebeu algo (metadados::ChavesDeIndice). Prova C113.

- **MET3 \[SIM\]** Registros de acesso do Marco Civil (art. 15) selados, com tamanho fixo, para a chave jurídica que não fica no servidor e é dividida entre custodiantes; quem opera o servidor escreve, mas não lê (metadados::selar_registro). Prova C113.

- **MET4 \[SIM\]** Ninguém sozinho liga um metadado a uma pessoa: o 1º retransmissor vê o IP mas não o destino; o servidor vê a caixa do dia mas não o IP nem o remetente; ligar os dois exige combinar empresas diferentes, e com Tor nem assim.

- **MET5 \[PARCIAL\]** O instante da entrega não pode ser zerado sem computação confidencial com atestação remota, recuperação privada de informação ou rede de mistura; decisões abertas, com custos documentados (NimbusOne_Metadados_O_Que_Sobra.md).

## A6Z8. API, ambiente do aparelho, rotação de chaves e disco

- **API1 \[SIM\]** Respostas públicas genéricas: toda falha de autorização dá a mesma resposta (401), sem revelar se a caixa existe nem o motivo; corpo sem dado além do necessário (autenticacao::resposta_publica). Prova C114.

- **API2 \[SIM\]** Sem CORS: nenhum cabeçalho Access-Control-\*; app nativo não manda Origin; navegador só na rota do Envio e só da origem oficial configurada; origem refletida, curinga e "null" recusados (autenticacao::origem_permitida). Prova C114.

- **API3 \[SIM\]** Cabeçalhos da API: sem cache, sem moldura (X-Frame-Options DENY e frame-ancestors none), sem farejamento de tipo, HSTS; a página do Envio também com X-Frame-Options e HSTS (autenticacao::cabecalhos_da_api). Prova C114.

- **API4 \[SIM\]** Nada de token na URL: não existe token de sessão; parâmetro, fragmento ou sufixo na URL fazem a rota não existir. A rota de abrir link do Envio entrou no inventário, com análise e limite. Prova C114.

- **AMB1 \[SIM\]** Diagnóstico local do aparelho (bloqueio de tela, patch de segurança, teclado de terceiros, opções de desenvolvedor, fontes desconhecidas, cópia do sistema), com aviso e como resolver; nada sai do aparelho; nenhum SDK de terceiros (ambiente::avaliar). Prova C115.

- **AMB2 \[SIM\]** Sem bloqueio de tela, o aparelho fica vermelho; patch com mais de 90 dias, teclado de terceiros, opções de desenvolvedor ou fontes desconhecidas limitam a amarelo; vale sempre o mais restritivo entre integridade e ambiente (ambiente::combinar). Prova C115.

- **ROT1 \[SIM\]** Chave de assinatura do servidor diferente a cada dia, certificada por pelo menos 2 de 3 chaves-raiz offline; o app só aceita a do dia corrente, com 10 minutos de folga; a de ontem já não vale (chaves_do_servidor::conferir). Prova C116.

- **ROT2 \[SIM\]** Tabela de rotação de todas as chaves no TRD §4 (mensagem, recebimento, caixa, índice, disco, servidor, registros legais, raízes).

- **MET6 \[SIM\]** Apagamento criptográfico no disco: cada bloco, já cifrado de ponta a ponta, é selado de novo com chave do dia amarrada ao índice; destruída a chave, nem as sobras físicas do disco abrem (metadados::ChavesDeIndice::selar_no_disco). Prova C117.

- **MET7 \[PARCIAL\]** Chave jurídica por mês, com as partes dos custodiantes destruídas depois de 7 meses: o prazo legal de 6 meses do Marco Civil passa a ser cumprido pela destruição da chave (processo; validar com o advogado).

## A6Z9. Ataque vermelho

- **RED1 \[SIM\]** Ataque vermelho permanente: os ataques executados contra o próprio Nimbus (nomes com letras de imitação, valores extremos, martelo aleatório no porteiro) viraram testes que reprovam se a defesa sumir (tests/ataque_vermelho.rs). Prova C118.

- **RED2 \[SIM\]** Nomes de contato: letras de imitação (largura total, matemáticas, circuladas, símbolos com forma de letra) e mistura de alfabetos recusadas; letras gregas e cirílicas parecidas comparadas como latinas (apresentacao::nome_permitido). Prova C118.

- **RED3 \[PARCIAL\]** Quem administra a máquina de produção pode rodar outro programa; o selo protege só o caminho oficial. Fecha com acesso de duas pessoas, imagens assinadas conferidas pelo orquestrador e computação confidencial com atestação (NimbusOne_Relatorio_de_Ataque.md).

## A6Z10. Vigia de sensores durante o uso e autenticidade (nunca ser imitado)

- **VSE1 \[SIM\]** Leitura dos sensores vale no máximo 5 segundos e não pode vir do futuro (tolerância de 2 s); leitura vencida ou adiantada fecha tudo: conteúdo, microfone, câmera e teclado (vigia_de_sensores::decidir).

- **VSE2 \[SIM\]** App fora do primeiro plano não mostra conteúdo, não grava e não filma.

- **VSE3 \[SIM\]** Tela gravada, espelhada ou transmitida, ou tela externa conectada, esconde o conteúdo; captura de tela tirada avisa os outros participantes pelo Escudo de Conversa.

- **VSE4 \[SIM\]** Sobreposição de tela ou acessibilidade não confiável é sinal de ataque ativo: a sessão é bloqueada, as chaves de sessão saem da memória e o PIN é pedido de novo.

- **VSE5 \[SIM\]** Teclado de terceiros nunca recebe o que se digita no Nimbus (iOS: o app recusa teclados de terceiros; Android: teclado fora da lista de confiáveis é bloqueado e o aprendizado do teclado é desligado).

- **VSE6 \[SIM\]** Microfone só em chamada ou gravação de voz e só sem outro app gravando junto; no Android o sistema silencia um dos dois e isso é detectado; no iOS, a interrupção do áudio do Nimbus por outro app conta como outro app gravando.

- **VSE7 \[SIM\]** Áudio saindo para outro aparelho (fone, carro, caixa de som, AirPlay) não bloqueia, mas o app avisa sempre.

- **VSE8 \[SIM\]** Câmera só no uso que pede câmera, sem outro app com a câmera, sem a tela sendo capturada e sem sessão bloqueada. Todo sinal desconhecido conta como risco (na dúvida, fecha).

- **AUT1 \[SIM\]** O app confere, em tempo constante, que o certificado que o assinou é um dos fixados no código (autenticidade::assinatura_do_app_confere); assinatura diferente significa cópia adulterada e as chaves não são abertas.

- **AUT2 \[SIM\]** Todo endereço de site recebido numa mensagem é extraído com limites (64 KiB de texto, 100 endereços, 253 caracteres por domínio), sem pânico com qualquer entrada (fuzzing).

- **AUT3 \[SIM\]** Endereço parecido com um oficial é acusado antes de abrir: até 2 letras de diferença, letras de outro alfabeto, "rn" no lugar de "m", o oficial escondido dentro de outro domínio, a marca usada em outro domínio, codificação "xn--" e usuário falso antes de "@" (caso Privnote, 2020).

- **AUT4 \[SIM\]** A lista de domínios oficiais vem da configuração assinada do app, nunca da rede.

- **AUT5 \[SIM\]** Nunca existe versão web que baixe o código do servidor a cada acesso: o servidor poderia entregar outro código.

## A6Z11. Proteção do usuário sem expor nada (exigências da App Store)

- **BLQ1 \[SIM\]** Bloquear contato: a lista fica só no aparelho, dentro do contêiner cifrado; o servidor nunca sabe quem você bloqueou (protecao_do_usuario::ListaDeBloqueio).

- **BLQ2 \[SIM\]** Tudo o que vem de quem está bloqueado (mensagem, chamada, arquivo, convite, atualização de perfil) é descartado em silêncio, antes de qualquer resposta.

- **BLQ3 \[SIM\]** Quem foi bloqueado não recebe recibo, toque nem erro, e não consegue saber que foi bloqueado.

- **BLQ4 \[SIM\]** A lista é guardada sempre com o mesmo tamanho (1.024 vagas); lista adulterada (repetida, com lixo na sobra ou contagem impossível) é recusada sem quebrar o app.

- **DEN1 \[SIM\]** Denúncia da CONTA, nunca do conteúdo: o NimbusChat não deixa copiar, encaminhar nem exportar mensagens, e a denúncia não abre exceção.

- **DEN2 \[SIM\]** O pacote leva só a impressão digital da conta denunciada e o motivo; nenhuma mensagem, nenhum texto livre, nenhum dado de quem denuncia.

- **DEN3 \[SIM\]** Só por iniciativa da pessoa e com consentimento explícito; nunca automática.

- **DEN4 \[SIM\]** Cifrado só para a chave da moderação (configuração assinada do app) e sempre do mesmo tamanho.

- **DEN5 \[SIM\]** Denunciar também bloqueia a conta denunciada.

- **APG1 \[SIM\]** Apagar a conta dentro do app (regra 5.1.1(v) da App Store): pedido assinado pela chave da própria conta, válido por 5 minutos e de uso único.

- **APG2 \[SIM\]** Exige o PIN real. Sob o PIN de coação o pedido não sai, nada é apagado e o app mostra a mesma falha genérica de rede (aprovado pelo dono em 06/10/2026).

- **APG3 \[SIM\]** O servidor só aceita pedido da própria conta, dentro do prazo, nunca do futuro e nunca repetido; então apaga caixa, pré-chaves e avisos de entrega.

- **APG4 \[SIM\]** No aparelho, apagar a conta destrói as chaves: o que restar na memória do celular fica ilegível para sempre (apagamento criptográfico).

- **HIS17 \[SIM\]** Retirar mensagem: só quem a escreveu; antes de lida, sempre; depois de lida, salvo se algum participante desligar (basta um, para guardar a prova de um assédio). Foto da tela tirada por outro aparelho nunca pode ser retirada.

- **HIS18 \[SIM\]** O único aviso que existe é "visualizada": opcional, recíproco e desligado por padrão. Não existem "entregue", "digitando", "online" nem "visto por último" (decisão do dono em 06/10/2026).

## A7. Escudo de Conversa: se a outra pessoa estiver gravando

Como os dois lados da conversa usam o NimbusOne, o app de cada um vigia o próprio aparelho e informa o estado aos demais participantes, de forma cifrada e assinada, a cada poucos segundos. Se algo mudar em qualquer aparelho, a conversa é protegida em todos ao mesmo tempo.

| **Situação no aparelho de qualquer participante**                                    | **O que acontece**                                                                                                                                                                                    |
|--------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Gravação de tela, espelhamento ou transmissão (iPhone)**                           | A conversa é borrada na hora em todos os aparelhos; aviso: “Captura de tela ativa no aparelho de \[nome\] — conversa pausada”. A gravação registra só o borrão. Volta ao normal quando a captura para |
| **App gravador de tela (Android)**                                                   | A gravação sai preta: o app usa a proteção de tela do sistema, que bloqueia captura por outros apps. Se o sistema sinalizar a captura, também borra e avisa                                           |
| **Print de tela (iPhone)**                                                           | A camada segura faz o print sair preto; o sistema informa o print depois que ele ocorre: todos são avisados e conteúdos de visualização única são destruídos                                          |
| **Gravação do áudio de uma chamada**                                                 | Android: o app bloqueia a captura do seu áudio por outros apps. iPhone: gravação de tela ativa pausa o áudio e avisa. Gravador externo (outro aparelho) não é detectável                              |
| **Aparelho fica amarelo ou vermelho (acessibilidade suspeita, sobreposição, root)**  | Conversa borrada para todos; aviso de “ambiente inseguro” sem revelar detalhes do aparelho alheio                                                                                                     |
| **Participante some (sem estado por 10 segundos) ou estado falsificado ou repetido** | Conversa borrada — na dúvida, fecha                                                                                                                                                                   |
| **Câmera de outro aparelho apontada para a tela**                                    | Impossível detectar ou impedir. Mitigação: visualização única, Conversa Blindada e prazos curtos. Marca d’água invisível foi descartada: seria um rastro de quem recebeu                              |

- **EC0 \[SIM\]** Modo “Conversa Blindada” (escolhido por quem inicia): conteúdo só aparece enquanto todos os participantes estiverem verdes e sem captura; mensagens novas ficam retidas até isso ocorrer.

## A7C. Blindagem de anonimato — o que o app nunca toca

Cifrar esconde o conteúdo. Para blindar de verdade, é preciso esconder também os rastros em volta: quem fala com quem, a que horas, de onde e por quanto tempo. Estas proibições valem para os dois produtos e são verificadas automaticamente pela varredura de código.

- **ANO1 \[SIM\]** O app nunca pede nem lê: localização, agenda telefônica do sistema, SMS, identificador de aparelho, identificador de publicidade, lista de apps instalados. A varredura reprova qualquer uma dessas chamadas.

- **ANO2 \[SIM\]** Sem telefone e sem e-mail em nenhuma etapa: cadastro, entrada, recuperação ou suporte. Nada de código por SMS.

- **ANO3 \[SIM\]** Tráfego de cobertura (código pronto, módulo trafego): o app envia um pacote a cada intervalo fixo; quando não há mensagem, envia enchimento indistinguível. Para quem observa a rede, mensagens de texto, voz curta e silêncio ficam iguais. Custo declarado: consumo de dados e bateria; haverá modo econômico.

- **ANO3b \[PARCIAL\]** Alcance honesto: o tráfego de cobertura não esconde envios grandes (fotos pesadas, vídeos, documentos): o volume aparece na rede, ainda que cifrado. Opção “envio discreto”: o arquivo sobe devagar dentro da batida constante, sem pico — mais lento, mas invisível no ritmo.

- **ANO4 \[SIM\]** Todo envio tem tamanho padronizado em faixas fixas: o volume não denuncia se foi um “ok” ou uma foto.

- **ANO5 \[SIM\]** O servidor nunca vê o IP (relay independente, ARQ2) e nunca guarda quem falou com quem; o remetente é oculto até para o servidor.

- **ANO6 \[SIM\]** Notificações com texto genérico e identificador rotativo; modo sem notificação para quem exige o máximo.

- **ANO7 \[SIM\]** PIN de coação e cofre-isca (código pronto, módulo coacao): sob ameaça, o cliente entrega o segredo de coação e abre um cofre com conteúdo inofensivo, sem nada aparecer na tela; o cofre real é bloqueado e as outras sessões, cortadas. A comparação é em tempo constante, para o tempo de resposta não denunciar qual segredo foi digitado.

- **ANO8 \[SIM\]** Os dois cofres têm tamanho de dados padronizado: o volume não revela que existe um cofre real além da isca.

- **ANO9 \[PARCIAL\]** Padrão de acesso: ao baixar blocos do servidor cego, o servidor vê quais identificadores são pedidos juntos e quando, e poderia ligá-los a uma mesma sessão. Mitigação: todos os pedidos pelo relay que oculta o IP (ARQ2), leituras de cobertura misturadas às reais e sessões curtas. Recuperação privada de informação (PIR) fica como evolução, pelo custo computacional.

- **ANO10 \[SIM\]** Rede: Automático por padrão (rota rápida em duas etapas no dia a dia, Tor sozinho nos momentos de identidade, nas conversas blindadas e quando a rede bloqueia), Sempre Tor ou Rápido (sem Tor) à escolha do usuário (NET1–NET14). Nenhuma consulta DNS em aberto.

- **ANO11 \[SIM\]** Isolamento: pelo Tor, cada lote de pedidos por um circuito diferente; pela rota rápida, cada lote por uma conexão nova ao retransmissor da 1ª etapa.

- **ANO12 \[SIM\]** Sem login nos pedidos de leitura: cada lote é pago com ficha anônima de uso único (Privacy Pass), não com a identidade da conta.

- **ANO13 \[SIM\]** Endereços derivados e lotes de tamanho fixo com cobertura embaralhada (código pronto, módulo acesso): o servidor recebe pedidos que não consegue ligar entre si nem ao dono; o aparelho nem precisa guardar a lista do que existe.

- **ANO14 \[PARCIAL\]** Chamadas de voz e vídeo não passam pelo Tor (a rede Tor não transporta o tipo de tráfego de tempo real das chamadas): vão por retransmissores de terceiros independentes, que veem o IP mas não o conteúdo; nem nós nem a outra ponta veem o IP.

## A8. Clonagem e apps gravadores: o que dá para impedir

- **CG1 \[SIM\]** Clonagem da conta ou do aparelho: a chave do aparelho não sai do chip de segurança; o clone não autentica e dispara a derrubada de todas as sessões, com aviso ao dono (código pronto, módulo sessoes).

- **CG2 \[SIM\]** Podemos impedir o funcionamento do NimbusOne: diante de gravador, sobreposição, acessibilidade suspeita, root ou app adulterado, o NimbusOne pausa, borra ou bloqueia (verde/amarelo/vermelho) e avisa os outros participantes.

- **CG3 \[NÃO\]** Não podemos impedir o funcionamento do outro app: iOS e Android não permitem que um app encerre, desinstale ou examine outro. Quem promete isso está enganando. O efeito prático é equivalente: com o NimbusOne fechado ou borrado, o gravador não captura nada útil.

## A9. O que aprendemos com a Hypori (usada pelo governo e pelas Forças Armadas dos EUA)

Fatos publicados pela própria Hypori: o aparelho pessoal só exibe a tela e recebe toques; nenhum dado da organização é gravado nele; o modelo parte do princípio de que o aparelho pode estar comprometido; componentes de proteção de chaves com validação FIPS; certificação NIAP Common Criteria; autorização FedRAMP High e DoD IL5; listagem no programa CSfC da NSA; operação em AWS e Azure.

- **HY1 \[SIM\]** Adotamos: premissa formal de que o aparelho do cliente pode estar comprometido (toda decisão sensível é reconfirmada no servidor e exige biometria).

- **HY2 \[SIM\]** Adotamos: “só pixels” no Cloud Phone — nenhum arquivo de terceiros chega ao aparelho sem ação explícita do cliente.

- **HY3 \[SIM\]** Adotamos: edição Enterprise/FIPS com suíte AES-256-GCM (código pronto) em módulo com validação FIPS 140-3, autenticação corporativa por certificado digital (PKI) e rota de certificações (Common Criteria e equivalentes). Canal natural de venda: Defensys.

- **HY4 \[SIM\]** Adotamos: infraestrutura em mais de um provedor de nuvem, para escala e para não depender de um só fornecedor.

- **HY5 \[SIM\]** Vamos além no desenho: (1) a Hypori opera sob controle administrativo da organização cliente; o NimbusOne consumidor é desenhado para que nem o operador acesse (chave dividida + computação confidencial); (2) apps Nimbus nunca são decifrados no servidor (modo efêmero), enquanto streaming de pixels exige decifrar na nuvem; (3) pós-quântico desde o início; (4) Escudo de Conversa, anonimato e registro público de chaves.

- **HY6 \[NÃO\]** Onde a Hypori está à frente hoje: anos de operação, auditorias e certificações governamentais. “Melhor que eles” vale para o desenho; em maturidade, a distância só se fecha com as provas deste manual e certificações. Comparações públicas exigem verificar o estado atual da Hypori antes (validar).

**PARTE B — ARQUITETURA (para o time técnico)**

## B1. Componentes

- **CO1 \[SIM\]** Nimbus OS (cliente): app único — iOS, Android, computador e web. Núcleo em Rust compartilhado (cripto, protocolo, sincronização) via UniFFI; interface nativa (SwiftUI, Jetpack Compose, Tauri, WebAssembly). Tela inicial unificada com apps Nimbus e apps do Cloud Phone lado a lado.

- **CO2 \[SIM\]** Cofre Nimbus (servidor): guarda só blocos cifrados com tamanho padronizado; sem estado identificável; replicado em 2+ países.

- **CO3 \[SIM\]** Relay NimbusChat: em memória (RAM-only), sem disco, sem swap, sem logs; entrega envelopes com remetente oculto.

- **CO4 \[SIM\]** Relay de rede independente (Oblivious HTTP, RFC 9458): vê o IP, não vê conteúdo; a Nimbus vê conteúdo cifrado, não vê o IP. Opção Tor/mixnet.

- **CO5 \[PARCIAL\]** Cloud Phone: uma máquina virtual confidencial (AMD SEV-SNP ou Intel TDX) por cliente, Android reforçado construído de forma reproduzível; vídeo e toque por WebRTC cifrado terminando dentro da máquina. Limites: apps só-ARM exigem tradução em x86; licença do Google para Play Store (validar); custo por cliente a medir.

- **CO6 \[SIM\]** Custodiante da meia-chave (KS1): serviço mínimo, de outra empresa e outra jurisdição, que só libera a meia-chave mediante atestação válida + autorização assinada pelo aparelho do cliente.

- **CO7 \[SIM\]** Registro público de chaves e de imagens: toda chave de aparelho e toda imagem do Cloud Phone entram num log só-de-acréscimo auditável.

## B2. Modo efêmero dos apps Nimbus

- **EF1 \[SIM\]** Nenhum conteúdo gravado no aparelho: dados baixados cifrados, abertos em memória não gerenciada, zerados ao fechar o app, ao bloquear a tela e após 60 s em segundo plano.

- **EF2 \[SIM\]** Única coisa persistente no aparelho: a chave do aparelho, dentro do chip de segurança (Secure Enclave / StrongBox / TPM), inextraível.

- **EF3 \[PARCIAL\]** Cache offline opcional só no NimbusOne (OFF3, OFF4): cifrado, com prazo e limite de tamanho; desligado por padrão; desligado sempre no Álbum Cofre; itens com validade só aparecem com hora confirmada pelo servidor (OFF2). O NimbusChat não tem cache offline.

- **EF4 \[SIM\]** Câmera e microfone: captura em memória, cifra imediata, envio direto ao cofre; nunca a Fototeca, nunca gravador do sistema; sem GPS/EXIF.

## B2B. Não deixar rastro — requisitos detalhados

- **RZ1 \[SIM\]** Conteúdo nunca é gravado em disco em claro nem em área compartilhada (galeria, Downloads, Arquivos, MediaStore). Arquivos inevitáveis: proteção completa do sistema (NSFileProtectionComplete) e fora de qualquer backup.

- **RZ2 \[SIM\]** Sem cache do sistema para conteúdo: sem URLCache compartilhado, sem NSCache/LruCache com conteúdo, sem cache de imagem em disco. Miniaturas só na memória, recriadas a cada abertura. A varredura reprova o uso desses caches.

- **RZ3 \[SIM\]** Memória: conteúdo em claro só em buffers não gerenciados (sodium_malloc), zerados logo após o uso; nunca em String/Data comuns, que podem ser paginadas. Zerar ao fechar, ao bloquear a tela e após 60 s em segundo plano.

- **RZ4 \[SIM\]** Teclado: teclados de terceiros bloqueados; no sistema, aprendizado personalizado desligado (iOS: autocorreção desligada nos campos sensíveis; Android: IME_FLAG_NO_PERSONALIZED_LEARNING), para o que se digita não vazar para o dicionário do aparelho.

- **RZ5 \[SIM\]** Sistema operacional não indexa nem sugere conteúdo do app: sem Spotlight, Siri, sugestões, atalhos ou pré-visualização em notificação. A varredura reprova essas APIs.

- **RZ6 \[SIM\]** Alternador de apps: tela coberta ao sair (sem miniatura do conteúdo); área de transferência limpa ao sair e ao trocar de app.

- **RZ7 \[SIM\]** Ao sair da conversa, apagar = destruir a chave (memória flash não se sobrescreve de forma confiável); o apagamento criptográfico (SV2) garante que o que restou fica ilegível.

- **RZ8 \[PARCIAL\]** Limite: o iOS e o Android mantêm buffers internos de câmera, áudio e tela fora do controle do app. Mitigação: captura própria em memória e cifra imediata; não há como zerar buffers do sistema.

- **RZ9 \[SIM\]** Notificações locais e remotas só com texto genérico e identificador rotativo; ao apagar ou vencer um item, cancelar a notificação agendada (UNUserNotificationCenter.removePendingNotificationRequests / NotificationManager.cancel). Varredura: regra nimbus-notificacao-com-conteudo.

- **RZ10 \[SIM\]** Arquivo importado de fora (compartilhar, arquivos do sistema): cifrar imediatamente e apagar a cópia que o sistema deixa em tmp/Inbox. Varredura: regra nimbus-importacao-deixa-copia.

- **RZ11 \[SIM\]** Relatórios de falha: sem ferramenta de terceiros; conteúdo e chaves só em memoria::Segredo (Debug mostra só o tamanho); release com panic=abort; mensagens de erro sem conteúdo (regra nimbus-conteudo-em-mensagem-de-erro).

- **RZ12 \[SIM\]** Memória: mlock nos buffers de segredo onde o sistema permite (computador); iOS e Android não fazem swap para disco. Edição de computador exige FileVault/BitLocker por causa da hibernação.

- **RZ13 \[SIM\]** Envio para outros apps pelo sistema (folha de compartilhar, Abrir em...) proibido. Únicas saídas: o Nimbus Envio (para quem não tem o app) e o componente ExportacaoAutorizada, só para arquivo com autorização assinada do autor (LIB). Mensagem nunca sai. Regra nimbus-compartilhar-para-fora.

- **MTA1 \[SIM\]** Nome do arquivo, tipo, data e pasta ou álbum são cifrados junto com o conteúdo, num único bloco de tamanho padronizado (código pronto, módulo pacote).

- **MTA2 \[SIM\]** No aparelho, o nome só aparece decifrado na memória; arquivos temporários, quando inevitáveis, recebem nome aleatório.

- **MTA3 \[SIM\]** O servidor não guarda nome, tamanho real, data, pasta, dono, conversa nem quantidade de itens de ninguém.

## B2C. Validade automática — requisitos técnicos

- **VAL1 \[SIM\]** Todo item (compromisso, nota, contato, arquivo, foto, item do cofre) aceita validade opcional (módulo validade).

- **VAL2 \[SIM\]** Agenda: referência = fim do compromisso; demais: referência = criação. Validade máxima 10 anos; zero é recusado.

- **VAL3 \[SIM\]** Gravação com historico::guardar amarrado ao instante de apagar (janela de tempo); destruição com destruir_janelas_vencidas a cada abertura do app e em segundo plano quando o sistema permitir.

- **VAL4 \[SIM\]** A interface esconde o item no instante exato (validade::deve_esconder), antes da destruição criptográfica.

- **VAL5 \[SIM\]** Ao vencer: cancelar lembretes agendados (RZ9) e remover o item de qualquer cache em memória.

- **VAL6 \[SIM\]** Alterar a validade grava o item de novo, na janela nova; o bloco antigo morre na janela antiga.

- **VAL7 \[SIM\]** Chaves de janela sincronizadas entre os aparelhos do dono sempre cifradas; nunca em backup do sistema (RZ1).

- **VAL8 \[PARCIAL\]** Aparelho desligado por muito tempo: a chave da janela vencida é destruída na primeira abertura; enquanto isso, o servidor já esconde e recusa o bloco vencido.

## B2D. Nimbus Envio e Sala Blindada — requisitos técnicos

- **ENV1 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **ENV2 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **ENV3 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **ENV4 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **ENV5 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **ENV6 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **ENV7 \[NÃO\]** Retirado em 06/10/2026 por decisão do dono: não existe link nem página web; envio só dentro do app, e quem recebe instala o Nimbus (Registro, item 72).

- **SB1 \[SIM\]** Sala Blindada: o documento vive na máquina virtual confidencial do remetente; quem recebe vê só pixels, sempre dentro do app Nimbus, por WebRTC cifrado. Não existe acesso pelo navegador (somente app, decisão de 06/10/2026).

- **SB2 \[SIM\]** Sem marca d’água, nunca (LIB16): nada na página identifica quem vê.

- **SB3 \[SIM\]** Sem download, sem impressão, sem copiar texto: a página nunca recebe o arquivo, só a imagem.

- **SB4 \[SIM\]** Acesso só pelo app, por convite para um contato Nimbus, com prazo; revogação encerra a sessão ao vivo. Quem não tem o Nimbus precisa instalar o app.

- **SB5 \[PARCIAL\]** Limite: foto da tela por outro aparelho é possível e não pode ser detectada. Para o que não pode vazar em hipótese alguma: nível máximo (SB7) e acesso curto. A proteção do documento dentro da máquina é por hardware (CP8).

- **SB6 \[NÃO\]** Retirado em 06/10/2026: não existe Sala Blindada no navegador (somente app).

- **SB7 \[SIM\]** Sala Blindada nível máximo: só abre dentro de um app Nimbus, onde valem o bloqueio de print e gravação e o Escudo. Exige que o destinatário instale o app. Recomendado para o que não pode vazar em hipótese alguma.

## B2G. Notificações, discrição e superfícies do sistema — requisitos técnicos

- **NOT1 \[SIM\]** Cadência fixa (módulo notificacao): a cada janela, exatamente uma notificação por aparelho ativo, real ou de cobertura, cifrada para o aparelho e do mesmo tamanho (cerca de 1,9 KB, dentro do limite de 4 KB do serviço da Apple — validar).

- **NOT2 \[SIM\]** No aparelho, a extensão de notificação decifra: real → texto genérico “Nova atividade”; cobertura → descarte em silêncio.

- **NOT3 \[PARCIAL\]** iPhone: o descarte em silêncio exige a permissão de filtragem da Apple (pedido formal; concedida caso a caso). Com ela, a cadência fixa completa; sem ela, NOT5.

- **NOT4 \[SIM\]** Janela padrão de 15 minutos para mensagens; chamadas usam janela imediata (o usuário precisa atender), com o custo declarado de revelar o horário da chamada.

- **NOT5 \[SIM\]** Sem a permissão da Apple: entrega agrupada em janelas de 15 minutos (o horário exato some; a existência de atividade na janela fica visível), ou modo sem notificação.

- **NOT6 \[SIM\]** Android: sem Google — conexão própria de baixo consumo ou padrão aberto UnifiedPush; medir bateria.

- **NOT7 \[SIM\]** Nenhum texto do usuário em notificação, nunca (varredura reprova).

- **DIS1 \[SIM\]** iPhone: ícones alternativos (setAlternateIconName) e nome curto neutro definido na publicação; orientação no app para o recurso do sistema “Exigir Face ID” e “Ocultar”.

- **DIS2 \[SIM\]** Android: troca de ícone e nome por activity-alias; fachada funcional (ex.: calculadora que abre o app com código) só na distribuição direta e declarada na loja; Espaço Privado do Android 15.

- **DIS3 \[SIM\]** Android: FLAG_SECURE, importantForContentCapture=noExcludeDescendants e importantForAutofill=noExcludeDescendants em todas as telas de conteúdo.

- **DIS4 \[SIM\]** iPhone: conteúdo em camada segura, writingToolsBehavior = .none, sem App Intents nem NSUserActivity com conteúdo.

- **DIS5 \[SIM\]** Descritor de cobrança neutro com o nome jurídico da empresa; tela no app explicando como a cobrança aparece; vale-assinatura em dinheiro.

## B2E. Publicação, registro público e servidor oculto — requisitos técnicos

- **BC5 \[SIM\]** Registro público (módulo registro): árvore de Merkle conforme a RFC 9162; folhas para cada chave de aparelho, cada imagem do Nimbus OS e cada versão do app; provas de inclusão verificadas pelo app. Próximo passo: prova de consistência (RFC 9162, 2.1.4) e troca de raízes entre clientes, contra visões diferentes do registro.

- **BL4b \[SIM\]** Publicação de versão (módulo aprovacao, tipo Publicacao): mínimo 3 de 5 assinaturas distintas, incluindo o auditor externo; o app só instala atualização cujo hash esteja no registro público e com as assinaturas exigidas.

- **BC6 \[PARCIAL\]** Servidor opcional como serviço onion da rede Tor, para esconder a localização; retransmissores operados por terceiros independentes. Medir latência antes de ligar por padrão.

## B3. Apps Nimbus — requisitos

- **CH1 \[SIM\]** NimbusChat: texto, emojis, figurinhas e GIFs próprios cifrados, reações, respostas, enquetes, editar, apagar para todos, grupos (MLS, RFC 9420), protocolo do Signal (PQXDH + Triple Ratchet) via libsignal (licença AGPL — parecer).

- **CH2 \[SIM\]** Temporárias: Modo A após abrir (inclui visualização única) e Modo B após enviar, combináveis; destruição no servidor, no aparelho e da chave; prazo pelo relógio do servidor.

- **CL1 \[SIM\]** Chamadas Nimbus (voz e vídeo, 1:1 e em grupo com SFrame, RFC 9605), cifradas ponta a ponta, sem número de telefone e sem registro no sistema. Sem ponte para a rede telefônica comum (decisão TF1).

- **CT1 \[SIM\]** Contatos e Agenda: cifrados, sincronizados por CRDT, importação/exportação vCard e iCalendar; lembretes gerados no aparelho.

- **NT1 \[SIM\]** Notas e Arquivos: cifrados; busca feita no aparelho; compartilhamento por link cifrado com prazo.

- **AL1 \[SIM\]** Álbum Comum: cifrado, miniaturas só em memória, desbloqueio por biometria.

- **AL2 \[SIM\]** Álbum Cofre: segunda chave (senha própria + biometria ou chave física FIDO2), itens com chave individual, anti-print e anti-gravação, bloqueio em 30 s, PIN de coação que abre cofre-isca, tamanho de dados fixo (negação plausível).

- **PW1 \[SIM\]** Senhas: cofre cifrado, preenchimento automático, códigos de uso único, chaves de passe.

- **ND1 \[SIM\]** NimbusDrop e Nimbus Ceremony: troca por proximidade e cadastro presencial de aparelhos, assinatura híbrida. Arquivo recebido por proximidade chega lacrado e só abre depois de o servidor entregar a chave (ND2 a ND5).

- **AR1 \[PARCIAL\]** Anti-captura: camada segura contra print e detecção de captura e gravação (técnica não oficial no iOS; patente Confide — parecer). Sem marca d’água: ela seria um rastro de quem recebeu. Foto de outro aparelho é impossível de impedir.

## B3A. NimbusChat — o chat mais privado que podemos provar

Nenhum produto pode se declarar “o melhor do universo”; o que se pode é adotar o protocolo mais analisado do mundo e ir além dele onde é possível provar. Base e acréscimos:

- **NC1 \[SIM\]** Herdado (o mais analisado publicamente): protocolo do Signal via libsignal — PQXDH + Triple Ratchet; remetente oculto; grupos MLS (RFC 9420). Não reescrever: usar a biblioteca.

- **NC2 \[SIM\]** Acréscimo 1 — identidade: conta sem número de telefone, e-mail ou documento; (o Signal passou a testar o mesmo em 2026 — ver A0B) contato só por convite, QR ou Nimbus Ceremony.

- **NC3 \[SIM\]** Acréscimo 2 — rede: o servidor não vê o IP (Oblivious HTTP por relay independente, RFC 9458); chamadas sempre por relay.

- **NC4 \[SIM\]** Acréscimo 3 — operador vigiado: registro público de chaves; aparelho fantasma inserido pelo próprio operador é detectado e exibido.

- **NC5 \[SIM\]** Acréscimo 4 — aparelho: modo efêmero (nada gravado), anti-captura, verificação do aparelho (DV), PIN de coação, bloqueio automático.

- **NC6 \[SIM\]** Acréscimo 5 — temporárias Modo A/B com prazo assinado pelo servidor (código pronto, módulo expiracao) e destruição da chave.

- **NC7 \[SIM\]** Acréscimo 6 — tamanho: todo envelope preenchido em faixas fixas (código pronto, módulo envelope).

- **NC8 \[SIM\]** Acréscimo 7 — pagamento anônimo (Privacy Pass) e zero SDK de terceiros.

- **NC9 \[PARCIAL\]** Comparações públicas com Signal, WhatsApp ou outros só depois de verificar o estado atual de cada um (validar) — os concorrentes evoluem e uma comparação errada destrói credibilidade.

- **NC11 \[SIM\]** Ferramentas de mensagem completas: áudio (mensagem de voz com forma de onda), foto e vídeo (câmera própria), vídeo circular curto, documentos e arquivos, figurinhas e figurinhas animadas próprias, GIFs de pacote próprio, emojis, reações, responder, citar, menções, enquetes, editar, apagar para todos, fixar, marcar, rascunho, busca local, transcrição de áudio no próprio aparelho (sem serviço externo).

- **NC12 \[SIM\]** Visualização única para todo tipo de conteúdo: abre uma vez, nunca é guardada, encaminhada, copiada ou citada, e é destruída ao fechar (módulos politica e liberacao).

- **NC13 \[SIM\]** Temporizador em dois eixos combináveis: por conversa e por mensagem; vence o mais restritivo (módulo politica).

## B3B. Escudo de Conversa — requisitos técnicos

- **EC1 \[SIM\]** iOS: observar UIScreen.isCaptured / UITraitCollection.sceneCaptureState (gravação, espelhamento, AirPlay) e userDidTakeScreenshotNotification; conteúdo renderizado em camada segura.

- **EC2 \[SIM\]** Android: FLAG_SECURE em todas as janelas de conteúdo; callbacks de detecção de print (Android 14+) e de gravação de tela (versões recentes — validar nível de API); toques recusados com janela obscurecida.

- **EC3 \[SIM\]** Áudio: Android com captura de reprodução bloqueada (allowAudioPlaybackCapture=false / ALLOW_CAPTURE_BY_NONE); iOS pausa o áudio da chamada enquanto houver captura de tela.

- **EC4 \[SIM\]** Estado de segurança assinado pela chave do aparelho, enviado pelo canal cifrado a cada 3 s; validade de 10 s contada a partir da chegada neste aparelho; carimbo mais de 30 s no futuro é recusado; número de sequência contra repetição (código pronto, módulo escudo).

- **EC5 \[SIM\]** Decisão local e simultânea em cada aparelho: borra e informa quem causou e por quê; nunca revela a lista de apps de ninguém.

- **EC6 \[SIM\]** Sem marca d’água em conversa, nem visível nem invisível (LIB16): nada identifica quem viu. A proteção contra foto da tela é a visualização única, a Conversa Blindada e os prazos curtos.

- **EC7 \[SIM\]** Conversa Blindada: servidor retém mensagens novas até receber de todos os destinatários estado verde atestado.

- **EC8 \[PARCIAL\]** Limite: cliente adulterado poderia mentir o estado; mitigado por atestação obrigatória na conexão e pela recusa do servidor a clientes não atestados. Atacante com recursos de nível estatal pode contornar.

- **EC9 \[NÃO\]** Câmera ou gravador de outro aparelho: indetectável.

## B3C. Chamadas e voz — requisitos técnicos

- **VC1 \[SIM\]** Sinalização de chamada por notificação comum (UserNotifications), NÃO por PushKit/CallKit, para a chamada não entrar em Recentes; áudio em segundo plano ativado ao aceitar.

- **VC2 \[PARCIAL\]** Alternativa LiveCommunicationKit (iOS 17.4+) dá áudio de chamada sem a interface de telefone do CallKit; avaliar se mantém a chamada fora de Recentes (validar). Se registrar, usar a rota VC1.

- **VC3 \[SIM\]** Android: FLAG_SECURE na tela da chamada; captura do áudio por outros apps bloqueada (ALLOW_CAPTURE_BY_NONE).

- **VC4 \[SIM\]** Modulador de voz (módulo voz): janelas de 2048 com sobreposição de 75%, aplicado antes da cifra; predefinições e ajuste de -12 a +12 semitons.

## B3D. Blindagem de anonimato — requisitos técnicos

- **BA1 \[SIM\]** Permissões do app limitadas a câmera, microfone e notificações, sob pedido explícito e revogáveis. Nunca: localização, contatos do sistema, SMS, telefone, calendário do sistema.

- **BA2 \[SIM\]** Tráfego de cobertura (módulo trafego): intervalo padrão de 500 ms com pacote de enchimento; o servidor descarta o enchimento sem registrar. Medir bateria e dados e definir perfis (normal e econômico).

- **BA3 \[SIM\]** PIN de coação (módulo coacao): dois verificadores no aparelho, derivados por Argon2id (RFC 9106, 64 MiB), comparação em tempo constante; PIN real, de coação e errado levam o mesmo tempo (prova C80).

- **BA4 \[SIM\]** Nada de identificadores persistentes: sem identificador de aparelho ou de publicidade, sem impressão digital do aparelho; a chave do aparelho serve só para autenticar e fica no chip.

- **BA5 \[SIM\]** Conteúdo fora de qualquer backup do sistema (iCloud e Google Backup) e fora da indexação do sistema.

- **BA6 \[SIM\]** Registro do servidor sem IP, sem identificador de conta e sem tamanho de mensagem; só contadores agregados.

## B3E. Edições

- **FP1 \[SIM\]** Consumidor: anônima, suíte XChaCha20-Poly1305, sem identidade.

- **FP2 \[SIM\]** Enterprise/FIPS: suíte AES-256-GCM + ML-KEM-1024 + ML-DSA em módulo criptográfico com validação FIPS 140-3 (validar disponibilidade do módulo com ML-KEM), login por certificado PKI e MFA, painel de administração que gerencia acessos sem acesso a conteúdo.

## B4. Identidade, recuperação e troca de aparelho

- **ID1 \[SIM\]** Conta sem telefone, e-mail ou documento; chave do aparelho em hardware; atestação do app a cada conexão; clone não autentica.

- **ID2 \[SIM\]** Tentativa sem credencial válida (quem só sabe o identificador da conta) é recusada e contada, sem derrubar ninguém. Com credencial válida, sessão paralela, aparelho não registrado ou app sem atestação derrubam todas as sessões, avisam o dono e registram o histórico “Fui eu / Não fui eu” cifrado só para ele; painel da empresa só com contagens.

- **RC1 \[SIM\]** Recuperação (mínimo dois métodos obrigatórios): frase de 24 palavras; chave física FIDO2; guardiões (3 de 5, Shamir); Nimbus Ceremony a partir de outro aparelho.

- **RC2 \[SIM\]** Aparelho roubado: revogação imediata a partir do novo; a meia-chave do Cloud Phone do aparelho antigo é invalidada no custodiante.

- **RC3 \[NÃO\]** Perda de todos os aparelhos e de todos os métodos = dados irrecuperáveis. É a prova de que a empresa não tem a chave.

## B5. Pilha de padrões

| **Função**               | **Padrão**                                                   | **Referência**                    |
|--------------------------|--------------------------------------------------------------|-----------------------------------|
| **Troca de chaves**      | X25519 + ML-KEM-1024 híbrido                                 | NIST FIPS 203                     |
| **Assinaturas**          | Ed25519 + ML-DSA-87; versões do app com SLH-DSA              | NIST FIPS 204 e 205               |
| **Cifra**                | XChaCha20-Poly1305 (256 bits)                                | libsodium / RustCrypto            |
| **Derivação e senhas**   | HKDF-SHA-512; Argon2id                                       | IETF RFC 9106                     |
| **Conversa e grupos**    | PQXDH + Triple Ratchet; MLS                                  | Signal; IETF RFC 9420             |
| **Mídia em grupo**       | SFrame                                                       | IETF RFC 9605                     |
| **Ocultação de IP**      | Oblivious HTTP                                               | IETF RFC 9458                     |
| **Pagamento anônimo**    | Privacy Pass; assinatura cega RSA                            | IETF RFC 9576–9578; RFC 9474      |
| **Login**                | Chaves de passe FIDO2/WebAuthn; OPAQUE para PIN              | W3C; IETF (validar número da RFC) |
| **Cloud Phone**          | AMD SEV-SNP / Intel TDX com atestação remota                 | Documentação AMD/Intel            |
| **Cadeia de construção** | Builds reproduzíveis, SLSA nível 3, Sigstore, SBOM CycloneDX | slsa.dev; cyclonedx.org           |
| **Verificação formal**   | ProVerif ou Tamarin                                          | Ferramentas acadêmicas abertas    |

## B6. Mapa de dados

| **Dado**                      | **Quem vê**                                            | **A empresa pode entregar**                      | **Retenção**                   |
|-------------------------------|--------------------------------------------------------|--------------------------------------------------|--------------------------------|
| **Conteúdo dos apps Nimbus**  | Só o cliente e seus contatos                           | Blocos ilegíveis                                 | Até expiração ou exclusão      |
| **Conteúdo do Cloud Phone**   | Apps dentro da máquina do cliente, só durante a sessão | Disco cifrado; a meia-chave da Nimbus não existe | Enquanto o cliente mantiver    |
| **IP**                        | Só o relay independente                                | Nada                                             | Zero                           |
| **Identidade**                | Não existe                                             | Nada                                             | —                              |
| **Pagador ↔ conta**           | Processador vê o pagador; Nimbus vê ficha cega         | Nada que ligue                                   | Ficha descartada               |
| **Número virtual**            | Operadora parceira                                     | Nada                                             | —                              |
| **Chaves públicas e imagens** | Público (log auditável)                                | O que já é público                               | Permanente                     |
| **Notificações**              | Apple/Google e Nimbus veem token sem identidade        | Token                                            | Rotativo; modo sem notificação |

## B7. Regras de engenharia inegociáveis

- **EN1** Sem SDK de terceiros de análise, publicidade ou falhas; sem criptografia própria; sem acesso humano à produção (sem SSH, imagens imutáveis).

- **EN2** Toda alteração: revisão de duas pessoas, varredura nimbus-regras.yml limpa, testes aprovados, procedência assinada.

- **EN3** Código do cliente e da imagem do Cloud Phone aberto para auditoria; builds verificáveis.

- **EN4** Falha crítica: correção em até 7 dias; relatório de transparência público.

## B8. Código entregue (nimbus-nucleo.zip) — todo em português

| **Módulo**                  | **Requisito**              | **Situação**                                                                                                                                                                                                                                                                                                                  |
|-----------------------------|----------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **envelope**                | CRY1, CRY3, ARQ4, FP1, FP2 | Pronto: cifra híbrida pós-quântica, suíte padrão e FIPS, preenchimento, zeragem de memória, chave degenerada recusada na leitura, limite de 64 MiB conferido antes de alocar; 16 testes, inclusive remetente malicioso                                                                                                        |
| **coacao**                  | ANO7, AL2, AR6             | Pronto: PIN de coação e cofre-isca; verificador derivado por Argon2id (RFC 9106, 64 MiB), comparação em tempo constante e as duas comparações sempre feitas; limite de tentativas de PIN; 8 testes, mais a prova de tempo C80                                                                                                 |
| **trafego**                 | ANO3, ANO4                 | Pronto: batida constante, enchimento, espalhamento de rajada; 5 testes. Medir bateria e dados                                                                                                                                                                                                                                 |
| **politica**                | CH2, CH3, AR2              | Pronto: prazos por conversa e por mensagem (vence a mais restritiva), visualização única para todo tipo de conteúdo, citar (desligado por padrão) e Conversa Blindada. O que sai da conversa não é decidido aqui; 8 testes                                                                                                    |
| **historico**               | EF5, EF6, CH4              | Pronto: histórico no servidor cego, aberto só no aparelho, apagamento criptográfico por janela; 7 testes (inclui cópia guardada por terceiro ficando ilegível)                                                                                                                                                                |
| **expiracao**               | CH2, NC6, HIS15            | Pronto: Modo A, Modo B, visualização única, prazo para ler (mensagem não lida some em até 48 h; o remetente pode pedir menos), tudo com assinatura híbrida pós-quântica; 11 testes                                                                                                                                            |
| **integridade**             | DV2, DV3                   | Pronto: níveis verde/amarelo/vermelho e permissões, com perfil desconhecido e instalação lateral; 13 testes, inclusive atestação de hardware e desafio de uso único. Coleta dos sinais: camadas nativas (a construir)                                                                                                         |
| **sessoes**                 | ID2, CG1                   | Pronto: sessão única; tentativa sem credencial recusada sem derrubar o dono; clone com credencial derruba todas; revogação de aparelho roubado; contagens anônimas; 7 testes. Distribuição em memória entre servidores: a construir                                                                                           |
| **escudo**                  | EC4, EC5, EC7              | Pronto: estado assinado, validade contada a partir da chegada, carimbo adiantado recusado, anti-repetição, decisão de borrar; 11 testes. Coleta nativa dos sinais de captura: a construir                                                                                                                                     |
| **voz**                     | VM1, VM5, VN1, VN2         | Pronto: Nível 1 deslocador de tom em tempo real com desvio por chamada; Nível 2 orquestração da conversão neural (vozes-alvo, orçamento de latência, memória protegida); 16 testes. Modelo neural e inferência nativa: a fornecer                                                                                             |
| **validade**                | VAL1–VAL8                  | Pronto: agenda contando do fim do compromisso, demais itens da criação, limites, integração com apagamento criptográfico; 9 testes                                                                                                                                                                                            |
| **envio**                   | ENV1–ENV7                  | Pronto: cifra em blocos contra corte/troca/alteração, chave no fragmento do link, senha, prazo, downloads e revogação; 12 testes                                                                                                                                                                                              |
| **higiene**                 | HIG0                       | Pronto: JPEG e PNG sem EXIF/GPS/autor/datas, CRC conferido, nome neutro, recusa de PDF/Office sem achatar; 8 testes                                                                                                                                                                                                           |
| **memoria**                 | RZ3, RZ11, RZ12            | Pronto: segredo zerado, travado na RAM quando possível, nunca impresso; 4 testes                                                                                                                                                                                                                                              |
| **registro**                | BC2, BC5, BL1, CO7         | Pronto: árvore de Merkle da RFC 9162, provas de inclusão para qualquer tamanho, raiz; 6 testes (todas as posições de 1 a 33 entradas)                                                                                                                                                                                         |
| **aprovacao**               | ASS2–ASS4, BL4b            | Pronto: k de n com assinatura híbrida pós-quântica, vinculação dinâmica, prazo, carência, publicação de versões; 10 testes                                                                                                                                                                                                    |
| **pacote**                  | MTA1–MTA3                  | Pronto: nome, tipo, data e pasta dentro do bloco cifrado, tamanho independente do nome; 5 testes                                                                                                                                                                                                                              |
| **acesso**                  | ANO9, ANO13                | Pronto: endereços derivados, lotes fixos com cobertura embaralhada; 7 testes                                                                                                                                                                                                                                                  |
| **rede**                    | NET1–NET14                 | Pronto: Automático, Sempre Tor e Rápido; Tor nos momentos de identidade e nas conversas blindadas; ponte quando a rede bloqueia; nunca desce do Tor sozinho; sorteio de operador; cabeçalhos uniformes; Bluetooth que muda; 12 testes                                                                                         |
| **notificacao**             | NOT1–NOT4                  | Pronto: cadência fixa, real e cobertura do mesmo tamanho, texto genérico, descarte em silêncio; 6 testes                                                                                                                                                                                                                      |
| **contatos**                | CTT1–CTT8                  | Pronto: nome salvo, chaves e notas do contato num bloco cifrado de tamanho fixo; nome de perfil cifrado com chave entregue só aos contatos; 8 testes                                                                                                                                                                          |
| **assinatura**              | PQS1–PQS6                  | Pronto: assinatura híbrida Ed25519 + ML-DSA-87 (FIPS 204); as duas precisam conferir; contexto por uso; 9 testes (inclui quebrar só uma das duas)                                                                                                                                                                             |
| **senha**                   | SEN1–SEN4                  | Pronto: Argon2id com parâmetros da RFC 9106 (64 MiB, 3 passagens, 4 faixas); 4 testes                                                                                                                                                                                                                                         |
| **cofre**                   | COF1–COF8                  | Pronto: álbuns, notas, compromissos, senhas e ajustes com tipo escondido e classes de tamanho; 8 testes                                                                                                                                                                                                                       |
| **conversa**                | HIS1–HIS16                 | Pronto: regras pela mais restritiva entre os participantes; mensagens em 48 h e não lidas em até 48 h; chamada atendida sem registro; chamada não atendida visível 48 h para quem recebeu (quem ligou não apaga); sem marcas; conversa vazia some; apagar tudo só com consentimento; caixa do servidor em até 48 h; 16 testes |
| **liberacao**               | LIB1–LIB16                 | Pronto: mensagem não sai; cartão encaminhável (texto ou localização, só no envio, até 4 h, só no Nimbus); arquivo só com liberação assinada, com prazo de 30 s a 4 h contado da abertura; item some ao fim se o autor pedir; um salto; revogação; aviso ao autor; saída única com metadados limpos; 16 testes                 |
| **apresentacao**            | APR1–APR4                  | Pronto: nome por contato, sugerido ou travado, assinado e versionado; personas com chaves separadas; 7 testes                                                                                                                                                                                                                 |
| **inspecao**                | INS1–INS3                  | Pronto: tipo real pelos primeiros bytes; bloqueio de programas, instaladores, scripts, macros e extensão mentirosa; PDF e Office só como imagem; 5 testes                                                                                                                                                                     |
| **direto**                  | DIR1                       | Pronto: envio direto inspecionado, sem metadados, cifrado para quem recebe, entrega única, até 48 horas; 4 testes                                                                                                                                                                                                             |
| **modos**                   | MOD1–MOD8                  | Pronto: padrão de envio por tipo de conteúdo e escolha na hora; a conversa é o teto; nenhum modo passa de 48 horas; 6 testes                                                                                                                                                                                                  |
| **album**                   | ALB1–ALB8                  | Pronto: álbuns e coleções automáticas por regra (contato de origem, marcação ou padrão); nome da coleção, legenda, pessoas e data cifrados no cofre; 6 testes                                                                                                                                                                 |
| **chamada**                 | CH1–CH5                    | Pronto: sinalização de chamada com atendimento humano obrigatório — nenhuma mensagem da rede liga o microfone; só o toque em atender ativa o áudio; 13 testes                                                                                                                                                                 |
| **panico**                  | PAN1–PAN9                  | Pronto: travas de emergência decididas dentro do aparelho — palavra mágica, toque de pânico, longe de casa, movimento brusco, coação e sem me mexer —, cada uma com a reação escolhida; 7 testes                                                                                                                              |
| **recuperacao**             | REC1–REC6                  | Pronto: recuperação com 3 a 5 amigos de confiança (k de n), cada parte cifrada só para o seu guardião; 4 testes                                                                                                                                                                                                               |
| **corporativo**             | COR1–COR16                 | Pronto: lista de contas assinada pela empresa, validade curta, relógio confiável, anti-volta de lista, recuperação só de conta suspensa, k de n para reativar, suspender em massa e trocar administradores; 18 testes                                                                                                         |
| **custodia**                | COR5                       | Pronto: chave mestra dividida entre até 7 guardiões, k de n, com compromisso contra peça adulterada; 8 testes em todas as combinações de guardiões                                                                                                                                                                            |
| **convites**                | SPAM1–SPAM6                | Pronto: conversa só com convite de uso único e prazo; servidor guarda só o hash; faxina de usados e vencidos; 7 testes                                                                                                                                                                                                        |
| **tests/robustez**          | —                          | Pronto: milhares de entradas aleatórias e adulteradas em todos os leitores de dados (higiene, envelope, envio, pacote, notificação, registro); nenhum derruba o app; 5 testes                                                                                                                                                 |
| **tests/propriedades**      | VER2                       | Pronto: 8 leis de segurança conferidas com centenas de casos sorteados cada (ex.: a mensagem nunca afrouxa a regra da conversa; qualquer alteração do pedido invalida as aprovações)                                                                                                                                          |
| **verificacao/mutantes.py** | VER4                       | Pronto: 353 defeitos plantados em pontos críticos; todos pegos pelos testes (na primeira rodada, 6 passaram e os testes foram reforçados)                                                                                                                                                                                     |

## B9. Telas dos dois aplicativos (protótipo navegável)

O arquivo NimbusOne_Prototipo.html traz todas as 66 telas, cada uma com o que implementar e a verdade de seus limites. É a referência visual obrigatória do time; o texto das telas é definitivo, exceto os preços.

| **Aplicativo**            | **Telas**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | **Requisitos principais**                                                                                                                                                   |
|---------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **NimbusChat (27 telas)** | Abertura · boas-vindas · criar conta · frase de recuperação · PIN e PIN de coação · conversas · conversa · opções de envio · regras da conversa · chamada · chamada em grupo · adicionar contato · palavras de confirmação · salvar contato · como você aparece · histórico de chamadas · alerta do Escudo · aparelho inseguro · ajustes · meu jeito de enviar · privacidade avançada · aparelhos e acessos · licença · conta corporativa · acesso suspenso · enviar documento · envio direto · link criado · destinatário no navegador · recuperar conta · amigos de confiança | DV, ID1, RC1–RC4, ANO7, NC1–NC14, CH2, VC1, VM1, VM5, EC1–EC7, SES6, SES7, F10, ENV1–ENV7, HIG0, NET1–NET14, NOT1–NOT12, DIS1–DIS5, CTT1–CTT5, COR1–COR16, PLN1, HIS1–HIS13 |
| **NimbusOne (27 telas)**  | Abertura · entrar · início do NimbusOne · agenda com validade · nota com validade · Álbum Cofre · trazer do celular · verificação do ambiente · celular virtual · app rodando · atalho do NimbusChat · instalar apps · Sala Blindada · aprovações · assinar documento · aprovar pagamento · ajustes e aparelhos · proteção do PIN · travas de emergência · álbuns · planos · contratação · área do cliente · console da empresa · suspender acesso · chave mestra e guardiões · recuperar dados                                                                                 | VAL1–VAL8, AL1, AL2, MTA1–MTA3, COF1–COF8, CO1, CO5, CP1–CP8, KS1–KS4, DP2, WA1, LM8, SB1–SB7, ASS1–ASS10, BIO1–BIO3, COR1–COR16, PLN2–PLN4                                 |

- **PL1 \[PARCIAL\]** Planos e preços: valem os da seção A0E (licença única do NimbusChat; NimbusOne Cofre, Completo, Empresas e Governo e Defesa), que substituem esta versão anterior.

- **PL2 \[SIM\]** Pagamento sem identificação pelo site (ficha anônima) como opção recomendada; compra pela loja como alternativa, declarando ao usuário que a loja sabe da compra (F10, D13).

## B10. Exemplo executável de ponta a ponta

Comando: cargo run --example fluxo_completo. Percorre, em 34 passos, o caminho real do NimbusOne: verificação do aparelho, cadastro sem telefone, PIN de coação, sessão única e bloqueio de clone, temporizador por conversa e por mensagem, prazo assinado pelo servidor, Escudo de Conversa, histórico no servidor cego com apagamento criptográfico, envelope pós-quântico, tráfego de cobertura, modulador de voz, agenda que se apaga sozinha, limpeza de metadados, Nimbus Envio, memória blindada, registro público de chaves, aprovação de pagamento por 2 de 3 executivos, nomes de arquivo cifrados, pedidos que o servidor não consegue ligar, rota de rede por tipo de tráfego, notificações em cadência fixa, contato salvo com nome cifrado, convite de uso único, cofre universal com tipo escondido, assinatura híbrida pós-quântica, suspensão corporativa sem leitura, recuperação com a chave mestra, histórico que some para os dois e a regra “mensagem nunca sai; arquivo só com autorização assinada”. Cada passo cita o requisito deste manual e tem teste correspondente. É o primeiro arquivo que o time deve ler e rodar.

## B11. Revisão final do código (v19): o que foi encontrado e corrigido

Revisão linha por linha, compilação com os avisos mais rigorosos do Rust e testes com milhares de entradas aleatórias. Registro honesto do que estava errado e agora está corrigido e testado:

- **REV1 \[SIM\]** Negação de serviço: qualquer pessoa que soubesse o identificador de uma conta derrubava o dono repetidamente, só tentando entrar. Agora, tentativa sem credencial é recusada sem afetar ninguém; só clone com credencial derruba as sessões.

- **REV2 \[SIM\]** Chave pública X25519 degenerada (de ordem baixa) era aceita; agora é recusada já na leitura, e o envelope confere a contribuição dos dois lados da troca de chaves.

- **REV3 \[SIM\]** O envelope não tinha limite de tamanho, e o comprimento era guardado em 32 bits (poderia corromper conteúdos gigantes). Agora há limite de 64 MiB conferido antes de alocar; arquivos maiores vão pelo módulo envio, em blocos.

- **REV4 \[SIM\]** Escudo de Conversa: um carimbo de tempo adiantado mantinha o estado “fresco” indefinidamente. Agora a validade conta da chegada, e carimbo adiantado é recusado.

- **REV5 \[SIM\]** Voz: a memória da conversão neural podia crescer e realocar, deixando cópia da voz real em memória liberada. Agora a capacidade é fixa, com recusa quando cheia, e o espaço livre é zerado.

- **REV6 \[SIM\]** Nome de arquivo: a memória do item selado podia realocar com nomes longos, deixando cópia do nome. Agora a capacidade é calculada antes.

- **REV7 \[SIM\]** PIN de coação igual ao real era aceito (o cofre-isca nunca abriria); agora é recusado no cadastro. Quórum de aprovação com a mesma chave repetida na lista parecia válido; agora conta pessoas distintas.

- **REV8 \[SIM\]** Tipos com segredo (chaves, sementes, voz, verificadores) agora mostram só “oculto” se forem impressos por engano; compilação rigorosa sem nenhum aviso passou a ser etapa obrigatória do pipeline.

- **REV9 \[SIM\]** Plano corporativo: o travamento sem internet dependia do relógio do celular — um demitido poderia voltar a hora e usar para sempre. Agora a hora usada é confiável e reinício sem internet exige conectar.

- **REV10 \[SIM\]** Ferramenta de mutação: ao devolver o arquivo original com a data antiga, o compilador mantinha o programa com o defeito plantado; uma rodada interrompida deixava um defeito no código. Agora guarda cópia antes, atualiza a data e se restaura sozinha.

- **REV11 \[SIM\]** Teste fraco: um administrador sozinho podia declarar “basta uma assinatura” para trocar os administradores — pego pela mutação e reforçado. Uma verificação que sempre passava foi removida.

- **REV12 \[SIM\]** Rastros fora da conversa: a lista de conversas mostrava prévia das mensagens. Agora prévia, contador no ícone do app, confirmação de leitura e “digitando…” vêm desligados por padrão.

**PARTE C — INSTRUÇÕES AO TIME TÉCNICO**

- **IT1** Ordem de leitura: este manual → protótipo HTML (todas as telas, com o que implementar em cada uma) → rodar cargo run --example fluxo_completo → LEIAME do nimbus-nucleo → proposta Nimbus Trusted Banking (só na Fase 5).

- **IT2** Repositório único: /nucleo (Rust: nimbus-nucleo, depois libsignal e MLS), /ios (SwiftUI), /android (Kotlin/Compose), /desktop (Tauri), /web (WebAssembly), /server (Rust: relay, cofre, registro de chaves), /cloudphone (imagem Android reproduzível, custodiante), /provas (scripts das provas da Parte E).

- **IT3** Primeiros passos: rodar ./verificar.sh no kit (verificação completa, em falha fechada) antes de qualquer alteração e antes de cada versão; ./verificar.sh --rapido ajuda no dia a dia, mas termina em PARCIAL e não vale como verificação.

- **IT4** Definição de pronto de qualquer tarefa: requisito com código do manual citado no commit; testes novos; varredura limpa; revisão de duas pessoas; nenhuma dependência nova sem aprovação e inventário atualizado.

- **IT5** Proibições absolutas: criptografia própria; SDK de terceiros; log de conteúdo, IP ou identificador; acesso SSH à produção; contornar ou falsificar atestações de Apple, Google ou bancos.

- **IT9** Regra de separação: o repositório do NimbusChat não pode depender de nenhum componente exclusivo do Nimbus OS. O que é comum (nimbus-nucleo, identidade, registro de chaves) fica em biblioteca compartilhada; o resto é independente. Se o NimbusChat deixar de compilar e rodar sozinho, a entrega está reprovada.

- **IT10** Proibido integrar telefonia comum (SIP, PSTN, provedores de número virtual). A varredura (regra nimbus-telefonia-comum) reprova a entrega.

- **IT8** Todo o projeto em português: nomes de módulos, funções, variáveis, comentários, testes, mensagens de commit e documentação. Exceções permitidas: nomes de padrões e bibliotecas de terceiros (ML-KEM, WebRTC, FLAG_SECURE), que não se traduzem.

- **IT6** Dúvida de interpretação de requisito: registrar a pergunta e parar; nunca presumir em item de segurança.

- **IT7** Cada entrega de fase: enviar o repositório ao fundador para a revisão de primeira camada (Claude) antes do auditor.

**PARTE D — PLANO DE TRABALHO**

*Sem datas: a equipe deve estimar prazos após ler este documento. Nenhuma fase começa sem o portão da anterior aprovado.*

| **Fase**                                                | **Entregas**                                                                                                                                                                          | **Responsáveis**                                     | **Portão (aprovado se)**                                                                                                                                                                                                                                                                   |
|---------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **0 — Fundação**                                        | Modelo de ameaças; contrato com auditor independente; pareceres: patente Confide, licença AGPL, termos de WhatsApp/Meta, App Store, licença Google; escolha do custodiante e do relay | Claude (documentos), fundador (contratos), advogados | Auditor aprova o modelo de ameaças; todos os pareceres emitidos                                                                                                                                                                                                                            |
| **1 — Núcleo**                                          | Núcleo Rust (a partir do nimbus-nucleo), identidade, recuperação, registro de chaves, relay em memória, cofre, relay de IP; pipeline com varredura e SBOM                             | Time (Claude entrega protótipos e testes)            | C1 parcial, C4, C5, C7, C8, C33, C35, C39, C42, C45, C46, C50, C51, C52 aprovados                                                                                                                                                                                                          |
| **2 — Produto NimbusChat**                              | Nimbus OS com modo efêmero: Chat, temporárias, chamadas, contatos, agenda, notas, dois álbuns, arquivos, senhas                                                                       | Time; Claude revisa cada entrega                     | C2, C10, C12, C14, C16, C17, C18, C20, C21, C22, C23, C24, C25, C26, C27, C28, C29, C30, C31, C37, C38, C41, C43, C44, C47, C48, C49, C53, C54, C55, C56, C57, C58, C59, C60, C61, C62, C63, C64, C65, C66, C67, C68, C69, C70, C71, C72, C73, C74, C75, C76, C77, C78, C79, C80 aprovados |
| **3 — Lançamento do NimbusChat (produto independente)** | Auditoria completa, verificação formal, programa de recompensa por falhas, relatório de transparência                                                                                 | Auditor; especialista em verificação formal          | C1, C9, C11 aprovados → lançamento                                                                                                                                                                                                                                                         |
| **4 — Produto Nimbus OS (piloto)**                      | Máquina confidencial, chave dividida, custodiante, 20 contas com WhatsApp/Signal/Telegram/Instagram por 30 dias; medir custo por cliente                                              | Time; auditor especializado                          | C13, C15, C32, C34, C36, C40, C-WA aprovados; custo aceito pelo fundador                                                                                                                                                                                                                   |
| **5 — Expansão**                                        | Navegador Isolado; banco piloto (Nimbus Trusted Banking); edição Enterprise/FIPS via Defensys                                                                                         | Time; comercial                                      | Provas específicas de cada módulo                                                                                                                                                                                                                                                          |

**PARTE E — PROTOCOLO DE VALIDAÇÃO**

| **Prova** | **O que é**                                                                                                                                                                                                                                                      | **Quem executa**                                                                       | **Aprovado somente se**                                                                                                                                              |
|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **C1**    | Auditoria de código e criptografia                                                                                                                                                                                                                               | Auditor independente (Claude: primeira camada)                                         | Só padrões da B5; nenhuma falha crítica/alta aberta                                                                                                                  |
| **C2**    | Servidor entregue: acesso administrativo total por uma semana, contas em uso                                                                                                                                                                                     | Auditor                                                                                | Só blocos cifrados, chaves públicas e contagens                                                                                                                      |
| **C3**    | Tentar ligar pagador à conta                                                                                                                                                                                                                                     | Auditor                                                                                | Impossível                                                                                                                                                           |
| **C4**    | Clone, sessão paralela, app adulterado                                                                                                                                                                                                                           | Auditor                                                                                | Bloqueados e avisados                                                                                                                                                |
| **C5**    | Aparelho fantasma inserido pelo operador                                                                                                                                                                                                                         | Auditor                                                                                | Detectado e exibido a dono e contatos                                                                                                                                |
| **C6**    | Histórico do repositório                                                                                                                                                                                                                                         | Auditor + Claude                                                                       | 100% das alterações com duas aprovações                                                                                                                              |
| **C7**    | Varredura nimbus-regras.yml                                                                                                                                                                                                                                      | Time/Claude                                                                            | Zero ocorrências não justificadas                                                                                                                                    |
| **C8**    | Captura de tráfego do aparelho por 48 h                                                                                                                                                                                                                          | Auditor                                                                                | Só destinos do relay; tudo cifrado                                                                                                                                   |
| **C9**    | Build publicado = código auditado                                                                                                                                                                                                                                | Auditor                                                                                | Hashes conferem                                                                                                                                                      |
| **C10**   | Perícia forense no aparelho após uso intenso                                                                                                                                                                                                                     | Perito independente                                                                    | Nenhum conteúdo recuperado                                                                                                                                           |
| **C11**   | Modelo formal dos protocolos                                                                                                                                                                                                                                     | Especialista                                                                           | Sigilo e autenticação provados                                                                                                                                       |
| **C12**   | Troca de aparelho e roubo simulado, cada método de recuperação                                                                                                                                                                                                   | Time + auditor                                                                         | Vida restaurada; antigo revogado                                                                                                                                     |
| **C13**   | Cloud Phone: hospedeiro tenta ler memória e disco; imagem adulterada tenta receber chave                                                                                                                                                                         | Auditor de computação confidencial                                                     | Falham todas as tentativas                                                                                                                                           |
| **C14**   | Modo efêmero: perícia 1 minuto após fechar o app                                                                                                                                                                                                                 | Perito                                                                                 | Nenhum conteúdo em disco ou memória recuperável                                                                                                                      |
| **C15**   | Chave dividida: Nimbus sozinha e custodiante sozinho tentam abrir um disco                                                                                                                                                                                       | Auditor                                                                                | Ambos falham                                                                                                                                                         |
| **C16**   | Verificação do aparelho: aparelho com root, sobreposição, acessibilidade maliciosa, gravação de tela                                                                                                                                                             | Auditor                                                                                | Níveis e bloqueios corretos; servidor recusa operação sensível mesmo com cliente adulterado                                                                          |
| **C17**   | Temporárias: relógio do aparelho alterado, prazo adulterado, aparelho offline                                                                                                                                                                                    | Auditor                                                                                | Nenhum prolongamento; destruição confirmada                                                                                                                          |
| **C18**   | Escudo de Conversa: gravar tela, espelhar, dar print, gravar áudio, falsificar ou repetir estado, desligar a rede de um participante                                                                                                                             | Auditor                                                                                | Conversa borrada em todos os aparelhos em até 3 s; aviso correto; nenhuma captura legível                                                                            |
| **C19**   | Edição FIPS: módulo criptográfico em uso tem certificado FIPS 140-3 vigente                                                                                                                                                                                      | Auditor                                                                                | Certificado conferido no programa oficial do NIST (CMVP)                                                                                                             |
| **C20**   | Chamada: gravador de tela e de chamada no aparelho do outro participante; conferir Recentes do iPhone                                                                                                                                                            | Auditor                                                                                | Nenhum áudio útil capturado; nenhuma chamada em Recentes                                                                                                             |
| **C21**   | Blindagem de anonimato: varredura das permissões e chamadas proibidas; captura de tráfego com o app parado e em conversa                                                                                                                                         | Auditor                                                                                | Nenhuma permissão ou chamada da lista ANO1; o padrão de tráfego não distingue silêncio de conversa                                                                   |
| **C22**   | PIN de coação: medir o tempo de resposta dos três casos e confirmar que a tela não muda                                                                                                                                                                          | Auditor                                                                                | Tempos indistinguíveis; cofre-isca abre sem sinal; cofre real bloqueado e sessões cortadas                                                                           |
| **C23**   | Separação dos produtos: tentar instalar ou executar o NimbusChat dentro do Cloud Phone; abrir o atalho do NimbusChat a partir do Nimbus OS                                                                                                                       | Auditor                                                                                | Instalação recusada; o atalho sai da máquina virtual e abre o app nativo; nenhuma mensagem é decifrada no servidor                                                   |
| **C24**   | Histórico no servidor: com acesso total ao servidor, tentar ler qualquer bloco; após vencer a janela, tentar ler uma cópia previamente guardada                                                                                                                  | Auditor                                                                                | Nenhum bloco legível; a cópia após o vencimento é indecifrável (apagamento criptográfico comprovado)                                                                 |
| **C25**   | Não deixar rastro: perícia de memória e de disco durante e após o uso; inspeção de caches, teclado, Spotlight/sugestões                                                                                                                                          | Perito                                                                                 | Conteúdo só na memória durante a exibição; nada em disco, cache, dicionário do teclado ou índice do sistema                                                          |
| **C26**   | Modulador Nível 2: com a conversão neural ligada, gravar a saída e tentar recuperar a voz original; medir latência de ida                                                                                                                                        | Auditor de áudio                                                                       | O timbre original não é recuperável da saída; a voz real não aparece em memória fora da sessão; latência dentro do orçamento declarado                               |
| **C27**   | Modulador de voz: comparar amostras original e transmitida                                                                                                                                                                                                       | Auditor + especialista em áudio                                                        | Somente voz modificada no tráfego; latência dentro da meta; inteligibilidade preservada                                                                              |
| **C28**   | Validade: compromisso e nota com prazo; conferir sumiço na hora, destruição da chave ao fim da janela, cópia guardada ilegível e lembrete cancelado                                                                                                              | Auditor                                                                                | Item invisível no prazo; cópia indecifrável após a janela; nenhuma notificação pendente com o item                                                                   |
| **C29**   | Nimbus Envio: inspecionar tudo o que chega ao servidor; cortar, reordenar e alterar blocos; esgotar, expirar e revogar o link                                                                                                                                    | Auditor                                                                                | O servidor nunca recebe a chave; toda adulteração falha; link recusado nas três situações                                                                            |
| **C30**   | Higiene: fotos reais de vários celulares e documentos com autor e histórico                                                                                                                                                                                      | Perito                                                                                 | Nenhum metadado identificador sobra; PDF e Office só saem achatados                                                                                                  |
| **C31**   | Falha forçada e memória: provocar falhas com conteúdo aberto; examinar relatório de falha, registros e arquivo de troca                                                                                                                                          | Perito                                                                                 | Nenhum conteúdo em relatório, registro ou swap                                                                                                                       |
| **C32**   | Sala Blindada: tentar baixar, imprimir, copiar e extrair o arquivo pela página; revogar com sessão aberta                                                                                                                                                        | Auditor                                                                                | Nenhum arquivo obtido; sessão encerrada na revogação; nenhuma marca que identifique quem viu                                                                         |
| **C33**   | Publicação: tentar publicar versão com uma assinatura só; instalar atualização fora do registro público                                                                                                                                                          | Auditor                                                                                | Publicação recusada; o app não instala a versão                                                                                                                      |
| **C34**   | Aprovação de pagamento: trocar valor ou conta depois das aprovações; aprovar com a mesma pessoa duas vezes; ordem por áudio falsificado com IA                                                                                                                   | Auditor                                                                                | Todas as tentativas falham; nenhuma ordem por voz produz aprovação                                                                                                   |
| **C35**   | Registro público: provas de inclusão; tentativa de inserir aparelho fantasma                                                                                                                                                                                     | Auditor                                                                                | Provas válidas conferem; aparelho fantasma é denunciado ao dono e aos contatos                                                                                       |
| **C36**   | O que você vê é o que você assina: tentar trocar o documento entre a exibição e a assinatura, com o aparelho do executivo comprometido                                                                                                                           | Auditor                                                                                | Troca detectada; assinatura recusada                                                                                                                                 |
| **C37**   | Nomes cifrados: procurar nomes de arquivos, pastas e álbuns no servidor, nos registros e no armazenamento do aparelho                                                                                                                                            | Perito                                                                                 | Nenhum nome encontrado em lugar algum                                                                                                                                |
| **C38**   | Ligação de pedidos: com acesso total ao servidor, tentar agrupar pedidos de uma mesma pessoa por endereço, horário, conexão ou ficha                                                                                                                             | Auditor                                                                                | Agrupamento não melhor que o acaso                                                                                                                                   |
| **C39**   | Cadeia de fornecimento: introduzir dependência maliciosa, commit sem revisão e versão com uma assinatura só                                                                                                                                                      | Auditor                                                                                | As três tentativas são barradas pelo pipeline, pela revisão ou pela publicação k de n                                                                                |
| **C40**   | Sala Blindada: navegador e nível máximo — tentar baixar, copiar, imprimir, gravar e fotografar                                                                                                                                                                   | Auditor                                                                                | Sem arquivo, sem texto, impressão em branco; no nível máximo, print e gravação bloqueados                                                                            |
| **C41**   | Exposições do sistema: DNS, notificações, widgets, IA do sistema, Bluetooth, cabeçalhos                                                                                                                                                                          | Auditor                                                                                | Nenhuma exposição listada em A6M ocorre além das declaradas como inevitáveis                                                                                         |
| **C42**   | Velocidade: medir as metas NET14 nas três escolhas de rede, em aparelho de entrada e de topo                                                                                                                                                                     | Time + auditor                                                                         | Metas da rota rápida atingidas; troca automática para o Tor em poucos segundos; lentidão do Sempre Tor medida e informada ao usuário                                 |
| **C43**   | Notificações: com acesso ao lado do servidor, conferir uma notificação por aparelho por janela, tamanhos iguais e descarte em silêncio no aparelho                                                                                                               | Auditor                                                                                | Nenhuma diferença observável entre aparelho com e sem mensagem                                                                                                       |
| **C44**   | Discrição: ícone e nome alternativos, fachada no Android, fatura com descritor neutro, Bluetooth sem identificador fixo                                                                                                                                          | Auditor                                                                                | Nenhum elemento revela “Nimbus” fora do que foi declarado como inevitável                                                                                            |
| **C45**   | Rotas: com captura de tráfego, conferir Tor em todos os momentos de identidade e conversas blindadas no Automático, nenhum Tor no Rápido, ponte quando a rede bloqueia e nenhuma descida silenciosa do Tor                                                       | Auditor                                                                                | Todas as situações seguem a tabela A6N                                                                                                                               |
| **C46**   | Robustez: milhares de entradas aleatórias e adulteradas em todos os leitores de dados, e compilação rigorosa sem avisos                                                                                                                                          | Pipeline + auditor                                                                     | Nenhuma queda; nenhum aviso                                                                                                                                          |
| **C47**   | Nomes: procurar o nome salvo dos contatos e os nomes de perfil no servidor, registros, notificações, tela de chamada bloqueada e armazenamento                                                                                                                   | Perito                                                                                 | Nenhum nome encontrado; blocos de tamanho fixo                                                                                                                       |
| **C48**   | Grupos: com acesso ao servidor, tentar descobrir quem participa de cada grupo                                                                                                                                                                                    | Auditor                                                                                | Membros não identificáveis pelo servidor                                                                                                                             |
| **C49**   | Página do navegador: conferir hash publicado, política de segurança de conteúdo e ausência de scripts de terceiros; tentar injetar código                                                                                                                        | Auditor                                                                                | Hash confere; injeção bloqueada                                                                                                                                      |
| **C50**   | Método: rodar propriedades, robustez e mutação no pipeline                                                                                                                                                                                                       | Pipeline                                                                               | Todas as propriedades valem; nenhuma queda; todos os mutantes pegos (151 de 151 na v29); dependências auditadas contra a base RustSec, sem vulnerabilidade em aberto |
| **C51**   | Assinatura pós-quântica: forjar só a parte clássica, só a pós-quântica, misturar contextos; conferir tamanhos do FIPS 204                                                                                                                                        | Auditor                                                                                | Todas as tentativas recusadas; tamanhos conforme o padrão                                                                                                            |
| **C52**   | Invasão: com acesso total ao servidor e a um celular bloqueado, tentar ler qualquer item, nome ou tipo                                                                                                                                                           | Perito + auditor                                                                       | Nada legível; nenhum tipo de item distinguível                                                                                                                       |
| **C53**   | Notificações: Android sem nenhum tráfego para o Google; iPhone com carga da Apple só em ruído cifrado de tamanho fixo                                                                                                                                            | Auditor                                                                                | Nenhuma informação de remetente, conteúdo ou tipo fora do aparelho                                                                                                   |
| **C54**   | Link de envio: pré-visualização em outros apps, histórico do navegador, cabeçalhos, robôs de e-mail                                                                                                                                                              | Auditor                                                                                | Nada sobre quem enviou; chave fora do histórico; download único intacto                                                                                              |
| **C55**   | Suspensão: aparelho com e sem internet; relógio voltado; reinício sem internet; lista forjada, antiga ou com número pulado                                                                                                                                       | Auditor                                                                                | Conta suspensa trava e apaga as chaves; sem internet trava em até 15 minutos; nenhuma lista inválida aceita                                                          |
| **C56**   | Recuperação: tentar conta ativa; 1 administrador; 1 guardião; pacote de outra conta                                                                                                                                                                              | Auditor                                                                                | Todas recusadas; com 2 administradores e 2 guardiões, abre só a conta suspensa                                                                                       |
| **C57**   | Administrador desonesto: suspender em massa sozinho, reativar sozinho, trocar administradores sozinho                                                                                                                                                            | Auditor                                                                                | Todas exigem 2 de 3 e ficam no registro de auditoria                                                                                                                 |
| **C58**   | Transparência: aviso na conta corporativa e para contatos externos; conta pessoal intacta após a suspensão                                                                                                                                                       | Auditor                                                                                | Avisos presentes; nada da conta pessoal afetado                                                                                                                      |
| **C59**   | Chamadas atendidas: perícia nos dois aparelhos após chamadas feitas e recebidas                                                                                                                                                                                  | Perito                                                                                 | Nenhum registro de chamada atendida em nenhum dos dois aparelhos, nem no registro de chamadas do sistema                                                             |
| **C60**   | Sem marcas: chamadas, chamadas perdidas, mensagens expiradas e apagadas; conversa esvaziada; troca de aparelho do contato                                                                                                                                        | Auditor                                                                                | Nenhuma marca na conversa; conversa vazia some da lista dos dois; alerta de aparelho novo aparece como faixa e some ao confirmar                                     |
| **C61**   | Prazos e consentimento: regras aplicadas nos dois lados e em grupo; apagar tudo com e sem consentimento; caixa do servidor com aparelho desligado                                                                                                                | Auditor                                                                                | Sempre vale a mais restritiva; apagar tudo só com todos de acordo; mensagem não entregue apagada no prazo ou em 7 dias                                               |
| **C62**   | Chamadas não atendidas: quem ligou pede “apagar na hora”; conferir o aviso no aparelho de quem recebeu e o prazo                                                                                                                                                 | Auditor                                                                                | Quem recebeu vê quem ligou pelo prazo dele; depois, nada; quem ligou segue a regra mais restritiva                                                                   |
| **C63**   | Não lidas: mensagem nunca aberta, com prazo de conversa longo; pedido de prazo maior que 48 h; caixa do servidor com aparelho desligado                                                                                                                          | Auditor                                                                                | Some em até 48 horas em todos os casos, inclusive no servidor; o prazo adulterado invalida a assinatura                                                              |
| **C64**   | Liberação: mensagem; arquivo sem liberação; cada nível; prazo (antes de abrir, dentro, no fim); item que some; cartão encaminhável (no envio, depois do envio, sem prazo, acima de 4 h, voz); um salto; liberação adulterada; revogação; saída por outro caminho | Auditor                                                                                | Mensagem não sai; cartão só dentro do Nimbus e no prazo; tudo o mais recusado                                                                                        |
| **C65**   | Remetente malicioso: envelopes com conteúdo fora do padrão, vazio, curto ou com tamanho declarado falso                                                                                                                                                          | Auditor                                                                                | Todos recusados sem travar o app                                                                                                                                     |
| **C66**   | PIN: errar em sequência, tentar durante a espera, mudar o relógio, chegar ao 10º erro                                                                                                                                                                            | Auditor                                                                                | Espera que dobra; no 10º erro as chaves do aparelho são apagadas; a conta volta só pela frase de recuperação                                                         |
| **C67**   | Mensagem lida com prazo longo pedido; mensagem sem prazo; relógio do aparelho adiantado e atrasado                                                                                                                                                               | Auditor                                                                                | Nenhuma mensagem passa de 48 horas depois de enviada, em nenhum caso                                                                                                 |
| **C68**   | Como você aparece: nome travado, tentativa de editar, cartão adulterado, versão antiga reenviada, persona trocada                                                                                                                                                | Auditor                                                                                | Nome travado nunca muda nem aparece o nome real; adulterados recusados; personas não se ligam                                                                        |
| **C69**   | Convite interceptado: alguém usa o código antes; comparar as palavras de confirmação                                                                                                                                                                             | Auditor                                                                                | Palavras diferentes denunciam a interceptação                                                                                                                        |
| **C70**   | Inspeção: programa com nome de foto, instalador, script, documento com macro, PDF com código, extensão mentirosa                                                                                                                                                 | Auditor                                                                                | Todos bloqueados ou mostrados só como imagem; nenhum código roda                                                                                                     |
| **C71**   | Envio direto: segunda retirada, prazo vencido, remessa trocada, captura de tráfego                                                                                                                                                                               | Auditor                                                                                | Entrega única; nada depois do prazo; nenhum IP de aparelho visível ao outro                                                                                          |
| **C72**   | Link: download sem permissão; cache, histórico e memória do navegador depois de fechar; link reaberto                                                                                                                                                            | Perito                                                                                 | Sem download não permitido; nada em cache; link queimado não abre nada                                                                                               |
| **C73**   | IP: captura de tráfego no servidor e no outro aparelho, em todos os modos de rede e em chamadas                                                                                                                                                                  | Perito                                                                                 | O IP do aparelho nunca aparece                                                                                                                                       |
| **C74**   | Conta corporativa: procurar o nome da organização ou dos administradores no aparelho do funcionário, no de quem conversa com ele e no servidor                                                                                                                   | Auditor                                                                                | Não aparece em lugar nenhum                                                                                                                                          |
| **C75**   | Envio: padrão por tipo, escolha na hora, conversa mais restritiva, desfazer em 5 s                                                                                                                                                                               | Auditor                                                                                | A mensagem sai sempre com o mais restritivo; nada passa de 48 horas; nada sai antes dos 5 s                                                                          |
| **C76**   | Aparelho corrompido: root escondido, Frida, emulador, chave de atestação vazada, jailbreak, bootloader destravado                                                                                                                                                | Perito                                                                                 | Todos vermelhos, sem credencial no servidor                                                                                                                          |
| **C77**   | Atestação reenviada de um aparelho limpo a partir de um comprometido                                                                                                                                                                                             | Perito                                                                                 | Recusada: o desafio é de uso único e vence em 60 segundos                                                                                                            |
| **C78**   | Amigos de confiança: 1 amigo, parte adulterada, amigo errado, 2 amigos                                                                                                                                                                                           | Auditor                                                                                | Só 2 amigos verdadeiros recuperam                                                                                                                                    |
| **C79**   | Fuzzing: entradas aleatórias e malformadas em 19 leitores que recebem bytes de fora, com verificação de estouro ligada (50 mil por alvo; 2 milhões no modo completo)                                                                                             | Auditor                                                                                | Nenhum leitor entra em pânico: o app nunca cai por dado malformado                                                                                                   |
| **C80**   | Tempo constante: PIN real, PIN de coação e PIN errado (no primeiro e no último dígito)                                                                                                                                                                           | Auditor                                                                                | Mesmo tempo nos quatro casos (diferença abaixo de 25%)                                                                                                               |
| **C81**   | Sigilo de metadados: nomes/notas curtos e longos; mesmo contato cifrado duas vezes                                                                                                                                                                               | Auditor                                                                                | Cifras do mesmo tamanho; duas cifras diferentes — o comprimento e a repetição não vazam                                                                              |
| **C82**   | Limite de senha configurável: 3 erros livres, espera que dobra, apaga no 10º; e a política que nunca apaga; a espera conta só tempo real (adiantar o relógio do sistema ou reiniciar não encurta)                                                                | Auditor                                                                                | Bloqueia por tempo crescente e apaga só no limite escolhido                                                                                                          |
| **C83**   | Ataque de força bruta máximo: 45 milhões de entradas aleatórias e malformadas contra os 19 leitores que recebem bytes de fora                                                                                                                                    | Auditor                                                                                | Nenhum pânico, nenhum travamento (o CI repete com 54 milhões)                                                                                                        |
| **C84**   | Integridade do pacote: núcleo, manual e protótipo comparados byte a byte entre o pacote e os dois kits (verificar_pacote.py)                                                                                                                                     | Auditor                                                                                | Tudo idêntico; nenhum artefato pesado; nenhum termo de versão superada                                                                                               |
| **C85**   | Números dos materiais: testes, módulos, telas, provas e defeitos plantados citados em decks, manual e leia-mes, conferidos contra a fonte de verdade apurada do código (verificar_numeros.py)                                                                    | Auditor                                                                                | Nenhum número desatualizado fora do histórico de auditoria                                                                                                           |
| **C86**   | Adversário ativo no meio do caminho (modelo Dolev-Yao): guardar, repetir, reordenar, replantar em outra conversa e adulterar mensagens legítimas                                                                                                                 | Auditor                                                                                | Todas rejeitadas mesmo cifradas: a identidade da conversa e o número de sequência ficam presos no dado autenticado                                                   |
| **C87**   | Link ou nome com reordenação de texto (RTLO / unicode bidirecional), como no CVE-2022-28345 do Signal                                                                                                                                                            | Auditor                                                                                | Caractere enganoso removido antes de exibir; o nome/endereço real aparece (inspecao::sanear_texto_exibido)                                                           |
| **C88**   | Chamada forçada a atender sem toque do usuário, como no CVE-2019-17191 do Signal                                                                                                                                                                                 | Auditor                                                                                | Nenhuma mensagem da rede liga o microfone; só a ação local de atender ativa o áudio (chamada::Chamada)                                                               |
| **C89**   | Computador: sem chip de segurança ou sem disco cifrado fica vermelho; limpo fica amarelo, nunca verde; Cofre, visualização única e NimbusDrop bloqueados                                                                                                         | Time (teste automático) e auditor                                                      | Todos os casos no nível esperado (integridade::avaliar)                                                                                                              |
| **C90**   | Vínculo do computador: sem biometria no celular não vincula; o vínculo aparece para os contatos; cai no 8º dia sem o celular                                                                                                                                     | Time, em aparelhos reais                                                               | Os três comportamentos confirmados                                                                                                                                   |
| **C91**   | Hora confirmada: atrasar e adiantar a hora do sistema; 20 minutos sem o servidor; repetir uma resposta de hora antiga                                                                                                                                            | Time e auditor                                                                         | Nenhum prazo estendido; conteúdo oculto após 15 minutos; hora repetida recusada (expiracao::hora_confiavel, expiracao::conferir_hora)                                |
| **C92**   | Sem rede mesh: varredura nimbus-sem-rede-mesh e tentativa de entrega por Bluetooth ou Wi-Fi entre dois aparelhos                                                                                                                                                 | Time                                                                                   | Zero ocorrências; nada recebido fora do servidor                                                                                                                     |
| **C93**   | NimbusDrop lacrado: receber sem internet, mudar a hora, tentar em aparelho amarelo, captar o Bluetooth, alterar o prazo que viaja junto                                                                                                                          | Time e auditor                                                                         | Não abre sem conexão, vencido ou fora do verde; bloco captado ilegível (direto::abrir_lacrado)                                                                       |
| **C94**   | Envio direto com remetente falso: remessa sem assinatura, assinada por outra chave, ou assinada para outra pessoa e recifrada                                                                                                                                    | Time (teste automático) e auditor                                                      | Nenhuma abre; o app mostra “remetente não confirmado” (direto::abrir_assinado)                                                                                       |
| **C95**   | Registro reescrito ou bifurcado: trocar uma entrada antiga, mostrar raízes diferentes do mesmo tamanho, adulterar a prova                                                                                                                                        | Time (teste automático) e auditor                                                      | Toda variação recusada (registro::verificar_consistencia)                                                                                                            |
| **C96**   | Chave ML-KEM malformada: coeficientes acima do módulo                                                                                                                                                                                                            | Time (teste automático)                                                                | Recusada na leitura (envelope::ChavePublica::de_bytes)                                                                                                               |
| **C97**   | Convite: tentar confirmar sem comparar as palavras; tocar numa palavra errada                                                                                                                                                                                    | Time (teste automático) e teste de usabilidade                                         | Sem a palavra certa, o contato não é confirmado (convites::confirmacao_correta)                                                                                      |
| **C98**   | Vínculo por QR recebido, por link ou pela câmera do sistema; celular amarelo; sem biometria; palavras diferentes; celular e computador conectados ao mesmo tempo                                                                                                 | Time (teste automático) e pentest (PT-27)                                              | Só o vínculo iniciado no app, com tudo conferido, passa; celular e computador vinculado convivem (sessoes::pode_vincular)                                            |
| **C99**   | Nome reservado, disfarçado, com caractere enganoso, parecido com outro contato; troca de nome; cartão forjado                                                                                                                                                    | Time (teste automático)                                                                | Recusado ou com aviso; mudança sempre visível (apresentacao::nome_permitido)                                                                                         |
| **C100**  | Recuperação com amigos: pedir a parte por mensagem; concluir antes de 3 dias; aparelho antigo cancelando                                                                                                                                                         | Time (teste automático) e pentest (PT-28)                                              | Sem conferência ao vivo não libera; antes da espera não conclui; cancelamento vence (recuperacao::pode_concluir)                                                     |
| **C101**  | Recuperação de dados corporativos com urgência                                                                                                                                                                                                                   | Time (teste automático) e pentest (PT-30)                                              | Recusada antes de 24 h (corporativo::autorizar_recuperacao)                                                                                                          |
| **C102**  | Chaves de recebimento: servidor publica chave própria, troca a pública, estica o prazo; aparelho tomado depois da destruição                                                                                                                                     | Time (teste automático) e auditor                                                      | Chave trocada recusada; envelope antigo ilegível depois da destruição (prechaves::Chaveiro::apagar_vencidas)                                                         |
| **C103**  | Assinatura pós-quântica hedged: duas assinaturas da mesma mensagem                                                                                                                                                                                               | Time (teste automático)                                                                | Diferentes e ambas válidas (assinatura::ChaveDeAssinatura::assinar)                                                                                                  |
| **C104**  | Rotas: caminho torto, rota inexistente, sem prova, prova de outra rota, corpo trocado, fora da janela, repetida, titular forjado, chave do tipo errado, revogada                                                                                                 | Time (teste automático) e pentest (PT-35)                                              | Toda variação recusada (autenticacao::Porteiro::autorizar)                                                                                                           |
| **C105**  | Limites: martelar uma rota, errar seguidamente, encher a memória com chaves inventadas                                                                                                                                                                           | Time (teste automático) e pentest (PT-36)                                              | Cota, castigo até 24 h e memória com teto (limites::Limitador)                                                                                                       |
| **C106**  | Banco: ler, apagar e depositar sem autorização; ajuste torto; prazo acima de 48 h; alterar bloco; ler registros; desligar RLS                                                                                                                                    | Time (teste_do_banco.sh e mutacao_do_banco.sh) e pentest (PT-37)                       | Todos recusados num PostgreSQL real; 12 de 12 defeitos plantados no esquema pegos                                                                                    |
| **C107**  | Rate limit: martelar cada rota, errar a prova em sequência, tentar criar rota sem limite                                                                                                                                                                         | Time (teste automático e catraca) e pentest (PT-38)                                    | Recusado antes da assinatura; castigo que dobra; catraca reprova rota sem limite (autenticacao::Porteiro::autorizar)                                                 |
| **C108**  | Sentinela: cada invariante violada (rota sem limite, caminho torto, rota repetida, janela, prazos, hora, chaves de recebimento, esperas, Argon2id) e cada sabotagem do banco                                                                                     | Time (teste automático, mutação e mutacao_do_banco.sh)                                 | Toda violação trava com código, lugar e conserto (sentinela::conferir_valores; verificar_banco_na_partida.sql)                                                       |
| **C109**  | Proteção do repositório: afrouxar aprovações, tirar assinatura obrigatória, dar passe livre                                                                                                                                                                      | Time (teste da lógica do vigia) e pentest (PT-40)                                      | Vigia reprova toda variação (vigia-da-protecao.yml)                                                                                                                  |
| **C110**  | Selo: portão ausente, reprovado ou inventado; artefato trocado; versão antiga; data no futuro; uma assinatura só; mesma chave duas vezes; chave fora das fixadas; assinatura de outro selo                                                                       | Time (teste automático, mutação e ferramenta de linha de comando)                      | Toda variação recusada; servidor e app não funcionam (selo::conferir; sentinela::autoverificar_com_selo)                                                             |
| **C111**  | Governança: regra inexistente, inativa ou com passe livre; commit sem assinatura; sem pedido de mudança; autor aprovando a si mesmo; mesma pessoa duas vezes; aprovação retirada                                                                                 | Time (portoes_de_governanca.py --teste) e pentest (PT-42)                              | Toda variação reprova o portão e impede o selo                                                                                                                       |
| **C112**  | Selo com duas de três chaves; organização sem segundo fator obrigatório, com um administrador só ou com uma pessoa só na segurança                                                                                                                               | Time (teste automático e portoes_de_governanca.py --teste, 20 casos) e pentest (PT-44) | Selo recusado; portão reprovado                                                                                                                                      |
| **C113**  | Metadados: ligar caixas entre dias; conferir endereço no disco antigo; ler registro de acesso como operador; inferir o IP pelo tamanho do registro                                                                                                               | Time (teste automático e mutação) e pentest (PT-45)                                    | Nenhuma ligação; disco antigo inútil depois de 48 h; registro ilegível e de tamanho fixo (metadados)                                                                 |
| **C114**  | Testes clássicos de API: martelar rotas, origem refletida/curinga/null, token e parâmetros na URL, enumeração pela mensagem de erro, moldura, dado demais na resposta                                                                                            | Time (teste automático e mutação) e pentest (PT-46)                                    | Toda variação recusada ou com resposta genérica (autenticacao)                                                                                                       |
| **C115**  | Ambiente: sem bloqueio de tela, patch velho, teclado de terceiros, opções de desenvolvedor, fontes desconhecidas                                                                                                                                                 | Time (teste automático e mutação) e teste em aparelho real                             | Teto rebaixado e aviso com como resolver; nada sai do aparelho (ambiente)                                                                                            |
| **C116**  | Chave do dia do servidor: usar a de ontem, uma raiz só, raiz estranha, mesma raiz duas vezes, certificado de outro dia, chave trocada                                                                                                                            | Time (teste automático e mutação)                                                      | Toda variação recusada (chaves_do_servidor::conferir)                                                                                                                |
| **C117**  | Disco do servidor: abrir sobra de bloco depois da destruição da chave do dia; usar o bloco em outra caixa; adulterar                                                                                                                                             | Time (teste automático e mutação)                                                      | Nada abre (metadados::ChavesDeIndice::abrir_do_disco)                                                                                                                |
| **C118**  | Ataque vermelho: nomes com letras de imitação e alfabetos misturados, valores extremos, 20 mil pedidos aleatórios no porteiro                                                                                                                                    | Time (tests/ataque_vermelho.rs e mutação) e pentest (PT-48)                            | Todos recusados ou respondidos com código genérico; nenhuma queda                                                                                                    |
| **C-WA**  | Piloto de 30 dias, 20 contas de terceiros no Cloud Phone                                                                                                                                                                                                         | Time                                                                                   | Taxa de bloqueio documentada e aceita                                                                                                                                |

## Verificar antes de usar

- **VF1** LM1/LM2 — política vigente do Google (Play Integrity) e texto atual da regra 4.2.7 da Apple.

- **VF2** CO5 — disponibilidade de máquinas virtuais confidenciais com atestação nos provedores escolhidos e licença do Google para Play Store em Android virtual.

- **VF3** KS1 — viabilidade contratual de um custodiante independente em outra jurisdição.

- **VF4** EC2/FP2 — nível de API do Android para detecção de gravação de tela e disponibilidade de módulo FIPS 140-3 com ML-KEM.

- **VF5** A9 — estado atual das certificações e da arquitetura da Hypori antes de qualquer comparação pública.

- **VF23** A0R — Marco Civil, art. 15: validar com advogado se guardar o IP do retransmissor atende a lei, e o regulamento de segurança dos registros (Decreto nº 8.771/2016).

- **VF24** A0V — conferir no texto compilado da LGPD os dispositivos citados na Política de Privacidade.

- **VF25** A0Q — conferir em cada navegador o suporte a Clear-Site-Data e o descarte da página ao sair.

- **VF21** A0J — conferir no texto vigente das regras da App Store (3.1.1 e 3.1.3) se resgatar ficha comprada fora é aceito no país de venda.

- **VF22** A0J — conferir a política de pagamentos do Google Play em cada país de venda.

- **VF20** A0G — confirmar no iPhone e no Android que salvar pelo componente autorizado não deixa cópia em pasta temporária do app (RZ10).

- **VF19** A0F — em setores regulados (financeiro, saúde, governo), validar com jurídico as obrigações de guarda de comunicações antes de oferecer os prazos curtos nas contas corporativas.

- **VF16** A0C — validar com jurídico, em cada país de venda, a recuperação de dados de contas corporativas e o texto do aviso ao funcionário (proteção de dados e legislação trabalhista).

- **VF17** A0E — preços do Completo e do Empresas contra o custo real da máquina confidencial por usuário, medido no piloto.

- **VF18** A0D — confirmar em iOS e Android que um app adulterado ou reassinado não acessa as chaves guardadas pelo app original.

- **VF14** NOT3 — pedido à Apple da permissão de filtragem de notificações (com.apple.developer.usernotifications.filtering), explicando o uso; a Apple concede caso a caso e pode recusar.

- **VF15** A6M — regra de funções escondidas da App Store (2.3.1) para o modo discreto no iPhone, e regras do adquirente e das bandeiras para o descritor neutro na fatura.

- **VF12** A0B — situação atual do Signal Login (cadastro sem telefone) no blog oficial do Signal, e demais recursos de WhatsApp e Signal na data de qualquer comparação pública.

- **VF13** ANO10/ANO14 — maturidade do Arti para uso em celular e latência medida; retransmissores de chamada por operadores independentes.

- **VF11** A6J — citação do Regulamento Delegado (UE) 2018/448, art. 5 (vinculação dinâmica), a conferir na fonte oficial antes de usar em material externo; requisitos do provedor de assinatura qualificada escolhido.

- **VF9** ENV0b — conferir na fonte oficial da Mozilla o motivo do encerramento do Firefox Send antes de citar publicamente.

- **VF10** SB1 — custo de manter a Sala Blindada numa máquina virtual confidencial; medir no piloto antes de definir em que plano entra.

- **VF8** VM3/VM5 — grau de reversibilidade do deslocamento de tom e viabilidade de conversão neural de voz rodando no aparelho, com latência aceitável para chamada.

- **VF7** ANO3 — consumo real de bateria e de dados do tráfego de cobertura, medido em aparelhos de entrada e de topo.

- **VF6** GR1 — documentação da Apple sobre o alcance da gravação de chamadas a cada versão do iOS, e o equivalente no Android.

- **VF26** PC3 — documentação atual da Apple sobre o App Attest em Mac (DCAppAttestService.isSupported) e eventual abertura no macOS 27 para outros tipos de app.

- **VF27** PC5 — exclusão de captura de janela no Windows (SetWindowDisplayAffinity) e no macOS, em cada versão suportada.

- **VF28** PQ4 — estado do rascunho X-Wing no IETF e decisão do auditor entre ML-KEM-768 (X-Wing) e o combinador próprio com ML-KEM-1024.
