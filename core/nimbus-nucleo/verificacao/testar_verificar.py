#!/usr/bin/env python3
"""Defeitos plantados contra o próprio verificar.sh: ferramentas falsas simulam cada situação e o teste exige
que o verificar.sh REPROVE (falha fechada). Não compila nada: roda em segundos.
Uso: python3 verificacao/testar_verificar.py [núcleo a testar]
"""
import os, shutil, stat, subprocess, sys, tempfile
NUCLEO = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
import re
# O piso de testes vem do próprio verificar.sh: o teste acompanha o piso sozinho quando ele sobe.
PISO = int(re.search(r'MINIMO_DE_TESTES=(\d+)', open(os.path.join(NUCLEO, 'verificar.sh')).read()).group(1))

CARGO_FALSO = r'''#!/bin/sh
case "$*" in
  *--version*) echo "cargo 1.91.1 (falso)";;
  *"--example"*) i=0; while [ $i -lt ${PASSOS:-34} ]; do echo "[$i] passo"; i=$((i+1)); done;;
  *"--test fuzzing"*) echo "test result: ok. ${FUZZ_OK:-1} passed";;
  *"--test timing"*|*"--test tempo_de_recusa"*) echo "test result: ok. 1 passed";;
  *"--no-run"*) exit 0;;
  *"test --lib"*) echo "test result: ok. 449 passed";;
  *test*) [ -n "${MUDAR_CODIGO:-}" ] && echo "// alterado durante a verificação" >> src/lib.rs; echo "test result: ok. ${TESTES:-PISO} passed";;
esac
exit 0
'''
def exe(caminho, texto):
    open(caminho, 'w').write(texto); os.chmod(caminho, os.stat(caminho).st_mode | stat.S_IEXEC)

def rodar(nome, esperado, env_extra=None, sem=(), args=(), motivo=''):
    t = tempfile.mkdtemp()
    try:
        d = os.path.join(t, 'n'); shutil.copytree(NUCLEO, d, ignore=shutil.ignore_patterns('target', '.git', '__pycache__'))
        # O contrato é conferido pelo verificar.sh. No layout antigo ele ficava dentro do próprio núcleo; no
        # repositório ele está na raiz, dois níveis acima. Sem copiá-lo para a cópia de trabalho, o verificar.sh
        # morria na conferência de citações — antes da etapa que cada caso quer plantar — e os nove casos
        # reprovavam pelo motivo errado. Achado ao rodar o teste depois da importação, 26/10/2026.
        for nome_ in ('CONTRATO_DA_PONTE.md',):
            origem_ = os.path.join(NUCLEO, nome_)
            if not os.path.isfile(origem_):
                origem_ = os.path.abspath(os.path.join(NUCLEO, '..', '..', nome_))
            if os.path.isfile(origem_): shutil.copy2(origem_, os.path.join(d, nome_))
        bin_ = os.path.join(t, 'bin'); os.makedirs(bin_)
        exe(os.path.join(bin_, 'cargo'), CARGO_FALSO.replace('PISO', str(PISO)))
        if 'semgrep' not in sem: exe(os.path.join(bin_, 'semgrep'), '#!/bin/sh\nexit 0\n')
        for f in ('python3', 'git', 'sh', 'awk', 'grep', 'sed', 'df', 'tail', 'cat', 'mktemp', 'dirname', 'mv', 'printf', 'timeout', 'echo'):
            p = shutil.which(f)
            if p and f not in sem: os.symlink(p, os.path.join(bin_, f))
        db = os.path.join(t, 'db')
        if 'base' not in sem:
            os.makedirs(os.path.join(db, 'crates', 'nada'))
            open(os.path.join(db, 'crates', 'nada', 'x.txt'), 'w').write('x')
            g = shutil.which('git')
            subprocess.run([g, 'init', '-q', db]); subprocess.run([g, '-C', db, 'add', '.'])
            subprocess.run([g, '-C', db, '-c', 'user.name=t', '-c', 'user.email=t@t', 'commit', '-qm', 'b'])
        env = {'PATH': bin_, 'HOME': t, 'TMPDIR': t, 'ADVISORY_DB': db, 'CARGO': 'cargo', 'NIMBUS_DENTRO_DO_TESTE_DO_VERIFICADOR': '1'}
        env.update(env_extra or {})
        r = subprocess.run(['/bin/sh', os.path.join(d, 'verificar.sh'), *args], env=env, capture_output=True, text=True, timeout=600)
        saida = r.stdout + r.stderr
        aprovou = r.returncode == 0 and 'Tudo certo' in saida
        certo = (not aprovou) if esperado == 'reprova' else aprovou
        if esperado == 'reprova' and motivo and motivo not in saida: certo = False  # reprovou, mas pelo motivo errado
        if esperado == 'parcial': certo = r.returncode == 3 and 'Tudo certo' not in saida and 'PARCIAL' in saida
        print(f"  {'✔' if certo else '✘'} {nome}: código {r.returncode}{' e disse Tudo certo' if 'Tudo certo' in saida else ''}")
        return certo
    finally:
        shutil.rmtree(t, ignore_errors=True)

casos = [
    rodar('sem semgrep instalado', 'reprova', sem=('semgrep',), motivo='semgrep não encontrado'),
    rodar('sem base de avisos', 'reprova', sem=('base',), motivo='NÃO rodou'),
    rodar('modo rápido não vale como verificação', 'parcial', args=('--rapido',)),
    rodar('fuzzing que não rodou nenhum teste', 'reprova', {'FUZZ_OK': '0'}, motivo='o fuzzing não rodou'),
    rodar('um teste a menos que o mínimo', 'reprova', {'TESTES': str(PISO - 1)}, motivo='testes rodaram'),
    rodar('exemplo de ponta a ponta encolhido', 'reprova', {'PASSOS': '20'}, motivo='passos (mínimo 33)'),
    rodar('mutação em que nenhum defeito é pego', 'reprova', motivo='VIVO'),
    rodar('pasta de compilação compartilhada com outra cópia', 'reprova', {'CARGO_TARGET_DIR': '/tmp/compartilhada'}, motivo='CARGO_TARGET_DIR definido'),
    rodar('código alterado no meio da verificação', 'reprova', {'MUDAR_CODIGO': '1'}, args=('--rapido',), motivo='o código mudou durante a verificação'),
]
print(f'verificar.sh: {sum(casos)} de {len(casos)} situações tratadas em falha fechada.')
sys.exit(0 if all(casos) else 1)
