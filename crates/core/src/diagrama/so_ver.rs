//! Os diagramas que só se VEEM no terminal (ciclo 499): pizza, mapa
//! mental, linha do tempo, jornada, classes e entidade-relação.
//!
//! A janela desenha qualquer mermaid pelo mermaid.js; a TUI só desenhava o
//! que o construtor monta (496–497), e o resto aparecia como código. Estes
//! tipos não entram no construtor, então o leitor daqui é o oposto do de
//! [`super::ler`]: TOLERANTE. O de lá é estrito porque o que ele não
//! entende seria apagado no "Aplicar"; aqui nada volta pro arquivo, e uma
//! linha que não se entende (um `style`, um `click`, um ícone) só fica de
//! fora do desenho.

use super::texto::{cortar, grafo, largura_de, FormaG, LigG, Linha, NaoDesenha, NoG, Papel, Traco, ROTULO_MAX};

/// O código pronto pra ver: sem comentários (`%%`), sem diretivas
/// (`%%{…}%%`) e sem o cabeçalho `---` do mermaid, de onde sai o título
/// quando há. Vale pros tipos do construtor também: o leitor estrito
/// recusa comentário (pra não apagá-lo ao reescrever), e só pra VER o
/// comentário não faz falta.
pub fn limpar(codigo: &str) -> (String, Option<String>) {
    let mut titulo = None;
    let mut linhas = codigo.lines().peekable();
    // O cabeçalho só conta na primeira linha que não é branca.
    while linhas.peek().is_some_and(|l| l.trim().is_empty()) {
        linhas.next();
    }
    if linhas.peek().is_some_and(|l| l.trim() == "---") {
        linhas.next();
        for l in linhas.by_ref() {
            let t = l.trim();
            if t == "---" {
                break;
            }
            if let Some(v) = t.strip_prefix("title:") {
                titulo = Some(v.trim().trim_matches(['"', '\'']).to_string()).filter(|s| !s.is_empty());
            }
        }
    }
    let mut fora = String::new();
    for l in linhas {
        if l.trim_start().starts_with("%%") {
            continue;
        }
        fora.push_str(l);
        fora.push('\n');
    }
    (fora, titulo)
}

/// O nome do tipo, pro rótulo do bloco; `None` é "não é daqui".
pub fn tipo(codigo: &str) -> Option<&'static str> {
    let primeira = codigo.lines().map(str::trim).find(|l| !l.is_empty())?;
    let palavra = primeira.split_whitespace().next()?;
    Some(match palavra {
        "pie" => "pizza",
        "mindmap" => "mapa mental",
        "timeline" => "linha do tempo",
        "journey" => "jornada",
        "classDiagram" | "classDiagram-v2" => "classes",
        "erDiagram" => "entidades",
        _ => return None,
    })
}

/// O desenho em no máximo `largura` células, ou `None` quando o tipo não
/// é deste módulo.
pub fn desenhar(codigo: &str, largura: usize) -> Option<Result<Vec<Linha>, NaoDesenha>> {
    let primeira = codigo.lines().map(str::trim).find(|l| !l.is_empty())?;
    let palavra = primeira.split_whitespace().next()?;
    let corpo: Vec<&str> = codigo.lines().skip_while(|l| l.trim().is_empty()).skip(1).collect();
    let linhas = match palavra {
        "pie" => pizza(primeira, &corpo, largura),
        "mindmap" => cabe(mapa(&corpo), largura),
        "timeline" => cabe(linha_do_tempo(&corpo), largura),
        "journey" => cabe(jornada(&corpo), largura),
        "classDiagram" | "classDiagram-v2" => classes(&corpo, largura),
        "erDiagram" => entidades(&corpo, largura),
        _ => return None,
    };
    Some(linhas)
}

/// Como [`super::texto::desenhar_inteiro`]: na largura dele quando não cabe.
pub fn desenhar_inteiro(codigo: &str, largura: usize) -> Option<Result<Vec<Linha>, NaoDesenha>> {
    let mut largura = largura;
    for _ in 0..3 {
        match desenhar(codigo, largura)? {
            Err(NaoDesenha::NaoCabe { precisa }) if precisa > largura => largura = precisa,
            r => return Some(r),
        }
    }
    desenhar(codigo, largura)
}

fn largura_das(linhas: &[Linha]) -> usize {
    linhas.iter().map(|l| l.iter().map(|(_, s)| largura_de(s)).sum::<usize>()).max().unwrap_or(0)
}

/// Recusa o que passa da largura, como os desenhos do construtor.
fn cabe(linhas: Vec<Linha>, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let precisa = largura_das(&linhas);
    if precisa > largura {
        return Err(NaoDesenha::NaoCabe { precisa });
    }
    Ok(linhas)
}

