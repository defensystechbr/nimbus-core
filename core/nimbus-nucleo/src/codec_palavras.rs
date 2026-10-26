//! # Módulo `codec_palavras` — BIP-39 para a frase de recuperação pessoal (Rodada 21, 11/10/2026)
//!
//! **O que é, em uma frase:** converte os 32 bytes da frase de recuperação pessoal
//! (`recuperacao::TAM_CHAVE_DA_FRASE`) em 24 palavras em português e de volta — uma TERCEIRA forma de
//! escrever o MESMO segredo, ao lado do hex e do QR que já existiam. Pedido do fundador: *"acho que as
//! palvras mágicas poderiam entrar também"*.
//!
//! **Aditivo, nunca substituto — e nunca um fator a mais de recuperação.** Isto já foi dito com todas as
//! letras no comentário do módulo `recuperacao` (correção feita antes de implementar, não aceita como
//! veio): quem souber a frase pessoal em QUALQUER codificação — hex, QR ou estas 24 palavras — tem o
//! MESMO fator único, não um fator extra. Contar as "palavras mágicas" como uma alternativa independente
//! do esquema de 2-de-3 teria dado uma falsa sensação de segurança. Este módulo não decide nada sobre
//! quantas trilhas são precisas para recuperar a conta — isso é só do módulo `recuperacao`. Este módulo
//! só sabe escrever e ler 32 bytes como palavras.
//!
//! **Não é uma frase-senha memorável — é só outra forma de escrever um segredo já aleatório.** No uso
//! mais comum do BIP-39 (carteiras de criptomoeda), as 24 palavras às vezes são tratadas como algo para
//! guardar em papel, não para memorizar — e é exatamente esse o papel aqui também: os 32 bytes de entrada
//! já foram gerados por `OsRng` dentro de `recuperacao::preparar_frase_pessoal`, com 256 bits de entropia
//! verdadeira. As palavras não adicionam nem retiram entropia nenhuma — só tornam esses mesmos 256 bits
//! mais fáceis de transcrever à mão e de detectar um erro de transcrição (ver checksum, abaixo) do que uma
//! string hexadecimal de 64 caracteres. Continua tão impossível de memorizar de cabeça quanto o hex ou o
//! QR — e essa é a intenção: são 2048 palavras possíveis por posição, não um vocabulário pequeno de frase
//! de efeito.
//!
//! **Algoritmo: o padrão BIP-39 publicado, não um desenho próprio.** Para um segredo de 256 bits
//! (`TAM_SEGREDO` = 32 bytes): o checksum é o primeiro byte (8 bits mais significativos) de
//! SHA-256(segredo); os 256 bits do segredo são concatenados com esses 8 bits de checksum, dando 264
//! bits; esses 264 bits são lidos em 24 grupos de 11 bits cada (bit mais significativo primeiro em cada
//! grupo), e cada grupo de 11 bits é o índice (0 a 2047) de uma palavra na lista oficial. Decodificar
//! inverte o processo e RECALCULA o checksum sobre o segredo decodificado — se não bater com os 8 bits
//! que vieram embutidos nas palavras, a decodificação é rejeitada (`ChecksumInvalido`) em vez de devolver
//! um segredo errado em silêncio. Isto pega a grande maioria dos erros de dedo ou de memória ao anotar ou
//! transcrever uma palavra (1 chance em 256 de um erro passar batido pelo checksum, não zero — honesto,
//! não "detecção perfeita").
//!
//! **A lista de palavras é a oficial do padrão, buscada da fonte primária — não inventada por este
//! trabalho.** Baixada em 11/10/2026 direto do repositório canônico do padrão,
//! `https://raw.githubusercontent.com/bitcoin/bips/master/bip-0039/portuguese.txt`. Conferida byte a byte
//! antes de ser copiada para este arquivo: exatamente 2048 linhas, zero duplicatas, zero linhas vazias,
//! ordem alfabética, SHA-256 do arquivo baixado = `2685e9c194c82ae67e10ba59d9ea5345a23dc093e92276fc5361f6667d79cd3f`
//! (comando: `sha256sum portuguese.txt`, reconferido de forma independente com `sha256sum` do coreutils
//! nesta mesma rodada, 11/10/2026, após uma primeira transcrição deste hash no comentário deste módulo
//! ter saído com 63 caracteres em vez de 64 — falha achada e corrigida nesta auditoria, ver FECHO).
//! Autenticidade
//! cruzada confirmada comparando com a lista irmã em inglês do mesmo padrão, que começa com as palavras
//! universalmente conhecidas "abandon", "ability", "able" — confirma que a fonte é o repositório genuíno
//! do BIP-39, não uma página adulterada. A lista oficial em português NÃO tem acentos (é uma propriedade
//! genuína do padrão publicado, pensada para teclados e fontes limitados — não um defeito de acentuação
//! introduzido ao baixar ou copiar este arquivo).
//!
//! **Responsabilidade dividida com a tela (fora deste núcleo):** `decodificar` compara as palavras
//! exatamente como estão na lista oficial — tudo minúsculo, sem acento, sem espaço extra. Normalizar o que
//! a pessoa digitou (remover espaços nas pontas, baixar a caixa) é tarefa da tela que ainda vai chamar
//! esta função, não deste módulo; aqui a entrada é tratada como já normalizada, e qualquer divergência de
//! formatação é reportada como `PalavraDesconhecida`, igual a uma palavra realmente errada.
use sha2::{Digest, Sha256};

