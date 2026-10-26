# Perguntas que testam o Nimbus — segurança, privacidade, anonimato, criptografia e operação

**Versão 37 · Confidencial**

Cada pergunta tem a situação de hoje: **R** = respondida, com prova ou documento indicado; **P** = parcial (o núcleo responde, falta app, servidor, infraestrutura ou terceiro); **A** = aberta. Toda pergunta aberta vira tarefa; toda resposta nova vira teste. Use esta lista em cada revisão, em cada auditoria e a cada mudança de fluxo — e acrescente perguntas, nunca tire.

## 1. Quem chega sem se identificar

| # | Pergunta | Situação |
|---|---|---|
| 1 | O que um usuário não autenticado consegue acessar? | R — `ACESSO_SEM_AUTENTICACAO.md`, gerado do código |
| 2 | Existe rota que não está no inventário? | R — negar por padrão; igualdade exata (C104) |
| 3 | Uma rota pública vaza se um endereço, convite ou caixa existe? | R — endereços e códigos de 256 bits; análise por rota |
| 4 | Dá para martelar uma rota pública até cansar o servidor? | R — limite antes da verificação cara (C107) |
| 5 | Dá para encher a memória do limitador com origens inventadas? | R — memória com teto, falha fechada (C105) |
| 6 | A resposta de erro muda conforme o motivo e vira oráculo? | P — núcleo distingue internamente; o servidor deve responder genérico (T3) |
| 7 | O tempo de resposta denuncia se algo existe? | A — medir no servidor real (pentest PT-35) |
| 8 | Há cabeçalho, versão ou rodapé que revele tecnologia e versão? | A — conferir na infraestrutura (T12) |

## 2. Autenticação e autorização

| # | Pergunta | Situação |
|---|---|---|
| 9 | Dá para usar a prova de um pedido em outro pedido, rota ou corpo? | R — assinatura cobre tudo (C104) |
| 10 | Dá para repetir um pedido legítimo? | R — número de uso único |
| 11 | Uma chave de caixa abre rota de aparelho? | R — tipo de titular conferido |
| 12 | Revogar um aparelho tem efeito imediato? | R — `Porteiro::revogar` |
| 13 | Dá para acessar a caixa de outra pessoa mesmo com erro no servidor? | R — RLS forçada como segunda barreira (C106) |
| 14 | O computador vinculado chega a ter os poderes do celular? | R — teto amarelo (C89) |
| 15 | Um QR enviado por terceiros vincula aparelho? | R — só pela tela do app (C98) |
| 16 | Clonar o aparelho dá uma sessão paralela que passa despercebida? | R — clone derruba tudo (sessoes) |
| 17 | O administrador corporativo lê conversas? | R — não; recuperação só de conta suspensa, 2 de 3 + 24 h |
| 18 | Existe conta de "suporte" ou "admin" com poder especial na API? | R — não existe rota de administração |

## 3. Entradas

| # | Pergunta | Situação |
|---|---|---|
| 19 | Algum leitor trava, entra em laço ou estoura com entrada malformada? | R — fuzzing de 19 leitores, estouro aritmético interrompe |
| 20 | Existe SQL montado com texto? | R — regra de varredura proíbe |
| 21 | Um nome com caractere bidirecional engana a leitura? | R — saneado (C87) e recusado em nome (C99) |
| 22 | Um arquivo disfarçado executa ou engana pela extensão? | R — tipo real pelos bytes (C40) |
| 23 | Imagem ou documento carregam metadados de quem enviou? | R — limpeza (higiene) |
| 24 | Dá para mandar corpo gigante e esgotar memória? | R — limite de corpo por rota e de envelope |
| 25 | O app aceita entrada de outro app (deep link, intenção do sistema) para ação sensível? | R — vínculo e convite só por tela própria (EO1, EO2) |
| 26 | Resposta de IA entra em algum fluxo? | R — não há IA no produto |

## 4. Criptografia

