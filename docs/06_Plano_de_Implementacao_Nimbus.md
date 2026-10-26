# 06 · Plano de Implementação — Nimbus

**Versão 37 · Tarefas detalhadas em `PLANO_DE_TAREFAS.md` de cada kit · Critério de pronto: `CHECKLIST_DE_ENTRADA_EM_PRODUCAO.md`**

## Fase 0 — Fundação (antes de qualquer código de app)

1. Organização no GitHub, equipes, chave física, proteção da branch, token do vigia, ambiente `liberacao` (`GUIA_DE_LIBERACAO.md` §2).
2. Três chaves do selo e três chaves-raiz do servidor geradas offline, por pessoas diferentes; auditor independente contratado.
3. Decisões abertas fechadas: operadores independentes do retransmissor e do proxy, computação confidencial, distribuição (lojas e/ou web), versões mínimas de sistema.
4. Advogado: Termos, Política, Tolerância Zero, tese do art. 15 do Marco Civil, controle de exportação.

5. **Tarefa T0:** com a proteção ativa, importar o código atual por um único pedido revisado integralmente por duas pessoas, módulo por módulo (`REVISAO_INTEGRAL_T0.md`). Nada da Fase 1 começa antes.

**Saída:** a importação revisada passa por todos os 17 portões e gera um selo com três assinaturas.

## Fase 1 — Ponte e esqueleto (T1–T3)

Ponte UniFFI expondo as funções do núcleo; esqueleto dos apps com sentinela, integridade e diagnóstico de ambiente na abertura; servidor com porteiro, limitador, sentinela, banco e rotação diária de chaves. **Saída:** app abre, cria conta, faz a primeira troca de mensagem em ambiente de teste.

## Fase 2 — Comunicação (T4–T7)

Protocolo de mensagens (libsignal, MLS), contatos com confirmação por palavra, conversas, prazos, visualização única, notificações em cadência fixa, chamadas. **Saída:** as 49 regras da ponte com teste; testes em aparelho real.

## Fase 3 — Arquivos e NimbusOne (T8–T11B)

Envio direto assinado, NimbusDrop lacrado, cofre, aprovações, recuperação (frase, chave física, amigos), console corporativo, app de computador. **Saída:** todas as telas do protótipo funcionando, com a "verdade da tela" conferida.

## Fase 4 — Endurecimento e terceiros (T12–T13)

Infraestrutura (TLS, cofre, memória travada, cópias cifradas), auditoria criptográfica, pentest humano (PT-01 a PT-48), teste automatizado com agentes de IA em ambiente de teste, teste de usabilidade, confirmação do mapa do ASVS, publicação nas lojas. **Saída:** checklist de entrada em produção inteiro marcado.

## Fase 5 — Piloto controlado

Grupo pequeno de clientes, com programa de recompensa aberto, sentinela diária, revisão semanal das contagens anônimas e calibração dos limites de uso.

## Regras de execução em todas as fases

- Toda mudança: teste que falha antes e passa depois, defeito plantado, duas aprovações, todos os portões.
- Nada de "depois a gente arruma": item de segurança aberto bloqueia a fase.
- Qualquer reprovação da sentinela diária para tudo até a correção.
- O registro permanente ganha uma linha a cada pedido de segurança do fundador.

## Quem faz o quê

| Papel | Responsabilidade |
|---|---|
| Responsável de segurança (2 pessoas) | Portões, selo (segunda chave), revisão de código de segurança, resposta a incidentes |
| Engenharia do núcleo | Núcleo, ponte, testes, defeitos plantados |
| Engenharia de apps | iOS, Android, computador; regras da ponte |
| Engenharia do servidor | Servidor, banco, infraestrutura |
| Auditor independente (de fora) | Terceira chave do selo; auditoria |
| Advogado | Documentos jurídicos, pedidos de autoridade |
| Fundador | Decisões de produto e autorização de produção |