/// `chave valor` de uma linha (`title Plano` → `Some("Plano")`).
fn depois_de<'a>(linha: &'a str, chave: &str) -> Option<&'a str> {
    let resto = linha.strip_prefix(chave)?;
    (resto.is_empty() || resto.starts_with(char::is_whitespace)).then(|| resto.trim())
}

/// O número no jeito do português: vírgula e sem zeros à toa.
fn numero(v: f64, casas: usize) -> String {
    let s = format!("{v:.casas$}");
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    s.replace('.', ",")
}

// ── pizza ─────────────────────────────────────────────────────────────

/// As fatias como barras: no terminal não há círculo que se leia, e barras
/// lado a lado comparam melhor que fatias (é o que a pizza quer dizer).
fn pizza(cabecalho: &str, corpo: &[&str], largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let mostrar_valor = cabecalho.contains("showData");
    let mut titulo = depois_de(cabecalho, "pie").and_then(|r| depois_de(r.trim_start_matches("showData").trim(), "title")).map(str::to_string);
    let mut fatias: Vec<(String, f64)> = Vec::new();
    for l in corpo.iter().map(|l| l.trim()) {
        if let Some(t) = depois_de(l, "title") {
            titulo = Some(t.to_string());
        } else if l.starts_with('"') {
            let Some(fim) = l[1..].find('"') else { continue };
            let nome = &l[1..1 + fim];
            let Some(v) = l[2 + fim..].trim().strip_prefix(':').and_then(|v| v.trim().parse::<f64>().ok()) else { continue };
            if v.is_finite() && v >= 0.0 {
                fatias.push((nome.to_string(), v));
            }
        }
    }
    let total: f64 = fatias.iter().map(|(_, v)| v).sum();
    let mut fora: Vec<Linha> = Vec::new();
    if let Some(t) = titulo {
        fora.push(vec![(Papel::Titulo, t)]);
    }
    if fatias.is_empty() || total <= 0.0 {
        return Ok(fora);
    }
    let nome_w = fatias.iter().map(|(n, _)| largura_de(n)).max().unwrap_or(0).min(24);
    let valores: Vec<String> = fatias.iter().map(|(_, v)| if mostrar_valor { format!(" {}", numero(*v, 2)) } else { String::new() }).collect();
    let valor_w = valores.iter().map(|v| largura_de(v)).max().unwrap_or(0);
    const BARRA_MIN: usize = 10;
    // nome, espaço, barra, espaço, "100%", valor
    let fixo = nome_w + 1 + 1 + 4 + valor_w;
    if largura < fixo + BARRA_MIN {
        return Err(NaoDesenha::NaoCabe { precisa: fixo + BARRA_MIN });
    }
    let barra_w = (largura - fixo).min(40);
    let maior = fatias.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    // Cada fatia numa cor, como a pizza da janela: as quatro barras do tema
    // e o marco, em rodízio.
    const CORES: [Papel; 5] = [Papel::Barra, Papel::BarraAtiva, Papel::BarraFeita, Papel::Marco, Papel::BarraCritica];
    for (i, ((nome, v), valor)) in fatias.iter().zip(&valores).enumerate() {
        let nome = cortar(nome, nome_w);
        let oitavos = ((v / maior) * (barra_w * 8) as f64).round() as usize;
        let mut barra = "█".repeat(oitavos / 8);
        if oitavos % 8 > 0 {
            barra.push(['▏', '▎', '▍', '▌', '▋', '▊', '▉'][oitavos % 8 - 1]);
        }
        let barra_ocupa = largura_de(&barra);
        let pct = format!("{:>3}%", (v / total * 100.0).round() as i64);
        fora.push(vec![
            (Papel::Nome, format!("{nome}{} ", " ".repeat(nome_w - largura_de(&nome)))),
            (CORES[i % CORES.len()], barra),
            (Papel::Fundo, format!("{} ", " ".repeat(barra_w - barra_ocupa))),
            (Papel::Rotulo, pct),
            (Papel::Nome, valor.clone()),
        ]);
    }
    Ok(fora)
}

// ── mapa mental ───────────────────────────────────────────────────────

/// O texto de um nó do mapa: sem o id e sem a forma (`id((texto))`,
/// `id[texto]`, `id{{texto}}`, `))texto((`…), sem a classe (`:::x`).
fn texto_do_no(t: &str) -> String {
    let t = t.split(":::").next().unwrap_or(t).trim();
    let pares = [("((", "))"), ("))", "(("), ("{{", "}}"), ("(", ")"), (")", "("), ("[", "]")];
    for (abre, fecha) in pares {
        if let Some(i) = t.find(abre) {
            if t.ends_with(fecha) && t.len() >= i + abre.len() + fecha.len() {
                let miolo = &t[i + abre.len()..t.len() - fecha.len()];
                return miolo.trim().trim_matches('"').trim_matches('`').to_string();
            }
        }
    }
    t.trim_matches('"').trim_matches('`').to_string()
}

