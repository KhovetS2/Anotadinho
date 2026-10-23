//! Quanto da memória do processo é do Rust (ciclo 483).
//!
//! O `smaps` do app junta tudo no mesmo heap: o glibc `malloc` atende o
//! GTK, o lado de fora do WebKit, o driver de vídeo e o nosso código. Pela
//! leitura de fora não dá pra separar, e o palpite sem medida já apontou
//! o culpado errado uma vez (ciclo 482, a página inicial que era o grafo).
//!
//! O alocador global do Rust só é chamado pelo código Rust, então contar
//! ali dá exatamente a nossa parte. Custa dois `fetch_add` por alocação,
//! sem trava.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// O `System` com contagem dos bytes vivos e do pico.
pub struct AlocadorContado {
    vivos: AtomicUsize,
    pico: AtomicUsize,
}

/// A foto do contador, em bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct MemoriaDoRust {
    pub vivos: usize,
    pub pico: usize,
}

impl AlocadorContado {
    pub const fn novo() -> Self {
        Self { vivos: AtomicUsize::new(0), pico: AtomicUsize::new(0) }
    }

    pub fn foto(&self) -> MemoriaDoRust {
        MemoriaDoRust {
            vivos: self.vivos.load(Ordering::Relaxed),
            pico: self.pico.load(Ordering::Relaxed),
        }
    }

    fn somar(&self, n: usize) {
        let agora = self.vivos.fetch_add(n, Ordering::Relaxed) + n;
        self.pico.fetch_max(agora, Ordering::Relaxed);
    }

    fn tirar(&self, n: usize) {
        self.vivos.fetch_sub(n, Ordering::Relaxed);
    }
}

impl Default for AlocadorContado {
    fn default() -> Self {
        Self::novo()
    }
}

// SAFETY: repassa tudo ao `System` sem mexer em ponteiro nem layout; a
// contagem só acontece quando o `System` devolveu sucesso.
unsafe impl GlobalAlloc for AlocadorContado {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() {
            self.somar(layout.size());
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc_zeroed(layout);
        if !p.is_null() {
            self.somar(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.tirar(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, novo: usize) -> *mut u8 {
        let p = System.realloc(ptr, layout, novo);
        // Falhou: o bloco antigo continua vivo e do mesmo tamanho.
        if !p.is_null() {
            if novo >= layout.size() {
                self.somar(novo - layout.size());
            } else {
                self.tirar(layout.size() - novo);
            }
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conta_vivos_e_guarda_o_pico() {
        let a = AlocadorContado::novo();
        let l = Layout::from_size_align(1000, 8).unwrap();
        unsafe {
            let p = a.alloc(l);
            let q = a.alloc_zeroed(l);
            assert_eq!(a.foto(), MemoriaDoRust { vivos: 2000, pico: 2000 });
            let p = a.realloc(p, l, 3000);
            assert_eq!(a.foto(), MemoriaDoRust { vivos: 4000, pico: 4000 });
            let p = a.realloc(p, Layout::from_size_align(3000, 8).unwrap(), 500);
            assert_eq!(a.foto().vivos, 1500);
            a.dealloc(q, l);
            a.dealloc(p, Layout::from_size_align(500, 8).unwrap());
        }
        assert_eq!(a.foto(), MemoriaDoRust { vivos: 0, pico: 4000 });
    }
}
