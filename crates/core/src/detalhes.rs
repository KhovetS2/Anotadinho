//! Os detalhes da tarefa que vieram do card do kanban (ciclo 476):
//! checklist, comentários e anexos.
//!
//! Tudo mora no CORPO da página, em seções que a pessoa leria no
//! markdown puro (RNF2 da spec `tarefa-rica-home-e-memoria`). O card do
//! kanban guarda isso num bloco YAML porque ele é um item dentro de uma
//! página; a tarefa É a página.
//!
//! - checklist: os `- [ ]` da seção **Critérios de aceite**, que a tarefa
//!   já tinha. Fazer uma lista à parte duplicaria os critérios.
//! - comentários: `- **QUANDO** — texto` na seção **Comentários**.
//! - anexos: `- [nome](assets/x)` na seção **Anexos**, relativos à raiz
//!   do vault como todo link de arquivo do app.
//!
//! As funções que escrevem devolvem o texto INTEIRO novo e mexem só no
//! trecho delas: marcar um item troca um caractere, e nada mais.

use serde::{Deserialize, Serialize};

/// A seção dos critérios, que é a checklist.
pub const SECAO_CRITERIOS: &str = "Critérios de aceite";
/// A seção dos comentários.
pub const SECAO_COMENTARIOS: &str = "Comentários";
/// A seção dos anexos.
pub const SECAO_ANEXOS: &str = "Anexos";

/// Um item da checklist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDaChecklist {
    /// O critério, sem a marca.
    pub texto: String,
    /// `[x]`.
    pub feito: bool,
}

/// Um comentário, com quando foi feito.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comentario {
    /// Como foi gravado (`AAAA-MM-DD HH:MM`).
    pub quando: String,
    /// O texto, com as linhas de continuação juntadas por `\n`.
    pub texto: String,
}

/// Um arquivo anexado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anexo {
    /// O nome de exibição.
    pub nome: String,
    /// Relativo à raiz do vault (`assets/x.pdf`).
    pub caminho: String,
}

/// Compara nomes de seção sem acento e sem caixa: `## Criterios de
/// aceite` escrito à mão também vale.
fn mesmo_nome(a: &str, b: &str) -> bool {
    let simples = |s: &str| {
        s.trim()
            .to_lowercase()
            .chars()
            .map(|c| match c {
                'á' | 'à' | 'â' | 'ã' => 'a',
                'é' | 'ê' => 'e',
                'í' => 'i',
                'ó' | 'ô' | 'õ' => 'o',
                'ú' => 'u',
                'ç' => 'c',
                outro => outro,
            })
            .collect::<String>()
    };
    simples(a) == simples(b)
}

/// O título se a linha abre uma seção de nível 2 (`## X`).
fn titulo_h2(linha: &str) -> Option<&str> {
    linha.strip_prefix("## ").map(str::trim)
}

/// As linhas da seção `nome`: `(primeira, fim)` em índices de linha, com
/// `fim` exclusivo — a próxima `## `, ou o fim do texto. Fence de código
/// não conta: um `## ` dentro de ``` não abre nem fecha seção.
fn secao(linhas: &[&str], nome: &str) -> Option<(usize, usize)> {
    let mut em_fence = false;
    let mut inicio = None;
    for (i, l) in linhas.iter().enumerate() {
        if l.trim_start().starts_with("```") {
            em_fence = !em_fence;
            continue;
        }
        if em_fence {
            continue;
        }
        if let Some(t) = titulo_h2(l) {
            if let Some(ini) = inicio {
                return Some((ini, i));
            }
            if mesmo_nome(t, nome) {
                inicio = Some(i + 1);
            }
        }
    }
    inicio.map(|ini| (ini, linhas.len()))
}

/// `Some((feito, texto, coluna_do_colchete))` se a linha é um item de checklist
/// com texto. O item vazio do esqueleto (`- [ ] `) não conta: é lugar
/// pra escrever, não critério.
fn item(linha: &str) -> Option<(bool, &str, usize)> {
    let recuo = linha.len() - linha.trim_start().len();
    let resto = &linha[recuo..];
    let resto = resto.strip_prefix("- ").or_else(|| resto.strip_prefix("* "))?;
    let marca = resto.get(..3)?;
    let feito = match marca {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    let texto = resto[3..].trim();
    if texto.is_empty() {
        return None;
    }
    // `recuo` + o `- ` de dois caracteres: o `[` está aqui, e a marca
    // logo depois dele.
    Some((feito, texto, recuo + 2))
}

fn linhas_da_secao<'a>(texto: &'a str, nome: &str) -> Vec<(usize, &'a str)> {
    let linhas: Vec<&str> = texto.lines().collect();
    let Some((ini, fim)) = secao(&linhas, nome) else { return Vec::new() };
    let mut em_fence = false;
    let mut fora = Vec::new();
    for (i, l) in linhas.iter().enumerate().take(fim).skip(ini) {
        if l.trim_start().starts_with("```") {
            em_fence = !em_fence;
            continue;
        }
        if !em_fence {
            fora.push((i, *l));
        }
    }
    fora
}

