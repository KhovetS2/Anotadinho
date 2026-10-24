//! A cor do código por linguagem (ciclo 495, RF2 da spec
//! `tui-codigo-e-diagramas`).
//!
//! O `syntect` quebra o código em trechos com escopo do Sublime
//! (`keyword.control.rust`, `string.quoted.double`…). O escopo vira uma
//! [`Categoria`] nossa, e a categoria vira cor pelo tema da TUI — os temas
//! do syntect nem são compilados. Assim o código combina com o resto da
//! tela, nos cinco temas.
//!
//! As gramáticas pesam: carregam uma vez, na primeira vez que um bloco com
//! linguagem é desenhado, e não na partida (o mesmo princípio do ciclo 486
//! na janela). E o resultado de cada bloco fica guardado: a tela redesenha
//! a cada tecla, e repassar o mesmo código pelo parser a cada quadro seria
//! o custo inteiro de novo.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;

use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};

/// O papel de um trecho de código. Cinco cores e o texto comum: mais que
/// isso, e o terminal vira arco-íris.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Categoria {
    Texto,
    Palavra,
    Cadeia,
    Comentario,
    Numero,
    Tipo,
}

/// Um trecho de uma linha com a categoria dele.
pub type Trecho = (Categoria, String);

/// Linguagens que a cerca escreve de um jeito e as gramáticas padrão do
/// syntect conhecem por outro (ou não conhecem: TypeScript e TOML não vêm
/// no pacote padrão).
fn apelido(lingua: &str) -> &str {
    match lingua {
        "ts" | "typescript" | "tsx" | "jsx" | "mjs" | "cjs" => "js",
        "shell" | "zsh" | "console" => "sh",
        "yml" => "yaml",
        "rs" => "rust",
        "py" | "python3" => "python",
        "md" => "markdown",
        "htm" => "html",
        outra => outra,
    }
}

fn sintaxes() -> &'static SyntaxSet {
    static S: OnceLock<SyntaxSet> = OnceLock::new();
    S.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// A gramática da linguagem da cerca, se houver uma.
fn gramatica(lingua: &str) -> Option<&'static SyntaxReference> {
    let l = lingua.trim().to_lowercase();
    if l.is_empty() {
        return None;
    }
    let ss = sintaxes();
    let l = apelido(&l);
    ss.find_syntax_by_token(l).or_else(|| ss.find_syntax_by_name(l))
}

/// TOML não vem nas gramáticas padrão do syntect, nem INI, que seria a
/// mais próxima. É uma linguagem de linha, então se pinta à mão: seção,
/// chave, string, número, booleano e comentário.
fn e_toml(lingua: &str) -> bool {
    lingua.trim().eq_ignore_ascii_case("toml")
}

