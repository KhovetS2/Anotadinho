//! O repositório do projeto, visto pelo app (ciclo 517): verificar, saber
//! o que a tarefa mudou, o branch da tarefa e o commit ao aceitar.
//!
//! Até aqui o app só sabia o que o AGENTE dizia: "RESULTADO: concluida"
//! levava a tarefa pra revisão, e a revisão era ler o relato. Num SaaS, o
//! que importa é o código: os testes passaram? o que mudou? Este módulo é
//! a resposta, e é o mesmo pra janela, a TUI e o CLI.
//!
//! O sistema entra por PARÂMETRO ([`Shell`]): o comando de verificação
//! roda em `sh -c` no Unix e em `cmd /C` no Windows, e quem escolhe é a
//! borda (`Shell::do_sistema`) — um `cfg!` aqui no meio faria o teste medir
//! só a máquina de quem roda (AGENTS.md).

use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Quanto da saída da verificação fica: o fim, que é onde o teste diz o
/// que quebrou. Uma suíte inteira no relato da tarefa afogaria o resto.
pub const LINHAS_DA_SAIDA: usize = 60;

/// Quem interpreta o comando de verificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// `sh -c "<comando>"`.
    Sh,
    /// `cmd /C "<comando>"`.
    Cmd,
}

impl Shell {
    /// O da máquina que roda: a ÚNICA leitura do sistema neste módulo.
    pub fn do_sistema() -> Self {
        if cfg!(windows) {
            Shell::Cmd
        } else {
            Shell::Sh
        }
    }

    fn programa_e_args(self, comando: &str) -> (&'static str, Vec<String>) {
        match self {
            Shell::Sh => ("sh", vec!["-c".into(), comando.into()]),
            Shell::Cmd => ("cmd", vec!["/C".into(), comando.into()]),
        }
    }
}

/// Como a verificação terminou.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verificacao {
    /// O comando que rodou.
    pub comando: String,
    /// Saiu com 0 dentro do prazo.
    pub passou: bool,
    /// O código de saída; `None` quando estourou o prazo ou não subiu.
    pub codigo: Option<i32>,
    /// Quanto levou.
    pub segundos: u64,
    /// As últimas [`LINHAS_DA_SAIDA`] linhas de stdout + stderr.
    pub saida: String,
    /// Estourou o prazo e foi morto.
    pub estourou: bool,
}

impl Verificacao {
    /// O relato que vai pra conversa da tarefa.
    pub fn relato(&self) -> String {
        let como = if self.estourou {
            format!("estourou o prazo depois de {} s e foi interrompida", self.segundos)
        } else if self.passou {
            format!("passou em {} s", self.segundos)
        } else {
            match self.codigo {
                Some(c) => format!("FALHOU (código {c}) em {} s", self.segundos),
                None => "não conseguiu rodar".to_string(),
            }
        };
        let mut t = format!("**Verificação do projeto** — `{}` {como}.", self.comando);
        if !self.saida.trim().is_empty() {
            t.push_str("\n\n```\n");
            t.push_str(self.saida.trim_end());
            t.push_str("\n```");
        }
        t
    }
}

fn ultimas_linhas(texto: &str, n: usize) -> String {
    let linhas: Vec<&str> = texto.lines().collect();
    let ini = linhas.len().saturating_sub(n);
    let mut fora = String::new();
    if ini > 0 {
        fora.push_str(&format!("… ({ini} linha(s) antes)\n"));
    }
    fora.push_str(&linhas[ini..].join("\n"));
    fora
}

/// Roda `comando` em `cwd`, com prazo. stdout e stderr juntos, na ordem em
/// que cada um terminou de chegar (lidos em threads, senão um pipe cheio
/// trava o filho).
pub fn verificar(cwd: &str, comando: &str, prazo: Duration, shell: Shell) -> Verificacao {
    let (programa, args) = shell.programa_e_args(comando);
    let inicio = Instant::now();
    let filho = anotadinho_vault::comando(programa)
        .args(&args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut filho = match filho {
        Ok(f) => f,
        Err(e) => {
            return Verificacao {
                comando: comando.into(),
                passou: false,
                codigo: None,
                segundos: 0,
                saida: format!("não subiu `{programa}`: {e}"),
                estourou: false,
            }
        }
    };
    let ler = |r: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut s = String::new();
            if let Some(mut r) = r {
                let mut b = Vec::new();
                let _ = r.read_to_end(&mut b);
                s = String::from_utf8_lossy(&b).into_owned();
            }
            s
        })
    };
    let out = ler(filho.stdout.take().map(|o| Box::new(o) as Box<dyn Read + Send>));
    let err = ler(filho.stderr.take().map(|o| Box::new(o) as Box<dyn Read + Send>));
    let mut estourou = false;
    let status = loop {
        match filho.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) if inicio.elapsed() >= prazo => {
                let _ = filho.kill();
                let _ = filho.wait();
                estourou = true;
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => break None,
        }
    };
    let mut saida = out.join().unwrap_or_default();
    let e = err.join().unwrap_or_default();
    if !e.trim().is_empty() {
        if !saida.is_empty() && !saida.ends_with('\n') {
            saida.push('\n');
        }
        saida.push_str(&e);
    }
    let codigo = status.and_then(|s| s.code());
    Verificacao {
        comando: comando.into(),
        passou: !estourou && codigo == Some(0),
        codigo,
        segundos: inicio.elapsed().as_secs(),
        saida: ultimas_linhas(&saida, LINHAS_DA_SAIDA),
        estourou,
    }
}

