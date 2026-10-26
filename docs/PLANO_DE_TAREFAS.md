# Plano de tarefas — NimbusChat

**T1 · Ponte do núcleo para Swift e Kotlin** (UniFFI). Aceite: selar e abrir envelope, conferir assinatura híbrida e calcular `liberacao::pode` nas duas plataformas.
**T2 · Esqueleto dos apps iOS e Android** sem SDK de terceiros; tela protegida; seleção de texto desligada; teclados de terceiros bloqueados; escudo nativo (`apps/*/escudo/EscudoNativo.kt`, `apps/*/Escudo/EscudoNativo.swift`) compilado e ligado ao `vigia_de_sensores::decidir` pela ponte. Aceite: print sai preto; varredura limpa; testes VSE1–VSE8 e RSD1–RSD12 aprovados em aparelho real, Android e iPhone.
**T3 · Servidor** (ver `servidor/README.md`). Aceite: nenhum conteúdo legível no banco; registros de acesso só com IP do retransmissor.
**T4 · Protocolo:** libsignal (conversas) e OpenMLS (grupos), depois do parecer sobre a licença AGPL.
**T5 · Conta:** abertura, boas-vindas, criar conta, frase, PIN e PIN de coação, tentativas de PIN, recuperar. Provas C3, C19, C66.
**T6 · Conversas:** lista, conversa, opções de envio (visualização única, temporizadores, não lidas, voz, cartão), regras da conversa. Provas C59–C64, C67.
**T7 · Chamadas** por retransmissor (sem CallKit, sem telefonia), modulador de voz, grupo, histórico só das não atendidas.
**T8 · Contatos:** convite, palavras de confirmação, salvar contato, como você aparece, personas, Escudo. Provas C68, C69.
**T9 · Ajustes, privacidade, aparelhos, rede** (IP nunca exposto) **e notificações**. Sem cache offline e sem rede mesh; prazos só com `expiracao::hora_confiavel`. Provas C73, C91, C92.
**T10 · Licença, pagamento, conta corporativa e acesso suspenso** (`docs/PAGAMENTOS.md`). Prova C74.
**T11 · Enviar documento, envio direto, link, página do destinatário e NimbusDrop lacrado** (`direto::lacrar`, `direto::abrir_lacrado`). Provas C70–C72, C93.
**T11B · App de computador (macOS e Windows)** como aparelho vinculado, nunca acima do amarelo (`integridade::avaliar`): chave no TPM 2.0 ou Secure Enclave, disco cifrado, janela fora da captura, vínculo que cai em 7 dias. Provas C89, C90.
**T12 · Infraestrutura** (ver `infra/README.md`).
**T13 · Provas com auditor independente e publicação nas lojas** (decisões humanas).
