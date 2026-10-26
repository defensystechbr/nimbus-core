# Notas jurídicas sobre os Termos de Uso — para o advogado (instruções, não fazem parte dos Termos)

**Versão de 2 de outubro de 2026.** Este arquivo explica as escolhas dos Termos de Uso, o que a lei não permite afastar e o que o advogado precisa conferir e decidir antes da publicação. **Os Termos não devem ser publicados sem revisão de advogado.**

## 1. A verdade sobre "isentar ao máximo"

1. **Relação de consumo:** cláusula que impossibilite, exonere ou atenue a responsabilidade do fornecedor por vícios é **nula de pleno direito** (art. 51, I, da Lei nº 8.078/1990). Escrever uma isenção total para pessoa física não protege — o juiz anula a cláusula e ela ainda pode ser lida como abusiva. Por isso os Termos **não** isentam genericamente: eles (a) descrevem com clareza a natureza do Serviço (a impossibilidade de recuperar Conteúdo é o produto informado, não um vício) e (b) listam as hipóteses em que a própria lei afasta a responsabilidade — culpa exclusiva do consumidor ou de terceiro, caso fortuito e força maior.
2. **Consumidor pessoa jurídica:** o mesmo art. 51, I, permite **limitar** a indenização "em situações justificáveis". A cláusula 10.4 usa essa porta e justifica a limitação pela natureza do Serviço.
3. **Contratação empresarial que não é consumo** (empresa que contrata como insumo): vale a liberdade contratual, com presunção de paridade e simetria (art. 421-A do Código Civil). O Anexo I limita danos indiretos, lucros cessantes e valor total, sem alcançar dolo.
4. **LGPD:** o regime de responsabilidade dos arts. 42 a 45 da Lei nº 13.709/2018 não pode ser afastado por contrato. A melhor defesa é a do art. 43: provar que não houve tratamento do dado alegado ou que não houve violação da lei. **A arquitetura do Nimbus é a prova**: o Operador não tem o Conteúdo nem as chaves.

## 2. Por que os Termos não dizem "não coletamos nenhuma informação"

Seria falso — e uma afirmação falsa em termos e propaganda gera responsabilidade em vez de afastá-la. O Operador **trata**: chaves públicas, blocos cifrados com prazo, registros de acesso do art. 15 da Lei nº 12.965/2014 (data, hora e IP do retransmissor, por 6 meses — obrigação legal de provedor de aplicações constituído como pessoa jurídica com fins econômicos), ficha anônima de licença, confirmação de pagamento e aceite assinado dos Termos. Identificadores técnicos podem ser dado pessoal mesmo sem nome. A frase correta, e defensável, é: **"não temos acesso ao conteúdo e tratamos só o mínimo, listado item por item"**. Os Termos, a Política de Privacidade e o código dizem a mesma coisa; qualquer mudança no que o sistema trata exige atualizar os três juntos.

## 3. Fundamento de cada cláusula sensível

| Cláusula | Fundamento | Observação |
|---|---|---|
| 2.1 — 18 anos ou mais | Evita o regime especial de dados de crianças e adolescentes (art. 14 da LGPD) | Não há verificação de idade sem coletar dado; a declaração é do usuário |
| 3 — natureza do Serviço | Dever de informação clara e prévia (Lei nº 8.078/1990) | É a cláusula que mais protege: transforma "não recuperamos" em característica informada |
| 4 — dados tratados | Princípios de transparência e necessidade da LGPD; art. 15 do Marco Civil | Tem de bater com a Política e com o código |
| 7 — autoridades | Arts. 15 e 22 do Marco Civil (registros só com ordem judicial) | Entrega só o que existe |
| 8.3 — arrependimento | Art. 49 da Lei nº 8.078/1990 | Compras pela loja seguem também as regras da loja |
| 10.1 — excludentes | Culpa exclusiva do consumidor ou de terceiro (art. 14, § 3º, da Lei nº 8.078/1990); caso fortuito e força maior (art. 393 do Código Civil) | Descreve a lei; não cria isenção |
| 10.4 — consumidor PJ | Art. 51, I, parte final, da Lei nº 8.078/1990 | Limite: valor pago em 12 meses |
| 17.2 — foro do consumidor | Facilitação da defesa do consumidor (art. 101, I, da Lei nº 8.078/1990) | Eleger outro foro para pessoa física seria nulo |
| Anexo I.1 — operador | Art. 39 da LGPD | O cliente é o controlador dos dados de seus integrantes |
| Anexo I.6 — limitação B2B | Art. 421-A do Código Civil (incluído pela Lei nº 13.874/2019) | Não alcança dolo |

## 4. O que o advogado precisa decidir

1. A pessoa jurídica que será o Operador, sua sede (que define o foro do item 17.3) e o encarregado de dados.
2. Se o registro de acesso com o IP do retransmissor cumpre o art. 15 do Marco Civil ou se é preciso outra solução (tese a firmar; consta das pendências do projeto).
3. Controle de exportação de criptografia nos países onde o app for distribuído.
4. Redação final da Política de Tolerância Zero e sua harmonia com estes Termos.
5. O mecanismo de aceite: o app registra a versão aceita assinada pela chave da conta — confirmar que serve como prova de aceite.

## 5. Verificar antes de usar (texto oficial, no Planalto)

1. Lei nº 8.078/1990: arts. 14, § 3º; 49; 51, I; e 101, I — conferir a redação vigente e alterações posteriores (a Lei nº 14.181/2021 acrescentou incisos ao art. 51).
2. Lei nº 13.709/2018: arts. 14, 39, 42 e 43 — conferir alterações (a Lei nº 13.853/2019 alterou a LGPD).
3. Lei nº 12.965/2014, art. 15; e Código Civil, arts. 393 e 421-A.
