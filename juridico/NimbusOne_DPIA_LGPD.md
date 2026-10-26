# Relatório de Impacto à Proteção de Dados Pessoais (RIPD/DPIA) — NimbusOne e NimbusChat

**Status: ESBOÇO para revisão jurídica — não é peça final, não deve ser publicado nem apresentado a terceiros sem validação de advogado.** Versão 0.1 · 12/10/2026 · Elaborado internamente, com apoio de IA, a partir da documentação técnica e de privacidade já verificada do projeto (ver seção 12, fontes).

---

## 0. Como ler este documento

Este é o entregável identificado como pendente (item 3.2 do rastreador de entregáveis, `NimbusOne_DPIA_LGPD.md`) e genuinamente ausente até hoje — confirmado pela varredura registrada em `PENDENCIAS_E_DECISOES.md`, seção 44, e pelo próprio modelo de ameaça do projeto (`NimbusOne_Modelo_Ameaca_STRIDE_LINDDUN.md`, "Verificar antes de usar", item 3: "LINDDUN-N (conformidade) depende do DPIA... está no Lote 2"). Este esboço fecha essa lacuna com uma minuta completa e tecnicamente fundamentada — não com uma afirmação de conformidade. Toda afirmação jurídica abaixo vem de leitura direta da fonte primária (Lei nº 13.709/2018, no Planalto) ou de um guia oficial de elaboração de RIPD, nunca de memória ou de blog de terceiro; onde a fonte não foi suficiente para fechar um ponto, isto está marcado explicitamente em **[VERIFICAR]**, não preenchido por estimativa.

Postura editorial deliberada (pedido explícito do dono: expor o mínimo aceitável): este documento não afirma conformidade que ainda não existe, não promete uma certificação que ainda não foi contratada, e não usa linguagem absoluta ("totalmente seguro", "impossível de violar") em nenhum ponto — tanto por precisão técnica quanto porque uma afirmação de conformidade incorreta é, ela mesma, um risco jurídico maior do que a lacuna honestamente registrada.

---

## 1. Identificação

| Campo | Valor |
|---|---|
| Controlador (Operador, LGPD art. 5º, VI) | **[A PREENCHER]** — a pessoa jurídica ainda não está nomeada com CNPJ em nenhum documento do projeto; `Termos_de_Uso.md` e `Politica_de_Privacidade.md` referem-se a ela só como "a pessoa jurídica identificada no rodapé do site oficial". Este RIPD não pode ser considerado completo sem este campo preenchido — é o primeiro item da seção 11 (ações pendentes). |
| Produtos cobertos | NimbusChat (mensageria cifrada) e NimbusOne (cofre pessoal, celular virtual, Sala Blindada, aprovações corporativas) — uma conta, dois domínios de confiança (ver `ARQUITETURA_NIMBUSCHAT_E_NIMBUSONE.md`) |
| Encarregado (DPO, LGPD art. 41) | **[A PREENCHER]** — `Politica_de_Privacidade.md` §8 só diz que "o canal do encarregado... está indicado no site oficial"; a Lei exige identidade e contato **publicamente** divulgados (art. 41, §1º) — isto ainda não está nomeado em nenhum lugar deste projeto. Segundo item pendente da seção 11. |
| Elaborado por | Equipe interna do produto, com apoio de IA, sob supervisão do fundador | 12/10/2026 |
| Revisado por | **[A PREENCHER — pendente de advogado]** | — |
| Aprovado por | **[A PREENCHER — cabe exclusivamente ao controlador, nunca a quem elabora ou revisa]** | — |
| Próxima revisão obrigatória | Antes do lançamento público; depois, a cada mudança relevante de finalidade, dado, base legal, agente ou compartilhamento, ou incidente de segurança — e pelo menos uma vez ao ano, mesmo sem mudança (boa prática) |

---

## 2. Por que este RIPD existe — base legal e critério de obrigatoriedade

**Definição legal (LGPD, Lei nº 13.709/2018, art. 5º, XVII):** relatório de impacto é "a documentação do controlador que contém a descrição dos processos de tratamento de dados pessoais que podem gerar riscos [às liberdades civis e aos direitos fundamentais], bem como medidas, salvaguardas e mecanismos de mitigação de risco" (citação literal do artigo, conferida no Planalto em 12/10/2026).

