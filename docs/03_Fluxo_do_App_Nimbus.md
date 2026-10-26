# 03 · Fluxo do App — NimbusChat e NimbusOne

**Versão 37 · As 66 telas estão no protótipo e em `TELAS.md` (cada uma com o que implementar e a "verdade desta tela"). Aqui ficam os caminhos entre elas e as verificações de cada passo.**

## 1. Primeira abertura

1. Abertura → sentinela (núcleo) + integridade + diagnóstico do ambiente (`ambiente::avaliar`). Vermelho: tela "Ambiente inseguro" com o motivo e como resolver; nada mais abre.
2. Boas-vindas → explica em três frases: sem telefone, sem cópia na nuvem, ninguém lê nada.
3. Aceite dos Termos → assinado pela chave recém-criada no chip.
4. Criação da conta → par de chaves no chip; registro no registro público.
5. PIN → Argon2id; escolha da política de erros (Equilibrado ou Tolerante); PIN de coação opcional.
6. Recuperação → frase (conferida por 3 palavras sorteadas), chave física e/ou amigos de confiança.
7. Pronto → lista de conversas vazia, com "Adicionar contato" em destaque.

## 2. Adicionar contato

1. "Adicionar contato" → QR presencial (preferido) ou código de convite de uso único que vale 10 minutos.
2. Confirmação → a outra pessoa lê as 6 palavras da tela dela; você toca na palavra pedida entre 4 opções. Sem botão "são iguais".
3. Nome → você salva o rótulo; nomes reservados recusados; nome parecido com outro contato ganha aviso.

## 3. Conversar

1. Lista → conversa → mensagem cifrada no aparelho (libsignal), enviada à caixa do dia da relação pelo retransmissor.
2. Prazo → escolhido no envio (1 min a 48 h); visualização única opcional.
3. Recebimento → entrega por lote com endereços falsos; notificação genérica ("Nova mensagem") em cadência fixa.
4. Troca de aparelho do contato → faixa que não se silencia.

## 4. Chamada

Iniciar → proxy de chamadas (o IP de um não chega ao outro) → microfone só por toque local → encerrar apaga as chaves da chamada.

## 5. Envio direto e NimbusDrop

- Envio direto: escolher contato → arquivo inspecionado e limpo → assinado e selado para a chave de recebimento do dia → entrega única em até 48 h → quem recebe confere o remetente; se não for contato, "remetente não confirmado" e não abre.
- NimbusDrop: arquivo chega por proximidade **lacrado**; a chave chega pelo servidor; só abre em aparelho verde com hora confirmada.

## 6. Nimbus Envio (retirado)

Retirado em 06/10/2026 por decisão do dono: não existe link nem página web para quem não tem o app. Quem recebe instala o Nimbus; o envio acontece só dentro do app (`MUDANCA_ENVIO_SO_DENTRO_DO_APP.md`).

## 7. Vincular computador

Só pela tela "Vincular aparelho" do celular → QR mostrado no computador → biometria → palavras iguais nos dois → computador entra no nível amarelo; vínculo cai em 7 dias sem confirmação.

## 8. Recuperar conta

- Frase ou chave física → volta na hora; aparelho antigo é revogado.
- Amigos de confiança → cada amigo confere ao vivo as palavras do aparelho novo e faz uma pergunta pessoal → 3 dias de espera com aviso em todos os aparelhos antigos, que cancelam com um toque.

## 9. NimbusOne

- Cofre → itens cifrados no aparelho; PIN de coação abre o cofre-isca.
- Aprovações → pedido assinado; beneficiário novo espera 24 h; nada vale por voz, e-mail ou telefone.
- Console corporativo → lista de contas assinada; suspensão imediata; recuperação de dados com 2 de 3 administradores, 2 de 3 guardiões e 24 h visíveis a todos.

## 10. Estados de erro (todos com saída clara)

| Situação | O que o app mostra |
|---|---|
| Sem internet | "Conectando…"; nada com prazo aparece depois de 15 min sem hora confirmada |
| Servidor travado pela sentinela | "Manutenção de segurança em andamento. Nada foi perdido." |
| Selo inválido | O app não abre e orienta a atualizar pela loja |
| Limite de uso | "Muitas tentativas. Tente em alguns minutos." |
| Contato com chave trocada | Faixa permanente até reconferir as palavras |
