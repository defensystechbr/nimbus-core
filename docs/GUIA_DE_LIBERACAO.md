# Guia de liberação — sem todos os portões, nada roda

**Versão 37 · Para os engenheiros · Siga na ordem; nenhum passo é opcional**

O servidor e os apps só funcionam com um **selo de liberação válido** para o artefato exato que está rodando (módulo `selo`, conferido pela sentinela na partida). O selo só existe se **todos os 17 portões** passaram e se ele tiver a assinatura de **três chaves diferentes** fixadas no código: CI, responsável de segurança e auditor independente de fora da empresa. Se qualquer passo deste guia não for feito — inclusive ligar a proteção do repositório —, o CI não emite o selo e o sistema não anda.

## 1. Os 17 portões do selo

| Portão | Quem confere | Como |
|---|---|---|
| `testes` | CI, tarefa `verificacao` | `./verificar.sh --completo` (mínimo de testes fixado pela catraca) |
| `mutacao` | CI, `verificacao` | Todos os defeitos plantados pegos |
| `fuzzing` | CI, `verificacao` | 2 milhões de entradas por leitor, nenhum pânico |
| `tempo_constante` | CI, `verificacao` | PIN real, de coação e errado no mesmo tempo |
| `catraca` | CI, `catraca` | Nenhuma proteção a menos que antes |
| `varredura` | CI, `verificacao` | 47 regras próprias, zero ocorrências |
| `segredos` | CI, `segredos` | gitleaks em toda a história do repositório |
| `dependencias` | CI, `verificacao` | Nenhuma vulnerabilidade conhecida em aberto |
| `banco_rls` | CI, `banco` | Testes de RLS e sentinela do banco num PostgreSQL real |
| `banco_mutacao` | CI, `banco` | Todos os defeitos plantados no esquema pegos |
| `sentinela` | CI, `verificacao` | Autoverificação e testes da sentinela |
| `procedencia` | CI, `procedencia` | Certificado de procedência gerado |
| `consistencia_dos_documentos` | CI, `verificacao` | Documentos conferem com o protótipo e o núcleo |
| `protecao_do_repositorio` | CI, `governanca`, na API do GitHub | A regra ativa é igual à do código |
| `commits_assinados` | CI, `governanca`, na API do GitHub | Assinatura do commit verificada pelo GitHub |
| `revisao_de_duas_pessoas` | CI, `governanca`, na API do GitHub | 2 aprovações finais de pessoas diferentes do autor |
| `administracao_dupla` | CI, `governanca`, na API do GitHub | Organização exige segundo fator de todos; pelo menos 2 administradores; equipe de segurança com pelo menos 2 membros |

Os portões de governança são conferidos na API do GitHub — não na palavra de ninguém — e de novo na própria tarefa de liberação.

## 2. Configuração única (antes do primeiro código)

1. **Organização e equipes** no GitHub: `@nimbus/seguranca` (responsável de segurança e substituto — no mínimo 2 pessoas) e `@nimbus/nucleo`. **Pelo menos 2 administradores da organização**, e nenhum deles é a única pessoa da equipe de segurança. O portão `administracao_dupla` confere isso na API a cada liberação e todo dia.
2. **Segundo fator obrigatório na organização** (o portão confere) e **chave física (FIDO2) para cada administrador e membro da segurança**. A API do GitHub não informa qual segundo fator cada pessoa usa; por isso, a chave física é conferida por processo: cada administrador registra, num termo assinado guardado pelo responsável de segurança, o modelo e o número de série da chave, e a revisão trimestral confere na tela de segurança da conta de cada um.
3. **Suba o repositório** a partir do kit (`git init`, primeiro commit **assinado**).
4. **Aplique a proteção da branch** (um administrador, uma vez):
   `sh .github/regras/aplicar_protecao.sh dono/repositorio`
5. **Crie o token do vigia**: token de granularidade fina, permissões **somente leitura** em "Administration" do repositório e em "Members" e "Administration" da organização. Guarde como segredo `PROTECAO_LEITURA` do repositório.
6. **Gere as sementes dos três assinantes do selo**, cada uma numa máquina diferente, desconectada da internet, por pessoas diferentes:
   `cargo run --example selo -- gerar-semente ci.semente` (CI)
   `cargo run --example selo -- gerar-semente seguranca.semente` (responsável de segurança)
   `cargo run --example selo -- gerar-semente auditoria.semente` (auditor independente, de fora da empresa — obrigatório)
   Guarde cada semente em cofre offline (papel em cofre físico ou chave física), **nunca** em repositório, servidor, nuvem ou variável de ambiente. O `.gitignore` já bloqueia `*.semente`.