**Art. 38 e parágrafo único:** a Autoridade Nacional de Proteção de Dados (ANPD) pode determinar ao controlador que elabore RIPD, inclusive para dados sensíveis; o conteúdo mínimo exigido é a descrição dos tipos de dados coletados, a metodologia de coleta e segurança, e a análise do controlador sobre suas próprias medidas de mitigação de risco.

**Critério de alto risco (não é uma imposição direta da ANPD neste momento, é autoavaliação preventiva):** a ANPD não determinou, até onde esta pesquisa alcançou, que o Nimbus elabore um RIPD. Este documento é proativo, pelo mesmo raciocínio que o guia oficial de elaboração de RIPD do Governo Federal usa para órgãos públicos (adaptado aqui para uma empresa privada — **[VERIFICAR]** se a Resolução CD/ANPD nº 2/2022 citada abaixo é a referência certa e vigente, já que ela trata originalmente de agentes de pequeno porte, não de um critério geral de alto risco; esta pesquisa não leu o texto integral da resolução, só a citação de um guia de terceiro — ver seção 12):
- **Critério geral:** tratamento em larga escala OU com afetação significativa de direitos.
- **Critério específico:** qualquer um entre: tecnologia emergente, vigilância de área pública, decisão automatizada, dados sensíveis, ou dados de crianças/adolescentes/idosos.

Aplicado ao Nimbus: o produto processa comunicações privadas de pessoas físicas, com a meta de operar em larga escala (milhares a milhões de contas, se o lançamento for bem-sucedido), e usa técnicas de privacidade pouco comuns no mercado (custódia mútua com chave efêmera por item, recuperação k-de-n por guardiões, apagamento criptográfico, anti-forense). Isto plausivelmente cruza tanto o critério geral quanto o específico ("tecnologia emergente"), o que justifica elaborar este RIPD de forma preventiva, mesmo sem determinação explícita da ANPD. **Não trata dados de crianças por política (seção 8), mas sem verificação técnica de idade — ver ressalva na mesma seção.**

---

## 3. Papéis dos agentes de tratamento (LGPD art. 5º, VI e VII)

| Papel | Quem | Fundamento |
|---|---|---|
| Controlador | O Operador (pessoa jurídica — **[A PREENCHER]**), que decide finalidade e meios do tratamento | Define a Política de Privacidade, os Termos de Uso e a arquitetura do produto |
| Operador (no sentido técnico da LGPD — não confundir com "Operador" como o produto chama a si mesmo nos Termos) | Provedores de infraestrutura (hospedagem, retransmissores) que guardam apenas blocos cifrados, sem chave | `Politica_de_Privacidade.md` §3; nunca recebem dado em claro |
| Titular | A pessoa física usuária da conta | Sujeito dos direitos da seção 7 |

**Nota de precisão, para não confundir o leitor:** os Termos de Uso chamam a empresa de "Operador" como nome de marca/papel comercial; a LGPD usa "operador" para um papel técnico diferente (quem trata em nome do controlador). Neste RIPD, "Operador" maiúsculo, sem outra qualificação, segue o uso do produto (= controlador); "operador" minúsculo, quando aparecer, segue o sentido técnico da lei. Recomendação para a revisão jurídica: considerar renomear um dos dois usos nos documentos públicos, para não gerar uma leitura equivocada por um fiscal ou auditor.

---

## 4. Contexto e descrição do tratamento

### 4.1 O que nunca é coletado (minimização por desenho, não por promessa)

Direto de `Politica_de_Privacidade.md` §1, já publicada: telefone, e-mail, documento, nome civil, foto de perfil, agenda de contatos (fica só no cofre cifrado do aparelho), conteúdo de qualquer natureza (cifrado ponta a ponta, sem chave no servidor), com quem a pessoa conversa, nomes dados a contatos/álbuns, localização, identificadores de publicidade, analytics ou rastreadores de terceiros, e o endereço IP real do aparelho (sempre por dois retransmissores independentes ou Tor).

### 4.2 O que é tratado, por quê, e por quanto tempo

