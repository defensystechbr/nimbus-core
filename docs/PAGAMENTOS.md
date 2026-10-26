# Pagamentos, planos e compra

## Planos (preço FECHADO na Rodada 25, 12/10/2026 — a tabela "valores sugeridos" em USD que existia aqui antes era de uma estrutura de produto já superada; corrigido na Rodada 26, 12/10/2026, ao auditar este arquivo contra a fonte atual)

A fonte da verdade dos preços é o código, não este arquivo: `pagamento_web/config_planos.mjs` (espelhado em `servidor-cobranca/src/main.rs`). Resumo, para não duplicar números que podem mudar:

| Plano | Cadência | Preço | Observação |
|---|---|---|---|
| Grátis | — | R$ 0 | Para sempre; nunca expira nem força upgrade; reversível a qualquer momento; 2 GB, retenção de 3 meses |
| Pro | Mensal (sem compromisso) | R$ 24,90/mês | Pode cancelar quando quiser |
| Pro | Anual (promocional) | R$ 19,90/mês (cobrança única anual de R$ 238,80) | ~20% de desconto sobre o mensal |
| Max | Mensal (sem compromisso) | R$ 59,90/mês | Pode cancelar quando quiser |
| Max | Anual (promocional) | R$ 47,90/mês (cobrança única anual de R$ 574,80) | ~20% de desconto sobre o mensal |

Raciocínio completo, fontes primárias (iCloud+, Google One, Proton Unlimited) e os dois bugs reais achados e corrigidos ao construir a cobrança por cadência: `RESPOSTA_RODADA25_PRECO_E_AUDITORIA.md` §1-2. Como o código funciona e o que ainda depende de uma conta Stripe real: `pagamento_web/LEIA-ME.md`. Esta tabela antiga (6 planos separados por produto, em dólar, "a validar com o piloto") não corresponde a nenhuma versão do código já construído — não há evidência de que tenha sido implementada assim em algum momento; tratada como suposição inicial superada, não como uma decisão revertida.

## Como a pessoa paga — e o que cada um fica sabendo

| Caminho | Quem vê o pagamento | O que a NimbusOne sabe |
|---|---|---|
| Assinatura na loja do celular (App Store, Google Play) | A loja sabe que aquela conta de loja comprou | O recibo é trocado por uma **ficha cega**: o servidor confere o recibo e emite a ficha sem conseguir ligá-la à conta Nimbus que vai usá-la |
| Site da NimbusOne, com ficha anônima | O processador de pagamento | Que houve uma compra; nunca qual conta usa a ficha |
| Fatura corporativa (Empresas, Governo) | O financeiro das duas empresas | Quantas licenças a empresa tem; nunca o conteúdo |

A ficha cega segue o padrão Privacy Pass: quem emite não consegue reconhecer a ficha quando ela é usada.

## Regras das lojas (conferidas em setembro de 2026)

- **iPhone, loja dos EUA:** desde maio de 2025 (regra 3.1.1(a)), o app pode ter botão ou link para comprar no site,
  sem autorização especial. A comissão sobre essas compras está em disputa judicial (Epic × Apple).
- **iPhone, demais países:** funções digitais pagas precisam da compra dentro do app, salvo autorização regional
  específica da Apple. Usar a assinatura da App Store e trocar o recibo por ficha cega.
- **Resgatar no app uma ficha comprada no site** pode ser tratado pela Apple como "chave de licença" (vedada pela
  regra 3.1.1) fora dos EUA — risco de revisão; validar antes de publicar.
- **Android:** o Google Play exige o faturamento dele para bens digitais, com programas de faturamento alternativo
  em alguns países — validar a regra vigente em cada país de venda.

## Regras de produto

- Preço por país (paridade de poder de compra) e moeda local — a definir com o financeiro.
- Descritor neutro na fatura do cartão e na fatura corporativa.
- Reembolso sem ligação com a conta: devolve-se o valor ao meio de pagamento e a ficha é invalidada.
- Impostos por país: validar com contador antes de vender.