7. **Fixe as chaves públicas no código do servidor e dos apps**: `cargo run --example selo -- publica <semente>` mostra cada uma; elas entram como constantes no código (mudança com duas aprovações).
8. **Crie o ambiente `liberacao`** no repositório, com **dois revisores obrigatórios**, a opção que **impede quem disparou de aprovar a própria execução** e só a branch `principal`; coloque nele o segredo `SELO_SEMENTE_CI` (conteúdo do `ci.semente`) e apague a semente da máquina.
8b. **Ative o Renovate** (`renovate.json` já está no repositório): atualizações de dependência chegam como pedidos de mudança, nunca automáticas, só depois de 14 dias da publicação (30 dias para as bibliotecas de criptografia), e passam por todos os portões.
8c. **A sentinela diária** (`sentinela-diaria.yml`) já roda sozinha: verificação completa com a base de vulnerabilidades do dia, banco e governança. Se reprovar, nada novo é liberado até a correção.
9. **Ligue o servidor com a sentinela**: na partida e a cada hora, `sentinela::autoverificar_com_selo` e `servidor/banco/verificar_banco_na_partida.sql`. Com qualquer achado, o servidor não atende e envia o relatório ao responsável de segurança.

## 3. A cada liberação

1. O pedido de mudança é aprovado por duas pessoas (a regra impede menos).
2. O CI roda todas as tarefas; só se todas passarem, a tarefa `liberacao` pede aprovação humana para rodar, refaz a governança, monta o selo e assina com a chave do CI.
3. O responsável de segurança e, depois, o auditor independente baixam o artefato e o selo e, **cada um por conta própria**, refazem a governança (`python3 verificacao/portoes_de_governanca.py` com token próprio de leitura) e conferem o selo:
   `cargo run --example selo -- conferir selo.txt nimbus-fonte.tar.gz chaves.txt <versão mínima> <hora>`
   Só se tudo bater, cada um assina offline:
   `cargo run --example selo -- assinar selo.txt seguranca.semente` e `cargo run --example selo -- assinar selo.txt auditoria.semente`
   (A conferência recusa com menos de 3 assinaturas: é esperado até a última.)
4. O selo completo vai junto do artefato. O servidor confere na partida; os apps só são enviados às lojas com selo completo, e o servidor recusa versão de app sem selo registrado.
5. Cada selo entra no registro público de versões.

## 4. O que acontece quando algo é pulado

| Se... | Então |
|---|---|
| A proteção da branch não foi aplicada ou foi afrouxada | Portão `protecao_do_repositorio` reprova; sem selo; nada roda; o vigia diário também reprova |
| Alguém fez commit sem assinatura | Portão `commits_assinados` reprova |
| A mudança entrou com menos de duas aprovações | Portão `revisao_de_duas_pessoas` reprova |
| Um teste, regra ou defeito plantado foi apagado | Catraca reprova |
| O artefato foi trocado depois do selo | O servidor recusa na partida (artefato diferente) |
| Alguém tenta voltar a uma versão antiga com falha conhecida | Recusado (versão mínima) |
| Uma ou duas pessoas assinam o selo | Recusado (menos de três chaves diferentes) |
| A organização tem um administrador só, ou uma pessoa só na segurança | Portão `administracao_dupla` reprova |
| Uma dependência nova é atualizada sozinha | Não acontece: Renovate sem fusão automática, com espera e revisão de duas pessoas |

## 5. O que este guia NÃO consegue impedir — dito com verdade

- **Conluio:** se as três pessoas donas das chaves do selo (uma delas de fora da empresa) combinarem com quem consegue mexer no GitHub, elas conseguem liberar algo ruim. As defesas são a independência dessas pessoas, o registro público de cada selo e a auditoria periódica. A tabela "quem consegue burlar" está no Registro, seção 8.
- **Quem tem acesso de administrador à máquina de produção** pode rodar outro programa, e um programa conferindo o próprio selo não prova nada (ataque vermelho, A7). O selo protege o caminho oficial; a máquina se protege com acesso de duas pessoas, imagens assinadas conferidas pelo orquestrador (fora do programa) e computação confidencial com atestação.
- **Quem tem administração total do GitHub** pode desligar regras; o vigia e o portão de proteção acusam, mas a confiança final está nas pessoas com esse poder — por isso, no mínimo duas, com chave física, e nenhuma delas sozinha no time de segurança.
- **O próprio GitHub** é um fornecedor: se ele falhar, as conferências na API falham junto. A procedência assinada e o registro público ajudam a perceber.

## 6. Verificar antes de usar
1. Os nomes dos campos da regra (`protecao-principal.json`) na documentação atual da API de rulesets do GitHub.
2. A configuração de ambientes com revisores obrigatórios no plano do GitHub contratado.
3. Que o certificado de procedência funciona no plano contratado (repositório privado exige Enterprise Cloud).
