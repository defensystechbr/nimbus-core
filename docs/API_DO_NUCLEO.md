# API pública do núcleo — inventário para a ponte nativa

*Gerado por máquina (`gerar_api.py`) a partir do código. Não edite à mão: rode o script de novo depois de qualquer mudança no núcleo.*

**428 funções públicas em 58 módulos.**

## `acesso`

Endereços derivados e lotes de tamanho fixo com cobertura embaralhada: o servidor não liga pedidos entre si

**Tipos:** `Endereco` (type), `Semente` (struct), `ErroDeLote` (enum), `Lote` (struct)

**Funções:**

```rust
pub fn nova() -> Self
pub fn de_bytes(b: [u8; 32]) -> Self
pub fn endereco(semente: &Semente, contexto: &str, indice: u64) -> Endereco
pub fn montar_lotes(reais: &[Endereco], tamanho: usize) -> Result<Vec<Lote>, ErroDeLote>
```

## `album`

Álbuns e coleções automáticas: cada foto entra sozinha na coleção certa por regra (contato de origem, marcação ou padrão); tudo cifrado no cofre, inclusive o nome da coleção, a legenda e a data

**Tipos:** `Colecao` (struct), `Origem` (struct), `Regra` (enum), `Regras` (struct)

**Funções:**

```rust
pub fn escolher(&self, origem: &Origem) -> Option<u32>
pub fn contar_por_colecao(colecoes_das_midias: &[u32]) -> BTreeMap<u32, usize>
```

## `ambiente`

Diagnóstico local do aparelho (bloqueio de tela, patch, teclado, opções de desenvolvedor, fontes desconhecidas); nada sai do aparelho

**Tipos:** `SinaisLocais` (struct), `Aviso` (struct), `Diagnostico` (struct)

**Constantes:** `DIAS_MAXIMOS_SEM_PATCH`

**Funções:**

```rust
pub fn avaliar(s: &SinaisLocais) -> Diagnostico
pub fn combinar(integridade: Nivel, d: &Diagnostico) -> Nivel
```

## `apresentacao`

Como você aparece para cada contato: nome sugerido ou travado, assinado; mudança chega a todos; personas com chaves separadas, que ninguém liga entre si

**Tipos:** `ErroDeApresentacao` (enum), `Cartao` (struct), `Persona` (struct)

**Constantes:** `TAM_MAXIMO_DO_NOME`, `TERMOS_RESERVADOS`

**Funções:**

```rust
pub fn forma_de_comparacao(nome: &str) -> String
pub fn nome_permitido(nome: &str) -> Result<(), ErroDeApresentacao>
pub fn parecido_com<'a>(nome: &str, outros: &[&'a str]) -> Option<&'a str>
pub fn mudanca_de_nome(atual: Option<&Cartao>, novo: &Cartao) -> Option<(String, String)>
pub fn emitir(chave: &ChaveDeAssinatura, nome: &str, nome_travado: bool, versao: u64) -> Result<Cartao, ErroDeApresentacao>
pub fn conferir(chave_do_contato: &ChaveDeVerificacao, c: &Cartao) -> bool
pub fn atualizar(chave_do_contato: &ChaveDeVerificacao, atual: Option<Cartao>, novo: Cartao) -> Option<Cartao>
pub fn nome_exibido<'a>(cartao: Option<&'a Cartao>, nome_que_salvei: Option<&'a str>) -> Option<&'a str>
pub fn subtitulo<'a>(cartao: Option<&'a Cartao>, nome_que_salvei: Option<&'a str>) -> Option<&'a str>
pub fn pode_editar_nome(cartao: Option<&Cartao>) -> bool
pub fn nova(apelido: &str) -> Self
pub fn verificacao(&self) -> ChaveDeVerificacao
pub fn cartao(&self, nome: &str, nome_travado: bool, versao: u64) -> Result<Cartao, ErroDeApresentacao>
```

## `aprovacao`

Assinaturas e aprovações k de n com vinculação dinâmica: documentos, pagamentos e publicação de versões (nenhum programador publica sozinho)

**Tipos:** `TipoDePedido` (enum), `Pedido` (struct), `Aprovacao` (struct), `Resultado` (enum)

**Funções:**

```rust
pub fn hash_do_conteudo(conteudo: &[u8]) -> [u8; 32]
pub fn aprovar(chave: &ChaveDeAssinatura, p: &Pedido) -> Aprovacao
pub fn avaliar(p: &Pedido, aprovacoes: &[Aprovacao], autorizados: &[ChaveDeVerificacao], agora: u64) -> Resultado
pub fn carencia_em_segundos(beneficiario_ja_usado: bool, valor_centavos: u64, limite_centavos: u64, espera: u64) -> u64
```

## `assinatura`

Assinatura híbrida Ed25519 + ML-DSA-87 (FIPS 204, CNSA 2.0): as duas precisam conferir; contexto por uso

**Tipos:** `ChaveDeAssinatura` (struct), `ChaveDeVerificacao` (struct), `AssinaturaHibrida` (struct), `ErroDeAssinatura` (enum)

**Constantes:** `TAM_VERIFICACAO_PQ`, `TAM_ASSINATURA_PQ`

**Funções:**

```rust
pub fn gerar() -> ChaveDeAssinatura
pub fn de_semente(semente: &[u8; 64]) -> ChaveDeAssinatura
pub fn verificacao(&self) -> ChaveDeVerificacao
pub fn assinar(&self, contexto: &[u8], mensagem: &[u8]) -> Result<AssinaturaHibrida, ErroDeAssinatura>
pub fn verificar(&self, contexto: &[u8], mensagem: &[u8], a: &AssinaturaHibrida) -> bool
pub fn para_bytes(&self) -> Vec<u8>
pub fn de_bytes(b: &[u8]) -> Result<Self, ErroDeAssinatura>
pub fn impressao_digital(&self) -> [u8; 32]
pub fn partes(&self) -> ([u8; 64], &[u8])
pub fn de_partes(ed: [u8; 64], pq: Vec<u8>) -> Self
```

## `autenticacao`

Servidor: rotas negadas por padrão, prova por recurso (não por pessoa), assinatura que cobre método, caminho, corpo e horário, número de uso único

**Tipos:** `Exigencia` (enum), `TipoDeTitular` (enum), `Rota` (struct), `Prova` (struct), `Recusa` (enum), `Autorizado` (enum), `Porteiro` (struct)

**Constantes:** `JANELA_SEGUNDOS`, `MAXIMO_DE_NUMEROS_GUARDADOS`, `LIMITE_MAXIMO_POR_MINUTO`, `CASTIGO_INICIAL`, `CHAVES_POR_ROTA`, `TETO_TITULARES_DESCONHECIDOS_POR_MINUTO`, `ROTAS`

**Funções:**

```rust
pub fn rota(metodo: &str, caminho: &str) -> Option<&'static Rota>
pub fn provar(chave: &ChaveDeAssinatura, metodo: &str, caminho: &str, corpo: &[u8], agora: u64) -> Prova
pub fn registrar(&mut self, chave: ChaveDeVerificacao, tipo: TipoDeTitular) -> [u8; 32]
pub fn revogar(&mut self, titular: &[u8; 32])
pub fn autorizar(&mut self, metodo: &str, caminho: &str, corpo: &[u8], prova: Option<&Prova>, ficha_valida: bool, chave_de_limite: &[u8], agora: u64) -> Result<Autorizado, Recusa>
pub fn resposta_publica(r: &Result<Autorizado, Recusa>) -> (u16, &'static str)
pub fn cabecalhos_da_api() -> Vec<(&'static str, &'static str)>
pub fn origem_permitida(origem: Option<&str>) -> bool
```

## `autenticidade`

Nunca ser imitado: certificado do app conferido em tempo constante; endereços parecidos com os oficiais (caso Privnote) acusados antes de abrir

**Tipos:** `Motivo` (enum)

**Constantes:** `MAXIMO_DE_TEXTO`, `MAXIMO_DE_ENDERECOS`, `MAXIMO_DO_DOMINIO`

**Funções:**

```rust
pub fn assinatura_do_app_confere(resumo: &[u8; 32], fixados: &[[u8; 32]]) -> bool
pub fn imitacao(dominio: &str, oficiais: &[&str]) -> Option<Motivo>
pub fn extrair_dominios(texto: &str) -> Vec<(String, bool)>
pub fn avaliar_mensagem(texto: &str, oficiais: &[&str]) -> Vec<(String, Motivo)>
```

## `chamada`

Estados da chamada: nenhuma mensagem que chega pela rede liga o microfone; só a ação local de atender (prova C88)

**Tipos:** `Estado` (enum), `Fim` (enum), `Evento` (enum), `Chamada` (struct)

**Constantes:** `SEGUNDOS_DE_TOQUE`

**Funções:**

```rust
pub fn recebida() -> Self
pub fn estado(&self) -> Estado
pub fn microfone_ligado(&self) -> bool
pub fn aplicar(&mut self, evento: Evento) -> Estado
```

## `chamada_em_grupo`

Chamada em grupo (v37, terceira rodada) como malha de chamadas 1:1 independentes, reaproveitando `chamada` sem alterá-lo: cada perna exige o próprio atendimento local, nenhuma ativa as outras; teto de participantes pela banda da malha

**Tipos:** `IdParticipante` (type), `ErroDeChamadaEmGrupo` (enum), `ChamadaEmGrupo` (struct)

