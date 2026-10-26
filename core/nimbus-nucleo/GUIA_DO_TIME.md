# Guia do Time — construir o NimbusOne rápido e sem brecha

Leia junto com o **Manual Técnico v37** (era v31 nesta linha, desatualizado por várias rodadas — corrigido na Rodada 26, 12/10/2026) e o **protótipo** (cada tela diz quais funções chamar).
Tudo em português: nomes de módulos, funções, variáveis, testes, commits e documentação.

## 1. O que vamos construir (decisão final)

| App | Contém | Onde roda |
|---|---|---|
| **NimbusChat** | Só comunicação: conversas 1:1 e em grupo, chamadas de voz e vídeo 1:1 e em grupo, mensagens de voz, fotos, vídeos e arquivos dentro da conversa, temporárias, visualização única, Escudo de Conversa, modulador de voz | No aparelho (modo efêmero) |
| **Plano corporativo (os dois apps)** | Console da empresa: convidar, suspender sem ler, reativar (2 de 3 administradores), recuperar dados de conta suspensa com a chave mestra (2 de 3 administradores + 2 de 3 guardiões), registro de auditoria | Regras no módulo `corporativo`; chave mestra no módulo `custodia` |
| **NimbusOne** | O celular blindado: **Cofre Pessoal** (agenda, notas, álbuns, arquivos, senhas, todos com validade) + **celular virtual** (Nimbus OS) + **Sala Blindada** + **assinatura e aprovações** | Cofre Pessoal no aparelho; celular virtual, Sala e aprovações de alto valor na máquina confidencial |

- Mesma conta e mesmo `nimbus-nucleo` nos dois. Passagem de conteúdo entre os apps **só pelo cofre cifrado**, nunca pela folha de compartilhar do sistema.
- **Não existe NimbusBank.** Bancos entram pelo programa Nimbus Trusted Banking e pelas aprovações do NimbusOne.
- No iPhone, o celular virtual **não é oferecido** (regra 4.2.7(e) da App Store não aceita cliente leve de app na nuvem) — só Android e computador; app nativo para iPhone só onde a Apple permitir distribuição alternativa de app notarizado. Decisão do dono em 06/10/2026 (`MUDANCA_ENVIO_SO_DENTRO_DO_APP.md`, item 14; registro 73). O resto do NimbusOne (Cofre Pessoal) é app nativo nos três sistemas.

## 2. Não reinvente: use componentes prontos (validar licença e versão antes de adotar)

| Necessidade | Componente | Licença (validar) | Observação |
|---|---|---|---|
| Conversa 1:1 | libsignal | AGPL-3.0 | Parecer jurídico na Fase 0 antes de usar |
| Grupos | OpenMLS (MLS, RFC 9420) | MIT | Conferir maturidade |
| Rota rápida em duas etapas | Oblivious HTTP (RFC 9458) com retransmissor de terceiro; ECH (RFC 9849) | Padrões abertos | Contratar o operador da 1ª etapa |
| Tor (escolhas Automático e Sempre Tor) | Arti (Tor em Rust), com pontes | MIT / Apache-2.0 | Conferir suporte em celular |
| Notificações sem Google no Android | Conexão própria ou UnifiedPush | Padrão aberto | Medir bateria |
| Chamadas | WebRTC | BSD | Mídia cifrada ponta a ponta |
| Chamadas em grupo | Servidor de mídia seletivo com SFrame (ex.: LiveKit) | Apache-2.0 | O servidor repassa mídia cifrada sem abrir |
| Ponte Rust ↔ Swift/Kotlin | UniFFI | MPL-2.0 | Um só núcleo para iOS, Android e computador |
| Pós-quântico | `ml-kem` (troca de chaves) e `ml-dsa` (assinatura), já no núcleo | MIT / Apache-2.0 | Suíte alinhada à CNSA 2.0; bibliotecas ainda sem auditoria formal publicada — conferir antes do lançamento |
| Senhas e PINs | `argon2` (Argon2id), já no núcleo | MIT / Apache-2.0 | Parâmetros da RFC 9106 |
| Chaves de passe | APIs nativas (AuthenticationServices, Credential Manager) | Sistema | Biometria nunca sai do chip |

## 3. Ordem de construção (trilhas em paralelo)

1. **Fase 0 (1ª semana útil):** pareceres (libsignal AGPL, patente Confide, termos Meta, App Store), contratação do auditor, repositório e pipeline (`.github/workflows/nimbus-ci.yml` já pronto).
2. **Fase 1 — núcleo:** partir do `nimbus-nucleo` (725 testes rápidos — era 571 nesta linha, desatualizado por várias rodadas, corrigido na Rodada 26, 12/10/2026 — + fuzzing de 50 mil entradas por alvo e 2 milhões no fuzzing profundo, ambos no ./verificar.sh + prova de tempo constante; provas de aparelho sob demanda; 353 defeitos plantados — era 321 nesta linha, mesma correção —, todos exigidos pegos; dependências auditadas; Rust 1.91; tudo em ./verificar.sh). Ligar ao Swift/Kotlin por UniFFI. Rede pelo módulo `rede` (`rede::decidir` em toda chamada de rede): Automático por padrão, Sempre Tor e Rápido (sem Tor) como escolhas; circuito Tor mantido aquecido em segundo plano para a troca ser imediata. Identidade sem telefone, sessões, registro público. Medir velocidade desde o primeiro dia (seção 8).
3. **Fase 2 — NimbusChat** (trilha A) **e Cofre Pessoal do NimbusOne** (trilha B), em paralelo, sobre o mesmo núcleo.
4. **Fase 3 — lançamento do NimbusChat** após auditoria.
5. **Fase 4 — celular virtual, Sala Blindada e aprovações** (piloto).
6. **Fase 5 — expansão:** Enterprise, bancos, navegador isolado.

