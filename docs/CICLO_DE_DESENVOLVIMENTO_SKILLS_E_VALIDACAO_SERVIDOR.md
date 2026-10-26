# NimbusChat — Ciclo de desenvolvimento, skills a usar e validação do plano e do servidor

**Data:** 19/10/2026 · **Status:** estudo. Complementa `PLANO_DE_EXECUCAO_MVP_PRIVADO.md`.
**Base:** leitura de `servidor/README.md`, `servidor/banco/esquema.sql`, `docs/05_Esquema_Backend_Nimbus.md`, `NimbusOne_Checklist_de_Seguranca_do_Backend.md`, `core/nimbus-nucleo/Cargo.toml`, `src/autenticacao.rs`, `ROTAS_PUBLICAS_ANALISADAS.json` e verificações por `grep` no código (resultados na seção 3).

---

## 1. Skills por etapa do ciclo

Skills desta sessão que cabem em cada etapa. Onde há mais de uma, a primeira é a principal.

| Etapa | Skills e agentes | Uso no NimbusChat |
|---|---|---|
| 0. Descoberta e decisão | `superpowers:brainstorming`, `grill-me`, `grill-with-docs`, `office-hours`, `gsd-spec-phase` | Fechar as perguntas abertas antes de código. `grill-with-docs` cruza as respostas com `docs/`. |
| 0. Arquitetura e registro | `architecture-decision-records`, `software-architecture-design`, `c4-context`/`c4-container`, `mermaid-expert`, `plan-eng-review` | ADRs das decisões D1 a D14 (em vez de enterrar em `PENDENCIAS_E_DECISOES.md`, 460 KB). Diagramas C4 do caminho do pedido. |
| 1. Modelo de ameaça | `threat-modeling-expert`, `stride-analysis-patterns`, `attack-tree-construction`, `cso` | Refazer o modelo para o MVP (relay único, push, KMP). Hoje a análise assume dois operadores. |
| 1. Spike | `gsd-spike`, `gsd-sketch`, `rust-pro`, `systems-programming-rust-project`, `android-jetpack-compose-expert`, `kotlin-coroutines-expert`, `swiftui-expert-skill` | Compilar o núcleo para Android e iOS, testar KMP, push e proteção de tela. |
| 2. Planejamento por fase | `gsd-ns-workflow`, `gsd-mvp-phase`, `superpowers:writing-plans`, `gsd-plan-review-convergence`, `plan-eng-review`, `autoplan` | Cada fase vira plano com tarefas e critérios de aceite. `gsd-mvp-phase` fatia vertical (um fluxo ponta a ponta por vez). |
| 3. Servidor (Rust + PostgreSQL) | `rust-pro`, `rust-async-patterns`, `backend-architect`, `api-design-principles`, `openapi-spec-generation`, `postgres-best-practices`, `postgresql`, `database-design`, `database-migrations-sql-migrations`, `database-architect` | Servidor em Rust (axum ou similar), esquema e migrações. `api-architect` (agente) para o contrato HTTP. |
| 3. Segurança do servidor | `api-security-best-practices`, `testing-api-security-with-owasp-top-10`, `implementing-api-rate-limiting-and-throttling`, `implementing-secrets-management-with-vault`, `performing-security-headers-audit`, `performing-ssl-tls-security-assessment`, agente `owasp-auditor` | Rate limit sem IP, Vault, TLS, cabeçalhos. |
| 3. Cripto e memória | `zeroize-audit`, `constant-time-analysis`, `sharp-edges`, `memory-safety-patterns`, `implementing-end-to-end-encryption-for-messaging`, `implementing-digital-signatures-with-ed25519` | Conferir zeroização, tempo constante e uso correto de libsignal/OpenMLS. |
| 4. Apps (KMP + ponte) | `mobile-security-coder`, `mobile-design`, `android_ui_verification`, `ios-developer`, `frontend-design-ui-ux`, `design-html`, `design:design-handoff` | UI a partir de `TELAS.md` e do protótipo. Handoff do design (já feito em `/design`) para código. |
| 5. Desenvolvimento diário com IA | `superpowers:test-driven-development`, `tdd`, `superpowers:subagent-driven-development`, `superpowers:using-git-worktrees`, `superpowers:executing-plans`, `gsd-ns-workflow` | Teste que falha antes. Um worktree por tarefa. |
| 5. Governança e supply chain | `git-advanced-workflows`, `github-actions-templates`, `securing-github-actions-workflows`, `gha-security-review`, `implementing-secret-scanning-with-gitleaks`, `implementing-semgrep-for-custom-sast-rules`, `semgrep-rule-creator`, `supply-chain-risk-auditor`, `analyzing-sbom-for-supply-chain-vulnerabilities`, `devops-skills:github-actions-generator` e `-validator` | Reproduzir no CI os 17 portões, semgrep e gitleaks. Renovate com espera. |
| 6. Revisão | `superpowers:requesting-code-review`, `gsd-code-review`, `code-review-security`, `differential-review`, `variant-analysis`, `spec-to-code-compliance`, `review`, `pr-review-toolkit:review-pr`, agente `coderabbit:code-reviewer` | `spec-to-code-compliance` confere código contra `CONTRATO_DA_PONTE.md`. `differential-review` em PR de protocolo. |
| 7. Testes e QA | `qa`, `qa-only`, `e2e-testing`, `testing-patterns`, `performing-fuzzing-with-aflplusplus`, `implementing-fuzz-testing-in-cicd-with-aflplusplus`, agente `qa-engineer` | Aparelho real é obrigatório; automação não substitui. |
| 7. Verificação antes de declarar pronto | `superpowers:verification-before-completion`, `verification-loop`, `gsd-verify-work`, `gsd-secure-phase` | Alinha com a regra "entrega só declarada pelo `entregar.py`". |
| 8. Pentest e auditoria (Fase 6) | agente `cybersecurity-pentest`, `performing-web-application-penetration-test` (API), `performing-android-app-static-analysis-with-mobsf`, `performing-ios-app-security-assessment`, `legal:*`, `gdpr-data-handling`, `privacy-by-design` | Só em sistema próprio, com autorização. Jurídico antes da loja. |
| 9. Infra e operação (Proxmox) | `devops-skills:dockerfile-generator`, `bash-script-generator`, `terraform-generator` (se IaC), `hardening-docker-containers-for-production`, `observability-engineer`, `prometheus-configuration`, `grafana-dashboards`, `incident-runbook-templates`, MCP `ssh-manager` | Observabilidade com Prometheus e Grafana (stack da organização). Todo acesso a servidor pelo `ssh-manager`, com backup e log de auditoria. |
| 10. Entrega e pós-entrega | `ship`, `land-and-deploy`, `canary`, `document-release`, `retro`, `investigate`, `superpowers:finishing-a-development-branch` | `ship` e `land-and-deploy` só depois que o selo de liberação existir. |
| Guardas de segurança do agente | `careful`, `freeze`, `guard`, `unfreeze` | `freeze` no núcleo e em `verificar.sh` quando a IA trabalhar fora deles. |

