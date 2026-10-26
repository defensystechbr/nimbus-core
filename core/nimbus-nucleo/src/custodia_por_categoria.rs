//! # Módulo `custodia_por_categoria` — política de continuidade em servidor, por tipo de item (novo, 10/10/2026)
//!
//! Resposta ao pedido do dono de granularidade: em vez de um único interruptor "ligar backup" para o
//! cofre inteiro, cada `cofre::Tipo` (Foto, Vídeo, Áudio, Arquivo, Álbum, Nota, Compromisso, Senha, Ajuste
//! — a taxonomia já existente em `cofre.rs`, nenhuma categoria nova inventada aqui) tem sua própria
//! política: o dono decide, categoria por categoria, se aquele tipo de conteúdo também usa
//! `custodia_do_cofre` (2 de 2, aparelho+servidor) como complemento de continuidade, ou fica só no
//! aparelho.
//!
//! **Posição deste módulo, não só do dono — dada depois de pesar os dois lados, não por concordância
//! automática (pedido explícito do dono: "quero sua opinião"): o padrão de fábrica é só o aparelho, para
//! TODAS as nove categorias, sem exceção — inclusive Senha.** Argumento considerado e rejeitado: deixar
//! Senha com o complemento de servidor PRÉ-MARCADO por padrão, porque perder o gerenciador de senhas para
//! sempre é catastrófico e o conteúdo não é sensível a ordem judicial (são segredos, não fotos íntimas).
//! Isto parece razoável isoladamente, mas quebra a única regra que dá à Decisão 2 de
//! `ARQUITETURA_NIMBUSCHAT_E_NIMBUSONE.md` seu valor — "aparelho como raiz por padrão, sempre, SEM
//! EXCEÇÃO" — precisamente para a pessoa que nunca abre os ajustes (a maioria, e quem um padrão de
//! fábrica mais protege). Uma única categoria com padrão de servidor já reintroduz, por uma porta lateral,
//! a mesma dependência numa promessa de comportamento futuro da empresa que a Decisão 2 inteira existe
//! para evitar. **O que o módulo FAZ para compensar, sem comprometer o padrão:** `recomendadas_no_primeiro_
//! uso`, abaixo — um sinal de PRODUTO (nunca aplicado pelo núcleo) para destacar Senha e Compromisso na
//! tela de ajustes, com a razão econômica explicada ali mesmo. Recomendar com clareza não é o mesmo que
//! pré-marcar; a diferença é exatamente o que separa autonomia real de um padrão escondido atrás de uma
//! recomendação.
//!
//! Reaproveita `custodia_do_cofre` sem mudar nada nele: este módulo só decide QUAIS tipos de item, entre
//! os já guardados no cofre, têm o mecanismo 2 de 2 ativo — a própria divisão/reconstrução da chave do
//! cofre continua sendo uma única operação por conta. O que muda por categoria é só se um item daquele
//! tipo é incluído no que atravessa para o complemento de servidor (Decisão 3 de
//! `ARQUITETURA_NIMBUSCHAT_E_NIMBUSONE.md`: travessia deliberada, nunca espelho automático) ou fica,
//! desde o início, só localmente.
//!
//! **O que este módulo NÃO prova (fica para quem integrar):** nenhuma campanha de mutação própria ainda;
//! nenhuma tela real (ver guia visual desta mesma entrega); nenhuma ligação de fato com o momento em que
//! um item é salvo no cofre — este módulo só guarda e consulta a política, quem decide CHAMAR
//! `custodia_do_cofre` com base nela é a ponte.
use crate::cofre::Tipo;

/// Política de continuidade de UMA categoria. Dois estados só, de propósito — nunca um terceiro
/// "automático" disfarçado, que na prática seria um padrão de servidor escondido atrás de um nome bonito.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Politica {
    /// Nunca sai do aparelho por este mecanismo. Padrão de fábrica de TODAS as categorias, sem exceção.
    #[default]
    SoNoAparelho,
    /// Também usa `custodia_do_cofre` (2 de 2, aparelho+servidor) como complemento de continuidade —
    /// só depois de o dono ligar explicitamente para aquela categoria especificamente.
    ComComplementoDeServidor,
}

/// Uma política por categoria, para as nove de `cofre::Tipo`. Configuração do dono, não segredo: não
/// precisa ficar escondida do servidor, mas também não precisa viajar para lá — só o aparelho consulta,
/// ao decidir se chama `custodia_do_cofre` para um item recém-guardado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Politicas {
    foto: Politica,
    video: Politica,
    audio: Politica,
    arquivo: Politica,
    album: Politica,
    nota: Politica,
    compromisso: Politica,
    senha: Politica,
    ajuste: Politica,
}

impl Default for Politicas {
    /// Todas as nove categorias começam só no aparelho — sem exceção, nem para Senha (ver doc do módulo).
    fn default() -> Self {
        Politicas {
            foto: Politica::SoNoAparelho,
            video: Politica::SoNoAparelho,
            audio: Politica::SoNoAparelho,
            arquivo: Politica::SoNoAparelho,
            album: Politica::SoNoAparelho,
            nota: Politica::SoNoAparelho,
            compromisso: Politica::SoNoAparelho,
            senha: Politica::SoNoAparelho,
            ajuste: Politica::SoNoAparelho,
        }
    }
}

