//! Transclusão resolvida: `![[Página]]` vira o conteúdo dela (ciclo 414).
//!
//! A janela já DESENHA transclusão desde o ciclo 170: a página mostra o
//! pedaço de outra, ao vivo. O que não existia era resolver o texto —
//! trocar o marcador pelo conteúdo — e é isso que serve pra montar
//! contexto de prompt.
//!
//! # Por que isto muda a montagem de contexto
//!
//! Hoje anexar é escolher página inteira, uma a uma, na mão. Com a
//! transclusão resolvida, dá pra manter no vault uma página-recorte —
//! "o que o agente precisa saber pra mexer nas specs" — feita de
//! `![[Padrões de nomenclatura#Regras]]` e `![[Spec atual]]`, e anexar
//! só ELA. O recorte passa a ser um documento do vault: versionado,
//! editável, revisável, e igual pra janela, TUI e CLI.
//!
//! # Os três cuidados
//!
//! - **Ciclo**: A transclui B que transclui A. Aqui o segundo encontro
//!   vira aviso, não recursão infinita.
//! - **Profundidade**: cada nível puxa mais texto; três é o teto.
//! - **Honestidade**: o que não deu pra trazer (página não existe, seção
//!   errada, ciclo) NÃO some — vira uma linha visível. Contexto que
//!   mente por omissão é pior que contexto faltando, porque ninguém
//!   descobre.
//!
//! Quem lê arquivo é quem chama: aqui entra uma função de busca, e o
//! módulo continua sem IO.

use crate::links;

/// Até onde a transclusão desce. Além disso, vira aviso.
pub const PROFUNDIDADE_MAXIMA: usize = 3;

/// O alvo de um `![[…]]`, já separado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alvo {
    /// Título ou nome de arquivo da página.
    pub titulo: String,
    /// `#Seção`, quando pedida.
    pub secao: Option<String>,
    /// `^bloco`, quando pedido.
    pub bloco: Option<String>,
}

/// Separa `Página#Seção` / `Página^bloco` / `Página`.
///
/// O `^` é testado primeiro porque um id de bloco não contém `#`, mas
/// uma seção pode conter `^` no texto.
pub fn analisar_alvo(bruto: &str) -> Alvo {
    let bruto = bruto.trim();
    if let Some((titulo, bloco)) = bruto.split_once('^') {
        return Alvo {
            titulo: titulo.trim().to_string(),
            secao: None,
            bloco: Some(bloco.trim().to_string()).filter(|b| !b.is_empty()),
        };
    }
    match bruto.split_once('#') {
        Some((titulo, secao)) => Alvo {
            titulo: titulo.trim().to_string(),
            secao: Some(secao.trim().to_string()).filter(|s| !s.is_empty()),
            bloco: None,
        },
        None => Alvo { titulo: bruto.to_string(), secao: None, bloco: None },
    }
}

/// O pedaço do corpo que o alvo pede. `Err` com o motivo, na voz de quem
/// vai ler o aviso no prompt.
pub fn recortar(corpo: &str, alvo: &Alvo) -> Result<String, String> {
    if let Some(id) = &alvo.bloco {
        return match links::find_block(corpo, id) {
            Some(t) => Ok(t.trim().to_string()),
            None => Err(format!("a página {} não tem o bloco ^{id}", alvo.titulo)),
        };
    }
    if let Some(secao) = &alvo.secao {
        return match links::extract_section(corpo, secao) {
            Some(t) => Ok(t.trim().to_string()),
            None => Err(format!("a página {} não tem a seção {secao}", alvo.titulo)),
        };
    }
    Ok(corpo.trim().to_string())
}

/// O que a resolução produziu.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resolvido {
    /// O texto com os `![[…]]` trocados pelo conteúdo.
    pub texto: String,
    /// As páginas que entraram, na ordem, sem repetir — é o que a tela
    /// mostra como "trouxe também".
    pub trazidas: Vec<String>,
    /// O que não deu pra trazer, com o motivo.
    pub avisos: Vec<String>,
}

