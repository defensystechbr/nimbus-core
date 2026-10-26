#!/usr/bin/env python3
"""Portões de governança do selo de liberação, conferidos na API do GitHub (não na palavra de ninguém).

  protecao_do_repositorio  — TODAS as regras do código (.github/regras/*.json: branch principal, todas as branches e
                             tags) estão ativas e iguais, com o mesmo alvo e as mesmas branches/tags, sem passe livre
  commits_assinados        — o commit que vai virar artefato tem assinatura verificada pelo GitHub
  revisao_de_duas_pessoas  — o pedido de mudança desse commit teve pelo menos 2 aprovações finais de pessoas
                             diferentes do autor
  administracao_dupla      — a organização exige segundo fator para todos, tem pelo menos 2 administradores e a
                             equipe de segurança tem pelo menos 2 membros (ninguém sozinho nela)
                             LIMITE: a API do GitHub não diz QUAL segundo fator cada pessoa usa; "chave física" é
                             conferida por processo (GUIA_DE_LIBERACAO.md), não por esta verificação.

Saída (stdout): "protecao_do_repositorio=1,commits_assinados=1,revisao_de_duas_pessoas=1" (0 quando reprova).
Os motivos de cada reprovação vão para stderr. Sai com código 1 se qualquer portão reprovar.
Uso no CI: GH_TOKEN (leitura do repositório e dos pedidos) e PROTECAO_LEITURA (leitura de administração).
Uso em teste: python3 portoes_de_governanca.py --teste
"""
import json, os, sys, urllib.request

def comparar_protecao(ativa, esperada):
    erros = []
    if ativa.get('enforcement') != 'active': erros.append('regra não está ativa')
    if ativa.get('bypass_actors'): erros.append('existe quem possa passar por cima da regra')
    # Alvo e condições têm de ser exatamente os do código: uma regra completa apontando para outra branch
    # (ou para tags em vez de branches) protege o nada.
    if ativa.get('target', 'branch') != esperada.get('target', 'branch'): erros.append('alvo da regra diferente do código')
    ref_a = ((ativa.get('conditions') or {}).get('ref_name') or {})
    ref_e = ((esperada.get('conditions') or {}).get('ref_name') or {})
    if sorted(ref_a.get('include', [])) != sorted(ref_e.get('include', [])) or sorted(ref_a.get('exclude', [])) != sorted(ref_e.get('exclude', [])):
        erros.append('a regra protege branches ou tags diferentes das do código')
    tipos = {r['type']: r.get('parameters', {}) for r in ativa.get('rules', [])}
    for r in esperada['rules']:
        t = r['type']
        if t not in tipos: erros.append(f'regra ausente: {t}'); continue
        for k, v in r.get('parameters', {}).items():
            if k == 'required_status_checks':
                faltam = {c['context'] for c in v} - {c['context'] for c in tipos[t].get(k, [])}
                if faltam: erros.append(f'verificações obrigatórias ausentes: {sorted(faltam)}')
            elif k == 'required_approving_review_count':
                if tipos[t].get(k, 0) < v: erros.append(f'aprovações abaixo de {v}')
            elif tipos[t].get(k) != v: erros.append(f'{t}.{k} diferente do código')
    return erros

def aprovacoes_validas(revisoes, autor):
    ultima = {}
    for r in revisoes:  # a API devolve em ordem cronológica; vale a última de cada pessoa
        u = (r.get('user') or {}).get('login')
        if (r.get('user') or {}).get('type') != 'User': continue  # robô ou aplicativo não conta como pessoa
        if u and u != autor and r.get('state') in ('APPROVED', 'CHANGES_REQUESTED', 'DISMISSED'): ultima[u] = r['state']
    return sorted(u for u, s in ultima.items() if s == 'APPROVED')

def comparar_todas(pares):
    """pares = [(nome, ativa, esperada)]. Toda regra do código (.github/regras/*.json) tem de estar ativa e igual."""
    erros = []
    for nome, ativa, esperada in pares:
        e = comparar_protecao(ativa, esperada) if ativa else ['regra ausente']
        erros += [f'{nome}: {x}' for x in e]
    return erros

def avaliar_administracao(org, admins, equipe_seguranca):
    motivos = []
    if not org or not org.get('two_factor_requirement_enabled'): motivos.append('a organização não exige segundo fator para todos os membros')
    if len(admins or []) < 2: motivos.append(f'só {len(admins or [])} administrador(es) da organização (mínimo 2)')
    if len(equipe_seguranca or []) < 2: motivos.append(f'equipe de segurança com {len(equipe_seguranca or [])} membro(s) (mínimo 2)')
    return motivos

