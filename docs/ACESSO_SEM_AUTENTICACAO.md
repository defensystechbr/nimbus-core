# O que um usuário não autenticado consegue acessar

*Gerado do código por `gerar_inventario_publico.py`. Fonte: `autenticacao::ROTAS` e `ROTAS_PUBLICAS_ANALISADAS.json`.*

O servidor tem **10 rotas**, e nada além delas existe (negar por padrão). Toda rota tem limite de uso.

| Rota | Exige | Limite por minuto |
|---|---|---|
| POST /v1/hora | Sem prova nenhuma | 120 |
| GET /v1/prechave | Sem prova nenhuma | 120 |
| POST /v1/convite/resgatar | Sem prova nenhuma | 10 |
| POST /v1/caixa/criar | Só ficha anônima de licença (não identifica ninguém) | 10 |
| POST /v1/caixa/depositar | Só ficha anônima de licença (não identifica ninguém) | 120 |
| POST /v1/caixa/retirar | Chave da caixa | 120 |
| POST /v1/caixa/apagar | Chave da caixa | 120 |
| POST /v1/prechave/publicar | Chave do aparelho | 10 |
| POST /v1/aparelho/registrar | Chave do aparelho | 5 |
| POST /v1/aparelho/revogar | Chave do aparelho | 10 |

## Rotas acessíveis sem identificar quem chama

### POST /v1/hora — Sem prova nenhuma

- **Devolve:** Hora atual do servidor assinada junto com o desafio de 32 bytes enviado pelo próprio pedido
- **Expõe:** A hora do servidor (pública por natureza)
- **Riscos considerados:** Repetição de resposta antiga; servidor falso
- **Defesa:** Resposta presa ao desafio de uso único e assinada; aparelho recusa hora que volta no tempo (expiracao::conferir_hora); 120 por minuto por origem
- **Limite:** 120 por minuto por origem; castigo que dobra a cada falha

### GET /v1/prechave — Sem prova nenhuma

- **Devolve:** Chave de recebimento pública de uma caixa, assinada pela identidade
- **Expõe:** Que aquele endereço de 32 bytes existe e a impressão digital da identidade de quem o publicou — só para quem já conhece o endereço (contatos)
- **Riscos considerados:** Enumeração de endereços; troca de chave pelo servidor
- **Defesa:** Endereço aleatório de 256 bits (adivinhar é inviável); assinatura da identidade conferida por quem envia (prechaves::conferir); 120 por minuto por origem
- **Limite:** 120 por minuto por origem; castigo que dobra a cada falha

### POST /v1/convite/resgatar — Sem prova nenhuma

- **Devolve:** Se o código de convite de uso único existe, o endereço exclusivo da relação; depois disso o código queima
- **Expõe:** Nada sem o código; com o código, só o endereço daquela relação
- **Riscos considerados:** Adivinhar códigos; interceptar o código antes do dono
- **Defesa:** Código de 256 bits; servidor guarda só o resumo; uso único; prazo de 10 minutos; palavras de confirmação obrigatórias (EO1); 10 por minuto por origem e castigo que dobra
- **Limite:** 10 por minuto por origem; castigo que dobra a cada falha

### POST /v1/caixa/criar — Só ficha anônima de licença (não identifica ninguém)

- **Devolve:** Confirmação de criação de uma caixa com a chave pública informada
- **Expõe:** Nada
- **Riscos considerados:** Criação em massa (consumo de recurso)
- **Defesa:** Exige ficha anônima de licença; 10 por minuto por origem; RLS do banco só aceita com ficha conferida
- **Limite:** 10 por minuto por origem; castigo que dobra a cada falha

### POST /v1/caixa/depositar — Só ficha anônima de licença (não identifica ninguém)

- **Devolve:** Confirmação de depósito de bloco cifrado numa caixa existente
- **Expõe:** Nada (não diz se a caixa tem outros blocos)
- **Riscos considerados:** Encher caixas alheias (spam cifrado); consumo de recurso
- **Defesa:** Exige ficha anônima de licença; tamanho máximo; prazo de 48 h no banco; 120 por minuto por origem; o destinatário só abre o que vier assinado por contato (DIR8)
- **Limite:** 120 por minuto por origem; castigo que dobra a cada falha

## Fora da API

- **Página do Nimbus Envio:** estática; só abre um arquivo com o link completo (identificador de 256 bits + chave depois do `#`, que nunca chega ao servidor); o link queima depois de aberto e vence em até 48 h.
- **Retransmissores (OHTTP) e proxy de chamadas:** encaminham bytes cifrados; não guardam nada.
- **Registro público de chaves e de versões:** público por desenho (transparência); só chaves públicas e resumos.
- **Lojas e site:** informação pública do produto.
- **Sem rota de diagnóstico, administração, documentação interativa ou métrica exposta.**