/// Resolve as transclusões de um texto.
///
/// `buscar` recebe o título e devolve `(path, corpo)` da página — o path
/// é o que identifica a página pro controle de ciclo, porque dois
/// títulos podem apontar pro mesmo arquivo.
///
/// `origem` é o path de quem está sendo resolvido: sem ele, uma página
/// que transclui a si mesma passaria uma vez antes de o ciclo ser
/// notado.
pub fn resolver(
    texto: &str,
    origem: &str,
    buscar: &mut dyn FnMut(&str) -> Option<(String, String)>,
) -> Resolvido {
    let mut visitadas: Vec<String> = vec![origem.to_string()];
    let mut r = Resolvido::default();
    r.texto = resolver_em(texto, &mut visitadas, 0, buscar, &mut r.trazidas, &mut r.avisos);
    r
}

fn resolver_em(
    texto: &str,
    visitadas: &mut Vec<String>,
    profundidade: usize,
    buscar: &mut dyn FnMut(&str) -> Option<(String, String)>,
    trazidas: &mut Vec<String>,
    avisos: &mut Vec<String>,
) -> String {
    let mut fora = String::new();
    let mut em_cerca = false;
    for linha in texto.split_inclusive('\n') {
        let t = linha.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            em_cerca = !em_cerca;
            fora.push_str(linha);
            continue;
        }
        // Dentro de bloco de código `![[x]]` é texto, não transclusão —
        // a mesma regra da extração de links.
        if em_cerca || !linha.contains("![[") {
            fora.push_str(linha);
            continue;
        }
        fora.push_str(&resolver_na_linha(linha, visitadas, profundidade, buscar, trazidas, avisos));
    }
    fora
}

fn resolver_na_linha(
    linha: &str,
    visitadas: &mut Vec<String>,
    profundidade: usize,
    buscar: &mut dyn FnMut(&str) -> Option<(String, String)>,
    trazidas: &mut Vec<String>,
    avisos: &mut Vec<String>,
) -> String {
    let mut fora = String::new();
    let mut resto = linha;
    while let Some(pos) = resto.find("![[") {
        let (antes, daqui) = resto.split_at(pos);
        let Some(fim) = daqui.find("]]") else {
            // Marcador sem fechamento: texto comum.
            fora.push_str(antes);
            fora.push_str(daqui);
            return fora;
        };
        let bruto = &daqui[3..fim];
        fora.push_str(antes);
        fora.push_str(&trazer(bruto, visitadas, profundidade, buscar, trazidas, avisos));
        resto = &daqui[fim + 2..];
    }
    fora.push_str(resto);
    fora
}

/// Traz um alvo, ou o aviso do que impediu.
fn trazer(
    bruto: &str,
    visitadas: &mut Vec<String>,
    profundidade: usize,
    buscar: &mut dyn FnMut(&str) -> Option<(String, String)>,
    trazidas: &mut Vec<String>,
    avisos: &mut Vec<String>,
) -> String {
    let alvo = analisar_alvo(bruto);
    if alvo.titulo.is_empty() {
        return String::new();
    }
    if profundidade >= PROFUNDIDADE_MAXIMA {
        return nota(avisos, format!("transclusão de {bruto} passou de {PROFUNDIDADE_MAXIMA} níveis"));
    }
    let Some((path, corpo)) = buscar(&alvo.titulo) else {
        return nota(avisos, format!("a página {} não existe", alvo.titulo));
    };
    if visitadas.contains(&path) {
        return nota(avisos, format!("transclusão de {} em ciclo", alvo.titulo));
    }
    let recorte = match recortar(&corpo, &alvo) {
        Ok(t) => t,
        Err(motivo) => return nota(avisos, motivo),
    };
    if !trazidas.contains(&path) {
        trazidas.push(path.clone());
    }
    visitadas.push(path.clone());
    let dentro = resolver_em(&recorte, visitadas, profundidade + 1, buscar, trazidas, avisos);
    // Fora do caminho, a página volta a poder ser transcluída: duas
    // seções irmãs podem citar a mesma referência sem virar "ciclo".
    visitadas.retain(|v| v != &path);
    // O conteúdo trazido vem rotulado: quem lê (pessoa ou modelo)
    // precisa saber que aquilo é de outro arquivo, e de qual.
    format!("[de {}]\n{dentro}", rotulo(&alvo))
}

fn rotulo(alvo: &Alvo) -> String {
    match (&alvo.secao, &alvo.bloco) {
        (_, Some(b)) => format!("{} ^{b}", alvo.titulo),
        (Some(s), _) => format!("{} › {s}", alvo.titulo),
        _ => alvo.titulo.clone(),
    }
}