/// Uma linha de TOML em trechos. `em_cadeia` é a string de várias linhas
/// (`"""`/`'''`) que veio aberta da linha anterior.
fn linha_toml(linha: &str, em_cadeia: &mut Option<&'static str>) -> Vec<Trecho> {
    let mut fora: Vec<Trecho> = Vec::new();
    let mut resto = linha;
    if let Some(fecho) = *em_cadeia {
        match resto.find(fecho) {
            Some(i) => {
                fora.push((Categoria::Cadeia, resto[..i + 3].to_string()));
                resto = &resto[i + 3..];
                *em_cadeia = None;
            }
            None => return vec![(Categoria::Cadeia, linha.to_string())],
        }
    }
    let t = resto.trim_start();
    if fora.is_empty() && t.starts_with('[') {
        // `[seção]` e `[[tabela]]`, com o comentário depois, se houver.
        let (sec, com) = match t.find('#') {
            Some(i) => (&resto[..resto.len() - t.len() + i], &t[i..]),
            None => (resto, ""),
        };
        fora.push((Categoria::Palavra, sec.to_string()));
        fora.push((Categoria::Comentario, com.to_string()));
        return juntar(fora);
    }
    // `chave =`: a chave é tudo antes do primeiro `=` fora de string.
    if fora.is_empty() {
        if let Some(i) = resto.find('=').filter(|i| !resto[..*i].contains(['"', '\'', '#'])) {
            fora.push((Categoria::Tipo, resto[..i].to_string()));
            fora.push((Categoria::Texto, "=".to_string()));
            resto = &resto[i + 1..];
        }
    }
    let mut chars = resto.char_indices().peekable();
    let mut solto = String::new();
    let solto_flush = |fora: &mut Vec<Trecho>, solto: &mut String| {
        if solto.is_empty() {
            return;
        }
        let palavra = solto.trim();
        let c = if palavra == "true" || palavra == "false" || (!palavra.is_empty() && palavra.chars().all(|c| c.is_ascii_digit() || "+-._:eETZ".contains(c)) && palavra.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '+' || c == '-')) {
            Categoria::Numero
        } else {
            Categoria::Texto
        };
        fora.push((c, std::mem::take(solto)));
    };
    while let Some((i, c)) = chars.next() {
        match c {
            '#' => {
                solto_flush(&mut fora, &mut solto);
                fora.push((Categoria::Comentario, resto[i..].to_string()));
                return juntar(fora);
            }
            '"' | '\'' => {
                solto_flush(&mut fora, &mut solto);
                let tripla: &'static str = if c == '"' { "\"\"\"" } else { "\'\'\'" };
                if resto[i..].starts_with(tripla) {
                    match resto[i + 3..].find(tripla) {
                        Some(f) => {
                            let fim = i + 3 + f + 3;
                            fora.push((Categoria::Cadeia, resto[i..fim].to_string()));
                            while chars.peek().is_some_and(|(j, _)| *j < fim) {
                                chars.next();
                            }
                        }
                        None => {
                            fora.push((Categoria::Cadeia, resto[i..].to_string()));
                            *em_cadeia = Some(tripla);
                            return juntar(fora);
                        }
                    }
                    continue;
                }
                let mut fim = resto.len();
                let mut escapado = false;
                for (j, d) in resto[i + 1..].char_indices() {
                    if escapado {
                        escapado = false;
                    } else if d == '\\' && c == '"' {
                        escapado = true;
                    } else if d == c {
                        fim = i + 1 + j + 1;
                        break;
                    }
                }
                fora.push((Categoria::Cadeia, resto[i..fim].to_string()));
                while chars.peek().is_some_and(|(j, _)| *j < fim) {
                    chars.next();
                }
            }
            ',' | '[' | ']' | '{' | '}' | ' ' => {
                solto_flush(&mut fora, &mut solto);
                fora.push((Categoria::Texto, c.to_string()));
            }
            _ => solto.push(c),
        }
    }
    solto_flush(&mut fora, &mut solto);
    juntar(fora)
}

/// A gramática conhece esta linguagem? Sem carregar nada quando a cerca
/// não diz linguagem nenhuma.
pub fn conhece(lingua: Option<&str>) -> bool {
    lingua.is_some_and(|l| e_toml(l) || gramatica(l).is_some())
}

struct Seletores {
    comentario: Scope,
    cadeia: Scope,
    numero: Scope,
    constante: Scope,
    palavra: Scope,
    operador: Scope,
    armazenamento: Scope,
    tag: Scope,
    tipo: Scope,
    suporte_tipo: Scope,
    suporte_classe: Scope,
    classe: Scope,
    atributo: Scope,
    titulo: Scope,
}

fn seletores() -> &'static Seletores {
    static S: OnceLock<Seletores> = OnceLock::new();
    S.get_or_init(|| {
        let s = |n: &str| Scope::new(n).expect("escopo válido");
        Seletores {
            comentario: s("comment"),
            cadeia: s("string"),
            numero: s("constant.numeric"),
            constante: s("constant.language"),
            palavra: s("keyword"),
            operador: s("keyword.operator"),
            armazenamento: s("storage"),
            tag: s("entity.name.tag"),
            tipo: s("entity.name.type"),
            suporte_tipo: s("support.type"),
            suporte_classe: s("support.class"),
            classe: s("entity.name.class"),
            atributo: s("entity.other.attribute-name"),
            titulo: s("markup.heading"),
        }
    })
}

/// A categoria do escopo mais interno que diz alguma coisa. Um operador
/// dentro de uma string continua string; um comentário dentro de qualquer
/// coisa é comentário.
fn categoria(pilha: &ScopeStack) -> Categoria {
    let s = seletores();
    for escopo in pilha.as_slice().iter().rev() {
        let e = |sel: &Scope| sel.is_prefix_of(*escopo);
        if e(&s.comentario) {
            return Categoria::Comentario;
        }
        if e(&s.cadeia) {
            return Categoria::Cadeia;
        }
        if e(&s.numero) || e(&s.constante) {
            return Categoria::Numero;
        }
        if e(&s.tipo) || e(&s.suporte_tipo) || e(&s.suporte_classe) || e(&s.classe) || e(&s.atributo) {
            return Categoria::Tipo;
        }
        // `=`, `>`, `+` são `keyword.operator` no Sublime. Pintados como
        // palavra-chave, a linha inteira fica roxa de pontuação.
        if e(&s.operador) {
            return Categoria::Texto;
        }
        if e(&s.palavra) || e(&s.armazenamento) || e(&s.tag) || e(&s.titulo) {
            return Categoria::Palavra;
        }
    }
    Categoria::Texto
}

