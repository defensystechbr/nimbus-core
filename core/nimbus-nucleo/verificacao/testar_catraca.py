#!/usr/bin/env python3
"""Defeitos plantados contra a própria catraca. Cada caso copia o núcleo para uma pasta temporária,
planta uma forma de afrouxar a segurança e exige que a catraca reprove. Se a catraca deixar passar
qualquer caso, este teste reprova. Uso: python3 verificacao/testar_catraca.py [núcleo a testar]
"""
import os, shutil, subprocess, sys, tempfile
NUCLEO = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FURO = '\npub fn furo(p: *const u8) -> u8 { unsafe { *p } }\n'

def no_fim(rel, texto):
    def f(d): open(os.path.join(d, rel), 'a', encoding='utf-8').write(texto)
    return f
def no_inicio(rel, texto):
    def f(d):
        p = os.path.join(d, rel); s = open(p, encoding='utf-8').read(); open(p, 'w', encoding='utf-8').write(texto + s)
    return f
def trocar(rel, de, para):
    def f(d):
        p = os.path.join(d, rel); s = open(p, encoding='utf-8').read()
        assert de in s, f'trecho não encontrado em {rel}: {de}'
        open(p, 'w', encoding='utf-8').write(s.replace(de, para, 1))
    return f

CASOS = [
    ('unsafe liberado no módulo inteiro (#![allow])', [no_inicio('src/panico.rs', '#![allow(unsafe_code)]\n'), no_fim('src/panico.rs', FURO)]),
    ('unsafe liberado em lista de lints', [no_fim('src/panico.rs', '\n#[allow(unsafe_code, dead_code)]' + FURO)]),
    ('unsafe liberado por cfg_attr', [no_fim('src/panico.rs', '\n#[cfg_attr(all(), allow(unsafe_code))]' + FURO)]),
    ('unsafe liberado por expect', [no_fim('src/panico.rs', '\n#[expect(unsafe_code)]' + FURO)]),
    ('unsafe liberado por warn', [no_fim('src/panico.rs', '\n#[warn(unsafe_code)]' + FURO)]),
    ('unsafe num teste de integração', [no_fim('tests/robustez.rs', FURO)]),
    ('proibição retirada do lib.rs', [trocar('src/lib.rs', '#![deny(unsafe_code)]', '')]),
    ('selo com duas assinaturas', [trocar('src/selo.rs', 'MINIMO_DE_ASSINATURAS: usize = 3', 'MINIMO_DE_ASSINATURAS: usize = 2')]),
    ('portão obrigatório removido', [trocar('src/selo.rs', '"revisao_de_duas_pessoas", "administracao_dupla",', '"revisao_de_duas_pessoas",')]),
    ('regra de varredura removida', [trocar('nimbus-regras.yml', '  - id: nimbus-sem-rede-mesh', '  - id: nimbus-sem-rede-mesh-desligada')]),
    ('teste da catraca apagado do verificar.sh', [trocar('verificar.sh', 'python3 verificacao/testar_catraca.py', 'true')]),
    ('varredura voltando a ser opcional', [trocar('verificar.sh', 'precisa semgrep', 'true || precisa semgrep')]),
    ('regra de tags apagada', [lambda d: os.remove(os.path.join(d, '.github', 'regras', 'protecao-tags.json'))]),
    ('tags sem proteção contra apagar', [trocar('.github/regras/protecao-tags.json', '"type": "deletion"', '"type": "creation"')]),
    ('passe livre na regra de todas as branches', [trocar('.github/regras/protecao-todas-as-branches.json', '"bypass_actors": []', '"bypass_actors": [{"actor_id": 1, "actor_type": "RepositoryRole", "bypass_mode": "always"}]')]),
    ('regra contra eval retirada', [trocar('nimbus-regras.yml', '  - id: nimbus-sem-eval-nas-ferramentas', '  - id: nimbus-sem-eval-desligada')]),
    ('regra contra log nativo retirada', [trocar('nimbus-regras.yml', '  - id: nimbus-nativo-sem-log', '  - id: nimbus-nativo-log-liberado')]),
    ('Trojan Source: inversão de leitura num comentário', [no_fim('src/panico.rs', '\n// texto \u202e invertido\n')]),
    ('caractere invisível num script', [no_fim('verificacao/auditar_dependencias.py', '\n# x\u200by\n')]),
    ('gatilho perigoso no CI (pull_request_target)', [trocar('.github/workflows/nimbus-ci.yml', 'on:', 'on:\n  pull_request_target:')]),
    ('checkout guardando credencial', [trocar('.github/workflows/nimbus-ci.yml', 'persist-credentials: false', 'persist-credentials: true')]),
    ('ação do CI sem hash fixo', [trocar('.github/workflows/nimbus-ci.yml', 'actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1', 'actions/checkout@v7')]),
    ('build.rs plantado', [lambda d: open(os.path.join(d, 'build.rs'), 'w').write('fn main() {}\n')]),
    ('dependência desviada por [patch]', [no_fim('Cargo.toml', '\n[patch.crates-io]\nsubtle = { git = "https://exemplo.invalid/subtle" }\n')]),
    ('origem das dependências trocada (.cargo/config.toml)', [lambda d: (os.makedirs(os.path.join(d, '.cargo'), exist_ok=True), open(os.path.join(d, '.cargo', 'config.toml'), 'w').write('[source.crates-io]\nreplace-with = "falso"\n'))]),
    ('pacote sem checksum no Cargo.lock', [trocar('Cargo.lock', 'checksum = ', 'xchecksum = ')]),
    ('Cargo.lock esvaziado', [lambda d: open(os.path.join(d, 'Cargo.lock'), 'w').write('')]),
    ('gancho de pre-commit sem hash', [trocar('.pre-commit-config.yaml', 'rev: 83d9cd684c87d95d656c1458ef04895a7f1cbd8e', 'rev: v8.30.1')]),
    ('módulo de link web de volta', [lambda d: open(os.path.join(d, 'src', 'envio.rs'), 'w').write('pub fn montar_link() {}\n')]),
    ('regra contra link web retirada', [trocar('nimbus-regras.yml', '  - id: nimbus-sem-link-web', '  - id: nimbus-link-web-liberado')]),
    ('parada por estouro desligada', [trocar('Cargo.toml', 'overflow-checks = true', 'overflow-checks = false')]),
    ('texto de evento de PR interpolado direto num passo run: (injeção real, não dentro de env:)', [no_fim('.github/workflows/nimbus-ci.yml', '\n  passo-de-teste-plantado:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: |\n          echo "${{ github.event.pull_request.title }}"\n')]),
]
falhas = 0
for nome, acoes in CASOS:
    t = tempfile.mkdtemp()
    try:
        d = os.path.join(t, 'n'); shutil.copytree(NUCLEO, d, ignore=shutil.ignore_patterns('target', '.git', '__pycache__'))
        try:
            for a in acoes: a(d)
        except AssertionError as e:  # caso que não consegue ser plantado também reprova: o teste ficou velho
            falhas += 1; print(f'  ✘ {nome}: caso não plantado ({e})'); continue
        r = subprocess.run([sys.executable, os.path.join(d, 'verificacao', 'catraca.py')], capture_output=True, text=True)
        pegou = r.returncode != 0
        falhas += not pegou
        print(f"  {'✔' if pegou else '✘'} {nome}: {'reprovado' if pegou else 'PASSOU SEM SER VISTO'}")
    finally:
        shutil.rmtree(t)
print(f'Catraca: {len(CASOS) - falhas} de {len(CASOS)} afrouxamentos pegos.')
sys.exit(1 if falhas else 0)
