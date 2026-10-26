#!/usr/bin/env python3
"""Auditoria de vulnerabilidades das dependências, sem depender de rede.

Compara CADA entrada do Cargo.lock (inclusive versões duplicadas da mesma biblioteca) com a base de avisos
RustSec (advisory-db). Falha fechada:
  código 1 — alguma dependência em versão vulnerável;
  código 2 — a auditoria NÃO rodou (base ausente, base sem data, base velha ou Cargo.lock ausente).
Nunca devolve 0 sem ter auditado tudo.
Uso: python3 verificacao/auditar_dependencias.py [caminho/para/advisory-db]
Variáveis: ADVISORY_DB (caminho da base), IDADE_MAXIMA_DA_BASE_DIAS (padrão 7).
"""
import datetime, glob, json, os, re, subprocess, sys, tomllib

def riscos_aceitos(raiz):
    """Riscos aceitos com prazo, de verificacao/riscos_aceitos.json.

    Regras duras, para a exceção não virar porta:
      - casa por (crate, aviso) EXATO, nunca por crate nem por prefixo;
      - só vale para aviso SEM correção publicada: se houver versão em `patched`, a exceção é ignorada;
      - a justificativa tem de continuar citada no código apontado por `referencia`, senão não vale;
      - vencido `revisar_em`, não vale.
    """
    p = os.path.join(raiz, 'verificacao', 'riscos_aceitos.json')
    if not os.path.isfile(p): return {}
    dados = json.load(open(p, encoding='utf-8'))
    hoje = datetime.date.today()
    aceitos = {}
    for e in dados.get('aceitos', []):
        for campo in ('crate', 'aviso', 'motivo', 'referencia', 'revisar_em'):
            if not e.get(campo):
                print(f'✘ riscos_aceitos.json: entrada sem "{campo}": {e}'); sys.exit(2)
        try: vence = datetime.date.fromisoformat(e['revisar_em'])
        except ValueError:
            print(f'✘ riscos_aceitos.json: revisar_em inválido em {e["crate"]} {e["aviso"]}: {e["revisar_em"]}'); sys.exit(2)
        if hoje >= vence:
            print(f'  • risco aceito VENCIDO em {e["revisar_em"]}: {e["crate"]} {e["aviso"]} volta a reprovar')
            continue
        ref = os.path.join(raiz, e['referencia'].split(':')[0])
        if not os.path.isfile(ref) or e['aviso'] not in open(ref, encoding='utf-8').read():
            print(f'  • risco aceito sem justificativa no código ({e["referencia"]} não cita {e["aviso"]}): não vale')
            continue
        aceitos[(e['crate'], e['aviso'])] = e
    return aceitos

def versao(v):
    """Versão semver comparável: (maior, menor, correção, é_final, pré-lançamento)."""
    nucleo, _, pre = v.strip().partition('-')
    nums = [int(x) for x in re.findall(r'\d+', nucleo)][:3]
    nums += [0] * (3 - len(nums))
    return (*nums, 0 if pre else 1, pre)

def satisfaz(ver, faixa):
    """faixa no estilo Cargo/semver: '>= 0.9.8', '^0.3.1', '>= 0.8.4, < 0.9.0', '>= 0.1.0-rc.3'."""
    v = versao(ver)
    for parte in faixa.split(','):
        parte = parte.strip()
        m = re.match(r'(>=|<=|<|>|=|\^|~)?\s*([0-9][0-9A-Za-z.\-+]*)$', parte)
        if not m: raise ValueError(f'faixa não entendida: {faixa!r}')
        op, alvo = m.group(1) or '^', versao(m.group(2))
        if op == '^':
            # ^x.y.z: >= x.y.z e < próxima versão incompatível (regra do Cargo)
            if not v >= alvo: return False
            if alvo[0] > 0: teto = (alvo[0] + 1, 0, 0)
            elif alvo[1] > 0: teto = (0, alvo[1] + 1, 0)
            else: teto = (0, 0, alvo[2] + 1)
            if not v[:3] < teto: return False
        elif op == '~':
            if not (v >= alvo and v[:2] == alvo[:2]): return False
        elif op == '>=' and not v >= alvo: return False
        elif op == '<=' and not v <= alvo: return False
        elif op == '<' and not v < alvo: return False
        elif op == '>' and not v > alvo: return False
        elif op == '=' and not v == alvo: return False
    return True

