# Checklist de entrada em produção — Nimbus

**Versão 37 · Nada vai para produção com uma caixa vazia.** Cada item diz quem confere e como. Os itens marcados com ⚙ são conferidos pela máquina a cada liberação (portões do selo) — se faltarem, o servidor e os apps não funcionam. Os itens marcados com ✍ dependem de pessoas e precisam de registro assinado pelo responsável de segurança.

## 0. Antes de qualquer trabalho (Tarefa T0)
- [ ] ⚙ Proteção da branch ativa e igual à do código, conferida pelos portões `protecao_do_repositorio` e `administracao_dupla`, ANTES do primeiro commit de trabalho novo
- [ ] ✍ Importação do código atual num único pedido, revisado integralmente por duas pessoas, módulo por módulo, com o termo de `REVISAO_INTEGRAL_T0.md` assinado
- [ ] ✍ Nenhuma tarefa T1 em diante aberta antes dos dois itens acima

## A. Organização e pessoas
- [ ] ✍ Responsável de segurança e substituto nomeados por escrito (equipe `@nimbus/seguranca`, 2+ pessoas)
- [ ] ⚙ Organização no GitHub exige segundo fator; 2+ administradores; 2+ na segurança (portão `administracao_dupla`)
- [ ] ✍ Chave física (FIDO2) de cada administrador e membro da segurança registrada em termo; revisão trimestral marcada
- [ ] ✍ Auditor independente de fora da empresa contratado, com a terceira chave do selo
- [ ] ✍ Custodiantes da chave jurídica (2 de 3) nomeados; procedimento de abertura só com ordem judicial; chave jurídica por mês, partes destruídas aos 7 meses
- [ ] ✍ Três guardiões das chaves-raiz do servidor (2 de 3), em organizações diferentes; cerimônia mensal dos certificados do dia registrada

## B. Repositório e liberação
- [ ] ⚙ Regra da branch aplicada (`aplicar_protecao.sh`) e igual à do código (portão `protecao_do_repositorio`; vigia diário)
- [ ] ⚙ Commits assinados (portão `commits_assinados`)
- [ ] ⚙ Duas aprovações finais de pessoas diferentes do autor (portão `revisao_de_duas_pessoas`)
- [ ] ✍ Ambiente `liberacao` com 2 revisores e bloqueio de autoaprovação; segredo `SELO_SEMENTE_CI` só nele
- [ ] ✍ Três sementes do selo geradas offline, por pessoas diferentes; chaves públicas fixadas no código
- [ ] ⚙ Token `PROTECAO_LEITURA` só de leitura criado
- [ ] ⚙ Renovate ativo, sem fusão automática, com espera (catraca confere)
- [ ] ⚙ Sentinela diária ativa (catraca confere)

## C. Código e testes (todos ⚙, no CI)
- [ ] Testes, mutação completa, fuzzing profundo, tempo constante e tempo de recusa (`./verificar.sh`, falha fechada: etapa ausente reprova)
- [ ] Conferidores conferidos: `testar_catraca.py`, `testar_auditoria.py`, `testar_verificar.py`, `testar_conferidores.py` e `entregar.py --testar` aprovados
- [ ] Versão entregue pelo `entregar.py` **no CI** (fluxo `ci/entrega.yml`, executor com 4+ núcleos, 8+ GB de memória e 20+ GB livres), com o `./verificar.sh` completo rodado inteiro dentro de cada kit e `ENTREGA_DECLARADA.txt` guardado junto do selo
- [ ] Escudo nativo com análise sintática aprovada (`conferir_sintaxe_nativa.py`) **e** compilado no Android SDK e no Xcode
- [ ] Lista `REQUISITOS_CITADOS_SEM_DEFINICAO.json` zerada (todos os requisitos citados pelo núcleo definidos no Manual)
- [ ] `cargo clippy --all-targets --locked -- -D warnings` ligado no CI e no `verificar.sh`, sem aviso (hoje não roda: tarefa da T1)
- [ ] Varredura (47 regras), segredos (gitleaks), dependências (todas as versões do Cargo.lock, base com até 7 dias), consistência dos documentos
- [ ] Catraca: nenhuma proteção a menos
- [ ] Banco: testes de RLS, sentinela do banco e defeitos plantados no esquema
- [ ] Certificado de procedência gerado

