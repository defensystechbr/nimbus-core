# 01 · PRD — Documento de Requisitos de Produto — NimbusChat e NimbusOne

**Versão 37 · Confidencial · Fonte de verdade detalhada: Manual Técnico v37 (598 requisitos) e SRS**

## 1. Visão

Comunicação e guarda de informação em que **ninguém além das pessoas envolvidas consegue ler nada — nem a própria Nimbus**. Quando uma autoridade pede, a Nimbus cumpre a lei e entrega o que existe: quase nada, porque o produto foi desenhado para não guardar.

## 2. Problema

Mensageiros e nuvens populares protegem o conteúdo, mas acumulam metadados (quem fala com quem, quando, de onde), exigem número de telefone, guardam cópias, dependem de senhas e podem ser tomados por golpe de SIM, QR ou falso suporte. Empresas e pessoas expostas (advogados, executivos, jornalistas, médicos, figuras públicas) precisam de sigilo real, sem depender da boa vontade de quem opera o servidor.

## 3. Princípios inegociáveis

1. **Acesso zero:** o servidor nunca tem chave de conteúdo.
2. **Guardar o mínimo pelo menor tempo:** nada no servidor passa de 48 h; chaves que permitiriam ler o passado são destruídas.
3. **Nenhuma pessoa sozinha** consegue liberar código, ler registros, recuperar dados ou afrouxar regras.
4. **Falha fechada:** se algo estiver errado, o sistema para e diz como corrigir.
5. **Verdade:** nenhuma promessa que o código não prove; limites escritos.

## 4. Públicos

| Público | Necessidade | Plano |
|---|---|---|
| Pessoa física exposta | Conversar e guardar sem rastro | NimbusChat + NimbusOne pessoal |
| Profissional com sigilo (advogado, médico) | Envio seguro a clientes, cofre, aprovação de documentos | NimbusOne Profissional |
| Empresa | Equipes, administração sem acesso a conteúdo, recuperação controlada | NimbusOne Corporativo |

## 5. Produtos e funções

**NimbusChat (36 telas):** conversas e grupos cifrados (libsignal e MLS); chamadas de voz e vídeo por retransmissor; mensagens que somem (prazos de 1 minuto a 48 horas); visualização única; envio direto assinado; NimbusDrop lacrado; convites por código de uso único com confirmação de palavras; personas; Escudo; sem número de telefone.

**NimbusOne (30 telas):** cofre pessoal; celular virtual; Sala Blindada; aprovações de pagamentos e documentos; console corporativo com recuperação 2 de 3 + 2 de 3 + 24 h; amigos de confiança; PIN de coação com cofre-isca.

**Computador:** aparelho vinculado, nunca acima do nível amarelo; cofre, visualização única e NimbusDrop ficam só no celular.

## 6. Requisitos não funcionais

| Tema | Requisito |
|---|---|
| Criptografia | Troca de chaves e assinatura híbridas pós-quânticas; nenhum algoritmo próprio |
| Privacidade | Sem telefone, e-mail ou nome; IP do aparelho nunca chega ao servidor; caixas que mudam todo dia |
| Disponibilidade | Servidor sem estado de conteúdo; falha fechada com mensagem clara ao usuário |
| Desempenho | Ver TRD §8 (metas medidas em aparelho real antes do lançamento) |
| Usabilidade | Cada tela com "verdade desta tela"; teste com 5 a 8 pessoas antes de publicar |
| Acessibilidade | Tamanho de fonte do sistema, leitor de tela, contraste (UI/UX §6) |
| Conformidade | LGPD, Marco Civil (art. 15), CDC; Termos e Política iguais ao que o código faz |

## 7. Fora do escopo da versão 1 (decidido)

Rede mesh e modo offline no NimbusChat; IA embarcada ou de terceiros; cópia de segurança na nuvem; login por telefone ou e-mail; anúncios; SDK de análise ou de falhas de terceiros.

## 8. Métricas de sucesso (sem coletar dado de usuário)

Medidas sem rastrear ninguém: contagens anônimas no servidor (pedidos recusados por tipo, filas, tempos de entrega), avaliações nas lojas, testes de usabilidade, achados de pentest e do programa de recompensa, e o número de pedidos de autoridade respondidos com "não existe".

## 9. Riscos de produto

| Risco | Resposta |
|---|---|
| Usuário perde todos os meios de recuperação | Explicado no cadastro e nos Termos; amigos de confiança e chave física recomendados |
| Golpe de engenharia social | 27 vetores tratados (documento de engenharia social) |
| Atrito de segurança espanta usuário | Esperas e confirmações só onde há risco real; testes de usabilidade |
| Lojas mudam regras | Plano por loja; build reproduzível no Android |

## 10. Critério de pronto

`CHECKLIST_DE_ENTRADA_EM_PRODUCAO.md` inteiro marcado, com selo de liberação assinado por três chaves.
