# nimbus-nucleo — núcleo de segurança do NimbusOne

Referência: **Manual Técnico NimbusOne v37.0** (era v31.0 nesta linha — desatualizado por várias rodadas; conferido e corrigido na Rodada 26, 12/10/2026). Tudo em português: código, comentários e testes.
Situação: **protótipo**. Nada vai para produção antes da prova C1 (auditoria independente).

## Módulos

| Módulo | Requisito | Para que serve |
|---|---|---|
| `album` | ALB1–ALB8 | Álbuns e coleções automáticas: a foto entra sozinha na coleção certa por regra (contato de origem, marcação ou padrão); nome da coleção, legenda, pessoas e data vão cifrados no cofre |
| `envelope` | CRY1, CRY3, CRY9, ARQ4, FP1, FP2 | Cifra híbrida X25519 + ML-KEM-1024; suíte padrão XChaCha20-Poly1305 e suíte FIPS AES-256-GCM; preenchimento de tamanho; zeragem de memória. Uso: cofres, álbuns, arquivos, agenda, notas, backup |
| `expiracao` | CH2 | Mensagens temporárias Modo A (após abrir), Modo B (após enviar) e visualização única, com prazo assinado pelo servidor; hora confirmada: sem servidor há mais de 15 min, nada com prazo aparece (OFF2) |
| `integridade` | DV2, DV3, PC1–PC5 | Verificação do aparelho ao abrir o app: níveis verde, amarelo e vermelho, e o que cada um permite; computador nunca passa do amarelo; não existe cliente web |
| `panico` | PAN1–PAN9 | Travas de emergência, decididas dentro do aparelho e sem avisar ninguém: palavra mágica, toque de pânico, longe de casa, movimento brusco, coação (cofre-isca) e sem me mexer; cada uma com a reação escolhida, de trancar a tela a apagar as chaves |
| `sessoes` | ID2, CG1 | Uma sessão por APARELHO registrado (não uma sessão por pessoa): celular e computador vinculado (PC1, nunca independente) convivem de propósito; sessão paralela (o MESMO aparelho conectando duas vezes), clone ou app adulterado derrubam todas as sessões; painel só com contagens |
| `escudo` | EC1–EC9 | Escudo de Conversa: estado de segurança assinado de cada participante; captura de tela, nível não verde, silêncio, falsificação ou repetição borram a conversa em todos os aparelhos |
| `grupo` | — | Conversa em grupo (até 8 pessoas): só a camada de membros e a regra efetiva (combinação mais restritiva de todos); o protocolo de cifra do grupo é o MLS, fora deste núcleo |
| `chamada_em_grupo` | — | Chamada de áudio em grupo (até 8 pessoas) como malha de ligações 1:1 — reaproveita `chamada` sem modificação, perna por perna; não é um SFU |
| `voz` | VZ4, VC4, VN1, VN2 | Nível 1: deslocador de tom em tempo real, com desvio sorteado por chamada (reversível em parte). Nível 2: conversão neural para voz-alvo fixa (não reversível ao timbre original), orquestrada aqui e com inferência nativa; a voz real nunca sai do aparelho. 20 testes (era 15 nesta linha; contagem direta de `#[test]` em `src/voz.rs`, Rodada 26, 12/10/2026 — achado ao acaso nesta mesma auditoria, não parte de uma reconferência sistemática de todas as contagens por módulo desta tabela) |
| `politica` | CH2, CH3, AR2 | Regras por conversa e por mensagem (vence a mais restritiva); visualização única para foto, vídeo, áudio, vídeo circular e arquivo; bloqueios de encaminhar, copiar, salvar e citar |
| `historico` | EF5, EF6, CH4 | Histórico no servidor, cifrado, aberto só no aparelho; identificadores aleatórios; apagamento criptográfico por janela de tempo (cópias guardadas por terceiros ficam ilegíveis) |
| `coacao` | AL2, AR6 | PIN de coação e cofre-isca, com comparação em tempo constante |
| `validade` | VAL1–VAL8 | Apagar sozinho: agenda (contando do fim do compromisso), notas, contatos, arquivos e fotos, com apagamento criptográfico |
| `higiene` | HIG1–HIG6 | Remove EXIF, GPS, autor, comentários e datas de JPEG e PNG; nome de arquivo neutro; recusa PDF e Office sem achatar |
| `registro` | BL1, BL6, CO7, BC2 | Registro público só-de-acréscimo (árvore de Merkle, RFC 9162) para chaves de aparelhos, imagens do Nimbus OS e versões do app; prova de inclusão verificável por qualquer pessoa |
| `aprovacao` | ASS1–ASS12, BL4b | Aprovações k de n com vinculação dinâmica (hash do documento, valor e beneficiário); carência para beneficiário novo; também usada para publicar versões — nenhum programador publica sozinho |
| `pacote` | MTA1–MTA3 | Nome do arquivo, tipo, data e pasta cifrados junto com o conteúdo, num bloco de tamanho padronizado |
| `acesso` | ANO9–ANO14 | Endereços de blocos derivados de semente secreta e lotes de tamanho fixo com cobertura embaralhada: o servidor não liga pedidos entre si nem ao dono |
| `rede` | NET1–NET14 | Três escolhas: Automático (padrão: rota rápida em duas etapas no dia a dia; Tor sozinho nos momentos de identidade, nas conversas blindadas e com ponte quando a rede bloqueia), Sempre Tor e Rápido (sem Tor, para países onde usar Tor chama atenção). Chamadas por retransmissor independente; nunca desce do Tor sozinho; operador da 1ª etapa sorteado entre vários; cabeçalhos uniformes; Bluetooth que muda |
| `notificacao` | NOT1–NOT7 | Cadência fixa de notificações: uma por aparelho por janela, real ou de cobertura, mesmo tamanho, texto genérico |
| `contatos` | CTT1–CTT8 | O nome que você salva, as chaves e as notas do contato num bloco cifrado de tamanho fixo; nome de perfil cifrado com chave entregue só aos contatos |
| `convites` | SPAM1–SPAM6 | Conversa só com convite de uso único e prazo; o servidor guarda só o hash |
| `assinatura` | PQS1–PQS6 | Assinatura híbrida Ed25519 + ML-DSA-87 (FIPS 204; a exigida pela CNSA 2.0): as duas precisam conferir; usada em aprovações, prazos de mensagens temporárias e Escudo de Conversa |
| `senha` | SEG5, ENV3, BA3 | Argon2id (RFC 9106: 64 MiB, 3 passagens, 4 faixas) no PIN de coação e na senha de link |
| `cofre` | COF1–COF8 | Álbuns e nomes de álbuns, notas, compromissos, senhas e ajustes: tudo cifrado, com o tipo do item dentro da cifra e classes de tamanho — por fora, uma senha e uma nota são idênticas |
| `conversa` | HIS1–HIS13 | O que fica e por quanto tempo, para os dois lados: cada pessoa escolhe e vale a combinação mais restritiva; mensagens em 3 dias e não lidas em até 48 h; chamada atendida não deixa registro em nenhum aparelho; chamada não atendida fica 48 h na lista de quem recebeu (quem ligou não apaga esse aviso); nenhuma marca na conversa; conversa vazia some da lista dos dois; apagar tudo dos dois só com o consentimento dos dois; alerta de aparelho novo nunca silenciado |
| `apresentacao` | APR1–APR8 | Como você aparece para cada contato: nome sugerido (a pessoa pode trocar) ou travado (ninguém troca), assinado por você; quando você muda, a versão nova chega a todos; personas com chaves separadas |
| `inspecao` | INS1–INS9 | Todo arquivo conferido antes de abrir ou importar: tipo real pelos primeiros bytes, bloqueio de programas, instaladores, scripts, macros e extensão mentirosa; PDF e Office só como imagem |
| `direto` | DIR1–DIR8, ND2–ND5 | Arquivo de Nimbus para Nimbus sem conversa e sem marca: inspecionado, sem metadados, cifrado para quem recebe, entregue uma vez só pelos retransmissores, até 48 h; NimbusDrop lacrado: por proximidade, só abre com a chave entregue pelo servidor (ND2–ND5) |
| `modos` | MOD1–MOD8 | Padrão de envio por tipo de conteúdo (de fábrica: foto e vídeo em visualização única, voz discreta, texto e arquivo normais) e escolha na hora; a mensagem sai com o mais restritivo entre a conversa e a escolha; nenhum modo passa de 48 h |
| `recuperacao` | REC1–REC6 | Recuperação com 3 a 5 amigos de confiança, k de n; cada parte cifrada só para o seu guardião; parte adulterada ou insuficiente é recusada |
| `liberacao` | LIB1–LIB16 | Mensagem não sai (não se copia, não se guarda, não vai para fora). Exceção estreita: cartão encaminhável — texto ou localização marcados ANTES do envio, prazo obrigatório de até 4 h, só dentro do Nimbus, um salto só. Arquivo só sai com liberação assinada pelo autor (nada, cofre, encaminhar no Nimbus, levar para fora), com prazo opcional de 30 s a 4 h contado da abertura; o item pode sumir ao fim; revogação; quem encaminha só reduz; aviso ao autor; sem marca d’água |
| `corporativo` | COR1–COR24 | Plano corporativo: a empresa suspende o acesso sem ler nada (lista de contas assinada pelos administradores dela, nunca pelo servidor; validade de 15 minutos; relógio que o funcionário não consegue voltar); recuperação só de conta já suspensa, com 2 de 3 administradores e a chave mestra; reativação, suspensão em massa e troca de administradores com várias assinaturas |
| `custodia` | GUA1–GUA8 | Chave mestra dividida entre até 7 guardiões, k de n, sem aritmética criptográfica própria; peça adulterada é detectada |
| `memoria` | RZ3, RZ11, RZ12 | Segredo em memória blindada: zerado, travado na RAM quando possível, nunca impresso em registro ou relatório de falha |
| `chamada` | C88 | Estados da chamada: nenhuma mensagem que chega pela rede liga o microfone; só a ação local de atender |
| `metadados` | MET1–MET5 | Caixa que muda todo dia, índice cego no disco, registros legais selados fora do alcance do operador |
| `ambiente` | AMB1–AMB4 | Diagnóstico local do aparelho; nada sai do aparelho |
| `chaves_do_servidor` | ROT1–ROT4 | Chave do servidor diferente a cada dia, certificada por 2 de 3 raízes offline |
| `prechaves` | FS1–FS5 | Chaves de recebimento de vida curta, assinadas pela identidade e destruídas depois do prazo: envelopes antigos ficam ilegíveis para sempre |
| `autenticacao` | SRV1–SRV6 | Servidor: rotas negadas por padrão, prova por recurso, assinatura que cobre método, caminho, corpo e horário, número de uso único |
| `limites` | SRV7–SRV9 | Servidor: cota por janela, castigo que dobra até 24 h, memória com teto e falha fechada |
| `selo` | SLO1–SLO7 | Selo de liberação: artefato exato, todos os 17 portões aprovados, três chaves diferentes (CI, responsável de segurança e auditor independente); sem ele, nada roda |
| `sentinela` | SEN1–SEN5 | Autoverificação na partida e periódica; qualquer achado trava e o relatório diz onde e como corrigir |
| `trafego` | ANO1–ANO3 | Batida constante de envio: silêncio e conversa ficam iguais para quem observa a rede |
| `protecao_do_usuario` | BLQ1–BLQ4, DEN1–DEN5, APG1–APG4 | Bloquear contato, denunciar por iniciativa do usuário e apagar a conta, sem o servidor ver nada além do que já via |

