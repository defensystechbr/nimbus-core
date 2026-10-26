#!/usr/bin/env python3
"""Catraca de segurança: as proteções só podem aumentar, nunca diminuir.

Quem fizer manutenção no futuro (de dentro ou contratado) não consegue, sem que a verificação reprove:
apagar testes, defeitos plantados ou regras de varredura; criar rota sem limite de uso; liberar `unsafe`;
desligar a parada por estouro aritmético; tirar a RLS forçada de uma tabela do banco.
Baixar um mínimo exige mudar CATRACA.json — arquivo protegido por CODEOWNERS — e é uma edição visível na revisão
do pedido de mudança, sob o regime de aprovador único declarado em docs/DECISAO_APROVADOR_UNICO.md.

Uso: python3 verificacao/catraca.py [pasta do banco do servidor, opcional]
"""
import glob, json, os, re, sys

NUCLEO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
falhas = []
def falha(m): falhas.append(m); print(f'  ✗ {m}')
def ler(p): return open(os.path.join(NUCLEO, p), encoding='utf-8').read()
def codigo(texto):
    # Só o que é código de verdade: tira linhas de comentário (// e #) e comentários no fim da linha.
    linhas = []
    for l in texto.split('\n'):
        t = l.strip()
        if t.startswith('//') or (t.startswith('#') and not t.startswith('#![') and not t.startswith('#[')): continue
        linhas.append(re.sub(r'\s//.*$', '', l))
    return '\n'.join(linhas)

c = json.loads(ler('CATRACA.json'))

# 1. Testes: o mínimo do verificar.sh nunca abaixo do mínimo da catraca.
m = re.search(r'MINIMO_DE_TESTES=(\d+)', ler('verificar.sh'))
if not m or int(m.group(1)) < c['testes_minimos']: falha(f"MINIMO_DE_TESTES abaixo de {c['testes_minimos']}")

# 2. Defeitos plantados.
n_mut = len(re.findall(r'^\s+\("src/', ler('verificacao/mutantes.py'), re.M))
if n_mut < c['defeitos_plantados_minimos']: falha(f"defeitos plantados: {n_mut} < {c['defeitos_plantados_minimos']}")

# 3. Regras de varredura: quantidade e as obrigatórias pelo nome.
regras = re.findall(r'^\s+- id:\s+(\S+)', ler('nimbus-regras.yml'), re.M)
if len(regras) < c['regras_minimas']: falha(f"regras de varredura: {len(regras)} < {c['regras_minimas']}")
for r in c['regras_obrigatorias']:
    if r not in regras: falha(f'regra obrigatória ausente: {r}')

# 4. Módulos obrigatórios declarados.
lib = codigo(ler('src/lib.rs'))
for mod in c['modulos_obrigatorios']:
    if f'\npub mod {mod};' not in lib: falha(f'módulo obrigatório ausente: {mod}')

# 5. Código inseguro proibido; parada em estouro aritmético e abort em pânico na produção.
if not re.search(r'(?m)^#!\[(deny|forbid)\(unsafe_code\)\]\s*$', lib): falha('lib.rs não proíbe unsafe')
# Qualquer forma de afrouxar a proibição conta: #[allow(...)], #![allow(...)] (vale para o módulo inteiro),
# lista com outras lints, cfg_attr, expect e warn. E a palavra-chave `unsafe` só pode existir em memoria.rs.
FROUXO = re.compile(r'#!?\[[^\]]*\b(?:allow|expect|warn)\s*\([^)]*\bunsafe_code\b')
PALAVRA_UNSAFE = re.compile(r'\bunsafe\b(?!_code)')
liberados, fora_da_memoria = 0, []
for raiz_, _, arqs in os.walk(NUCLEO):
    if os.sep + 'target' in raiz_ or os.sep + '.git' in raiz_: continue
    for f in arqs:
        if not f.endswith('.rs'): continue
        caminho = os.path.join(raiz_, f); rel = os.path.relpath(caminho, NUCLEO)
        texto = codigo(open(caminho, encoding='utf-8').read())
        n = len(FROUXO.findall(texto))
        if rel == os.path.join('src', 'memoria.rs'): liberados += n
        elif n: fora_da_memoria.append(f'{rel} (liberação da proibição)')
        if rel != os.path.join('src', 'memoria.rs') and PALAVRA_UNSAFE.search(texto): fora_da_memoria.append(f'{rel} (unsafe)')
