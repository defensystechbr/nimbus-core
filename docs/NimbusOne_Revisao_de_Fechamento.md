# Revisão de fechamento — respostas, melhorias, ferramentas e limites

**Versão 37 · Confidencial**

## 1. A tabela de "quem consegue", melhorada

| Objetivo | Uma pessoa sozinha? | Mínimo combinado |
|---|---|---|
| Ler conteúdo já enviado, pelo servidor | Não | **Impossível mesmo com todos** — não existe chave no servidor |
| Ler conteúdo antigo a partir do disco ou de cópias do servidor | Não | **Impossível** — conteúdo cifrado de ponta a ponta e, por cima, selado com chave do dia destruída em 48 h |
| Falsificar a hora ou respostas do servidor | Não | 2 de 3 guardiões das chaves-raiz; e a chave do dia vale só um dia |
| Pôr código malicioso pelo caminho oficial de liberação | Não | 3, uma de fora da empresa |
| Rodar código malicioso com acesso de administrador à máquina de produção | **Sim, hoje** (ataque vermelho, A7) | 1 — sem conteúdo; fecha com acesso de duas pessoas, infraestrutura imutável e computação confidencial |
| Publicar app malicioso | Não | 3, uma de fora |
| Afrouxar o GitHub sem ninguém perceber | Não | 3 |
| Recuperar dados de funcionário | Não | 4, e 24 h visíveis a todos os administradores |
| Perceber, no instante da entrega, que "um endereço do dia, sem dono conhecido, recebeu um bloco" | Sim, quem opera o servidor | 1 — sem conteúdo, remetente ou IP; nada disso fica guardado |
| **Ligar** isso a uma pessoa | Não | 2 empresas diferentes (retransmissor e servidor); com Tor, impossível |
| Ler os registros exigidos por lei | Não | 2 de 3 custodiantes **e** ordem judicial; depois de 6 meses, impossível |

## 2. "Não dá para tornar impossível para todos ver as informações do servidor?"

O **conteúdo** já é impossível para todos. Para os **metadados guardados**, agora também: o disco só tem índice cego e blocos selados com chave do dia, destruída em 48 h — inclusive as sobras que discos de estado sólido não apagam de verdade. O único ponto que não dá para zerar por software é o **instante da entrega** dentro da memória do servidor (o carteiro precisa ler o endereço para entregar). Os três caminhos para cobrir também esse instante estão em `NimbusOne_Metadados_O_Que_Sobra.md`, com custo e a confiança nova que cada um exige.

**Quando a autoridade pedir:** o Operador cumpre a ordem e entrega o que existe — registros selados (com a chave jurídica e a ordem judicial) e dados de licença. Conteúdo, contatos, IP de aparelho e histórico não existem para entregar.

## 3. "Essa é a melhor maneira? Tem algo melhor?" — cada item, com verdade

| Item | É o melhor hoje? | O que existe de melhor e o que fizemos |
|---|---|---|
| Caixa que muda todo dia | É um bom equilíbrio, não o máximo | O máximo é endereço novo a cada mensagem, que obriga o aparelho a buscar muito mais endereços (bateria e banda). Com os lotes de endereços falsos, o ganho de trocar a cada mensagem é pequeno. Fica como opção para a Sala Blindada |
| Índice cego com chave destruída em 48 h | Era bom; ficou melhor | **Acrescentado nesta revisão:** o bloco no disco é selado de novo com chave do dia (`selar_no_disco`), o que cobre as sobras físicas de discos de estado sólido |
| Registros legais selados | Era bom; ficou melhor | **Acrescentado como regra:** chave jurídica por mês, com as partes destruídas depois de 7 meses — o prazo legal de 6 meses passa a ser cumprido pela matemática, não pela boa vontade |
| Ninguém sozinho liga metadado a pessoa | Sim, com operadores independentes | Melhor ainda: computação confidencial com atestação (decisão aberta) e rede de mistura na Sala Blindada |
| Auditor independente com a terceira chave | Necessário; software não substitui julgamento | Software complementa: testemunhas independentes do registro público (vários servidores de terceiros que assinam o estado do registro) e verificadores de build reproduzível. Eles provam "o que foi publicado", não "se é seguro" |
| Administrador aplicar a proteção e gerar chaves offline | Sim | Melhor: commits assinados com chave física (que exige toque humano) e acesso ao GitHub por login único da empresa com chave física; fica como recomendação no guia |
| Revisão do advogado | Obrigatória | Melhor: parecer formal assinado sobre o art. 15 do Marco Civil e sobre os Termos; e conferir, a cada mudança, que Termos, Política e código dizem a mesma coisa |
| Auditoria criptográfica e pentest humano | Obrigatórios | Melhor: somar programa de recompensa contínuo e teste automatizado com agentes de IA em ambiente de teste (seção 4); humano continua obrigatório antes de produção |
| Ataques a AMD SEV-SNP e Intel TDX | Sim, conferir antes | Melhor: usar computação confidencial como camada a mais, **nunca** como única — o desenho atual não depende dela |
| Campos e plano do GitHub | Sim | Já está como código e com vigia; o único passo humano é aplicar e conferir o plano contratado |
| Independência das três pessoas com chave | Sim, é o ponto central | Melhor: organizações diferentes, sem relação de subordinação, de preferência em jurisdições diferentes, com cerimônias de chave registradas e troca planejada das pessoas |
| Retomar numa conversa nova | Funciona | Melhor: anexar o pacote, o registro e o `RETOMAR_ESTADO_DO_PROJETO.md`; suas regras permanentes de segurança também ficam salvas nas suas preferências |

