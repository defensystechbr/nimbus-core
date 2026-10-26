#!/usr/bin/env python3
"""Defeitos plantados contra o auditor de dependências. Cada caso monta um núcleo e uma base de avisos
falsos numa pasta temporária e confere o código de saída. Se o auditor deixar passar um caso, este teste reprova.
Uso: python3 verificacao/testar_auditoria.py [caminho do auditor a testar]
"""
import os, shutil, subprocess, sys, tempfile
AUDITOR = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(os.path.abspath(__file__)), 'auditar_dependencias.py')

def aviso(id_, patched, extra=''):
    return f'```toml\n[advisory]\nid = "{id_}"\npackage = "x"\ndate = "2026-01-01"\n{extra}\n[versions]\npatched = {patched}\n```\n\n# Aviso de teste\n'

def caso(nome, lock, avisos, esperado, base_git=True):
    t = tempfile.mkdtemp()
    try:
        os.makedirs(f'{t}/n/verificacao'); shutil.copy(AUDITOR, f'{t}/n/verificacao/a.py')
        open(f'{t}/n/Cargo.lock', 'w').write(lock)
        db = f'{t}/db'
        if avisos is not None:
            for (crate, arq), txt in avisos.items():
                os.makedirs(f'{db}/crates/{crate}', exist_ok=True); open(f'{db}/crates/{crate}/{arq}', 'w').write(txt)
            if base_git:
                subprocess.run(['git', 'init', '-q', db]); subprocess.run(['git', '-C', db, 'add', '.'])
                subprocess.run(['git', '-C', db, '-c', 'user.name=t', '-c', 'user.email=t@t', 'commit', '-qm', 'base'])
        r = subprocess.run([sys.executable, f'{t}/n/verificacao/a.py', db], capture_output=True, text=True)
        ok = (r.returncode != 0) if esperado == 'reprova' else (r.returncode == 0)
        print(f"  {'✔' if ok else '✘'} {nome}: saiu {r.returncode} (esperado: {esperado})")
        return ok
    finally:
        shutil.rmtree(t)

L = lambda *p: ''.join(f'[[package]]\nname = "{n}"\nversion = "{v}"\n\n' for n, v in p)
casos = [
    ('versão antiga vulnerável escondida atrás de uma duplicata segura', L(('lib', '0.9.0'), ('lib', '1.2.0')),
     {('lib', 'R-1.md'): aviso('R-1', '[">= 1.0.0"]')}, 'reprova'),
    ('aviso com a palavra withdrawn só em comentário', L(('lib', '0.9.0')),
     {('lib', 'R-2.md'): aviso('R-2', '[">= 1.0.0"]', '#withdrawn = "YYYY-MM-DD"')}, 'reprova'),
    ('base de avisos ausente', L(('lib', '0.9.0')), None, 'reprova'),
    ('base sem data conferível (não é clone git)', L(('lib', '0.9.0')),
     {('outra', 'R-3.md'): aviso('R-3', '[">= 1.0.0"]')}, 'reprova', False),
    ('pré-lançamento abaixo da correção', L(('lib', '1.0.0-rc.1')), {('lib', 'R-4.md'): aviso('R-4', '[">= 1.0.0"]')}, 'reprova'),
    ('aviso retirado de verdade não acusa', L(('lib', '0.9.0')),
     {('lib', 'R-5.md'): aviso('R-5', '[">= 1.0.0"]', 'withdrawn = "2026-02-01"')}, 'aprova'),
    ('versão corrigida aprova', L(('lib', '1.0.0')), {('lib', 'R-6.md'): aviso('R-6', '[">= 1.0.0"]')}, 'aprova'),
]
resultados = [caso(*c) for c in casos]
print(f'Auditor de dependências: {sum(resultados)} de {len(resultados)} casos corretos.')
sys.exit(0 if all(resultados) else 1)
