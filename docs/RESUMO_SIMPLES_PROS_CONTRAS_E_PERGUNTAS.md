# NimbusChat em linguagem simples

Este texto é para quem não é da área técnica. Ele explica o que o projeto é, o que está bom, o que preocupa, o que precisa ser decidido e os passos para executar. Onde há uma palavra difícil, ela vem explicada ao lado.

---

## 1. O que é o NimbusChat

Um aplicativo de mensagens (como o WhatsApp) feito para que **ninguém consiga ler as conversas, nem a própria empresa**. A ideia é guardar o mínimo possível. Se uma autoridade pedir dados, a empresa entrega o que existe, e quase nada existe.

**Em que ponto estamos hoje**

Pense numa casa. O que está pronto é **o cofre**: a parte que tranca e destranca as mensagens. Ele foi muito bem testado. **Ainda não existem** as paredes e o telhado: o aplicativo de celular, o servidor que leva as mensagens de um celular a outro e a parte de hospedagem. Ou seja, o projeto ainda não é um aplicativo que se possa instalar e usar.

---

## 2. O que está bom (prós)

1. **O cofre é sólido.** Foi testado de muitas formas, inclusive com erros plantados de propósito para ver se os testes os pegavam. Todos foram pegos.
2. **Usa criptografia conhecida.** Não inventou nada novo. Criptografia inventada em casa costuma ter falhas escondidas, e a conhecida já foi estudada por muita gente.
3. **Pensa no futuro.** A criptografia escolhida resiste até a computadores quânticos, que ainda não existem, mas poderão existir.
4. **Diz a verdade sobre os limites.** Por exemplo: se o celular estiver desbloqueado e com vírus, o vírus vê a tela. Nenhum sistema evita isso, e o projeto avisa.
5. **Já foi atacado de propósito.** Foram feitos 12 ataques contra o próprio sistema, e 6 falhas reais foram achadas e corrigidas.
6. **O código será aberto.** Qualquer pessoa pode conferir se o aplicativo faz o que promete. Isso combina com a ideia de "não temos nada a esconder".
7. **Telas já desenhadas.** São 66 telas de exemplo, o que evita retrabalho depois.

---

## 3. O que preocupa (contras)

| # | Preocupação | Explicação simples | Quão sério |
|---|---|---|---|
| 1 | Nada está ligado ainda | O cofre existe, mas nenhum celular o usa. O trabalho difícil é ligar as peças. | Alto |
| 2 | Muitas regras para cumprir | Existem 49 regras de segurança para o aplicativo de celular. Cada uma exige código e teste num celular de verdade. Muitas são de recursos que o MVP nem terá. | Alto |
| 3 | Conflito com a ferramenta escolhida | Uma regra manda **apagar da memória** os segredos assim que acabar de usar. A ferramenta escolhida para as telas (Kotlin/Compose) guarda textos na memória e não deixa apagar de verdade. | Alto |
| 4 | Esconder o endereço de internet do usuário | O projeto promete que o servidor não vê de onde o usuário vem. Isso só funciona com **duas empresas diferentes** no meio do caminho. Se for tudo no mesmo servidor nosso, a promessa não vale. | Alto |
| 5 | Verificação de aparelho | O app confere se o celular é "confiável" usando serviços do Google e da Apple. Celulares com sistema alternativo, muito usados por quem quer privacidade, podem ser barrados. | Médio a alto |
| 6 | Chaves que mudam todo dia | Por segurança, endereços e chaves mudam diariamente. Falta definir o que acontece se o celular ficar dias desligado ou com a hora errada. | Alto |
| 7 | Depender da hora do servidor | O prazo das mensagens que somem usa a hora que o servidor informa. Se o servidor cair, o que acontece? Não está definido. | Médio |
| 8 | Bateria | Para dificultar espionagem, o app envia tráfego falso o tempo todo. O gasto de bateria ainda não foi medido. | Médio |
| 9 | Guardar dados no celular | A regra diz que toda criptografia deve ficar no "cofre". Ferramentas comuns para guardar dados no celular quebrariam essa regra. | Médio |
| 10 | Quem confere o trabalho | Boa parte será escrita por IA. Se quem aprova não entende o que lê, a revisão não vale. Só uma auditoria humana especializada confirma que o sistema é seguro. | Alto |
| 11 | Notificações | Para avisar "chegou mensagem", o iPhone passa pela Apple e o Android pelo Google. Eles não veem o conteúdo, mas sabem que houve um aviso. | Médio |
| 12 | Loja e leis | Para publicar nas lojas e operar no Brasil são necessários advogado e ajustes (denúncia de conteúdo, guarda de registros). Fica fora do teste inicial. | Alto, mas só depois |
| 13 | Muito papel, pouco código | Há muita documentação e algumas cópias repetidas dos mesmos arquivos, o que causa confusão. | Médio |
| 14 | A pasta não está no Git | Git é o sistema que guarda o histórico de mudanças e exige aprovação de duas pessoas. Sem ele, não dá para aplicar a regra de segurança do projeto. | Médio |