**Constantes:** `MAX_PARTICIPANTES`

**Funções:**

```rust
pub fn nova() -> Self
pub fn convidar(&mut self, quem: IdParticipante) -> Result<(), ErroDeChamadaEmGrupo>
pub fn aplicar(&mut self, quem: &IdParticipante, evento: Evento) -> Result<Estado, ErroDeChamadaEmGrupo>
pub fn participantes_com_audio_ligado(&self) -> Vec<IdParticipante>
pub fn alguma_perna_ativa(&self) -> bool
pub fn tamanho(&self) -> usize
pub fn remover(&mut self, quem: &IdParticipante) -> Result<(), ErroDeChamadaEmGrupo>
```

## `chaves_do_servidor`

Chave de assinatura do servidor diferente a cada dia, certificada por 2 de 3 raízes offline; a de ontem já não vale

**Tipos:** `CertificadoDoDia` (struct), `RecusaDoCertificado` (enum)

**Constantes:** `DIA`, `FOLGA`, `MINIMO_DE_RAIZES`

**Funções:**

```rust
pub fn certificar(raiz: &ChaveDeAssinatura, dia: u64, chave_do_dia: &ChaveDeVerificacao) -> ([u8; 32], AssinaturaHibrida)
pub fn conferir(c: &CertificadoDoDia, raizes_fixadas: &[ChaveDeVerificacao], agora: u64) -> Result<ChaveDeVerificacao, RecusaDoCertificado>
```

## `checkup`



**Tipos:** `Config` (struct), `Severidade` (enum), `Item` (enum)

**Constantes:** `MINIMO_RECOMENDADO_DE_GUARDIOES`

**Funções:**

```rust
pub fn severidade(self) -> Severidade
pub fn avaliar(c: &Config) -> Vec<Item>
pub fn vazia() -> Self
```

## `coacao`

PIN de coação, cofre-isca e negação plausível, em tempo constante

**Tipos:** `Verificador` (struct), `ErroDeCoacao` (enum), `Abertura` (enum), `Politica` (struct), `LimiteParaApagar` (enum), `TetoDiurno` (struct), `Tentativas` (struct), `RelogioDeEspera` (struct), `AposErro` (enum)

**Constantes:** `TETO_DIURNO_MINIMO_HORAS`, `TETO_DIURNO_MAXIMO_HORAS`, `PISO_NOTURNO_SEGUNDOS`, `ERROS_SEM_ESPERA`

**Funções:**

```rust
pub fn novo(sal: [u8; 16], segredo_real: &[u8], segredo_de_coacao: &[u8]) -> Result<Self, ErroDeCoacao>
pub fn avaliar(&self, digitado: &[u8]) -> Abertura
pub fn tentativas(self) -> u32
pub fn de_horas(horas: u8) -> Self
pub fn em_segundos(self) -> u64
pub fn equilibrada() -> Self
pub fn rigida() -> Self
pub fn tolerante() -> Self
pub fn espera_crescente_sem_apagar(apos_tentativas: u32, espera_inicial_segundos: u64) -> Self
pub fn espera_crescente_com_teto_diurno(apos_tentativas: u32, espera_inicial_segundos: u64, teto: TetoDiurno) -> Self
pub fn apagar_apos_escolha_do_usuario(limite: LimiteParaApagar) -> Self
pub fn valida(self) -> Self
pub fn avancar(&mut self, inicializacao: u64, monotonico: u64) -> u64
pub fn pode_tentar(&self, pol: &Politica, agora: u64) -> bool
pub fn errou(&mut self, pol: &Politica, agora: u64) -> AposErro
pub fn acertou(&mut self)
pub fn erros_consecutivos(&self) -> u32
pub fn espera_apos_reinicio_noturno(&mut self, agora: u64)
```

## `cobranca`



**Tipos:** `ChavePublicaDeEmissao` (type), `PlanoPago` (enum), `Periodo` (struct), `ErroDeCobranca` (enum), `PagamentoConfirmado` (struct), `Ficha` (struct), `EmissorDeFichas` (struct)

**Funções:**

```rust
pub fn valido(&self) -> bool
pub fn proximo(&self) -> Periodo
pub fn novo() -> Self
pub fn gerar_chave_do_periodo(&mut self, plano: PlanoPago, periodo: Periodo) -> Result<(), ErroDeCobranca>
pub fn chave_publica(&self, plano: PlanoPago, periodo: Periodo) -> Option<&ChavePublicaDeEmissao>
pub fn assinar_cego( &mut self, pagamento: &PagamentoConfirmado, msg_cega: &blind_rsa_signatures::BlindMessage, ) -> Result<BlindSignature, ErroDeCobranca>
pub fn preparar_pedido_de_ficha(pk: &ChavePublicaDeEmissao) -> Result<(Vec<u8>, BlindingResult), ErroDeCobranca>
pub fn finalizar_ficha( pk: &ChavePublicaDeEmissao, assinatura_cega: &BlindSignature, resultado_de_cegar: &BlindingResult, token: Vec<u8>, ) -> Result<Ficha, ErroDeCobranca>
pub fn ficha_e_valida(pk: &ChavePublicaDeEmissao, ficha: &Ficha) -> bool
pub fn chave_publica_para_navegador(pk: &ChavePublicaDeEmissao) -> Result<Vec<u8>, ErroDeCobranca>
```

## `codec_palavras`

BIP-39 para a frase de recuperação pessoal: 32 bytes ↔ 24 palavras em português, wordlist oficial do padrão (não fabricada), checksum de 8 bits que detecta erro de dedo; aditivo, nunca substitui hex/QR — zero criptografia nova, só codificação determinística

**Tipos:** `ErroDeCodec` (enum)

**Constantes:** `TAM_SEGREDO`, `NUM_PALAVRAS`

**Funções:**

```rust
pub fn codificar(segredo: &[u8; TAM_SEGREDO]) -> [&'static str; NUM_PALAVRAS]
pub fn decodificar(palavras: &[&str]) -> Result<[u8; TAM_SEGREDO], ErroDeCodec>
```

## `cofre`

Álbuns, notas, compromissos, senhas e ajustes cifrados com o tipo dentro da cifra e classes de tamanho

**Tipos:** `Tipo` (enum), `Item` (struct), `ErroDoCofre` (enum), `PrazoDeRetencaoDoCofre` (struct), `ErroDeRetencao` (enum), `TetoDeInatividadeDoCofre` (struct), `PoliticaDeTrancaDoCofre` (struct)

**Constantes:** `CLASSES`, `PRAZO_MINIMO_DIAS`, `PRAZO_MAXIMO_DIAS`, `TETO_DE_INATIVIDADE_MINIMO_SEGUNDOS`, `TETO_DE_INATIVIDADE_MAXIMO_SEGUNDOS`

**Funções:**

```rust
pub fn classe_para(n: usize) -> Option<usize>
pub fn selar(dono: &ChavePublica, item: &Item) -> Result<Vec<u8>, ErroDoCofre>
pub fn de_dias(dias: u64) -> Self
pub fn em_dias(self) -> u64
pub fn em_segundos(self) -> u64
pub fn dentro_do_periodo_de_aviso(expira_em: u64, agora: u64, duracao_do_aviso: u64) -> bool
pub fn guardar_no_historico( servidor: &mut historico::ServidorCego, chaves: &mut historico::ChavesDoDono, dono: &ChavePublica, item: &Item, prazo: PrazoDeRetencaoDoCofre, ) -> Result<historico::IdDeBloco, ErroDeRetencao>
pub fn abrir_do_historico( servidor: &historico::ServidorCego, chaves: &historico::ChavesDoDono, sec: &ChaveSecreta, publ: &ChavePublica, id: &historico::IdDeBloco, ) -> Result<Item, ErroDeRetencao>
pub fn abrir(sec: &ChaveSecreta, publ: &ChavePublica, bloco: &[u8]) -> Result<Item, ErroDoCofre>
pub fn de_segundos(s: u32) -> Self
pub fn em_segundos(self) -> u32
pub fn deve_trancar( politica: &PoliticaDeTrancaDoCofre, ultimo_uso_em: u64, agora: u64, nivel: crate::integridade::Nivel, ) -> bool
```

## `comando_remoto`

Comando de bloquear/apagar, por dois caminhos independentes, para alimentar `panico::Gatilho::ComandoRemoto`: (a) assinado por outro aparelho já confiado na conta — exige um segundo aparelho/computador já vinculado; (b) assinado por `k` de `n` guardiões (mesmo quórum de `recuperacao::preparar_tres_trilhas`), nunca 1 sozinho — cobre o caso sem nenhum segundo aparelho à mão. Os dois vinculados ao aparelho-alvo certo e protegidos contra repetição por contador crescente — zero criptografia nova, mesma assinatura híbrida de sempre, contextos de domínio separados entre os dois caminhos e entre isto e a recuperação de conta

**Tipos:** `IdAparelho` (type), `ComandoDeProtecao` (struct), `Recusa` (enum), `AssinaturaDeGuardiao` (struct), `ComandoPorGuardioes` (struct), `RecusaDeGuardioes` (enum)

**Funções:**