/// Tamanho do segredo que este codec lê e escreve: os mesmos 32 bytes de `recuperacao::TAM_CHAVE_DA_FRASE`.
/// Não importamos `recuperacao` aqui para este módulo continuar uma ferramenta de baixo nível, sem
/// depender de quem o usa (é `recuperacao` que pode vir a usar `codec_palavras`, nunca o contrário); a
/// igualdade dos dois números é garantida por teste (`tamanho_bate_com_o_modulo_recuperacao`, abaixo), não
/// por suposição.
pub const TAM_SEGREDO: usize = 32;
/// 24 palavras: 256 bits de segredo + 8 bits de checksum = 264 bits = 24 × 11 bits.
pub const NUM_PALAVRAS: usize = 24;
const TAM_LISTA: usize = 2048;
const BITS_POR_PALAVRA: u32 = 11;
const BITS_CHECKSUM: usize = 8;

/// A lista oficial de 2048 palavras do BIP-39 em português — proveniência e hash de conferência no
/// comentário do módulo, acima. Ordem alfabética original preservada (não é usada pelo algoritmo, que só
/// depende da posição de cada palavra ser estável e única — mas preservar a ordem oficial facilita
/// conferir esta cópia contra a fonte).
#[rustfmt::skip]
pub static LISTA_DE_PALAVRAS: [&str; TAM_LISTA] = [
    "abacate", "abaixo", "abalar", "abater", "abduzir", "abelha", "aberto", "abismo",
    "abotoar", "abranger", "abreviar", "abrigar", "abrupto", "absinto", "absoluto", "absurdo",
    "abutre", "acabado", "acalmar", "acampar", "acanhar", "acaso", "aceitar", "acelerar",
    "acenar", "acervo", "acessar", "acetona", "achatar", "acidez", "acima", "acionado",
    "acirrar", "aclamar", "aclive", "acolhida", "acomodar", "acoplar", "acordar", "acumular",
    "acusador", "adaptar", "adega", "adentro", "adepto", "adequar", "aderente", "adesivo",
    "adeus", "adiante", "aditivo", "adjetivo", "adjunto", "admirar", "adorar", "adquirir",
    "adubo", "adverso", "advogado", "aeronave", "afastar", "aferir", "afetivo", "afinador",
    "afivelar", "aflito", "afluente", "afrontar", "agachar", "agarrar", "agasalho", "agenciar",
    "agilizar", "agiota", "agitado", "agora", "agradar", "agreste", "agrupar", "aguardar",
    "agulha", "ajoelhar", "ajudar", "ajustar", "alameda", "alarme", "alastrar", "alavanca",
    "albergue", "albino", "alcatra", "aldeia", "alecrim", "alegria", "alertar", "alface",
    "alfinete", "algum", "alheio", "aliar", "alicate", "alienar", "alinhar", "aliviar",
    "almofada", "alocar", "alpiste", "alterar", "altitude", "alucinar", "alugar", "aluno",
    "alusivo", "alvo", "amaciar", "amador", "amarelo", "amassar", "ambas", "ambiente",
    "ameixa", "amenizar", "amido", "amistoso", "amizade", "amolador", "amontoar", "amoroso",
    "amostra", "amparar", "ampliar", "ampola", "anagrama", "analisar", "anarquia", "anatomia",
    "andaime", "anel", "anexo", "angular", "animar", "anjo", "anomalia", "anotado",
    "ansioso", "anterior", "anuidade", "anunciar", "anzol", "apagador", "apalpar", "apanhado",
    "apego", "apelido", "apertada", "apesar", "apetite", "apito", "aplauso", "aplicada",
    "apoio", "apontar", "aposta", "aprendiz", "aprovar", "aquecer", "arame", "aranha",
    "arara", "arcada", "ardente", "areia", "arejar", "arenito", "aresta", "argiloso",
    "argola", "arma", "arquivo", "arraial", "arrebate", "arriscar", "arroba", "arrumar",
    "arsenal", "arterial", "artigo", "arvoredo", "asfaltar", "asilado", "aspirar", "assador",
    "assinar", "assoalho", "assunto", "astral", "atacado", "atadura", "atalho", "atarefar",
    "atear", "atender", "aterro", "ateu", "atingir", "atirador", "ativo", "atoleiro",
    "atracar", "atrevido", "atriz", "atual", "atum", "auditor", "aumentar", "aura",
    "aurora", "autismo", "autoria", "autuar", "avaliar", "avante", "avaria", "avental",
    "avesso", "aviador", "avisar", "avulso", "axila", "azarar", "azedo", "azeite",
    "azulejo", "babar", "babosa", "bacalhau", "bacharel", "bacia", "bagagem", "baiano",
    "bailar", "baioneta", "bairro", "baixista", "bajular", "baleia", "baliza", "balsa",
    "banal", "bandeira", "banho", "banir", "banquete", "barato", "barbado", "baronesa",
    "barraca", "barulho", "baseado", "bastante", "batata", "batedor", "batida", "batom",
    "batucar", "baunilha", "beber", "beijo", "beirada", "beisebol", "beldade", "beleza",
    "belga", "beliscar", "bendito", "bengala", "benzer", "berimbau", "berlinda", "berro",
    "besouro", "bexiga", "bezerro", "bico", "bicudo", "bienal", "bifocal", "bifurcar",
    "bigorna", "bilhete", "bimestre", "bimotor", "biologia", "biombo", "biosfera", "bipolar",
    "birrento", "biscoito", "bisneto", "bispo", "bissexto", "bitola", "bizarro", "blindado",
    "bloco", "bloquear", "boato", "bobagem", "bocado", "bocejo", "bochecha", "boicotar",
    "bolada", "boletim", "bolha", "bolo", "bombeiro", "bonde", "boneco", "bonita",
    "borbulha", "borda", "boreal", "borracha", "bovino", "boxeador", "branco", "brasa",
    "braveza", "breu", "briga", "brilho", "brincar", "broa", "brochura", "bronzear",
    "broto", "bruxo", "bucha", "budismo", "bufar", "bule", "buraco", "busca",
    "busto", "buzina", "cabana", "cabelo", "cabide", "cabo", "cabrito", "cacau",
    "cacetada", "cachorro", "cacique", "cadastro", "cadeado", "cafezal", "caiaque", "caipira",
    "caixote", "cajado", "caju", "calafrio", "calcular", "caldeira", "calibrar", "calmante",
    "calota", "camada", "cambista", "camisa", "camomila", "campanha", "camuflar", "canavial",
    "cancelar", "caneta", "canguru", "canhoto", "canivete", "canoa", "cansado", "cantar",
    "canudo", "capacho", "capela", "capinar", "capotar", "capricho", "captador", "capuz",
    "caracol", "carbono", "cardeal", "careca", "carimbar", "carneiro", "carpete", "carreira",
    "cartaz", "carvalho", "casaco", "casca", "casebre", "castelo", "casulo", "catarata",
    "cativar", "caule", "causador", "cautelar", "cavalo", "caverna", "cebola", "cedilha",
    "cegonha", "celebrar", "celular", "cenoura", "censo", "centeio", "cercar", "cerrado",
    "certeiro", "cerveja", "cetim", "cevada", "chacota", "chaleira", "chamado", "chapada",
    "charme", "chatice", "chave", "chefe", "chegada", "cheiro", "cheque", "chicote",
    "chifre", "chinelo", "chocalho", "chover", "chumbo", "chutar", "chuva", "cicatriz",
    "ciclone", "cidade", "cidreira", "ciente", "cigana", "cimento", "cinto", "cinza",
    "ciranda", "circuito", "cirurgia", "citar", "clareza", "clero", "clicar", "clone",
    "clube", "coado", "coagir", "cobaia", "cobertor", "cobrar", "cocada", "coelho",
    "coentro", "coeso", "cogumelo", "coibir", "coifa", "coiote", "colar", "coleira",
    "colher", "colidir", "colmeia", "colono", "coluna", "comando", "combinar", "comentar",
    "comitiva", "comover", "complexo", "comum", "concha", "condor", "conectar", "confuso",
    "congelar", "conhecer", "conjugar", "consumir", "contrato", "convite", "cooperar", "copeiro",
    "copiador", "copo", "coquetel", "coragem", "cordial", "corneta", "coronha", "corporal",
    "correio", "cortejo", "coruja", "corvo", "cosseno", "costela", "cotonete", "couro",
    "couve", "covil", "cozinha", "cratera", "cravo", "creche", "credor", "creme",
    "crer", "crespo", "criada", "criminal", "crioulo", "crise", "criticar", "crosta",
    "crua", "cruzeiro", "cubano", "cueca", "cuidado", "cujo", "culatra", "culminar",
    "culpar", "cultura", "cumprir", "cunhado", "cupido", "curativo", "curral", "cursar",
    "curto", "cuspir", "custear", "cutelo", "damasco", "datar", "debater", "debitar",
    "deboche", "debulhar", "decalque", "decimal", "declive", "decote", "decretar", "dedal",
    "dedicado", "deduzir", "defesa", "defumar", "degelo", "degrau", "degustar", "deitado",
    "deixar", "delator", "delegado", "delinear", "delonga", "demanda", "demitir", "demolido",
    "dentista", "depenado", "depilar", "depois", "depressa", "depurar", "deriva", "derramar",
    "desafio", "desbotar", "descanso", "desenho", "desfiado", "desgaste", "desigual", "deslize",
    "desmamar", "desova", "despesa", "destaque", "desviar", "detalhar", "detentor", "detonar",
    "detrito", "deusa", "dever", "devido", "devotado", "dezena", "diagrama", "dialeto",
    "didata", "difuso", "digitar", "dilatado", "diluente", "diminuir", "dinastia", "dinheiro",
    "diocese", "direto", "discreta", "disfarce", "disparo", "disquete", "dissipar", "distante",
    "ditador", "diurno", "diverso", "divisor", "divulgar", "dizer", "dobrador", "dolorido",
    "domador", "dominado", "donativo", "donzela", "dormente", "dorsal", "dosagem", "dourado",
    "doutor", "drenagem", "drible", "drogaria", "duelar", "duende", "dueto", "duplo",
    "duquesa", "durante", "duvidoso", "eclodir", "ecoar", "ecologia", "edificar", "edital",
    "educado", "efeito", "efetivar", "ejetar", "elaborar", "eleger", "eleitor", "elenco",
    "elevador", "eliminar", "elogiar", "embargo", "embolado", "embrulho", "embutido", "emenda",
    "emergir", "emissor", "empatia", "empenho", "empinado", "empolgar", "emprego", "empurrar",
    "emulador", "encaixe", "encenado", "enchente", "encontro", "endeusar", "endossar", "enfaixar",
    "enfeite", "enfim", "engajado", "engenho", "englobar", "engomado", "engraxar", "enguia",
    "enjoar", "enlatar", "enquanto", "enraizar", "enrolado", "enrugar", "ensaio", "enseada",
    "ensino", "ensopado", "entanto", "enteado", "entidade", "entortar", "entrada", "entulho",
    "envergar", "enviado", "envolver", "enxame", "enxerto", "enxofre", "enxuto", "epiderme",
    "equipar", "ereto", "erguido", "errata", "erva", "ervilha", "esbanjar", "esbelto",
    "escama", "escola", "escrita", "escuta", "esfinge", "esfolar", "esfregar", "esfumado",
    "esgrima", "esmalte", "espanto", "espelho", "espiga", "esponja", "espreita", "espumar",
    "esquerda", "estaca", "esteira", "esticar", "estofado", "estrela", "estudo", "esvaziar",
    "etanol", "etiqueta", "euforia", "europeu", "evacuar", "evaporar", "evasivo", "eventual",
    "evidente", "evoluir", "exagero", "exalar", "examinar", "exato", "exausto", "excesso",
    "excitar", "exclamar", "executar", "exemplo", "exibir", "exigente", "exonerar", "expandir",
    "expelir", "expirar", "explanar", "exposto", "expresso", "expulsar", "externo", "extinto",
    "extrato", "fabricar", "fabuloso", "faceta", "facial", "fada", "fadiga", "faixa",
    "falar", "falta", "familiar", "fandango", "fanfarra", "fantoche", "fardado", "farelo",
    "farinha", "farofa", "farpa", "fartura", "fatia", "fator", "favorita", "faxina",
    "fazenda", "fechado", "feijoada", "feirante", "felino", "feminino", "fenda", "feno",
    "fera", "feriado", "ferrugem", "ferver", "festejar", "fetal", "feudal", "fiapo",
    "fibrose", "ficar", "ficheiro", "figurado", "fileira", "filho", "filme", "filtrar",
    "firmeza", "fisgada", "fissura", "fita", "fivela", "fixador", "fixo", "flacidez",
    "flamingo", "flanela", "flechada", "flora", "flutuar", "fluxo", "focal", "focinho",
    "fofocar", "fogo", "foguete", "foice", "folgado", "folheto", "forjar", "formiga",
    "forno", "forte", "fosco", "fossa", "fragata", "fralda", "frango", "frasco",
    "fraterno", "freira", "frente", "fretar", "frieza", "friso", "fritura", "fronha",
    "frustrar", "fruteira", "fugir", "fulano", "fuligem", "fundar", "fungo", "funil",
    "furador", "furioso", "futebol", "gabarito", "gabinete", "gado", "gaiato", "gaiola",
    "gaivota", "galega", "galho", "galinha", "galocha", "ganhar", "garagem", "garfo",
    "gargalo", "garimpo", "garoupa", "garrafa", "gasoduto", "gasto", "gata", "gatilho",
    "gaveta", "gazela", "gelado", "geleia", "gelo", "gemada", "gemer", "gemido",
    "generoso", "gengiva", "genial", "genoma", "genro", "geologia", "gerador", "germinar",
    "gesso", "gestor", "ginasta", "gincana", "gingado", "girafa", "girino", "glacial",
    "glicose", "global", "glorioso", "goela", "goiaba", "golfe", "golpear", "gordura",
    "gorjeta", "gorro", "gostoso", "goteira", "governar", "gracejo", "gradual", "grafite",
    "gralha", "grampo", "granada", "gratuito", "graveto", "graxa", "grego", "grelhar",
    "greve", "grilo", "grisalho", "gritaria", "grosso", "grotesco", "grudado", "grunhido",
    "gruta", "guache", "guarani", "guaxinim", "guerrear", "guiar", "guincho", "guisado",
    "gula", "guloso", "guru", "habitar", "harmonia", "haste", "haver", "hectare",
    "herdar", "heresia", "hesitar", "hiato", "hibernar", "hidratar", "hiena", "hino",
    "hipismo", "hipnose", "hipoteca", "hoje", "holofote", "homem", "honesto", "honrado",
    "hormonal", "hospedar", "humorado", "iate", "ideia", "idoso", "ignorado", "igreja",
    "iguana", "ileso", "ilha", "iludido", "iluminar", "ilustrar", "imagem", "imediato",
    "imenso", "imersivo", "iminente", "imitador", "imortal", "impacto", "impedir", "implante",
    "impor", "imprensa", "impune", "imunizar", "inalador", "inapto", "inativo", "incenso",
    "inchar", "incidir", "incluir", "incolor", "indeciso", "indireto", "indutor", "ineficaz",
    "inerente", "infantil", "infestar", "infinito", "inflamar", "informal", "infrator", "ingerir",
    "inibido", "inicial", "inimigo", "injetar", "inocente", "inodoro", "inovador", "inox",
    "inquieto", "inscrito", "inseto", "insistir", "inspetor", "instalar", "insulto", "intacto",
    "integral", "intimar", "intocado", "intriga", "invasor", "inverno", "invicto", "invocar",
    "iogurte", "iraniano", "ironizar", "irreal", "irritado", "isca", "isento", "isolado",
    "isqueiro", "italiano", "janeiro", "jangada", "janta", "jararaca", "jardim", "jarro",
    "jasmim", "jato", "javali", "jazida", "jejum", "joaninha", "joelhada", "jogador",
    "joia", "jornal", "jorrar", "jovem", "juba", "judeu", "judoca", "juiz",
    "julgador", "julho", "jurado", "jurista", "juro", "justa", "labareda", "laboral",
    "lacre", "lactante", "ladrilho", "lagarta", "lagoa", "laje", "lamber", "lamentar",
    "laminar", "lampejo", "lanche", "lapidar", "lapso", "laranja", "lareira", "largura",
    "lasanha", "lastro", "lateral", "latido", "lavanda", "lavoura", "lavrador", "laxante",
    "lazer", "lealdade", "lebre", "legado", "legendar", "legista", "leigo", "leiloar",
    "leitura", "lembrete", "leme", "lenhador", "lentilha", "leoa", "lesma", "leste",
    "letivo", "letreiro", "levar", "leveza", "levitar", "liberal", "libido", "liderar",
    "ligar", "ligeiro", "limitar", "limoeiro", "limpador", "linda", "linear", "linhagem",
    "liquidez", "listagem", "lisura", "litoral", "livro", "lixa", "lixeira", "locador",
    "locutor", "lojista", "lombo", "lona", "longe", "lontra", "lorde", "lotado",
    "loteria", "loucura", "lousa", "louvar", "luar", "lucidez", "lucro", "luneta",
    "lustre", "lutador", "luva", "macaco", "macete", "machado", "macio", "madeira",
    "madrinha", "magnata", "magreza", "maior", "mais", "malandro", "malha", "malote",
    "maluco", "mamilo", "mamoeiro", "mamute", "manada", "mancha", "mandato", "manequim",
    "manhoso", "manivela", "manobrar", "mansa", "manter", "manusear", "mapeado", "maquinar",
    "marcador", "maresia", "marfim", "margem", "marinho", "marmita", "maroto", "marquise",
    "marreco", "martelo", "marujo", "mascote", "masmorra", "massagem", "mastigar", "matagal",
    "materno", "matinal", "matutar", "maxilar", "medalha", "medida", "medusa", "megafone",
    "meiga", "melancia", "melhor", "membro", "memorial", "menino", "menos", "mensagem",
    "mental", "merecer", "mergulho", "mesada", "mesclar", "mesmo", "mesquita", "mestre",
    "metade", "meteoro", "metragem", "mexer", "mexicano", "micro", "migalha", "migrar",
    "milagre", "milenar", "milhar", "mimado", "minerar", "minhoca", "ministro", "minoria",
    "miolo", "mirante", "mirtilo", "misturar", "mocidade", "moderno", "modular", "moeda",
    "moer", "moinho", "moita", "moldura", "moleza", "molho", "molinete", "molusco",
    "montanha", "moqueca", "morango", "morcego", "mordomo", "morena", "mosaico", "mosquete",
    "mostarda", "motel", "motim", "moto", "motriz", "muda", "muito", "mulata",
    "mulher", "multar", "mundial", "munido", "muralha", "murcho", "muscular", "museu",
    "musical", "nacional", "nadador", "naja", "namoro", "narina", "narrado", "nascer",
    "nativa", "natureza", "navalha", "navegar", "navio", "neblina", "nebuloso", "negativa",
    "negociar", "negrito", "nervoso", "neta", "neural", "nevasca", "nevoeiro", "ninar",
    "ninho", "nitidez", "nivelar", "nobreza", "noite", "noiva", "nomear", "nominal",
    "nordeste", "nortear", "notar", "noticiar", "noturno", "novelo", "novilho", "novo",
    "nublado", "nudez", "numeral", "nupcial", "nutrir", "nuvem", "obcecado", "obedecer",
    "objetivo", "obrigado", "obscuro", "obstetra", "obter", "obturar", "ocidente", "ocioso",
    "ocorrer", "oculista", "ocupado", "ofegante", "ofensiva", "oferenda", "oficina", "ofuscado",
    "ogiva", "olaria", "oleoso", "olhar", "oliveira", "ombro", "omelete", "omisso",
    "omitir", "ondulado", "oneroso", "ontem", "opcional", "operador", "oponente", "oportuno",
    "oposto", "orar", "orbitar", "ordem", "ordinal", "orfanato", "orgasmo", "orgulho",
    "oriental", "origem", "oriundo", "orla", "ortodoxo", "orvalho", "oscilar", "ossada",
    "osso", "ostentar", "otimismo", "ousadia", "outono", "outubro", "ouvido", "ovelha",
    "ovular", "oxidar", "oxigenar", "pacato", "paciente", "pacote", "pactuar", "padaria",
    "padrinho", "pagar", "pagode", "painel", "pairar", "paisagem", "palavra", "palestra",
    "palheta", "palito", "palmada", "palpitar", "pancada", "panela", "panfleto", "panqueca",
    "pantanal", "papagaio", "papelada", "papiro", "parafina", "parcial", "pardal", "parede",
    "partida", "pasmo", "passado", "pastel", "patamar", "patente", "patinar", "patrono",
    "paulada", "pausar", "peculiar", "pedalar", "pedestre", "pediatra", "pedra", "pegada",
    "peitoral", "peixe", "pele", "pelicano", "penca", "pendurar", "peneira", "penhasco",
    "pensador", "pente", "perceber", "perfeito", "pergunta", "perito", "permitir", "perna",
    "perplexo", "persiana", "pertence", "peruca", "pescado", "pesquisa", "pessoa", "petiscar",
    "piada", "picado", "piedade", "pigmento", "pilastra", "pilhado", "pilotar", "pimenta",
    "pincel", "pinguim", "pinha", "pinote", "pintar", "pioneiro", "pipoca", "piquete",
    "piranha", "pires", "pirueta", "piscar", "pistola", "pitanga", "pivete", "planta",
    "plaqueta", "platina", "plebeu", "plumagem", "pluvial", "pneu", "poda", "poeira",
    "poetisa", "polegada", "policiar", "poluente", "polvilho", "pomar", "pomba", "ponderar",
    "pontaria", "populoso", "porta", "possuir", "postal", "pote", "poupar", "pouso",
    "povoar", "praia", "prancha", "prato", "praxe", "prece", "predador", "prefeito",
    "premiar", "prensar", "preparar", "presilha", "pretexto", "prevenir", "prezar", "primata",
    "princesa", "prisma", "privado", "processo", "produto", "profeta", "proibido", "projeto",
    "prometer", "propagar", "prosa", "protetor", "provador", "publicar", "pudim", "pular",
    "pulmonar", "pulseira", "punhal", "punir", "pupilo", "pureza", "puxador", "quadra",
    "quantia", "quarto", "quase", "quebrar", "queda", "queijo", "quente", "querido",
    "quimono", "quina", "quiosque", "rabanada", "rabisco", "rachar", "racionar", "radial",
    "raiar", "rainha", "raio", "raiva", "rajada", "ralado", "ramal", "ranger",
    "ranhura", "rapadura", "rapel", "rapidez", "raposa", "raquete", "raridade", "rasante",
    "rascunho", "rasgar", "raspador", "rasteira", "rasurar", "ratazana", "ratoeira", "realeza",
    "reanimar", "reaver", "rebaixar", "rebelde", "rebolar", "recado", "recente", "recheio",
    "recibo", "recordar", "recrutar", "recuar", "rede", "redimir", "redonda", "reduzida",
    "reenvio", "refinar", "refletir", "refogar", "refresco", "refugiar", "regalia", "regime",
    "regra", "reinado", "reitor", "rejeitar", "relativo", "remador", "remendo", "remorso",
    "renovado", "reparo", "repelir", "repleto", "repolho", "represa", "repudiar", "requerer",
    "resenha", "resfriar", "resgatar", "residir", "resolver", "respeito", "ressaca", "restante",
    "resumir", "retalho", "reter", "retirar", "retomada", "retratar", "revelar", "revisor",
    "revolta", "riacho", "rica", "rigidez", "rigoroso", "rimar", "ringue", "risada",
    "risco", "risonho", "robalo", "rochedo", "rodada", "rodeio", "rodovia", "roedor",
    "roleta", "romano", "roncar", "rosado", "roseira", "rosto", "rota", "roteiro",
    "rotina", "rotular", "rouco", "roupa", "roxo", "rubro", "rugido", "rugoso",
    "ruivo", "rumo", "rupestre", "russo", "sabor", "saciar", "sacola", "sacudir",
    "sadio", "safira", "saga", "sagrada", "saibro", "salada", "saleiro", "salgado",
    "saliva", "salpicar", "salsicha", "saltar", "salvador", "sambar", "samurai", "sanar",
    "sanfona", "sangue", "sanidade", "sapato", "sarda", "sargento", "sarjeta", "saturar",
    "saudade", "saxofone", "sazonal", "secar", "secular", "seda", "sedento", "sediado",
    "sedoso", "sedutor", "segmento", "segredo", "segundo", "seiva", "seleto", "selvagem",
    "semanal", "semente", "senador", "senhor", "sensual", "sentado", "separado", "sereia",
    "seringa", "serra", "servo", "setembro", "setor", "sigilo", "silhueta", "silicone",
    "simetria", "simpatia", "simular", "sinal", "sincero", "singular", "sinopse", "sintonia",
    "sirene", "siri", "situado", "soberano", "sobra", "socorro", "sogro", "soja",
    "solda", "soletrar", "solteiro", "sombrio", "sonata", "sondar", "sonegar", "sonhador",
    "sono", "soprano", "soquete", "sorrir", "sorteio", "sossego", "sotaque", "soterrar",
    "sovado", "sozinho", "suavizar", "subida", "submerso", "subsolo", "subtrair", "sucata",
    "sucesso", "suco", "sudeste", "sufixo", "sugador", "sugerir", "sujeito", "sulfato",
    "sumir", "suor", "superior", "suplicar", "suposto", "suprimir", "surdina", "surfista",
    "surpresa", "surreal", "surtir", "suspiro", "sustento", "tabela", "tablete", "tabuada",
    "tacho", "tagarela", "talher", "talo", "talvez", "tamanho", "tamborim", "tampa",
    "tangente", "tanto", "tapar", "tapioca", "tardio", "tarefa", "tarja", "tarraxa",
    "tatuagem", "taurino", "taxativo", "taxista", "teatral", "tecer", "tecido", "teclado",
    "tedioso", "teia", "teimar", "telefone", "telhado", "tempero", "tenente", "tensor",
    "tentar", "termal", "terno", "terreno", "tese", "tesoura", "testado", "teto",
    "textura", "texugo", "tiara", "tigela", "tijolo", "timbrar", "timidez", "tingido",
    "tinteiro", "tiragem", "titular", "toalha", "tocha", "tolerar", "tolice", "tomada",
    "tomilho", "tonel", "tontura", "topete", "tora", "torcido", "torneio", "torque",
    "torrada", "torto", "tostar", "touca", "toupeira", "toxina", "trabalho", "tracejar",
    "tradutor", "trafegar", "trajeto", "trama", "trancar", "trapo", "traseiro", "tratador",
    "travar", "treino", "tremer", "trepidar", "trevo", "triagem", "tribo", "triciclo",
    "tridente", "trilogia", "trindade", "triplo", "triturar", "triunfal", "trocar", "trombeta",
    "trova", "trunfo", "truque", "tubular", "tucano", "tudo", "tulipa", "tupi",
    "turbo", "turma", "turquesa", "tutelar", "tutorial", "uivar", "umbigo", "unha",
    "unidade", "uniforme", "urologia", "urso", "urtiga", "urubu", "usado", "usina",
    "usufruir", "vacina", "vadiar", "vagaroso", "vaidoso", "vala", "valente", "validade",
    "valores", "vantagem", "vaqueiro", "varanda", "vareta", "varrer", "vascular", "vasilha",
    "vassoura", "vazar", "vazio", "veado", "vedar", "vegetar", "veicular", "veleiro",
    "velhice", "veludo", "vencedor", "vendaval", "venerar", "ventre", "verbal", "verdade",
    "vereador", "vergonha", "vermelho", "verniz", "versar", "vertente", "vespa", "vestido",
    "vetorial", "viaduto", "viagem", "viajar", "viatura", "vibrador", "videira", "vidraria",
    "viela", "viga", "vigente", "vigiar", "vigorar", "vilarejo", "vinco", "vinheta",
    "vinil", "violeta", "virada", "virtude", "visitar", "visto", "vitral", "viveiro",
    "vizinho", "voador", "voar", "vogal", "volante", "voleibol", "voltagem", "volumoso",
    "vontade", "vulto", "vuvuzela", "xadrez", "xarope", "xeque", "xeretar", "xerife",
    "xingar", "zangado", "zarpar", "zebu", "zelador", "zombar", "zoologia", "zumbido",
];