---

## 4. Decisões já tomadas

| Tema | Decisão |
|---|---|
| Quem faz | Você e 1 ou 2 desenvolvedores, com a IA escrevendo a maior parte do código |
| Objetivo | Teste privado com um grupo pequeno, sem publicar nas lojas |
| Código | Aberto |
| Celulares | Android e iPhone ao mesmo tempo |
| Telas | Uma só base de telas para os dois sistemas (Kotlin Multiplatform + Compose) |
| Prazo | Cerca de 3 meses para a primeira parte. Está apertado |
| Primeira parte (Marco A) | Mensagens de texto entre duas pessoas, mensagens que somem, PIN de emergência |
| Segunda parte (Marco B) | Grupos e envio de fotos e arquivos |
| Servidor | Em máquina virtual própria, no Proxmox |
| Avisos no celular | Usar o serviço da Apple e do Google, sem conteúdo |

---

## 5. Perguntas que precisam de resposta

Para cada uma, **indico a opção que recomendo**. Se não souber, responda "pode seguir a recomendação" que eu aplico.

**Sobre segurança e promessas**

1. **O endereço de internet do usuário ficará escondido do servidor?**
   - A) No teste inicial, não. O servidor e o intermediário são da mesma empresa, e escrevemos isso com clareza nas telas. *(recomendado)*
   - B) Sim, usando uma segunda empresa para o intermediário. Custa mais e demora mais.

2. **Como cumprir a regra de "apagar segredos da memória" com a ferramenta de telas escolhida?**
   - A) Os segredos ficam só dentro do cofre e as telas só recebem textos curtos e temporários. *(recomendado)*
   - B) Reescrever a regra e avisar o usuário do limite.
   - C) Trocar a ferramenta de telas por uma nativa de cada celular.

3. **Celulares com sistema alternativo (por exemplo, GrapheneOS) podem usar o app?**
   - A) Sim, com aviso de nível de segurança. *(recomendado para o teste)*
   - B) Não. Só aparelhos verificados pelo Google e pela Apple.

4. **O app gasta bateria para enviar tráfego falso e dificultar espionagem. Entra no teste inicial?**
   - A) Não. Fica para depois da medição. *(recomendado)*
   - B) Sim, e medimos o gasto desde o início.

**Sobre como o app funciona**

5. **Se um celular ficar mais de 2 dias desligado ou com a hora errada, o que acontece?** Precisamos de uma regra. Sugestão: o app pede para entrar de novo, as mensagens antigas que já venceram são perdidas e o usuário é avisado.
   - A) Aceito a sugestão. *(recomendado)*
   - B) Quero outra regra.

6. **Se o servidor ficar fora do ar, as mensagens que somem...**
   - A) Continuam bloqueadas até o servidor voltar. É mais seguro. *(recomendado)*
   - B) Somem pelo relógio do celular. É mais prático, mas menos confiável.

7. **Como guardar as mensagens no celular?** A regra do projeto exige que o cofre faça toda a criptografia. Aceita que a equipe técnica desenhe isso dentro do cofre, mesmo que demore mais?
   - A) Sim. *(recomendado)*
   - B) Não, usar uma ferramenta pronta e registrar a exceção.

**Sobre o servidor**

8. **Onde guardar as chaves mais importantes do servidor?**
   - A) Em um programa de cofre de senhas (Vault) no próprio servidor. *(recomendado para o teste)*
   - B) Em um equipamento físico seguro (HSM). Mais seguro e mais caro.

9. **Quem guardará as chaves-mestras, e quantas pessoas são necessárias para usá-las?** Sugestão: 3 pessoas guardam, e 2 precisam concordar.
   - A) Aceito a sugestão. *(recomendado)*
   - B) Prefiro outro formato.

**Sobre o time e o processo**