/// A árvore pela indentação, desenhada com `├─`/`└─`.
fn mapa(corpo: &[&str]) -> Vec<Linha> {
    let mut nos: Vec<(usize, String)> = Vec::new();
    for l in corpo {
        let t = l.trim();
        if t.is_empty() || t.starts_with("::icon(") {
            continue;
        }
        let recuo = l.len() - l.trim_start().len();
        nos.push((recuo, texto_do_no(t)));
    }
    let Some((recuo_raiz, raiz)) = nos.first().cloned() else { return Vec::new() };
    // Profundidade de cada nó: a pilha dos recuos dos ancestrais.
    let mut prof = Vec::with_capacity(nos.len());
    let mut pilha: Vec<usize> = vec![recuo_raiz];
    for (recuo, _) in nos.iter().skip(1) {
        while pilha.len() > 1 && *pilha.last().unwrap() >= *recuo {
            pilha.pop();
        }
        pilha.push(*recuo);
        prof.push(pilha.len() - 1);
    }
    let mut fora: Vec<Linha> = vec![vec![(Papel::Caixa, "( ".into()), (Papel::Titulo, raiz), (Papel::Caixa, " )".into())]];
    let n = prof.len();
    // É o último irmão? Olha à frente até achar alguém da mesma
    // profundidade (irmão) ou mais raso (acabou o pai).
    let ultimo = |i: usize| -> bool {
        for &p in &prof[i + 1..] {
            if p < prof[i] {
                return true;
            }
            if p == prof[i] {
                return false;
            }
        }
        true
    };
    let mut abertos: Vec<bool> = Vec::new(); // por profundidade: ainda tem irmão embaixo
    for i in 0..n {
        let p = prof[i].max(1);
        abertos.truncate(p - 1);
        let mut prefixo = String::new();
        for &aberto in &abertos {
            prefixo.push_str(if aberto { "│  " } else { "   " });
        }
        let fim = ultimo(i);
        prefixo.push_str(if fim { "└─ " } else { "├─ " });
        while abertos.len() < p - 1 {
            abertos.push(false);
        }
        abertos.push(!fim);
        let papel = if p == 1 { Papel::Secao } else { Papel::Nome };
        fora.push(vec![(Papel::Seta, prefixo), (papel, cortar(&nos[i + 1].1, 60))]);
    }
    fora
}

// ── linha do tempo ────────────────────────────────────────────────────

fn linha_do_tempo(corpo: &[&str]) -> Vec<Linha> {
    let mut fora: Vec<Linha> = Vec::new();
    // (seção, período, eventos)
    let mut itens: Vec<(Option<String>, String, Vec<String>)> = Vec::new();
    let mut secao: Option<String> = None;
    for l in corpo.iter().map(|l| l.trim()) {
        if l.is_empty() {
            continue;
        }
        if let Some(t) = depois_de(l, "title") {
            fora.push(vec![(Papel::Titulo, t.to_string())]);
        } else if let Some(s) = depois_de(l, "section") {
            secao = Some(s.to_string());
        } else if let Some(resto) = l.strip_prefix(':') {
            // Continuação: mais eventos do período de cima.
            if let Some(ultimo) = itens.last_mut() {
                ultimo.2.extend(resto.split(" : ").map(|e| e.trim().trim_start_matches(':').trim().to_string()).filter(|e| !e.is_empty()));
            }
        } else {
            let mut partes = l.split(':').map(str::trim);
            let periodo = partes.next().unwrap_or("").to_string();
            let eventos: Vec<String> = partes.filter(|e| !e.is_empty()).map(str::to_string).collect();
            itens.push((secao.clone(), periodo, eventos));
        }
    }
    let periodo_w = itens.iter().map(|(_, p, _)| largura_de(p)).max().unwrap_or(0).min(20);
    let mut secao_vista: Option<&Option<String>> = None;
    for (k, (secao, periodo, eventos)) in itens.iter().enumerate() {
        if secao.is_some() && secao_vista != Some(secao) {
            fora.push(vec![(Papel::Secao, format!("━━ {}", secao.as_deref().unwrap_or("")))]);
        }
        secao_vista = Some(secao);
        let periodo = cortar(periodo, periodo_w);
        let vazio = " ".repeat(periodo_w);
        let pad = " ".repeat(periodo_w - largura_de(&periodo));
        if eventos.is_empty() {
            fora.push(vec![(Papel::Nome, format!("{periodo}{pad} ")), (Papel::Seta, "●".into())]);
        }
        for (i, ev) in eventos.iter().enumerate() {
            let (cabeca, marca) = if i == 0 { (format!("{periodo}{pad} "), "●─ ") } else { (format!("{vazio} "), "├─ ") };
            fora.push(vec![(Papel::Nome, cabeca), (Papel::Seta, marca.into()), (Papel::Rotulo, cortar(ev, 60))]);
        }
        if k + 1 < itens.len() {
            fora.push(vec![(Papel::Fundo, format!("{vazio} ")), (Papel::Seta, "│".into())]);
        }
    }
    fora
}