As conversas 1:1 e em grupo do NimbusChat **não** usam este núcleo para o protocolo de mensagens:
usam a libsignal (PQXDH + Triple Ratchet) e o MLS (RFC 9420).

## Como testar

    ./verificar.sh              # verificação completa em falha fechada: testes, catraca, fuzzing, varredura, auditoria,
                                # mutação (353 defeitos plantados), tempo constante e fuzzing de 2 milhões (≈ 30 min)
    ./verificar.sh --rapido     # só as etapas rápidas; termina em PARCIAL (código 3) e não vale como verificação
    cargo test --locked                              # 725 testes rápidos (≈ 1 min; era 571 nesta linha — desatualizado por várias rodadas, conferido ao vivo e corrigido na Rodada 26, 12/10/2026)
    cargo test --profile verificacao --test fuzzing -- --ignored   # C79: 50 mil entradas por alvo (FUZZ_RODADAS=2000000 no modo completo)
    cargo test --profile verificacao --test timing -- --ignored    # C80: PIN real, de coação e errado no mesmo tempo (≈ 1 min)
    cargo test --test aparelho_real -- --ignored   # prova C76 no aparelho (ver docs/roteiro-aparelho-real.md)
    python3 verificacao/mutantes.py 1 20   # ou em faixas (útil com limite de tempo)