| # | Pergunta | Situação |
|---|---|---|
| 27 | Quebrar só o algoritmo clássico ou só o pós-quântico basta para ler? | R — híbrido: precisa dos dois |
| 28 | A mesma chave é usada para duas finalidades? | R — HKDF com contexto por uso |
| 29 | Algum nonce pode se repetir com a mesma chave? | R — chave única por envelope; nonce de 192 bits |
| 30 | Chave pública malformada é aceita? | R — verificação FIPS 203 (C96) |
| 31 | Assinatura é determinística e vulnerável a falha induzida? | R — ML-DSA hedged (C103) |
| 32 | Comparação de segredo vaza por tempo? | R — tempo constante (C80) |
| 33 | Quem grava o tráfego hoje e toma o aparelho depois lê o passado? | R — chaves de recebimento destruídas (C102) |
| 34 | E depois de um aparelho comprometido, o futuro volta a ficar seguro? | P — conversas: Triple Ratchet (libsignal); envelopes: chave nova por dia |
| 35 | A biblioteca pós-quântica tem auditoria formal? | A — conferir antes do lançamento |
| 36 | O gerador de números aleatórios pode falhar em silêncio? | R — sentinela confere a cada partida |
| 37 | Existe plano para trocar um algoritmo quebrado? | R — formatos com versão |
| 38 | A implementação passa numa bateria de conformidade independente? | A — rodar Crucible ou equivalente |

## 5. Tempo, relógio e contagem

| # | Pergunta | Situação |
|---|---|---|
| 39 | Mudar a hora do aparelho estende algum prazo? | R — hora assinada pelo servidor + relógio monotônico (C91) |
| 40 | Uma resposta de hora antiga, repetida, é aceita? | R — presa ao desafio de uso único |
| 41 | Adiantar o relógio pula as esperas do PIN? | R — só tempo real conta (C82) |
| 42 | Reiniciar o aparelho zera a espera do PIN? | R — reinício não soma tempo |
| 43 | O prazo de um arquivo pode ser esticado por quem recebe? | R — prazo no dado autenticado |
| 44 | Uma corrida entre dois pedidos simultâneos quebra alguma regra (uso único, entrega única)? | P — estruturas do núcleo com dono único; testar concorrência no servidor real |
| 45 | O que acontece na virada de horário de verão e de ano? | R — tudo em segundos desde a época, sem fuso |
| 46 | Contadores podem dar a volta? | R — aritmética saturada e parada em estouro |

## 6. Privacidade e anonimato

| # | Pergunta | Situação |
|---|---|---|
| 47 | O servidor sabe quem fala com quem? | P — só caixa e horário de entrega; endereço por relação; retransmissores |
| 48 | O servidor liga duas caixas à mesma pessoa? | R — prova por recurso, não por pessoa |
| 49 | O servidor vê o IP do aparelho? | R — retransmissores independentes |
| 50 | O tamanho de uma mensagem revela o conteúdo? | R — preenchimento em classes (C81) |
| 51 | O horário das notificações revela quando há mensagem? | P — cadência fixa; no iPhone depende de permissão da Apple |
| 52 | O pagamento liga a pessoa à conta? | R — ficha cega |
| 53 | Apple e Google ficam sabendo do quê? | R — declarado: que o app está instalado, avisos e checagem de integridade; nunca remetente ou conteúdo |
| 54 | Um registro de acesso exigido por lei permite reidentificar alguém? | P — cifrado, mínimo, com IP do retransmissor |
| 55 | Dados de teste ou de piloto podem vazar identidade real? | A — política de dados de teste para o piloto |
| 56 | Telemetria ou relatório de falha envia algo? | R — proibido por regra; nenhum SDK de terceiros |
| 57 | O teclado do sistema aprende o que se digita? | R — aprendizado desligado nos campos |
| 58 | A IA do sistema operacional lê a tela? | R — proteção contra IA do sistema |

## 7. Aparelho e app

| # | Pergunta | Situação |
|---|---|---|
| 59 | Celular roubado e bloqueado entrega algo? | R — chaves no chip; dados cifrados |
| 60 | Root, jailbreak, emulador ou ganchos passam despercebidos? | P — atestação de hardware; teste em aparelho real (roteiro) |
| 61 | Print e gravação de tela funcionam? | P — bloqueados no Android; detectados e avisados no iPhone |
| 62 | O alternador de apps mostra conteúdo? | R — tela neutra |
| 63 | Conteúdo decifrado sobra na memória depois de fechar? | P — zerado no núcleo; confirmar na ponte (PT-16) |
| 64 | Backup do sistema copia algo legível? | P — conteúdo fora do backup; conferir configuração nos apps |
| 65 | O app recusa rodar se adulterado? | P — atestação; sentinela na abertura |

## 8. Servidor, banco e infraestrutura

