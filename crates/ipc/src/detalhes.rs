//! IO dos detalhes da tarefa (ciclo 476): tags, prazo, checklist,
//! comentários e anexos.
//!
//! Pela PESSOA, grava direto, como `mudar_etapa_tarefa`: a janela e a TUI
//! chamam. A regra de onde cada coisa mora no texto é do
//! `core::detalhes` e do `core::projeto`; aqui só se lê, confere que é
//! tarefa e grava.

use anotadinho_core::detalhes;
use anotadinho_core::projeto::{self, Tarefa};
use anotadinho_vault::VaultIo;

use crate::{handle_read_page, handle_write_page};

/// Lê a página e confirma que é tarefa ANTES de qualquer escrita: as
/// funções de texto mexeriam em qualquer `.md`.
fn ler_tarefa(vault_path: &str, caminho: &str) -> Result<String, String> {
    let atual = handle_read_page(vault_path.to_string(), caminho.to_string())?;
    Tarefa::ler(caminho, &atual).ok_or_else(|| format!("{caminho} não é uma tarefa de projeto"))?;
    Ok(atual)
}

/// Troca as tags da tarefa.
pub fn handle_tags_da_tarefa(vault_path: String, caminho: String, tags: Vec<String>) -> Result<(), String> {
    let atual = ler_tarefa(&vault_path, &caminho)?;
    handle_write_page(vault_path, caminho, projeto::com_tags(&atual, &tags)?)
}

/// Põe (`Some`) ou tira (`None`) o prazo.
pub fn handle_prazo_da_tarefa(vault_path: String, caminho: String, prazo: Option<String>) -> Result<(), String> {
    let atual = ler_tarefa(&vault_path, &caminho)?;
    handle_write_page(vault_path, caminho, projeto::com_prazo(&atual, prazo.as_deref())?)
}

/// Marca ou desmarca o item `indice` da checklist.
pub fn handle_alternar_item_da_tarefa(vault_path: String, caminho: String, indice: usize) -> Result<(), String> {
    let atual = ler_tarefa(&vault_path, &caminho)?;
    handle_write_page(vault_path, caminho, detalhes::alternar_item(&atual, indice)?)
}

/// Acrescenta um comentário. `quando` vem de quem chama (o relógio é
/// do frontend, como em todo o resto).
pub fn handle_comentar_tarefa(vault_path: String, caminho: String, quando: String, texto: String) -> Result<(), String> {
    let atual = ler_tarefa(&vault_path, &caminho)?;
    handle_write_page(vault_path, caminho, detalhes::acrescentar_comentario(&atual, &quando, &texto)?)
}

/// Copia `origem` pro vault (sem sobrescrever nada) e lista na tarefa.
/// Devolve o caminho no vault.
pub fn handle_anexar_na_tarefa(vault_path: String, caminho: String, origem: String) -> Result<String, String> {
    let atual = ler_tarefa(&vault_path, &caminho)?;
    let origem = origem.trim().trim_matches('"').trim_matches('\'').to_string();
    let no_vault = VaultIo::open(&vault_path).copiar_para_assets_sem_sobrescrever(&origem).map_err(|e| e.to_string())?;
    let nome = std::path::Path::new(&origem)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| no_vault.clone());
    match detalhes::acrescentar_anexo(&atual, &nome, &no_vault) {
        Ok(novo) => {
            handle_write_page(vault_path, caminho, novo)?;
            Ok(no_vault)
        }
        Err(e) => {
            // A cópia sem link seria lixo em `assets/` que ninguém acha.
            let _ = std::fs::remove_file(std::path::Path::new(&vault_path).join(&no_vault));
            Err(e)
        }
    }
}

/// O programa que abre um arquivo com o app padrão do sistema. O sistema
/// entra por PARÂMETRO: com `cfg!` aqui dentro, o teste só exerceria o
/// ramo da máquina de quem roda.
pub fn programa_que_abre(macos: bool, windows: bool) -> &'static str {
    if macos {
        "open"
    } else if windows {
        "explorer"
    } else {
        "xdg-open"
    }
}

/// O caminho absoluto de um anexo, SE ele é um arquivo dentro de
/// `assets/` do vault.
///
/// A trava existe porque o caminho vem do texto de uma página: sem ela,
/// um link `../../usr/bin/x` escrito à mão (ou colado) faria o app mandar
/// o sistema abrir qualquer coisa. `canonicalize` resolve `..` e links
/// simbólicos antes da comparação.
pub fn anexo_no_vault(vault_path: &str, caminho: &str) -> Result<std::path::PathBuf, String> {
    let raiz = std::path::Path::new(vault_path);
    let assets = raiz.join("assets").canonicalize().map_err(|_| "o vault não tem a pasta assets/".to_string())?;
    let alvo = raiz
        .join(caminho.trim().trim_start_matches("./"))
        .canonicalize()
        .map_err(|_| format!("não achei {caminho}"))?;
    if !alvo.starts_with(&assets) || !alvo.is_file() {
        return Err(format!("{caminho} não é um arquivo de assets/ do vault"));
    }
    Ok(alvo)
}

/// Abre um anexo com o app padrão do sistema.
pub fn handle_abrir_anexo(vault_path: String, caminho: String) -> Result<(), String> {
    let alvo = anexo_no_vault(&vault_path, &caminho)?;
    std::process::Command::new(programa_que_abre(cfg!(target_os = "macos"), cfg!(windows)))
        .arg(&alvo)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("não abriu {caminho}: {e}"))
}