Testado em Rust 1.91 (exigido pela assinatura pós-quântica ML-DSA); versões fixadas no `Cargo.lock`.

## Comece por aqui

1. Leia `GUIA_DO_TIME.md` (o que construir, em que ordem, com quais componentes prontos).
2. Rode o exemplo abaixo.
3. O pipeline pronto está em `.github/workflows/nimbus-ci.yml`; as regras de dependências, em `deny.toml`.

## Exemplo de ponta a ponta

    cargo run --example fluxo_completo

Mostra 33 passos reais do NimbusOne — verificação do aparelho, cadastro sem telefone, PIN de coação,
sessão única, temporizador por conversa e por mensagem, prazo assinado, Escudo de Conversa, histórico no
servidor cego com apagamento criptográfico, envelope pós-quântico, tráfego de cobertura, modulador de voz,
agenda que se apaga sozinha, limpeza de metadados, memória blindada, registro público de
chaves, aprovação de pagamento por 2 de 3 executivos, nomes de arquivo cifrados, pedidos não ligáveis, rota de rede
por tipo de tráfego e notificações em cadência fixa. Cada passo cita o requisito do Manual e tem teste correspondente.

## Como a verificação funciona (quatro camadas automáticas)

1. **Testes de unidade** (665; era 302 nesta linha, desatualizado por várias rodadas — conferido ao vivo, `cargo test --lib`, Rodada 26, 12/10/2026): cada regra de segurança com exemplos.
2. **Testes de propriedades** (17, em `tests/propriedades.rs`; era 15, conferido por contagem direta de `#[test]` no arquivo, Rodada 26): leis que valem para QUALQUER entrada, com
   centenas de casos sorteados — por exemplo, “uma mensagem nunca afrouxa a regra da conversa”.