// ── git ──────────────────────────────────────────────────────────────

fn git(repo: &str, args: &[&str]) -> Result<String, String> {
    let saida = anotadinho_vault::comando("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| format!("git não rodou: {e}"))?;
    if saida.status.success() {
        Ok(String::from_utf8_lossy(&saida.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&saida.stderr).trim().to_string())
    }
}

/// O repositório é git?
pub fn e_git(repo: &str) -> bool {
    Path::new(repo).is_dir() && git(repo, &["rev-parse", "--is-inside-work-tree"]).is_ok_and(|s| s.trim() == "true")
}

/// O commit de agora (`HEAD`); `None` fora de git ou sem commit nenhum.
pub fn base_atual(repo: &str) -> Option<String> {
    git(repo, &["rev-parse", "HEAD"]).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// O branch de agora; `None` com HEAD solto ou fora de git.
pub fn branch_atual(repo: &str) -> Option<String> {
    git(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty() && s != "HEAD")
}

/// O nome do branch de uma tarefa: `tarefa/003-listar-os-livros`.
pub fn nome_do_branch(numero: u32, titulo: &str) -> String {
    let slug: String = anotadinho_core::fluxo::slug_de_titulo(titulo);
    let slug: String = slug.chars().take(40).collect();
    format!("tarefa/{numero:03}-{}", slug.trim_matches('-'))
}

/// Põe o repositório no branch da tarefa: cria a partir de onde está, ou
/// troca pra ele se já existe (a segunda rodada de uma tarefa volta pro
/// mesmo branch). Mudança não commitada vai junto, como o `git switch` faz.
pub fn ir_pro_branch(repo: &str, nome: &str) -> Result<(), String> {
    if branch_atual(repo).as_deref() == Some(nome) {
        return Ok(());
    }
    let existe = git(repo, &["rev-parse", "--verify", "--quiet", &format!("refs/heads/{nome}")]).is_ok();
    if existe {
        git(repo, &["switch", nome]).map(|_| ())
    } else {
        git(repo, &["switch", "-c", nome]).map(|_| ())
    }
}

/// Um arquivo mudado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArquivoMudado {
    /// `M`, `A`, `D`, `R` ou `?` (novo, fora do git).
    pub estado: String,
    /// Relativo à raiz do repositório.
    pub caminho: String,
}

/// O que a tarefa mudou no repositório desde `base`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mudancas {
    /// O commit de onde a comparação parte.
    pub base: Option<String>,
    /// O branch de agora.
    pub branch: Option<String>,
    /// Os arquivos mudados, com o estado.
    pub arquivos: Vec<ArquivoMudado>,
    /// O diff unificado, com os arquivos novos inteiros.
    pub diff: String,
    /// Quando não deu pra saber (fora de git, base sumida).
    pub aviso: Option<String>,
}

/// Teto do diff: um `package-lock.json` novo não pode travar a tela.
pub const TETO_DO_DIFF: usize = 400_000;
/// Arquivo novo maior que isto entra só pelo nome.
const TETO_DO_ARQUIVO_NOVO: u64 = 64_000;