// ── jornada ───────────────────────────────────────────────────────────

fn jornada(corpo: &[&str]) -> Vec<Linha> {
    let mut fora: Vec<Linha> = Vec::new();
    let mut tarefas: Vec<(Option<String>, String, u8, String)> = Vec::new();
    let mut secao = None;
    for l in corpo.iter().map(|l| l.trim()) {
        if let Some(t) = depois_de(l, "title") {
            fora.push(vec![(Papel::Titulo, t.to_string())]);
        } else if let Some(s) = depois_de(l, "section") {
            secao = Some(s.to_string());
        } else if let Some((nome, resto)) = l.split_once(':') {
            let (nota, quem) = resto.split_once(':').unwrap_or((resto, ""));
            let Ok(nota) = nota.trim().parse::<u8>() else { continue };
            tarefas.push((secao.clone(), nome.trim().to_string(), nota.min(5), quem.trim().to_string()));
        }
    }
    let nome_w = tarefas.iter().map(|t| largura_de(&t.1)).max().unwrap_or(0).min(28);
    let mut secao_vista: Option<Option<String>> = None;
    for (secao, nome, nota, quem) in tarefas {
        if secao.is_some() && secao_vista.as_ref() != Some(&secao) {
            fora.push(vec![(Papel::Secao, secao.clone().unwrap_or_default())]);
        }
        secao_vista = Some(secao);
        let nome = cortar(&nome, nome_w);
        // A nota em estrelas, na cor do humor: 1–2 crítica, 3 neutra, 4–5 boa.
        let cor = match nota {
            0..=2 => Papel::BarraCritica,
            3 => Papel::Marco,
            _ => Papel::BarraFeita,
        };
        fora.push(vec![
            (Papel::Nome, format!("  {nome}{}  ", " ".repeat(nome_w - largura_de(&nome)))),
            (cor, "★".repeat(nota as usize)),
            (Papel::Vida, "☆".repeat(5 - nota as usize)),
            (Papel::Rotulo, if quem.is_empty() { String::new() } else { format!("  {quem}") }),
        ]);
    }
    fora
}

// ── classes ───────────────────────────────────────────────────────────

/// `Nome~T~` → `Nome<T>`, a notação de genérico que se lê.
fn generico(s: &str) -> String {
    let mut fora = String::new();
    let mut aberto = false;
    for c in s.chars() {
        if c == '~' {
            fora.push(if aberto { '>' } else { '<' });
            aberto = !aberto;
        } else {
            fora.push(c);
        }
    }
    fora
}

/// Um id de classe ou entidade: letras, dígitos, `_`, `-` e o `~` do
/// genérico.
fn eh_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '~' | '.'))
}

struct Caixa {
    id: String,
    rotulo: String,
    corpo: Vec<String>,
}

fn caixa_de<'a>(caixas: &'a mut Vec<Caixa>, id: &str) -> &'a mut Caixa {
    let id = id.split(":::").next().unwrap_or(id).trim();
    let base = id.split('~').next().unwrap_or(id).to_string();
    if let Some(i) = caixas.iter().position(|c| c.id == base) {
        return &mut caixas[i];
    }
    caixas.push(Caixa { id: base, rotulo: generico(id), corpo: Vec::new() });
    caixas.last_mut().unwrap()
}

/// Uma relação de classes: a ponta da esquerda, o traço e a da direita.
fn relacao_de_classe(op: &str) -> Option<(&str, bool, &str)> {
    let (meio, pontilhado) = if let Some(i) = op.find("..") {
        (i, true)
    } else {
        (op.find("--")?, false)
    };
    let esq = &op[..meio];
    let dir = &op[meio + 2..];
    let ok = |s: &str| matches!(s, "" | "<|" | "|>" | "*" | "o" | "<" | ">");
    (ok(esq) && ok(dir)).then_some((esq, pontilhado, dir))
}

fn sem_aspas(s: &str) -> &str {
    s.trim().trim_matches('"')
}

