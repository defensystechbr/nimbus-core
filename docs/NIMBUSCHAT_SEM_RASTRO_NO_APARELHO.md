# NimbusChat sem rastro no aparelho

**Versão 37 · 06/10/2026 · Confidencial · Para os engenheiros do NimbusChat (versão pessoal)**

Pedido do dono: o NimbusChat não pode deixar no aparelho chaves, códigos, bytes, pedaços de conversa, autenticações nem nada que uma ferramenta de perícia (online ou offline) consiga recuperar; o que precisar existir tem de ser apagado de forma irrecuperável. Este documento diz o que é possível, o que não é, como cada caminho de vazamento é fechado, quem garante e como provar.

## 1. A verdade técnica primeiro

1. **"Riscar por cima" não funciona de forma confiável em celular.** A memória flash grava cada alteração em outro lugar e só limpa os blocos antigos quando o controlador decide. Sobrescrever um arquivo não garante que a versão antiga sumiu. O método que funciona é o **apagamento criptográfico**: nada é gravado sem cifra, a chave fica no chip seguro ou só na memória, e apagar a chave torna tudo o que foi gravado com ela irrecuperável, esteja onde estiver no chip.
2. **O sistema operacional registra o uso dos apps** em bases próprias: quando o app abriu, por quanto tempo, que houve conexão, que chegou uma notificação. Nenhum app consegue apagar esses registros. Uma perícia pode provar **que** o Nimbus foi usado e **quando**; não pode ler **o que** foi conversado.
3. **Celular desbloqueado e infectado durante o uso** vê o que a pessoa vê. Limite declarado.
4. **Memória da interface:** o texto mostrado na tela passa pelas estruturas do iOS e do Android, que não podem ser zeradas pelo app. O núcleo zera o que é dele; a camada nativa mantém o texto o menor tempo possível e cobre a tela ao sair. Limite declarado e reduzido, não eliminado.

## 2. Ameaças consideradas

- Aparelho apreendido **desligado** ou reiniciado (antes do primeiro desbloqueio).
- Aparelho apreendido **ligado e bloqueado** (depois do primeiro desbloqueio): o sistema já liberou as chaves dele, mas a chave do Nimbus só existe com o app aberto e desbloqueado (RSD15).
- **Cópia completa do sistema de arquivos** por ferramenta de perícia.
- **Backup** em nuvem ou no computador.
- **Perícia online:** o que o servidor guarda (só blocos cifrados, no máximo 48 horas na caixa).

## 3. Cada caminho de vazamento, a defesa e a prova

| # | Onde poderia ficar | Defesa | Quem garante | Como se prova | Situação |
|---|---|---|---|---|---|
| R1 | Conteúdo em disco | O núcleo nunca grava em disco; tudo o que o app guarda é bloco que o núcleo selou, em classes de tamanho fixo | Núcleo | `tests/sonda_de_vazamento.rs`: canário em UTF-8, UTF-16, base64, hexadecimal e invertido nunca aparece nos bytes guardados; o tamanho não revela conteúdo nem tipo; a sonda é atacada com o canário plantado | **Provado no núcleo** (6 de 6, 06/10/2026) |
| R2 | Banco de dados, preferências, pastas temporárias e de cache, galeria, armazenamento externo | Proibidos; só o contêiner cifrado do núcleo | Varredura `nimbus-nativo-sem-disco-em-claro` | Plantio de cada caso (7 de 7 pegos) | **Barreira ativa** |
| R3 | Cache de rede, cookies, credenciais | A camada nativa não abre conexão; toda rede passa pelo núcleo, sem cache | Varredura `nimbus-rede-fora-da-rota` (ela barrou uma sessão de rede que eu tinha escrito no código nativo) | Plantio | **Barreira ativa** |
| R4 | Gravação de voz e câmera em arquivo | Gravador e câmera só na memória; o núcleo cifra antes de qualquer gravação | Varredura (proíbe o gravador que escreve arquivo) + teste em aparelho | Plantio + RSD6 | Barreira ativa; teste na T2 |
| R5 | Chaves no aparelho | Chave no chip seguro (StrongBox no Android, Secure Enclave e Keychain no iPhone), só com o aparelho desbloqueado, só neste aparelho, nunca em backup, nunca exportável | Escudo nativo (`ChaveDoAparelho`, `ChavesDoAparelho`) + catraca exige as travas | Catraca + teste em aparelho | Código de partida; compilar na T2 |
| R6 | Keychain que sobrevive à desinstalação (iPhone) | Na primeira execução sem o contêiner do Nimbus, apaga todo item de instalação anterior | Escudo nativo (`apagarChavesDeInstalacaoAnterior`) | Teste: instalar, usar, desinstalar, reinstalar e procurar itens | Código de partida |
| R7 | Chave do Nimbus em memória com o aparelho apreendido ligado | A chave só existe com o app aberto; ao ir para o segundo plano, o núcleo zera a chave | Escudo nativo (`GuardiaoDaChave`) + núcleo (`memoria`) | Despejo de memória do processo depois de sair do app | Código de partida |
| R8 | Memória do núcleo indo para o disco (alguns Android gravam memória na flash) | Segredos do núcleo em memória travada (não vai para o disco) e zerada ao fim | Núcleo (`memoria`) | Testes do núcleo | Feito no núcleo |
| R9 | Teclado, área de transferência, captura e seletor de apps, notificações, backup do sistema, miniaturas, busca do sistema, registros do sistema, ditado, acessibilidade, memória da interface | RSD1–RSD12 | Escudo nativo + varreduras (log, compartilhar, proteção retirada, área de transferência) | Testes RSD em aparelho real | Barreiras ativas; testes na T2 |
| R10 | Compartilhar, exportar, imprimir, arrastar para outro app | Não existe; proibido no código | Varredura `nimbus-nativo-sem-compartilhar` | Plantio | **Barreira ativa** |
| R11 | Sinais de presença (digitando, entregue, online, visto por último) | Não existem; só "visualizada", opcional e recíproco | Núcleo + varredura `nimbus-sem-sinais-de-presenca` | Plantio + defeitos plantados | **Feito** |
| R12 | Página ou cliente web | Não existe | Varredura `nimbus-sem-link-web` + catraca | Plantio | **Feito** |
| R13 | Desinstalação | O contêiner some; o que sobrar na flash está cifrado com chave destruída | Apagamento criptográfico | Teste em aparelho | Depende de R5 a R7 |
| R14 | Servidor (perícia online) | Só blocos cifrados, caixa por no máximo 48 horas, nada sobre quem fala com quem | Núcleo + banco com RLS | Testes do núcleo e do banco | Feito no núcleo |