## 4. Ferramentas obrigatórias

**No desenvolvimento e no CI (todas já ligadas ou configuradas no pacote):** Rust fixado na versão 1.91.1; compilação estrita (proibição de `unsafe` e verificação de estouro ligada); **clippy ainda não roda no CI — ligar na T1 com `cargo clippy --all-targets --locked -- -D warnings`**; testes, defeitos plantados (mutação), fuzzing e tempo constante; semgrep com 47 regras próprias; gitleaks antes do commit e no CI; RustSec e `cargo-deny`; catraca; Renovate com espera; sentinela diária; vigia da proteção; certificado de procedência; testes do banco num PostgreSQL real.

**Recomendadas para somar (não instaladas aqui):** `cargo-vet` (auditoria registrada de cada dependência); fuzzing contínuo (ClusterFuzzLite); Kani (verificação de modelo nas máquinas de estado); Miri (comportamento indefinido no pouco código de baixo nível); Tamarin ou ProVerif (verificação formal dos protocolos próprios).

**No servidor:** módulo de hardware ou cofre da nuvem; memória travada e despejo de falha desligado; firewall negando por padrão; varredura das imagens e da infraestrutura como código antes de publicar (por exemplo, Trivy); detecção em tempo de execução no sistema operacional do servidor (por exemplo, Falco) — **sem** registrar conteúdo nem dado de usuário.

**No ambiente de teste, para atacar o próprio sistema:** Strix (agentes de IA que atacam e comprovam falhas; código aberto, roda com Docker e uma chave de modelo de IA); MobSF (análise dos apps compilados); Frida e objection (nas mãos do pentest); OWASP ZAP (API do servidor); nuclei e testssl.sh (infraestrutura). **Atenção ao Strix:** ele envia código e respostas a um provedor de IA; só pode rodar contra o ambiente de teste, sem dado real, e com modelo local ou provedor com contrato de não retenção. Os resultados variam entre execuções; é camada a mais, não portão.

**O que vem embarcado nos apps:** só código próprio — núcleo, sentinela, integridade, diagnóstico do ambiente (`ambiente`) e a atestação do próprio sistema (App Attest e Play Integrity). **Nunca embarcar:** antivírus de terceiros, SDK de análise, de falhas, de anúncios ou de configuração remota. Um "varredor do aparelho inteiro" coletaria dados do usuário e, no iPhone, nem é permitido pelo sistema; o diagnóstico local faz o que é possível sem coletar nada.

## 5. Os testes clássicos de API (1 a 7)

Todos respondidos e testados no núcleo — tabela completa em `05_Esquema_Backend_Nimbus.md`, §5: limite de uso em toda rota; nenhum CORS; resposta sem dado algum além do necessário; nenhum token em URL nem vivo depois de sair (não existe token: cada pedido é assinado e vale uma vez); mesma resposta para toda falha de autorização; proteção contra moldura; SQL só com parâmetros e RLS contra acesso a objeto de outro.

## 6. O que eu não consegui fazer aqui — dito com verdade

1. **Rodar o Strix:** este ambiente não tem Docker nem chave de modelo de IA, e rodá-lo enviaria o código a um provedor de IA sem a sua decisão. A configuração está descrita para o time rodar no ambiente de teste.
2. **Testar em aparelho real e medir desempenho:** depende dos apps compilados.
3. **Ativar qualquer coisa no GitHub, na nuvem ou nas lojas:** não tenho acesso; tudo está como código e com conferência automática.
4. **Provar ausência total de falhas:** nenhuma revisão prova; o processo torna falhas raras, pega as que surgem e registra.

## 7. Perguntas que passam a ser feitas a cada pedido

1. Essa é a forma mais segura que existe, ou só a mais conhecida?
2. Se esta camada falhar, qual pega? E se essa também falhar?
3. Quantas pessoas, empresas e chaves precisariam combinar para burlar isto?
4. O que fica guardado, por quanto tempo, e que chave o torna ilegível depois?
5. O que um usuário não autenticado vê agora?
6. Isso coleta algum dado que não precisaria?
7. Um golpista em cada ponta conseguiria usar isto?
8. O que acontece quando o relógio, a rede ou o fornecedor falha?
9. A catraca e os portões pegariam alguém tirando esta proteção?
10. O que o nosso checklist não cobre — e qual teste prova que agora cobre?

## 8. Verificar antes de usar
1. O comportamento atual do Strix (instalação, provedores de IA aceitos, retenção de dados do provedor escolhido) na documentação oficial do projeto.
2. Vulnerabilidades publicadas contra AMD SEV-SNP e Intel TDX antes de decidir pela computação confidencial.
3. Com o advogado: a guarda de 6 meses do art. 15 do Marco Civil cumprida por chave mensal destruída aos 7 meses.