/// `de "1" --> "*" para : rótulo` em pedaços.
fn partes_da_relacao(l: &str) -> Option<(String, String, String, String, String, String)> {
    let (antes, rotulo) = match l.split_once(" : ") {
        Some((a, r)) => (a, r.trim().to_string()),
        None => match l.rsplit_once(':') {
            // `A --> B:rótulo` sem espaço; mas não o `::` de nada.
            Some((a, r)) if !a.ends_with(':') && !r.starts_with(':') && (a.contains("--") || a.contains("..")) => (a, r.trim().to_string()),
            _ => (l, String::new()),
        },
    };
    // Os pedaços: id, "card"?, op, "card"?, id.
    let mut toks: Vec<String> = Vec::new();
    let mut atual = String::new();
    let mut aspas = false;
    for c in antes.chars() {
        match c {
            '"' => {
                atual.push(c);
                if aspas {
                    toks.push(std::mem::take(&mut atual));
                }
                aspas = !aspas;
            }
            c if c.is_whitespace() && !aspas => {
                if !atual.is_empty() {
                    toks.push(std::mem::take(&mut atual));
                }
            }
            c => atual.push(c),
        }
    }
    if !atual.is_empty() {
        toks.push(atual);
    }
    let card = |t: &String| t.starts_with('"');
    let mut it = toks.into_iter().peekable();
    let de = it.next()?;
    let card_de = if it.peek().is_some_and(card) { sem_aspas(&it.next()?).to_string() } else { String::new() };
    let op = it.next()?;
    let card_para = if it.peek().is_some_and(card) { sem_aspas(&it.next()?).to_string() } else { String::new() };
    let para = it.next()?;
    if it.next().is_some() {
        return None;
    }
    Some((de, card_de, op, card_para, para, rotulo))
}

fn classes(corpo: &[&str], largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let mut caixas: Vec<Caixa> = Vec::new();
    let mut ligs: Vec<(String, String, String, Traco, bool)> = Vec::new();
    let mut horizontal = false;
    let mut dentro: Option<String> = None;
    let mut anotacoes: Vec<(String, String)> = Vec::new();
    for l in corpo.iter().map(|l| l.trim()) {
        if l.is_empty() {
            continue;
        }
        if let Some(id) = dentro.clone() {
            if l == "}" {
                dentro = None;
            } else if l.starts_with("<<") && l.ends_with(">>") {
                anotacoes.push((id, l[2..l.len() - 2].to_string()));
            } else {
                caixa_de(&mut caixas, &id).corpo.push(cortar(&generico(l), 40));
            }
            continue;
        }
        if let Some(d) = depois_de(l, "direction") {
            horizontal = matches!(d, "LR" | "RL");
            continue;
        }
        if ["note", "style", "classDef", "cssClass", "click", "callback", "link", "title", "accTitle", "accDescr"]
            .iter()
            .any(|k| depois_de(l, k).is_some() || l.starts_with(&format!("{k}:")))
            || l == "}"
            || l.starts_with("namespace ")
        {
            continue;
        }
        if let Some(resto) = depois_de(l, "class") {
            let resto = resto.trim();
            let (cabeca, abre) = match resto.strip_suffix('{') {
                Some(c) => (c.trim(), true),
                None => (resto, false),
            };
            // `class X["Rótulo"]`
            let (id, rotulo) = match cabeca.split_once('[') {
                Some((id, r)) => (id.trim(), Some(sem_aspas(r.trim_end_matches(']')).to_string())),
                None => (cabeca, None),
            };
            if !eh_id(id.split(":::").next().unwrap_or(id)) {
                continue;
            }
            let c = caixa_de(&mut caixas, id);
            if let Some(r) = rotulo {
                c.rotulo = r;
            }
            let base = c.id.clone();
            if abre {
                dentro = Some(base);
            }
            continue;
        }
        if l.starts_with("<<") {
            if let Some((a, id)) = l[2..].split_once(">>") {
                anotacoes.push((caixa_de(&mut caixas, id.trim()).id.clone(), a.to_string()));
            }
            continue;
        }
        if let Some((de, card_de, op, card_para, para, rotulo)) = partes_da_relacao(l) {
            if let Some((esq, pontilhado, dir)) = relacao_de_classe(&op) {
                if eh_id(&de) && eh_id(&para) {
                    let de = caixa_de(&mut caixas, &de).id.clone();
                    let para = caixa_de(&mut caixas, &para).id.clone();
                    // Quem tem o losango ou o triângulo fica em cima: o pai
                    // da herança, o todo da composição. A seta simples
                    // aponta pra onde aponta.
                    let (cima, baixo, ponta, tipo) = match (esq, dir) {
                        ("<|", _) => (de, para, false, if pontilhado { "realiza" } else { "herda" }),
                        (_, "|>") => (para, de, false, if pontilhado { "realiza" } else { "herda" }),
                        ("*", _) => (de, para, false, "compõe"),
                        (_, "*") => (para, de, false, "compõe"),
                        ("o", _) => (de, para, false, "agrega"),
                        (_, "o") => (para, de, false, "agrega"),
                        (_, ">") => (de, para, true, if pontilhado { "usa" } else { "" }),
                        ("<", _) => (para, de, true, if pontilhado { "usa" } else { "" }),
                        _ => (de, para, false, ""),
                    };
                    let mut r: Vec<String> = Vec::new();
                    if !tipo.is_empty() {
                        r.push(tipo.to_string());
                    }
                    if !rotulo.is_empty() {
                        r.push(rotulo);
                    }
                    if !card_de.is_empty() || !card_para.is_empty() {
                        let a = if card_de.is_empty() { "?" } else { card_de.as_str() };
                        let b = if card_para.is_empty() { "?" } else { card_para.as_str() };
                        r.push(format!("{a}–{b}"));
                    }
                    let traco = if pontilhado { Traco::Pontilhado } else { Traco::Cheio };
                    ligs.push((cima, baixo, r.join(" · "), traco, ponta));
                    continue;
                }
            }
        }
        // `Nome : +membro`
        if let Some((id, membro)) = l.split_once(':') {
            let id = id.trim();
            if eh_id(id) && !membro.trim().is_empty() {
                caixa_de(&mut caixas, id).corpo.push(cortar(&generico(membro.trim()), 40));
            }
        }
    }
    for (id, a) in anotacoes {
        if let Some(c) = caixas.iter_mut().find(|c| c.id == id) {
            c.rotulo = format!("«{a}» {}", c.rotulo);
        }
    }
    desenhar_caixas(caixas, ligs, horizontal, largura)
}