if liberados > c['unsafe_liberados_maximo']: falha(f'unsafe liberado em {liberados} lugares (máximo {c["unsafe_liberados_maximo"]})')
for r_ in sorted(set(fora_da_memoria)): falha(f'unsafe fora de src/memoria.rs: {r_}')
cargo = codigo(ler('Cargo.toml'))
rel = cargo[cargo.find('[profile.release]'):]
rel = rel[:rel.find('\n[', 1)] if '\n[' in rel[1:] else rel
if not re.search(r'(?m)^overflow-checks\s*=\s*true\s*$', rel) or not re.search(r'(?m)^panic\s*=\s*"abort"\s*$', rel): falha('perfil de produção sem overflow-checks = true ou panic = "abort"')

# 5b. Os conferidores são conferidos: os testes que plantam defeitos na catraca, no auditor e no verificar.sh
# têm de existir e ser chamados pelo verificar.sh. Apagar um deles seria tirar uma proteção sem ninguém ver.
ver = ler('verificar.sh')
for t_ in ('testar_catraca.py', 'testar_auditoria.py', 'testar_verificar.py'):
    if not os.path.isfile(os.path.join(NUCLEO, 'verificacao', t_)): falha(f'conferidor de conferidor ausente: verificacao/{t_}')
    if f'verificacao/{t_}' not in ver: falha(f'verificar.sh não roda verificacao/{t_}')
for padrao_ in (r'^precisa semgrep "', r'^precisa git "', r'^precisa python3 "', r'^set -eu$', r'^  echo "PARCIAL: ',
                r'^  exit 3$', r'python3 verificacao/mutantes\.py', r'^python3 verificacao/mutantes\.py --conferir ', r'^confere_identidade$', r'^\[ -z "\$\{CARGO_TARGET_DIR:-\}" \] \|\| falha ', r'^falha\(\) \{ .*exit 1; \}$'):
    if not re.search(padrao_, ver, re.M): falha(f'verificar.sh perdeu a falha fechada ({padrao_})')

# 6. Toda rota com limite de uso, dentro do teto.
aut = codigo(ler('src/autenticacao.rs'))
rotas = re.findall(r'Rota \{ metodo: "\w+", caminho: "([^"]+)",[^}]*por_minuto: (\d+) \}', aut)
todas = re.findall(r'Rota \{ metodo: "\w+", caminho: "([^"]+)"', aut)
if len(rotas) != len(todas) or not rotas: falha('existe rota sem limite de uso (por_minuto)')
teto = int(re.search(r'LIMITE_MAXIMO_POR_MINUTO: u32 = (\d+)', ler('src/autenticacao.rs')).group(1))
if teto > c['limite_maximo_por_minuto']: falha(f'teto de limite por rota subiu para {teto}')
for caminho, lim in rotas:
    if int(lim) > teto or int(lim) < 1: falha(f'rota {caminho} com limite fora do intervalo')

# 6b. Toda rota sem prova de titular (pública ou só com ficha) precisa de análise registrada.
analisadas = json.loads(ler('ROTAS_PUBLICAS_ANALISADAS.json'))['rotas']
for caminho, exig in re.findall(r'caminho: "([^"]+)", exigencia: Exigencia::(Publica|Ficha)', ler('src/autenticacao.rs')):
    a = analisadas.get(caminho)
    if not a or not all(a.get(k) for k in ('devolve', 'expoe', 'riscos', 'defesa')):
        falha(f'rota sem prova de titular sem análise registrada: {caminho}')