#[cfg(test)]
mod testes {
    use super::*;

    fn vault_com_tarefa() -> (tempfile::TempDir, String, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let v = dir.path().to_string_lossy().to_string();
        crate::handle_criar_projeto(v.clone(), "Saas".into(), Some("saas".into()), "2026-10-22".into()).unwrap();
        let c = crate::handle_criar_tarefa(v.clone(), "saas".into(), "Login".into(), "2026-10-22".into()).unwrap();
        (dir, v, c)
    }

    #[test]
    fn os_detalhes_vao_pra_pagina_e_voltam_na_tarefa() {
        let (_d, v, c) = vault_com_tarefa();
        handle_tags_da_tarefa(v.clone(), c.clone(), vec!["#auth".into(), "front".into(), "auth".into()]).unwrap();
        handle_prazo_da_tarefa(v.clone(), c.clone(), Some("2026-11-05".into())).unwrap();
        assert!(handle_prazo_da_tarefa(v.clone(), c.clone(), Some("5/11".into())).is_err());
        handle_comentar_tarefa(v.clone(), c.clone(), "2026-10-22 14:30".into(), "olhei".into()).unwrap();
        let fora = tempfile::TempDir::new().unwrap();
        let arq = fora.path().join("tela.png");
        std::fs::write(&arq, b"png").unwrap();
        let no_vault = handle_anexar_na_tarefa(v.clone(), c.clone(), arq.to_string_lossy().to_string()).unwrap();
        assert_eq!(no_vault, "assets/tela.png");

        let texto = handle_read_page(v.clone(), c.clone()).unwrap();
        let t = Tarefa::ler(&c, &texto).unwrap();
        assert_eq!(t.tags, ["auth", "front"]);
        assert_eq!(t.prazo.as_deref(), Some("2026-11-05"));
        assert_eq!(t.comentarios.len(), 1);
        assert_eq!(t.anexos[0].caminho, "assets/tela.png");
        assert!(t.vencida("2026-11-06") && !t.vencida("2026-11-05"));
        // O esqueleto da tarefa nova tem um item VAZIO nos critérios: ele
        // não é critério ainda, e a checklist começa sem nada.
        assert_eq!(t.progresso(), (0, 0));
        // E a seção dos critérios continua lá, intacta.
        assert!(texto.contains("## Critérios de aceite\n\n- [ ] \n"), "{texto}");
    }

    #[test]
    fn marcar_a_checklist_pela_pessoa() {
        let (_d, v, c) = vault_com_tarefa();
        let texto = handle_read_page(v.clone(), c.clone()).unwrap();
        let com_itens = texto.replace("## Critérios de aceite\n\n- [ ] \n", "## Critérios de aceite\n\n- [ ] entra\n- [ ] sai\n");
        handle_write_page(v.clone(), c.clone(), com_itens).unwrap();
        handle_alternar_item_da_tarefa(v.clone(), c.clone(), 1).unwrap();
        let t = Tarefa::ler(&c, &handle_read_page(v.clone(), c.clone()).unwrap()).unwrap();
        assert_eq!(t.progresso(), (1, 2));
        assert!(t.checklist[1].feito && !t.checklist[0].feito);
    }

    #[test]
    fn so_abre_o_que_esta_em_assets() {
        let (d, v, _c) = vault_com_tarefa();
        std::fs::create_dir_all(d.path().join("assets")).unwrap();
        std::fs::write(d.path().join("assets/a.pdf"), b"x").unwrap();
        std::fs::write(d.path().join("pages/fora.md"), b"x").unwrap();
        assert!(anexo_no_vault(&v, "assets/a.pdf").is_ok());
        assert!(anexo_no_vault(&v, "./assets/a.pdf").is_ok());
        assert!(anexo_no_vault(&v, "assets/../pages/fora.md").is_err(), "o `..` escapa de assets/");
        assert!(anexo_no_vault(&v, "pages/fora.md").is_err());
        assert!(anexo_no_vault(&v, "/etc/hostname").is_err());
        assert!(anexo_no_vault(&v, "assets").is_err(), "pasta não é anexo");
        assert!(anexo_no_vault(&v, "assets/nao-existe.pdf").is_err());
    }

    #[test]
    fn o_programa_de_cada_sistema() {
        assert_eq!(programa_que_abre(true, false), "open");
        assert_eq!(programa_que_abre(false, true), "explorer");
        assert_eq!(programa_que_abre(false, false), "xdg-open");
    }

    #[test]
    fn pagina_que_nao_e_tarefa_nao_e_tocada() {
        let (_d, v, _c) = vault_com_tarefa();
        handle_write_page(v.clone(), "pages/nota.md".into(), "## Critérios de aceite\n\n- [ ] x\n".into()).unwrap();
        assert!(handle_alternar_item_da_tarefa(v.clone(), "pages/nota.md".into(), 0).is_err());
        assert!(handle_comentar_tarefa(v.clone(), "pages/nota.md".into(), "a".into(), "b".into()).is_err());
        assert_eq!(handle_read_page(v, "pages/nota.md".into()).unwrap(), "## Critérios de aceite\n\n- [ ] x\n");
    }
}