/// Erros de `decodificar` — `codificar` não aparece aqui porque é infalível (o tipo `&[u8; TAM_SEGREDO]`
/// do parâmetro já torna o único jeito de errar, tamanho errado, impossível de representar).
#[derive(Debug, PartialEq, Eq)]
pub enum ErroDeCodec {
    /// Entrada com um número de palavras diferente de `NUM_PALAVRAS` (24).
    QuantidadeDePalavrasErrada,
    /// Alguma palavra não está em `LISTA_DE_PALAVRAS` — erro de dedo, ou não é a lista oficial.
    PalavraDesconhecida,
    /// Os 8 bits de checksum embutidos nas palavras não batem com o SHA-256 recalculado do segredo
    /// decodificado — sinal de erro de transcrição; a decodificação é rejeitada, nunca "arredondada"
    /// para o segredo mais parecido.
    ChecksumInvalido,
}

/// Bytes para bits, byte mais significativo primeiro dentro de cada byte (MSB first) — a ordem exigida
/// pelo BIP-39 tanto para os bits do segredo quanto para os do checksum.
fn bytes_para_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for &byte in bytes {
        for deslocamento in (0..8).rev() { bits.push((byte >> deslocamento) & 1 == 1); }
    }
    bits
}

/// Bits para bytes (inverso de `bytes_para_bits`). Só é chamada aqui sobre os 256 primeiros bits já
/// decodificados — sempre múltiplo de 8 por construção deste módulo, nunca por entrada externa direta.
fn bits_para_bytes(bits: &[bool]) -> Vec<u8> {
    debug_assert_eq!(bits.len() % 8, 0, "uso interno apenas: sempre chamada com múltiplo de 8 bits");
    bits.chunks(8).map(|grupo| grupo.iter().fold(0u8, |acc, &b| (acc << 1) | (b as u8))).collect()
}