```rust
pub fn emitir(emissor: &ChaveDeAssinatura, aparelho_alvo: IdAparelho, reacao: Reacao, contador: u64) -> ComandoDeProtecao
pub fn validar( chave_do_emissor: &ChaveDeVerificacao, cmd: &ComandoDeProtecao, aparelho_deste_dispositivo: &IdAparelho, ultimo_contador_aceito: u64, ) -> Result<Reacao, Recusa>
pub fn guardiao_assina(guardiao: &ChaveDeAssinatura, aparelho_alvo: IdAparelho, reacao: Reacao, contador: u64) -> AssinaturaDeGuardiao
pub fn juntar_assinaturas(aparelho_alvo: IdAparelho, reacao: Reacao, contador: u64, assinaturas: Vec<AssinaturaDeGuardiao>) -> ComandoPorGuardioes
pub fn validar_por_guardioes( chaves_dos_guardioes: &[([u8; 32], ChaveDeVerificacao)], cmd: &ComandoPorGuardioes, aparelho_deste_dispositivo: &IdAparelho, ultimo_contador_aceito: u64, k: u8, ) -> Result<Reacao, RecusaDeGuardioes>
```

## `contatos`

O nome que você salva, as chaves e as notas do contato num bloco cifrado de tamanho fixo; nome de perfil cifrado com chave entregue só a contatos

**Tipos:** `Contato` (struct), `ErroDeContato` (enum), `AvisoDeIdentidade` (enum), `ErroDeNomesProibidos` (enum), `ChaveDePerfil` (struct)

**Constantes:** `TAM_FIXO_DO_CONTATO`, `TAM_FIXO_DO_PERFIL`, `AVISO_DE_CONTATO_INATIVO_SEGUNDOS`, `CARENCIA_APOS_AVISO_SEGUNDOS`, `MIN_NOMES_PROIBIDOS`, `MAX_NOMES_PROIBIDOS`

**Funções:**

```rust
pub fn selar_contato(dono: &ChavePublica, c: &Contato) -> Result<Vec<u8>, ErroDeContato>
pub fn abrir_contato(sec: &ChaveSecreta, publ: &ChavePublica, bloco: &[u8]) -> Result<Contato, ErroDeContato>
pub fn deve_avisar_inatividade(ultima_interacao: u64, agora: u64) -> bool
pub fn deve_remover_por_inatividade(ultima_interacao: u64, avisado_em: Option<u64>, agora: u64) -> bool
pub fn avaliar_identidade(conhecida: Option<[u8; 32]>, desta_mensagem: [u8; 32]) -> AvisoDeIdentidade
pub fn deve_mostrar_aviso_de_identidade(a: AvisoDeIdentidade) -> bool
pub fn normalizar_nome(nome: &str) -> String
pub fn validar_nomes_proibidos(nomes: &[String]) -> Result<Vec<String>, ErroDeNomesProibidos>
pub fn nome_permitido(proibidos_normalizados: &[String], tentativa: &str) -> bool
pub fn nova() -> Self
pub fn de_bytes(b: [u8; 32]) -> Self
pub fn exportar(&self) -> Zeroizing<[u8; 32]>
pub fn cifrar_nome_de_perfil(k: &ChaveDePerfil, nome: &str) -> Result<Vec<u8>, ErroDeContato>
pub fn decifrar_nome_de_perfil(k: &ChaveDePerfil, bloco: &[u8]) -> Result<String, ErroDeContato>
```

## `conversa`

Regras da conversa para os dois lados (a mais restritiva vence): mensagens e não lidas em até 48 h; chamada atendida não fica registrada; chamada não atendida fica 48 h para quem recebeu (quem ligou não apaga esse aviso); sem marcas; conversa vazia some; apagar tudo só com consentimento; caixa do servidor em até 48 h

**Tipos:** `Preferencias` (struct), `Regras` (type), `Direcao` (enum), `Midia` (enum), `RegistroDeChamada` (struct), `Evento` (enum), `Exibicao` (enum)

**Constantes:** `HORA`, `PADRAO_MENSAGENS`, `PADRAO_CHAMADAS_NAO_ATENDIDAS`, `PADRAO_NAO_LIDAS`, `RETENCAO_MAXIMA_NA_CAIXA`, `PADRAO_PRAZO_DO_NOME_NA_LISTA`

**Funções:**

```rust
pub fn padrao() -> Self
pub fn combinar(participantes: &[Preferencias]) -> Regras
pub fn nome_deve_sumir_da_lista(regras: &Regras, exibido_desde: u64, agora: u64) -> bool
pub fn prazo_efetivo_para_contato(preferencias: &Preferencias, excecao_do_contato: Option<PrazoDeVisibilidade>) -> PrazoDeVisibilidade
pub fn aceitar_retirada(regras: &Regras, autor_da_original: &[u8; 32], quem_pede: &[u8; 32], ja_lida: bool) -> bool
pub fn mostrar_visualizada(regras: &Regras) -> bool
pub fn aplicar_na_politica(regras: &Regras, conversa: &mut politica::Regras)
pub fn registrar_chamada(regras: &Regras, minhas: &Preferencias, conversa: [u8; 16], inicio: u64, fim: u64, direcao: Direcao, midia: Midia) -> Option<RegistroDeChamada>
pub fn registros_vigentes(registros: Vec<RegistroDeChamada>, agora: u64) -> Vec<RegistroDeChamada>
pub fn registrar_chamada_com_prazo_de_visibilidade(regras: &Regras, minhas: &Preferencias, conversa: [u8; 16], inicio: u64, fim: u64, direcao: Direcao, midia: Midia, prazo_escolhido: Option<PrazoDeVisibilidade>) -> Option<RegistroDeChamada>
pub fn onde_mostrar(regras: &Regras, e: Evento) -> Exibicao
pub fn conversa_deve_sumir(regras: &Regras, mensagens_restantes: usize) -> bool
pub fn pode_apagar_tudo_de_todos(regras: &Regras) -> bool
pub fn prazo_na_caixa_do_servidor(regras: &Regras) -> u64
```

## `convites`

Sem telefone e sem spam: conversa só com convite de uso único e prazo; o servidor guarda só o hash

**Tipos:** `Convite` (type), `RecusaDeConvite` (enum), `Convites` (struct)

**Constantes:** `PALAVRAS`

**Funções:**

```rust
pub fn criar(&mut self, agora: u64, validade_segundos: u64) -> Convite
pub fn resgatar(&mut self, convite: &[u8; 32], agora: u64) -> Result<(), RecusaDeConvite>
pub fn limpar(&mut self, agora: u64)
pub fn quantidade(&self) -> usize
pub fn palavras_de_confirmacao(chave_a: &[u8], chave_b: &[u8]) -> [&'static str; 6]
pub fn palavras_com_dominio(dominio: &[u8], chave_a: &[u8], chave_b: &[u8]) -> [&'static str; 6]
pub fn opcoes_para_confirmar(palavras: &[&'static str; 6], posicao: usize) -> [&'static str; 4]
pub fn confirmacao_correta(palavras: &[&'static str; 6], posicao: usize, escolhida: &str) -> bool
```

## `corporativo`

Plano corporativo: suspender sem ler (lista de contas assinada pela empresa, validade curta, relógio confiável), recuperar só conta suspensa com a chave mestra, k de n para reativar, suspender em massa e trocar administradores

**Tipos:** `IdConta` (type), `Politica` (struct), `Empresa` (struct), `Lista` (struct), `Violacao` (enum), `Quadro` (struct), `Decisao` (enum), `Motivo` (enum), `OrdemDeRecuperacao` (struct)

**Constantes:** `CARENCIA_DA_RECUPERACAO`

**Funções:**

```rust
pub fn padrao() -> Self
pub fn assinar_lista(chave: &ChaveDeAssinatura, l: &Lista) -> Aprovacao
pub fn exigido_para_transicao(p: &Politica, anterior: &Lista, nova: &Lista) -> Result<u8, Violacao>
pub fn validar_lista(e: &Empresa, anterior: Option<&Lista>, nova: &Lista, agora: u64) -> Result<(), Violacao>
pub fn publicar(&mut self, e: &Empresa, nova: Lista, agora: u64) -> Result<(), Violacao>
pub fn decidir_no_aparelho<'a>(e: &Empresa, ultima_aceita: Option<&'a Lista>, nova: Option<&'a Lista>, conta: &IdConta, agora: u64) -> (Decisao, Option<&'a Lista>)
pub fn tempo_confiavel(relogio_do_aparelho: u64, ultimo_horario_do_servidor: u64, decorrido_monotonico: Option<u64>) -> Option<u64>
pub fn decidir_com_relogio_seguro<'a>(e: &Empresa, ultima_aceita: Option<&'a Lista>, nova: Option<&'a Lista>, conta: &IdConta, relogio_do_aparelho: u64, ultimo_horario_do_servidor: u64, decorrido_monotonico: Option<u64>) -> (Decisao, Option<&'a Lista>)
pub fn selar_pacote_de_recuperacao(chave_mestra: &ChavePublica, conta: &IdConta, chaves_da_conta: &[u8]) -> Result<Vec<u8>, Violacao>
pub fn assinar_ordem(chave: &ChaveDeAssinatura, o: &OrdemDeRecuperacao) -> Aprovacao
pub fn autorizar_recuperacao(e: &Empresa, lista_vigente: &Lista, o: &OrdemDeRecuperacao, agora: u64) -> Result<(), Violacao>
pub fn abrir_pacote_de_recuperacao(e: &Empresa, lista_vigente: &Lista, o: &OrdemDeRecuperacao, chave_mestra_secreta: &[u8], pacote: &[u8], agora: u64) -> Result<Zeroizing<Vec<u8>>, Violacao>
pub fn validar_troca_de_administradores(e: &Empresa, novos: &[ChaveDeVerificacao], numero: u64, emitida_em: u64, valida_ate: u64, minimo_declarado: u8, assinaturas: &[Aprovacao], agora: u64) -> Result<(), Violacao>
pub fn assinar_troca_de_administradores(chave: &ChaveDeAssinatura, e: &Empresa, novos: &[ChaveDeVerificacao], numero: u64, emitida_em: u64, valida_ate: u64, minimo_declarado: u8) -> Aprovacao
```

