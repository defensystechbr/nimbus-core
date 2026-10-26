# NimbusChat — repositório para o time e o Claude Code

**Comece por `docs/COMECE_AQUI_TECNICOS.md`.**

**Estado em 26/10/2026: desenvolvimento liberado.** A proteção da branch principal está ativa no GitHub
(`.github/regras/protecao-principal.json`: pedido de mudança obrigatório, commit assinado, histórico linear, sem
força, sem apagamento, verificações obrigatórias, ninguém com passe livre) e a importação do código passou por ela.
O regime de aprovação é **aprovador único, o dono** — decisão registrada em `docs/DECISAO_APROVADOR_UNICO.md`, com
o que ela custa dito sem enfeite.

1. Rode `./verificar.sh`: ele confere tudo o que já foi feito e reprova se qualquer etapa falhar ou deixar de rodar.
2. Abra esta pasta no Claude Code: ele lê `CLAUDE.md` sozinho.
3. Cole `prompts/00-comece-aqui.md`; depois `prompts/01-proxima-tarefa.md` a cada tarefa.
4. Revise cada tarefa antes de aprovar a próxima. A revisão é do dono, no pedido de mudança; o merge é a aprovação.

O que continua valendo sem exceção: núcleo só muda com prova (`./verificar.sh` verde), nenhuma proteção a menos
(catraca), commit assinado, e nenhuma liberação sem os portões — liberar é ato manual do dono, no ambiente
`liberacao`.

Conteúdo: núcleo de segurança pronto (`core/`), telas (`docs/TELAS.md`), tarefas (`docs/PLANO_DE_TAREFAS.md`), manual do produto (`docs/MANUAL_NimbusChat.md`), manual completo (`docs/referencia/`), pagamentos, documentos jurídicos (`juridico/`), protótipo e o verificador de consistência (`verificacao/consistencia.py`).