impl Politicas {
    pub fn para(&self, tipo: Tipo) -> Politica {
        match tipo {
            Tipo::Foto => self.foto,
            Tipo::Video => self.video,
            Tipo::Audio => self.audio,
            Tipo::Arquivo => self.arquivo,
            Tipo::Album => self.album,
            Tipo::Nota => self.nota,
            Tipo::Compromisso => self.compromisso,
            Tipo::Senha => self.senha,
            Tipo::Ajuste => self.ajuste,
        }
    }

    /// Única forma de mudar uma política: sempre explícita, uma categoria por vez — nunca um "ligar tudo"
    /// de uma vez só (isso recriaria, por atalho de interface, o interruptor único que esta rodada existe
    /// para substituir).
    pub fn definir(&mut self, tipo: Tipo, pol: Politica) {
        let campo = match tipo {
            Tipo::Foto => &mut self.foto,
            Tipo::Video => &mut self.video,
            Tipo::Audio => &mut self.audio,
            Tipo::Arquivo => &mut self.arquivo,
            Tipo::Album => &mut self.album,
            Tipo::Nota => &mut self.nota,
            Tipo::Compromisso => &mut self.compromisso,
            Tipo::Senha => &mut self.senha,
            Tipo::Ajuste => &mut self.ajuste,
        };
        *campo = pol;
    }

    pub fn usa_complemento_de_servidor(&self, tipo: Tipo) -> bool {
        self.para(tipo) == Politica::ComComplementoDeServidor
    }
}

/// Recomendação de PRODUTO, nunca aplicada pelo núcleo (ver doc do módulo): categorias onde vale destacar
/// a opção do complemento na tela de primeiro uso, com a razão econômica explicada na própria tela —
/// "perder isto para sempre costuma doer mais, e o conteúdo não costuma ser sensível a uma ordem judicial
/// contra o servidor sozinho". Isto é só um sinal de UX; nunca pré-marca nada em `Politicas::default()`.
pub fn recomendadas_no_primeiro_uso() -> &'static [Tipo] {
    &[Tipo::Senha, Tipo::Compromisso]
}

#[cfg(test)]
mod testes {
    use super::*;

    const TODAS: [Tipo; 9] = [
        Tipo::Foto, Tipo::Video, Tipo::Audio, Tipo::Arquivo, Tipo::Album,
        Tipo::Nota, Tipo::Compromisso, Tipo::Senha, Tipo::Ajuste,
    ];

    #[test]
    fn padrao_de_fabrica_e_so_no_aparelho_para_todas_as_categorias_sem_excecao() {
        let p = Politicas::default();
        for tipo in TODAS {
            assert_eq!(p.para(tipo), Politica::SoNoAparelho, "{tipo:?}");
            assert!(!p.usa_complemento_de_servidor(tipo), "{tipo:?}");
        }
    }

    #[test]
    fn definir_muda_so_a_categoria_pedida_as_demais_continuam_no_padrao() {
        let mut p = Politicas::default();
        p.definir(Tipo::Senha, Politica::ComComplementoDeServidor);
        assert!(p.usa_complemento_de_servidor(Tipo::Senha));
        for tipo in TODAS {
            if tipo != Tipo::Senha {
                assert!(!p.usa_complemento_de_servidor(tipo), "{tipo:?} não deveria ter mudado");
            }
        }
    }

    #[test]
    fn definir_pode_desligar_de_volta() {
        let mut p = Politicas::default();
        p.definir(Tipo::Foto, Politica::ComComplementoDeServidor);
        assert!(p.usa_complemento_de_servidor(Tipo::Foto));
        p.definir(Tipo::Foto, Politica::SoNoAparelho);
        assert!(!p.usa_complemento_de_servidor(Tipo::Foto));
    }

    #[test]
    fn recomendacao_de_primeiro_uso_nunca_pre_marca_nada() {
        // A própria existência da recomendação não pode, sozinha, mudar o padrão de fábrica — checado
        // lendo as duas coisas juntas, para que uma mudança futura em uma não quebre esta garantia em
        // silêncio.
        let p = Politicas::default();
        for tipo in recomendadas_no_primeiro_uso() {
            assert!(!p.usa_complemento_de_servidor(*tipo), "recomendar {tipo:?} não é o mesmo que pré-marcar");
        }
    }

    #[test]
    fn categorias_diferentes_podem_ter_politicas_diferentes_ao_mesmo_tempo() {
        let mut p = Politicas::default();
        p.definir(Tipo::Senha, Politica::ComComplementoDeServidor);
        p.definir(Tipo::Foto, Politica::SoNoAparelho);
        p.definir(Tipo::Nota, Politica::ComComplementoDeServidor);
        assert!(p.usa_complemento_de_servidor(Tipo::Senha));
        assert!(!p.usa_complemento_de_servidor(Tipo::Foto));
        assert!(p.usa_complemento_de_servidor(Tipo::Nota));
        assert!(!p.usa_complemento_de_servidor(Tipo::Video), "categoria nunca tocada continua no padrão");
    }
}
