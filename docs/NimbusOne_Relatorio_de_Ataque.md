# Relatório de ataque — o Nimbus atacado por dentro

**Versão 37 · Confidencial · Ataque feito contra o próprio código, no ambiente de desenvolvimento, com a autorização do fundador**

Objetivo: tentar de verdade quebrar o que foi construído — não reler, atacar. Cada ataque abaixo foi **executado**, e o resultado é o que aconteceu, não o que se esperava. Os ataques que acharam falha viraram teste permanente em `tests/ataque_vermelho.rs`: se a defesa sumir um dia, o teste reprova.

## 1. Resultado em uma tabela

| # | Ataque executado | Resultado | Gravidade | Situação |
|---|---|---|---|---|
| A1 | Nome de contato "Suporte Nimbus" escrito com letras de imitação: largura total (Ｎｉｍｂｕｓ), letras matemáticas (𝐍𝐢𝐦𝐛𝐮𝐬) e N grego (Νimbus) | **3 dos 7 disfarces passaram** pela proteção de nomes reservados | Alta (abre o golpe do falso suporte) | **Corrigido**: letras de imitação recusadas; mistura de alfabetos (latino com grego ou cirílico) recusada; letras gregas e cirílicas parecidas comparadas como latinas. 7 de 7 recusados agora; nomes reais gregos e cirílicos continuam aceitos |
| A2 | Enganar a catraca transformando em comentário a proibição de código inseguro, a parada por estouro e a conferência do selo | **A catraca aceitou** a proibição de código inseguro comentada | Alta (manutenção maliciosa passaria pela catraca) | **Corrigido**: a catraca só conta código de verdade, sem comentários. Os 3 disfarces agora reprovam |
| A3 | Derrubar a ferramenta do selo com hexadecimal malformado e acentuado | Recusou com erro; não caiu | Baixa | Endurecido: só aceita dígitos hexadecimais antes de qualquer conversão |
| A4 | Valores extremos de tempo e época (o maior número possível) em todos os módulos novos | **Estouro aritmético na faxina das chaves de índice** (o servidor pararia) | Média (negação de serviço) | **Corrigido** com aritmética saturada; procurado o mesmo padrão no resto do código: nenhum outro ponto de risco |
| A5 | Martelo aleatório: 20 mil pedidos com método, caminho, corpo, hora, prova e origem aleatórios contra o porteiro | Nenhuma queda; toda resposta foi um dos 6 códigos genéricos | — | Virou teste permanente |
| A6 | Injeção pelo ajuste da caixa no banco | **A documentação mandava montar `SET LOCAL` com texto** — o tipo de SQL que a nossa própria regra proíbe | Alta se seguida ao pé da letra | **Corrigido**: instrução trocada por `set_config('nimbus.caixa', $1, true)` com parâmetro; ataque de injeção testado num PostgreSQL real: valor vira texto e não abre nada |
| A7 | Quem tem acesso de administrador à máquina de produção roda um servidor modificado | **O selo não impede.** O servidor confere o próprio selo, e um binário modificado pode mentir sobre si mesmo | Alta (limite de desenho) | **Não corrigível só por software** — ver seção 2 |

## 2. A descoberta mais importante: o selo não protege contra quem controla a máquina

Até aqui, a tabela dizia "pôr código malicioso rodando no servidor: 3 pessoas, uma de fora". **Isso só vale para o caminho oficial de liberação.** Quem tem acesso de administrador à máquina de produção pode rodar outro programa, e esse programa pode dizer que tem o selo certo — um programa conferindo a si mesmo não prova nada.

**O que esse servidor malicioso consegue e o que não consegue:**

| Consegue | Não consegue |
|---|---|
| Ver os metadados do instante da entrega e passar a guardá-los (anulando a rotação e o índice cego daquele período em diante) | Ler conteúdo: não existe chave de conteúdo no servidor |
| Negar serviço | Atrasar o relógio dos aparelhos para esticar prazos: o app usa sempre o maior entre o relógio próprio e o do servidor |
| Usar a chave do dia para assinar respostas daquele dia | Usar a chave depois do dia, nem criar certificado novo: as raízes estão offline, com 2 de 3 pessoas |
| | Ver o IP dos aparelhos: ele para no retransmissor de outra empresa |

**Tabela corrigida:**