3. **Testes de robustez** (7, em `tests/robustez.rs`; era 8, mesma conferência da Rodada 26): milhares de entradas aleatórias e adulteradas; nada
   pode derrubar o app.
4. **Teste por mutação** (`verificacao/mutantes.py`): planta 353 defeitos em pontos críticos (era 321 nesta linha, desatualizado — conferido contra `CATRACA.json`/`NUMEROS.json`, Rodada 26) e exige que
   todos sejam pegos. À prova de interrupção: guarda o original antes de plantar e restaura sozinho. Mede se os testes são fortes de verdade. Na revisão v20, 6 defeitos passaram na
   primeira rodada; os testes foram reforçados e hoje os 38 são pegos (número da própria revisão v20, histórico — não o total atual de 353 acima).

## Compilação rigorosa (zero avisos)

    RUSTFLAGS="-D warnings -W unused-results -W missing-debug-implementations -W rust-2018-idioms" cargo build --lib --examples

Tipos que guardam segredos (chaves, sementes, voz, verificadores) mostram só `<oculto>` quando impressos.

## Varredura obrigatória antes de cada entrega

    semgrep --config nimbus-regras.yml .

São 47 regras (era 38 nesta linha — desatualizado desde antes de 11/10/2026, quando a varredura já tinha
crescido sem esta linha ser corrigida; conferido agora por contagem direta de `- id:` em `nimbus-regras.yml`,
idêntico entre a cópia da raiz do pacote e esta do núcleo). O próprio núcleo passa nelas com zero ocorrências
(o exemplo de demonstração fica fora, porque imprime conteúdo de propósito para ensinar).

## Pendências para o auditor

1. `envelope`: conferir o combinador híbrido contra o rascunho IETF X-Wing.
2. `envelope`, suíte FIPS: em produção, trocar por módulo com validação FIPS 140-3.
3. `expiracao`: assinatura híbrida Ed25519 + ML-DSA-87 já implementada (módulo `assinatura`).
4. `integridade`: os sinais são coletados pelas camadas nativas (Swift e Kotlin); o servidor reconfirma
   com a atestação. Ver o Manual, seção DV.
5. `sessoes`: em produção, o registro fica em memória distribuída sem disco, e a derrubada é propagada
   a todos os servidores.
6. `escudo`: a coleta do estado de captura é nativa (iOS: `UIScreen.isCaptured` e `sceneCaptureState`;
   Android: `FLAG_SECURE` e retornos de captura). Envio sugerido a cada 3 s; validade de 10 s.
7. `coacao`: derivação por Argon2id já implementada (módulo `senha`); o auditor confirma os parâmetros.
8. `trafego`: medir o consumo de bateria e de dados da batida constante e definir o intervalo por perfil
   de uso; oferecer modo econômico com o limite declarado ao usuário.
9. `voz`, Nível 2: a inferência da rede neural roda na camada nativa (Core ML no iOS, LiteRT/NNAPI no
   Android); este crate faz a orquestração, o orçamento de latência e a proteção de memória. Fornecer o
   modelo de conversão de voz e medir latência real por aparelho antes de habilitar em chamada. Declarar
   ao usuário que sotaque, vocabulário e ritmo continuam identificando a pessoa.
10. `historico`: a chave-mestra, em produção, fica embrulhada pela chave do chip de segurança do aparelho
    (Secure Enclave ou StrongBox) e as chaves de janela, sincronizadas cifradas entre os aparelhos do dono.
11. `envio`: produção com derivação de senha por Argon2id; o identificador do link deve ser aleatório
    (32 bytes); o servidor apaga o bloco ao esgotar, expirar ou ser revogado; só contas NimbusOne criam links,
    com limites de tamanho e quantidade.