/// Junta trechos vizinhos da mesma categoria: menos `Span` na tela.
fn juntar(trechos: Vec<Trecho>) -> Vec<Trecho> {
    let mut fora: Vec<Trecho> = Vec::with_capacity(trechos.len());
    for (c, t) in trechos {
        if t.is_empty() {
            continue;
        }
        match fora.last_mut() {
            Some((ultima, texto)) if *ultima == c => texto.push_str(&t),
            _ => fora.push((c, t)),
        }
    }
    fora
}

/// O código em linhas de trechos, ou `None` se a linguagem não tem
/// gramática (aí o bloco sai sem cor, RF3). As linhas já vêm com a
/// tabulação expandida, uma por linha do arquivo.
pub fn realcar(linhas: &[String], lingua: &str) -> Option<Vec<Vec<Trecho>>> {
    REALCADOS.with(|n| n.set(n.get() + 1));
    if e_toml(lingua) {
        let mut em_cadeia = None;
        return Some(linhas.iter().map(|l| linha_toml(l, &mut em_cadeia)).collect());
    }
    let g = gramatica(lingua)?;
    let ss = sintaxes();
    let mut estado = ParseState::new(g);
    let mut pilha = ScopeStack::new();
    let mut fora = Vec::with_capacity(linhas.len());
    for linha in linhas {
        // As gramáticas `_newlines` esperam a quebra no fim: sem ela, um
        // comentário de linha (`//`) vazaria pra linha seguinte.
        let com_quebra = format!("{linha}\n");
        let Ok(ops) = estado.parse_line(&com_quebra, ss) else {
            // Gramática que falhou no meio: o resto do bloco sem cor, mas
            // com o texto inteiro.
            fora.push(vec![(Categoria::Texto, linha.clone())]);
            continue;
        };
        let mut trechos = Vec::new();
        let mut pos = 0;
        for (i, op) in ops {
            let i = i.min(linha.len());
            if i > pos {
                trechos.push((categoria(&pilha), linha[pos..i].to_string()));
                pos = i;
            }
            let _ = pilha.apply(&op);
        }
        if pos < linha.len() {
            trechos.push((categoria(&pilha), linha[pos..].to_string()));
        }
        fora.push(juntar(trechos));
    }
    Some(fora)
}

thread_local! {
    /// Quantos blocos passaram pelo parser: o teste de escala confere que
    /// só os da tela passam, e uma vez.
    static REALCADOS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// O realce de cada bloco já visto, por (linguagem, código).
    static GUARDADOS: RefCell<HashMap<(String, String), Rc<Vec<Vec<Trecho>>>>> = RefCell::new(HashMap::new());
}

/// Quantos blocos ficam guardados. Passou disso, esvazia: uma página com
/// mais blocos que isso não cabe numa tela, e o que está na tela volta a
/// ser guardado no quadro seguinte.
const GUARDADOS_MAX: usize = 256;

/// Quantas vezes o parser rodou nesta thread.
pub fn realcados() -> usize {
    REALCADOS.with(|n| n.get())
}