# 6c. Selo de liberação: portões obrigatórios intactos, três assinaturas no mínimo, sentinela confere o selo.
selo_rs = codigo(ler('src/selo.rs'))
portoes = re.findall(r'"(\w+)"', selo_rs.split('PORTOES_OBRIGATORIOS: &[&str] = &[')[1].split('];')[0])
for p_ in c['portoes_obrigatorios']:
    if p_ not in portoes: falha(f'portão obrigatório removido do selo: {p_}')
m_ = re.search(r'MINIMO_DE_ASSINATURAS: usize = (\d+)', selo_rs)
if not m_ or int(m_.group(1)) < 3: falha('selo aceita menos de três assinaturas')
sent = codigo(ler('src/sentinela.rs'))
if 'pub fn autoverificar_com_selo' not in sent or 'selo::conferir(' not in sent: falha('sentinela não confere o selo de liberação')

# 6e. Regras do repositório como código: a principal, todas as branches e as tags; nenhuma com passe livre.
EXIGIDAS_ = {'protecao-principal.json': {'deletion', 'non_fast_forward', 'required_linear_history', 'required_signatures', 'pull_request', 'required_status_checks'},
             'protecao-todas-as-branches.json': {'required_signatures'},
             'protecao-tags.json': {'deletion', 'update', 'non_fast_forward', 'required_signatures'}}
KIT_REGRAS_ = os.path.normpath(os.path.join(NUCLEO, '..', '..', '.github', 'regras'))  # quando o núcleo está dentro de um kit
for raiz_regras in [os.path.join(NUCLEO, '.github', 'regras')] + ([KIT_REGRAS_] if os.path.basename(os.path.dirname(NUCLEO)) == 'core' else []):
    for arq_, tipos_ in EXIGIDAS_.items():
        cam_ = os.path.join(raiz_regras, arq_)
        if not os.path.isfile(cam_): falha(f'regra do repositório ausente: {arq_}'); continue
        r_ = json.load(open(cam_, encoding='utf-8'))
        if r_.get('enforcement') != 'active' or r_.get('bypass_actors'): falha(f'{arq_}: regra não ativa ou com passe livre')
        faltam_ = tipos_ - {x['type'] for x in r_.get('rules', [])}
        if faltam_: falha(f'{arq_}: regras retiradas {sorted(faltam_)}')
    if 'for ARQ in "$DIR"/*.json' not in open(os.path.join(raiz_regras, 'aplicar_protecao.sh'), encoding='utf-8').read():
        falha('aplicar_protecao.sh não aplica todas as regras da pasta')

# 6f. Escudo nativo: quando o núcleo está num kit, os dois arquivos do escudo existem e mantêm as travas.
if os.path.basename(os.path.dirname(NUCLEO)) == 'core':
    KIT_ = os.path.dirname(os.path.dirname(NUCLEO))
    # Nome do produto, nao o nome do diretorio do kit. Derivar do basename reprovava a catraca
    # sempre que o repositorio era clonado com outro nome (nimbus-core em vez de nimbuschat).
    NOME_ = 'nimbuschat'
    for rel_, travas_ in ((f'apps/{NOME_}-android/escudo/EscudoNativo.kt', ['FLAG_SECURE', 'setRecentsScreenshotEnabled(false)', 'filterTouchesWhenObscured = true', 'IME_FLAG_NO_PERSONALIZED_LEARNING', 'clearPrimaryClip()', 'isClientSilenced', 'onCameraUnavailable', 'setIsStrongBoxBacked(true)', 'setUnlockedDeviceRequired(true)', 'esquecerChaveAoSair']),
                          (f'apps/{NOME_}-ios/Escudo/EscudoNativo.swift', ['id != .keyboard', 'UIPasteboard.general.items = []', 'isExcludedFromBackup = true', 'FileProtectionType.complete', 'videoDeviceInUseByAnotherClient', 'sceneCaptureState', 'tomarMicrofoneComExclusividade', 'options: [])', 'kSecAttrAccessibleWhenUnlockedThisDeviceOnly', 'apagarChavesDeInstalacaoAnterior', 'esquecerChaveAoSair'])):
        cam_ = os.path.join(KIT_, rel_)
        if not os.path.isfile(cam_): falha(f'escudo nativo ausente: {rel_}'); continue
        t_ = open(cam_, encoding='utf-8').read()
        for x_ in travas_:
            if x_ not in t_: falha(f'escudo nativo perdeu a trava {x_} em {rel_}')

