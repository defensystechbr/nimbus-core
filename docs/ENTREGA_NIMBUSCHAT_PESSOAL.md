# Entrega final do NimbusChat — versão de uso pessoal

**Pedido do dono em 06/10/2026:** entregar aos engenheiros todo o material do NimbusChat, primeiro só a versão de uso pessoal, com todos os recursos de segurança, criptografia, anonimato e proteção contra ataques e engenharia social; refazer todos os testes, ataques e verificações; revisar todas as travas.

Este arquivo é o roteiro de retomada. Cada linha diz se está feita e como foi conferida. Ao retomar, comece pela primeira linha "a fazer". Decisões anteriores que valem aqui: envio só dentro do app; somente app, sem nada web (`MUDANCA_ENVIO_SO_DENTRO_DO_APP.md`); o NimbusChat roda no iPhone pela App Store, cumprindo as regras da Apple.

## Escopo da versão pessoal

- **Entra:** tudo das telas do NimbusChat, exceto as duas da conta corporativa.
- **Fica para depois, sem apagar nada do núcleo:** "Conta corporativa" (`contacorp`) e "Acesso suspenso" (`suspenso`), que pertencem à versão empresa.
- **Falta e é exigido pela App Store** (regras 1.2 e 5.1.1(v), conferidas na fonte em 06/10/2026):
  - bloquear contato;
  - denunciar mensagens, por iniciativa do usuário, que escolhe o que enviar;
  - apagar a conta dentro do app.

  O núcleo **não tinha** nenhum dos três. O módulo `apresentacao` até dizia ao usuário que ele podia bloquear um contato a qualquer momento, sem que isso existisse.

## Decisões do dono incorporadas (06/10/2026)

- Coação ao apagar a conta: o app mostra falha de rede e nada é apagado (aprovado).
- Denúncia só da conta, sem nenhum conteúdo (o app não deixa nada sair).
- Retirar mensagem: antes de lida sempre; depois de lida salvo se algum participante desligar.
- Só existe o aviso "visualizada": opcional, recíproco e desligado por padrão. Não existem "entregue", "digitando", "online" nem "visto por último" (decisão de 06/10/2026, Registro item 76).
- Nada sai do aparelho por padrão; à Apple, a verdade: entregamos tudo o que temos e, por engenharia, não temos conteúdo (regra 17).
- Notificações: Android e computador por conexão própria, sem o Google; iPhone pelo serviço da Apple, sempre vazio e cifrado; nenhuma prévia por padrão; nunca o nome de quem liga (Registro, item 78).
- Sem histórico no servidor na versão pessoal; ao trocar de aparelho, o novo começa vazio; um aparelho por vez.

## Fases

| # | Fase | Situação |
|---|---|---|
| 1 | Este roteiro e o escopo | Feito |
| 2 | Núcleo: bloquear, denunciar (só a conta) e apagar a conta; retirar mensagem; só o aviso "visualizada" (opcional e recíproco); barreiras contra compartilhar e contra sinais de presença | **Concluída.** Suíte 457 aprovados, 0 falhas, 0 avisos (06/10/2026, `cargo test --locked --no-fail-fast`, pasta própria); defeitos plantados novos pegos: 21 de 21 (nº 270–290, antes da retirada dos avisos) e, depois da retirada, 5 de 5 (nº 284–288); pisos 457 testes e 288 defeitos plantados |
| 2b | Sem rastro no aparelho: sonda de vazamento, barreira contra gravar fora do contêiner, escudo nativo com chave só com o app aberto e Keychain limpo, documento `NIMBUSCHAT_SEM_RASTRO_NO_APARELHO.md` | Feito: suíte 463 aprovados, 0 falhas, 0 avisos (06/10/2026, `cargo test --locked --no-fail-fast`, pasta própria); piso 463 testes. Decisões do dono sobre notificação e histórico incorporadas (item 78). |
| 3 | Material do NimbusChat: protótipo, telas, Manual, documentos e guia visual | **Feito (06/10/2026):** protótipo sem as telas corporativas (NimbusChat pessoal: 27 telas) e com bloquear, denunciar só a conta, apagar conta, retirar mensagem, "visualizada" e notificação sem prévia; "Recuperar conta" agora diz a verdade (o aparelho novo começa vazio); texto da notificação na tela igual ao do código ("Nova atividade"); `TELAS.md` passou a ser gerado de verdade do protótipo (`gerar_telas.py`); pasta `web/envio` retirada do kit; Manual com ENV0–ENV9 e SB6 marcados como retirados, SB1 e SB4 só no app, e 15 requisitos novos (BLQ1–BLQ4, DEN1–DEN5, APG1–APG4, HIS17, HIS18); mapa do ASVS regerado do repositório oficial; todos os documentos corrigidos; números e fatos conferem em 179 materiais; consistência e catraca dos dois kits aprovadas; guias visuais em PDF regerados |
| 4 | Ataques e varreduras de novo sobre o material final | **Feito (06/10/2026):** texto invisível em todos os materiais e formatos: nenhum (ataques plantados pegos); sintaxe do escudo nativo aprovada (erro plantado pego; não é compilação); catraca 31 de 31 afrouxamentos pegos; auditor 7 de 7; verificar.sh 9 de 9; portão de entrega 16 de 16; governança 20 de 20; dependências: 84 versões, base RustSec de 3 dias, nenhuma vulnerabilidade; consistência, catraca e varredura dos dois kits aprovadas; gitleaks pegou o segredo plantado e não achou nenhum segredo nas fontes nem nas saídas |
| 5 | Provas longas sobre o código final | **Feito (06/10/2026), código `31b0cc40…`:** tempo constante e tempo de recusa sozinhos na máquina, aprovados; suíte 463 aprovados; fuzzing profundo de 2 milhões de entradas em 19 alvos, nenhum pânico; mutação 288 de 288 em oito faixas (três interrompidas por reinício foram refeitas do começo); medidor: zero registros que não valem |
| 6 | (próxima ao retomar) Medidor de provas, pré-conferência pelo portão e as três partes, com o pacote do NimbusChat pronto para a equipe | A fazer |

## Regra desta entrega

Nada é declarado pronto antes da fase 6 aprovar. Cada fase fecha com a suíte completa aprovada e o roteiro atualizado.
