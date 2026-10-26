//! # Módulo `album` — álbuns e coleções automáticas (ALB1–ALB8)
//!
//! Uma foto ou vídeo pode entrar sozinho na coleção certa (Família, Trabalho, Favoritos e as que você criar).
//! Você marca a coleção ao subir, ou deixa uma **regra** decidir — por contato de origem, por marcação, ou
//! como padrão. **Tudo é cifrado no cofre** (módulo `cofre`): o nome da coleção, a legenda, quem aparece, a
//! data — nada disso existe em texto claro em lugar nenhum, nem no servidor. Este módulo só organiza; quem
//! cifra e guarda é o cofre.
//!
//! A visualização (grade estilo galeria, capa de cada coleção) é da camada de tela; aqui ficam a estrutura e
//! as regras, com testes.
use std::collections::BTreeMap;

/// Uma coleção. O `id` é interno; o `nome` é escolhido por você e guardado cifrado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Colecao { pub id: u32, pub nome: String, pub favorita: bool }

/// De onde veio a mídia, para as regras decidirem a coleção. Preenchido pelo app.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Origem {
    /// Identificador do contato que enviou (se veio de uma conversa).
    pub de_contato: Option<u32>,
    /// Marcação que você tocou ao subir (ex.: "trabalho").
    pub marcacao: Option<String>,
}

/// Regra de organização automática. A primeira que casar decide a coleção.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Regra {
    /// Mídia vinda deste contato vai para esta coleção.
    DoContato { contato: u32, colecao: u32 },
    /// Mídia com esta marcação vai para esta coleção.
    ComMarcacao { marcacao: String, colecao: u32 },
    /// Qualquer mídia sem outra regra vai para esta coleção (o "resto").
    Padrao { colecao: u32 },
}

/// As regras do usuário, na ordem em que devem ser testadas.
#[derive(Clone, Debug, Default)]
pub struct Regras { pub regras: Vec<Regra> }

impl Regras {
    /// Decide em qual coleção a mídia entra. `None` = nenhuma regra casou (o app pergunta ou deixa "sem coleção").
    pub fn escolher(&self, origem: &Origem) -> Option<u32> {
        for r in &self.regras {
            match r {
                Regra::DoContato { contato, colecao } if origem.de_contato == Some(*contato) => return Some(*colecao),
                Regra::ComMarcacao { marcacao, colecao } if origem.marcacao.as_deref() == Some(marcacao) => return Some(*colecao),
                Regra::Padrao { colecao } => return Some(*colecao),
                _ => {}
            }
        }
        None
    }
}

/// Conta quantas mídias há em cada coleção (para mostrar na grade). Entra tudo já decidido pelo app.
pub fn contar_por_colecao(colecoes_das_midias: &[u32]) -> BTreeMap<u32, usize> {
    let mut m = BTreeMap::new();
    for &c in colecoes_das_midias { *m.entry(c).or_insert(0) += 1; }
    m
}

#[cfg(test)]
mod testes {
    use super::*;

    fn regras() -> Regras {
        Regras { regras: vec![
            Regra::DoContato { contato: 10, colecao: 1 },      // 1 = Família
            Regra::ComMarcacao { marcacao: "trabalho".into(), colecao: 2 }, // 2 = Trabalho
            Regra::Padrao { colecao: 3 },                       // 3 = Geral
        ] }
    }

    #[test]
    fn foto_do_contato_familia_vai_para_familia() {
        assert_eq!(regras().escolher(&Origem { de_contato: Some(10), marcacao: None }), Some(1));
    }

    #[test]
    fn foto_marcada_trabalho_vai_para_trabalho() {
        assert_eq!(regras().escolher(&Origem { de_contato: Some(99), marcacao: Some("trabalho".into()) }), Some(2));
    }

    #[test]
    fn a_primeira_regra_que_casa_vence() {
        // vem do contato 10 (Família) E marcada trabalho: a regra do contato está antes, então Família.
        assert_eq!(regras().escolher(&Origem { de_contato: Some(10), marcacao: Some("trabalho".into()) }), Some(1));
    }

    #[test]
    fn sem_marcacao_e_de_desconhecido_cai_no_padrao() {
        assert_eq!(regras().escolher(&Origem { de_contato: Some(77), marcacao: None }), Some(3));
    }

    #[test]
    fn sem_regra_padrao_pode_nao_casar_nada() {
        let r = Regras { regras: vec![Regra::DoContato { contato: 1, colecao: 1 }] };
        assert_eq!(r.escolher(&Origem { de_contato: Some(2), marcacao: None }), None);
    }

    #[test]
    fn contagem_por_colecao_para_a_grade() {
        let m = contar_por_colecao(&[1, 1, 2, 3, 3, 3]);
        assert_eq!((m[&1], m[&2], m[&3]), (2, 1, 3));
    }
}