## `custodia`

Chave mestra dividida entre guardiões, k de n (partilha replicada por XOR), com compromisso contra peças adulteradas

**Tipos:** `Peca` (struct), `PartesDoGuardiao` (struct), `ErroDeCustodia` (enum)

**Constantes:** `MAX_GUARDIOES`

**Funções:**

```rust
pub fn dividir(segredo: &[u8], n: u8, k: u8) -> Result<(Vec<PartesDoGuardiao>, [u8; 64]), ErroDeCustodia>
pub fn reconstruir(participantes: &[&PartesDoGuardiao]) -> Result<Zeroizing<Vec<u8>>, ErroDeCustodia>
pub fn para_bytes(&self) -> Zeroizing<Vec<u8>>
pub fn de_bytes(b: &[u8]) -> Result<Self, ErroDeCustodia>
```

## `custodia_do_cofre`

Chave do cofre pessoal dividida 2 de 2 entre aparelho e servidor: nenhum dos dois sozinho reconstrói nada, nem o servidor sob ordem judicial — reaproveita `custodia` sem criptografia nova; cache local da chave reconstruída agora tem prazo (`verificar_cache`, 72h, mesmo padrão offline da Cláusula 6a) — um aparelho isolado da rede não reabre o cofre sozinho para sempre; falta transporte de rede e rota de servidor, ver o próprio módulo

**Tipos:** `ErroDoCacheLocal` (enum), `ErroDeCustodiaDoCofre` (enum)

**Constantes:** `PRAZO_MAXIMO_DO_CACHE_SEGUNDOS`, `N`, `K`

**Funções:**

```rust
pub fn verificar_cache(reconstruida_em: u64, agora: u64) -> Result<(), ErroDoCacheLocal>
pub fn dividir_entre_aparelho_e_servidor( chave_do_cofre: &[u8], ) -> Result<(PartesDoGuardiao, PartesDoGuardiao), ErroDeCustodiaDoCofre>
pub fn reconstruir_a_partir_das_duas_pecas( peca_aparelho: &PartesDoGuardiao, peca_servidor: &PartesDoGuardiao, ) -> Result<Zeroizing<Vec<u8>>, ErroDeCustodiaDoCofre>
```

## `custodia_mutua`

Aba trancada por contato ("cofre da conversa"): item sob guarda dos dois lados ao mesmo tempo; qualquer um apaga para os dois a qualquer momento (par de chaves descartável destruído, não só removido de uma lista); sem metadado de participantes no servidor

**Tipos:** `PrazoDeCustodiaMutua` (struct), `Id` (type), `Erro` (enum), `ServidorCego` (struct), `Referencia` (struct)

**Constantes:** `PRAZO_MINIMO_DIAS`, `PRAZO_MAXIMO_DIAS`

**Funções:**

```rust
pub fn de_dias(dias: u64) -> Self
pub fn em_dias(self) -> u64
pub fn em_segundos(self) -> u64
pub fn para_bytes(&self) -> Zeroizing<Vec<u8>>
pub fn de_bytes(b: &[u8]) -> Result<Self, Erro>
pub fn encaminhar( servidor: &mut ServidorCego, item: &Item, agora: u64, prazo: PrazoDeCustodiaMutua, ) -> Result<Referencia, Erro>
pub fn abrir(servidor: &ServidorCego, referencia: &Referencia, agora: u64) -> Result<Item, Erro>
pub fn revogar(servidor: &mut ServidorCego, referencia: &Referencia) -> bool
pub fn remover_vencidos(servidor: &mut ServidorCego, agora: u64) -> usize
pub fn preparar_pacote_de_transferencia( referencias: &[Referencia], agora: u64, chave_local_do_cofre: &[u8], chave_publica_do_aparelho_novo: &ChavePublica, ) -> Result<Vec<u8>, Erro>
pub fn abrir_pacote_de_transferencia( secreta_do_aparelho_novo: &ChaveSecreta, publica_do_aparelho_novo: &ChavePublica, pacote: &[u8], ) -> Result<(Vec<Referencia>, Zeroizing<Vec<u8>>), Erro>
```

## `custodia_por_categoria`

Política de continuidade em servidor por `cofre::Tipo` (granularidade por categoria): padrão de fábrica é só o aparelho para as nove categorias, sem exceção — o dono liga o complemento de `custodia_do_cofre` categoria por categoria, nunca todas de uma vez

**Tipos:** `Politica` (enum), `Politicas` (struct)

**Funções:**

```rust
pub fn para(&self, tipo: Tipo) -> Politica
pub fn definir(&mut self, tipo: Tipo, pol: Politica)
pub fn usa_complemento_de_servidor(&self, tipo: Tipo) -> bool
pub fn recomendadas_no_primeiro_uso() -> &'static [Tipo]
```

## `direto`

Arquivo de Nimbus para Nimbus fora da conversa: inspecionado, sem metadados, cifrado para quem recebe, entregue uma vez pelos retransmissores (nunca direto entre aparelhos), prazo de até 48 h; NimbusDrop lacrado: por proximidade, só abre com a chave entregue pelo servidor (ND2–ND5)

**Tipos:** `ErroDireto` (enum), `Remessa` (struct), `Recebido` (struct), `Caixa` (struct), `ChaveDoLacre` (struct), `Lacrado` (struct), `MotivoDoLacre` (enum), `ErroDoLacre` (enum)

**Constantes:** `PRAZO_MAXIMO`

**Funções:**

```rust
pub fn preparar(destino: &ChavePublica, id: [u8; 16], nome: &str, dados: &[u8], agora: u64, prazo: u64) -> Result<Remessa, ErroDireto>
pub fn abrir(sec: &ChaveSecreta, publ: &ChavePublica, r: &Remessa, agora: u64) -> Result<Recebido, ErroDireto>
pub fn depositar(&mut self, r: Remessa)
pub fn retirar(&mut self, id: &[u8; 16], agora: u64) -> Result<Remessa, ErroDireto>
pub fn limpar(&mut self, agora: u64)
pub fn quantidade(&self) -> usize
pub fn preparar_assinado(remetente: &assinatura::ChaveDeAssinatura, destino: &ChavePublica, id: [u8; 16], nome: &str, dados: &[u8], agora: u64, prazo: u64) -> Result<Remessa, ErroDireto>
pub fn abrir_assinado(sec: &ChaveSecreta, publ: &ChavePublica, r: &Remessa, agora: u64) -> Result<(Recebido, assinatura::ChaveDeVerificacao), ErroDireto>
pub fn de_bytes(b: [u8; 32]) -> Self
pub fn exportar(&self) -> Zeroizing<[u8; 32]>
pub fn lacrar(destino: &ChavePublica, id: [u8; 16], nome: &str, dados: &[u8], agora: u64, prazo: u64) -> Result<(Lacrado, ChaveDoLacre), ErroDireto>
pub fn pode_abrir_lacrado(l: &Lacrado, chave_chegou: bool, hora_confiavel: Option<u64>, nivel: integridade::Nivel) -> Result<(), MotivoDoLacre>
pub fn abrir_lacrado(sec: &ChaveSecreta, publ: &ChavePublica, l: &Lacrado, chave: Option<&ChaveDoLacre>, hora_confiavel: Option<u64>, nivel: integridade::Nivel) -> Result<Recebido, ErroDoLacre>
```

## `envelope`

Cifra híbrida pós-quântica (suíte padrão e suíte FIPS), preenchimento de tamanho, zeragem de memória

**Tipos:** `Erro` (enum), `Suite` (enum), `ChavePublica` (struct), `ChaveSecreta` (struct)

**Constantes:** `VERSAO`, `TAM_X25519`, `TAM_MAXIMO`, `TAM_CHAVE_SECRETA`, `TAM_CHAVE_SIMETRICA`

**Funções:**

```rust
pub fn tamanho_permitido(n: usize) -> bool
pub fn para_bytes(&self) -> Zeroizing<Vec<u8>>
pub fn de_bytes(b: &[u8]) -> Result<Self, Erro>
pub fn gerar_par_de_chaves() -> (ChavePublica, ChaveSecreta)
pub fn para_bytes(&self) -> Vec<u8>
pub fn de_bytes(b: &[u8]) -> Result<Self, Erro>
pub fn tamanho_preenchido(n: usize) -> usize
pub fn selar(destinatario: &ChavePublica, texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro>
pub fn selar_com(suite: Suite, destinatario: &ChavePublica, texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro>
pub fn selar_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], texto_claro: &[u8], aad: &[u8]) -> Result<Vec<u8>, Erro>
pub fn abrir_com_chave_simetrica(chave: &[u8; TAM_CHAVE_SIMETRICA], envelope: &[u8], aad: &[u8]) -> Result<Zeroizing<Vec<u8>>, Erro>
pub fn abrir(chave_secreta: &ChaveSecreta, chave_publica: &ChavePublica, envelope: &[u8], aad: &[u8]) -> Result<Zeroizing<Vec<u8>>, Erro>
```

