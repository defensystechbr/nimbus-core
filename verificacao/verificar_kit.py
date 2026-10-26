#!/usr/bin/env python3
"""Verificação de integridade do próprio kit, sem depender de nada externo.
Confere que o núcleo, o manual e o protótipo do kit estão presentes e que a consistência interna passa.
Uso: python3 verificacao/verificar_kit.py  (a partir da raiz do kit)"""
import os, sys, subprocess, hashlib
raiz = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
falhas = []
def ok(m): print(f'  \u2713 {m}')
def falha(m): falhas.append(m); print(f'  \u2717 {m}')
essenciais = ['core/nimbus-nucleo/src/lib.rs', 'core/nimbus-nucleo/verificar.sh', 'verificar.sh',
              'docs/PROTOTIPO.html', 'docs/TELAS.md', 'docs/referencia/MANUAL_TECNICO_COMPLETO.docx',
              'docs/roteiro-aparelho-real.md']
for e in essenciais:
    ok(f'presente: {e}') if os.path.isfile(f'{raiz}/{e}') else falha(f'FALTA: {e}')
r = subprocess.run([sys.executable, 'verificacao/consistencia.py'], cwd=raiz, capture_output=True, text=True)
ok('consistência interna (telas × protótipo, funções × núcleo, termos)') if r.returncode == 0 else falha(f'consistência: {r.stdout.strip()}')
print()
sys.exit(1 if falhas else 0)