Atalho que mais economiza tempo: **cada tela do protótipo já diz a função do núcleo que ela chama.** Construa a tela e ligue a função; não redesenhe a segurança.

## 4. Regras inegociáveis (reprovam a entrega)

- Nada de criptografia própria. Nada de SDK de terceiros. Nada de log com conteúdo, IP, nome de arquivo ou identificador.
- Nada escrito em disco em claro; nomes de arquivos e pastas sempre dentro do `pacote::selar_item`.
- Todo tráfego pela camada `rede` do núcleo — nunca pela rede do sistema (a varredura reprova). Toda operação pergunta a rota a `rede::decidir`; nunca decidir rota na interface. Chamadas por retransmissores independentes.
- Notificações só pela cadência fixa do módulo `notificacao`, com texto genérico.
- Mensagem nunca sai: não existe botão de copiar, encaminhar ou salvar para mensagem, nem print. Arquivo só sai por `liberacao::pode`, com a autorização assinada do autor; para fora, só pelo componente `ExportacaoAutorizada` — em qualquer outro lugar a varredura reprova.
- Toda regra de histórico passa por `conversa::combinar` (a mais restritiva entre os participantes); o app nunca grava chamada, marca ou conversa fora do que `conversa` decide.
- Sem telefone, SMS, agenda do sistema, localização, identificador de aparelho.
- `cargo test --locked`, compilação rigorosa sem nenhum aviso e `semgrep --config nimbus-regras.yml --error .` com zero ocorrências.

## 5. Blindagem contra código malicioso — de pessoas ou de IA

- **Ninguém sozinho:** ramo `principal` protegido, commits assinados, chave física para quem tem acesso ao repositório. Revisão humana em toda alteração: **duas pessoas** é a meta; o regime vigente é aprovador único (o dono), declarado em `docs/DECISAO_APROVADOR_UNICO.md`.
- **Ninguém publica sozinho:** versão só com 3 de 5 assinaturas (módulo `aprovacao`, tipo Publicação), incluindo o auditor externo; o app só instala versão presente no registro público (módulo `registro`).
- **Dependências travadas:** `Cargo.lock`, `Package.resolved`, lockfile do Gradle; `cargo audit` e `cargo deny` no pipeline; inventário SBOM a cada versão; compilação de release sem acesso à internet.
- **IA no desenvolvimento:** permitida como assistente. Todo código gerado por IA passa pela mesma revisão em dupla, testes e varredura. IA **nunca** recebe segredos, chaves ou dados de usuários, nunca tem permissão de escrita no ramo principal nem acesso à produção. Cuidado com instruções escondidas em arquivos, tickets ou páginas lidas por agentes de IA. Toda biblioteca sugerida por IA é conferida pelo nome exato — atacantes registram nomes parecidos ou inventados por IA.
- **Compilação verificável:** mesma fonte → mesmo binário (Android integral; iOS no nível possível).

## 6. Toda regra nova vem com três testes

Para cada regra de segurança nova: um teste de exemplo, uma propriedade (lei para qualquer entrada) e um
mutante no `verificacao/mutantes.py` que prove que o teste pega o defeito. Mutante vivo reprova a entrega.

## 6b. Definição de pronto de qualquer tarefa

Requisito do manual citado no commit · testes novos passando · varredura com zero ocorrências · revisão de duas pessoas · nenhuma dependência nova sem aprovação e SBOM atualizado · tela conferida contra o protótipo.

## 7. Validação pelo Claude (primeira camada, antes do auditor)

Envie o repositório em .zip ao fundador. Eu rodo os testes, o exemplo, a varredura e reviso cada alteração contra o manual e o protótipo. Não substitui o auditor independente (prova C1).

## 8. Velocidade é requisito de segurança

Se o app for lento, o usuário desiste — e segurança sem usuário não protege ninguém. Metas a medir desde a Fase 1,
em aparelho de entrada e de topo, e revisar no piloto:

- Mensagem de texto entregue em até 1 segundo em 95% dos envios, no modo rápido.
- Abertura do app até a lista de conversas em até 2 segundos.
- Consumo de bateria do tráfego de cobertura e da cadência de notificações abaixo de um limite definido no piloto.
- Sempre Tor: medir e mostrar ao usuário, com honestidade, que é mais lento.
- Automático: a troca para o Tor nos momentos de identidade deve levar no máximo alguns segundos, com circuito aquecido.

Estas são metas de projeto, não resultados medidos.
