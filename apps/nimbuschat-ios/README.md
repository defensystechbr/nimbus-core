# NimbusChat — iOS

Swift e SwiftUI. Telas: `docs/TELAS.md`. Núcleo pela ponte da T1. Sem CallKit; camada segura contra print; teclados de terceiros bloqueados; seleção de texto desligada nas mensagens.

## Escudo nativo (T2)

`Escudo/EscudoNativo.swift`: código de partida, **não compilado na máquina que gerou o pacote**. Lê os sinais oficiais (captura e gravação de tela, tela externa, interrupção do áudio e da câmera por outro app, rota de áudio), recusa teclados de terceiros, cobre o conteúdo no seletor de apps e tira os arquivos do backup. A decisão é sempre do núcleo: `vigia_de_sensores::decidir`. Aceite: testes RSD1–RSD12 e VSE1–VSE8 em aparelho real (`docs/RESIDUOS_NO_APARELHO_E_LICOES.md`).