| Objetivo | Uma pessoa sozinha? | Mínimo combinado |
|---|---|---|
| Pôr código malicioso pelo caminho oficial de liberação | Não | 3, uma de fora da empresa |
| Rodar código malicioso tendo acesso de administrador à máquina de produção | **Sim, hoje** | 1 — sem conteúdo, mas com metadados do instante da entrega |
| Com as medidas da seção 3 aplicadas | Não | 2 (acesso à produção exige duas pessoas) e, com computação confidencial, também quebrar o hardware |

## 3. O que fecha a A7, em ordem

1. **Acesso à produção só com duas pessoas**: ninguém entra sozinho na máquina; entrada de emergência com aprovação de outra pessoa e registro que o próprio operador não apaga.
2. **Infraestrutura imutável**: as máquinas só rodam imagens assinadas, conferidas **por fora do programa** (pelo orquestrador, antes de iniciar); ninguém altera máquina em funcionamento — troca-se a máquina.
3. **Computação confidencial com atestação remota**: o módulo de hardware só entrega a assinatura do dia a um servidor cuja medição bate com o artefato do selo. Aí sim um servidor modificado não consegue assinar nada. É a decisão aberta do plano Completo — esta descoberta a torna **recomendada**.
4. **Vigilância de fora**: monitoramento externo dos certificados do dia e do registro público, por quem não opera o servidor.

## 3b. Segunda onda (executada depois do "continuar")

| # | Ataque executado | Resultado | Gravidade | Situação |
|---|---|---|---|---|
| B1 | Medir o tempo da recusa do porteiro: titular inexistente contra titular registrado com assinatura errada | **O tempo revelava se uma caixa ou aparelho existe**: 0,7 µs contra 202 µs (razão 293) | Média (enumeração por tempo) | **Corrigido**: a assinatura é sempre conferida, contra uma chave fictícia quando o titular não existe; razão medida depois: 1,01. Medição permanente em `tests/tempo_de_recusa.rs`, rodada pelo `./verificar.sh --completo` |
| B2 | Respostas malformadas da API do GitHub contra o conferidor de governança; esconder uma revisão "pedir mudança" depois da página 1 | Toda resposta malformada reprovou (falha fechada); **mas só a 1ª página de revisões era lida** | Baixa (exige revisores em conluio) | **Corrigido**: todas as páginas são lidas; pedido sem autor identificável reprova. 13 de 13 casos corretos |
| B3 | Adulterar 300 vezes, ao acaso, um selo válido (um caractere por vez) | 0 aceitos, 0 quedas | — | Resistiu |
| B4 | Disfarçar nome reservado com marcas combinantes, seletores de variação, hífen invisível, espaços estranhos | Todos recusados | — | Resistiu; virou teste permanente |
| B5 | Papel do app no banco tentando virar outro papel (SET ROLE, set_config, SET SESSION AUTHORIZATION), conectado como o servidor real | Todos recusados | — | Resistiu; teste permanente com conexão real e defeito plantado (15 de 15 no banco); sentinela do banco passa a conferir que o app não herda outro papel |

**Erro meu no caminho, registrado:** o primeiro teste do B5 rodou conectado como superusuário e "mostrou" uma troca de papel que, conectado como o app, não acontece. O teste foi refeito do jeito certo. O mesmo valeu para a primeira medição do B1, contaminada pelo próprio castigo do limitador; foi refeita com origens separadas.

## 4. O que foi atacado e resistiu (ataques anteriores repetidos nesta rodada)

Criptografia (cifra, assinatura, adulteração, chave malformada), prazos e relógio, PIN e esperas, recuperação, convites, vínculo de aparelho, envio direto, NimbusDrop, registro público, rotas, limites, banco (14 defeitos plantados no esquema), governança e selo — todos pela suíte de testes, pelos defeitos plantados nas partes alteradas e pelo banco real.

## 5. O que este ataque não cobriu, dito com verdade

- Apps compilados e aparelhos reais (ainda não existem): engenharia reversa, ganchos em tempo de execução, canais laterais físicos.
- Infraestrutura real (rede, nuvem, TLS, retransmissores).
- Ataque com agentes de IA (Strix ou equivalente): não roda aqui (sem Docker e sem chave de modelo; e enviaria o código a um provedor de IA).
- Bibliotecas de terceiros por dentro (libsignal, RustCrypto): confiamos nas auditorias delas e nas atualizações com espera.

Esses ficam para o pentest humano e para o teste com agentes de IA no ambiente de teste (PT-01 a PT-48).

## 6. Verificar antes de usar
1. As opções de computação confidencial do provedor escolhido e os ataques publicados contra elas.
2. O mecanismo de conferência de imagem assinada do orquestrador escolhido (conferido fora do programa que roda).
