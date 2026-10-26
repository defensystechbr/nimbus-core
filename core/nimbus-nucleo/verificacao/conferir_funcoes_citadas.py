#!/usr/bin/env python3
"""Confere se os nomes de função/tipo citados nos documentos como prova de que uma regra está
implementada ainda existem no código-fonte de verdade.

Por que isto existe (achado do dono em 09/10/2026): `CONTRATO_DA_PONTE.md` cita nomes como
`modulo::funcao` ou `modulo::Tipo::metodo` na coluna "Função do núcleo" de cada regra, como prova de
que o núcleo já implementa aquilo. Até agora nada no CI conferia se esses nomes CONTINUAM existindo —
um nome pode ser apagado, renomeado, ou (como na regra 35 desta mesma rodada) o documento pode descrever
como "já implementado" algo que na verdade ainda faltava escrever, e a verificação completa continuaria
"tudo certo" do mesmo jeito, porque testes e mutação só conferem o código, nunca se a PROSA sobre o
código continua verdadeira. Este script fecha esse buraco especificamente para nomes citados.

O que faz, documento por documento: extrai todo trecho entre crases (`` `...` ``) que pareça uma
referência de código Rust e confere contra `src/*.rs` de verdade:

  - `modulo::item`             -> `item` precisa existir no TOPO de `src/<modulo>.rs`
                                   (fn, struct, enum, trait, type, const ou static)
  - `modulo::Tipo::membro`     -> `Tipo` precisa existir em `src/<modulo>.rs` E `membro` precisa
                                   existir dentro do corpo de `Tipo` (variante/campo) ou de algum
                                   `impl` desse `Tipo`, no MESMO arquivo
  - `modulo::sub::membro`      -> o mesmo que a linha acima, mas aceitando também que `sub` seja um
                                   `mod sub { ... }` (ex.: o `#[cfg(test)] mod testes { ... }` que
                                   praticamente todo arquivo de `src/*.rs` deste projeto usa) em vez de
                                   struct/enum/trait — achado desta sessão (10/10/2026): antes desta
                                   correção, QUALQUER citação no formato `modulo::testes::nome_do_teste`
                                   reprovava sempre (FALTANDO), mesmo com o teste citado existindo e
                                   passando, porque o conferidor só sabia procurar `sub` como tipo,
                                   nunca como módulo — ver `mod_definido`, abaixo
  - `Tipo::membro` (sem módulo)-> procura `Tipo` (struct/enum/trait) em TODOS os arquivos de `src/`;
                                   aceita se achar `membro` dentro do corpo ou de um `impl` em QUALQUER
                                   arquivo onde `Tipo` exista
  - `modulo::*` ou `Tipo::*`   -> curinga: só confere que o módulo ou tipo existe; não afirma nada
                                   sobre item nenhum dentro dele (por desenho do `*`)
  - um arquivo `.rs` citado (ex.: `coacao.rs`, `tests/adversario_ativo.rs`) -> confere que o arquivo
                                   existe no disco
  - identificador solto, sem `::` (ex.: `agora`, `valida_ate`) -> claim fraca por natureza (não diz
                                   module nem papel); reportado à parte, NUNCA reprova a verificação
                                   por si só — ver "Dois níveis de severidade" abaixo
  - qualquer outra coisa entre crases (prosa, nome de arquivo `.md`/`.json`, API nativa de
                                   iOS/Android como `CLOCK_MONOTONIC` ou `CXCallObserver`) -> fora do
                                   escopo deste conferidor (ele só confere o núcleo em Rust); listado
                                   como informativo, nunca reprova

Dois níveis de severidade (deliberado — ver `PENDENCIAS_E_DECISOES.md` para o motivo):
  FALTANDO  -> reprova a verificação (código de saída 1). Só para citações `::`-qualificadas
               (`modulo::item`, `modulo::Tipo::membro`, `Tipo::membro`) ou arquivo `.rs`: são
               afirmações específicas e checáveis, e citam um módulo/tipo que deveria existir.
  REVISAR   -> NÃO reprova por si só. Para identificador solto (sem `::`) que não apareceu em
               nenhum arquivo de `src/`. Um nome sem módulo pode ser perfeitamente genuíno (um
               parâmetro, um campo) ou pode citar algo que o núcleo nunca teve por desenho (ex.:
               `CLOCK_MONOTONIC`, `kSecAccessControlBiometryCurrentSet` — símbolos de plataforma
               nativa que o `src/*.rs` nunca vai conter). Transformar isso em reprovação automática
               faria o verificador gritar lobo em citação legítima de API nativa; por isso fica
               visível, para revisão humana, sem travar o CI.

Isto é "grep com reconhecimento de bloco" (conta chaves para achar o corpo de um tipo/impl), não um
parser de Rust de verdade: é auditável a olho, não precisa de nenhuma dependência além da biblioteca
padrão do Python, e resolve exatamente o risco apontado — sem fingir uma precisão que não tem. Qualquer
citação que caia fora do que este script sabe classificar é LISTADA, nunca engolida em silêncio:
ambiguidade conta como coisa a revisar, não como aprovação.

Fora de escopo, de propósito (não é regressão deste script, é limite conhecido — ver
PENDENCIAS_E_DECISOES.md): citação a OUTRO documento (`.md`, `.json`, ...) não é conferida contra o
disco aqui, porque pelo menos um caso real (`PLANO_DE_TAREFAS.md`, citado em `CONTRATO_DA_PONTE.md`
mas que vive dentro de cada kit entregável, não neste repositório de trabalho) provaria falso positivo
numa checagem ingênua de caminho. Conferir referências entre documentos é um problema diferente
(precisa saber QUAL kit/camada cada documento pertence) e fica para outra rodada, declarado em aberto.

Uso: python3 verificacao/conferir_funcoes_citadas.py [documento.md ...]
     Sem argumento: confere CONTRATO_DA_PONTE.md (o caso que motivou este script).
Saída: 0 = nenhuma citação `::`-qualificada ou de arquivo .rs ficou sem achar. 1 = alguma ficou
       (FALTANDO; a lista aparece antes). 2 = erro de uso ou de leitura de arquivo.
"""
import os
import re
import sys