/// A checklist: os itens com texto de **Critérios de aceite**.
pub fn checklist(texto: &str) -> Vec<ItemDaChecklist> {
    linhas_da_secao(texto, SECAO_CRITERIOS)
        .into_iter()
        .filter_map(|(_, l)| item(l).map(|(feito, t, _)| ItemDaChecklist { texto: t.to_string(), feito }))
        .collect()
}

/// Junta as linhas de volta preservando o `\n` final do original.
fn juntar(linhas: &[String], original: &str) -> String {
    let mut s = linhas.join("\n");
    if original.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Marca ou desmarca o item `indice` da checklist (a contagem de
/// [`checklist`]). Só o caractere da marca muda.
pub fn alternar_item(texto: &str, indice: usize) -> Result<String, String> {
    let alvo = linhas_da_secao(texto, SECAO_CRITERIOS)
        .into_iter()
        .filter_map(|(i, l)| item(l).map(|(feito, _, col)| (i, feito, col)))
        .nth(indice)
        .ok_or_else(|| format!("a checklist não tem o item {}", indice + 1))?;
    let mut linhas: Vec<String> = texto.lines().map(str::to_string).collect();
    let (i, feito, col) = alvo;
    let nova = if feito { ' ' } else { 'x' };
    linhas[i].replace_range(col + 1..col + 2, &nova.to_string());
    Ok(juntar(&linhas, texto))
}

/// Os comentários, na ordem da página. Linhas recuadas logo abaixo de um
/// comentário são a continuação dele.
pub fn comentarios(texto: &str) -> Vec<Comentario> {
    let mut fora: Vec<Comentario> = Vec::new();
    for (_, l) in linhas_da_secao(texto, SECAO_COMENTARIOS) {
        if let Some(resto) = l.strip_prefix("- **") {
            if let Some((quando, depois)) = resto.split_once("**") {
                let t = depois.trim_start().trim_start_matches('—').trim_start_matches('-').trim();
                fora.push(Comentario { quando: quando.trim().to_string(), texto: t.to_string() });
                continue;
            }
        }
        if l.starts_with("  ") && !l.trim().is_empty() {
            if let Some(ultimo) = fora.last_mut() {
                ultimo.texto.push('\n');
                ultimo.texto.push_str(l.trim());
            }
        }
    }
    fora
}

/// Os anexos: `- [nome](caminho)` em **Anexos**.
pub fn anexos(texto: &str) -> Vec<Anexo> {
    linhas_da_secao(texto, SECAO_ANEXOS)
        .into_iter()
        .filter_map(|(_, l)| {
            let r = l.trim_start().strip_prefix("- [")?;
            let (nome, depois) = r.split_once("](")?;
            let (caminho, _) = depois.split_once(')')?;
            // O espaço foi gravado como `%20` pra o link valer no
            // markdown; quem usa o caminho quer o arquivo de verdade.
            Some(Anexo { nome: nome.to_string(), caminho: caminho.trim().replace("%20", " ") })
        })
        .collect()
}

/// Acrescenta `linhas_novas` no fim da seção `nome` (antes da próxima
/// `## `), criando a seção no fim da página se ela não existir.
fn acrescentar_na_secao(texto: &str, nome: &str, linhas_novas: &[String]) -> String {
    let linhas: Vec<&str> = texto.lines().collect();
    let mut fora: Vec<String> = linhas.iter().map(|l| l.to_string()).collect();
    match secao(&linhas, nome) {
        Some((ini, fim)) => {
            // Depois do último conteúdo da seção, não das linhas em branco
            // que a separam da próxima: o item novo fica colado aos outros.
            let mut pos = fim;
            while pos > ini && linhas[pos - 1].trim().is_empty() {
                pos -= 1;
            }
            let mut novas: Vec<String> = Vec::new();
            if pos == ini {
                // Seção vazia: uma linha em branco depois do título.
                novas.push(String::new());
            }
            novas.extend(linhas_novas.iter().cloned());
            if pos == fim && fim < linhas.len() {
                novas.push(String::new());
            }
            fora.splice(pos..pos, novas);
            juntar(&fora, texto)
        }
        None => {
            while fora.last().is_some_and(|l| l.trim().is_empty()) {
                fora.pop();
            }
            fora.push(String::new());
            fora.push(format!("## {nome}"));
            fora.push(String::new());
            fora.extend(linhas_novas.iter().cloned());
            juntar(&fora, "\n")
        }
    }
}

/// Acrescenta um comentário. Texto de várias linhas vira continuação
/// recuada, que [`comentarios`] junta de volta.
pub fn acrescentar_comentario(texto: &str, quando: &str, comentario: &str) -> Result<String, String> {
    let comentario = comentario.trim();
    if comentario.is_empty() {
        return Err("o comentário está vazio".into());
    }
    let mut partes = comentario.lines();
    let primeira = partes.next().unwrap_or("").trim();
    let mut linhas = vec![format!("- **{}** — {primeira}", quando.trim())];
    linhas.extend(partes.map(|l| format!("  {}", l.trim())));
    Ok(acrescentar_na_secao(texto, SECAO_COMENTARIOS, &linhas))
}

/// Acrescenta um anexo já copiado pro vault.
pub fn acrescentar_anexo(texto: &str, nome: &str, caminho: &str) -> Result<String, String> {
    if caminho.trim().is_empty() {
        return Err("o anexo não tem caminho".into());
    }
    // `]` e `)` no nome quebrariam o link: o nome é de exibição, então
    // troca-se o caractere em vez de escapar.
    let nome = nome.trim().replace(']', ")").replace('[', "(");
    let caminho = caminho.trim().replace(' ', "%20");
    Ok(acrescentar_na_secao(texto, SECAO_ANEXOS, &[format!("- [{nome}]({caminho})")]))
}

/// `AAAA-MM-DD` de verdade (mês e dia no intervalo), pra o prazo ser
/// comparável como texto.
pub fn data_valida(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: std::ops::Range<usize>| s.get(r).and_then(|x| x.parse::<u32>().ok());
    matches!((num(0..4), num(5..7), num(8..10)), (Some(_), Some(m), Some(d)) if (1..=12).contains(&m) && (1..=31).contains(&d))
}

#[cfg(test)]
mod testes {
    use super::*;

    const TAREFA: &str = concat!(
        "---\ntitle: T\ntype: tarefa\n---\n",
        "## Objetivo\n\nalgo\n\n",
        "## Critérios de aceite\n\n- [ ] entra com e-mail\n- [x] sai\n- [ ] \n```\n- [ ] dentro do fence\n```\n  - [ ] recuado\n\n",
        "## Abordagem\n\n- [ ] não é critério\n",
    );

    #[test]
    fn a_checklist_e_a_secao_dos_criterios_sem_o_item_vazio_nem_o_fence() {
        let c = checklist(TAREFA);
        let textos: Vec<&str> = c.iter().map(|i| i.texto.as_str()).collect();
        assert_eq!(textos, ["entra com e-mail", "sai", "recuado"]);
        assert_eq!(c.iter().map(|i| i.feito).collect::<Vec<_>>(), [false, true, false]);
    }

    #[test]
    fn alternar_troca_so_a_marca_daquele_item() {
        let novo = alternar_item(TAREFA, 2).unwrap();
        assert!(novo.contains("  - [x] recuado"), "{novo}");
        // O resto, byte a byte, igual.
        assert_eq!(novo.replace("  - [x] recuado", "  - [ ] recuado"), TAREFA);
        let de_volta = alternar_item(&alternar_item(TAREFA, 1).unwrap(), 1).unwrap();
        assert_eq!(de_volta, TAREFA);
        assert!(alternar_item(TAREFA, 3).is_err());
    }

    #[test]
    fn criterios_escrito_sem_acento_tambem_vale() {
        let t = "## Criterios de Aceite\n\n- [ ] um\n";
        assert_eq!(checklist(t).len(), 1);
    }

    #[test]
    fn comentario_cria_a_secao_e_depois_acrescenta_nela() {
        let t1 = acrescentar_comentario(TAREFA, "2026-10-22 14:30", "olhei o login").unwrap();
        assert!(t1.ends_with("## Comentários\n\n- **2026-10-22 14:30** — olhei o login\n"), "{t1}");
        let t2 = acrescentar_comentario(&t1, "2026-10-23 09:00", "falta o\nreset de senha").unwrap();
        let c = comentarios(&t2);
        assert_eq!(c.len(), 2);
        assert_eq!(c[1], Comentario { quando: "2026-10-23 09:00".into(), texto: "falta o\nreset de senha".into() });
        assert!(acrescentar_comentario(TAREFA, "x", "  ").is_err());
        // O resto da página não mudou.
        assert!(t2.starts_with(TAREFA.trim_end()));
    }

    #[test]
    fn comentario_numa_secao_do_meio_entra_antes_da_proxima() {
        let t = "## Comentários\n\n- **a** — um\n\n## Anexos\n\n- [x](assets/x)\n";
        let novo = acrescentar_comentario(t, "b", "dois").unwrap();
        assert_eq!(novo, "## Comentários\n\n- **a** — um\n- **b** — dois\n\n## Anexos\n\n- [x](assets/x)\n");
        // Seção vazia no meio.
        let vazia = "## Comentários\n\n## Anexos\n";
        assert_eq!(acrescentar_comentario(vazia, "b", "dois").unwrap(), "## Comentários\n\n- **b** — dois\n\n## Anexos\n");
    }

    #[test]
    fn anexo_vira_link_relativo_ao_vault() {
        let t = acrescentar_anexo(TAREFA, "Planilha [v2].xlsx", "assets/planilha v2.xlsx").unwrap();
        let a = anexos(&t);
        assert_eq!(a, [Anexo { nome: "Planilha (v2).xlsx".into(), caminho: "assets/planilha v2.xlsx".into() }]);
        assert!(t.contains("](assets/planilha%20v2.xlsx)"), "o link no markdown precisa valer: {t}");
    }

    #[test]
    fn datas() {
        assert!(data_valida("2026-11-05"));
        assert!(!data_valida("2026-13-01"));
        assert!(!data_valida("5/11/2026"));
        assert!(!data_valida("2026-1-05"));
    }
}