def avaliar(ativa, esperada, commit, pedidos, revisoes, org=None, admins=None, equipe_seguranca=None):
    res, motivos = {}, []
    e = comparar_protecao(ativa, esperada) if ativa else ['regra de proteção ausente']
    res['protecao_do_repositorio'] = not e; motivos += e
    ok = bool(((commit or {}).get('commit') or {}).get('verification', {}).get('verified'))
    res['commits_assinados'] = ok
    if not ok: motivos.append('commit sem assinatura verificada')
    pr = next((p for p in (pedidos or []) if p.get('merged_at')), None)
    if not pr: res['revisao_de_duas_pessoas'] = False; motivos.append('commit não veio de pedido de mudança aprovado')
    elif not (pr.get('user') or {}).get('login'):
        res['revisao_de_duas_pessoas'] = False; motivos.append('pedido de mudança sem autor identificável (falha fechada)')
    else:
        ap = aprovacoes_validas(revisoes or [], (pr.get('user') or {}).get('login'))
        res['revisao_de_duas_pessoas'] = len(ap) >= 2
        if len(ap) < 2: motivos.append(f'só {len(ap)} aprovação(ões) final(is) de pessoa diferente do autor')
    m = avaliar_administracao(org, admins, equipe_seguranca)
    res['administracao_dupla'] = not m; motivos += m
    return res, motivos

def api_todas(caminho, token):
    """Todas as páginas (o atacante não esconde uma revisão depois da página 1)."""
    itens, pagina = [], 1
    while True:
        sep = '&' if '?' in caminho else '?'
        lote = api(f'{caminho}{sep}per_page=100&page={pagina}', token)
        itens += lote
        if len(lote) < 100 or pagina >= 50: return itens
        pagina += 1