NUCLEO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RAIZ_DOCS = os.path.normpath(os.path.join(NUCLEO, "..", ".."))

RE_MOD_DECL = re.compile(r"^\s*(?:pub\s+)?mod\s+(\w+)\s*;", re.M)
RE_SPAN = re.compile(r"`([^`\n]+)`")
RE_CAMINHO_RS = re.compile(r"^[\w./-]+\.rs$")
RE_CAMINHO_CODIGO = re.compile(r"^[A-Za-z_]\w*(::[A-Za-z_]\w*)*(::\*)?$")
RE_SO_CURINGA_OU_ID = re.compile(r"^[A-Za-z_]\w*$")
EXTENSOES_FORA_DE_ESCOPO = (
    ".md", ".json", ".yml", ".yaml", ".toml", ".py", ".sh", ".csv",
    ".html", ".docx", ".pptx", ".pdf", ".zip", ".pem",
)


def item_no_topo(nome):
    return re.compile(
        r"\b(?:pub(?:\([^)]*\))?\s+)?(?:fn|struct|enum|trait|type|const|static)\s+"
        + re.escape(nome) + r"\b"
    )


def tipo_definido(nome):
    return re.compile(
        r"\b(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait)\s+" + re.escape(nome) + r"\b"
    )


def mod_definido(nome):
    """Igual a `tipo_definido`, mas para `mod nome { ... }` (nunca `mod nome;`, que é outro arquivo,
    já coberto por `descobrir_modulos`) — fecha o caso de `modulo::testes::funcao`, achado desta sessão
    (10/10/2026): ver a nota na docstring do módulo sobre `modulo::sub::membro`."""
    return re.compile(
        r"\b(?:pub(?:\([^)]*\))?\s+)?mod\s+" + re.escape(nome) + r"\s*\{"
    )


def descobrir_modulos():
    """Nome do módulo -> conteúdo de `src/<nome>.rs`, lido de `mod nome;` em `src/lib.rs`. Nunca uma
    lista copiada à mão: um módulo novo em lib.rs já entra aqui na próxima vez que o script rodar."""
    with open(os.path.join(NUCLEO, "src", "lib.rs"), encoding="utf-8") as f:
        texto_lib = f.read()
    mods = {}
    for nome in RE_MOD_DECL.findall(texto_lib):
        caminho = os.path.join(NUCLEO, "src", f"{nome}.rs")
        if os.path.isfile(caminho):
            with open(caminho, encoding="utf-8") as f:
                mods[nome] = f.read()
    return mods


def ate_fechar(texto, pos_abre):
    """pos_abre aponta para a '{' de abertura; devolve o texto até a '}' que fecha ela, por contagem
    simples de chaves (não distingue chave dentro de string/comentário — risco aceito e documentado:
    ver docstring do módulo; nenhum caso real encontrado nesta base de código)."""
    nivel = 0
    for i in range(pos_abre, len(texto)):
        if texto[i] == "{":
            nivel += 1
        elif texto[i] == "}":
            nivel -= 1
            if nivel == 0:
                return texto[pos_abre: i + 1]
    return texto[pos_abre:]