## `escudo`

Escudo de Conversa: borra a conversa em todos os aparelhos se alguém captura a tela

**Tipos:** `EstadoDoParticipante` (struct), `Motivo` (enum), `Acao` (enum), `Escudo` (struct)

**Constantes:** `VALIDADE_SEGUNDOS`, `TOLERANCIA_DE_RELOGIO`

**Funções:**

```rust
pub fn assinar_estado(chave: &ChaveDeAssinatura, participante: [u8; 32], conversa: [u8; 16], sequencia: u64, carimbo: u64, nivel: Nivel, captura_ativa: bool) -> EstadoDoParticipante
pub fn novo(conversa: [u8; 16], participantes: HashMap<[u8; 32], ChaveDeVerificacao>) -> Self
pub fn receber(&mut self, s: &EstadoDoParticipante, agora: u64)
pub fn decidir(&self, agora: u64) -> Acao
```

## `expiracao`

Mensagens temporárias Modo A e Modo B, com prazo assinado pelo servidor; hora confirmada, assinada pelo servidor com desafio de uso único (sem confirmação há mais de 15 min, nada com prazo aparece)

**Tipos:** `Politica` (struct), `Bilhete` (struct), `HoraDoServidor` (struct), `RecusaDeHora` (enum), `AceleracaoPorTentativas` (struct)

**Constantes:** `LIMITE_PARA_LER`, `VIDA_MAXIMA`, `TEMPORIZADORES`, `LIMITE_SEM_HORA_CONFIRMADA`

**Funções:**

```rust
pub fn emitir(chave_servidor: &ChaveDeAssinatura, id_mensagem: [u8; 16], agora_servidor: u64, politica: Politica) -> Bilhete
pub fn conferir(chave_publica_servidor: &ChaveDeVerificacao, b: &Bilhete) -> bool
pub fn prazo_final(b: &Bilhete, aberta_em: Option<u64>, fechada_em: Option<u64>) -> Option<u64>
pub fn expirou(b: &Bilhete, agora_servidor: u64, aberta_em: Option<u64>, fechada_em: Option<u64>) -> bool
pub fn emitir_hora(chave_servidor: &ChaveDeAssinatura, desafio: [u8; 32], agora_servidor: u64) -> HoraDoServidor
pub fn conferir_hora(chave_publica_servidor: &ChaveDeVerificacao, desafio_enviado: &[u8; 32], ultima_aceita: Option<u64>, h: &HoraDoServidor) -> Result<u64, RecusaDeHora>
pub fn hora_confiavel(relogio_do_aparelho: u64, ultimo_horario_do_servidor: u64, decorrido_monotonico: Option<u64>) -> Option<u64>
pub fn tempo_para_destruir_mesmo_offline(ultimo_horario_do_servidor: u64, decorrido_monotonico: u64) -> u64
pub fn padrao() -> Self
pub fn a_partir_da_politica(pol: &crate::coacao::Politica) -> Self
pub fn segundos_extras(&self, erros_consecutivos: u32) -> u64
pub fn tentativas_restantes_antes_de_acelerar(&self, erros_consecutivos: u32) -> u32
pub fn tempo_para_destruir_considerando_tentativas( ultimo_horario_do_servidor: u64, decorrido_monotonico: u64, erros_consecutivos: u32, aceleracao: &AceleracaoPorTentativas, ) -> u64
pub fn pode_exibir(b: &Bilhete, hora: Option<u64>, aberta_em: Option<u64>, fechada_em: Option<u64>) -> bool
```

## `grupo`

Conversa em grupo (v37, terceira rodada): só a camada de membros e a regra efetiva (combinação mais restritiva de sempre); o protocolo de mensagens em grupo continua sendo o MLS, fora deste núcleo

**Tipos:** `IdParticipante` (type), `IdGrupo` (type), `ErroDeGrupo` (enum), `Grupo` (struct)

**Constantes:** `MAX_MEMBROS`

**Funções:**

```rust
pub fn criar(id: IdGrupo, criador: IdParticipante, preferencias_do_criador: Preferencias) -> Self
pub fn id(&self) -> IdGrupo
pub fn criador(&self) -> IdParticipante
pub fn tamanho(&self) -> usize
pub fn e_membro(&self, quem: &IdParticipante) -> bool
pub fn e_admin(&self, quem: &IdParticipante) -> bool
pub fn esta_silenciado(&self, quem: &IdParticipante) -> bool
pub fn admins(&self) -> impl Iterator<Item = &IdParticipante>
pub fn adicionar_membro(&mut self, solicitante: &IdParticipante, novo: IdParticipante, preferencias: Preferencias) -> Result<(), ErroDeGrupo>
pub fn remover_membro(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo>
pub fn promover_admin(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo>
pub fn rebaixar_admin(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo>
pub fn silenciar(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo>
pub fn dessilenciar(&mut self, solicitante: &IdParticipante, alvo: IdParticipante) -> Result<(), ErroDeGrupo>
pub fn pode_enviar(&self, quem: &IdParticipante) -> bool
pub fn definir_teto_de_encaminhamento(&mut self, solicitante: &IdParticipante, teto: Option<Nivel>) -> Result<(), ErroDeGrupo>
pub fn teto_de_encaminhamento(&self) -> Option<Nivel>
pub fn atualizar_preferencias(&mut self, quem: &IdParticipante, novas: Preferencias) -> Result<(), ErroDeGrupo>
pub fn regras_efetivas(&self) -> Result<Regras, ErroDeGrupo>
pub fn membros(&self) -> impl Iterator<Item = &IdParticipante>
pub fn aceitar_retirada_no_grupo(&self, regras: &Regras, autor_da_original: &IdParticipante, quem_pede: &IdParticipante, ja_lida: bool) -> bool
```

## `higiene`

Arquivo sai sem rastro: remove EXIF/GPS/autor de JPEG e PNG, nome neutro; exige achatar PDF e Office

**Tipos:** `Formato` (enum), `ErroDeHigiene` (enum)

**Funções:**

```rust
pub fn detectar(dados: &[u8]) -> Formato
pub fn limpar(dados: &[u8]) -> Result<Vec<u8>, ErroDeHigiene>
pub fn limpar_jpeg(d: &[u8]) -> Result<Vec<u8>, ErroDeHigiene>
pub fn limpar_png(d: &[u8]) -> Result<Vec<u8>, ErroDeHigiene>
pub fn nome_neutro(extensao: &str) -> String
pub fn crc32(dados: &[u8]) -> u32
```

## `historico`

Histórico guardado cifrado no servidor e aberto só no aparelho; apagamento criptográfico por janela de tempo

**Tipos:** `IdDeBloco` (type), `ServidorCego` (struct), `ChavesDoDono` (struct), `ErroDeLeitura` (enum)

**Funções:**

```rust
pub fn novas(duracao_da_janela: u64) -> Self
pub fn destruir_janelas_vencidas(&mut self, agora: u64) -> usize
pub fn janelas_ativas(&self) -> usize
pub fn apagar_conta(self)
pub fn guardar(servidor: &mut ServidorCego, chaves: &mut ChavesDoDono, conteudo: &[u8], expira_em: Option<u64>) -> Result<IdDeBloco, envelope::Erro>
pub fn ler(servidor: &ServidorCego, chaves: &ChavesDoDono, id: &IdDeBloco) -> Result<Zeroizing<Vec<u8>>, ErroDeLeitura>
```

## `inspecao`

Tipo real pelos primeiros bytes; bloqueia programas, instaladores, scripts, macros e extensão mentirosa; PDF e Office só como imagem

**Tipos:** `TipoReal` (enum), `Motivo` (enum), `Decisao` (enum), `Laudo` (struct)

**Funções:**

```rust
pub fn tipo_real(d: &[u8]) -> TipoReal
pub fn inspecionar(nome: &str, dados: &[u8]) -> Laudo
pub fn e_caractere_enganoso(c: char) -> bool
pub fn sanear_texto_exibido(texto: &str) -> (String, usize)
pub fn texto_seguro_para_exibir(texto: &str) -> bool
```

## `integridade`

Verificação do aparelho ao abrir o app: verde, amarelo, vermelho; computador nunca passa do amarelo; não existe cliente web

**Tipos:** `Plataforma` (enum), `Sinais` (struct), `Inicio` (enum), `OrigemDaAtestacao` (enum), `Nivel` (enum), `Veredito` (struct), `Permissoes` (struct), `Desafios` (struct), `RecusaDeDesafio` (enum)

**Constantes:** `MESES_TOLERADOS_SEM_ATUALIZACAO`, `VALIDADE_DO_VEREDITO`, `VALIDADE_DO_DESAFIO`

**Funções:**

```rust
pub fn tem_atestacao_do_fabricante(p: Plataforma) -> bool
pub fn avaliar(p: Plataforma, s: &Sinais) -> Veredito
pub fn permissoes(nivel: Nivel) -> Permissoes
pub fn sessao_permitida(v: &Veredito) -> bool
pub fn precisa_reavaliar(avaliado_em: u64, agora: u64) -> bool
pub fn emitir(&mut self, agora: u64) -> [u8; 32]
pub fn consumir(&mut self, valor: &[u8; 32], agora: u64) -> Result<(), RecusaDeDesafio>
```

## `liberacao`

Mensagem não sai; exceção estreita: cartão encaminhável (texto ou localização, marcado no envio, prazo de até 4 h, só dentro do Nimbus, um salto). Arquivo só com liberação assinada pelo autor, com prazo opcional; revogação; aviso ao autor; saída única com metadados limpos