## 4. O que é essencial e o que não é

Toda função que cria superfície de vazamento foi revista. As marcadas "decisão do dono" não foram mudadas sem ele.

| Função | Vazamento que cria | Situação |
|---|---|---|
| Link e página web | Código do servidor rodando a cada acesso | **Retirada** |
| "Entregue", "digitando", "online", "visto por último" | Sinal de presença saindo do aparelho | **Retirados** |
| "Visualizada" | Um sinal, só se os dois quiserem | Opcional, desligado por padrão |
| Denúncia | Só a conta denunciada e o motivo | Mínimo exigido pela Apple; sem conteúdo |
| Compartilhar, copiar, encaminhar, imprimir | Conteúdo saindo do app | **Não existem** |
| Prévia de link | Conexão a um endereço | Desligada |
| Agenda de contatos do telefone, localização | Dados do aparelho | Nunca lidos |
| Notificação de mensagem e de ligação | No iPhone, a Apple sabe que chegou algo e quando (nunca o conteúdo) | **Decidido:** Android e computador por conexão própria, sem o Google; iPhone pelo serviço da Apple, sempre vazio e cifrado, em cadência fixa; texto genérico; nenhuma prévia; nunca o nome de quem liga |
| Histórico guardado cifrado no servidor | Blocos cifrados fora do aparelho | **Decidido: não existe na versão pessoal.** A catraca do kit reprova qualquer uso no app; ao trocar de aparelho, o novo começa vazio |
| Mensagens de voz, fotos, vídeo, chamadas | Arquivos e registros do sistema | Essenciais; só na memória e cifrados (R4); chamada atendida não deixa registro |

## 5. Como os técnicos provam (perícia de teste)

1. Aparelhos de teste próprios (um iPhone e um Android), nunca de uso pessoal.
2. Usar o app com **canários**: textos únicos em mensagens, nomes, notas, arquivos e chamadas.
3. Extrair o sistema de arquivos inteiro e procurar os canários em todas as formas (as mesmas da sonda do núcleo), dentro e fora do contêiner do app: preferências, caches, pastas temporárias, galeria, registros do sistema, base de notificações, cache do teclado.
4. Procurar chaves e autenticações: Keychain e Keystore antes e depois de desinstalar e reinstalar.
5. Despejo de memória do processo com o app aberto e depois de ir para o segundo plano.
6. Rodar as ferramentas abertas de análise de artefatos do sistema usadas em perícia para listar o que o sistema registrou sobre o app. O esperado: registros de **uso** (limite do item 1.2), nenhum **conteúdo**.
7. Todo canário achado vira falha bloqueante, teste permanente e barreira nova.

## 6. Verificar antes de usar

1. Nomes de API do iOS e do Android na versão mínima suportada (o escudo nativo ainda não foi compilado).
2. Se cada modelo de Android de teste grava memória na flash ("memória virtual" ou similar) e o comportamento do núcleo nesse caso.
3. Se a Apple concede a permissão de filtragem de notificações (necessária para descartar em silêncio as notificações de cobertura no iPhone).