def blocos_do_tipo(tipo, texto):
    """Trechos de `texto` que pertencem ao corpo do próprio `tipo` (struct/enum/trait Tipo { ... }),
    a algum `impl ... Tipo ... { ... }` do mesmo arquivo, ou a `mod Tipo { ... }` (achado desta sessão,
    10/10/2026 — ver `mod_definido` e a nota de `modulo::sub::membro` na docstring do módulo; sem isto,
    `membro_aparece` nunca chegava a olhar dentro de um `#[cfg(test)] mod testes { ... }`, por mais que
    o teste citado existisse e passasse)."""
    blocos = []
    for padrao in (
        r"\b(?:struct|enum|trait)\s+" + re.escape(tipo) + r"\b[^{};]{0,300}\{",
        r"\bimpl\b[^{};]{0,400}\b" + re.escape(tipo) + r"\b[^{};]{0,200}\{",
        r"\b(?:pub(?:\([^)]*\))?\s+)?mod\s+" + re.escape(tipo) + r"\s*\{",
    ):
        for m in re.finditer(padrao, texto):
            blocos.append(ate_fechar(texto, m.end() - 1))
    return blocos


def membro_aparece(membro, bloco):
    return bool(item_no_topo(membro).search(bloco)) or bool(
        re.search(r"\b" + re.escape(membro) + r"\b", bloco)
    )


def conferir_citacao(citacao, mods):
    """Devolve (status, detalhe). status em OK / OK_CURINGA / FALTANDO / REVISAR / FORA_DE_ESCOPO /
    None (span vazio ou claramente não é uma citação — nem chega a entrar no relatório)."""
    c = citacao.strip()
    if not c:
        return None
    c = re.sub(r"\(\)$", "", c)  # `modulo::funcao()` citada em estilo de chamada: mesma coisa que `modulo::funcao`

    if RE_CAMINHO_RS.match(c):
        candidatos = [c, os.path.join("src", c)]
        for rel in candidatos:
            if os.path.isfile(os.path.join(NUCLEO, rel)):
                return ("OK", f"arquivo encontrado: {rel}")
        return ("FALTANDO", f"arquivo citado não existe em nenhum destes caminhos: {candidatos}")

    if any(c.endswith(ext) for ext in EXTENSOES_FORA_DE_ESCOPO):
        return ("FORA_DE_ESCOPO", "referência a outro documento, não a código Rust — não conferida por este script (ver docstring)")

    if not RE_CAMINHO_CODIGO.match(c):
        return ("FORA_DE_ESCOPO", "não tem formato de caminho de código Rust (nem de arquivo .rs) — provavelmente prosa dentro de crases")

    partes = c.split("::")

    if len(partes) == 1:
        alvo = partes[0]
        if item_no_topo(alvo).search("\n".join(mods.values())):
            achado_em = [n for n, t in mods.items() if item_no_topo(alvo).search(t)]
            return ("OK", f"achado no topo de: {', '.join(achado_em)}")
        for nome_mod, texto in mods.items():
            if re.search(r"\b" + re.escape(alvo) + r"\b", texto):
                return ("OK", f"achado (busca fraca, sem módulo) em src/{nome_mod}.rs")
        return ("REVISAR", "identificador solto (sem `::`) não encontrado em nenhum arquivo de src/ — pode ser nome de API nativa (fora do núcleo Rust) ou citação legitimamente genérica; revisão humana decide, isto não reprova por si só")

    primeiro, resto = partes[0], partes[1:]
    if primeiro in mods:
        candidatos_arquivo = {primeiro: mods[primeiro]}
    else:
        candidatos_arquivo = {n: t for n, t in mods.items() if tipo_definido(primeiro).search(t)}
        if not candidatos_arquivo:
            return ("FALTANDO", f"`{primeiro}` não é módulo declarado em src/lib.rs nem struct/enum/trait em nenhum arquivo de src/")
        resto = partes  # primeiro segmento também é o Tipo a conferir dentro do próprio arquivo

    if resto[-1] == "*":
        return ("OK_CURINGA", "curinga `::*`: confere só que o módulo/tipo existe, por desenho não afirma nada sobre item nenhum dentro dele")

    if len(resto) == 1:
        item = resto[0]
        for nome_mod, texto in candidatos_arquivo.items():
            if item_no_topo(item).search(texto):
                return ("OK", f"achado no topo de src/{nome_mod}.rs")
        return ("FALTANDO", f"`{item}` não encontrado no topo de: {', '.join(f'src/{n}.rs' for n in candidatos_arquivo)}")

    if len(resto) == 2:
        tipo, membro = resto
        tipo_existe_em = [
            n for n, t in candidatos_arquivo.items()
            if tipo_definido(tipo).search(t) or mod_definido(tipo).search(t)
        ]
        if not tipo_existe_em:
            return ("FALTANDO", f"nem tipo (struct/enum/trait) nem `mod` chamado `{tipo}` encontrado em: {', '.join(f'src/{n}.rs' for n in candidatos_arquivo)}")
        for nome_mod in tipo_existe_em:
            for bloco in blocos_do_tipo(tipo, candidatos_arquivo[nome_mod]):
                if membro_aparece(membro, bloco):
                    return ("OK", f"achado dentro de `{tipo}` em src/{nome_mod}.rs")
        return ("FALTANDO", f"`{tipo}` existe (tipo ou mod) em {', '.join(f'src/{n}.rs' for n in tipo_existe_em)}, mas `{membro}` não foi encontrado no corpo dele nem em nenhum `impl` seu")

    return ("REVISAR", f"caminho com {len(resto)} segmentos depois do módulo/tipo — formato não previsto por este conferidor; revisar à mão")