10. **Quem é o revisor técnico?** Precisamos do nome de uma pessoa que consiga ler o código principal de segurança e aprovar. Se ninguém consegue, será preciso contratar uma revisão pontual.
11. **Quem é o segundo desenvolvedor e quando começa?**
12. **Onde ficará o código?** Precisamos criar o repositório (por exemplo, no GitHub) e ligar a verificação automática de qualidade.
    - A) GitHub privado agora, aberto depois. *(recomendado)*
    - B) Servidor próprio de código (Gitea/GitLab).
13. **Quem formará o grupo de teste, quantas pessoas e que celulares usam?** Sugestão: 5 a 8 pessoas conhecidas, com pelo menos 2 marcas diferentes de Android e 1 iPhone.
14. **O prazo de 3 meses ficou apertado depois que o PIN de emergência entrou. O que prefere?**
    - A) Estender para 4 meses. *(recomendado)*
    - B) Tirar outro item para manter 3 meses.

---

## 6. Passo a passo, em linguagem simples

**Fase 0: Preparar (semanas 0 a 2)**
1. Anotar por escrito as decisões da seção 4 e as respostas da seção 5.
2. Criar o repositório e exigir que toda mudança seja aprovada por **duas pessoas**.
3. Escolher quem revisa e quem é o segundo desenvolvedor.
4. Abrir as contas necessárias (Apple Developer e Google Play ou distribuição direta).

**Fase 1: Testar o que mais pode dar errado (semanas 2 a 5)**
5. Fazer o cofre funcionar dentro de um aplicativo simples no Android e no iPhone.
6. Testar em celulares de verdade: notificação, proteção contra captura de tela e tamanho do aplicativo.
7. Escrever um relatório curto com o que funcionou e o que não funcionou.
8. **Decisão:** seguir, ajustar ou parar. Se a base não funcionar, é melhor saber agora.

**Fase 2: Servidor básico (semanas 4 a 8)**
9. Criar o servidor que guarda mensagens trancadas por no máximo 48 horas.
10. Conferir que ele recusa tudo o que não for permitido.
11. Conferir que o banco de dados não guarda nada legível.

**Fase 3: Conta e conversa entre duas pessoas (semanas 7 a 11)**
12. Criar conta, PIN e frase de recuperação, sem pedir telefone nem e-mail.
13. Convidar um contato e confirmar que é a pessoa certa.
14. Enviar e receber mensagens cifradas entre um Android e um iPhone.
15. PIN de emergência: ao digitar o PIN falso, o app abre um cofre de mentira.

**Fase 4: Mensagens que somem e entrega do teste (semanas 10 a 13)**
16. Mensagens que desaparecem no prazo.
17. Entregar o app ao grupo de teste (arquivo de instalação no Android e TestFlight no iPhone).
18. Rodar todas as verificações automáticas.
19. Testar com 5 a 8 pessoas, sem ajudar, por 2 semanas.
20. **Decisão:** seguir para a segunda parte, corrigir ou parar.

**Fase 5: Grupos e envio de arquivos (meses 4 a 6)**
21. Conversas em grupo.
22. Envio de fotos e arquivos.
23. Mais testes e revisão por outra pessoa.

**Fase 6: Só se for abrir além do grupo de teste**
24. Auditoria independente de segurança.
25. Advogado para os Termos, a Política de Privacidade e as obrigações legais.
26. Duas empresas independentes para esconder o endereço dos usuários.
27. Orçamento aprovado para tudo isso.

---

## 7. Como trabalhar com a IA sem se perder

1. **Peça uma coisa por vez** e peça o plano antes do código.
2. **Mudanças pequenas.** Quanto menor, mais fácil de revisar de verdade.
3. **Os testes vêm primeiro.** Um teste só vale se foi visto falhando antes de o código existir.
4. **A IA nunca altera as regras de segurança nem os testes** para fazer algo passar. Se ela pedir isso, pare e pergunte.
5. **Nenhuma senha ou dado real** entra em conversa com a IA.
6. **Quem aprova precisa entender o que está aprovando.** Duas pessoas humanas. A IA não conta.
7. **Cada fase termina com algo rodando num celular de verdade**, não com mais um documento.

---

## 8. Resumo em uma frase

O projeto tem uma base técnica forte, mas ainda não é um aplicativo. Se cada fase terminar com algo funcionando em celulares reais e houver gente capaz de revisar o que a IA escreve, o teste privado é viável. Sem isso, o risco principal é gastar meses escrevendo sem nada que um usuário possa tocar.