12. `higiene`: PDF, Word, Excel e outros são achatados na camada nativa (páginas renderizadas como imagem)
    antes do envio; sem achatar, o envio é bloqueado.
13. `memoria`: todo conteúdo aberto e toda chave em uso passam por `Segredo`; na edição de computador,
    exigir disco cifrado do sistema, porque a hibernação grava a RAM em disco.
14. `registro`: implementar a prova de consistência (RFC 9162, 2.1.4) e a troca de raízes entre clientes
    (gossip), para impedir que o servidor mostre versões diferentes do registro a pessoas diferentes.
15. `aprovacao`: migrar para assinatura híbrida Ed25519 + ML-DSA-87; chaves dos aprovadores no chip seguro,
    liberadas só por biometria forte do sistema; para assinatura com validade jurídica máxima, integrar
    provedor de assinatura qualificada (ICP-Brasil / eIDAS) em formato PAdES com carimbo de tempo.
## Histórico de revisões

Registro do que mudou em cada versão. Os números de cada item valem para aquela versão; os atuais estão em "Como testar".

16. Revisão final (v19): chave pública X25519 degenerada é recusada já na leitura; envelope com limite de
    64 MiB (acima disso, módulo `envio`); tentativa de login sem credencial não derruba o dono (antes,
    derrubava — era uma porta para ataque de negação de serviço); Escudo conta a validade a partir da
    chegada, não do carimbo do outro lado; voz neural em memória de capacidade fixa (sem realocação que
    deixaria cópia da voz); PIN de coação igual ao real é recusado; quórum de aprovação conta pessoas
    distintas; nenhum tipo com segredo imprime o conteúdo.
17. Revisão v20: módulo `contatos` (o nome salvo no contato era responsabilidade da camada do app;
    agora é do núcleo, cifrado e em tamanho fixo), módulo `convites` (anti-spam sem telefone), apagar a
    conta (`ChavesDoDono::apagar_conta`), testes de propriedades e teste por mutação.
18. Revisão v21: assinatura pós-quântica entregue (ML-DSA-87 em modo híbrido) e aplicada a aprovações, prazos
    e Escudo; Argon2id no PIN de coação e na senha de link (eram pendências “trocar em produção”); cofre
    universal com tipo escondido; nome do arquivo do link dentro da cifra; cabeçalhos da página sem rastro
    de quem enviou; canal próprio de notificações no Android e no computador. Mutação: 47 de 47.
19. Revisão v22: plano corporativo (módulos `corporativo` e `custodia`). Encontrado e corrigido no próprio
    desenho: o travamento offline dependia do relógio do celular (o funcionário poderia voltar a hora); agora
    usa relógio confiável. Encontrado e corrigido na ferramenta de mutação: ao restaurar o original, a data
    antiga do arquivo fazia o compilador manter o programa com o defeito plantado; agora a data é atualizada
    e uma rodada interrompida é restaurada automaticamente. Mutação: 62 de 62.
20. Revisão v23: módulo `conversa` — histórico que some para os dois. Duas exceções de segurança de
    propósito: o alerta de troca de aparelho do contato nunca é silenciado (proteção contra aparelho fantasma),
    e apagar tudo dos dois aparelhos na hora exige o consentimento dos dois (sem isso, alguém poderia apagar
    à força a prova de um assédio no aparelho da vítima). Mutação: 72 de 72.
21. Revisão v24: mensagem não lida some em até 48 horas (prazo assinado no bilhete; o remetente pode pedir
    menos, nunca mais); chamada atendida some na hora, não atendida fica 48 h para quem recebeu; módulo
    `salvamento` (substituído na v25 pelo módulo `liberacao`). Um teste antigo garantia que mensagem
    nunca aberta não expirava — foi atualizado para a regra nova. Mutação: 86 de 86.
22. Revisão v25 (auditoria completa): a política antiga permitia copiar, encaminhar e salvar MENSAGENS quando
    a conversa permitia, e citar vinha ligado — contradizia a regra “mensagem nunca sai”. Agora a política só
    trata prazos, visualização única, citar (desligado) e Conversa Blindada; tudo o que sai da conversa é do
    módulo `liberacao`, com autorização assinada pelo autor. Conversões sem `unwrap` no código de produção;
    teste de remetente malicioso (conteúdo fora do padrão é recusado sem derrubar o app). Mutação completa em
    todos os módulos: 92 de 92.
23. Revisão v26: liberação com prazo (30 s a 4 h, contado da abertura; o item pode sumir ao fim); cartão
    encaminhável para texto e localização (só no envio, até 4 h, só no Nimbus, um salto); um salto para tudo
    o que chega encaminhado; limite de tentativas de PIN (espera que dobra; no 10º erro, apaga as chaves);
    marca d’água descartada por ser rastro. Mutação: 103 de 103.
