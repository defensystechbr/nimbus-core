#!/usr/bin/env python3
"""Verificador do pacote final, byte a byte e automático.

Extrai o NimbusOne_Pacote_Final.zip num diretório temporário e confere, sem intervenção humana:
  1. Que o núcleo dentro de cada kit é idêntico, byte a byte, ao nimbus-nucleo.zip avulso.
  2. Que o manual, o protótipo e as regras de varredura são os mesmos arquivos em toda parte.
  3. Que os dois kits passam na própria verificação de consistência (telas × protótipo, termos proibidos).
  4. Que nenhum artefato pesado (pasta target, arquivos .original) foi empacotado sem querer.
  5. Que não sobrou nenhum termo de versão superada.

Uso: python3 verificar_pacote.py [caminho/para/NimbusOne_Pacote_Final.zip]
Sai com código 0 se tudo bate; 1 na primeira divergência. Não altera nada.
"""
import sys, os, zipfile, tempfile, hashlib, subprocess, shutil, re, glob

SAIDA = os.environ.get('OUTPUTS', '/mnt/user-data/outputs')
PACOTE = sys.argv[1] if len(sys.argv) > 1 else os.path.join(SAIDA, 'NimbusOne_Pacote_Final.zip')
# Termos que indicam material de uma versão anterior deixado para trás (fora do histórico de auditoria).
TERMOS_SUPERADOS = [r'\b334 testes\b', r'\b338 testes\b', r'1 milhão', r'Manual Técnico v(29|3[0-6])\b', r'55 telas\b']

falhas = []
def ok(msg): print(f'  \u2713 {msg}')
def falha(msg): falhas.append(msg); print(f'  \u2717 {msg}')

def sha(caminho):
    h = hashlib.sha256()
    with open(caminho, 'rb') as f:
        for bloco in iter(lambda: f.read(65536), b''): h.update(bloco)
    return h.hexdigest()

def arvore_hashes(raiz):
    """sha256 de cada arquivo sob raiz, com caminho relativo — a impressão digital da árvore."""
    r = {}
    for base, _, arquivos in os.walk(raiz):
        for nome in arquivos:
            caminho = os.path.join(base, nome)
            r[os.path.relpath(caminho, raiz)] = sha(caminho)
    return r

print(f'Verificação do pacote: {PACOTE}')
if not os.path.isfile(PACOTE):
    print('  pacote não encontrado'); sys.exit(1)