def frente_toml(texto):
    m = re.search(r'```toml\s*\n(.*?)\n```', texto, re.S)
    if not m: raise ValueError('aviso sem bloco toml')
    return tomllib.loads(m.group(1))

def idade_da_base(db):
    r = subprocess.run(['git', '-C', db, 'log', '-1', '--format=%cI'], capture_output=True, text=True)
    if r.returncode != 0 or not r.stdout.strip(): return None
    data = datetime.datetime.fromisoformat(r.stdout.strip())
    return (datetime.datetime.now(datetime.timezone.utc) - data).days

def main():
    raiz = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    lock = os.path.join(raiz, 'Cargo.lock')
    db = sys.argv[1] if len(sys.argv) > 1 else os.environ.get('ADVISORY_DB', '/tmp/advisory-db')
    maximo = int(os.environ.get('IDADE_MAXIMA_DA_BASE_DIAS', '7'))
    if not os.path.isdir(os.path.join(db, 'crates')):
        print('✘ auditoria de dependências NÃO rodou: base de avisos não encontrada.')
        print('  Obtenha: git clone --depth 1 https://github.com/RustSec/advisory-db /tmp/advisory-db')
        return 2
    idade = idade_da_base(db)
    if idade is None:
        print('✘ auditoria NÃO rodou: não consigo saber a data da base (precisa ser um clone git do advisory-db).'); return 2
    if idade > maximo:
        print(f'✘ auditoria NÃO rodou: base de avisos com {idade} dias (máximo {maximo}). Atualize com git pull.'); return 2
    if not os.path.isfile(lock):
        print('✘ auditoria NÃO rodou: Cargo.lock não encontrado.'); return 2
    entradas = re.findall(r'name = "([^"]+)"\nversion = "([^"]+)"', open(lock).read())
    if not entradas:
        print('✘ auditoria NÃO rodou: Cargo.lock sem pacotes.'); return 2
    aceitos = riscos_aceitos(raiz)
    vulneraveis, aceitas_lidas, avisos_lidos = [], [], 0
    for crate, ver in entradas:
        for md in sorted(glob.glob(os.path.join(db, 'crates', crate, '*.md'))):
            dados = frente_toml(open(md, encoding='utf-8').read())
            adv, vers = dados.get('advisory', {}), dados.get('versions', {})
            avisos_lidos += 1
            if 'withdrawn' in adv: continue          # só o campo de verdade, nunca a palavra no texto
            seguro = any(satisfaz(ver, f) for f in vers.get('patched', []) + vers.get('unaffected', []))
            if seguro: continue
            ident = adv.get('id', '?')
            risco = aceitos.get((crate, ident))
            if risco and vers.get('patched'):
                # Há correção publicada: aceitar seria escolher a vulnerabilidade em vez do upgrade.
                print(f'  • risco aceito para {crate} {ident} IGNORADO: existe versão corrigida ({vers["patched"]}); atualize')
                risco = None
            if risco:
                aceitas_lidas.append((crate, ver, ident, risco['referencia'], risco['revisar_em']))
            else:
                vulneraveis.append((crate, ver, ident, adv.get('informational', 'vulnerabilidade')))
    for c, v, i, ref, vence in aceitas_lidas:
        print(f'  ↷ risco aceito: {c} {v} — {i} (justificativa em {ref}; revisar até {vence})')
    if vulneraveis:
        print('✘ Dependências em versão vulnerável:')
        for c, v, i, t in vulneraveis: print(f'  {c} {v} — {i} ({t})')
        return 1
    nomes = len({c for c, _ in entradas})
    print(f'Auditoria de dependências: {len(entradas)} versões ({nomes} bibliotecas), {avisos_lidos} avisos conferidos, '
          f'base de {idade} dia(s); nenhuma vulnerabilidade conhecida em aberto.')
    return 0

sys.exit(main())