fn desenhar_caixas(
    caixas: Vec<Caixa>,
    ligs: Vec<(String, String, String, Traco, bool)>,
    horizontal: bool,
    largura: usize,
) -> Result<Vec<Linha>, NaoDesenha> {
    let idx = |id: &str| caixas.iter().position(|c| c.id == id);
    let ligs: Vec<LigG> = ligs
        .iter()
        .filter_map(|(de, para, rotulo, traco, ponta)| {
            Some(LigG { de: idx(de)?, para: idx(para)?, rotulo: cortar(rotulo, ROTULO_MAX), traco: *traco, ponta: *ponta })
        })
        .collect();
    let nos: Vec<NoG> = caixas
        .into_iter()
        .map(|c| NoG { rotulo: cortar(&c.rotulo, ROTULO_MAX + 12), forma: FormaG::Caixa, corpo: c.corpo })
        .collect();
    grafo(&nos, &ligs, horizontal, false, largura)
}

// ── entidade-relação ──────────────────────────────────────────────────

/// A cardinalidade de uma ponta do ER, em palavras de quem lê.
fn cardinalidade(s: &str) -> Option<&'static str> {
    Some(match s {
        "|o" | "o|" => "0..1",
        "||" => "1",
        "}o" | "o{" => "0..N",
        "}|" | "|{" => "1..N",
        _ => return None,
    })
}