/// As mudanças desde `base` (o commit em que a tarefa começou): o que o
/// agente commitou E o que deixou na árvore, mais os arquivos novos que o
/// git ainda não conhece. Sem base, contra o `HEAD`.
pub fn mudancas_desde(repo: &str, base: Option<&str>) -> Mudancas {
    let mut m = Mudancas { base: base.map(str::to_string), branch: branch_atual(repo), ..Default::default() };
    if !e_git(repo) {
        m.aviso = Some(format!("{repo} não é um repositório git: sem como saber o que mudou"));
        return m;
    }
    let contra = match base {
        Some(b) if git(repo, &["cat-file", "-e", &format!("{b}^{{commit}}")]).is_ok() => b.to_string(),
        Some(b) => {
            m.aviso = Some(format!("o commit de onde a tarefa partiu ({}) não existe mais; mostrando contra o HEAD", &b[..b.len().min(10)]));
            "HEAD".to_string()
        }
        None => "HEAD".to_string(),
    };
    // Repositório sem commit nenhum: não há contra o que comparar; tudo é
    // arquivo novo.
    let tem_head = base_atual(repo).is_some();
    if tem_head {
        if let Ok(nomes) = git(repo, &["diff", "--name-status", &contra]) {
            for l in nomes.lines() {
                let mut p = l.split('\t');
                let estado = p.next().unwrap_or("").chars().next().map(|c| c.to_string()).unwrap_or_default();
                if let Some(caminho) = p.last() {
                    m.arquivos.push(ArquivoMudado { estado, caminho: caminho.to_string() });
                }
            }
        }
        m.diff = git(repo, &["diff", "--no-color", &contra]).unwrap_or_default();
    }
    if let Ok(novos) = git(repo, &["ls-files", "--others", "--exclude-standard"]) {
        for caminho in novos.lines().filter(|l| !l.trim().is_empty()) {
            m.arquivos.push(ArquivoMudado { estado: "?".into(), caminho: caminho.to_string() });
            let cheio = Path::new(repo).join(caminho);
            let tamanho = std::fs::metadata(&cheio).map(|x| x.len()).unwrap_or(0);
            m.diff.push_str(&format!("diff --git a/{caminho} b/{caminho}\nnew file\n--- /dev/null\n+++ b/{caminho}\n"));
            match std::fs::read_to_string(&cheio) {
                Ok(t) if tamanho <= TETO_DO_ARQUIVO_NOVO => {
                    let n = t.lines().count();
                    m.diff.push_str(&format!("@@ -0,0 +1,{n} @@\n"));
                    for l in t.lines() {
                        m.diff.push('+');
                        m.diff.push_str(l);
                        m.diff.push('\n');
                    }
                }
                Ok(_) => m.diff.push_str(&format!("(arquivo novo de {tamanho} bytes: grande demais pra mostrar)\n")),
                Err(_) => m.diff.push_str("(arquivo binário)\n"),
            }
        }
    }
    if m.diff.len() > TETO_DO_DIFF {
        let mut corte = TETO_DO_DIFF;
        while !m.diff.is_char_boundary(corte) {
            corte -= 1;
        }
        m.diff.truncate(corte);
        m.diff.push_str("\n… (diff cortado: grande demais pra mostrar inteiro)\n");
    }
    m
}

/// `git add -A && git commit -m <mensagem>`. "Nada pra commitar" não é
/// erro: devolve `Ok(None)`. Devolve o commit novo.
pub fn commitar(repo: &str, mensagem: &str) -> Result<Option<String>, String> {
    if !e_git(repo) {
        return Err(format!("{repo} não é um repositório git"));
    }
    // O índice de antes, pra voltar a ele se o commit falhar (ciclo 520):
    // sem isto, um hook que recusa deixava TUDO no `add -A` — a tarefa
    // ficava em revisão, mas o índice da pessoa tinha mudado por baixo.
    let indice_antes = git(repo, &["write-tree"]).ok().map(|t| t.trim().to_string());
    git(repo, &["add", "-A"])?;
    let nada = git(repo, &["diff", "--cached", "--quiet"]).is_ok();
    if nada {
        return Ok(None);
    }
    if let Err(e) = git(repo, &["commit", "-m", mensagem]) {
        if let Some(t) = &indice_antes {
            let _ = git(repo, &["read-tree", t]);
        }
        return Err(format!("o commit falhou: {e}"));
    }
    Ok(base_atual(repo))
}

#[cfg(test)]
mod testes {
    use super::*;

    fn repo() -> (tempfile::TempDir, String) {
        let d = tempfile::TempDir::new().unwrap();
        let r = d.path().to_string_lossy().to_string();
        for args in [&["init", "-q", "-b", "main"][..], &["config", "user.email", "t@t"], &["config", "user.name", "t"]] {
            git(&r, args).unwrap();
        }
        (d, r)
    }

    #[test]
    fn verificar_passa_falha_e_estoura_o_prazo() {
        let d = tempfile::TempDir::new().unwrap();
        let cwd = d.path().to_string_lossy().to_string();
        let ok = verificar(&cwd, "echo tudo certo", Duration::from_secs(10), Shell::Sh);
        assert!(ok.passou && ok.codigo == Some(0) && ok.saida.contains("tudo certo"), "{ok:?}");
        let falha = verificar(&cwd, "echo quebrou >&2; exit 3", Duration::from_secs(10), Shell::Sh);
        assert!(!falha.passou && falha.codigo == Some(3) && falha.saida.contains("quebrou"), "{falha:?}");
        assert!(falha.relato().contains("FALHOU (código 3)"), "{}", falha.relato());
        let lento = verificar(&cwd, "sleep 5", Duration::from_millis(300), Shell::Sh);
        assert!(lento.estourou && !lento.passou, "{lento:?}");
        // Roda NO repositório.
        std::fs::write(d.path().join("marca.txt"), "x").unwrap();
        assert!(verificar(&cwd, "test -f marca.txt", Duration::from_secs(5), Shell::Sh).passou);
        // Saída longa: só o fim.
        let muito = verificar(&cwd, "seq 1 500", Duration::from_secs(10), Shell::Sh);
        assert!(muito.saida.contains("500") && !muito.saida.contains("\n1\n") && muito.saida.starts_with("… (440"), "{}", &muito.saida[..60]);
    }