fn nota(avisos: &mut Vec<String>, motivo: String) -> String {
    if !avisos.contains(&motivo) {
        avisos.push(motivo.clone());
    }
    format!("[transclusão não resolvida: {motivo}]")
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Um vault de mentira: título → (path, corpo).
    fn vault<'a>(paginas: &'a [(&'a str, &'a str, &'a str)]) -> impl FnMut(&str) -> Option<(String, String)> + 'a {
        move |titulo: &str| {
            paginas
                .iter()
                .find(|(t, _, _)| t.eq_ignore_ascii_case(titulo))
                .map(|(_, path, corpo)| (path.to_string(), corpo.to_string()))
        }
    }

    #[test]
    fn separa_titulo_secao_e_bloco() {
        assert_eq!(analisar_alvo("Spec"), Alvo { titulo: "Spec".into(), secao: None, bloco: None });
        assert_eq!(
            analisar_alvo(" Spec # Regras "),
            Alvo { titulo: "Spec".into(), secao: Some("Regras".into()), bloco: None }
        );
        assert_eq!(
            analisar_alvo("Spec^abc123"),
            Alvo { titulo: "Spec".into(), secao: None, bloco: Some("abc123".into()) }
        );
    }

    #[test]
    fn traz_a_pagina_inteira_a_secao_e_o_bloco() {
        let mut v = vault(&[(
            "Padrões",
            "pages/padroes.md",
            "# Padrões\n\n## Regras\n\nsempre em minúsculas ^r1\n\n## Exemplos\n\nalfa\n",
        )]);
        let r = resolver("antes\n\n![[Padrões#Regras]]\n\ndepois\n", "pages/a.md", &mut v);
        assert!(r.texto.contains("sempre em minúsculas"), "{}", r.texto);
        assert!(!r.texto.contains("Exemplos"), "a seção pedida é só a dela:\n{}", r.texto);
        assert!(r.texto.contains("[de Padrões › Regras]"), "{}", r.texto);
        assert_eq!(r.trazidas, ["pages/padroes.md"]);
        assert!(r.avisos.is_empty());
        // Página inteira e bloco.
        let r = resolver("![[Padrões]]", "pages/a.md", &mut v);
        assert!(r.texto.contains("Exemplos") && r.texto.contains("Regras"));
        let r = resolver("![[Padrões^r1]]", "pages/a.md", &mut v);
        assert!(r.texto.contains("sempre em minúsculas") && !r.texto.contains("Exemplos"), "{}", r.texto);
    }

    #[test]
    fn o_que_nao_deu_pra_trazer_vira_linha_visivel() {
        let mut v = vault(&[("Padrões", "pages/padroes.md", "# Padrões\n\n## Regras\n\nx\n")]);
        let r = resolver("![[Sumida]]\n![[Padrões#Não existe]]\n", "pages/a.md", &mut v);
        assert!(r.texto.contains("[transclusão não resolvida: a página Sumida não existe]"), "{}", r.texto);
        assert!(r.texto.contains("não tem a seção Não existe"), "{}", r.texto);
        assert_eq!(r.avisos.len(), 2);
        assert!(r.trazidas.is_empty());
    }

    #[test]
    fn ciclo_para_no_segundo_encontro() {
        let mut v = vault(&[
            ("A", "pages/a.md", "corpo de A\n\n![[B]]\n"),
            ("B", "pages/b.md", "corpo de B\n\n![[A]]\n"),
        ]);
        let r = resolver("![[B]]", "pages/a.md", &mut v);
        assert!(r.texto.contains("corpo de B"), "{}", r.texto);
        assert!(r.texto.contains("em ciclo"), "{}", r.texto);
        assert_eq!(r.trazidas, ["pages/b.md"]);
        // E a página que transclui a si mesma nem entra uma vez.
        let r = resolver("![[A]]", "pages/a.md", &mut v);
        assert!(r.texto.contains("em ciclo") && !r.texto.contains("corpo de A"), "{}", r.texto);
    }

    #[test]
    fn a_profundidade_tem_teto() {
        let mut v = vault(&[
            ("N1", "pages/n1.md", "um\n![[N2]]\n"),
            ("N2", "pages/n2.md", "dois\n![[N3]]\n"),
            ("N3", "pages/n3.md", "três\n![[N4]]\n"),
            ("N4", "pages/n4.md", "quatro\n"),
        ]);
        let r = resolver("![[N1]]", "pages/raiz.md", &mut v);
        assert!(r.texto.contains("um") && r.texto.contains("dois") && r.texto.contains("três"), "{}", r.texto);
        assert!(!r.texto.contains("quatro"), "o quarto nível não entra:\n{}", r.texto);
        assert!(r.avisos.iter().any(|a| a.contains("níveis")), "{:?}", r.avisos);
    }

    #[test]
    fn dentro_de_bloco_de_codigo_e_texto() {
        let mut v = vault(&[("Padrões", "pages/padroes.md", "conteúdo\n")]);
        let fonte = "```\n![[Padrões]]\n```\n![[Padrões]]\n";
        let r = resolver(fonte, "pages/a.md", &mut v);
        assert!(r.texto.contains("```\n![[Padrões]]\n```"), "dentro da cerca fica como está:\n{}", r.texto);
        assert_eq!(r.trazidas.len(), 1, "só a de fora foi trazida");
    }

    #[test]
    fn a_mesma_pagina_em_dois_lugares_entra_nos_dois() {
        // Não é ciclo: são irmãos, não aninhados.
        let mut v = vault(&[("Comum", "pages/comum.md", "trecho comum\n")]);
        let r = resolver("![[Comum]]\n\ne também\n\n![[Comum]]\n", "pages/a.md", &mut v);
        assert_eq!(r.texto.matches("trecho comum").count(), 2, "{}", r.texto);
        assert_eq!(r.trazidas, ["pages/comum.md"], "na lista, sem repetir");
        assert!(r.avisos.is_empty(), "{:?}", r.avisos);
    }

    #[test]
    fn texto_sem_transclusao_volta_igual() {
        let mut v = vault(&[]);
        let fonte = "# Título\n\nnada aqui [[um wikilink]] e ![imagem](x.png)\n";
        assert_eq!(resolver(fonte, "pages/a.md", &mut v).texto, fonte);
    }
}

// ---------------------------------------------------------------------
// Consulta como contexto (ciclo 418)
// ---------------------------------------------------------------------

/// Quantas páginas uma consulta traz, quando ela não diz o limite.
///
/// Uma consulta sem teto numa página-recorte é um jeito fácil de mandar
/// o vault inteiro pro modelo sem perceber. Dez é o que cabe num prompt
/// e ainda responde "o que está em rascunho?".
pub const PAGINAS_DA_CONSULTA: usize = 10;

/// Troca cada embed de consulta pelo conteúdo das páginas que ele acha
/// (ciclo 418).
///
/// É o que torna a página-recorte VIVA: `{{ type: "query" }} from:
/// pages/specs, where: status=rascunho` como contexto significa "as
/// specs em rascunho de hoje", não a lista que alguém digitou mês
/// passado.
///
/// `rodar` recebe a consulta e devolve `(path, título, corpo)` das
/// páginas achadas, na ordem — quem tem o índice é quem executa.
pub fn resolver_consultas(
    texto: &str,
    rodar: &mut dyn FnMut(&crate::query::Query) -> Vec<(String, String, String)>,
) -> Resolvido {
    let mut r = Resolvido::default();
    for seg in crate::embed::segment(texto) {
        match seg {
            crate::embed::DocSegment::Markdown(t) => r.texto.push_str(&t),
            crate::embed::DocSegment::Embed(crate::embed::EmbedData::Query(q)) => {
                let teto = q.limit.unwrap_or(PAGINAS_DA_CONSULTA);
                let achadas = rodar(&q);
                if achadas.is_empty() {
                    let motivo = format!("a consulta em {} não achou nada", q.from.clone().unwrap_or_else(|| "todo o vault".into()));
                    r.avisos.push(motivo.clone());
                    r.texto.push_str(&format!("[consulta sem resultado: {motivo}]\n"));
                    continue;
                }
                let total = achadas.len();
                for (path, titulo, corpo) in achadas.into_iter().take(teto) {
                    if !r.trazidas.contains(&path) {
                        r.trazidas.push(path.clone());
                    }
                    r.texto.push_str(&format!("[da consulta · {titulo}]\n{}\n\n", corpo.trim()));
                }
                // O que ficou de fora é dito: um recorte que silencia
                // metade do resultado engana quem confia nele.
                if total > teto {
                    let sobra = total - teto;
                    r.avisos.push(format!("a consulta achou {total}; {sobra} não entraram (limite {teto})"));
                    r.texto.push_str(&format!("[mais {sobra} página(s) fora do limite de {teto}]\n"));
                }
            }
            // Outro embed não vira contexto: kanban e calendário como
            // texto solto no prompt são ruído, e a página original
            // continua a um wikilink de distância.
            crate::embed::DocSegment::Embed(outro) => {
                r.texto.push_str(&format!("[embed {} — abra a página pra ver]\n", outro.kind().type_name()));
            }
        }
    }
    r
}

#[cfg(test)]
mod testes_de_consulta {
    use super::*;
    use crate::query::Query;

    const RECORTE: &str = "O que está em rascunho:\n\n{{ type: \"query\" }}\nfrom: pages/specs\nwhere:\n- field: status\n  value: rascunho\n{{ /query }}\n";

    fn pagina(n: &str) -> (String, String, String) {
        (format!("pages/specs/{n}.md"), n.to_string(), format!("corpo de {n}"))
    }

    #[test]
    fn a_consulta_vira_o_conteudo_das_paginas_achadas() {
        let mut vistas: Vec<Query> = Vec::new();
        let mut rodar = |q: &Query| {
            vistas.push(q.clone());
            vec![pagina("alfa"), pagina("beta")]
        };
        let r = resolver_consultas(RECORTE, &mut rodar);
        assert!(r.texto.contains("O que está em rascunho:"), "{}", r.texto);
        assert!(r.texto.contains("[da consulta · alfa]") && r.texto.contains("corpo de beta"), "{}", r.texto);
        assert!(!r.texto.contains("type: \"query\""), "o embed some, o conteúdo fica:\n{}", r.texto);
        assert_eq!(r.trazidas, ["pages/specs/alfa.md", "pages/specs/beta.md"]);
        assert!(r.avisos.is_empty());
        // A consulta chegou inteira em quem executa.
        assert_eq!(vistas.len(), 1);
        assert_eq!(vistas[0].from.as_deref(), Some("pages/specs"));
        assert_eq!(vistas[0].conditions.len(), 1, "o filtro chega junto: {:?}", vistas[0].conditions);
    }

    #[test]
    fn o_limite_corta_e_diz_quanto_ficou_de_fora() {
        let muitas: Vec<(String, String, String)> = (0..14).map(|i| pagina(&format!("p{i}"))).collect();
        let mut rodar = |_: &Query| muitas.clone();
        let r = resolver_consultas(RECORTE, &mut rodar);
        assert_eq!(r.trazidas.len(), PAGINAS_DA_CONSULTA);
        assert!(r.texto.contains("mais 4 página(s) fora do limite de 10"), "{}", r.texto);
        assert!(r.avisos.iter().any(|a| a.contains("14")), "{:?}", r.avisos);
        // O limite da própria consulta manda.
        let com_limite = RECORTE.replace("from: pages/specs", "from: pages/specs\nlimit: 2");
        let r = resolver_consultas(&com_limite, &mut rodar);
        assert_eq!(r.trazidas.len(), 2, "{}", r.texto);
    }

    #[test]
    fn consulta_vazia_aparece_como_aviso() {
        let mut rodar = |_: &Query| Vec::new();
        let r = resolver_consultas(RECORTE, &mut rodar);
        assert!(r.texto.contains("consulta sem resultado"), "{}", r.texto);
        assert_eq!(r.avisos.len(), 1);
        assert!(r.trazidas.is_empty());
    }

    #[test]
    fn outro_embed_vira_recado_curto() {
        let com_kanban = "antes\n\n{{ type: \"kanban\" }}\ncolumns: []\n{{ /kanban }}\n\ndepois\n";
        let mut rodar = |_: &Query| Vec::new();
        let r = resolver_consultas(com_kanban, &mut rodar);
        assert!(r.texto.contains("[embed kanban — abra a página pra ver]"), "{}", r.texto);
        assert!(r.texto.contains("antes") && r.texto.contains("depois"), "{}", r.texto);
    }

    #[test]
    fn texto_sem_embed_volta_igual() {
        let mut rodar = |_: &Query| Vec::new();
        let fonte = "# Só texto\n\nnada de embed aqui\n";
        assert_eq!(resolver_consultas(fonte, &mut rodar).texto, fonte);
    }
}