| Dado | Finalidade (LGPD art. 6º, I) | Base legal provável | Retenção |
|---|---|---|---|
| Chaves públicas da conta | Permitir que outras pessoas cifrem mensagens para o titular | Execução do contrato (art. 7º, V) | Enquanto a conta existir |
| Blocos cifrados (mensagens, cofre, aba trancada) | Entregar/guardar conteúdo que só o titular e seus contatos conseguem abrir | Execução do contrato | Mensagens: até 48h; custódia mútua/cofre: até o prazo escolhido, teto de 182 dias (`custodia_mutua::PRAZO_MAXIMO_DIAS`) |
| Registros de acesso (data, hora, IP do retransmissor — nunca o do aparelho), cifrados | Cumprir obrigação legal | **Obrigação legal, art. 7º, II — Marco Civil da Internet, art. 15** | 6 meses, depois apagados; só entregues mediante ordem judicial |
| Aceite dos Termos de Uso (versão + data, assinado pela chave da conta) | Provar qual versão vale para a conta | Cumprimento de obrigação legal / exercício regular de direitos (art. 7º, II e VI) | Enquanto a conta existir + prazo de prescrição |
| Ficha anônima de licença | Liberar recursos pagos sem ligar pagamento à identidade | Execução do contrato | Enquanto a licença valer |
| Dados de pagamento | Processar a compra | Execução do contrato — **mas o AGENTE é a loja/processador, não o Operador** (ver seção 4.3) | Conforme a política da loja/processador — fora do controle direto do Operador |
| Dados de conta corporativa (nº de licenças, lista de signatários autorizados, assinada pela organização) | Prestar o serviço contratado pela organização | Execução do contrato | Enquanto durar o contrato |

Esta tabela resume; o inventário técnico completo e mantido vivo de cada operação (inclusive as dos mecanismos mais novos — custódia mútua, pedido de guarda, comando remoto, recuperação por guardiões) é `CONTRATO_DA_PONTE.md`, que deve ser consultado junto com este RIPD, nunca substituído por ele — este documento resume para fins de LGPD; aquele é a fonte técnica viva.

### 4.3 Compartilhamento (LGPD art. 7º, caput — toda hipótese precisa de base legal própria)

Com ninguém, exceto: (a) processadores de pagamento e lojas de aplicativo, no limite da compra — o Operador recebe só a confirmação; (b) autoridades, somente mediante ordem judicial, e somente o que existe (não é possível entregar o que nunca foi coletado); (c) provedores de infraestrutura, só blocos cifrados, nunca chaves.

### 4.4 Transferência internacional (LGPD art. 33) — ponto que exige leitura jurídica cuidadosa, não tratamento superficial

`Politica_de_Privacidade.md` §6 afirma que retransmissores/infraestrutura "podem estar em outros países" e que, como só trafegam blocos cifrados sem chave, "nenhum conteúdo legível é transferido". **Isto é tecnicamente correto, mas não resolve sozinho a pergunta jurídica.** O próprio `Termos_de_Uso.md` (§4, item 3) já reconhece isto com precisão, e este RIPD adota a MESMA posição, pelo motivo de exposição mínima (dado cifrado, mas vinculável a uma chave de conta, tende a ser tratado como dado pessoal pseudonimizado, não como dado anonimizado — a diferença importa: LGPD art. 12 exige impossibilidade de reversão por meios técnicos razoáveis E disponíveis para que um dado deixe de ser pessoal; como o próprio titular consegue decifrar com sua chave, a reversibilidade existe, só não está ao alcance do Operador): tratar blocos cifrados vinculados a uma conta como dado pessoal para efeitos de transferência internacional, mesmo sendo ilegíveis para o Operador — a leitura mais conservadora, não a mais confortável. Sob essa leitura, a transferência internacional de blocos cifrados ainda precisa de uma das hipóteses do art. 33 (incisos I a IX) para ser regular — a mais provável aplicável aqui é o inciso II (salvaguardas do próprio controlador — cláusulas contratuais, certificação) ou o inciso IX (hipóteses do art. 7º, aqui execução de contrato). **[VERIFICAR COM ADVOGADO]:** qual hipótese do art. 33 fundamenta formalmente esta transferência, e se os países onde a infraestrutura/retransmissores operam exigem cláusula contratual específica.

### 4.5 Direitos dos titulares (LGPD art. 18; Marco Civil, art. 7º, X)