24. Revisão v27: nenhuma mensagem vive mais de 48 horas, lida ou não (`expiracao::VIDA_MAXIMA`); temporizadores
    de 1 minuto a 48 horas no envio; módulos `apresentacao`, `inspecao` e `direto`; palavras de confirmação nos
    convites; link com download só se o remetente permitir, prazo até 48 h e limpeza do navegador ao encerrar.
    Com o teto de 48 h valendo para tudo, a trava antiga de “não lida no máximo 48 h” ficou redundante: o
    defeito plantado que a testava virou equivalente e foi removido; a redundância saiu do código. Contagem de
    módulos corrigida (a v25 e a v26 diziam 30; eram 29; agora são 32). Mutação: 122 de 122.
25. Revisão v28: módulos `modos` (padrão de envio por tipo e escolha na hora) e `recuperacao` (amigos de
    confiança); integridade com atestação de hardware (inicialização verificada, origem da atestação,
    revogação), aparelho sem senha, ganchos e emulador; acessibilidade de terceiros contida pelo sistema
    (Android 14+) só amarela, para não tirar do app quem usa gerenciador de senhas; desafio de uso único
    contra atestação reenviada; `verificar.sh` para verificar tudo em um comando. Mutação: 143 de 143.
26. Revisão v29: auditoria de dependências contra a base RustSec (71 pacotes, nenhuma vulnerabilidade em
    aberto), agora dentro do verificar.sh; testes de autoverificação da criptografia (tests/criptografia.rs:
    bit trocado, dado associado, chave errada, envelopes distintos, assinatura híbrida); testes prontos para
    o programador rodar em aparelho real (tests/aparelho_real.rs, ignorados por padrão) e de replay de
    atestação (tests/servidor_atestacao.rs); roteiro em docs/roteiro-aparelho-real.md. Regra de varredura de
    log ajustada para não pegar prints de arquivos de teste (que não vão para produção). Nessa versão,
    334 testes automáticos; 143 de 143 na mutação.
27. Revisão v30 (auditoria adversarial): varredura de todo o código de produção em busca de pânicos, estouros
    de índice, comparações fora de tempo constante, aleatoriedade fraca e segredos sem apagar. Resultado:
    o núcleo já estava sólido (todo leitor de dados externos checa limites; PIN e senhas usam ct_eq; tudo é
    OsRng; todo segredo é Zeroizing). O que faltava eram PROVAS: fuzzing dos leitores com verificação de
    estouro ligada (tests/fuzzing.rs, 50 mil entradas por alvo e depois 2 milhões, prova C79, ambos no verificar.sh) e prova de tempo
    constante do PIN de coação (tests/timing.rs). 355 testes rápidos; 143 de 143 na mutação.
28. Revisão v31: o teste de tempo constante fazia 9.500 verificações de PIN com Argon2id de 64 MiB e não era
    marcado como pesado; o cargo test (primeiro passo do verificar.sh e da integração contínua) o rodava em
    modo de depuração e parecia travar por horas. Agora há um único teste C80 (tests/timing.rs) com os quatro
    casos — PIN real, de coação, errado no primeiro e no último dígito —, 15 medições cada, cerca de 6 s. Ele e
    o fuzzing (C79) ficam fora do cargo test do dia a dia e rodam no perfil verificacao (otimizado, com
    verificação de estouro aritmético e asserções ligadas). Versão do Rust alinhada em 1.91.1 com a integração
    contínua. Resultado desta versão: 334 testes rápidos; fuzzing de 2 milhões de entradas por alvo sem pânico;
    tempo constante com diferença máxima de 5%; 143 de 143 na mutação.
29. Revisão v34: a mutação passou a ter 151 defeitos plantados e os 151 foram pegos (rodada inteira, em
    faixas). As tabelas de módulos do lib.rs e deste LEIAME não listavam `album` e `panico` — incluídos (36
    módulos). O verificar.sh agora restaura sozinho arquivos deixados por uma mutação interrompida e avisa
    quando há pouco disco antes do modo completo: uma mutação interrompida com o disco cheio chegou a
    corromper a compilação e fazer um teste falhar sem motivo.
