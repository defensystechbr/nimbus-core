# Matriz de ataques e barreiras — o que está coberto, por quem e o que falta

**Versão 37 · 05/10/2026 · Confidencial**

Nenhum sistema prova que resiste a "todos os ataques do universo". Esta matriz diz, para cada classe de ataque conhecida, qual barreira existe, se ela é conferida por máquina com defeito plantado, e o que ainda depende de pessoas ou não está coberto. Classe de ataque nova vira linha nova; linha nunca sai.

| # | Classe de ataque | Barreira | Conferida por máquina, atacada com defeito plantado | Ainda depende de pessoa ou não coberto |
|---|---|---|---|---|
| 1 | Alteração maliciosa do código por uma pessoa de dentro | Proteção de branch como código (principal, todas as branches, tags), duas aprovações humanas, commits assinados, CODEOWNERS | Catraca e vigia (`portoes_de_governanca.py --teste`, 20 casos) | **T0 não feita**: a proteção ainda não está ativa no GitHub |
| 2 | Texto invisível ou invertido ("Trojan Source"), caracteres que imitam outros | Proibição em todo texto do núcleo e dos kits; varredura de força bruta em todos os formatos, inclusive dentro de .docx, .pptx e .zip | `catraca.py` §6h; `varrer_invisiveis.py` (ataca a si mesmo) | Identificadores com letras de outro alfabeto: o compilador Rust avisa, mas o clippy/avisos ainda não reprovam no CI |
| 3 | Cadeia de suprimentos do código (dependência desviada, script de compilação, origem trocada) | Sem `build.rs`, sem `[patch]`/`[replace]`, sem dependência por git ou caminho, sem `.cargo/config`, todo pacote do crates.io com checksum, Cargo.lock obrigatório | `catraca.py` §6i (`testar_catraca.py`) | Pacote legítimo comprometido na origem: só a auditoria diária e a revisão das atualizações pegam |
| 4 | Dependência com vulnerabilidade conhecida | Auditoria de todas as versões do Cargo.lock contra a base RustSec do dia | `auditar_dependencias.py` (`testar_auditoria.py`, 7 casos) | Vulnerabilidade ainda não publicada |
| 5 | Ataque ao CI (pedido de fora com segredos, injeção pelo título, ação trocada) | Sem `pull_request_target`/`workflow_run`, sem texto de PR em comando, permissões mínimas, ações por hash, checkout sem credencial, ferramentas com versão fixada | `catraca.py` §6g | Comprometimento do próprio GitHub |
| 6 | Segredo no código | gitleaks (pre-commit, CI e portão de entrega), regras de varredura | Atacado com segredos plantados antes de cada uso | Segredo de formato desconhecido pelo gitleaks |
| 7 | Conferidor cego ou que mente | Todo conferidor atacado com defeito plantado antes de virar prova | `testar_*`, `testar_conferidores.py`, `entregar.py --testar` | Classe de defeito que ninguém pensou em plantar |
| 8 | Número velho citado como prova | Identidade do código dentro de cada registro, faixas de mutação sem buraco | `medir_provas.py` (atacado em `testar_conferidores.py`) | — |
| 9 | Pacote entregue diferente do provado | Portão de entrega, manifesto, identidade do código conferida no pacote extraído | `entregar.py` | Declaração só no CI: **ainda não rodou lá** |
| 10 | Execução de código por arquivo de dados (eval sobre o protótipo) | `eval`/`exec`/`compile` proibidos nas ferramentas | Regra de varredura; `testar_conferidores.py` | — |
| 11 | Leitura da tela, gravação, captura, sobreposição, acessibilidade espiã | `vigia_de_sensores` (VSE1–VSE8), escudo nativo | Mutação 288 de 288 no núcleo | **Escudo nativo não compilado**; iPhone não informa microfone ou câmera de outro app — o Nimbus previne tomando o microfone com sessão exclusiva (a confirmar em aparelho) |
| 12 | Microfone ou câmera usados por outro app junto com o Nimbus | Android: silenciamento e indisponibilidade da câmera detectados; iPhone: sessão de áudio exclusiva e câmera bloqueada em segundo plano pelo sistema | Núcleo: VSE6 e VSE8 com defeitos plantados | Teste em aparelho real (T2) |
| 13 | Imitação do app ou do site (caso Privnote) | Certificado do app conferido em tempo constante; endereços parecidos acusados antes de abrir | `autenticidade` (AUT1–AUT5), fuzzing de 20 alvos | Nome e domínio oficiais ainda não definidos; loja e distribuição oficial dependem do lançamento |
| 14 | Resíduos no aparelho (teclado, área de transferência, backup, miniaturas, notificações) | RSD1–RSD12; área de transferência só é limpa, nunca escrita | Regras de varredura do código nativo (log, proteção retirada, área de transferência) | Testes RSD em aparelho real (T2) |
| 15 | Celular desbloqueado e infectado durante o uso | — | — | **Limite declarado**: quem controla o aparelho vê o que o usuário vê |
| 16 | Administrador da máquina de produção | Duas pessoas, imagem assinada | — | **Limite declarado** (A7): fecha com computação confidencial |
| 17 | Engenharia social contra a equipe (pedido falso de mudança, "urgência") | Ninguém sozinho libera nada; selo com três chaves; auditor de fora | Portões do selo | Treinamento e processo humano |
| 18 | Engenharia social contra o usuário (link falso, pagamento trocado) | Acusação de domínio imitador; pagamento assinado; nunca existir versão web que baixe código | AUT1–AUT5 | Educação do usuário; canais oficiais definidos no lançamento |
| 19 | Mudança automática perigosa ("conserto automático" usado como gatilho) | Bloquear é automático; consertar é de gente (SEN4) | Catraca | — |
| 20 | Erro do próprio ambiente de medição (pasta compartilhada, disco cheio, máquina que reinicia) | Pasta de compilação própria obrigatória; identidade no começo e no fim; espaço mínimo; declaração só no CI | `testar_verificar.py` (9 situações); `entregar.py --testar` | — |

## O que só gente consegue fazer

Revisão humana independente (T0), auditor externo, pentest humano (PT-01 a PT-48), teste em aparelhos reais e teste com pessoas reais. Máquinas pegam o que foi pensado; essas etapas existem para pegar o que ninguém pensou.
