#!/usr/bin/env python3
"""Consistência do kit: telas × protótipo, funções citadas × núcleo, termos proibidos. Rode sempre ao fim de uma tarefa."""
import re, glob, sys, os
raiz = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(raiz)
falhas = []
proto = open('docs/PROTOTIPO.html', encoding='utf-8').read()
m = re.search(r'const ORDEM=\{chat:(\[[^\]]*\]),os:(\[[^\]]*\])\}', proto)
import ast  # nunca eval: o protótipo é dado, não código
ordem = {'chat': ast.literal_eval(m.group(1)), 'os': ast.literal_eval(m.group(2))}
telas = open('docs/TELAS.md', encoding='utf-8').read()
ids = re.findall(r'identificador `([a-z_]+)`', telas)
app = 'chat' if 'NimbusChat' in telas.splitlines()[0] else 'os'
if ids != ordem[app]: falhas.append(f'docs/TELAS.md não bate com o protótipo ({len(ids)} × {len(ordem[app])} telas)')
fonte = ' '.join(open(f, encoding='utf-8').read() for f in glob.glob('core/nimbus-nucleo/src/*.rs'))
docs = ' '.join(open(f, encoding='utf-8').read() for f in glob.glob('docs/*.md') + glob.glob('*.md') + glob.glob('prompts/*.md'))
# Rodada 26 (12/10/2026): um trecho que MOSTRA, como exemplo, uma citação antiga incorreta usa o
# escape padrão do Markdown para exibir um crase literal (``` `` `texto` `` ```) — sem esta exclusão,
# o conferidor confundia "citar o erro para explicar que foi corrigido" com "citar como referência
# válida" (achado ao reconciliar PENDENCIAS_E_DECISOES.md contra este conferidor; o próprio parágrafo
# que descreve a correção de 'clippy::bool_assert_comparison' citado assim recriava o falso positivo).
docs_para_citacoes = re.sub(r'``[^\n]*?``', ' ', docs)
# Namespaces que não são do núcleo: `env::var` num exemplo de código de documento é Rust padrão (ou de uma
# biblioteca), não uma citação de módulo do núcleo. Sem esta lista, o exemplo que mostra justamente o padrão
# PROIBIDO (`env::var` para segredo de produção, em PENDENCIAS_E_DECISOES.md) era lido como citação inválida e
# reprovava a consistência — o documento que denuncia o defeito virava o defeito. Achado em 26/10/2026.
FORA_DO_NUCLEO = {'env', 'std', 'core', 'alloc', 'tokio', 'serde', 'clippy', 'rusqlite', 'ring', 'zeroize'}
for mod, a, b in set(re.findall(r'`([a-z_]+)::([A-Za-z_0-9]+)(?:::([A-Za-z_0-9]+))?', docs_para_citacoes)):
    if mod in FORA_DO_NUCLEO: continue
    nome = b or a
    # Rodada 26: 'static' estava faltando (pegava `const`/`fn`/`struct`/`enum`/`type` mas não
    # `pub static NOME: ...` — achado real via `codec_palavras::LISTA_DE_PALAVRAS`, que existe no
    # núcleo como `pub static`, não `const`). Classe de pontuação também ampliada: uma variante de
    # enum sem vírgula final antes de `}`, ou usada como `Enum::Variante)`/`;`, não batia com a
    # classe antiga `[\{\(,]` — achado real via `recuperacao::SemTestemunhaViva` (variante de
    # `ErroDeRecuperacao`, existe no código, só não era reconhecida por esta checagem).
    if not re.search(r'(fn|struct|enum|const|static|type)\s+' + re.escape(nome) + r'\b', fonte) and not re.search(r'\b' + re.escape(nome) + r'\s*[\{\(\),;\}]', fonte):
        falhas.append(f'função citada que não existe no núcleo: {mod}::{nome}')
tudo = docs + proto
for termo in ['marca invisível', 'Alfa Engenharia', 'watermark', 'salvamento::']:
    if termo in tudo: falhas.append(f'termo proibido encontrado: {termo}')
# Rodada 26: '72 h'/'72 horas' como termo proibido literal passou a dar falso positivo — desde que
# a carência de recuperação (`recuperacao::pode_concluir`) e o cache local da custódia do cofre
# (`custodia_do_cofre::verificar_cache`) passaram a usar 72h de forma legítima e corrente, o mesmo
# texto deixou de apontar só para a alegação antiga e já superada ("mensagens duram/somem em 72h";
# a regra vigente é 48h). Verificado nesta rodada: toda ocorrência de '72 h'/'72 horas' nos arquivos
# que este conferidor lê é sobre recuperação/custódia, nenhuma sobre duração de mensagem — por isso
# a checagem agora é específica ao padrão perigoso, não ao número sozinho.
if re.search(r'mensage\w*[^.]{0,60}72\s*h(oras)?\b', tudo, re.IGNORECASE) or re.search(r'72\s*h(oras)?\b[^.]{0,60}mensage\w*', tudo, re.IGNORECASE):
    falhas.append('termo proibido encontrado: alegação de mensagens durarem/somem em 72h (regra vigente é 48h)')
print('\n'.join(falhas) if falhas else 'Consistência: tudo certo.')
sys.exit(1 if falhas else 0)
