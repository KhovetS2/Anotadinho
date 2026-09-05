//! Cache em disco do índice do vault (ciclo 171).
//!
//! `scan_vault` (ciclo 150) lê o vault inteiro a cada chamada, e cada
//! embed que precisa dele chama por conta própria: o painel tem 3
//! consultas + 1 cronograma em modo vault = 4 varreduras completas só
//! pra desenhar uma página. Com 24 páginas isso é 7ms e ninguém nota;
//! com 2 mil, nota.
//!
//! Aqui o resultado de cada página fica guardado junto de uma marca de
//! versão (mtime + tamanho, a MESMA do ciclo 173). Na varredura
//! seguinte, só o que mudou é lido e reparseado.
//!
//! É CACHE, não fonte da verdade: arquivo corrompido, de versão antiga
//! ou ausente é simplesmente ignorado e reconstruído. Nada aqui pode
//! fazer o app falhar.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anotadinho_core::PageIndexEntry;
use serde::{Deserialize, Serialize};

/// Sobe quando o formato de `PageIndexEntry` mudar de um jeito que
/// invalide o que está gravado — cache de versão diferente é descartado
/// inteiro em vez de tentar migrar.
const VERSAO: u32 = 1;

/// Uma entrada do cache: a página indexada + a marca do arquivo.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entrada {
    /// `<mtime em nanos>-<tamanho>`, igual ao `page_version` do
    /// `VaultIo`.
    versao_arquivo: String,
    entry: PageIndexEntry,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Arquivo {
    versao: u32,
    paginas: HashMap<String, Entrada>,
}

/// Cache carregado na memória, pronto pra consultar e regravar.
#[derive(Debug, Default)]
pub struct IndexCache {
    caminho: PathBuf,
    paginas: HashMap<String, Entrada>,
    sujo: bool,
}

impl IndexCache {
    /// Carrega o cache do vault. Nunca falha: cache ausente, ilegível
    /// ou de outra versão vira cache vazio.
    pub fn carregar(raiz: &Path) -> Self {
        let caminho = raiz.join(".anotadinho").join("index.json");
        let paginas = std::fs::read_to_string(&caminho)
            .ok()
            .and_then(|txt| serde_json::from_str::<Arquivo>(&txt).ok())
            .filter(|a| a.versao == VERSAO)
            .map(|a| a.paginas)
            .unwrap_or_default();
        Self { caminho, paginas, sujo: false }
    }

    /// Entrada guardada pra esse path, se a marca do arquivo bater.
    pub fn obter(&self, path: &str, versao_arquivo: &str) -> Option<&PageIndexEntry> {
        self.paginas
            .get(path)
            .filter(|e| e.versao_arquivo == versao_arquivo)
            .map(|e| &e.entry)
    }

    /// Guarda (ou atualiza) a entrada de uma página.
    pub fn guardar(&mut self, path: &str, versao_arquivo: String, entry: PageIndexEntry) {
        self.paginas
            .insert(path.to_string(), Entrada { versao_arquivo, entry });
        self.sujo = true;
    }

    /// Descarta páginas que não existem mais — sem isso o cache só
    /// cresce, e uma página apagada voltaria a aparecer numa consulta
    /// se o cache fosse lido sem confronto.
    pub fn manter_apenas(&mut self, paths: &[String]) {
        let antes = self.paginas.len();
        // Conjunto, e não `paths.iter().any(...)` dentro do `retain`.
        //
        // A busca linear fazia disto um `O(n²)` no caminho MAIS quente
        // que existe: cada embed de consulta da página chama
        // `scan_vault` por conta própria, e toda varredura termina
        // aqui. Num vault de 4 mil páginas eram 16 milhões de
        // comparações de string por varredura, várias vezes por página
        // aberta.
        let atuais: std::collections::HashSet<&str> =
            paths.iter().map(String::as_str).collect();
        self.paginas.retain(|p, _| atuais.contains(p.as_str()));
        if self.paginas.len() != antes {
            self.sujo = true;
        }
    }

    /// Grava o cache se algo mudou. Falha de escrita é IGNORADA de
    /// propósito: vault somente-leitura ou sem permissão continua
    /// funcionando, só sem cache.
    pub fn salvar(&self) {
        if !self.sujo {
            return;
        }
        let Some(pasta) = self.caminho.parent() else { return };
        if std::fs::create_dir_all(pasta).is_err() {
            return;
        }
        let arquivo = Arquivo { versao: VERSAO, paginas: self.paginas.clone() };
        if let Ok(txt) = serde_json::to_string(&arquivo) {
            let _ = std::fs::write(&self.caminho, txt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entrada_exemplo(path: &str) -> PageIndexEntry {
        PageIndexEntry {
            path: path.to_string(),
            title: "T".into(),
            ..Default::default()
        }
    }

    #[test]
    fn guarda_e_devolve_quando_a_versao_bate() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut cache = IndexCache::carregar(dir.path());
        assert!(cache.obter("pages/a.md", "v1").is_none());

        cache.guardar("pages/a.md", "v1".into(), entrada_exemplo("pages/a.md"));
        assert!(cache.obter("pages/a.md", "v1").is_some());
        // Marca diferente = arquivo mudou; o cache não serve.
        assert!(cache.obter("pages/a.md", "v2").is_none());
    }

    #[test]
    fn sobrevive_a_ida_e_volta_pro_disco() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut cache = IndexCache::carregar(dir.path());
        cache.guardar("pages/a.md", "v1".into(), entrada_exemplo("pages/a.md"));
        cache.salvar();

        let relido = IndexCache::carregar(dir.path());
        assert_eq!(relido.obter("pages/a.md", "v1").map(|e| e.path.clone()), Some("pages/a.md".into()));
    }

    #[test]
    fn cache_corrompido_vira_cache_vazio() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".anotadinho")).unwrap();
        std::fs::write(dir.path().join(".anotadinho/index.json"), "{ isso não é json").unwrap();
        let cache = IndexCache::carregar(dir.path());
        assert!(cache.obter("pages/a.md", "v1").is_none(), "devia ignorar e seguir");
    }

    #[test]
    fn cache_de_outra_versao_e_descartado() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".anotadinho")).unwrap();
        std::fs::write(
            dir.path().join(".anotadinho/index.json"),
            r#"{"versao":999,"paginas":{}}"#,
        )
        .unwrap();
        let cache = IndexCache::carregar(dir.path());
        assert!(cache.obter("qualquer", "v").is_none());
    }

    #[test]
    fn manter_apenas_remove_pagina_apagada() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut cache = IndexCache::carregar(dir.path());
        cache.guardar("pages/a.md", "v1".into(), entrada_exemplo("pages/a.md"));
        cache.guardar("pages/b.md", "v1".into(), entrada_exemplo("pages/b.md"));
        cache.manter_apenas(&["pages/a.md".to_string()]);
        assert!(cache.obter("pages/a.md", "v1").is_some());
        assert!(cache.obter("pages/b.md", "v1").is_none());
    }
}
