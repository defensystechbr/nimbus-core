# Matriz de rastreabilidade — requisito → núcleo → teste → prova

*Gerada por máquina (`gerar_matriz.py`) a partir de `NimbusOne_Manual_Tecnico_v37.docx` e do código do núcleo. Para refazer depois de qualquer mudança: `python3 gerar_matriz.py <manual.docx> <nimbus-nucleo> <saída>`.*

**Como ler.** "Situação no manual" é a viabilidade decidida (SIM, PARCIAL, NÃO) — não quer dizer que já está pronto. O que já existe em código está na coluna "Evidência no núcleo". Requisitos sem função do núcleo são trabalho dos apps, do servidor, da infraestrutura ou de processo: é exatamente a lista que o time constrói a partir da Tarefa T1.

**Total:** 598 requisitos.

| Situação no manual | Quantidade |
|---|---|
| NÃO | 40 |
| PARCIAL | 67 |
| SIM | 491 |

| Evidência no núcleo | Quantidade |
|---|---|
| sem função do núcleo citada (app, servidor, infraestrutura ou processo) | 360 |
| núcleo: módulo declarado responsável | 109 |
| núcleo: módulo declarado responsável e requisito citado no código | 74 |
| núcleo: função existe e é exercitada em teste | 55 |

## Tabela completa