# 6g. Fluxos do CI blindados (núcleo e kit): nada de gatilho que roda código de fora com segredos
# (pull_request_target, workflow_run) nem permissão "write-all" em lugar nenhum do arquivo — mas texto de
# PR/issue (`${{ github.event.* }}`, `${{ github.head_ref }}`) só é risco de injeção quando entra DIRETO no
# texto de um passo `run:` (a shell lê isso como comando); passado por `env:` é o jeito seguro e documentado
# pelo próprio GitHub (o runner define a variável, nunca reaproveita o texto como script) — por isso só o
# corpo de blocos `run:` é varrido para esses três, nunca o arquivo inteiro (achado em 13/10/2026: a varredura
# de arquivo inteiro reprovava `revisao-humana-para-ia.yml` por `${{ github.event.pull_request.number }}`
# usado com segurança dentro de `env:`, nos dois passos que chamam `gh api` — falso positivo, não injeção).
# Além disso: permissões declaradas no topo, ação sempre fixada por hash e checkout sem guardar credencial.
def _fluxos():
    pastas_ = [os.path.join(NUCLEO, '.github', 'workflows')]
    if os.path.basename(os.path.dirname(NUCLEO)) == 'core':
        pastas_.append(os.path.normpath(os.path.join(NUCLEO, '..', '..', '.github', 'workflows')))
    for pasta_ in pastas_:
        for wf_ in sorted(glob.glob(os.path.join(pasta_, '*.yml'))): yield wf_
def _corpo_dos_run(t_):
    """Só o texto de dentro de cada passo `run:` (onde a shell de fato lê o conteúdo como comando) — nunca o
    arquivo inteiro, para não confundir com um valor seguro passado por `env:`. Cobre `run: |`/`run: >`
    (com ou sem indicador de chomping `-`/`+`) e `run: comando na mesma linha`, com ou sem o traço de item de
    lista YAML (`- run: ...`) na mesma linha — as duas formas são comuns num passo de workflow."""
    linhas_ = t_.split('\n'); saida_ = []; i_ = 0
    while i_ < len(linhas_):
        m_ = re.match(r'^(\s*(?:-\s*)?)run:\s*([|>][+-]?)\s*(?:#.*)?$', linhas_[i_])
        if m_:
            indent_ = len(m_.group(1)); i_ += 1
            while i_ < len(linhas_) and (linhas_[i_].strip() == '' or len(linhas_[i_]) - len(linhas_[i_].lstrip(' ')) > indent_):
                saida_.append(linhas_[i_]); i_ += 1
            continue
        m2_ = re.match(r'^\s*(?:-\s*)?run:\s*(.+)$', linhas_[i_])
        if m2_: saida_.append(m2_.group(1))
        i_ += 1
    return '\n'.join(saida_)