tmp = tempfile.mkdtemp(prefix='verif-pacote-')
try:
    with zipfile.ZipFile(PACOTE) as z: z.extractall(f'{tmp}/p')
    p = f'{tmp}/p'
    # desempacota os três zips internos
    for interno, destino in [('NimbusChat_Kit_ClaudeCode.zip', 'c'), ('NimbusOne_Kit_ClaudeCode.zip', 'o'), ('nimbus-nucleo.zip', 'n')]:
        caminho = os.path.join(p, interno)
        if not os.path.isfile(caminho): falha(f'falta {interno} no pacote'); continue
        with zipfile.ZipFile(caminho) as z: z.extractall(f'{tmp}/{destino}')

    nucleo_avulso = f'{tmp}/n/nimbus-nucleo'
    kit_chat = f'{tmp}/c/nimbuschat'
    kit_one = f'{tmp}/o/nimbusone'

    # 1. núcleo idêntico byte a byte nos dois kits e no avulso
    if os.path.isdir(nucleo_avulso):
        base = arvore_hashes(nucleo_avulso)
        for nome, kit in [('NimbusChat', kit_chat), ('NimbusOne', kit_one)]:
            alvo_nucleo = f'{kit}/core/nimbus-nucleo'
            if not os.path.isdir(alvo_nucleo): falha(f'{nome}: núcleo ausente'); continue
            # o kit acrescenta docs/roteiro-aparelho-real.md ao núcleo; comparo o resto byte a byte
            k = arvore_hashes(alvo_nucleo)
            extras_esperados = {'docs/roteiro-aparelho-real.md'}
            faltando = set(base) - set(k)
            divergentes = [f for f in base if f in k and base[f] != k[f]]
            inesperados = set(k) - set(base) - extras_esperados
            if faltando: falha(f'{nome}: {len(faltando)} arquivo(s) do núcleo faltando: {sorted(faltando)[:3]}')
            elif divergentes: falha(f'{nome}: {len(divergentes)} arquivo(s) do núcleo diferentes: {sorted(divergentes)[:3]}')
            elif inesperados: falha(f'{nome}: arquivo(s) inesperado(s) no núcleo: {sorted(inesperados)[:3]}')
            else: ok(f'{nome}: núcleo idêntico byte a byte ao avulso ({len(base)} arquivos)')
    else:
        falha('nimbus-nucleo.zip não pôde ser lido')

    # 2. manual, protótipo, regras e verificar.sh idênticos em toda parte
    for rotulo, no_pacote, nos_kits in [
        ('manual', (sorted(os.path.basename(m) for m in glob.glob(os.path.join(p, 'NimbusOne_Manual_Tecnico_v*.docx'))) or ['NimbusOne_Manual_Tecnico_ausente.docx'])[-1], 'docs/referencia/MANUAL_TECNICO_COMPLETO.docx'),
        ('protótipo', 'NimbusOne_Prototipo.html', 'docs/PROTOTIPO.html'),
    ]:
        alvo = os.path.join(p, no_pacote)
        if not os.path.isfile(alvo): falha(f'{rotulo}: {no_pacote} ausente no pacote'); continue
        h = sha(alvo)
        iguais = all(os.path.isfile(f'{kit}/{nos_kits}') and sha(f'{kit}/{nos_kits}') == h for kit in [kit_chat, kit_one])
        ok(f'{rotulo}: idêntico no pacote e nos dois kits') if iguais else falha(f'{rotulo}: difere entre pacote e algum kit')

    # verificar.sh idêntico nos dois kits (raiz) e no núcleo de cada um
    vers = [f'{kit_chat}/verificar.sh', f'{kit_one}/verificar.sh']
    if all(os.path.isfile(v) for v in vers):
        ok('verificar.sh idêntico nos dois kits') if sha(vers[0]) == sha(vers[1]) else falha('verificar.sh difere entre os kits')

    # 3. cada kit passa na própria consistência
    for nome, kit in [('NimbusChat', kit_chat), ('NimbusOne', kit_one)]:
        r = subprocess.run([sys.executable, 'verificacao/consistencia.py'], cwd=kit, capture_output=True, text=True)
        ok(f'{nome}: consistência interna (telas × protótipo, termos)') if r.returncode == 0 else falha(f'{nome}: consistência falhou — {r.stdout.strip()} {r.stderr.strip()}')

    # guias visuais: no pacote e dentro do kit de cada produto, idênticos
    for nome, kit in [('NimbusChat', kit_chat), ('NimbusOne', kit_one)]:
        g_pac = os.path.join(p, f'Guia_Visual_{nome}.pdf'); g_kit = f'{kit}/docs/Guia_Visual_{nome}.pdf'
        if not (os.path.isfile(g_pac) and os.path.isfile(g_kit)): falha(f'{nome}: Guia Visual ausente no pacote ou no kit')
        elif sha(g_pac) != sha(g_kit): falha(f'{nome}: Guia Visual difere entre pacote e kit')
        else: ok(f'{nome}: Guia Visual idêntico no pacote e no kit')

    # pacotes por produto: o kit e o guia dentro de cada um são os mesmos do pacote completo
    for nome in ['NimbusChat', 'NimbusOne']:
        pp = os.path.join(os.path.dirname(PACOTE), f'{nome}_Pacote.zip')
        if not os.path.isfile(pp): falha(f'{nome}_Pacote.zip não encontrado'); continue
        d = f'{tmp}/prod_{nome}'
        with zipfile.ZipFile(pp) as z: z.extractall(d)
        itens = {f'{nome}_Kit_ClaudeCode.zip', f'Guia_Visual_{nome}.pdf', f'LEIA-ME_{nome}.md'}
        faltam = [i for i in itens if not os.path.isfile(f'{d}/{i}')]
        if faltam: falha(f'{nome}_Pacote.zip sem: {faltam}'); continue
        iguais = all(sha(f'{d}/{i}') == sha(os.path.join(p, i)) for i in itens if os.path.isfile(os.path.join(p, i)))
        ok(f'{nome}_Pacote.zip: kit, guia e leia-me idênticos ao pacote completo') if iguais else falha(f'{nome}_Pacote.zip difere do pacote completo')

    # 3b. entrega aos técnicos: os mesmos arquivos no pacote e dentro de cada kit
    for nome, kit in [('NimbusChat', kit_chat), ('NimbusOne', kit_one)]:
        docs_tec = ['COMECE_AQUI_TECNICOS.md', 'PENDENCIAS_E_DECISOES.md', 'NimbusOne_Engenharia_Social_e_Seguranca_Operacional.md', 'NimbusOne_Ataques_com_IA_e_Defesa_em_Profundidade.md', 'NimbusOne_Checklist_de_Seguranca_do_Backend.md', 'REGISTRO_E_POLITICA_PERMANENTE_DE_SEGURANCA.md', 'ACESSO_SEM_AUTENTICACAO.md', 'PERGUNTAS_DE_TESTE_E_VERIFICACAO.md', 'MAPA_ASVS_5.0.csv', 'GUIA_DE_LIBERACAO.md', 'NimbusOne_Metadados_O_Que_Sobra.md', 'CHECKLIST_DE_ENTRADA_EM_PRODUCAO.md', '01_PRD_Nimbus.md', '02_TRD_Nimbus.md', '03_Fluxo_do_App_Nimbus.md', '04_UI_UX_Design_Nimbus.md', '05_Esquema_Backend_Nimbus.md', '06_Plano_de_Implementacao_Nimbus.md', 'NimbusOne_Revisao_de_Fechamento.md', 'NimbusOne_Relatorio_de_Ataque.md', 'CONTRATO_DA_PONTE.md', 'API_DO_NUCLEO.md',
                    'MATRIZ_DE_RASTREABILIDADE.md', 'MATRIZ_DE_RASTREABILIDADE.csv']
        dif = [d for d in docs_tec if not (os.path.isfile(os.path.join(p, d)) and os.path.isfile(f'{kit}/docs/{d}')
               and sha(os.path.join(p, d)) == sha(f'{kit}/docs/{d}'))]
        ok(f'{nome}: entrega aos técnicos idêntica no pacote e no kit') if not dif else falha(f'{nome}: entrega aos técnicos difere ou falta: {dif}')

    # 3c. banco do servidor idêntico nos dois kits
    bancos = [arvore_hashes(f'{k}/servidor/banco') if os.path.isdir(f'{k}/servidor/banco') else None for k in (kit_chat, kit_one)]
    ok('banco do servidor (esquema e testes) idêntico nos dois kits') if bancos[0] and bancos[0] == bancos[1] else falha('banco do servidor ausente ou diferente entre os kits')

    # 3d. proteções de repositório presentes nos dois kits e no núcleo
    for nome, raiz_ in [('NimbusChat', kit_chat), ('NimbusOne', kit_one), ('núcleo', nucleo_avulso)]:
        faltam = [a for a in ['.gitignore', '.gitleaks.toml', '.pre-commit-config.yaml', 'SECURITY.md', '.github/CODEOWNERS', '.github/regras/protecao-principal.json', '.github/workflows/vigia-da-protecao.yml', 'renovate.json'] if not os.path.isfile(f'{raiz_}/{a}')]
        ok(f'{nome}: proteções de repositório presentes') if not faltam else falha(f'{nome}: faltam {faltam}')

    # 4. nada de target/ nem .original empacotado
    lixo = []
    for kit in [kit_chat, kit_one, nucleo_avulso]:
        for base_dir, dirs, arquivos in os.walk(kit):
            if 'target' in dirs: lixo.append(f'{os.path.relpath(base_dir, tmp)}/target')
            lixo += [os.path.relpath(os.path.join(base_dir, a), tmp) for a in arquivos if a.endswith('.original')]
    ok('nenhum artefato pesado empacotado (sem target/ nem .original)') if not lixo else falha(f'artefatos indevidos: {lixo[:3]}')

    # 5. termos de versão superada (fora das linhas de histórico de auditoria AUDxx)
    superados = []
    for base_dir, _, arquivos in os.walk(p):
        for nome in arquivos:
            if not nome.endswith(('.md', '.txt', '.sh', '.yml', '.toml', '.rs')): continue
            caminho = os.path.join(base_dir, nome)
            try: linhas = open(caminho, encoding='utf-8').read().split('\n')
            except (UnicodeDecodeError, IsADirectoryError): continue
            for l in linhas:
                if 'AUD' in l: continue  # histórico de auditoria cita números antigos de propósito
                for termo in TERMOS_SUPERADOS:
                    if re.search(termo, l): superados.append(f'{os.path.relpath(caminho, p)}: {termo}')
    ok('nenhum termo de versão superada nos textos do pacote') if not superados else falha(f'termos superados: {superados[:4]}')

finally:
    shutil.rmtree(tmp, ignore_errors=True)

print()
if falhas:
    print(f'REPROVADO: {len(falhas)} divergência(s).'); sys.exit(1)
print('Pacote íntegro: tudo bate byte a byte.'); sys.exit(0)