**Atenção:** nenhuma skill substitui um revisor humano de protocolo (regra 11). `gstack` pede `/browse` para navegação web, não ferramentas de Chrome.

---

## 2. Validação dos passos do plano (achados)

| # | Achado | Gravidade | Correção sugerida |
|---|---|---|---|
| V1 | **Portões não seguram nada.** O cronograma sobrepõe fases: Fase 2 começa na semana 4 (P1 fecha na 5), Fase 3 na 7 (P2 fecha na 8), Fase 4 na 10 (Fase 3 fecha na 11). Se o portão reprova, já se gastou trabalho em cima da base ruim. | Alta | **Decidido (D15, 19/10/2026): sobrepor e aceitar o refazer.** Regras de contenção na seção 5A do plano (etiquetas `depende-de-Px`, teto de 30%). |
| V2 | **A Fase 2 omite metade do servidor.** Ver seção 3: faltam persistência de convites, pré-chaves, aparelhos, fichas e números de uso único; rota de push; entrega de mensagens sem consulta periódica. | Alta | Acrescentar tarefas 2.9 a 2.14 (seção 4). |
| V3 | **libsignal e OpenMLS ainda não estão no núcleo.** `Cargo.toml` não as lista; só aparecem em comentários (`lib.rs`, `prechaves.rs`, `grupo.rs`). D7 diz "encapsular no núcleo", mas isso é trabalho novo, não ajuste. Ver S13. | Alta | Tarefa explícita na Fase 1: "libsignal compila dentro de `nimbus-nucleo` e fala com `envelope`". |
| V4 | **Dependências PQ pré-1.0.** `ml-kem =0.2.1` e `ml-dsa =0.1.1` (RustCrypto) são versões iniciais, fixadas por igualdade. Sem auditoria pública conhecida; verifique no repositório oficial. | Média a alta | Registrar como risco; trocar para libs auditadas quando existirem; teste de vetores oficiais NIST (KAT). |
| V5 | **`blind-rsa-signatures` com advisório RUSTSEC-2023-0071.** `cobranca.rs` comenta o mecanismo. Em MVP sem pagamento o módulo nem precisa existir. | Média | Tirar `cobranca` do build do MVP (ver S15). |
| V6 | **T0 não é executável.** O diretório não é repositório git. | Média | Tarefa 0.2 começa por `git init` e importação revisada. |
| V7 | **Contagem de rotas diverge.** Docs dizem 11 rotas. `ROTAS` no código tem 10. `ROTAS_PUBLICAS_ANALISADAS.json` ainda descreve `/v1/envio/abrir` (Envio web, já retirado). | Média | Corrigir o JSON e os documentos. A catraca deveria ter pego. Perguntar por quê (S18). |
| V8 | **Plano aceita iOS e Android sem decidir distribuição.** TestFlight exige conta paga e revisão de build externa. | Baixa | Tarefa 0.5 já cobre; dar prazo. |
| V9 | **"Revisão por um dos devs" sem critério.** Falta checklist do que é revisar protocolo. | Média | Criar checklist de revisão de cripto (seção 5). |
| V10 | **Medição de bateria e dados fica para depois, mas o push de D12 já gera tráfego.** | Baixa | Medir no spike. |

