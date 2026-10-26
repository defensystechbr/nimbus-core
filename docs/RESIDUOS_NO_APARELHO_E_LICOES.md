# Resíduos no aparelho, lições de mercado e o que falta provar

**Versão 37 · 04/10/2026 · Confidencial · Complementa `NimbusOne_Metadados_O_Que_Sobra.md` e o `CONTRATO_DA_PONTE.md`**

Este documento é para os engenheiros das tarefas de app (T1 em diante). Cada linha é um lugar onde conteúdo ou metadado pode sobrar no celular **fora** do núcleo Rust, a defesa exigida e o teste de aceite. O núcleo já cobre o que é dele (cifra, memória travada, apagamento) e, desde 04/10/2026, a decisão contínua do que pode ficar aberto (`vigia_de_sensores`, VSE1–VSE8) e a defesa contra imitação (`autenticidade`, AUT1–AUT5). A camada nativa de partida está em `apps/*-android/escudo/EscudoNativo.kt` e `apps/*-ios/Escudo/EscudoNativo.swift`: **não compilada na máquina que gerou o pacote**; ela só lê os sinais e executa o que o núcleo decidir. Estes itens entram como critério de aceite da T2. As APIs citadas são as oficiais de cada sistema; conferir na documentação da versão mínima suportada antes de implementar.

## 1. Onde o conteúdo pode sobrar no aparelho

| # | Lugar | Defesa exigida | Teste de aceite |
|---|---|---|---|
| RSD1 | Teclado (aprendizado de palavras, sugestões) | Android: `IME_FLAG_NO_PERSONALIZED_LEARNING` em todo campo de texto. iOS: bloquear teclados de terceiros (`application(_:shouldAllowExtensionPointIdentifier:)` recusando `.keyboard`) e desligar correção e sugestão nos campos sensíveis | Digitar palavra inédita no app; ela não aparece como sugestão em outro app |
| RSD2 | Área de transferência | O Nimbus nunca põe conteúdo nela: não existe copiar mensagem (LIB1). Ao sair e ao trocar de app, ela é limpa (RZ6: `clearPrimaryClip()` no Android 9+, `UIPasteboard.general.items = []` no iOS). A regra `nimbus-area-de-transferencia` reprova qualquer escrita | Tentar copiar dentro do app: não há como; sair do app: a área de transferência está vazia |
| RSD3 | Captura de tela e imagem do seletor de apps | Android: `FLAG_SECURE` em toda janela com conteúdo. iOS: não há como bloquear a captura; cobrir a tela ao perder o foco (a foto do seletor de apps) e avisar o remetente quando houver captura ou gravação de tela detectada | Abrir o seletor de apps: só aparece a capa. Android: captura sai preta |
| RSD4 | Notificações | O servidor de push só recebe um aviso vazio; o texto é decifrado no aparelho e, por padrão, a notificação diz só "Nova mensagem" | Interceptar o envio ao serviço de push: nenhum conteúdo nem remetente |
| RSD5 | Cópia de segurança do sistema (iCloud, Google) | iOS: arquivos marcados como fora do backup e com proteção completa; Android: `allowBackup=false` e regras de extração vazias | Restaurar backup em aparelho novo: nada do Nimbus volta |
| RSD6 | Miniaturas, decodificadores e galeria | Mídia decodificada só em memória; nunca gravar na galeria nem em pasta compartilhada; sem cache de miniatura em disco | Procurar arquivos de imagem no contêiner do app depois de ver uma foto: nenhum |
| RSD7 | Pré-visualização de links | Desligada por padrão. Se o usuário ligar, a prévia é gerada no aparelho de quem envia e vai cifrada; quem recebe nunca acessa o endereço sozinho | Receber um link: nenhuma conexão sai do aparelho de quem recebe |
| RSD8 | Busca do sistema, sugestões e atalhos (Spotlight, Siri, atalhos do Android) | Não indexar nada; não doar atividades nem intenções | Buscar no sistema uma palavra de uma conversa: nada |
| RSD9 | Registros do sistema e relatórios de falha | Sem conteúdo nem identificador em log; relatório de falha desligado ou sem dados do usuário | Ler os registros do aparelho durante o uso: nada identificável |
| RSD10 | Ditado por voz | Verificar se o ditado do sistema envia áudio para servidor na versão mínima suportada; se enviar, desligar o ditado nos campos de conteúdo ou avisar | **Não verificado:** depende da versão do sistema e da configuração do aparelho |
| RSD11 | Serviços de acessibilidade (Android) | Detectar serviços de acessibilidade de terceiros ativos e avisar: eles conseguem ler a tela | Ativar um serviço de teste: o aviso aparece |
| RSD12 | Cópias de texto na memória da camada nativa | Strings imutáveis do Swift/Kotlin não podem ser apagadas: o conteúdo sensível atravessa a ponte como bytes e é zerado logo após o uso (regras do `CONTRATO_DA_PONTE.md`) | Despejo de memória do processo após fechar a conversa: o texto não aparece |

Limite que continua escrito: celular desbloqueado e infectado durante o uso vê o que o usuário vê. Nenhum item acima muda isso.

## 2. O que os quatro posts ensinam

**Lista de 20 requisitos mínimos.** HTTPS, senhas com hash, MFA, limite de uso, validação e sanitização de entrada, injeção de SQL, controle de acesso, segredos, CORS, logs, criptografia, dependências e monitoramento já estão cobertos no projeto. Os mais fracos nos documentos são **migrações com volta atrás (rollback)**, **expiração de sessão** e **cópias de segurança do servidor com teste de restauração**: entram no `CHECKLIST_DE_ENTRADA_EM_PRODUCAO.md` (seção F) como itens a provar. O item "PMP" do vídeo não tem significado claro e não foi considerado.

**PrivNote (mensagem que some depois de lida).** A lição não é a cifra, é a **imitação**: em 2020 um site quase idêntico (privnotes.com) lia as mensagens e trocava endereços de bitcoin por outros, e como a nota some depois de lida a vítima não tinha como conferir. Para o Nimbus:
- o aplicativo só é distribuído assinado, por loja oficial ou página própria com hash publicado, e o app mostra como conferir;
- nenhuma versão web que baixe o código do servidor a cada acesso, porque o servidor poderia entregar outro código;
- todo dado de pagamento (conta, chave, carteira) que passe pelo Nimbus vai assinado por quem enviou, e "aprovar pagamento" mostra o conteúdo assinado;
- "sem rastro" nunca é prometido: quem recebe pode fotografar a tela.

**Strix (agentes de IA que fazem pentest).** Útil como **mais uma barreira**, nunca como prova única. Regras se for usado:
- instalar por versão fixada do pacote, nunca por `curl ... | bash`;
- o repositório oficial é `usestrix/strix`; há muitas cópias com o mesmo nome;
- roda isolado, sem segredo de produção e contra ambiente de teste;
- o código é enviado ao provedor de IA configurado: só com provedor contratado e permitido pela política de dados, ou modelo local;
- achado vira teste permanente; ausência de achado não prova nada.

**Os 6 documentos antes de programar.** PRD, TRD, fluxo do app, UI/UX, esquema do backend e plano de implementação: o projeto já tem os seis (`01` a `06`), mais o contrato da ponte, a matriz de rastreabilidade e o manual técnico.

## 3. Verificar antes de usar

1. Os nomes de API citados na seção 1, na documentação oficial da versão mínima de iOS e Android que o Nimbus vai suportar.
2. Se o ditado do sistema (RSD10) envia áudio para fora do aparelho nessa versão.
