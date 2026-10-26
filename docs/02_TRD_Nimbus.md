# 02 · TRD — Documento de Requisitos Técnicos — Nimbus

**Versão 37 · Confidencial · Fontes detalhadas: Manual Técnico v37, `API_DO_NUCLEO.md`, `CONTRATO_DA_PONTE.md`**

## 1. Arquitetura em uma página

```
[Aparelho]  App nativo (Swift / Kotlin) ── ponte UniFFI ── núcleo Rust (58 módulos)
     │  chave no chip; conteúdo cifrado antes de sair
     ▼
[Retransmissor 1 — empresa A]  vê o IP, não vê o destino (OHTTP)
     ▼
[Servidor Nimbus — empresa B]  porteiro (rotas, prova, limites) → banco PostgreSQL com RLS forçada
     │  vê só caixa do dia, tamanho padrão e hora; guarda índice cego; nunca chave de conteúdo
[Proxy de chamadas]  encaminha mídia cifrada
[Registro público]  chaves e versões, com provas de inclusão e consistência
```

## 2. Pilha

| Camada | Tecnologia | Motivo |
|---|---|---|
| Núcleo | Rust 1.91.1, `#![deny(unsafe_code)]`, perfil de produção com parada em estouro | Memória segura, um código para todas as plataformas |
| Mensagens | libsignal (PQXDH + Triple Ratchet), MLS para grupos | Protocolos auditados |
| Apps | Swift (iOS), Kotlin (Android), apps de computador com o mesmo núcleo | Nativo: atestação, chip, proteção de tela |
| Servidor | Rust, sem estado de conteúdo; módulos `autenticacao`, `limites`, `sentinela`, `selo`, `metadados`, `chaves_do_servidor` | Mesmo código verificado do núcleo |
| Banco | PostgreSQL 16 com RLS ligada e forçada (`servidor/banco/`) | Segunda barreira por caixa |
| Segredos | Módulo de hardware ou cofre da nuvem, sem exportação | O servidor pede assinatura, não a chave |

## 3. Criptografia

Troca de chaves X25519 + ML-KEM-1024; assinatura Ed25519 + ML-DSA-87 (hedged); cifra XChaCha20-Poly1305 (AES-256-GCM na edição FIPS); Argon2id 64 MiB / 3 / 4; HKDF-SHA-512 com contexto por uso. Nenhum algoritmo próprio.

## 4. Rotação de chaves — tudo o que muda, e quando

| Chave | Frequência | Efeito se vazar |
|---|---|---|
| Chave de mensagem (libsignal) | A cada mensagem | Vale para aquela mensagem |
| Chave de recebimento (envio direto, NimbusDrop) | Nova a cada dia; destruída 48 h após vencer | Nada antigo abre depois da destruição |
| Caixa de cada relação | Muda a cada dia | Não liga dias diferentes |
| Chave do índice cego e do disco do servidor | Uma por dia; destruída 48 h depois | Disco antigo vira lixo |
| Chave de assinatura do servidor (hora e respostas) | Uma por dia, certificada por 2 de 3 raízes offline | Perde o valor no dia seguinte |
| Registros legais | Chave jurídica por mês; partes destruídas depois de 7 meses | Registro com mais de 6 meses fica impossível de abrir |
| Chaves-raiz do servidor e do selo | Offline; troca por cerimônia (anual ou em incidente) | Exigem 2 ou 3 pessoas diferentes |

**Por que não trocar a chave de identidade de cada pessoa todo dia:** os contatos teriam de reconferir as palavras a cada troca, e o golpista usaria isso para se passar por alguém ("troquei de chave, confirma aí"). A identidade fica estável e o que protege o passado gira por baixo dela.

## 5. Frequência das verificações automáticas

| Quando | O que roda | Quem reage |
|---|---|---|
| A cada pedido | Porteiro (rota exata, prova, janela, uso único), limitador, RLS do banco | Recusa na hora |
| A cada minuto | `sentinela::conferir_invariantes` (rápido) | Servidor sai do ar se falhar |
| A cada hora | `sentinela::autoverificar_com_selo` (com autotestes criptográficos), sentinela do banco, destruição de chaves vencidas | Servidor sai do ar e alerta |
| A cada abertura do app | Sentinela, integridade e diagnóstico do ambiente | App não abre conteúdo se falhar |
| A cada mudança de código | 17 portões, catraca, duas aprovações | Nada entra |
| Todo dia | Sentinela diária no CI (verificação completa com a base de vulnerabilidades do dia), vigia da proteção | Nada novo é liberado até corrigir |

## 6. Árvore de falhas — quantas coisas precisam dar errado para alguém ler conteúdo

| Caminho do atacante | Falhas independentes necessárias |
|---|---|
| Pelo servidor ou pelo banco | Impossível: não há chave de conteúdo em nenhum lugar do servidor |
| Pela rede gravada | Quebrar X25519 **e** ML-KEM-1024 **e** a cifra autenticada |
| Pelo celular roubado e bloqueado | Quebrar o chip seguro **e** passar o PIN (Argon2id com esperas e apagamento) **e** agir antes de as chaves de recebimento e as mensagens vencerem |
| Por versão maliciosa do app | Três chaves do selo (uma de fora da empresa) **e** duas aprovações **e** passar a catraca e os portões |
| Celular **desbloqueado e infectado** durante o uso | **Uma falha basta para ver o que está na tela naquele momento** — limite físico de qualquer sistema; reduzido por atestação, prazos curtos e diagnóstico do ambiente |

A última linha não pode ser zerada por software, e o produto diz isso ao usuário.

## 7. Contrato da ponte e do servidor

As 49 regras do `CONTRATO_DA_PONTE.md` (apps) e as 7 regras do `servidor/README.md` são obrigatórias e têm teste.

## 8. Desempenho — metas a medir em aparelho real antes do lançamento

Metas de projeto (não medidas ainda em aparelho real; o roteiro de medição está no núcleo): abrir conversa em menos de 300 ms; enviar e receber mensagem de texto em até 1 s em rede comum, somada a cadência de cobertura; derivação de PIN (Argon2id 64 MiB) entre 0,5 e 1,5 s em aparelho de entrada; uso de bateria do tráfego de cobertura medido e limitado antes do lançamento.

## 9. Ferramentas obrigatórias

Ver `NimbusOne_Revisao_de_Fechamento.md`, seção 4 (desenvolvimento, CI, servidor, apps e o que nunca embarcar).