| # | Pergunta | Situação |
|---|---|---|
| 66 | Quem leva o banco inteiro leva o quê? | R — só blocos cifrados com prazo |
| 67 | O papel do app no banco consegue desligar a proteção? | R — não (C106, sentinela do banco) |
| 68 | O servidor sobe com configuração insegura? | R — sentinela do núcleo e do banco travam a partida |
| 69 | Segredos do servidor podem ser lidos por quem administra a máquina? | P — cofre sem exportação; depende da configuração da nuvem (T12) |
| 70 | Existe rota de diagnóstico, métrica ou documentação exposta? | R — não existe no inventário |
| 71 | O servidor busca endereço vindo de fora (SSRF)? | R — nenhuma rota aceita endereço |
| 72 | A cópia de segurança é cifrada e a restauração testada? | A — configurar na infraestrutura |
| 73 | O ambiente de teste tem dados reais? | A — proibir por política e checagem |
| 74 | O que acontece se o servidor ficar fora do ar? | P — apps mostram "conectando"; nada se perde; plano de continuidade nos runbooks |

## 9. Cadeia de suprimentos e construção

| # | Pergunta | Situação |
|---|---|---|
| 75 | Uma dependência pode ser trocada por uma maliciosa? | P — versões travadas e auditadas; falta `cargo-vet` |
| 76 | Uma ação do CI pode ser trocada por versão maliciosa? | R — fixadas pelo hash do commit |
| 77 | Dá para provar de onde saiu um artefato? | P — certificado de procedência no CI (repositório privado exige GitHub Enterprise Cloud) |
| 78 | O app da loja é igual ao código? | P — reproduzível no Android; no iPhone não é possível provar |
| 79 | Um segredo pode entrar num commit? | R — gitleaks antes do commit e no CI |
| 80 | Um agente de IA pode ser enganado por texto escondido? | R — regra IA8; revisão humana |

## 10. Pessoas, processo e manutenção

| # | Pergunta | Situação |
|---|---|---|
| 81 | Uma pessoa sozinha consegue mudar o código principal ou publicar? | P — regra como código pronta; precisa ser aplicada uma vez por um administrador; vigia diário reprova se afrouxar |
| 82 | Quem faz manutenção consegue diminuir uma proteção sem ser notado? | R — catraca, CODEOWNERS, vigia da proteção |
| 83 | O que acontece se a chave de assinatura de versões vazar? | R — procedimento de rotação e revogação (CI de Segurança §5) |
| 84 | Um golpista consegue se passar pelo suporte? | R — não existe suporte dentro do app; nomes reservados |
| 85 | Uma ordem judicial falsa consegue dados? | R — só canal oficial conferido por advogado; não há conteúdo |
| 86 | Funcionário desligado continua com acesso? | P — procedimento escrito; depende de quem executa |
| 87 | Existe canal para pesquisador relatar falha? | P — `SECURITY.md`; publicar o canal no site |

## 11. Conformidade

| # | Pergunta | Situação |
|---|---|---|
| 88 | Cada requisito do OWASP ASVS 5.0.0 tem situação? | R — `MAPA_ASVS_5.0.csv` (345 requisitos, proposta para o auditor confirmar) |
| 89 | A Política de Privacidade descreve exatamente o que o código faz? | P — revisão de advogado |
| 90 | A avaliação de impacto (LGPD) está feita? | A — Lote 2 |
| 91 | O controle de exportação de criptografia está tratado? | A — Lote 2 |

## 12. Liberação e quem pode burlar

| # | Pergunta | Situação |
|---|---|---|
| 92 | Um artefato sem todos os portões consegue rodar? | R — selo recusa; sentinela não deixa funcionar (C110) |
| 93 | Uma pessoa sozinha consegue emitir um selo válido? | R — exige três chaves diferentes fixadas no código: CI, responsável de segurança e auditor de fora da empresa (`selo::MINIMO_DE_ASSINATURAS`, catraca) |
| 94 | Trocar o binário depois do selo passa? | R — artefato diferente |
| 95 | Voltar para uma versão antiga com falha conhecida passa? | R — versão mínima |
| 96 | O autor de uma mudança consegue contar como aprovador? | R — portão de governança ignora o autor (C111) |
| 97 | Alguém consegue retirar a aprovação e mesmo assim liberar? | R — vale a última revisão de cada pessoa |
| 98 | Um administrador do GitHub pode desligar a regra sem ninguém notar? | P — vigia e portão acusam e impedem o selo; a confiança final está em quem tem esse poder |
| 99 | Todas as pessoas com chave e aprovação combinadas conseguem liberar algo ruim? | A — sim, é o limite de qualquer processo; defesa: pessoas independentes, auditor externo como terceira chave, registro público |
| 100 | O que acontece se o GitHub falhar ou for invadido? | P — procedência assinada e registro público ajudam a perceber; plano de fornecedor alternativo em aberto |
| 101 | As sementes dos assinantes podem vazar? | P — geradas e guardadas offline; procedimento de troca e revogação a escrever no runbook |