30. Revisão v37: computador, hora confirmada e NimbusDrop lacrado. (a) `integridade::avaliar` ignorava a
    plataforma: no computador, sem App Attest (a Apple documenta que não está disponível em Mac), o aparelho
    ficaria sempre vermelho ou seria dado como verde sem prova. Agora `tem_atestacao_do_fabricante` limita
    computador e web ao amarelo, e o computador exige chave no chip (TPM 2.0 ou Secure Enclave) e disco
    cifrado — sem isso, vermelho. (b) `expiracao::hora_confiavel` e `pode_exibir`: todo prazo usa a última hora
    do servidor mais o relógio monotônico; sem confirmação há mais de 15 minutos, nada com prazo aparece.
    (c) `direto::lacrar`, `pode_abrir_lacrado` e `abrir_lacrado`: arquivo por proximidade em duas camadas, que
    só abre com a chave entregue pelo servidor, hora confirmada, dentro do prazo e em aparelho verde
    (`Permissoes::receber_por_proximidade`). (d) Regra de varredura `nimbus-sem-rede-mesh` (33ª). (e) Dois avisos
    de compilação dos testes (variante `Eu_desligo`, hoje `EuDesligo`, e uma importação sem uso) corrigidos.
    Resultado: 448 testes rápidos; 274 defeitos plantados, todos exigidos pegos; varredura e auditoria de dependências
    sem ocorrências; tempo constante aprovado.
31. Revisão v37, segunda rodada: (a) a hora do servidor entrava em `hora_confiavel` sem autenticação; quem
    controlasse a rede poderia repetir uma hora antiga e esticar todos os prazos. Agora `expiracao::emitir_hora`
    assina a hora junto com um desafio de uso único do aparelho e `expiracao::conferir_hora` recusa resposta de
    outro desafio, assinatura inválida e hora anterior à última aceita. (b) O prazo do NimbusDrop viajava fora do
    dado autenticado; agora `valida_ate` entra no AAD das duas camadas e alterá-lo invalida o lacre. 387 testes
    rápidos; 168 de 168 defeitos plantados pegos.
32. Revisão v37, terceira rodada (sistemática: todo campo que viaja pela rede e toda função que recebe a hora).
    (a) As esperas entre erros de PIN usavam a hora da ponte: adiantar o relógio do sistema pulava as esperas;
    no modo Tolerante, que nunca apaga, isso abria a força bruta. Agora `coacao::RelogioDeEspera` só soma o
    tempo real do relógio monotônico; reiniciar não soma nada. (b) O prazo da remessa direta (`direto::preparar`)
    ficava fora do AAD, a mesma classe de falha do lacre; agora está dentro. 448 testes rápidos; 255 de 255
    defeitos plantados pegos.
33. Revisão v37, quarta rodada (linha a linha de `envelope`, `assinatura`, `direto`, `registro`, `custodia`,
    `convites`, `senha`; clippy com regras de segurança; todas as conversões de tamanho conferidas). (a) Envio
    direto sem autenticação do remetente: `direto::preparar_assinado` e `direto::abrir_assinado` (a assinatura
    cobre destinatário, id e prazo). (b) `registro::prova_de_consistencia` e `registro::verificar_consistencia`
    (RFC 9162, 2.1.4), testadas em todos os pares de tamanho até 40. (c) `envelope::ChavePublica::de_bytes` faz a
    verificação da FIPS 203 (7.2). (d) `overflow-checks = true` no perfil de produção. Dois defeitos plantados
    sobreviveram na primeira passada e revelaram testes fracos, reforçados. 448 testes rápidos; 255 de 255.
34. Revisão v37, quinta rodada (engenharia social): `convites::opcoes_para_confirmar` e `confirmacao_correta`
    (sem botão "são iguais"); `sessoes::pode_vincular` e `palavras_do_vinculo` (vínculo só pela tela do app;
    golpe do QR) e uma sessão por aparelho registrado (o computador vinculado derrubava o celular);
    `apresentacao::nome_permitido`, `parecido_com`, `mudanca_de_nome` e `subtitulo`; `recuperacao` com
    conferência ao vivo e 3 dias de espera (`pode_concluir`); `corporativo::CARENCIA_DA_RECUPERACAO` (24 h).
    448 testes rápidos; 274 defeitos plantados, todos exigidos pegos.
35. Revisão v37, sexta rodada (atacante com IA e defesa em profundidade): módulo `prechaves` — chaves de
    recebimento de vida curta, assinadas pela identidade e destruídas 48 h depois do prazo, para dar sigilo
    futuro ao que usa `envelope` fora da libsignal (envio direto, NimbusDrop); `assinatura` passa a usar o
    ML-DSA-87 na variante hedged da FIPS 204. 448 testes rápidos; 47 módulos; 255 de 274 defeitos plantados.
36. Revisão v37, sétima rodada (back-end): módulos `autenticacao` (rotas negadas por padrão, prova por recurso,
    assinatura que cobre método, caminho, corpo e horário, número de uso único) e `limites` (cota, castigo que
    dobra até 24 h, memória com teto). O banco do servidor, com RLS forçada, está em `servidor/banco/` dos kits,
    testado num PostgreSQL 16 real. 448 testes rápidos; 47 módulos; 255 de 274 defeitos plantados.