def api(caminho, token):
    req = urllib.request.Request('https://api.github.com' + caminho, headers={'Authorization': f'Bearer {token}', 'Accept': 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28'})
    with urllib.request.urlopen(req, timeout=30) as r: return json.load(r)

def teste():
    esperada = {'target': 'branch', 'conditions': {'ref_name': {'include': ['refs/heads/principal'], 'exclude': []}}, 'rules': [{'type': 'required_signatures'}, {'type': 'pull_request', 'parameters': {'required_approving_review_count': 2, 'require_last_push_approval': True}}]}
    ativa_ok = {'enforcement': 'active', 'bypass_actors': [], 'target': 'branch', 'conditions': {'ref_name': {'include': ['refs/heads/principal'], 'exclude': []}}, 'rules': [{'type': 'required_signatures'}, {'type': 'pull_request', 'parameters': {'required_approving_review_count': 2, 'require_last_push_approval': True}}]}
    commit_ok = {'commit': {'verification': {'verified': True}}}
    pr = [{'merged_at': '2026-10-02', 'user': {'login': 'autor'}}]
    rev_ok = [{'user': {'login': 'ana', 'type': 'User'}, 'state': 'APPROVED'}, {'user': {'login': 'bia', 'type': 'User'}, 'state': 'APPROVED'}]
    org_ok, adm_ok, seg_ok = {'two_factor_requirement_enabled': True}, [{'login': 'x'}, {'login': 'y'}], [{'login': 'ana'}, {'login': 'bia'}]
    adm = (org_ok, adm_ok, seg_ok)
    casos = [
        ('tudo certo', (ativa_ok, esperada, commit_ok, pr, rev_ok) + adm, True),
        ('organização sem segundo fator obrigatório', (ativa_ok, esperada, commit_ok, pr, rev_ok, {'two_factor_requirement_enabled': False}, adm_ok, seg_ok), False),
        ('um administrador só', (ativa_ok, esperada, commit_ok, pr, rev_ok, org_ok, [{'login': 'x'}], seg_ok), False),
        ('uma pessoa sozinha na segurança', (ativa_ok, esperada, commit_ok, pr, rev_ok, org_ok, adm_ok, [{'login': 'ana'}]), False),
        ('sem regra', (None, esperada, commit_ok, pr, rev_ok) + adm, False),
        ('regra em teste, não ativa', (dict(ativa_ok, enforcement='evaluate'), esperada, commit_ok, pr, rev_ok) + adm, False),
        ('passe livre', (dict(ativa_ok, bypass_actors=[{'actor_id': 1}]), esperada, commit_ok, pr, rev_ok) + adm, False),
        ('commit sem assinatura', (ativa_ok, esperada, {'commit': {'verification': {'verified': False}}}, pr, rev_ok) + adm, False),
        ('sem pedido de mudança', (ativa_ok, esperada, commit_ok, [], rev_ok) + adm, False),
        ('autor aprovando a si mesmo', (ativa_ok, esperada, commit_ok, pr, [{'user': {'login': 'autor', 'type': 'User'}, 'state': 'APPROVED'}, {'user': {'login': 'ana', 'type': 'User'}, 'state': 'APPROVED'}]) + adm, False),
        ('mesma pessoa duas vezes', (ativa_ok, esperada, commit_ok, pr, [{'user': {'login': 'ana', 'type': 'User'}, 'state': 'APPROVED'}, {'user': {'login': 'ana', 'type': 'User'}, 'state': 'APPROVED'}]) + adm, False),
        ('pedido sem autor identificável', (ativa_ok, esperada, commit_ok, [{'merged_at': '2026-10-02', 'user': None}], rev_ok) + adm, False),
        ('aprovou e depois pediu mudança', (ativa_ok, esperada, commit_ok, pr, rev_ok + [{'user': {'login': 'bia', 'type': 'User'}, 'state': 'CHANGES_REQUESTED'}]) + adm, False),
        ('regra completa apontando para outra branch', (dict(ativa_ok, conditions={'ref_name': {'include': ['refs/heads/outra'], 'exclude': []}}), esperada, commit_ok, pr, rev_ok) + adm, False),
        ('regra com exceção para a principal', (dict(ativa_ok, conditions={'ref_name': {'include': ['refs/heads/principal'], 'exclude': ['refs/heads/principal']}}), esperada, commit_ok, pr, rev_ok) + adm, False),
        ('alvo trocado para tags', (dict(ativa_ok, target='tag'), esperada, commit_ok, pr, rev_ok) + adm, False),
        ('aprovação de robô contando como pessoa', (ativa_ok, esperada, commit_ok, pr, [rev_ok[0], {'user': {'login': 'revisor[bot]', 'type': 'Bot'}, 'state': 'APPROVED'}]) + adm, False),
    ]
    falhas = 0
    for nome, args, esperado in casos:
        res, _ = avaliar(*args)
        if all(res.values()) != esperado: print(f'FALHOU: {nome}'); falhas += 1
    # Regras extras (todas as branches, tags): ausente ou afrouxada reprova.
    tags = {'target': 'tag', 'conditions': {'ref_name': {'include': ['~ALL'], 'exclude': []}}, 'rules': [{'type': 'deletion'}, {'type': 'update'}]}
    tags_ok = dict(tags, enforcement='active', bypass_actors=[])
    for nome, pares, esperado in [('regras extras em dia', [('tags', tags_ok, tags)], True),
                                  ('regra de tags ausente', [('tags', None, tags)], False),
                                  ('tags sem proteção contra apagar', [('tags', dict(tags_ok, rules=[{'type': 'update'}]), tags)], False)]:
        casos.append(nome)
        if (not comparar_todas(pares)) != esperado: print(f'FALHOU: {nome}'); falhas += 1
    print(f'Portões de governança: {len(casos) - falhas} de {len(casos)} casos corretos.')
    sys.exit(1 if falhas else 0)

if __name__ == '__main__':
    if '--teste' in sys.argv: teste()
    repo, sha = os.environ['GITHUB_REPOSITORY'], os.environ['GITHUB_SHA']
    tok, tok_adm = os.environ['GH_TOKEN'], os.environ.get('PROTECAO_LEITURA', '')
    import glob
    esperada = json.load(open('.github/regras/protecao-principal.json'))
    extras = [json.load(open(f)) for f in sorted(glob.glob('.github/regras/*.json')) if not f.endswith('protecao-principal.json')]
    ativa, pares = None, []
    if tok_adm:
        todas = api_todas(f'/repos/{repo}/rulesets', tok_adm)
        def busca(nome):
            ids = [r['id'] for r in todas if r.get('name') == nome]
            return api(f'/repos/{repo}/rulesets/{ids[0]}', tok_adm) if len(ids) == 1 else None  # nenhuma ou repetida: reprova
        ativa = busca('principal-protegida')
        pares = [(e['name'], busca(e['name']), e) for e in extras]
    else:
        pares = [(e['name'], None, e) for e in extras]
    commit = api(f'/repos/{repo}/commits/{sha}', tok)
    pedidos = api(f'/repos/{repo}/commits/{sha}/pulls', tok)
    pr = next((p for p in pedidos if p.get('merged_at')), None)
    revisoes = api_todas(f"/repos/{repo}/pulls/{pr['number']}/reviews", tok) if pr else []
    dono = repo.split('/')[0]
    org = admins = seguranca = None
    if tok_adm:
        try:
            org = api(f'/orgs/{dono}', tok_adm)
            admins = api_todas(f'/orgs/{dono}/members?role=admin', tok_adm)
            seguranca = api_todas(f'/orgs/{dono}/teams/seguranca/members', tok_adm)
        except Exception as e:  # sem acesso = reprovado, nunca "passou"
            print('REPROVADO: não foi possível ler a administração da organização:', e, file=sys.stderr)
    res, motivos = avaliar(ativa, esperada, commit, pedidos, revisoes, org, admins, seguranca)
    extra = comparar_todas(pares)
    if extra: res['protecao_do_repositorio'] = False; motivos += extra
    for m in motivos: print('REPROVADO:', m, file=sys.stderr)
    print(','.join(f'{k}={int(v)}' for k, v in res.items()))
    sys.exit(0 if all(res.values()) else 1)