for wf_ in _fluxos():
    t_ = open(wf_, encoding='utf-8').read(); n_ = os.path.basename(wf_)
    for proibido_ in ('pull_request_target', 'workflow_run', 'write-all'):
        if proibido_ in t_: falha(f'{n_}: proibido no CI: {proibido_}')
    corpo_run_ = _corpo_dos_run(t_)
    for proibido_ in ('${{ github.event.', '${{github.event.', '${{ github.head_ref'):
        if proibido_ in corpo_run_: falha(f'{n_}: proibido interpolado dentro de run: {proibido_}')
    topo_ = t_.split('\njobs:')[0]
    if not re.search(r'^permissions:', topo_, re.M): falha(f'{n_}: sem permissões declaradas no topo')
    for m_ in re.finditer(r'uses:\s*([^\s#]+)', t_):
        if not re.search(r'@[0-9a-f]{40}$', m_.group(1)): falha(f'{n_}: ação sem hash fixo: {m_.group(1)}')
    for m_ in re.finditer(r'uses:\s*actions/checkout@', t_):
        if 'persist-credentials: false' not in t_[m_.end():m_.end() + 200]: falha(f'{n_}: checkout guardando credencial')

# 6h. Texto invisível ou que inverte a leitura (ataque "Trojan Source"): proibido em todo arquivo de texto do
# núcleo e do kit. O revisor vê uma coisa e o compilador lê outra.
INVISIVEIS_ = re.compile('[\u202a-\u202e\u2066-\u2069\u200b-\u200f\u2060\ufeff\u00ad]')
_raizes_ = [NUCLEO] + ([os.path.normpath(os.path.join(NUCLEO, '..', '..'))] if os.path.basename(os.path.dirname(NUCLEO)) == 'core' else [])
for raiz_t_ in _raizes_:
    for dir_, subs_, arqs_ in os.walk(raiz_t_):
        subs_[:] = [x for x in subs_ if x not in ('target', '.git', '__pycache__')]
        for a_ in arqs_:
            if not a_.endswith(('.rs', '.py', '.sh', '.kt', '.swift', '.yml', '.yaml', '.toml', '.json', '.md', '.sql', '.xml')): continue
            cam_ = os.path.join(dir_, a_)
            try: txt_ = open(cam_, encoding='utf-8').read()
            except UnicodeDecodeError: falha(f'arquivo de texto que não é UTF-8: {os.path.relpath(cam_, raiz_t_)}'); continue
            if INVISIVEIS_.search(txt_.lstrip('\ufeff') if a_.endswith('.csv') else txt_):
                falha(f'caractere invisível ou de inversão de leitura em {os.path.relpath(cam_, raiz_t_)}')

# 6i. Cadeia de suprimentos do código: nada roda na compilação sem ser visto, nenhuma dependência é desviada.
if os.path.exists(os.path.join(NUCLEO, 'build.rs')): falha('build.rs proibido: roda código arbitrário na compilação')
CT_ = ler('Cargo.toml')
if re.search(r'^\[(patch|replace)', CT_, re.M): falha('Cargo.toml com [patch] ou [replace]: desvio de dependência')
if re.search(r'^\s*\w[\w-]*\s*=\s*\{[^}]*\b(git|path)\s*=', CT_, re.M): falha('dependência por git ou por caminho: só crates.io com Cargo.lock')
if re.search(r'^\s*build\s*=', CT_, re.M) or re.search(r'proc-macro\s*=\s*true', CT_): falha('script de compilação ou macro procedural no núcleo')
for cfg_ in ('.cargo/config.toml', '.cargo/config'):
    if os.path.exists(os.path.join(NUCLEO, cfg_)): falha(f'{cfg_} proibido: pode trocar a origem das dependências')
LK_ = ler('Cargo.lock')
if LK_.count('[[package]]') < 2 or 'crates.io-index' not in LK_: falha('Cargo.lock vazio ou sem as dependências travadas')
for bloco_ in LK_.split('[[package]]')[1:]:
    if 'source = "registry+https://github.com/rust-lang/crates.io-index"' in bloco_ and not re.search(r'^checksum = "[0-9a-f]{64}"$', bloco_, re.M):
        falha('pacote do crates.io sem checksum no Cargo.lock')
    if 'source = ' in bloco_ and 'registry+https://github.com/rust-lang/crates.io-index' not in bloco_:
        falha('pacote de origem diferente do crates.io no Cargo.lock')
