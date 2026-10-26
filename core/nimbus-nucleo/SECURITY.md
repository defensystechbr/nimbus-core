# Política de segurança — Nimbus

## Como relatar uma falha
Escreva para o canal de segurança publicado no site oficial, cifrando com a chave pública ali divulgada.
Confirmamos em até 3 dias úteis, avaliamos em até 15 e damos crédito público se você quiser. Pesquisa de
boa-fé dentro do escopo publicado tem porto seguro: nenhuma ação legal. Prazo de divulgação: 90 dias.

## O que nunca pedimos
Frase de recuperação, PIN, palavras de confirmação, código de convite, cartão ou instalação fora da loja.

## Regras para quem mantém este código
1. Segredos nunca no repositório, em imagem, em registro ou em variável de ambiente de produção.
2. Nenhuma entrada vinda de fora é confiável: tudo passa pelos leitores do núcleo, com limite de tamanho.
3. Nenhum serviço de IA de terceiros recebe dado de usuário.
4. A catraca (`verificacao/catraca.py`) só deixa as proteções subirem.
5. Nada entra no código principal nem é publicado por uma pessoa só.
