//! O que a TUI precisa saber do sistema em que roda (ciclo 448).
//!
//! Tudo aqui recebe o ambiente e o sistema por PARÂMETRO, como
//! `agente::resolver_executavel`. Com `cfg!` no meio, o teste mediria a
//! máquina de quem roda — Linux — e o ramo do Windows nunca seria
//! exercido. Foi assim que a TUI nasceu gravando preferências em
//! `$HOME/.config`: no Windows `HOME` não existe, e tudo que a pessoa
//! configurava sumia ao fechar, com um aviso que não explicava nada.

/// Lê uma variável do ambiente real. Vazia conta como ausente: no
/// Windows `set X=` deixa a variável definida e sem valor.
pub fn ambiente(nome: &str) -> Option<String> {
    std::env::var_os(nome)
        .map(|v| v.to_string_lossy().to_string())
        .filter(|v| !v.trim().is_empty())
}

/// Onde mora `tui.json`.
///
/// No Windows é `%APPDATA%`, a pasta que o sistema dá a dados de
/// aplicativo. No Unix, a ordem de sempre (`XDG_CONFIG_HOME`, depois
/// `~/.config`): quem já tem preferências gravadas continua lendo do
/// mesmo lugar.
pub fn arquivo_de_preferencias(env: impl Fn(&str) -> Option<String>, windows: bool) -> Option<String> {
    let sep = if windows { '\\' } else { '/' };
    let base = if windows {
        env("APPDATA")?
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| format!("{}{sep}.config", sem_separador_final(&h))))?
    };
    Some(format!("{}{sep}anotadinho{sep}tui.json", sem_separador_final(&base)))
}

/// A pasta do usuário: `USERPROFILE` no Windows, `HOME` no resto.
///
/// No Windows, `HOME` entra como reserva: o Git Bash e o MSYS a definem,
/// e quem roda a TUI de dentro deles espera que ela valha.
pub fn pasta_do_usuario(env: impl Fn(&str) -> Option<String>, windows: bool) -> Option<String> {
    if windows {
        env("USERPROFILE").or_else(|| env("HOME"))
    } else {
        env("HOME")
    }
}

/// `~` no começo de um caminho digitado vira a pasta do usuário.
///
/// Só `~` sozinho ou seguido de separador: `~fulano` é a pasta de OUTRO
/// usuário no Unix, e virar `/home/eufulano` seria inventar caminho. No
/// Windows os dois separadores valem, porque o próprio sistema aceita
/// os dois.
pub fn expandir_til(caminho: &str, env: impl Fn(&str) -> Option<String>, windows: bool) -> String {
    let Some(resto) = caminho.strip_prefix('~') else {
        return caminho.to_string();
    };
    let separa = resto.is_empty() || resto.starts_with('/') || (windows && resto.starts_with('\\'));
    match pasta_do_usuario(env, windows) {
        Some(casa) if separa => format!("{}{resto}", sem_separador_final(&casa)),
        _ => caminho.to_string(),
    }
}

/// O caminho de um arquivo do vault no formato que o sistema entende.
///
/// O vault fala `/` (`assets/x.png`), e no Windows juntar isso a
/// `C:\vault` dá `C:\vault\assets/x.png` — que o Explorer trata como
/// argumento inválido e abre a pasta Documentos no lugar.
pub fn caminho_nativo(vault: &str, relativo: &str, windows: bool) -> String {
    if windows {
        let rel = relativo.replace('/', "\\");
        format!("{}\\{}", sem_separador_final(vault), rel.trim_start_matches('\\'))
    } else {
        format!("{}/{}", sem_separador_final(vault), relativo.trim_start_matches('/'))
    }
}

fn sem_separador_final(s: &str) -> &str {
    let t = s.trim_end_matches(['/', '\\']);
    // `/` sozinho é a raiz, não um separador sobrando.
    if t.is_empty() { s } else { t }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amb<'a>(pares: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |nome| pares.iter().find(|(n, _)| *n == nome).map(|(_, v)| v.to_string())
    }

    #[test]
    fn preferencias_no_windows_vao_pro_appdata_mesmo_sem_home() {
        let env = amb(&[("APPDATA", r"C:\Users\ana\AppData\Roaming"), ("USERPROFILE", r"C:\Users\ana")]);
        assert_eq!(
            arquivo_de_preferencias(env, true).as_deref(),
            Some(r"C:\Users\ana\AppData\Roaming\anotadinho\tui.json")
        );
    }

    #[test]
    fn preferencias_no_windows_ignoram_o_home_do_git_bash() {
        // Com HOME e APPDATA, quem manda é o APPDATA: senão a mesma
        // pessoa teria duas configurações, uma por terminal.
        let env = amb(&[("APPDATA", r"C:\R"), ("HOME", "/c/Users/ana")]);
        assert_eq!(arquivo_de_preferencias(env, true).as_deref(), Some(r"C:\R\anotadinho\tui.json"));
    }

    #[test]
    fn preferencias_no_unix_ficam_onde_sempre_estiveram() {
        assert_eq!(
            arquivo_de_preferencias(amb(&[("HOME", "/home/ana")]), false).as_deref(),
            Some("/home/ana/.config/anotadinho/tui.json")
        );
        assert_eq!(
            arquivo_de_preferencias(amb(&[("HOME", "/home/ana"), ("XDG_CONFIG_HOME", "/cfg/")]), false).as_deref(),
            Some("/cfg/anotadinho/tui.json")
        );
        assert_eq!(arquivo_de_preferencias(amb(&[]), false), None);
    }

    #[test]
    fn til_vira_a_pasta_do_usuario_em_cada_sistema() {
        let win = amb(&[("USERPROFILE", r"C:\Users\ana")]);
        assert_eq!(expandir_til(r"~\projetos", &win, true), r"C:\Users\ana\projetos");
        assert_eq!(expandir_til("~/projetos", &win, true), r"C:\Users\ana/projetos");
        assert_eq!(expandir_til("~", &win, true), r"C:\Users\ana");
        let unix = amb(&[("HOME", "/home/ana")]);
        assert_eq!(expandir_til("~/projetos", &unix, false), "/home/ana/projetos");
        // `\` não separa no Unix: é parte do nome.
        assert_eq!(expandir_til(r"~\x", &unix, false), r"~\x");
    }

    #[test]
    fn til_de_outro_usuario_e_caminho_sem_til_nao_mudam() {
        let unix = amb(&[("HOME", "/home/ana")]);
        assert_eq!(expandir_til("~bia/x", &unix, false), "~bia/x");
        assert_eq!(expandir_til("/tmp/x", &unix, false), "/tmp/x");
        assert_eq!(expandir_til("~/x", amb(&[]), false), "~/x");
    }

    #[test]
    fn caminho_pro_explorer_so_tem_contrabarra() {
        assert_eq!(caminho_nativo(r"C:\vault", "assets/x.png", true), r"C:\vault\assets\x.png");
        assert_eq!(caminho_nativo(r"C:\vault\", "/a/b.png", true), r"C:\vault\a\b.png");
        assert_eq!(caminho_nativo("/home/ana/vault", "assets/x.png", false), "/home/ana/vault/assets/x.png");
    }
}