GA_ = os.path.join(NUCLEO, '.gitattributes')
if os.path.exists(GA_) and re.search(r'\bfilter\s*=', open(GA_, encoding='utf-8').read()): falha('.gitattributes com filtro: roda programa ao clonar')
PC_ = os.path.join(NUCLEO, '.pre-commit-config.yaml')
if os.path.exists(PC_):
    for rev_ in re.findall(r'^\s*rev:\s*(\S+)', open(PC_, encoding='utf-8').read(), re.M):
        if not re.fullmatch(r'[0-9a-f]{40}', rev_): falha(f'gancho de pre-commit sem hash fixo: {rev_}')

# 6j. Nenhum link ou página web para abrir conteúdo fora do app (decisão do dono, 06/10/2026).
if os.path.exists(os.path.join(NUCLEO, 'src', 'envio.rs')) or re.search(r'^pub mod envio;', ler('src/lib.rs'), re.M):
    falha('módulo de link web (envio) voltou ao núcleo')
if re.search(r'caminho:\s*"/v1/envio', ler('src/autenticacao.rs')): falha('rota de link web voltou ao servidor')

# 6k. NimbusChat pessoal: sem histórico no servidor (decisão do dono, 06/10/2026). O app do NimbusChat não usa
# o módulo historico; a ponte do NimbusChat não o expõe. Tudo fica só no aparelho.
if os.path.basename(os.path.dirname(NUCLEO)) == 'core' and os.path.basename(os.path.dirname(os.path.dirname(NUCLEO))) == 'nimbuschat':
    KITC_ = os.path.dirname(os.path.dirname(NUCLEO))
    for dir_, subs_, arqs_ in os.walk(os.path.join(KITC_, 'apps')):
        for a_ in arqs_:
            if a_.endswith(('.kt', '.swift')) and re.search(r'(?i)\bhistorico\b', open(os.path.join(dir_, a_), encoding='utf-8').read()):
                falha(f'NimbusChat pessoal usando histórico no servidor: {os.path.relpath(os.path.join(dir_, a_), KITC_)}')
    cont_ = os.path.join(KITC_, 'docs', 'CONTRATO_DA_PONTE.md')
    if not os.path.isfile(cont_) or 'não expõe o módulo `historico`' not in open(cont_, encoding='utf-8').read():
        falha('contrato da ponte do NimbusChat sem a exclusão do histórico no servidor')

# 6d. CI do núcleo: ferramentas com versão fixada.
for wf in glob.glob(os.path.join(NUCLEO, '.github', 'workflows', '*.yml')):
    for l_ in open(wf, encoding='utf-8'):
        if re.search(r'cargo install (?![^\n]*--version)', l_) or re.search(r'pip install semgrep(?!==)', l_):
            falha(f'ferramenta sem versão fixada em {os.path.basename(wf)}: {l_.strip()}')

