//! # Módulo `memoria` — segredo em memória blindada (RZ3, RZ11, RZ12)
//!
//! Tudo o que é segredo ou conteúdo aberto passa por `Segredo`:
//! * **zerado** ao ser descartado (e em `limpar`);
//! * **travado na RAM** (mlock) onde o sistema permite, para não ir parar no arquivo de troca (swap);
//! * **nunca impresso**: `Debug` mostra só o tamanho, então conteúdo não vaza para registros nem para
//!   relatórios de falha (crash);
//! * acesso só por empréstimo curto (`usar`), sem cópias soltas.
//!
//! Limites declarados: iPhone e Android não fazem swap para disco (comprimem na própria RAM), então o
//! mlock importa sobretudo no computador. Em computador, a hibernação grava a RAM inteira em disco —
//! o mlock não impede isso; exigir disco cifrado do sistema (FileVault/BitLocker) na edição de computador.
use zeroize::Zeroize;

pub struct Segredo { dados: Vec<u8>, travado: bool }

#[cfg(unix)]
#[allow(unsafe_code)] // Único uso de `unsafe` permitido no núcleo (o resto é proibido em lib.rs).
fn travar(buf: &[u8]) -> bool {
    if buf.is_empty() { return false; }
    // Melhor esforço: pode falhar por limite do sistema (RLIMIT_MEMLOCK). Nunca é fatal.
    // SEGURANÇA: ponteiro e tamanho vêm de uma fatia válida e viva; mlock só lê o intervalo.
    unsafe { libc::mlock(buf.as_ptr() as *const libc::c_void, buf.len()) == 0 }
}
#[cfg(unix)]
#[allow(unsafe_code)] // Idem: par do mlock acima.
fn destravar(buf: &[u8]) { if !buf.is_empty() { let _ = unsafe { libc::munlock(buf.as_ptr() as *const libc::c_void, buf.len()) }; } }
#[cfg(not(unix))]
fn travar(_buf: &[u8]) -> bool { false }
#[cfg(not(unix))]
fn destravar(_buf: &[u8]) {}

impl Segredo {
    /// Cria já com o tamanho final (nunca cresce: crescer realocaria e deixaria cópia para trás).
    pub fn com_tamanho(tamanho: usize) -> Self {
        let dados = vec![0u8; tamanho];
        let travado = travar(&dados);
        Segredo { dados, travado }
    }

    /// Copia bytes para dentro da memória blindada. Quem chamou deve zerar a origem.
    pub fn de(bytes: &[u8]) -> Self {
        let mut s = Self::com_tamanho(bytes.len());
        s.dados.copy_from_slice(bytes);
        s
    }

    pub fn tamanho(&self) -> usize { self.dados.len() }
    pub fn travado_na_ram(&self) -> bool { self.travado }

    /// Empréstimo de leitura: o conteúdo só existe fora daqui durante a chamada.
    pub fn usar<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R { f(&self.dados) }
    pub fn usar_mut<R>(&mut self, f: impl FnOnce(&mut [u8]) -> R) -> R { f(&mut self.dados) }

    pub fn limpar(&mut self) { self.dados.zeroize_prefix(); }
}

trait ZerarTudo { fn zeroize_prefix(&mut self); }
impl ZerarTudo for Vec<u8> { fn zeroize_prefix(&mut self) { for b in self.iter_mut() { *b = 0; } core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst); } }

impl Drop for Segredo {
    fn drop(&mut self) {
        self.dados.zeroize_prefix();
        if self.travado { destravar(&self.dados); }
        self.dados.zeroize();
    }
}

impl core::fmt::Debug for Segredo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Segredo(<{} bytes ocultos>)", self.dados.len())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn debug_nunca_mostra_o_conteudo() {
        let s = Segredo::de(b"senha-do-cofre-123");
        let texto = format!("{s:?}");
        assert!(!texto.contains("senha"));
        assert!(texto.contains("18 bytes"));
    }

    #[test]
    fn limpar_zera_tudo() {
        let mut s = Segredo::de(b"conteudo sensivel");
        s.limpar();
        assert!(s.usar(|b| b.iter().all(|x| *x == 0)));
    }

    #[test]
    fn emprestimo_da_acesso_sem_copiar() {
        let s = Segredo::de(b"abc");
        assert_eq!(s.usar(|b| b.len()), 3);
        assert_eq!(s.tamanho(), 3);
    }

    #[test]
    fn travar_na_ram_nunca_derruba_o_app() {
        // Pode ser true ou false conforme o limite do sistema; o importante é não falhar.
        let s = Segredo::com_tamanho(4096);
        let _ = s.travado_na_ram();
        let vazio = Segredo::com_tamanho(0);
        assert!(!vazio.travado_na_ram());
    }
}