---

## 3. O servidor: o que existe, o que falta e o que não fecha

### 3.1 O que existe (verificado)

- **Banco:** 3 tabelas (`caixas`, `blocos`, `registros_de_acesso`), 3 papéis (`nimbus_dono`, `nimbus_app`, `nimbus_faxina`), RLS ligada e forçada, prazo de 48 h por restrição, faxina `SECURITY DEFINER`. 116 linhas de SQL, scripts de teste e de mutação.
- **Núcleo de servidor (Rust, biblioteca):** `autenticacao`, `limites`, `sentinela`, `metadados`, `chaves_do_servidor`, `selo`. Eles são funções e estruturas, não um programa.
- **Programa servidor:** não existe. Não há `main`, nem framework HTTP, nem TLS, nem OHTTP, nem pool de conexões.

### 3.2 Achados verificados no código

| # | Achado | Evidência |
|---|---|---|
| S-A | **Todo estado do porteiro vive em memória.** Números de uso único (`vistos`), titulares, limitadores, convites (`Convites.pendentes`) e envio direto (`direto::Caixa.remessas`) são `HashMap` em memória. | `autenticacao.rs:119-121`, `convites.rs:22`, `direto.rs:84`, `limites.rs:24` |
| S-B | **O banco só tem 3 tabelas, mas 4 rotas precisam de persistência.** `convite/resgatar`, `prechave/publicar`, `aparelho/registrar`, `aparelho/revogar` e o conjunto de fichas gastas não têm tabela. | `esquema.sql`: 3 `CREATE TABLE` |
| S-C | **Reiniciar o servidor reabre a janela de repetição.** `vistos` zera, e um pedido gravado vale outra vez por até 60 s. Dois servidores atrás de um balanceador não compartilham `vistos`. | Consequência de S-A |
| S-D | **Nenhuma rota de push.** O D12 (APNs e FCM com payload vazio) exige guardar o token do aparelho ligado a alguma caixa, o que contradiz "o servidor nunca liga duas caixas". | `ROTAS`: 10 entradas, nenhuma de push |
| S-E | **Entrega por consulta periódica.** Só existe `POST /v1/caixa/retirar`; sem fluxo contínuo. O app consulta a cada N segundos, com limite de 120/min por origem. Consome bateria e deixa padrão de tempo como metadado. | `ROTAS` |
| S-F | **Limite por "origem" quando todo mundo vem do mesmo relay.** Com um relay único (D13), todas as requisições têm a mesma origem. Um atacante esgota a cota de todos os usuários. | `docs/05`: "por origem, titular ou ficha; o servidor não vê o IP" |
| S-G | **A RLS não é independente do servidor.** As duas políticas (`caixa_autorizada`, `deposito_autorizado`) leem uma variável de sessão que o próprio app escreve com `set_config`. Quem executar código no servidor define qualquer caixa. A RLS protege contra esquecimento, não contra comprometimento. | `esquema.sql` linhas 31-43. O texto "duas camadas independentes" exagera |
| S-H | **Blocos de até 64 MiB em coluna `bytea`.** Uma linha de 64 MiB passa inteira pela memória do servidor e do banco. Com 120 depósitos por minuto de teto, a memória ou o disco satura facilmente. | `esquema.sql`: `CHECK (octet_length(bloco) BETWEEN 256 AND 67117056)` e `corpo_maximo` de 64 MiB |
| S-I | **Fichas anônimas exigidas para criar caixa e depositar, mas sem pagamento no MVP.** Quem emite a ficha? | `Exigencia::Ficha` em duas rotas; módulo `cobranca` depende de RSA cego |
| S-J | **Caixa muda todo dia.** Cada relação cria uma caixa nova por dia, e criar exige ficha, 10 por minuto. Com 20 contatos, são 20 criações por dia por pessoa. | TRD §4; `ROTAS` |
| S-K | **"Chave do dia certificada por 2 de 3 raízes offline."** Certificar todo dia exige trazer as raízes à tona todo dia ou pré-gerar lotes. Nenhum documento diz como. | `chaves_do_servidor::certificar`; TRD §4 |
| S-L | **"Servidor pede assinatura ao cofre, não a chave"** contra assinatura híbrida Ed25519 + ML-DSA-87. Vault Transit e HSMs podem não suportar ML-DSA. Verifique a documentação atual. | `assinatura.rs`; regra 13 do CLAUDE.md |
| S-M | **Sentinela por minuto derruba o servidor.** Falha fechada vira indisponibilidade com falso positivo. Uma VM no Proxmox, sem alta disponibilidade. | TRD §5 |
| S-N | **Backup contra retenção de 48 h.** Backup e WAL carregam blocos vencidos. A cifra por dia (`selar_no_disco`) resolve se a chave do dia for realmente destruída e nada fora dela guardar o conteúdo. Precisa de teste. | Checklist §4 item 7 |
| S-O | **Entrega de grupos (MLS) pede um serviço de entrega que ordene commits.** O modelo de caixa não ordena nem multiplica. | Marco B |
| S-P | **Registros de acesso do Marco Civil guardam só o IP do relay.** Pouco útil à autoridade e ainda assim exige chave jurídica mensal e destruição em 7 meses, sem cerimônia definida. | `esquema.sql`; TRD §4 |