# 7. Banco do servidor (quando a pasta é informada): toda tabela com RLS ligada e forçada.
if len(sys.argv) > 1:
    esq = open(os.path.join(sys.argv[1], 'esquema.sql'), encoding='utf-8').read()
    for t in re.findall(r'CREATE TABLE (nimbus\.\w+)', esq):
        if f'ALTER TABLE {t} ENABLE ROW LEVEL SECURITY' not in esq or f'ALTER TABLE {t} FORCE ROW LEVEL SECURITY' not in esq:
            falha(f'tabela {t} sem RLS ligada e forçada')
    if 'NOBYPASSRLS' not in esq: falha('papel do app pode ignorar RLS')
    if not os.path.isfile(os.path.join(sys.argv[1], 'verificar_banco_na_partida.sql')): falha('sentinela do banco ausente')
    # Kit: o CI precisa ter governança e liberação, e a regra da branch, duas aprovações e assinatura.
    kit = os.path.dirname(os.path.dirname(os.path.abspath(sys.argv[1])))
    ci = os.path.join(kit, '.github', 'workflows', 'nimbus-kit-ci.yml')
    if os.path.isfile(ci):
        t_ = open(ci, encoding='utf-8').read()
        for trecho in ['  governanca:', '  liberacao:', 'needs: [verificacao, segredos, banco, catraca, procedencia, governanca]', 'environment: liberacao', 'vars.LIBERACAO_ATIVA', './verificar.sh --completo', 'git clone --depth 1 https://github.com/RustSec/advisory-db']:
            if trecho not in t_: falha(f'CI do kit sem: {trecho.strip()}')
        sent_ = os.path.join(kit, '.github', 'workflows', 'sentinela-diaria.yml')
        if not os.path.isfile(sent_) or 'git clone --depth 1 https://github.com/RustSec/advisory-db' not in open(sent_, encoding='utf-8').read():
            falha('sentinela diária sem a base de avisos do dia')
        # Ferramentas de CI com versão fixada: instalar "a mais nova" no CI é porta de ataque à cadeia de suprimentos.
        for wf in glob.glob(os.path.join(kit, '.github', 'workflows', '*.yml')):
            for l_ in open(wf, encoding='utf-8'):
                if re.search(r'cargo install (?![^\n]*--version)', l_) or re.search(r'pip install semgrep(?!==)', l_):
                    falha(f'ferramenta sem versão fixada em {os.path.basename(wf)}: {l_.strip()}')
        regra = json.load(open(os.path.join(kit, '.github', 'regras', 'protecao-principal.json'), encoding='utf-8'))
        tipos = {r_['type']: r_.get('parameters', {}) for r_ in regra['rules']}
        if regra.get('bypass_actors') or regra.get('enforcement') != 'active': falha('regra da branch com passe livre ou inativa')
        if 'required_signatures' not in tipos: falha('regra da branch sem commit assinado')
        # O piso de aprovações vive em CATRACA.json: baixá-lo é uma edição visível num arquivo protegido, nunca um
        # número solto no meio do conferidor.
        if 'pull_request' not in tipos: falha('regra da branch sem pedido de mudança obrigatório')
        minimo_aprovacoes = int(c.get('aprovacoes_minimas', 2))
        if tipos.get('pull_request', {}).get('required_approving_review_count', 0) < minimo_aprovacoes:
            falha(f'regra da branch com menos de {minimo_aprovacoes} aprovação(ões)')
        exigidas = {x['context'] for x in c.get('verificacoes_obrigatorias', [])}
        presentes = {x['context'] for x in tipos.get('required_status_checks', {}).get('required_status_checks', [])}
        if exigidas - presentes: falha(f'CI do kit sem verificação obrigatória: {sorted(exigidas - presentes)}')
        if not os.path.isfile(os.path.join(kit, '.github', 'workflows', 'sentinela-diaria.yml')): falha('sentinela diária ausente')
        ren = os.path.join(kit, 'renovate.json')
        if not os.path.isfile(ren): falha('configuração de atualizações (renovate.json) ausente')
        else:
            rj = json.load(open(ren, encoding='utf-8'))
            dias = int(str(rj.get('minimumReleaseAge', '0')).split()[0] or 0)
            if rj.get('automerge') is not False or dias < 14: falha('atualizações automáticas ou sem espera mínima de 14 dias')
        gov = os.path.join(kit, 'verificacao', 'portoes_de_governanca.py')
        if not os.path.isfile(gov) or 'administracao_dupla' not in open(gov, encoding='utf-8').read(): falha('conferidor de governança ausente ou sem administração dupla')
    else:
        falha('CI do kit ausente')

if falhas:
    print(f'CATRACA: REPROVADO ({len(falhas)} proteção(ões) a menos).'); sys.exit(1)
print('Catraca: nenhuma proteção a menos.')