## 13. Atualizações, administração e jurídico

| # | Pergunta | Situação |
|---|---|---|
| 102 | Uma dependência recém-publicada e maliciosa entra sozinha? | R — espera de 14 dias (30 para criptografia), sem fusão automática, revisão de duas pessoas |
| 103 | Uma vulnerabilidade publicada hoje é percebida hoje? | R — sentinela diária com a base do dia |
| 104 | A organização no GitHub pode ter um administrador só? | R — portão `administracao_dupla` reprova |
| 105 | Dá para provar que cada administrador usa chave física? | P — a API não informa; conferido por termo e revisão trimestral |
| 106 | Dois de dentro da empresa conseguem liberar algo sozinhos? | R — não: o selo exige três chaves, uma de fora |
| 107 | Os Termos afirmam algo sobre dados que o sistema não cumpre? | R — lista exata, igual à Política e ao código; mudança no tratamento exige atualizar os três |
| 108 | Alguma cláusula dos Termos seria nula para o consumidor? | P — redigidos para não ser; revisão de advogado obrigatória |
| 109 | O aceite dos Termos prova qual versão vale, sem identificar a pessoa? | P — aceite assinado pela chave da conta; advogado confirma como prova |
| 110 | O registro de acesso com o IP do retransmissor cumpre o art. 15 do Marco Civil? | A — tese a firmar com o advogado |

## 14. Metadados

| # | Pergunta | Situação |
|---|---|---|
| 111 | O servidor liga a caixa de hoje à de ontem da mesma relação? | R — caixa muda todo dia (MET1) |
| 112 | Quem leva o disco do servidor consegue conferir se um endereço conhecido recebeu algo? | R — não, depois de 48 h: chave do índice destruída (MET2) |
| 113 | Quem opera o servidor lê os registros de acesso? | R — não: selados para chave jurídica fora do servidor (MET3) |
| 114 | O tamanho do registro selado revela o IP? | R — tamanho fixo |
| 115 | Duas empresas de operação combinadas ligam IP à caixa? | P — sim, sem Tor; com Tor, não; por isso operadores precisam ser independentes |
| 116 | A memória do servidor pode ser lida por quem opera a máquina? | A — sim, sem computação confidencial; decisão aberta |
| 117 | O horário de envio casa com o de entrega? | P — cadência fixa e cobertura reduzem; rede de mistura a avaliar |
| 118 | A chave do índice cego vai parar no disco por acidente (troca de memória, despejo de falha)? | P — exigir memória travada e despejo de falha desligado no servidor (T12) |

## 15. Fechamento

| # | Pergunta | Situação |
|---|---|---|
| 119 | A mensagem de erro revela se uma caixa existe ou por que a prova falhou? | R — resposta única para toda falha de autorização (C114) |
| 120 | Um site qualquer consegue chamar a API pelo navegador da vítima? | R — sem CORS; origem só a oficial e só na rota do Envio (C114) |
| 121 | A API pode ser exibida dentro de moldura de outro site? | R — X-Frame-Options e frame-ancestors na API; não existe página web do Nimbus (decisão de 06/10/2026) |
| 122 | Existe rota usada pelo produto que falta no inventário? | R — a do Envio faltava e entrou; a catraca exige análise de toda rota sem titular |
| 123 | Uma chave de assinatura do servidor roubada vale por quanto tempo? | R — no máximo um dia (C116) |
| 124 | Sobras físicas de disco do servidor podem ser lidas depois? | R — não: chave do dia destruída (C117) |
| 125 | O app avisa o dono sobre riscos do próprio aparelho sem coletar nada? | R — diagnóstico local (C115) |
| 126 | Um agente de IA atacando o ambiente de teste acha algo que os testes não acharam? | A — rodar Strix ou equivalente no ambiente de teste |

## 16. Ataque vermelho