---

## 4. Tarefas a acrescentar na Fase 2

- [ ] 2.9 Decidir onde vive o estado do porteiro: tabela no PostgreSQL (números de uso único com TTL, fichas gastas) ou Redis. Com servidor único, memória + persistência de `vistos` ao desligar pode bastar; documentar o limite.
- [ ] 2.10 Tabelas e migrações para convites (só hash), pré-chaves, aparelhos, fichas gastas. Reexecutar `teste_do_banco.sh` e `mutacao_do_banco.sh` com os defeitos novos.
- [ ] 2.11 Rota de push, com desenho que não ligue token a caixa (ou aceite o vazamento e o documente).
- [ ] 2.12 Política de limite quando o relay é único: limitar por titular e por ficha, nunca por origem.
- [ ] 2.13 Anexos fora do `bytea`: armazenamento de objetos próprio (disco, MinIO ou Ceph RGW), com envio em partes e teto de memória.
- [ ] 2.14 Emissão de fichas sem pagamento no MVP (ver S15).
- [ ] 2.15 Desenho de TLS, OHTTP, pool de conexões e HA mínima.
- [ ] 2.16 Prova de backup: restauração sem blocos vencidos legíveis.

---

## 5. Perguntas a responder (técnicas)

**Servidor e rede**
- S1. O servidor será um programa único ou vários (porteiro, entrega, faxina)? Qual framework HTTP em Rust, e quantas dependências novas ele traz?
- ~~S2. Onde vive o estado de anti-repetição (`vistos`)? Aceita perder 60 s de proteção a cada reinício no MVP?~~ **Respondida (D16, 2026-10-20): não aceita perder.** Persistir `vistos` no PostgreSQL com expiração. Pendente definir: tabela nova vs. outra estrutura, e custo de escrita por requisição.
- S3. Como o limite funciona com relay único? Por titular e por ficha, e como evitar que um atacante gaste fichas de outros?
- S4. O app consulta (polling) ou o servidor mantém conexão aberta? Qual a cadência e o gasto de bateria aceitável?
- S5. Push: aceita guardar token do aparelho no servidor, ligado a uma caixa, e declarar isso? Ou usa push "acorda-e-consulta" sem identificar a caixa (por exemplo, um token por aparelho, sem relação com caixas)?
- S6. OHTTP: implementa o relay e o gateway, ou usa serviço de terceiros no MVP? Quem opera o relay para que o IP realmente fique escondido?
- S7. Banco: um PostgreSQL só, em VM? Há réplica? Como é o backup, e como se prova que blocos vencidos não sobram nele?
- S8. Anexos: limite de tamanho do MVP (por exemplo, 10 MiB) e onde ficam armazenados?