**Tipos:** `Nivel` (enum), `Uso` (enum), `Destino` (enum), `Autorizacao` (struct), `Recusa` (enum), `Exportacao` (struct)

**Constantes:** `PRAZOS`, `PRAZO_MAXIMO_DO_CARTAO`

**Funções:**

```rust
pub fn sai_do_dominio_nimbus(d: Destino) -> bool
pub fn uso_correspondente(d: Destino) -> Option<Uso>
pub fn destino_bloqueado_pelo_teto(d: Destino, teto: Nivel) -> bool
pub fn e_mensagem(t: TipoDeConteudo) -> bool
pub fn pode_ser_cartao(t: TipoDeConteudo) -> bool
pub fn teto_da_conversa(limite_da_empresa: Option<Nivel>) -> Nivel
pub fn resolver_nivel_de_encaminhamento(master: Nivel, padrao_do_contato: Option<Nivel>, escolha_na_mensagem: Option<Nivel>, teto: Nivel) -> Nivel
pub fn autorizar(autor: &ChaveDeAssinatura, hash_do_conteudo: [u8; 32], nivel: Nivel, versao: u64, prazo_segundos: Option<u64>, some_ao_fim: bool) -> Autorizacao
pub fn nivel_autorizado(chave_do_autor: &ChaveDeVerificacao, autorizacoes: &[Autorizacao], hash: &[u8; 32], tipo: TipoDeConteudo, aberto_em: Option<u64>, agora: u64) -> Nivel
pub fn efetivo(tipo: TipoDeConteudo, visualizacao_unica: bool, teto: Nivel, autorizado: Nivel) -> Nivel
pub fn pode(tipo: TipoDeConteudo, nivel: Nivel, uso: Uso, recebido_por_encaminhamento: bool) -> bool
pub fn item_deve_sumir(a: &Autorizacao, aberto_em: Option<u64>, agora: u64) -> bool
pub fn ao_encaminhar(recebido: Nivel, escolhido: Nivel) -> Nivel
pub fn avisar_o_autor(uso: Uso) -> bool
pub fn preparar_para_fora(tipo: TipoDeConteudo, nivel: Nivel, dados: &[u8]) -> Result<Exportacao, Recusa>
```

## `limites`

Servidor: cota por janela, castigo que dobra a cada falha até 24 h, memória com teto e falha fechada

**Tipos:** `Limite` (enum), `Limitador` (struct)

**Constantes:** `CASTIGO_MAXIMO`

**Funções:**

```rust
pub fn novo(maximo: u32, janela: u64, castigo_inicial: u64, capacidade: usize) -> Self
pub fn permitir(&mut self, chave: &[u8], agora: u64) -> Result<(), Limite>
pub fn registrar_falha(&mut self, chave: &[u8], agora: u64) -> u64
pub fn registrar_sucesso(&mut self, chave: &[u8])
pub fn acompanhadas(&self) -> usize
```

## `memoria`

Segredo em memória blindada: zerado, travado na RAM, nunca impresso em registro ou relatório de falha

**Tipos:** `Segredo` (struct)

**Funções:**

```rust
pub fn com_tamanho(tamanho: usize) -> Self
pub fn de(bytes: &[u8]) -> Self
pub fn tamanho(&self) -> usize
pub fn travado_na_ram(&self) -> bool
pub fn usar<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R
pub fn usar_mut<R>(&mut self, f: impl FnOnce(&mut [u8]) -> R) -> R
pub fn limpar(&mut self)
```

## `metadados`

Caixa que muda todo dia, índice cego no disco com chave destruída depois de 48 h, registros legais selados para chave que não fica no servidor

**Tipos:** `ChavesDeIndice` (struct), `ErroDeRegistro` (enum)

**Constantes:** `SEGUNDOS_POR_EPOCA`, `VIDA_DA_CHAVE_DE_INDICE`, `TAM_DO_REGISTRO`

**Funções:**

```rust
pub fn epoca_de(agora: u64) -> u64
pub fn caixa_da_epoca(segredo_da_relacao: &[u8; 32], epoca: u64) -> [u8; 32]
pub fn indice(&mut self, epoca: u64, endereco: &[u8; 32]) -> [u8; 32]
pub fn indice_existente(&self, epoca: u64, endereco: &[u8; 32]) -> Option<[u8; 32]>
pub fn selar_no_disco(&mut self, epoca: u64, endereco: &[u8; 32], bloco: &[u8]) -> Vec<u8>
pub fn abrir_do_disco(&self, epoca: u64, endereco: &[u8; 32], selado: &[u8]) -> Option<Zeroizing<Vec<u8>>>
pub fn destruir_vencidas(&mut self, agora: u64) -> usize
pub fn selar_registro(chave_juridica: &ChavePublica, carimbo: u64, ip_do_retransmissor: &str) -> Result<Vec<u8>, ErroDeRegistro>
pub fn abrir_registro(sec: &ChaveSecreta, publ: &ChavePublica, selado: &[u8]) -> Result<(u64, String), ErroDeRegistro>
```

## `modos`

Seu padrão de envio por tipo (texto, foto e vídeo, voz, arquivo) e a escolha na hora; a conversa continua sendo o teto

**Tipos:** `Modo` (enum), `Categoria` (enum), `Padroes` (struct)

**Funções:**

```rust
pub fn categoria(t: TipoDeConteudo) -> Categoria
pub fn recomendados() -> Self
pub fn para(&self, t: TipoDeConteudo) -> Modo
pub fn regra_do_modo(m: Modo) -> Regras
pub fn regra_da_mensagem(conversa: &Regras, padroes: &Padroes, tipo: TipoDeConteudo, escolha_na_hora: Option<Modo>) -> Regras
pub fn rotulo(m: Modo) -> String
```

## `notificacao`

Cadência fixa: uma notificação por aparelho por janela, real ou de cobertura, do mesmo tamanho; texto genérico

**Tipos:** `Plataforma` (enum), `Canal` (enum), `Tipo` (enum), `Decisao` (enum)

**Constantes:** `TEXTO_GENERICO`, `TEXTO_CHAMADA_BLOQUEADO`

**Funções:**

```rust
pub fn canal_para(p: Plataforma) -> Canal
pub fn montar_rodada(aparelhos: &[ChavePublica], tem_novidade: &[bool]) -> Vec<Vec<u8>>
pub fn ao_receber(sec: &ChaveSecreta, publ: &ChavePublica, carga: &[u8]) -> Decisao
```

## `pacote`

Nome do arquivo, tipo, data e pasta cifrados junto com o conteúdo, num bloco de tamanho padronizado

**Tipos:** `Metadados` (struct), `ErroDePacote` (enum), `ErroDeSelarMidia` (enum)

**Funções:**

```rust
pub fn selar_item(chave: &ChavePublica, meta: &Metadados, conteudo: &[u8]) -> Result<Vec<u8>, ErroDePacote>
pub fn abrir_item(sec: &ChaveSecreta, publ: &ChavePublica, bloco: &[u8]) -> Result<(Metadados, Zeroizing<Vec<u8>>), ErroDePacote>
pub fn selar_midia( chave: &ChavePublica, meta: &Metadados, bytes_originais: &[u8], ) -> Result<Vec<u8>, ErroDeSelarMidia>
```

## `panico`

Travas de emergência decididas dentro do aparelho: palavra mágica, toque de pânico, longe de casa, movimento brusco, coação e sem me mexer, cada uma com a reação escolhida; mais um gatilho de origem remota (`comando_remoto`, nova, 10/10/2026), a única exceção deliberada a "nada sai para o servidor"

**Tipos:** `Reacao` (enum), `Gatilho` (enum), `Config` (struct), `Sinais` (struct)

**Funções:**

```rust
pub fn reacao_fixa(&self) -> Option<Reacao>
pub fn decidir(cfg: &Config, s: &Sinais) -> Option<Reacao>
```

## `pedido_de_guarda`

Guardar no cofre pessoal de quem recebe é pedido item por item pelo destinatário e aprovado ou negado pelo autor — nunca uma permissão genérica dada de antemão, e nunca confundir com a aba trancada (`custodia_mutua`, intocada); zero criptografia nova: aprovar produz a `liberacao::Autorizacao` normal (`Nivel::GuardarNoCofre`) que o resto do núcleo já valida; só o hash do conteúdo viaja, nunca um dado do remetente

**Tipos:** `Pedido` (struct), `ErroDePedido` (enum), `Resposta` (enum), `RespostaDeAbaTrancada` (enum)

**Funções:**

```rust
pub fn solicitar(quem_pede: &ChaveDeAssinatura, hash_do_conteudo: [u8; 32], criado_em: u64) -> Pedido
pub fn responder_para_destino( chave_do_solicitante: &ChaveDeVerificacao, pedido: &Pedido, autor: &ChaveDeAssinatura, versao: u64, aprovar: bool, prazo_segundos: Option<u64>, destino: Destino, ) -> Result<Resposta, ErroDePedido>
pub fn responder( chave_do_solicitante: &ChaveDeVerificacao, pedido: &Pedido, autor: &ChaveDeAssinatura, versao: u64, aprovar: bool, prazo_segundos: Option<u64>, ) -> Result<Resposta, ErroDePedido>
pub fn responder_pedido_de_aba_trancada( chave_do_solicitante: &ChaveDeVerificacao, pedido: &Pedido, aprovar: bool, prazo: PrazoDeCustodiaMutua, ) -> Result<RespostaDeAbaTrancada, ErroDePedido>
```

