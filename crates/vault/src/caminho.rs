//! A fronteira entre o caminho do sistema de arquivos e o caminho do vault.
//!
//! Dentro do vault, um caminho é sempre uma string relativa com `/`:
//! `pages/prompts-default/revisar-spec.md`. É essa forma que vai pro
//! frontend, pro `.md` (wikilinks, `contexto:`), pras chaves do cache de
//! índice e pra dez comparações espalhadas pelo código —
//! `path.starts_with("journals/")`, `rel.split('/')`,
//! `strip_prefix("pages/prompts-default/")`.
//!
//! Fora, o caminho é o que o sistema dá. No Linux e no macOS as duas
//! formas coincidem por acaso, e foi por isso que a diferença nunca
//! apareceu. No Windows, `strip_prefix` devolve `pages\journals\x.md` e
//! **nenhuma** daquelas comparações casa. Nenhuma delas dá erro, também:
//! a sidebar fica plana, os prompts padrão somem, os wikilinks não
//! resolvem — tudo em silêncio, que é o pior jeito de quebrar.
//!
//! Por isso a conversão mora aqui, num lugar só, e não em cada consumidor
//! (padrão da fronteira do sistema, ciclo 247): o caminho do sistema é
//! traduzido no ponto em que entra, e daí pra dentro só existe a forma do
//! vault.

use std::path::Path;

/// Caminho de `alvo` relativo a `raiz`, na forma do vault.
///
/// Devolve o caminho inteiro quando `alvo` está fora de `raiz` — é o
/// mesmo `unwrap_or(path)` que os chamadores já faziam, mantido para
/// nenhum deles precisar tratar erro por causa desta mudança.
pub fn relativo(raiz: &Path, alvo: &Path) -> String {
    recortar(&raiz.to_string_lossy(), &alvo.to_string_lossy(), cfg!(windows))
}

/// Troca os separadores do sistema pelos do vault.
///
/// Serve pra quem já tem o caminho relativo em mãos e só precisa da
/// forma canônica.
pub fn normalizar(bruto: &str) -> String {
    bruto.replace('\\', "/")
}

/// Forma comparável de um caminho absoluto.
///
/// Duas coisas acontecem aqui, e as duas são do Windows:
///
/// 1. Separadores viram `/`, pra `C:\Vault\pages` e `C:/Vault/pages`
///    serem o mesmo caminho — e são, o Win32 aceita os dois.
/// 2. O prefixo *verbatim* que `canonicalize` acrescenta some.
///    `std::fs::canonicalize` devolve `\\?\C:\Vault`, e o `notify`
///    entrega `C:\Vault\pages\x.md`. Sem tirar o prefixo, o
///    `strip_prefix` do watcher falha sempre e um caminho ABSOLUTO
///    escapa como se fosse relativo (item D2 do diagnóstico).
fn comparavel(bruto: &str) -> String {
    let s = bruto.replace('\\', "/");
    let s = if let Some(resto) = s.strip_prefix("//?/UNC/") {
        format!("//{resto}")
    } else if let Some(resto) = s.strip_prefix("//?/") {
        resto.to_string()
    } else {
        s
    };
    s.trim_end_matches('/').to_string()
}

/// O recorte propriamente dito, com a sensibilidade a caixa como
/// parâmetro em vez de `cfg!` embutido — é o que deixa os dois
/// comportamentos testáveis na máquina de quem desenvolve.
fn recortar(raiz: &str, alvo: &str, sem_caixa: bool) -> String {
    let r = comparavel(raiz);
    let a = comparavel(alvo);
    if r.is_empty() {
        return a;
    }
    let casa = |x: &str, y: &str| {
        if sem_caixa {
            x.eq_ignore_ascii_case(y)
        } else {
            x == y
        }
    };
    if casa(&a, &r) {
        return String::new();
    }
    if a.len() > r.len() && a.as_bytes()[r.len()] == b'/' && casa(&a[..r.len()], &r) {
        return a[r.len() + 1..].to_string();
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorta_caminho_posix() {
        assert_eq!(
            recortar("/home/e/Vault", "/home/e/Vault/pages/nota.md", false),
            "pages/nota.md"
        );
    }

    #[test]
    fn separador_do_windows_vira_barra() {
        assert_eq!(
            recortar(r"C:\Vault", r"C:\Vault\pages\prompts-default\revisar.md", true),
            "pages/prompts-default/revisar.md"
        );
    }

    #[test]
    fn prefixo_verbatim_do_canonicalize_nao_atrapalha() {
        // O caso D2: a raiz vem de `canonicalize` e o evento do `notify`
        // vem sem o prefixo. Antes, o `strip_prefix` falhava e o caminho
        // absoluto inteiro vazava como se fosse relativo ao vault.
        assert_eq!(
            recortar(r"\\?\C:\Vault", r"C:\Vault\journals\2026-09-05.md", true),
            "journals/2026-09-05.md"
        );
        assert_eq!(
            recortar(r"C:\Vault", r"\\?\C:\Vault\journals\2026-09-05.md", true),
            "journals/2026-09-05.md"
        );
    }

    #[test]
    fn compartilhamento_de_rede_tambem_recorta() {
        assert_eq!(
            recortar(r"\\?\UNC\servidor\notas", r"\\servidor\notas\pages\x.md", true),
            "pages/x.md"
        );
    }

    #[test]
    fn caixa_diferente_e_a_mesma_pasta_no_windows() {
        assert_eq!(
            recortar(r"C:\Vault", r"c:\vault\pages\x.md", true),
            "pages/x.md"
        );
    }

    #[test]
    fn caixa_diferente_e_outra_pasta_no_posix() {
        // No Linux `/home/Elis` e `/home/elis` são pastas diferentes, e
        // tratá-las como iguais recortaria o caminho errado.
        let fora = recortar("/home/Elis/Vault", "/home/elis/Vault/pages/x.md", false);
        assert_eq!(fora, "/home/elis/Vault/pages/x.md");
    }

    #[test]
    fn fora_da_raiz_devolve_o_caminho_inteiro() {
        assert_eq!(
            recortar("/home/e/Vault", "/etc/passwd", false),
            "/etc/passwd"
        );
    }

    #[test]
    fn prefixo_parcial_nao_conta_como_dentro() {
        // `/home/e/Vault2` começa com `/home/e/Vault`, mas é outro vault.
        assert_eq!(
            recortar("/home/e/Vault", "/home/e/Vault2/pages/x.md", false),
            "/home/e/Vault2/pages/x.md"
        );
    }

    #[test]
    fn barra_final_na_raiz_nao_desloca_o_recorte() {
        assert_eq!(
            recortar("/home/e/Vault/", "/home/e/Vault/pages/x.md", false),
            "pages/x.md"
        );
    }

    #[test]
    fn a_propria_raiz_vira_caminho_vazio() {
        assert_eq!(recortar("/home/e/Vault", "/home/e/Vault", false), "");
    }

    #[test]
    fn normalizar_troca_so_o_separador() {
        assert_eq!(normalizar(r"pages\sub\x.md"), "pages/sub/x.md");
        assert_eq!(normalizar("pages/sub/x.md"), "pages/sub/x.md");
    }
}
