# Metadados — o que sobra, quem vê e até onde dá para reduzir

**Versão 37 · Confidencial**

Pergunta do fundador: *"Não dá para tornar impossível para todos ver os metadados mínimos do servidor? Não dá para criptografar isso?"*

**Resposta curta e verdadeira:** dá para criptografar tudo o que fica **guardado** e para impedir que qualquer pessoa sozinha **ligue** um metadado a alguém. Não dá para zerar o que o servidor percebe **no instante em que entrega** um bloco: para entregar, ele precisa saber para qual caixa vai — assim como o carteiro precisa ler o endereço do envelope, mesmo sem abrir a carta. O que fizemos foi fazer esse endereço mudar todo dia, não identificar ninguém, não ficar guardado e não poder ser ligado a uma pessoa sem a combinação de várias partes independentes.

## 1. Cada metadado, quem vê e o que seria preciso para ligá-lo a alguém

| Metadado | Quem vê | Para ligar a uma pessoa seria preciso | O que o protege |
|---|---|---|---|
| Conteúdo | Ninguém além das pontas | Impossível mesmo com todos | Cifra ponta a ponta híbrida pós-quântica |
| Quem enviou | Ninguém | Impossível pelo servidor | Envelope sem remetente; remetente só dentro da cifra |
| IP do aparelho | Só o 1º retransmissor, que não sabe o destino | O operador do retransmissor **e** o do servidor juntos (empresas diferentes); com Tor, nem assim | Retransmissores independentes (OHTTP); Tor opcional e automático nos momentos de identidade |
| Que caixa recebeu um bloco, e quando | A memória do servidor, durante a entrega | Saber de quem é a caixa — e ela muda todo dia (MET1) | Caixa da época; envelope sem remetente |
| Que blocos alguém busca | Ninguém consegue separar os reais | Distinguir reais de falsos num lote embaralhado | Lotes de tamanho fixo com endereços falsos; cada lote por circuito diferente |
| Tamanho e horário | Quem observa a rede | Separar mensagem real de tráfego de cobertura | Classes de tamanho fixas; cadência fixa; tráfego de cobertura |
| O que ficou no disco do servidor | Ninguém, depois de 48 h | Impossível: a chave do índice foi destruída (MET2) | Índice cego com chave por dia, destruída |
| Registros de acesso exigidos por lei | Ninguém que opere o servidor | A chave jurídica, dividida entre custodiantes, **e** ordem judicial (MET3) | Registro selado, de tamanho fixo |
| Quem pagou | A loja do celular | Ligar a compra à conta: impossível pelo Nimbus | Ficha cega de licença |

**Linha nova da tabela de "quem consegue burlar":** ver metadados mínimos — **ninguém sozinho consegue ligar um metadado a uma pessoa**. O que uma pessoa sozinha (quem opera o servidor) percebe, e só durante a entrega, é que "um endereço do dia, sem dono conhecido, recebeu um bloco de tamanho padrão nesta hora". Para ligar isso a alguém, são precisos o operador do retransmissor e o do servidor juntos (empresas diferentes) — e, com Tor, nem eles.

## 2. O que entrou no código nesta revisão

1. **Caixa que muda todo dia** (`metadados::caixa_da_epoca`): o endereço de cada relação é recalculado por dia a partir do segredo da relação; os dois lados chegam ao mesmo endereço, e um observador não liga o de hoje ao de ontem.
2. **Índice cego no disco** (`metadados::ChavesDeIndice`): o banco guarda cada bloco sob um índice derivado do endereço com uma chave do dia, mantida só em memória ou em módulo de hardware e destruída 48 h depois. Quem levar o disco depois disso não consegue nem conferir se um endereço conhecido recebeu algo.
3. **Registros legais selados** (`metadados::selar_registro`): data, hora e IP do retransmissor são cifrados, com tamanho fixo, para uma chave jurídica que não fica no servidor e é dividida entre custodiantes (módulo `custodia`). Quem opera o servidor escreve, mas não consegue ler.

## 3. Para reduzir ainda mais — custo e o que cada passo exige

| Passo | O que esconde a mais | Custo e confiança exigida | Situação |
|---|---|---|---|
| Computação confidencial (AMD SEV-SNP ou Intel TDX) com atestação remota conferida pelo app | A memória do servidor fica cifrada até para quem opera a máquina | Custo de nuvem maior; passa a confiar no fabricante do processador; ataques físicos e de canal lateral contra essas tecnologias já foram publicados | Decisão aberta (plano Completo) |
| Recuperação privada de informação (PIR) | O servidor não sabe qual endereço foi buscado | Muito mais banda e processamento por busca | Hoje coberto em parte pelos lotes com endereços falsos |
| Rede de mistura (atraso e embaralhamento entre vários nós) | O horário de envio deixa de casar com o de entrega | Atraso de segundos a minutos; mais nós para operar | Avaliar para o modo Sala Blindada |
| Operadores independentes de verdade | Ligação IP–caixa exige combinar empresas diferentes | Contratos e auditoria de cada operador | Decisão aberta (escolha dos operadores) |

## 4. Verificar antes de usar
1. O estado atual das vulnerabilidades publicadas contra AMD SEV-SNP e Intel TDX, antes de decidir pela computação confidencial.
2. Com o advogado: se o registro de acesso com o IP do retransmissor cumpre o art. 15 do Marco Civil.