## `politica`

Regras por conversa e por mensagem: vence a mais restritiva; visualização única para toda mídia; bloqueios de encaminhar, copiar, salvar

**Tipos:** `TipoDeConteudo` (enum), `PrazoDeVisibilidade` (enum)

**Constantes:** `TRES_MESES_EM_SEGUNDOS`

**Funções:**

```rust
pub fn em_segundos(self) -> u64
```

## `prechaves`

Chaves de recebimento de vida curta, assinadas pela identidade e destruídas depois do prazo: envelopes antigos ficam ilegíveis para sempre, mesmo com o aparelho tomado

**Tipos:** `ErroDePrechave` (enum), `PrechavePublicada` (struct), `PrechaveSecreta` (struct), `Chaveiro` (struct)

**Constantes:** `VALIDADE_MAXIMA`, `SOBREVIDA_DA_SECRETA`

**Funções:**

```rust
pub fn gerar(identidade: &ChaveDeAssinatura, id: [u8; 16], agora: u64, validade: u64) -> (PrechavePublicada, PrechaveSecreta)
pub fn conferir(identidade: &ChaveDeVerificacao, p: &PrechavePublicada, agora: u64) -> Result<ChavePublica, ErroDePrechave>
pub fn selar(p: &PrechavePublicada, conferida: &ChavePublica, texto: &[u8], aad_do_uso: &[u8]) -> Result<Vec<u8>, ErroDePrechave>
pub fn guardar(&mut self, s: PrechaveSecreta)
pub fn quantidade(&self) -> usize
pub fn abrir(&self, id: &[u8; 16], envelope: &[u8], aad_do_uso: &[u8]) -> Result<Zeroizing<Vec<u8>>, ErroDePrechave>
pub fn apagar_vencidas(&mut self, agora: u64) -> usize
```

## `protecao_do_usuario`

Bloquear (lista só no aparelho, descarte em silêncio), denunciar só por iniciativa do usuário (cifrado para a moderação, tamanho fixo) e apagar a conta (pedido assinado, PIN real, uso único)

**Tipos:** `Digital` (type), `ListaDeBloqueio` (struct), `Entrada` (enum), `Destino` (enum), `ErroDeBloqueio` (enum), `Motivo` (enum), `ErroDeDenuncia` (enum), `PedidoDeApagar` (struct), `RecusaDeApagar` (enum)

**Constantes:** `MAXIMO_DE_BLOQUEADOS`, `TAMANHO_FIXO_DA_DENUNCIA`, `VALIDADE_DO_PEDIDO_DE_APAGAR`

**Funções:**

```rust
pub fn bloquear(&mut self, d: Digital) -> Result<(), ErroDeBloqueio>
pub fn desbloquear(&mut self, d: &Digital)
pub fn esta_bloqueado(&self, d: &Digital) -> bool
pub fn quantos(&self) -> usize
pub fn decidir(&self, remetente: &Digital, _tipo: Entrada) -> Destino
pub fn para_bytes_de_tamanho_fixo(&self) -> Vec<u8>
pub fn de_bytes(b: &[u8]) -> Option<Self>
pub fn preparar_denuncia( moderacao: &ChavePublica, denunciado: &Digital, motivo: Motivo, consentimento_explicito: bool, ) -> Result<Vec<u8>, ErroDeDenuncia>
pub fn pedir_apagar_conta( chave_da_conta: &ChaveDeAssinatura, conta: Digital, agora: u64, nonce: [u8; 16], pin_real_confirmado: bool, em_coacao: bool, ) -> Result<PedidoDeApagar, RecusaDeApagar>
pub fn conferir_pedido_de_apagar( verificacao_da_conta: &ChaveDeVerificacao, p: &PedidoDeApagar, agora: u64, nonces_ja_usados: &mut BTreeSet<[u8; 16]>, ) -> Result<(), RecusaDeApagar>
```

## `recuperacao`

Recuperar a conta: 3 trilhas independentes (guardiões k de n, frase pessoal, pergunta importante — Rodada 21), qualquer 2 das 3 reconstroem o segredo; 72h de carência cancelável pelo aparelho antigo; sem a trilha dos guardiões, exige testemunha viva de um guardião contra coação

**Tipos:** `ErroDeRecuperacao` (enum), `Pacote` (struct), `PacotePerguntaImportante` (struct), `Trilha` (type), `TresTrilhas` (struct)

**Constantes:** `TAM_CHAVE_DA_FRASE`, `CARENCIA_SEGUNDOS`, `TAM_MINIMO_RESPOSTA`, `N_TRILHAS`, `K_TRILHAS`

**Funções:**

```rust
pub fn preparar(segredo: &[u8], guardioes: &[ChavePublica], k: u8) -> Result<Vec<Pacote>, ErroDeRecuperacao>
pub fn abrir_para_devolver(sec: &ChaveSecreta, publ: &ChavePublica, pacote: &Pacote, palavras_conferidas_ao_vivo: bool) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao>
pub fn reunir(devolvidas: &[&[u8]]) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao>
pub fn preparar_frase_pessoal(segredo: &[u8]) -> Result<(Zeroizing<[u8; TAM_CHAVE_DA_FRASE]>, Vec<u8>), ErroDeRecuperacao>
pub fn abrir_frase_pessoal(chave: &[u8; TAM_CHAVE_DA_FRASE], bloco: &[u8]) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao>
pub fn palavras_do_pedido(chave_do_aparelho_novo: &[u8], chave_da_conta: &[u8]) -> [&'static str; 6]
pub fn pode_concluir(pedido_em: u64, agora: u64, cancelado_por_aparelho_antigo: bool) -> Result<(), ErroDeRecuperacao>
pub fn preparar_pergunta_importante(resposta: &[u8], segredo_da_trilha: &[u8]) -> Result<PacotePerguntaImportante, ErroDeRecuperacao>
pub fn abrir_pergunta_importante(resposta: &[u8], pacote: &PacotePerguntaImportante) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao>
pub fn preparar_tres_trilhas(segredo_da_conta: &[u8]) -> Result<TresTrilhas, ErroDeRecuperacao>
pub fn concluir_com_duas_trilhas(a: &Trilha, b: &Trilha) -> Result<Zeroizing<Vec<u8>>, ErroDeRecuperacao>
pub fn pode_concluir_duas_trilhas( pedido_em: u64, agora: u64, cancelado_por_aparelho_antigo: bool, usou_trilha_dos_guardioes: bool, alguma_testemunha_guardia_confirmou_ao_vivo: bool, ) -> Result<(), ErroDeRecuperacao>
```

## `rede`

Automático (Tor nos momentos de identidade, nas conversas blindadas e quando a rede bloqueia), Sempre Tor ou Rápido (sem Tor); nunca rebaixar em silêncio; vários operadores sorteados; cabeçalhos uniformes; Bluetooth sem identificador fixo

**Tipos:** `Preferencia` (enum), `Operacao` (enum), `EstadoDaRede` (enum), `Rota` (enum), `Aviso` (enum), `Decisao` (struct), `Falha` (enum)

**Funções:**

```rust
pub fn e_momento_de_identidade(self) -> bool
pub fn decidir(pref: Preferencia, op: Operacao, rede: EstadoDaRede) -> Result<Decisao, Falha>
pub fn ao_falhar(pref: Preferencia, rota_que_falhou: Rota) -> Result<Decisao, Falha>
pub fn sortear_primeira_etapa(quantidade_de_operadores: usize) -> Option<usize>
pub fn cabecalhos_uniformes() -> Vec<(&'static str, &'static str)>
pub fn identificador_bluetooth(segredo_do_qr: &[u8; 32], janela: u64) -> [u8; 16]
```

## `registro`

Registro público só-de-acréscimo (árvore de Merkle, RFC 9162): chaves, imagens e versões; aparelho fantasma e versão adulterada são denunciados

**Tipos:** `Hash` (type), `Registro` (struct)

**Funções:**

```rust
pub fn hash_de_folha(dado: &[u8]) -> Hash
pub fn acrescentar(&mut self, dado: &[u8]) -> usize
pub fn tamanho(&self) -> usize
pub fn raiz(&self) -> Hash
pub fn prova_de_inclusao(&self, indice: usize) -> Option<Vec<Hash>>
pub fn verificar_inclusao(dado: &[u8], indice: usize, tamanho: usize, prova: &[Hash], raiz: &Hash) -> bool
pub fn raiz_em(&self, tamanho: usize) -> Option<Hash>
pub fn prova_de_consistencia(&self, anterior: usize, atual: usize) -> Option<Vec<Hash>>
pub fn verificar_consistencia(anterior: usize, atual: usize, prova: &[Hash], raiz_anterior: &Hash, raiz_atual: &Hash) -> bool
```

## `retirada_de_aparelho`



**Tipos:** `IdAparelho` (type), `ConfirmacaoDeRecebimento` (struct), `RecusaDeConfirmacao` (enum), `Fase` (enum), `RecusaDeDestruicao` (enum), `SequenciaDeDestruicao` (struct)

**Constantes:** `PASSOS_NO_NUCLEO`, `PASSOS_SO_NA_PONTE`

**Funções:**