fn entidades(corpo: &[&str], largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let mut caixas: Vec<Caixa> = Vec::new();
    let mut ligs: Vec<(String, String, String, Traco, bool)> = Vec::new();
    let mut horizontal = false;
    let mut dentro: Option<String> = None;
    for l in corpo.iter().map(|l| l.trim()) {
        if l.is_empty() {
            continue;
        }
        if let Some(id) = dentro.clone() {
            if l == "}" {
                dentro = None;
                continue;
            }
            // `tipo nome PK, FK "comentário"`
            let sem_comentario = match l.find('"') {
                Some(i) => l[..i].trim(),
                None => l,
            };
            let mut p = sem_comentario.split_whitespace();
            let (Some(tipo), Some(nome)) = (p.next(), p.next()) else { continue };
            let chaves: String = p.collect::<Vec<_>>().join(" ").replace(',', "").split_whitespace().collect::<Vec<_>>().join(",");
            let mut item = format!("{nome}: {}", generico(tipo));
            if !chaves.is_empty() {
                item.push_str(&format!(" {chaves}"));
            }
            caixa_de(&mut caixas, &id).corpo.push(cortar(&item, 40));
            continue;
        }
        if let Some(d) = depois_de(l, "direction") {
            horizontal = matches!(d, "LR" | "RL");
            continue;
        }
        if let Some(cabeca) = l.strip_suffix('{') {
            let cabeca = cabeca.trim();
            let (id, rotulo) = match cabeca.split_once('[') {
                Some((id, r)) => (id.trim(), Some(sem_aspas(r.trim_end_matches(']')).to_string())),
                None => (cabeca, None),
            };
            let id = sem_aspas(id);
            let c = caixa_de(&mut caixas, id);
            if let Some(r) = rotulo {
                c.rotulo = r;
            }
            dentro = Some(c.id.clone());
            continue;
        }
        // `A ||--o{ B : rótulo`
        let (antes, rotulo) = match l.split_once(':') {
            Some((a, r)) => (a.trim(), sem_aspas(r).to_string()),
            None => (l, String::new()),
        };
        let toks: Vec<&str> = antes.split_whitespace().collect();
        if let [de, op, para] = toks[..] {
            if op.len() == 6 && op.is_char_boundary(2) && op.is_char_boundary(4) {
                let (e, meio, d) = (&op[..2], &op[2..4], &op[4..]);
                if let (Some(ce), Some(cd), true) = (cardinalidade(e), cardinalidade(d), meio == "--" || meio == "..") {
                    let de = caixa_de(&mut caixas, sem_aspas(de)).id.clone();
                    let para = caixa_de(&mut caixas, sem_aspas(para)).id.clone();
                    let mut r = rotulo;
                    if !r.is_empty() {
                        r.push_str(" · ");
                    }
                    r.push_str(&format!("{ce}–{cd}"));
                    let traco = if meio == ".." { Traco::Pontilhado } else { Traco::Cheio };
                    ligs.push((de, para, r, traco, false));
                    continue;
                }
            }
        }
        // Uma entidade sozinha numa linha.
        if toks.len() == 1 && eh_id(toks[0]) {
            caixa_de(&mut caixas, toks[0]);
        }
    }
    desenhar_caixas(caixas, ligs, horizontal, largura)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texto(fonte: &str, largura: usize) -> Vec<String> {
        let (limpo, _) = limpar(fonte);
        desenhar(&limpo, largura)
            .expect("o tipo devia ser deste módulo")
            .unwrap_or_else(|e| panic!("{e:?}"))
            .iter()
            .map(|l| l.iter().map(|(_, s)| s.as_str()).collect::<String>().trim_end().to_string())
            .collect()
    }

    #[test]
    fn limpar_tira_comentario_diretiva_e_cabecalho_e_guarda_o_titulo() {
        let (c, t) = limpar("---\ntitle: Meu fluxo\nconfig:\n  theme: dark\n---\n%%{init: {}}%%\ngraph TD\n  %% um comentário\n  A --> B\n");
        assert_eq!(t.as_deref(), Some("Meu fluxo"));
        assert_eq!(c, "graph TD\n  A --> B\n");
        // Comentário no meio de um fluxograma não impede mais o desenho.
        assert!(super::super::ler(&c).is_ok());
    }

    #[test]
    fn pizza_vira_barras_com_a_porcentagem() {
        let t = texto("pie showData\n  title Pets\n  \"Cães\" : 386\n  \"Gatos\" : 85.9\n  \"Ratos\" : 15\n", 60);
        assert_eq!(t[0], "Pets");
        assert!(t[1].starts_with("Cães  █") && t[1].ends_with(" 79% 386"), "{t:?}");
        assert!(t[2].starts_with("Gatos █") && t[2].ends_with(" 18% 85,9"), "{t:?}");
        // A maior enche a barra; as outras na proporção.
        let barra = |l: &str| l.chars().filter(|c| "█▏▎▍▌▋▊▉".contains(*c)).count();
        assert!(barra(&t[1]) > 4 * barra(&t[2]) - 2, "{t:?}");
        assert!(t.iter().all(|l| largura_de(l) <= 60));
        assert!(matches!(desenhar("pie\n \"a\" : 1\n", 8), Some(Err(NaoDesenha::NaoCabe { .. }))));
    }

    #[test]
    fn mapa_mental_vira_arvore_pela_indentacao() {
        let t = texto(
            "mindmap\n  root((Projeto))\n    Pesquisa\n      ::icon(fa fa-book)\n      Usuários\n      Mercado\n    Construção\n      id1[Protótipo]\n        Testes\n",
            80,
        );
        assert_eq!(
            t,
            [
                "( Projeto )",
                "├─ Pesquisa",
                "│  ├─ Usuários",
                "│  └─ Mercado",
                "└─ Construção",
                "   └─ Protótipo",
                "      └─ Testes",
            ]
        );
    }

    #[test]
    fn linha_do_tempo_com_secao_e_continuacao() {
        let t = texto(
            "timeline\n  title Redes\n  section Começo\n    2002 : LinkedIn\n    2004 : Facebook : Google\n         : Orkut\n  section Depois\n    2005 : YouTube\n",
            80,
        );
        let tudo = t.join("\n");
        assert_eq!(t[0], "Redes");
        assert_eq!(t[1], "━━ Começo");
        assert!(tudo.contains("2002 ●─ LinkedIn"), "{tudo}");
        assert!(tudo.contains("2004 ●─ Facebook\n     ├─ Google\n     ├─ Orkut"), "{tudo}");
        assert!(tudo.contains("━━ Depois\n2005 ●─ YouTube"), "{tudo}");
    }

    #[test]
    fn jornada_com_nota_em_estrelas() {
        let t = texto("journey\n  title Dia\n  section Manhã\n    Fazer café: 5: Eu\n    Trabalhar: 2: Eu, Gato\n", 80);
        assert_eq!(t[0], "Dia");
        assert_eq!(t[1], "Manhã");
        assert_eq!(t[2], "  Fazer café  ★★★★★  Eu");
        assert_eq!(t[3], "  Trabalhar   ★★☆☆☆  Eu, Gato");
    }

    #[test]
    fn classes_com_membros_e_heranca() {
        let t = texto(
            "classDiagram\n  class Animal {\n    <<abstract>>\n    +String nome\n    +comer() void\n  }\n  Animal <|-- Pato\n  Pato : +nadar()\n  Pato \"1\" --> \"*\" Ovo : bota\n  style Pato fill:#f9f\n",
            100,
        );
        let tudo = t.join("\n");
        assert!(tudo.contains("│ «abstract» Animal │"), "{tudo}");
        assert!(tudo.contains("├────") && tudo.contains("│ +String nome"), "{tudo}");
        assert!(tudo.contains("│ +comer() void"), "{tudo}");
        assert!(tudo.contains("▼") || tudo.contains("herda"), "{tudo}");
        assert!(tudo.contains("herda") && tudo.contains("bota · 1–*"), "{tudo}");
        assert!(tudo.contains("│ +nadar()"), "{tudo}");
        // O pai fica em cima do filho.
        let linha = |s: &str| t.iter().position(|l| l.contains(s)).unwrap();
        assert!(linha("Animal") < linha("Pato") && linha("Pato") < linha("Ovo"), "{tudo}");
        assert!(!tudo.contains("fill"), "{tudo}");
    }

    #[test]
    fn entidades_com_atributos_e_cardinalidade() {
        let t = texto(
            "erDiagram\n  CLIENTE ||--o{ PEDIDO : faz\n  PEDIDO ||--|{ ITEM : contém\n  CLIENTE {\n    string nome\n    string email PK \"o login\"\n  }\n",
            100,
        );
        let tudo = t.join("\n");
        assert!(tudo.contains("│ CLIENTE"), "{tudo}");
        assert!(tudo.contains("│ nome: string") && tudo.contains("│ email: string PK"), "{tudo}");
        assert!(!tudo.contains("o login"), "{tudo}");
        assert!(tudo.contains("faz · 1–0..N") && tudo.contains("contém · 1–1..N"), "{tudo}");
    }

    #[test]
    fn tipo_desconhecido_nao_e_deste_modulo() {
        assert!(desenhar("gitGraph\n  commit\n", 80).is_none());
        assert!(desenhar("graph TD\n A-->B\n", 80).is_none());
        assert_eq!(tipo("  \nerDiagram\n"), Some("entidades"));
    }

    #[test]
    fn largos_desenham_inteiros_e_nada_entra_em_panico() {
        // Entradas estranhas não derrubam ninguém.
        for f in [
            "pie\n", "pie\n \"a\" : -1\n \"b\" : x\n", "mindmap\n", "mindmap\n a\n b\n", "timeline\n : sem período\n",
            "journey\n tarefa: nota: x\n", "classDiagram\n class {\n }\n A <|-- \n", "erDiagram\n A ||--o{ \n",
            "classDiagram\n A --> A\n", "erDiagram\n A ||--|| A : eu\n",
        ] {
            let _ = desenhar_inteiro(f, 10);
        }
        let largo = "classDiagram\n  class UmaClasseComNomeMuitoComprido\n  class OutraClasseComNomeMuitoComprido\n  UmaClasseComNomeMuitoComprido --> OutraClasseComNomeMuitoComprido\n  direction LR\n";
        assert!(matches!(desenhar(largo, 30), Some(Err(NaoDesenha::NaoCabe { .. }))));
        let t = desenhar_inteiro(largo, 30).unwrap().unwrap();
        assert!(largura_das(&t) > 30);
    }
}