/// `realcar`, guardado.
pub fn realcar_guardado(linhas: &[String], lingua: &str) -> Option<Rc<Vec<Vec<Trecho>>>> {
    let chave = (lingua.to_string(), linhas.join("\n"));
    if let Some(r) = GUARDADOS.with(|g| g.borrow().get(&chave).cloned()) {
        return Some(r);
    }
    let r = Rc::new(realcar(linhas, lingua)?);
    GUARDADOS.with(|g| {
        let mut g = g.borrow_mut();
        if g.len() >= GUARDADOS_MAX {
            g.clear();
        }
        g.insert(chave, r.clone());
    });
    Some(r)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn de(linha: &str, lingua: &str) -> Vec<Trecho> {
        realcar(&[linha.to_string()], lingua).expect("gramática")[0].clone()
    }

    fn cat(trechos: &[Trecho], pedaco: &str) -> Categoria {
        trechos
            .iter()
            .find(|(_, t)| t.contains(pedaco))
            .unwrap_or_else(|| panic!("{pedaco:?} não está em {trechos:?}"))
            .0
    }

    #[test]
    fn rust_tem_palavra_cadeia_numero_comentario_e_tipo() {
        let t = de(r#"let x: String = "oi"; // fim"#, "rust");
        assert_eq!(cat(&t, "let"), Categoria::Palavra);
        assert_eq!(cat(&t, "\"oi\""), Categoria::Cadeia);
        assert_eq!(cat(&t, "// fim"), Categoria::Comentario);
        assert_eq!(cat(&t, "String"), Categoria::Tipo);
        assert_eq!(cat(&de("let n = 42;", "rs"), "42"), Categoria::Numero);
        assert_eq!(cat(&de("if a > b {}", "rust"), ">"), Categoria::Texto, "operador como palavra-chave");
        // O texto da linha volta inteiro, na ordem.
        assert_eq!(t.iter().map(|(_, s)| s.as_str()).collect::<String>(), r#"let x: String = "oi"; // fim"#);
    }

    #[test]
    fn a_lista_minima_da_spec_tem_gramatica() {
        // RF2: Rust, JS/TS, Python, shell, JSON, YAML, TOML, HTML, CSS,
        // SQL e Markdown — pelos nomes que as cercas costumam usar.
        for l in [
            "rust", "rs", "js", "javascript", "ts", "typescript", "python", "py", "sh", "bash", "shell",
            "json", "yaml", "yml", "toml", "html", "css", "sql", "markdown", "md",
        ] {
            assert!(conhece(Some(l)), "sem gramática pra {l}");
        }
        assert!(!conhece(Some("linguagem-que-nao-existe")));
        assert!(!conhece(None));
        assert!(!conhece(Some("")));
    }

    #[test]
    fn comentario_de_linha_nao_vaza_pra_proxima() {
        let r = realcar(&["# um comentário".into(), "x = 1".into()], "python").unwrap();
        assert_eq!(r[0][0].0, Categoria::Comentario);
        assert_eq!(cat(&r[1], "x"), Categoria::Texto);
        assert_eq!(cat(&r[1], "1"), Categoria::Numero);
    }

    #[test]
    fn string_de_varias_linhas_continua_string() {
        let r = realcar(&["s = \"\"\"abre".into(), "meio".into(), "fecha\"\"\"".into()], "python").unwrap();
        assert_eq!(r[1][0].0, Categoria::Cadeia, "{:?}", r[1]);
    }

    #[test]
    fn toml_a_mao() {
        let t = de("chave = \"valor # não é comentário\" # nota", "toml");
        assert_eq!(cat(&t, "chave"), Categoria::Tipo);
        assert_eq!(cat(&t, "\"valor # não é comentário\""), Categoria::Cadeia);
        assert_eq!(cat(&t, "# nota"), Categoria::Comentario);
        let t = de("n = 42, b = true", "toml");
        assert_eq!(cat(&t, "42"), Categoria::Numero);
        assert_eq!(cat(&t, "true"), Categoria::Numero);
        assert_eq!(de("[dependencies] # x", "toml")[0], (Categoria::Palavra, "[dependencies] ".into()));
        assert_eq!(cat(&de("[[bin]]", "toml"), "[[bin]]"), Categoria::Palavra);
        // String de várias linhas atravessa as linhas.
        let r = realcar(&["d = \"\"\"abre".into(), "meio = 1".into(), "fecha\"\"\" # c".into()], "toml").unwrap();
        assert_eq!(r[1], vec![(Categoria::Cadeia, "meio = 1".into())]);
        assert_eq!(cat(&r[2], "# c"), Categoria::Comentario);
        // O texto volta inteiro.
        let linha = "x = [1, 'a', {y = 2}] # z";
        assert_eq!(de(linha, "toml").iter().map(|(_, s)| s.as_str()).collect::<String>(), linha);
    }

    #[test]
    fn ts_pelo_apelido() {
        assert_eq!(cat(&de("const x = 1;", "ts"), "const"), Categoria::Palavra);
    }

    #[test]
    fn guardado_devolve_o_mesmo() {
        let l = vec!["fn f() {}".to_string()];
        let a = realcar_guardado(&l, "rust").unwrap();
        let b = realcar_guardado(&l, "rust").unwrap();
        assert!(Rc::ptr_eq(&a, &b), "realçou duas vezes o mesmo bloco");
        assert!(realcar_guardado(&l, "nada-disso").is_none());
    }
}
