# Roteiro: testes em aparelho real (provas C76 e C77)

Estes testes provam, no código que vai para produção, que um celular corrompido não consegue usar o app. O
núcleo já traz os testes prontos (`core/nimbus-nucleo/tests/aparelho_real.rs`). Duas etapas:

## Etapa 1 — ligar os sinais reais (uma vez, na Tarefa T1)

No arquivo `tests/aparelho_real.rs`, a função `sinais_do_aparelho` hoje devolve sinais fixos por cenário. Na
ponte da Tarefa T1, substitua o corpo dela pela leitura da atestação da plataforma:

- **Android:** Play Integrity (veredito do dispositivo e do app) + Key Attestation (estado da inicialização
  verificada, origem da chave: `AmbienteSeguro`/`ChipDedicado`, e a lista de revogação do Google). Detecte
  Frida/ganchos e emulador com as verificações do app; leia se há bloqueio de tela.
- **iOS:** App Attest (app genuíno) + DeviceCheck; jailbreak e depurador pelas verificações do app.

Preencha a struct `Sinais` do módulo `integridade` com esses valores. A partir daí, os testes usam os sinais
verdadeiros do aparelho em que rodam.

## Etapa 2 — rodar em cada aparelho preparado

| Cenário | Como preparar o aparelho | Resultado esperado |
|---|---|---|
| `limpo` (referência) | Aparelho de fábrica, atualizado, com bloqueio de tela | VERDE |
| `root_escondido_magisk` | Magisk com DenyList e módulo de ocultação (Shamiko) ligados | VERMELHO |
| `frida` | `frida-server` em execução, anexado ao app | VERMELHO |
| `emulador` | Android Studio Emulator ou equivalente | VERMELHO |
| `chave_atestacao_vazada` | Chave de atestação de fabricante que consta na lista de revogação | VERMELHO |
| `atestacao_software` | Aparelho cuja atestação só existe em software | VERMELHO |
| `jailbreak` (iOS) | iPhone com jailbreak | VERMELHO |
| `bootloader_destravado` | `fastboot flashing unlock` | VERMELHO |
| `sem_senha` | Remover a senha/PIN de bloqueio | VERMELHO |
| `grapheneos` | GrapheneOS com atestação de hardware | VERDE |

Comando, no aparelho conectado:

```bash
cd core/nimbus-nucleo
cargo test --test aparelho_real -- --ignored --nocapture
```

Cada cenário imprime `OK <cenário>: VERMELHO` (ou VERDE). Qualquer aparelho comprometido que saia diferente de
VERMELHO é uma falha que bloqueia o lançamento.

## Prova C77 — atestação reenviada (roda sem aparelho)

```bash
cargo test --test servidor_atestacao
```

Prova que a resposta de atestação de um aparelho limpo não pode ser reenviada de um aparelho comprometido (o
desafio é de uso único e vence em 60 segundos).

## Prova adicional recomendada

Além destes, contrate um teste de invasão (pentest) independente sobre o app compilado e a infraestrutura,
antes do lançamento. O código passar não substitui um atacante humano tentando quebrar o produto pronto.