/// Lê um grupo de bits (MSB primeiro) como um inteiro — usado para os 11 bits de cada palavra e para os
/// 8 bits do checksum.
fn bits_para_indice(bits: &[bool]) -> usize {
    bits.iter().fold(0usize, |acc, &b| (acc << 1) | (b as usize))
}

/// Escreve um índice como `n` bits (MSB primeiro) — o inverso de `bits_para_indice`.
fn indice_para_bits(indice: usize, n: u32) -> Vec<bool> {
    (0..n).rev().map(|deslocamento| (indice >> deslocamento) & 1 == 1).collect()
}

/// Codifica um segredo de `TAM_SEGREDO` bytes como `NUM_PALAVRAS` palavras da lista oficial. Infalível:
/// o tipo do parâmetro já garante o único jeito de errar (tamanho), então não há `Result` — "falhar
/// fechado" aqui significa não existir um caminho de erro para esquecer de tratar.
pub fn codificar(segredo: &[u8; TAM_SEGREDO]) -> [&'static str; NUM_PALAVRAS] {
    let checksum = Sha256::digest(segredo)[0];
    let mut bits = bytes_para_bits(segredo);
    bits.extend(bytes_para_bits(&[checksum]));
    debug_assert_eq!(bits.len(), TAM_SEGREDO * 8 + BITS_CHECKSUM);
    let mut palavras = [""; NUM_PALAVRAS];
    for (i, grupo) in bits.chunks(BITS_POR_PALAVRA as usize).enumerate() {
        palavras[i] = LISTA_DE_PALAVRAS[bits_para_indice(grupo)];
    }
    palavras
}