`Politica_de_Privacidade.md` §4 já declara os direitos; o diferencial do Nimbus é que a maior parte se exerce diretamente no aplicativo, sem precisar de um canal de atendimento humano para ser efetivo — apagar a conta destrói as chaves e torna o conteúdo no servidor permanentemente ilegível (equivalente técnico ao direito de eliminação, com prova verificável, não só uma promessa de processo interno). **Residual honesto:** os registros de acesso (6 meses, obrigação legal) não são elimináveis a pedido do titular antes do prazo — isto é uma exceção legal expressa (art. 16, II, da própria LGPD, cumprimento de obrigação legal pelo controlador), não um erro de desenho; vale registrar isto de forma explícita para quem ler só a tabela de direitos e estranhar a exceção.

---

## 5. Os dez princípios do art. 6º, um a um (nunca só transcritos — a exigência do guia oficial é mostrar COMO cada um é cumprido)

| Princípio | Como é cumprido | Lacuna conhecida |
|---|---|---|
| I. Finalidade | Cada dado da tabela 4.2 tem uma finalidade específica declarada; nenhum dado é tratado para propósito não informado | Nenhuma identificada |
| II. Adequação | O tratamento é compatível com o contexto informado (app de comunicação/custódia cifrada) | Nenhuma identificada |
| III. Necessidade | A minimização da seção 4.1 é o próprio produto, não uma política à parte — o que não é coletado não pode, por desenho, ser usado além do necessário | Nenhuma identificada |
| IV. Livre acesso | A pessoa vê, no próprio app, o que existe sobre a conta (ex.: tela 8, lista de itens na aba trancada, com prazo visível) | Não existe hoje uma tela única "meus dados" consolidando TODOS os itens da tabela 4.2 num só lugar — só o conteúdo, espalhado pelas telas de cada função |
| V. Qualidade dos dados | Dados técnicos (chaves, blocos) não têm "qualidade" no sentido de precisão factual sobre a pessoa — o princípio se aplica pouco a este produto por desenho | Nenhuma identificada |
| VI. Transparência | Política de Privacidade, Termos de Uso e "Verdade desta tela" (prática já formalizada no protótipo, tela a tela) | Nenhuma identificada |
| VII. Segurança | Seção 6 abaixo | Ver seção 6 |
| VIII. Prevenção | Modelo de ameaça STRIDE/LINDDUN já mantido e versionado | Nenhuma identificada |
| IX. Não discriminação | Não há tratamento de dado para fins de discriminação — o produto nem tem os dados que normalmente alimentam isso (demografia, localização, comportamento) | Nenhuma identificada |
| X. Responsabilização e prestação de contas | Este próprio RIPD, `CONTRATO_DA_PONTE.md`, `PENDENCIAS_E_DECISOES.md` e o pacote de prontidão para auditoria são a evidência | Falta a auditoria INDEPENDENTE de fato contratada — autoavaliação não basta para o princípio X (mesma ressalva já escrita em `NimbusOne_Audit_Readiness.md`, item "Verificar antes de usar") |

---

## 6. Segurança técnica e organizacional (LGPD art. 46, 47, 49)

Medidas já implementadas e testadas (fonte: `Politica_de_Privacidade.md` §5, `NimbusOne_Modelo_Ameaca_STRIDE_LINDDUN.md`): criptografia pós-quântica híbrida (ML-KEM-1024 com X25519; ML-DSA-87 com Ed25519), cifra de conteúdo XChaCha20-Poly1305/AES-256-GCM, senhas protegidas por Argon2id, apagamento criptográfico, núcleo com suíte de testes automatizados (725 testes, 0 falhas na última medição desta sessão — ver `PENDENCIAS_E_DECISOES.md` para o comando e a data exatos de cada medição, nunca um número solto sem procedência), proteção desde a concepção (art. 46, §2º — privacy/security by design já é a premissa do núcleo, não uma camada adicionada depois).

**O que ainda falta para o princípio da Responsabilização (art. 6º, X) estar completo, nomeado sem meias-palavras:** auditoria criptográfica formal por firma independente — ainda não contratada (`NimbusOne_Audit_Readiness.md`, confirmado nesta mesma pesquisa); pentest completo com firma externa — mesmo status; e a comunicação de incidente de segurança do art. 48 (ver seção 9) ainda não tem um procedimento formalmente escrito e testado especificamente para notificação à ANPD e aos titulares, distinto do CI de segurança técnico já existente.