    #[test]
    fn mudancas_desde_a_base_juntam_commit_arvore_e_arquivo_novo() {
        let (_d, r) = repo();
        std::fs::write(Path::new(&r).join("a.txt"), "um\n").unwrap();
        assert_eq!(commitar(&r, "início").unwrap().is_some(), true);
        let base = base_atual(&r).unwrap();
        // O agente commita uma mudança, deixa outra na árvore e cria um arquivo.
        std::fs::write(Path::new(&r).join("a.txt"), "um\ndois\n").unwrap();
        git(&r, &["commit", "-qam", "agente"]).unwrap();
        std::fs::write(Path::new(&r).join("a.txt"), "um\ndois\ntrês\n").unwrap();
        std::fs::write(Path::new(&r).join("novo.rs"), "fn main() {}\n").unwrap();
        let m = mudancas_desde(&r, Some(&base));
        assert!(m.aviso.is_none(), "{:?}", m.aviso);
        assert_eq!(m.arquivos, [ArquivoMudado { estado: "M".into(), caminho: "a.txt".into() }, ArquivoMudado { estado: "?".into(), caminho: "novo.rs".into() }]);
        assert!(m.diff.contains("+dois") && m.diff.contains("+três"), "{}", m.diff);
        assert!(m.diff.contains("+++ b/novo.rs") && m.diff.contains("+fn main() {}"), "{}", m.diff);
        assert_eq!(m.branch.as_deref(), Some("main"));
    }

    #[test]
    fn fora_de_git_avisa_e_commit_sem_nada_nao_e_erro() {
        let d = tempfile::TempDir::new().unwrap();
        let fora = d.path().to_string_lossy().to_string();
        assert!(mudancas_desde(&fora, None).aviso.is_some());
        assert!(commitar(&fora, "x").is_err());
        let (_d, r) = repo();
        std::fs::write(Path::new(&r).join("a.txt"), "um\n").unwrap();
        commitar(&r, "início").unwrap();
        assert_eq!(commitar(&r, "de novo").unwrap(), None, "nada pra commitar");
        // Base que não existe mais: compara com o HEAD e diz por quê.
        let m = mudancas_desde(&r, Some("0123456789abcdef0123456789abcdef01234567"));
        assert!(m.aviso.as_deref().is_some_and(|a| a.contains("não existe mais")), "{:?}", m.aviso);
    }

    #[test]
    fn commit_recusado_devolve_o_indice_como_estava() {
        let (_d, r) = repo();
        std::fs::write(Path::new(&r).join("a.txt"), "um\n").unwrap();
        commitar(&r, "início").unwrap();
        // A pessoa deixou um arquivo no índice e outro só na árvore.
        std::fs::write(Path::new(&r).join("preparado.txt"), "p\n").unwrap();
        git(&r, &["add", "preparado.txt"]).unwrap();
        std::fs::write(Path::new(&r).join("a.txt"), "dois\n").unwrap();
        // Um hook que recusa todo commit.
        let hook = Path::new(&r).join(".git/hooks/pre-commit");
        std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(commitar(&r, "tarefa").is_err());
        let no_indice = git(&r, &["diff", "--cached", "--name-only"]).unwrap();
        assert_eq!(no_indice.trim(), "preparado.txt", "o índice da pessoa mudou: {no_indice}");
    }

    #[test]
    fn branch_da_tarefa_cria_e_volta() {
        let (_d, r) = repo();
        std::fs::write(Path::new(&r).join("a.txt"), "um\n").unwrap();
        commitar(&r, "início").unwrap();
        let nome = nome_do_branch(3, "Listar os livros!");
        assert_eq!(nome, "tarefa/003-listar-os-livros");
        ir_pro_branch(&r, &nome).unwrap();
        assert_eq!(branch_atual(&r).as_deref(), Some(nome.as_str()));
        git(&r, &["switch", "-q", "main"]).unwrap();
        ir_pro_branch(&r, &nome).unwrap();
        assert_eq!(branch_atual(&r).as_deref(), Some(nome.as_str()), "a segunda rodada volta pro mesmo");
    }
}