/// Decodifica `NUM_PALAVRAS` palavras de volta ao segredo de `TAM_SEGREDO` bytes, revalidando o checksum
/// embutido. As palavras devem vir exatamente como na lista oficial (minúsculas, sem acento, sem espaço
/// nas pontas) — normalizar o que a pessoa digitou é responsabilidade de quem chama, não deste módulo
/// (ver nota de responsabilidade no comentário do módulo).
pub fn decodificar(palavras: &[&str]) -> Result<[u8; TAM_SEGREDO], ErroDeCodec> {
    if palavras.len() != NUM_PALAVRAS { return Err(ErroDeCodec::QuantidadeDePalavrasErrada); }
    let mut bits = Vec::with_capacity(NUM_PALAVRAS * BITS_POR_PALAVRA as usize);
    for palavra in palavras {
        let indice = LISTA_DE_PALAVRAS.iter().position(|w| *w == *palavra).ok_or(ErroDeCodec::PalavraDesconhecida)?;
        bits.extend(indice_para_bits(indice, BITS_POR_PALAVRA));
    }
    let bits_segredo = &bits[..TAM_SEGREDO * 8];
    let bits_checksum = &bits[TAM_SEGREDO * 8..];
    let mut segredo = [0u8; TAM_SEGREDO];
    segredo.copy_from_slice(&bits_para_bytes(bits_segredo));
    let checksum_informado = bits_para_indice(bits_checksum) as u8;
    let checksum_esperado = Sha256::digest(segredo)[0];
    if checksum_informado != checksum_esperado { return Err(ErroDeCodec::ChecksumInvalido); }
    Ok(segredo)
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn lista_tem_2048_palavras_unicas() {
        assert_eq!(LISTA_DE_PALAVRAS.len(), TAM_LISTA);
        let unicas: HashSet<&str> = LISTA_DE_PALAVRAS.iter().copied().collect();
        assert_eq!(unicas.len(), TAM_LISTA, "não pode haver palavra repetida na lista");
    }

    #[test]
    fn lista_e_ascii_e_bate_com_as_pontas_conferidas_na_fonte() {
        assert!(LISTA_DE_PALAVRAS.iter().all(|p| p.is_ascii() && !p.is_empty()), "propriedade genuína do padrão oficial, não um defeito de transcrição");
        assert_eq!(LISTA_DE_PALAVRAS[0], "abacate");
        assert_eq!(LISTA_DE_PALAVRAS[TAM_LISTA - 1], "zumbido");
    }

    #[test]
    fn lista_esta_em_ordem_alfabetica() {
        let mut ordenada = LISTA_DE_PALAVRAS.to_vec();
        ordenada.sort_unstable();
        assert_eq!(LISTA_DE_PALAVRAS.to_vec(), ordenada, "ordem alfabética é propriedade da lista oficial — divergência aqui indicaria cópia corrompida");
    }

    #[test]
    fn tamanho_bate_com_o_modulo_recuperacao() {
        assert_eq!(TAM_SEGREDO, crate::recuperacao::TAM_CHAVE_DA_FRASE, "este codec só faz sentido lendo/escrevendo exatamente o segredo da frase pessoal");
    }

    #[test]
    fn ida_e_volta_preserva_o_segredo() {
        for segredo in [[0u8; 32], [0xFFu8; 32], {
            let mut s = [0u8; 32];
            for (i, b) in s.iter_mut().enumerate() { *b = i as u8; }
            s
        }] {
            let palavras = codificar(&segredo);
            assert_eq!(palavras.len(), NUM_PALAVRAS);
            let refs: Vec<&str> = palavras.to_vec();
            assert_eq!(decodificar(&refs).unwrap(), segredo);
        }
    }

    #[test]
    fn checksum_pega_erro_de_transcricao_numa_palavra_do_meio() {
        let segredo = {
            let mut s = [0u8; 32];
            for (i, b) in s.iter_mut().enumerate() { *b = (i as u8).wrapping_mul(37).wrapping_add(11); }
            s
        };
        let palavras = codificar(&segredo);
        let mut alteradas: Vec<&str> = palavras.to_vec();
        // Troca a primeira palavra (bits do segredo, nunca do checksum) por outra palavra válida mas
        // diferente — simula o erro humano mais comum: lembrar ou copiar a palavra errada.
        alteradas[0] = if alteradas[0] == LISTA_DE_PALAVRAS[0] { LISTA_DE_PALAVRAS[1] } else { LISTA_DE_PALAVRAS[0] };
        assert_eq!(decodificar(&alteradas).unwrap_err(), ErroDeCodec::ChecksumInvalido);
    }

    #[test]
    fn rejeita_quantidade_errada_de_palavras() {
        let segredo = [7u8; 32];
        let palavras = codificar(&segredo);
        let poucas: Vec<&str> = palavras[..23].to_vec();
        assert_eq!(decodificar(&poucas).unwrap_err(), ErroDeCodec::QuantidadeDePalavrasErrada);
        let mut muitas: Vec<&str> = palavras.to_vec();
        muitas.push("abacate");
        assert_eq!(decodificar(&muitas).unwrap_err(), ErroDeCodec::QuantidadeDePalavrasErrada);
    }

    #[test]
    fn rejeita_palavra_fora_da_lista() {
        let segredo = [3u8; 32];
        let palavras = codificar(&segredo);
        let mut alteradas: Vec<&str> = palavras.to_vec();
        assert!(!LISTA_DE_PALAVRAS.contains(&"abracadabra9000"), "a palavra de teste precisa estar genuinamente fora da lista oficial");
        alteradas[5] = "abracadabra9000";
        assert_eq!(decodificar(&alteradas).unwrap_err(), ErroDeCodec::PalavraDesconhecida);
    }

    #[test]
    fn e_insensivel_a_qual_segredo_zero_produz_primeira_palavra_conhecida() {
        // Trinta e dois bytes zero → os primeiros 11 bits são todos zero → índice 0 → primeira palavra
        // da lista. Fixa, por construção do algoritmo — não uma coincidência da lista.
        let palavras = codificar(&[0u8; 32]);
        assert_eq!(palavras[0], LISTA_DE_PALAVRAS[0]);
    }
}
