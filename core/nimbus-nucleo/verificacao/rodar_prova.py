#!/usr/bin/env python3
"""Roda uma prova longa e grava um registro que carrega, DENTRO dele, a identidade do código medido.

O registro começa e termina com a identidade (SHA-256 de src, tests, examples, Cargo.toml e Cargo.lock), traz o
comando exato, a data de início e de fim (UTC) e o código de saída. Se o código mudar durante a prova, o registro
diz isso e a prova não vale. O medidor de provas só aceita registros neste formato.
Uso: python3 verificacao/rodar_prova.py <arquivo de registro> -- <comando...>
"""
import datetime, glob, hashlib, os, subprocess, sys

def identidade(nucleo):
    h = hashlib.sha256()
    arquivos = sorted(glob.glob(f'{nucleo}/src/*.rs') + glob.glob(f'{nucleo}/tests/*.rs') + glob.glob(f'{nucleo}/examples/*.rs')
                      + [f'{nucleo}/Cargo.toml', f'{nucleo}/Cargo.lock'])
    for f in arquivos:
        h.update(os.path.relpath(f, nucleo).encode() + b'\0' + open(f, 'rb').read() + b'\0')
    return h.hexdigest()

def agora(): return datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')

def main():
    if '--' not in sys.argv or len(sys.argv) < 4: print(__doc__); return 2
    registro = sys.argv[1]; cmd = sys.argv[sys.argv.index('--') + 1:]
    nucleo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if os.environ.get('CARGO_TARGET_DIR'): print('ERRO: CARGO_TARGET_DIR definido; a prova tem de usar a pasta de compilação desta cópia.'); return 2
    if glob.glob(f'{nucleo}/src/*.original'): print('ERRO: há um defeito plantado esquecido em src/ (arquivo .original); restaure antes.'); return 2
    antes = identidade(nucleo); inicio = agora()
    with open(registro, 'w', encoding='utf-8') as r:
        r.write(f'IDENTIDADE_INICIO: {antes}\nCOMANDO: {" ".join(cmd)}\nINICIO: {inicio}\n----\n'); r.flush()
        p = subprocess.run(cmd, cwd=nucleo, stdout=r, stderr=subprocess.STDOUT)
        depois = identidade(nucleo)
        r.write(f'\n----\nFIM: {agora()}\nCODIGO_DE_SAIDA: {p.returncode}\nIDENTIDADE_FIM: {depois}\n')
    if depois != antes: print('ERRO: o código mudou durante a prova; o registro não vale.'); return 1
    return p.returncode

if __name__ == '__main__':
    sys.exit(main())