**Chaves e segredos**
- S9. Como se emite o certificado do dia assinado pelas raízes offline? Lote pré-gerado de 30 dias, cerimônia mensal, ou outro?
- S10. A assinatura híbrida (Ed25519 + ML-DSA-87) do servidor cabe no Vault ou HSM escolhido? Se não, aceita a chave do dia em memória travada (`mlock`) e documenta o limite?
- S11. Quem custodia as 3 raízes offline e a chave jurídica? Há cerimônia escrita e gravada?
- S12. Memória de troca e despejo de falha: como são desligados na VM Proxmox, e quem confere?

**Protocolo e núcleo**
- S13. libsignal e OpenMLS entram como crates dentro do núcleo? Quem escreve o adaptador entre `envelope`/`conversa`/`grupo` e a libsignal, e qual módulo atual é descartado?
- S14. As dependências PQ pré-1.0 (`ml-kem`, `ml-dsa`) são aceitas no MVP? Há teste com vetores oficiais?
- S15. Quem emite as fichas no MVP, já que não há pagamento? Fichas gratuitas geradas pelo operador?
- S16. A rotação diária de caixa e o custo de criar caixas (10/min) valem para o MVP? Qual a quantidade máxima de contatos por pessoa?

**Processo**
- S17. Os 17 portões e a catraca rodam em qual CI? Linux e macOS? Quanto custa o tempo de ~25 min em cada PR?
- S18. Por que a catraca não pegou a divergência de rotas (10 no código, 11 nos documentos, e `/v1/envio/abrir` no JSON)? A catraca está medindo o que se pensa?
- S19. Qual critério de aceite de cada fase é medido em aparelho real, e quem mede?
- ~~S20.~~ Fechada (D15): sobrepor e refazer se o portão reprovar.

**Revisão humana**
- S21. Quem revisa protocolo? Checklist mínimo sugerido: (1) nonce nunca reaproveitado; (2) comparações em tempo constante; (3) segredos zerados com `zeroize`; (4) erros sem conteúdo; (5) estados do ratchet não regridem; (6) entradas com tamanho máximo; (7) uso de apenas funções do núcleo; (8) teste que falha antes.

---

## 6. Ordem de resposta sugerida

1. S20, S2 e S5: mudam o cronograma e o desenho do servidor.
2. S10 e S13: podem bloquear a Fase 1 inteira.
3. S3, S4, S6, S8, S15 e S16: definem o contrato HTTP e o banco.
4. S9, S11 e S12: ritos de operação.
5. S14, S17, S18, S19 e S21: processo e qualidade.
