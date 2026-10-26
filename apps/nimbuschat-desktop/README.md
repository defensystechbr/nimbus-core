# NimbusChat — computador (macOS e Windows)

Aparelho vinculado a um celular já ativo (manual, A6X; tarefa T11B). Núcleo pela mesma ponte da T1.

- Vincula por QR mostrado no computador e lido pelo celular, com biometria no celular; entra no registro público de chaves.
- Chave no chip do computador (Secure Enclave no Mac, TPM 2.0 no Windows) e disco do sistema cifrado (FileVault ou BitLocker). Sem isso, `integridade::avaliar` dá vermelho e não vincula.
- Nunca passa do amarelo (`integridade::tem_atestacao_do_fabricante`): sem Cofre, sem visualização única, sem NimbusDrop.
- O vínculo cai após 7 dias sem o celular confirmar; o celular derruba o computador a qualquer momento.
- Janela fora da captura do sistema (Windows: `WDA_EXCLUDEFROMCAPTURE`; macOS: janela não compartilhável) — validar em cada versão (VF27).
- Provas: C89 e C90.
