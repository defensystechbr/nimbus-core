# 04 · UI/UX Design — NimbusChat e NimbusOne

**Versão 37 · Referência visual: protótipo interativo e guias visuais em PDF (66 telas)**

## 1. Princípios

1. **A segurança aparece onde há risco, e só ali.** Confirmações e esperas existem nos momentos em que um golpe aconteceria (convite, vínculo, recuperação, aprovação); o resto flui.
2. **Cada tela diz a verdade sobre si mesma** ("verdade desta tela" em `TELAS.md`), em linguagem simples.
3. **Nenhuma decisão de segurança por botão de "OK" cego:** a confirmação exige uma ação que só quem conferiu consegue fazer.
4. **Nada de padrões enganosos:** sem contagem regressiva falsa, sem cores de alerta para vender plano, sem caixa pré-marcada.
5. **Erro sempre com saída:** toda mensagem de erro diz o que aconteceu e o que fazer.

## 2. Identidade visual

Tema escuro por padrão (menos brilho em ambiente público), com tema claro disponível. Cores semânticas fixas: verde (aparelho seguro), âmbar (atenção), vermelho (bloqueio), ciano (informação de privacidade). As cores nunca são o único sinal: sempre há texto e ícone (acessibilidade para daltonismo).

## 3. Componentes de segurança

| Componente | Comportamento |
|---|---|
| Selo de nível do aparelho | Verde, amarelo ou vermelho, com o motivo ao tocar |
| Faixa de mudança | Troca de aparelho ou de nome de um contato; não some sozinha antes de ser vista |
| Confirmação por palavra | Quatro opções; sem botão "são iguais" |
| Espera de segurança | Contagem real, motivo explicado e botão "cancelar" nos aparelhos antigos |
| Teclado próprio | Nos campos de PIN e frase; teclas embaralháveis |
| Tela neutra | No alternador de apps e na captura de tela |

## 4. Tela "Ambiente seguro" (nova)

Mostra o resultado do diagnóstico local (`ambiente::avaliar`): bloqueio de tela, patch de segurança, teclado de terceiros, opções de desenvolvedor, fontes desconhecidas e cópia do sistema. Cada item com "como resolver". Texto fixo no topo: **"Este diagnóstico é feito só no seu aparelho. Nada sai daqui."**

## 5. Escrita

Frases curtas; sem jargão ("cifrado" em vez de siglas); verbos no imperativo nas ações; números por extenso só até dez. Mensagens de golpe conhecidas citadas pelo nome na tela certa ("Nunca escaneie um código que alguém lhe enviou").

## 6. Acessibilidade

Fonte dinâmica do sistema; leitor de tela (VoiceOver e TalkBack) com rótulos em todos os controles; contraste mínimo de 4,5:1 no texto; alvos de toque de pelo menos 44 pt; nenhuma informação só por cor; animações respeitam "reduzir movimento".

## 7. Desempenho percebido

Abrir conversa sem tela de carregamento visível; envio com estado "enviando" imediato; listas virtualizadas; nenhuma animação bloqueia a interação.

## 8. Validação com pessoas

Antes da publicação e a cada mudança de fluxo sensível: 5 a 8 pessoas reais, tarefas guiadas (criar conta, adicionar contato, recuperar conta, reconhecer golpe), sem gravar rosto nem voz, anotações sem nome. Critério: ninguém confirma contato sem comparar as palavras; todos encontram como recuperar a conta.