---

## 7. Gestão de riscos — reaproveitando LINDDUN, nunca reescrevendo do zero

O modelo de ameaça do projeto já cobre privacidade formalmente (LINDDUN, não só STRIDE) — ver `NimbusOne_Modelo_Ameaca_STRIDE_LINDDUN.md`, seção 2. Este RIPD adota os mesmos sete riscos como a análise de risco exigida pelo art. 38, em vez de duplicar o trabalho com uma metodologia nova: ligabilidade, identificabilidade, não-repúdio indesejado, detectabilidade, divulgação de informação, inconsciência do usuário e descumprimento de política — cada um já com defesa, prova técnica (quando aplicável) e risco residual **honestamente nomeado**, não escondido (o próprio documento de ameaça já segue esta mesma ética, citada aqui por coerência interna, não por conveniência). O residual mais relevante para EFEITOS DE LGPD, não só de segurança técnica, é "N — Descumprimento de política": a prática tem de bater com o que está publicado, e isto é, precisamente, o que este RIPD — e sua revisão por advogado — existe para fechar.

**Risco adicional, próprio deste RIPD, não coberto no STRIDE/LINDDUN original porque é um risco jurídico, não técnico:** o risco de uma afirmação de conformidade (deste ou de outro documento) ser publicada antes da revisão de advogado, e divergir do que um fiscal da ANPD entenderia da lei — mitigado pelo próprio status deste documento (ESBOÇO, não publicável) e pela lista de pendências da seção 11.

---

## 8. Crianças e adolescentes (LGPD art. 14; Lei nº 15.211/2025 — ECA Digital)

Política atual: "O Serviço é destinado a maiores de 18 anos" (`Politica_de_Privacidade.md` §7) — uma exclusão por política, não por verificação técnica de idade. **Isto é uma lacuna real, nomeada, não uma falha escondida:** nenhum mecanismo de verificação de idade foi encontrado nesta pesquisa em nenhum documento do projeto. A Lei nº 15.211/2025 (ECA Digital), **[VERIFICAR]** — identificada nesta pesquisa como lei brasileira recente e especificamente dedicada a direitos digitais de crianças e adolescentes, mas seu texto integral não foi lido nesta sessão —, pode impor deveres adicionais a qualquer serviço digital acessível por menores, independentemente da política de idade mínima declarada. Recomendação para a revisão jurídica: confirmar se a mera declaração "18+", sem verificação técnica, é suficiente para o Nimbus sob a ECA Digital, ou se algum mecanismo adicional (ainda que simples, como autodeclaração ativa no cadastro) é exigível antes do lançamento.

---

## 9. Resposta a incidentes de segurança (LGPD art. 48)

A lei exige comunicação à ANPD e aos titulares em prazo razoável, com descrição dos dados afetados, das pessoas envolvidas, das medidas técnicas, dos riscos e das medidas de reversão. O CI de segurança do projeto (`NimbusOne_CI_de_Seguranca.md`) já cobre a parte técnica (SAST, DAST, SBOM, builds assinados) — **mas um procedimento de comunicação formal de incidente, com os papéis de quem decide notificar, em que prazo e com qual texto, não foi encontrado como documento próprio nesta pesquisa.** Ação pendente, listada na seção 11.

---

## 10. Versionamento, aprovação e publicização

Este documento é revisado a cada mudança relevante de finalidade, dado tratado, base legal, agente ou compartilhamento, ou incidente de segurança — e pelo menos uma vez ao ano mesmo sem mudança. A aprovação final cabe exclusivamente ao controlador (não a quem elabora ou revisa — boa prática adotada do guia oficial consultado, seção 12). Publicização: recomenda-se publicar uma versão resumida (transparência ativa), suprimindo detalhes operacionais de segurança que um atacante poderia explorar — a versão interna completa permanece íntegra e nunca pública.

---

## 11. Ações pendentes antes deste RIPD poder ser considerado completo (não apagar esta lista ao aprovar — é o checklist de saída do rascunho)

