> **PARE: nenhuma tarefa começa antes da Tarefa T0** — proteção da branch ativa no GitHub e o código inteiro revisado por duas pessoas, com termo assinado (`docs/REVISAO_INTEGRAL_T0.md`). Commit de trabalho feito antes disso é descartado.

# NimbusChat — repositório para o time e o Claude Code

**Comece por `docs/COMECE_AQUI_TECNICOS.md`.**

0. Antes de qualquer tarefa: a Tarefa T0 (`docs/REVISAO_INTEGRAL_T0.md`) — proteção da branch ativa e todo o código atual revisado por duas pessoas.
1. Rode `./verificar.sh`: ele confere tudo o que já foi feito e reprova se qualquer etapa falhar ou deixar de rodar.
2. Abra esta pasta no Claude Code: ele lê `CLAUDE.md` sozinho.
3. Cole `prompts/00-comece-aqui.md`; depois `prompts/01-proxima-tarefa.md` a cada tarefa.
4. Revise cada tarefa antes de aprovar a próxima.

Conteúdo: núcleo de segurança pronto (`core/`), telas (`docs/TELAS.md`), tarefas (`docs/PLANO_DE_TAREFAS.md`), manual do produto (`docs/MANUAL_NimbusChat.md`), manual completo (`docs/referencia/`), pagamentos, documentos jurídicos (`juridico/`), protótipo e o verificador de consistência (`verificacao/consistencia.py`).