```rust
pub fn hash_do_pacote(pacote: &[u8]) -> [u8; 32]
pub fn confirmar_recebimento( chave_do_novo: &ChaveDeAssinatura, aparelho_novo: IdAparelho, hash_do_pacote: [u8; 32], contador: u64, ) -> ConfirmacaoDeRecebimento
pub fn validar_confirmacao( chave_do_novo: &ChaveDeVerificacao, confirmacao: &ConfirmacaoDeRecebimento, aparelho_novo_esperado: &IdAparelho, hash_do_pacote_enviado: &[u8; 32], ultimo_contador_aceito: u64, ) -> Result<(), RecusaDeConfirmacao>
pub fn avancar_apos_confirmacao_valida(fase_atual: Fase) -> Result<Fase, RecusaDeDestruicao>
pub fn pode_destruir_o_antigo(fase_atual: Fase) -> Result<(), RecusaDeDestruicao>
```

## `selo`

Selo de liberação: artefato exato, todos os portões de segurança aprovados, assinado por três chaves diferentes fixadas no código (CI, responsável de segurança e auditor independente); sem ele, servidor e apps não funcionam

**Tipos:** `Selo` (struct), `RecusaDoSelo` (enum)

**Constantes:** `PORTOES_OBRIGATORIOS`, `MINIMO_DE_ASSINATURAS`

**Funções:**

```rust
pub fn resumir(artefato: &[u8]) -> [u8; 64]
pub fn canonico(s: &Selo) -> Result<Vec<u8>, RecusaDoSelo>
pub fn assinar(chave: &ChaveDeAssinatura, s: &Selo) -> Result<([u8; 32], AssinaturaHibrida), RecusaDoSelo>
pub fn conferir(s: &Selo, assinaturas: &[([u8; 32], AssinaturaHibrida)], artefato_rodando: &[u8; 64], chaves_fixadas: &[ChaveDeVerificacao], versao_minima: u64, agora: u64) -> Result<(), RecusaDoSelo>
```

## `senha`

Argon2id (RFC 9106: 64 MiB, 3 passagens, 4 faixas) para PIN de coação e senha de link

**Tipos:** `ErroDeSenha` (enum)

**Constantes:** `MEMORIA_KIB`, `PASSAGENS`, `FAIXAS`

**Funções:**

```rust
pub fn derivar<const N: usize>(senha: &[u8], sal: &[u8]) -> Result<Zeroizing<[u8; N]>, ErroDeSenha>
```

## `sentinela`

Autoverificação na partida e periódica: invariantes de segurança e autotestes criptográficos; qualquer achado trava (falha fechada) e o relatório diz onde e como corrigir

**Tipos:** `Achado` (struct), `Relatorio` (struct), `Valores` (struct)

**Funções:**

```rust
pub fn pode_funcionar(&self) -> bool
pub fn texto(&self) -> String
pub fn atuais() -> Self
pub fn conferir_invariantes() -> Relatorio
pub fn conferir_valores(v: &Valores) -> Relatorio
pub fn autoverificar() -> Relatorio
pub fn autoverificar_com_selo(s: &selo::Selo, assinaturas: &[([u8; 32], assinatura::AssinaturaHibrida)], artefato_rodando: &[u8; 64], chaves_fixadas: &[assinatura::ChaveDeVerificacao], versao_minima: u64, agora: u64) -> Relatorio
```

## `sessoes`

Sessão única, anti-clonagem, derrubada total, contagens anônimas

**Tipos:** `IdConta` (type), `IdAparelho` (type), `TetoDeSessaoDoComputador` (struct), `PeriodoDeArrependimento` (struct), `Gatilho` (enum), `Resultado` (enum), `Registro` (struct), `OrigemDoVinculo` (enum), `RecusaDeVinculo` (enum)

**Constantes:** `TETO_SESSAO_COMPUTADOR_MINIMO_HORAS`, `TETO_SESSAO_COMPUTADOR_MAXIMO_HORAS`, `ARREPENDIMENTO_MINIMO_MINUTOS`, `ARREPENDIMENTO_MAXIMO_MINUTOS`

**Funções:**

```rust
pub fn de_horas(horas: u8) -> Self
pub fn em_segundos(self) -> u64
pub fn de_minutos(minutos: u8) -> Self
pub fn em_segundos(self) -> u64
pub fn registrar_aparelho(&mut self, conta: IdConta, aparelho: IdAparelho)
pub fn revogar_aparelho(&mut self, conta: IdConta, aparelho: IdAparelho)
pub fn registrar_vinculo(&mut self, conta: IdConta, aparelho: IdAparelho, vinculado_em: u64)
pub fn dentro_do_periodo_de_arrependimento(&self, conta: &IdConta, aparelho: &IdAparelho, agora: u64, janela: PeriodoDeArrependimento) -> bool
pub fn revogar_todos_os_aparelhos(&mut self, conta: IdConta)
pub fn conectar(&mut self, conta: IdConta, aparelho: IdAparelho, credencial_valida: bool, atestado: bool, dia: u64) -> Resultado
pub fn desconectar(&mut self, conta: IdConta, aparelho: IdAparelho)
pub fn tem_sessao_ativa(&self, conta: &IdConta) -> bool
pub fn aparelhos_ativos_agora(&self, conta: &IdConta) -> Vec<IdAparelho>
pub fn conexoes_hoje(&self, conta: &IdConta, dia_de_hoje: u64) -> u32
pub fn registrar_conexao_de_computador(&mut self, conta: IdConta, aparelho: IdAparelho, agora: u64, teto: TetoDeSessaoDoComputador)
pub fn sessao_de_computador_expirou(&self, conta: &IdConta, aparelho: &IdAparelho, agora: u64) -> bool
pub fn expirar_computadores_vencidos(&mut self, agora: u64) -> Vec<(IdConta, IdAparelho)>
pub fn palavras_do_vinculo(chave_do_celular: &[u8], chave_do_aparelho_novo: &[u8]) -> [&'static str; 6]
pub fn pode_vincular(origem: OrigemDoVinculo, nivel_do_celular: crate::integridade::Nivel, biometria_ok: bool, palavras_conferidas: bool) -> Result<(), RecusaDeVinculo>
```

## `trafego`

Batida constante de envio: silêncio e conversa ficam iguais para quem observa a rede

**Tipos:** `Pacote` (enum), `Batida` (struct)

**Constantes:** `INTERVALO_MS`

**Funções:**

```rust
pub fn nova(inicio_ms: u64) -> Self
pub fn com_intervalo(inicio_ms: u64, intervalo_ms: u64) -> Self
pub fn enfileirar(&mut self)
pub fn pendentes(&self) -> usize
pub fn na_hora(&mut self, agora_ms: u64) -> Option<Pacote>
```

## `validade`

Apagar sozinho: agenda (após o fim do compromisso), notas, contatos, arquivos, fotos — com apagamento criptográfico

**Tipos:** `TipoDeItem` (enum), `Referencia` (enum), `Validade` (struct), `ErroDeValidade` (enum)

**Constantes:** `MINUTO`, `HORA`, `DIA`, `SEMANA`, `MES_30_DIAS`, `VALIDADE_MAXIMA`

**Funções:**

```rust
pub fn horas(n: u64) -> Self
pub fn dias(n: u64) -> Self
pub fn meses(n: u64) -> Self
pub fn apos_o_compromisso(segundos: u64) -> Self
pub fn predefinicoes(tipo: TipoDeItem) -> Vec<(&'static str, Validade)>
pub fn instante_de_apagar(v: &Validade, criado_em: u64, fim_do_compromisso: Option<u64>) -> Result<u64, ErroDeValidade>
pub fn guardar_com_validade( servidor: &mut ServidorCego, chaves: &mut ChavesDoDono, conteudo: &[u8], validade: Option<&Validade>, criado_em: u64, fim_do_compromisso: Option<u64>, ) -> Result<IdDeBloco, ErroDeValidade>
pub fn deve_esconder(instante_de_apagar: u64, agora: u64) -> bool
```

## `vigia_de_sensores`

Vigia contínuo durante o uso: microfone ou câmera de outro app, tela capturada ou externa, sobreposição, acessibilidade, teclado de terceiros; na dúvida, fecha

**Tipos:** `Sinal` (enum), `Uso` (enum), `Leitura` (struct), `Decisao` (struct)

**Constantes:** `VALIDADE_DA_LEITURA_SEGUNDOS`, `TOLERANCIA_DE_RELOGIO`

**Funções:**

```rust
pub fn decidir(uso: Uso, l: &Leitura, agora: u64) -> Decisao
```

## `voz`

Modulador de voz: Nível 1 deslocador de tom em tempo real; Nível 2 conversão neural para voz-alvo fixa (orquestração no aparelho, inferência nativa). Sempre antes de cifrar

**Tipos:** `Predefinicao` (enum), `PlanoDeJanela` (struct), `DeslocadorDeTom` (struct), `BufferDeAudio` (struct)

**Funções:**

```rust
pub fn semitons(self) -> i32
pub fn fator(self) -> f32
pub fn planejar(p: Predefinicao, _taxa_amostragem: u32) -> PlanoDeJanela
pub fn novo(fator: f32, janela_amostras: usize) -> Self
pub fn para_chamada(p: Predefinicao, janela_amostras: usize, desvio_semitons: f32) -> Self
pub fn fator(&self) -> f32
pub fn amostra(&mut self, x: f32) -> f32
pub fn processar(&mut self, bloco: &mut [f32])
pub fn limitar(amostras: &mut [f32])
```