1. **Nomear o controlador** com razão social e CNPJ — hoje só "a pessoa jurídica identificada no rodapé do site oficial", em nenhum lugar nomeada de fato nos documentos do projeto.
2. **Nomear o encarregado (DPO)**, com identidade e contato publicamente divulgáveis (LGPD art. 41, §1º — exigência legal explícita, não só boa prática).
3. **Revisão jurídica completa** deste esboço por advogado, antes de qualquer uso externo — nenhuma afirmação aqui substitui essa revisão.
4. **Confirmar a hipótese do art. 33** (transferência internacional) que fundamenta a operação dos retransmissores/infraestrutura fora do Brasil (seção 4.4).
5. **Avaliar a ECA Digital (Lei nº 15.211/2025)** contra a política atual de idade mínima sem verificação técnica (seção 8).
6. **Escrever o procedimento formal de comunicação de incidente** (art. 48), com papéis, prazos e modelo de texto (seção 9).
7. **Contratar auditoria criptográfica e pentest independentes** — sem isso, o princípio da Responsabilização (art. 6º, X) permanece incompleto, por mais sólida que seja a autoavaliação interna.
8. **Confirmar a aplicabilidade exata da Resolução CD/ANPD nº 2/2022** citada na seção 2 — esta pesquisa leu a citação de um guia de terceiro (ainda que oficial), nunca o texto integral da resolução.
9. **Decidir e publicar uma tela "meus dados"** consolidada, para fechar a pequena lacuna do princípio IV (Livre acesso) notada na seção 5.

---

## 12. Fontes consultadas nesta rodada (12/10/2026) — âncora de cada afirmação jurídica

- Lei nº 13.709/2018 (LGPD), texto consultado diretamente em planalto.gov.br — [Lei nº 13.709, de 14 de agosto de 2018](https://www.planalto.gov.br/ccivil_03/_ato2015-2018/2018/lei/l13709.htm). **Ressalva honesta sobre esta própria fonte:** a página raspada apresentou problema de codificação de caracteres acentuados e parece conter mais de uma versão sobreposta de alguns incisos (art. 5º, VIII e XIX, alterados por leis de 2025/2026) — os artigos usados neste RIPD (5º XVII, 6º, 33, 38, 41, 46–49) não exibiram nota de revogação/veto/vigência diferida, exceto art. 41 §4º (vetado, irrelevante para este documento); ainda assim, antes de qualquer uso externo, confirmar o texto consolidado (`L13709compilado.htm`, mencionado pela própria página como a versão de referência) com um advogado.
- Guia para Elaboração de RIPDs (PPSI 2.0, Secretaria de Governo Digital) — [guia-para-elaboracao-de-ripds-v0-1](https://www.gov.br/governodigital/pt-br/privacidade-e-seguranca/ppsi-2.0/arquivos/guia-para-elaboracao-de-ripds-v0-1-31072026.pdf) — usado como referência ESTRUTURAL (a sequência de seções deste RIPD segue este guia) — não como norma vinculante para uma empresa privada, já que o guia é escrito para órgãos públicos sob a Portaria SGD/MGI nº 9.511/2025.
- `Politica_de_Privacidade.md`, `Termos_de_Uso.md`, `NimbusOne_Modelo_Ameaca_STRIDE_LINDDUN.md`, `NimbusOne_Audit_Readiness.md`, `CONTRATO_DA_PONTE.md`, `PENDENCIAS_E_DECISOES.md` — lidos ou reconferidos diretamente nesta sessão, pacote `pacote_final/`.

---

## Verificar antes de usar

1. **Nenhuma linha deste documento substitui revisão de advogado especializado em proteção de dados** — é um esboço técnico-jurídico de apoio, elaborado com IA, não uma peça aprovada.
2. **A classificação de blocos cifrados como dado pessoal (não anonimizado), seção 4.4**, é a leitura mais conservadora adotada deliberadamente — mas é uma interpretação, não um texto literal da lei aplicado sem ambiguidade; confirmar com advogado antes de basear qualquer comunicação externa nela.
3. **A aplicabilidade da Resolução CD/ANPD nº 2/2022 ao critério de alto risco (seção 2)** foi tomada de um guia de terceiro (ainda que de órgão oficial), nunca do texto integral da resolução — reconferir na fonte direta da ANPD antes de citar externamente.