## D. Servidor e infraestrutura (✍, com evidência anexada)
- [ ] Servidor chama `sentinela::autoverificar_com_selo` na partida e a cada hora; banco roda `verificar_banco_na_partida.sql`
- [ ] Toda rota passa pelo porteiro e pelo limitador; inventário igual a `ACESSO_SEM_AUTENTICACAO.md`
- [ ] Banco guarda índice cego, nunca endereço; chaves de índice só em memória travada ou módulo de hardware
- [ ] Registros de acesso só selados para a chave jurídica pública do mês
- [ ] Respostas genéricas, sem CORS, cabeçalhos da API, origem oficial do Envio configurada
- [ ] Bloco no disco selado com a chave do dia; chave do servidor diferente a cada dia, com certificado
- [ ] Strix ou equivalente rodado no ambiente de teste, sem dado real, com achados corrigidos
- [ ] Segredos do servidor em módulo de hardware ou cofre, sem exportação; nada em arquivo ou variável de ambiente
- [ ] Memória de troca e despejo de falha desligados; disco e cópias cifrados, restauração testada
- [ ] Acesso de administrador à produção só com duas pessoas; entrada de emergência aprovada por outra pessoa e registrada fora do alcance do operador
- [ ] Máquinas só rodam imagens assinadas, conferidas pelo orquestrador antes de iniciar; nenhuma alteração em máquina em funcionamento
- [ ] Decisão sobre computação confidencial com atestação remota tomada e registrada (recomendada pelo ataque vermelho)
- [ ] Retransmissores e proxy de chamadas operados por empresas independentes, com contrato
- [ ] TLS 1.3; nenhum cabeçalho revela tecnologia ou versão; nenhuma rota de diagnóstico

## E. Apps (✍, com evidência anexada)
- [ ] As 49 regras do `CONTRATO_DA_PONTE.md`, cada uma com teste de ponte
- [ ] Testes em aparelho real (roteiro do núcleo) passando no Android e no iPhone
- [ ] Tela "Ambiente seguro" com o diagnóstico local; nenhum SDK de terceiros no app
- [ ] Apps enviados às lojas só com selo completo; servidor recusa versão de app sem selo

## F. Terceiros e jurídico
- [ ] ✍ Auditoria criptográfica independente concluída, achados corrigidos
- [ ] ✍ Pentest humano (PT-01 a PT-48) concluído, achados corrigidos
- [ ] Escudo nativo (`EscudoNativo.kt` e `EscudoNativo.swift`) compilado, ligado ao `vigia_de_sensores::decidir` pela ponte e aprovado nos testes VSE1–VSE8 em aparelho real, Android e iPhone
- [ ] Resíduos no aparelho RSD1 a RSD12 (`RESIDUOS_NO_APARELHO_E_LICOES.md`) com teste de aceite aprovado em aparelho real
- [ ] Migrações do banco com volta atrás testada; expiração de sessão testada; restauração da cópia de segurança do servidor testada
- [ ] ✍ Mapa do ASVS 5.0.0 confirmado pelo auditor
- [ ] ✍ Termos de Uso, Política de Privacidade, Tolerância Zero e notas jurídicas revisados por advogado
- [ ] ✍ Tese do art. 15 do Marco Civil (registro com IP do retransmissor) firmada pelo advogado
- [ ] ✍ Teste de usabilidade com 5 a 8 pessoas reais

## G. Assinatura de entrada em produção
- [ ] ✍ Responsável de segurança: todos os itens acima marcados, com evidência
- [ ] ✍ Auditor independente: confere a evidência e assina o primeiro selo de produção
- [ ] ✍ Fundador: autoriza a entrada em produção