| # | Pergunta | Situação |
|---|---|---|
| 127 | Um nome com letras de imitação ou alfabetos misturados passa pela proteção de nomes? | R — não, depois da correção (A1) |
| 128 | A catraca pode ser enganada transformando uma linha em comentário? | R — não, depois da correção (A2) |
| 129 | Valores extremos derrubam algum módulo? | R — o único ponto achado foi corrigido (A4) |
| 130 | Alguma instrução da documentação leva a SQL inseguro? | R — a única achada foi corrigida (A6) |
| 131 | Quem administra a máquina de produção consegue rodar outro programa? | A — sim, hoje (A7); fecha com duas pessoas, imagem assinada conferida por fora e computação confidencial |
| 132 | A catraca pode ser enganada para liberar `unsafe` (allow no módulo inteiro, lista de lints, cfg_attr, expect, warn, teste de integração)? | R — não, depois da correção de 03/10/2026; `testar_catraca.py` planta as seis formas e mais sete afrouxamentos |
| 133 | Uma versão antiga e vulnerável de uma dependência pode passar escondida atrás de outra versão da mesma biblioteca? | R — não, depois da correção: o auditor confere cada versão do Cargo.lock (`testar_auditoria.py`) |
| 134 | Um conferidor pode dizer "Tudo certo" sem ter rodado alguma etapa? | R — não, depois da correção: falha fechada (`testar_verificar.py`); o modo rápido termina em PARCIAL |
| 135 | Um arquivo do projeto (protótipo, documento) consegue executar código nas ferramentas de verificação? | R — não, depois da correção: leitura com `ast.literal_eval` e regra `nimbus-sem-eval-nas-ferramentas`; `testar_conferidores.py` planta o ataque |
| 136 | Uma regra de proteção completa, mas apontando para outra branch ou para tags, passa pelo vigia? | R — não, depois da correção: alvo e branches conferidos; `portoes_de_governanca.py --teste` |
| 137 | A aprovação de um robô ou aplicativo conta como uma das duas pessoas? | R — não, depois da correção: só contas do tipo pessoa contam |
| 138 | Uma palavra digitada no Nimbus aparece como sugestão do teclado em outro app? | P — depende da camada nativa (RSD1); teste de aceite em aparelho real |
| 139 | Com outro app gravando o microfone, o Nimbus libera o microfone numa chamada? | R — não no núcleo (`vigia_de_sensores::decidir`, VSE6); P — no aparelho, depende do escudo nativo (T2): Android detecta pelo silenciamento; iPhone só pela interrupção do áudio |
| 140 | Uma leitura de sensores velha ou de relógio adiantado mantém o conteúdo aberto? | R — não: vence em 5 s e fecha tudo (VSE1) |
| 141 | Um endereço como privnotes.com ou um domínio oficial escondido dentro de outro passa sem aviso? | R — não: `autenticidade::avaliar_mensagem` acusa (AUT3); defeitos plantados e fuzzing |
| 142 | Uma cópia do app assinada por outro certificado abre as chaves? | R — não no núcleo (AUT1); P — a camada nativa precisa passar o resumo do certificado real (T2) |
| 143 | Um caractere invisível ou que inverte a leitura (ataque "Trojan Source") pode entrar no código ou nos documentos sem ser visto? | R — não: a catraca proíbe em todo arquivo de texto do núcleo e dos kits; a varredura achou um no próprio contrato da ponte, já corrigido |
| 144 | Um fluxo do CI pode rodar código de um pedido de fora com acesso aos segredos? | R — não: `pull_request_target`, `workflow_run` e texto de PR dentro de comando são proibidos pela catraca; checkout sem credencial guardada |
| 145 | Uma entrega pode ser declarada com o pacote diferente do que foi provado? | R — não: `entregar.py` confere a identidade do código de cada kit extraído contra a das provas e só declara no fim |
| 146 | Uma entrega pode ser declarada fora do CI ou numa máquina fraca? | R — não: o portão recusa fora do GitHub Actions, com menos de 4 núcleos, 8 GB de memória ou 20 GB livres, ou com árvore git suja |
| 147 | Um registro de prova pode ser reaproveitado de outro código ou com o código mudado no meio? | R — não: a identidade fica dentro do registro, no começo e no fim; o medidor recusa |
| 148 | Uma dependência pode ser desviada para outro endereço ou rodar código na compilação? | R — não: `build.rs`, `[patch]`, dependência por git, `.cargo/config` e pacote sem checksum são proibidos pela catraca |
| 149 | No iPhone, outro app consegue gravar o microfone junto com uma chamada do Nimbus? | P — o Nimbus toma o microfone com sessão exclusiva e o sistema interrompe o outro; confirmar em aparelho real (T2) |

## Verificar antes de usar
1. As situações "R" valem para o núcleo e para os documentos desta versão; cada uma precisa ser repetida no app e no servidor reais.
2. As perguntas 7, 8, 44, 60, 61, 63 e 64 só se respondem com o sistema rodando e em aparelhos físicos.
