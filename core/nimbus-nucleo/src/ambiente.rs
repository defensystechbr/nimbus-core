//! # Módulo `ambiente` — diagnóstico do aparelho, só local, sem coletar nada (AMB1–AMB4)
//!
//! O app confere o ambiente em que está rodando e orienta o dono — **sem varrer outros apps, sem enviar
//! nada a ninguém e sem SDK de terceiros**. Os sinais vêm de interfaces públicas do sistema (Android e iOS),
//! ficam no aparelho e só viram duas coisas: avisos na tela "Ambiente seguro" e um teto de nível local.
//!
//! Por que não "varrer o aparelho inteiro": no iPhone o sistema não permite, e no Android listar todos os
//! apps é restrito pelas regras da loja e seria, em si, coleta de dado sensível. Um antivírus embarcado que
//! inspeciona o aparelho é exatamente o tipo de software que a privacidade do Nimbus recusa.

use crate::integridade::Nivel;

/// Sinais locais lidos das interfaces do sistema. Nenhum deles sai do aparelho.
#[derive(Debug, Clone, Copy, Default)]
pub struct SinaisLocais {
    /// Bloqueio de tela com senha, PIN ou biometria configurado.
    pub bloqueio_de_tela: bool,
    /// Dias desde o último patch de segurança do sistema (Android: SECURITY_PATCH; iOS: versão).
    pub dias_desde_o_patch: u32,
    /// Teclado ativo não é o do sistema.
    pub teclado_de_terceiros: bool,
    /// Opções de desenvolvedor ligadas.
    pub opcoes_de_desenvolvedor: bool,
    /// Instalação de apps de fontes desconhecidas permitida para algum app (Android).
    pub fontes_desconhecidas: bool,
    /// Cópia de segurança do sistema inclui dados de apps sem cifra de ponta a ponta (quando o sistema informa).
    pub backup_sem_cifra: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aviso { pub codigo: &'static str, pub texto: &'static str, pub como_resolver: &'static str }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostico { pub teto: Nivel, pub avisos: Vec<Aviso> }

/// Patch com mais de 90 dias rebaixa o aparelho a amarelo.
pub const DIAS_MAXIMOS_SEM_PATCH: u32 = 90;

/// Avalia o ambiente. Sem bloqueio de tela, nada fica acima de vermelho: o chip do aparelho não protege
/// chave nenhuma se qualquer um desbloqueia o celular.
pub fn avaliar(s: &SinaisLocais) -> Diagnostico {
    let mut avisos = Vec::new();
    let mut teto = Nivel::Verde;
    let mut rebaixar = |n: Nivel| { if (n as u8) > (teto as u8) { teto = n; } };
    if !s.bloqueio_de_tela {
        rebaixar(Nivel::Vermelho);
        avisos.push(Aviso { codigo: "AMB-BLOQUEIO", texto: "Seu celular está sem bloqueio de tela.", como_resolver: "Ative senha, PIN ou biometria nos ajustes do sistema." });
    }
    if s.dias_desde_o_patch > DIAS_MAXIMOS_SEM_PATCH {
        rebaixar(Nivel::Amarelo);
        avisos.push(Aviso { codigo: "AMB-PATCH", texto: "O sistema do seu celular está sem atualização de segurança há mais de 90 dias.", como_resolver: "Atualize o sistema; se o fabricante não atualiza mais, considere trocar de aparelho." });
    }
    if s.teclado_de_terceiros {
        rebaixar(Nivel::Amarelo);
        avisos.push(Aviso { codigo: "AMB-TECLADO", texto: "Você usa um teclado de terceiros, que pode registrar o que é digitado.", como_resolver: "Nos campos sensíveis o Nimbus usa o próprio teclado; para o resto, prefira o teclado do sistema." });
    }
    if s.opcoes_de_desenvolvedor {
        rebaixar(Nivel::Amarelo);
        avisos.push(Aviso { codigo: "AMB-DEV", texto: "As opções de desenvolvedor estão ligadas.", como_resolver: "Desligue as opções de desenvolvedor nos ajustes do sistema." });
    }
    if s.fontes_desconhecidas {
        rebaixar(Nivel::Amarelo);
        avisos.push(Aviso { codigo: "AMB-FONTES", texto: "Algum app tem permissão para instalar outros apps de fora da loja.", como_resolver: "Retire essa permissão nos ajustes do sistema." });
    }
    if s.backup_sem_cifra {
        avisos.push(Aviso { codigo: "AMB-BACKUP", texto: "A cópia de segurança do sistema não é cifrada de ponta a ponta.", como_resolver: "O Nimbus não entra nessa cópia; para os outros apps, ative a cópia cifrada do sistema." });
    }
    Diagnostico { teto, avisos }
}

/// Nível final: o mais restritivo entre o da integridade e o teto do ambiente.
pub fn combinar(integridade: Nivel, d: &Diagnostico) -> Nivel {
    if (d.teto as u8) > (integridade as u8) { d.teto } else { integridade }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn bom() -> SinaisLocais { SinaisLocais { bloqueio_de_tela: true, dias_desde_o_patch: 10, ..Default::default() } }

    #[test]
    fn aparelho_bem_cuidado_fica_verde_e_sem_avisos() {
        let d = avaliar(&bom());
        assert_eq!(d, Diagnostico { teto: Nivel::Verde, avisos: vec![] });
        assert_eq!(combinar(Nivel::Verde, &d), Nivel::Verde);
    }

    #[test]
    fn cada_risco_rebaixa_e_diz_como_resolver() {
        let casos: [(SinaisLocais, &str, Nivel); 5] = [
            (SinaisLocais { bloqueio_de_tela: false, ..bom() }, "AMB-BLOQUEIO", Nivel::Vermelho),
            (SinaisLocais { dias_desde_o_patch: DIAS_MAXIMOS_SEM_PATCH + 1, ..bom() }, "AMB-PATCH", Nivel::Amarelo),
            (SinaisLocais { teclado_de_terceiros: true, ..bom() }, "AMB-TECLADO", Nivel::Amarelo),
            (SinaisLocais { opcoes_de_desenvolvedor: true, ..bom() }, "AMB-DEV", Nivel::Amarelo),
            (SinaisLocais { fontes_desconhecidas: true, ..bom() }, "AMB-FONTES", Nivel::Amarelo),
        ];
        for (s, codigo, nivel) in casos {
            let d = avaliar(&s);
            assert_eq!(d.teto, nivel, "{codigo}");
            assert!(d.avisos.iter().any(|a| a.codigo == codigo && !a.como_resolver.is_empty()), "{codigo}");
            assert_eq!(combinar(Nivel::Verde, &d), nivel);
        }
        assert_eq!(avaliar(&SinaisLocais { dias_desde_o_patch: DIAS_MAXIMOS_SEM_PATCH, ..bom() }).teto, Nivel::Verde, "90 dias ainda vale");
        let b = avaliar(&SinaisLocais { backup_sem_cifra: true, ..bom() });
        assert_eq!(b.teto, Nivel::Verde, "backup do sistema só gera aviso: o Nimbus não entra nele");
        assert_eq!(b.avisos.len(), 1);
        // O mais restritivo vence: integridade vermelha não melhora por ambiente bom.
        assert_eq!(combinar(Nivel::Vermelho, &avaliar(&bom())), Nivel::Vermelho);
        // Sem bloqueio, vermelho mesmo com tudo o mais perfeito.
        assert_eq!(avaliar(&SinaisLocais { bloqueio_de_tela: false, ..bom() }).teto, Nivel::Vermelho);
    }
}