37. Revisão v37, oitava rodada: limite de uso obrigatório em toda rota de `autenticacao`, aplicado antes da
    verificação de assinatura; 5 regras de varredura novas (segredo no código, IA externa, senha em texto,
    variável de ambiente fora do cofre, SQL montado com texto) — 38 no total; `#![deny(unsafe_code)]`;
    `verificacao/catraca.py` + `CATRACA.json` (nenhuma proteção a menos); `.gitignore`, `.gitleaks.toml`,
    pre-commit, `SECURITY.md`, CODEOWNERS; ações do CI fixadas pelo hash. 448 testes; 255 de 255.
38. Revisão v37, nona rodada: módulo `sentinela` (invariantes de segurança e autotestes criptográficos na partida
    e periódicos; qualquer achado trava e o relatório diz onde e como corrigir), testado com cada invariante
    violada; `ROTAS_PUBLICAS_ANALISADAS.json` (a catraca reprova rota sem prova de titular sem análise);
    proteção da branch como código e vigia diário; certificado de procedência no CI. 448 testes; 47 módulos;
    255 de 274 defeitos plantados.
39. Revisão v37, décima rodada: módulo `selo` (artefato exato, 16 portões aprovados, duas chaves diferentes;
    versão mínima) e `sentinela::autoverificar_com_selo` — sem selo válido, nada funciona;
    `assinatura::de_semente` para os assinantes; `examples/selo.rs` (gerar-semente, publica, montar, assinar,
    conferir). 448 testes; 47 módulos; 255 de 274 defeitos plantados.
40. Revisão v37, décima primeira rodada: `selo::MINIMO_DE_ASSINATURAS` = 3 (CI, responsável de segurança e
    auditor independente) e portão `administracao_dupla`; `renovate.json` (sem fusão automática, espera de 14
    dias, 30 para criptografia). 448 testes; 255 de 274 defeitos plantados.
41. Revisão v37, final: módulo `metadados` (caixa que muda todo dia, índice cego com chave destruída em
    48 h, registros legais selados para chave fora do servidor); mutação completa de todos os defeitos
    plantados rodada de uma vez. 431 testes rápidos; 43 módulos; 242 defeitos plantados.
42. Revisão v37, fechamento: `autenticacao::resposta_publica`, `cabecalhos_da_api` e `origem_permitida`, e a rota
    `/v1/envio/abrir` no inventário; módulos `ambiente` (diagnóstico local) e `chaves_do_servidor` (chave do
    dia com 2 de 3 raízes); `metadados::ChavesDeIndice::selar_no_disco` (apagamento criptográfico). 448 testes
    rápidos; 47 módulos; 274 defeitos plantados.
43. Ataque vermelho: `apresentacao` recusa letras de imitação e mistura de alfabetos; `verificacao/catraca.py`
    ignora comentários; `metadados` sem estouro com época extrema; `tests/ataque_vermelho.rs` reproduz os
    ataques. 448 testes rápidos; 274 defeitos plantados.
44. Ataque vermelho, segunda onda: o porteiro sempre confere a assinatura (chave fictícia quando o titular não
    existe), para o tempo da recusa não revelar se uma caixa ou aparelho existe; medição permanente em
    `tests/tempo_de_recusa.rs` (rodada pelo `./verificar.sh`). 449 testes rápidos.
45. Revisão vigilante (03/10/2026): a catraca deixava passar quatro formas de liberar `unsafe` e `unsafe` em teste de
    integração (agora pega; `testar_catraca.py`, 13 afrouxamentos plantados); o auditor de dependências ignorava 13 versões
    duplicadas, pulava aviso que só citava "withdrawn" e aprovava sem base (agora confere as 84 versões, falha fechada;
    `testar_auditoria.py`); `verificar.sh` em falha fechada (`testar_verificar.py`); defeito plantado que não compila
    deixa de contar como pego; regra `nimbus-unsafe-fora-da-memoria` (39 regras). Requisitos do selo renumerados
    SLO1–SLO7 e do pacote MTA1–MTA3 (os antigos identificadores colidiam com outros requisitos).
46. Revisão vigilante, mutação refeita (04/10/2026): 2 defeitos plantados de `autenticacao` (nº 205 e 215) apontavam
    para linhas reescritas na segunda onda do ataque vermelho; reescritos e pegos. 255 de 255, medido de verdade.
    `mutantes.py --conferir` confere em segundos, a cada verificação, que todo trecho existe exatamente uma vez.
