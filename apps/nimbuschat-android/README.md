# NimbusChat — Android

Kotlin e Jetpack Compose. Telas: `docs/TELAS.md`. `FLAG_SECURE` em todas as janelas de conteúdo; notificações pelo canal próprio (sem Google); sem telefonia.

## Escudo nativo (T2)

`escudo/EscudoNativo.kt`: código de partida, **não compilado na máquina que gerou o pacote**. Lê os sinais oficiais (câmera e microfone de outro app, captura e gravação de tela, tela externa, sobreposição, acessibilidade, teclado, áudio externo) e protege janela, teclado e área de transferência. A decisão é sempre do núcleo: `vigia_de_sensores::decidir`. Aceite: testes RSD1–RSD12 e VSE1–VSE8 em aparelho real (`docs/RESIDUOS_NO_APARELHO_E_LICOES.md`).