| Requisito | Situação | Módulo | Funções do núcleo | Provas | Evidência |
|---|---|---|---|---|---|
| PRD5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PRD6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NB1 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC14 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DP1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DP2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DP4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DP5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DP3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| COR1 | SIM | corporativo | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| COR2 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR3 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR4 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR5 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR6 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR7 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR8 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR9 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR10 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR11 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR12 | PARCIAL | corporativo | — | — | núcleo: módulo declarado responsável |
| COR13 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR14 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR15 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| COR16 | PARCIAL | corporativo | — | — | núcleo: módulo declarado responsável |
| COR17 | SIM | corporativo | — | — | núcleo: módulo declarado responsável |
| PLN1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PLN2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PLN3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PLN4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HIS1 | SIM | conversa | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| HIS2 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS3 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS4 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS5 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS6 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS7 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS8 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS9 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS10 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS11 | PARCIAL | conversa | — | — | núcleo: módulo declarado responsável |
| HIS12 | PARCIAL | conversa | — | — | núcleo: módulo declarado responsável |
| HIS13 | SIM | conversa | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| HIS14 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS15 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| LIB1 | SIM | liberacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| LIB2 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB3 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB4 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB5 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB6 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB7 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB8 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB9 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB10 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB11 | PARCIAL | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB12 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB13 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB14 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB15 | SIM | liberacao | — | — | núcleo: módulo declarado responsável |
| LIB16 | SIM | liberacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| AUD1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD9 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD11 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD12 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD13 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD14 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD15 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD16 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD17 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD18 | SIM | — | — | C79 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD19 | SIM | — | — | C80 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD20 | SIM | — | — | C79; C80 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD21 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD22 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD23 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD24 | SIM | — | — | C81 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD25 | SIM | — | — | C82 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD26 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD27 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD28 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD29 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD30 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD31 | SIM | — | — | C86 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD32 | SIM | — | — | C87; C88 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD33 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD34 | SIM | — | expiracao::emitir_hora; expiracao::conferir_hora | — | núcleo: função existe e é exercitada em teste |
| AUD35 | SIM | — | coacao::RelogioDeEspera | — | núcleo: função existe e é exercitada em teste |
| AUD36 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD37 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD38 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD39 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD40 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD41 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD42 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD43 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD44 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD45 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AUD46 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG1 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG4 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG5 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG6 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG7 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG8 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SUG9 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PAG1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PAG2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PAG3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PAG4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PAG5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DUO1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DUO2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| KIT1 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| OPE1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| OPE2 | SIM | — | expiracao::TEMPORIZADORES | — | núcleo: função existe e é exercitada em teste |
| OPE3 | SIM | — | expiracao::VIDA_MAXIMA | — | núcleo: função existe e é exercitada em teste |
| OPE4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| APR1 | SIM | apresentacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| APR2 | SIM | apresentacao | — | — | núcleo: módulo declarado responsável |
| APR3 | SIM | apresentacao | — | — | núcleo: módulo declarado responsável |
| APR4 | PARCIAL | apresentacao | — | — | núcleo: módulo declarado responsável |
| CNV1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CNV2 | SIM | — | convites::palavras_de_confirmacao | — | núcleo: função existe e é exercitada em teste |
| INS1 | SIM | inspecao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| INS2 | SIM | inspecao | — | — | núcleo: módulo declarado responsável |
| INS3 | NÃO | inspecao | — | — | núcleo: módulo declarado responsável |
| IMP1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| IMP2 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DIR1 | SIM | direto | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| LNK1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LNK2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LNK3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| IPN1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MCI1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MCI2 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX9 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX11 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX12 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| UX13 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INT1 | SIM | — | integridade::Desafios | — | núcleo: função existe e é exercitada em teste |
| INT2 | SIM | — | integridade::sessao_permitida | — | núcleo: função existe e é exercitada em teste |
| INT3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INT4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INT5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INT6 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INT7 | NÃO | — | — | C76 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF1 | SIM | — | — | C79 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF2 | SIM | — | — | C80 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF4 | SIM | — | — | C1 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF5 | SIM | — | — | C76; C77 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF8 | SIM | — | — | C79 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF9 | SIM | — | — | C80 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VRF7 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AT1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AT2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AT3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AT4 | PARCIAL | — | — | C76 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REC1 | SIM | recuperacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| REC2 | SIM | recuperacao | — | — | núcleo: módulo declarado responsável |
| REC3 | PARCIAL | recuperacao | — | — | núcleo: módulo declarado responsável |
| APP1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| APP2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| APP3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| APP4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEP1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEP2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEP3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEP4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SV1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SV2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SV3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SV4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| KS1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| KS2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| KS3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| KS4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM1 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM2 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM3 | PARCIAL | — | — | C-WA | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM5 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM7 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM8 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| LM6 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DV1 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DV2 | SIM | integridade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| DV3 | SIM | integridade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| DV4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DV5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DV6 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GR1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GR2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GR3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GR4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VM1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VM2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VM3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VM5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VM5b | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VM4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VM7 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ID0 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| TF1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| TF2 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| TF3 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| TF4 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| TF5 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| TF6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VAL0 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VAL0b | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ENV0 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ENV0b | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HIG0 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ0 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ0b | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ0c | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BC1 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BC2 | SIM | registro | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| BC3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BC4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PG1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PG2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PG3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PG4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EX1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EX2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EX3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BIO1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BIO2 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BIO3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MAL1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MAL2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MAL3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MAL4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MAL5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MAL6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MAL7 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| FL1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| FL2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| FL3 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| FL4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NET1 | SIM | rede | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| NET2 | SIM | rede | rede::decidir | — | núcleo: função existe e é exercitada em teste |
| NET3 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET4 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET5 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET6 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET7 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET8 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET9 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET10 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET11 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET12 | SIM | rede | — | — | núcleo: módulo declarado responsável |
| NET13 | PARCIAL | rede | — | — | núcleo: módulo declarado responsável |
| NET14 | SIM | rede | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| CTT1 | SIM | contatos | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| CTT2 | SIM | contatos | — | — | núcleo: módulo declarado responsável |
| CTT3 | SIM | contatos | — | — | núcleo: módulo declarado responsável |
| CTT4 | SIM | contatos | — | — | núcleo: módulo declarado responsável |
| CTT5 | PARCIAL | contatos | — | — | núcleo: módulo declarado responsável |
| RSK1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RSK2 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RSK3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RSK4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RSK5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RSK6 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RSK7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RSK8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER1 | SIM | — | — | C79; C80 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VER8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INV1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INV2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| INV3 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NOT8 | SIM | — | notificacao::canal_para | — | núcleo: função existe e é exercitada em teste |
| NOT9 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NOT10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NOT11 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NOT12 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PQ1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PQ2 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PQ3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| POS1 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| POS2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| POS3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| WEB5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| WEB6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| WEB7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| WEB8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| WEB9 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PC1 | SIM | integridade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| PC2 | SIM | integridade | integridade::avaliar | C89 | núcleo: função existe e é exercitada em teste |
| PC3 | PARCIAL | integridade | integridade::tem_atestacao_do_fabricante | — | núcleo: função existe e é exercitada em teste |
| PC4 | SIM | integridade | — | C90 | núcleo: módulo declarado responsável |
| PC5 | PARCIAL | integridade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| OFF1 | SIM | — | — | C92 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| OFF2 | SIM | expiracao | expiracao::hora_confiavel; expiracao::pode_exibir; expiracao::conferir_hora | C91 | núcleo: função existe e é exercitada em teste |
| OFF3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| OFF4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ND2 | SIM | direto | direto::lacrar | — | núcleo: função existe e é exercitada em teste |
| ND3 | SIM | direto | — | — | núcleo: módulo declarado responsável |
| ND4 | SIM | direto | direto::pode_abrir_lacrado; direto::abrir_lacrado | C93 | núcleo: função existe e é exercitada em teste |
| ND5 | SIM | direto | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| PQ4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DIR8 | SIM | direto | direto::preparar_assinado; direto::abrir_assinado | C94 | núcleo: função existe e é exercitada em teste |
| BL6 | SIM | registro | registro::verificar_consistencia | C95 | núcleo: função existe e é exercitada em teste |
| CRY9 | SIM | envelope | envelope::ChavePublica::de_bytes | C96 | núcleo: função existe e é exercitada em teste |
| EO1 | SIM | — | convites::opcoes_para_confirmar; convites::confirmacao_correta | C97 | núcleo: função existe e é exercitada em teste |
| EO2 | SIM | — | sessoes::pode_vincular; sessoes::palavras_do_vinculo; sessoes::Registro::conectar | C98 | núcleo: função existe e é exercitada em teste |
| EO3 | SIM | — | apresentacao::nome_permitido; apresentacao::parecido_com; apresentacao::mudanca_de_nome; apresentacao::subtitulo | C99 | núcleo: função existe e é exercitada em teste |
| EO4 | PARCIAL | — | recuperacao::abrir_para_devolver; recuperacao::palavras_do_pedido; recuperacao::pode_concluir | C100 | núcleo: função existe e é exercitada em teste |
| EO5 | SIM | — | corporativo::autorizar_recuperacao | C101 | núcleo: função existe e é exercitada em teste |
| EO6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EO7 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EO8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EO9 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EO10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| FS1 | SIM | prechaves | prechaves::gerar | C102 | núcleo: função existe e é exercitada em teste |
| FS2 | SIM | prechaves | prechaves::conferir | C102 | núcleo: função existe e é exercitada em teste |
| FS3 | SIM | prechaves | prechaves::selar | C102 | núcleo: função existe e é exercitada em teste |
| FS4 | SIM | prechaves | prechaves::Chaveiro::apagar_vencidas | C102 | núcleo: função existe e é exercitada em teste |
| FS5 | SIM | prechaves | expiracao::hora_confiavel | — | núcleo: função existe e é exercitada em teste |
| PQS7 | SIM | — | assinatura::ChaveDeAssinatura::assinar | C103 | núcleo: função existe e é exercitada em teste |
| IA8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SRV1 | SIM | autenticacao | autenticacao::ROTAS; autenticacao::rota | C104 | núcleo: função existe e é exercitada em teste |
| SRV2 | SIM | autenticacao | autenticacao::Porteiro::autorizar | C104 | núcleo: função existe e é exercitada em teste |
| SRV3 | SIM | autenticacao | autenticacao::provar | C104 | núcleo: função existe e é exercitada em teste |
| SRV4 | SIM | autenticacao | — | C104 | núcleo: módulo declarado responsável |
| SRV5 | SIM | autenticacao | limites::Limitador | C105 | núcleo: função existe e é exercitada em teste |
| SRV6 | SIM | autenticacao | — | C106 | núcleo: módulo declarado responsável e requisito citado no código |
| SRV7 | SIM | limites | — | C106 | núcleo: módulo declarado responsável e requisito citado no código |
| SRV8 | SIM | limites | — | — | núcleo: módulo declarado responsável |
| SRV9 | SIM | limites | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| SRV10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEG1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEG2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEG3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEG4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEG5 | SIM | senha | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| SEG6 | SIM | — | autenticacao::Porteiro::autorizar | C107 | núcleo: função existe e é exercitada em teste |
| SEG7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEG8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SEN1 | SIM | sentinela | sentinela::autoverificar | C108 | núcleo: função existe e é exercitada em teste |
| SEN2 | SIM | sentinela | sentinela::Relatorio::texto | C108 | núcleo: função existe e é exercitada em teste |
| SEN3 | SIM | sentinela | — | C108 | núcleo: módulo declarado responsável |
| SEN4 | SIM | sentinela | — | — | núcleo: módulo declarado responsável |
| SEN5 | SIM | sentinela | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| GOV1 | SIM | — | — | C109 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GOV2 | SIM | — | — | C109 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GOV3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GOV4 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SLO1 | SIM | selo | selo::conferir | C110 | núcleo: função existe e é exercitada em teste |
| SLO2 | SIM | selo | — | C110 | núcleo: módulo declarado responsável |
| SLO3 | SIM | selo | sentinela::autoverificar_com_selo | C110 | núcleo: função existe e é exercitada em teste |
| SLO4 | SIM | selo | — | C111 | núcleo: módulo declarado responsável |
| SLO5 | PARCIAL | selo | — | — | núcleo: módulo declarado responsável |
| SLO6 | SIM | selo | — | — | núcleo: módulo declarado responsável |
| SLO7 | SIM | selo | selo::MINIMO_DE_ASSINATURAS | C112 | núcleo: função existe e é exercitada em teste |
| GOV5 | PARCIAL | — | — | C112 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| GOV6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ATU1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ATU2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| JUR1 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MET1 | SIM | metadados | metadados::caixa_da_epoca | C113 | núcleo: função existe e é exercitada em teste |
| MET2 | SIM | metadados | metadados::ChavesDeIndice | C113 | núcleo: função existe e é exercitada em teste |
| MET3 | SIM | metadados | metadados::selar_registro | C113 | núcleo: função existe e é exercitada em teste |
| MET4 | SIM | metadados | — | — | núcleo: módulo declarado responsável |
| MET5 | PARCIAL | metadados | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| API1 | SIM | — | autenticacao::resposta_publica | C114 | núcleo: função existe e é exercitada em teste |
| API2 | SIM | — | autenticacao::origem_permitida | C114 | núcleo: função existe e é exercitada em teste |
| API3 | SIM | — | autenticacao::cabecalhos_da_api | C114 | núcleo: função existe e é exercitada em teste |
| API4 | SIM | — | — | C114 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AMB1 | SIM | ambiente | ambiente::avaliar | C115 | núcleo: função existe e é exercitada em teste |
| AMB2 | SIM | ambiente | ambiente::combinar | C115 | núcleo: função existe e é exercitada em teste |
| ROT1 | SIM | chaves_do_servidor | chaves_do_servidor::conferir | C116 | núcleo: função existe e é exercitada em teste |
| ROT2 | SIM | chaves_do_servidor | — | — | núcleo: módulo declarado responsável |
| MET6 | SIM | — | metadados::ChavesDeIndice::selar_no_disco | C117 | núcleo: função existe e é exercitada em teste |
| MET7 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RED1 | SIM | — | — | C118 | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RED2 | SIM | — | apresentacao::nome_permitido | C118 | núcleo: função existe e é exercitada em teste |
| RED3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VSE1 | SIM | vigia_de_sensores | vigia_de_sensores::decidir | — | núcleo: função existe e é exercitada em teste |
| VSE2 | SIM | vigia_de_sensores | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VSE3 | SIM | vigia_de_sensores | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VSE4 | SIM | vigia_de_sensores | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VSE5 | SIM | vigia_de_sensores | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VSE6 | SIM | vigia_de_sensores | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VSE7 | SIM | vigia_de_sensores | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VSE8 | SIM | vigia_de_sensores | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| AUT1 | SIM | autenticidade | autenticidade::assinatura_do_app_confere | — | núcleo: função existe e é exercitada em teste |
| AUT2 | SIM | autenticidade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| AUT3 | SIM | autenticidade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| AUT4 | SIM | autenticidade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| AUT5 | SIM | autenticidade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| BLQ1 | SIM | protecao_do_usuario | protecao_do_usuario::ListaDeBloqueio | — | núcleo: função existe e é exercitada em teste |
| BLQ2 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| BLQ3 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| BLQ4 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| DEN1 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| DEN2 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável |
| DEN3 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável |
| DEN4 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável |
| DEN5 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| APG1 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| APG2 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| APG3 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| APG4 | SIM | protecao_do_usuario | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| HIS17 | SIM | conversa | — | — | núcleo: módulo declarado responsável |
| HIS18 | SIM | conversa | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| EC0 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ANO1 | SIM | trafego | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ANO2 | SIM | trafego | — | — | núcleo: módulo declarado responsável |
| ANO3 | SIM | trafego | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ANO3b | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ANO4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ANO5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ANO6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ANO7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ANO8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ANO9 | PARCIAL | acesso | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ANO10 | SIM | acesso | — | — | núcleo: módulo declarado responsável |
| ANO11 | SIM | acesso | — | — | núcleo: módulo declarado responsável |
| ANO12 | SIM | acesso | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ANO13 | SIM | acesso | — | — | núcleo: módulo declarado responsável |
| ANO14 | PARCIAL | acesso | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| CG1 | SIM | sessoes | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| CG2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CG3 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HY1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HY2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HY3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HY4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HY5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| HY6 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CO1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CO2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CO3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CO4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CO5 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CO6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CO7 | SIM | registro | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| EF1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EF2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EF3 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EF4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ3 | SIM | memoria | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| RZ4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ8 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ9 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RZ11 | SIM | memoria | memoria::Segredo | — | núcleo: função existe e é exercitada em teste |
| RZ12 | SIM | memoria | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| RZ13 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| MTA1 | SIM | pacote | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| MTA2 | SIM | pacote | — | — | núcleo: módulo declarado responsável |
| MTA3 | SIM | pacote | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VAL1 | SIM | validade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VAL2 | SIM | validade | — | — | núcleo: módulo declarado responsável |
| VAL3 | SIM | validade | historico::guardar | — | núcleo: função existe e é exercitada em teste |
| VAL4 | SIM | validade | validade::deve_esconder | — | núcleo: função existe e é exercitada em teste |
| VAL5 | SIM | validade | — | — | núcleo: módulo declarado responsável |
| VAL6 | SIM | validade | — | — | núcleo: módulo declarado responsável |
| VAL7 | SIM | validade | — | — | núcleo: módulo declarado responsável |
| VAL8 | PARCIAL | validade | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ENV1 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ENV2 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ENV3 | NÃO | senha | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ENV4 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ENV5 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ENV6 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ENV7 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SB1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SB2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SB3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SB4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SB5 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SB6 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| SB7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NOT1 | SIM | notificacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| NOT2 | SIM | notificacao | — | — | núcleo: módulo declarado responsável |
| NOT3 | PARCIAL | notificacao | — | — | núcleo: módulo declarado responsável |
| NOT4 | SIM | notificacao | — | — | núcleo: módulo declarado responsável |
| NOT5 | SIM | notificacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| NOT6 | SIM | notificacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| NOT7 | SIM | notificacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| DIS1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DIS2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DIS3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DIS4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| DIS5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BC5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BL4b | SIM | aprovacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| BC6 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ASS1 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ASS2 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS3 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS4 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS5 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS6 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS7 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS8 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS9 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| ASS10 | SIM | aprovacao | — | — | núcleo: módulo declarado responsável |
| CH1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CH2 | SIM | expiracao, politica | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| CL1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| CT1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NT1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AL1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AL2 | SIM | coacao | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| PW1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ND1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| AR1 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC9 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC11 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC12 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| NC13 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| EC1 | SIM | escudo | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| EC2 | SIM | escudo | — | — | núcleo: módulo declarado responsável |
| EC3 | SIM | escudo | — | — | núcleo: módulo declarado responsável |
| EC4 | SIM | escudo | — | — | núcleo: módulo declarado responsável |
| EC5 | SIM | escudo | — | — | núcleo: módulo declarado responsável |
| EC6 | SIM | escudo | — | — | núcleo: módulo declarado responsável |
| EC7 | SIM | escudo | — | — | núcleo: módulo declarado responsável |
| EC8 | PARCIAL | escudo | — | — | núcleo: módulo declarado responsável |
| EC9 | NÃO | escudo | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| VC1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VC2 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VC3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| VC4 | SIM | voz | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| BA1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BA2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BA3 | SIM | senha | — | C80 | núcleo: módulo declarado responsável e requisito citado no código |
| BA4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BA5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| BA6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| FP1 | SIM | envelope | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| FP2 | SIM | envelope | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| ID1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| ID2 | SIM | sessoes | — | — | núcleo: módulo declarado responsável e requisito citado no código |
| RC1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RC2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| RC3 | NÃO | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PL1 | PARCIAL | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| PL2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV1 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV2 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV3 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV4 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV5 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV6 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV7 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV8 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV9 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV10 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV11 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
| REV12 | SIM | — | — | — | sem função do núcleo citada (app, servidor, infraestrutura ou processo) |