def conferir_documento(caminho_doc, mods):
    with open(caminho_doc, encoding="utf-8") as f:
        texto = f.read()
    spans_vistos = {}
    for span in RE_SPAN.findall(texto):
        if span in spans_vistos:
            continue
        resultado = conferir_citacao(span, mods)
        if resultado is not None:
            spans_vistos[span] = resultado
    return spans_vistos


def localizar_documento(nome):
    if os.path.isfile(nome):
        return os.path.abspath(nome)
    # RAIZ_DOCS serve a árvore-fonte (pacote_final/, onde os documentos ficam direto ao lado de
    # nimbus-nucleo_src/); RAIZ_DOCS/docs serve o layout de cada kit (core/nimbus-nucleo/ + docs/
    # como irmãos dentro da raiz do kit) — achado e corrigido na Rodada 26 (12/10/2026): antes desta
    # correção, rodar este script de dentro de um kit descompactado sempre dava ERRO: documento não
    # encontrado, mesmo com CONTRATO_DA_PONTE.md presente em docs/ do kit (testado ao vivo, não só
    # deduzido) — por isso o verificar.sh de cada kit nunca chamava este conferidor até agora.
    for base in (RAIZ_DOCS, os.path.join(RAIZ_DOCS, "docs"), NUCLEO, os.getcwd()):
        candidato = os.path.join(base, nome)
        if os.path.isfile(candidato):
            return candidato
    return None


def main():
    nomes = sys.argv[1:] or ["CONTRATO_DA_PONTE.md"]
    mods = descobrir_modulos()
    if not mods:
        print("ERRO: nenhum módulo encontrado a partir de src/lib.rs — não dá para conferir nada.")
        return 2

    faltando_total = []
    revisar_total = []
    fora_de_escopo_total = []
    ok_total = 0

    for nome in nomes:
        caminho = localizar_documento(nome)
        if caminho is None:
            print(f"ERRO: documento não encontrado: {nome} (procurado em {RAIZ_DOCS}, {NUCLEO} e no diretório atual)")
            return 2
        resultados = conferir_documento(caminho, mods)
        print(f"\n{nome} ({len(resultados)} citação(ões) de código distintas encontradas entre crases):")
        for span, (status, detalhe) in sorted(resultados.items()):
            if status == "FALTANDO":
                faltando_total.append((nome, span, detalhe))
                print(f"  ✗ FALTANDO   `{span}` — {detalhe}")
            elif status == "REVISAR":
                revisar_total.append((nome, span, detalhe))
                print(f"  ? REVISAR    `{span}` — {detalhe}")
            elif status == "FORA_DE_ESCOPO":
                fora_de_escopo_total.append((nome, span))
            else:
                ok_total += 1

    if fora_de_escopo_total:
        print(f"\n(fora do escopo deste conferidor, não afeta o resultado: {len(fora_de_escopo_total)} trecho(s) entre crases que não são caminho de código Rust nem de arquivo .rs — nomes de outros documentos, prosa, APIs nativas de iOS/Android, etc.)")

    print(f"\n{ok_total} citação(ões) confirmada(s) no código-fonte.")
    if revisar_total:
        print(f"{len(revisar_total)} identificador(es) solto(s) (sem `::`) não encontrado(s) — para revisão humana, NÃO reprova por si só:")
        for nome, span, detalhe in revisar_total:
            print(f"  ? [{nome}] `{span}` — {detalhe}")
    if faltando_total:
        print(f"\nCONFERIDOR DE CITAÇÕES: REPROVADO ({len(faltando_total)} citação(ões) de código que não existem mais no código-fonte):")
        for nome, span, detalhe in faltando_total:
            print(f"  ✗ [{nome}] `{span}` — {detalhe}")
        return 1

    print("Conferidor de citações: nenhuma citação de código ficou sem achar no código-fonte.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
