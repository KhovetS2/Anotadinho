//! Os modos visuais do vim (ciclo 252).
//!
//! Três sabores, e a diferença entre eles é o que conta como "unidade":
//!
//! - **Visual** (`v`) — caractere. A seleção do navegador já sabe fazer
//!   isso; aqui é só `Selection.modify("extend", …)` no lugar de
//!   `"move"`, com os mesmos movimentos do modo normal.
//! - **Visual linha** (`V`) — o BLOCO. É o "linewise" do vim, e num
//!   editor de um contenteditable por bloco a linha é o bloco. Reusa
//!   inteira a seleção do ciclo 251.
//! - **Visual bloco** (`Ctrl+V`) — retângulo: as mesmas COLUNAS em vários
//!   blocos. Este módulo guarda âncora e foco como (bloco, coluna) e
//!   recorta a fatia de cada bloco no intervalo.
//!
//! Por que o retângulo é possível aqui sem o motor de seleção próprio
//! que a spec irmã declarou não-objetivo: ele não precisa de um REALCE
//! retangular, precisa de uma OPERAÇÃO retangular (copiar, apagar). As
//! coordenadas são offsets de caractere no bloco, que é a mesma régua
//! que a barra de formatação já usa desde o ciclo 244.

use wasm_bindgen::JsCast;

/// Canto do retângulo: em qual bloco, e em qual coluna dele.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Canto {
    pub bloco: usize,
    pub coluna: u32,
}

thread_local! {
    static ANCORA: std::cell::Cell<Option<Canto>> = const { std::cell::Cell::new(None) };
}

/// Onde o cursor está agora, em (bloco, coluna).
pub fn cursor() -> Option<Canto> {
    let sel = web_sys::window()?.get_selection().ok()??;
    let range = sel.get_range_at(0).ok()?;
    let no = range.start_container().ok()?;
    let el = no
        .dyn_ref::<web_sys::Element>()
        .cloned()
        .or_else(|| no.parent_element())?;
    let bloco_el = el
        .closest(&format!("[{}]", crate::nav_mode::ATTR_BLOCO_TEXTO))
        .ok()
        .flatten()?;
    let blocos = crate::selecao_blocos::blocos_do_documento();
    let bloco = blocos.iter().position(|b| b.is_same_node(Some(&bloco_el)))?;
    let (coluna, _) = crate::components::selection_toolbar::intervalo(&bloco_el, &range)?;
    Some(Canto { bloco, coluna })
}

/// Ancora o retângulo onde o cursor está.
pub fn ancorar() {
    ANCORA.with(|a| a.set(cursor()));
}

pub fn largar() {
    ANCORA.with(|a| a.set(None));
    crate::selecao_blocos::limpar();
}

pub fn ancora() -> Option<Canto> {
    ANCORA.with(|a| a.get())
}

/// Os limites normalizados do retângulo: (bloco inicial, bloco final,
/// coluna inicial, coluna final).
fn limites() -> Option<(usize, usize, u32, u32)> {
    let a = ancora()?;
    let f = cursor()?;
    let (b0, b1) = (a.bloco.min(f.bloco), a.bloco.max(f.bloco));
    let (c0, c1) = (a.coluna.min(f.coluna), a.coluna.max(f.coluna));
    Some((b0, b1, c0, c1))
}

/// Realça os blocos que participam do retângulo.
///
/// O realce é por BLOCO e não pinta o retângulo exato: pintar as colunas
/// exigiria desenhar o realce por conta, que é justamente o motor que a
/// spec irmã pôs como não-objetivo. O que a pessoa precisa saber é
/// quais blocos estão em jogo; a coluna ela vê pelo cursor.
pub fn realcar() {
    let Some((b0, b1, _, _)) = limites() else {
        return;
    };
    let blocos = crate::selecao_blocos::blocos_do_documento();
    for (i, bloco) in blocos.iter().enumerate() {
        if i >= b0 && i <= b1 {
            let _ = bloco
                .class_list()
                .add_1(crate::selecao_blocos::CLASSE_SELECIONADO);
        } else {
            let _ = bloco
                .class_list()
                .remove_1(crate::selecao_blocos::CLASSE_SELECIONADO);
        }
    }
}

/// A fatia de cada bloco dentro do retângulo, uma por linha.
///
/// Bloco mais curto que a coluna inicial entra como linha vazia, igual
/// ao vim: o retângulo não encolhe por causa de uma linha curta.
pub fn texto_do_retangulo() -> String {
    let Some((b0, b1, c0, c1)) = limites() else {
        return String::new();
    };
    let blocos = crate::selecao_blocos::blocos_do_documento();
    (b0..=b1)
        .filter_map(|i| blocos.get(i))
        .map(|bloco| {
            let texto: Vec<char> = bloco.text_content().unwrap_or_default().chars().collect();
            let ini = (c0 as usize).min(texto.len());
            let fim = (c1 as usize).min(texto.len());
            texto[ini..fim].iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Apaga a fatia do retângulo em cada bloco. Devolve se mexeu em algo.
pub fn apagar_retangulo() -> bool {
    let Some((b0, b1, c0, c1)) = limites() else {
        return false;
    };
    if c0 == c1 {
        return false;
    }
    let blocos = crate::selecao_blocos::blocos_do_documento();
    let mut mexeu = false;
    for i in b0..=b1 {
        let Some(bloco) = blocos.get(i) else { continue };
        let texto: Vec<char> = bloco.text_content().unwrap_or_default().chars().collect();
        let ini = (c0 as usize).min(texto.len());
        let fim = (c1 as usize).min(texto.len());
        if ini >= fim {
            continue;
        }
        let novo: String = texto[..ini].iter().chain(texto[fim..].iter()).collect();
        // `set_text_content` achata a formatação inline do bloco. É o
        // preço do recorte por coluna, e o vim faz o mesmo: retângulo
        // opera sobre caracteres, não sobre estrutura.
        bloco.set_text_content(Some(&novo));
        mexeu = true;
    }
    largar();
    mexeu
}

/// Estende a seleção de CARACTERE do modo visual comum.
pub fn estender(direcao: &str, granularidade: &str) {
    if let Some(sel) = web_sys::window()
        .and_then(|w| w.get_selection().ok())
        .flatten()
    {
        let _ = sel.modify("extend", direcao, granularidade);
    }
}

/// O texto selecionado agora (modo visual comum).
pub fn texto_selecionado() -> String {
    web_sys::window()
        .and_then(|w| w.get_selection().ok())
        .flatten()
        .map(|s| s.to_string().as_string().unwrap_or_default())
        .unwrap_or_default()
}

/// Põe o cursor no bloco `i`, na coluna `coluna` (ou no fim, se o bloco
/// for mais curto).
fn pousar(i: usize, coluna: u32) -> bool {
    let blocos = crate::selecao_blocos::blocos_do_documento();
    let Some(bloco) = blocos.get(i) else {
        return false;
    };
    let tamanho = bloco.text_content().unwrap_or_default().chars().count() as u32;
    let col = coluna.min(tamanho);
    if let Some(html) = bloco.dyn_ref::<web_sys::HtmlElement>() {
        let _ = html.focus();
    }
    crate::components::selection_toolbar::selecionar_intervalo(bloco, col, col).is_some()
}

/// `j`/`k` que ATRAVESSAM blocos, mantendo a coluna.
///
/// O problema que isto resolve é o que a spec chamou de "o vim ficou pra
/// trás": desde o ciclo 175 cada bloco é seu próprio `contenteditable`, e
/// `Selection.modify` não sai do host de edição. Na prática `j` no meio de
/// um parágrafo ia pro FIM dele e parava ali pra sempre — o modo normal
/// não conseguia percorrer a página.
///
/// A regra: tenta o movimento nativo primeiro (é ele que anda entre as
/// linhas de um bloco que tem quebra dentro); se o cursor parou na borda
/// do mesmo bloco, é porque não havia linha pra aquele lado — aí pula pro
/// bloco vizinho na mesma coluna.
pub fn mover_linha(frente: bool) -> bool {
    let Some(antes) = cursor() else { return false };
    let direcao = if frente { "forward" } else { "backward" };
    if let Some(sel) = web_sys::window()
        .and_then(|w| w.get_selection().ok())
        .flatten()
    {
        let _ = sel.modify("move", direcao, "line");
    }
    let Some(depois) = cursor() else { return false };
    if depois.bloco != antes.bloco {
        return true; // o nativo já atravessou
    }
    let blocos = crate::selecao_blocos::blocos_do_documento();
    let tamanho = blocos
        .get(depois.bloco)
        .map(|b| b.text_content().unwrap_or_default().chars().count() as u32)
        .unwrap_or(0);
    let na_borda = if frente {
        depois.coluna >= tamanho
    } else {
        depois.coluna == 0
    };
    if !na_borda {
        return true; // andou de linha DENTRO do bloco
    }
    let alvo = if frente {
        depois.bloco + 1
    } else {
        match depois.bloco.checked_sub(1) {
            Some(i) => i,
            None => return false,
        }
    };
    pousar(alvo, antes.coluna)
}

// ── Execução dos comandos do modo Normal (ciclo 254) ────────────────

use crate::vim_comandos::Movimento;

/// O bloco onde o cursor está.
pub fn bloco_atual() -> Option<web_sys::Element> {
    let sel = web_sys::window()?.get_selection().ok()??;
    let no = sel.anchor_node()?;
    let el = no
        .dyn_ref::<web_sys::Element>()
        .cloned()
        .or_else(|| no.parent_element())?;
    el.closest(&format!("[{}]", crate::nav_mode::ATTR_BLOCO_TEXTO))
        .ok()
        .flatten()
}

/// Traduz um movimento pro par (direção, granularidade) da
/// `Selection.modify`. `None` quando o movimento não é expressável
/// assim e precisa de tratamento próprio.
fn granularidade(mov: Movimento) -> Option<(&'static str, &'static str)> {
    Some(match mov {
        Movimento::Esquerda => ("backward", "character"),
        Movimento::Direita => ("forward", "character"),
        Movimento::PalavraFrente => ("forward", "word"),
        Movimento::PalavraTras => ("backward", "word"),
        Movimento::FimDaPalavra => ("forward", "word"),
        Movimento::InicioDaLinha => ("backward", "lineboundary"),
        Movimento::FimDaLinha => ("forward", "lineboundary"),
        Movimento::InicioDoDocumento => ("backward", "documentboundary"),
        Movimento::FimDoDocumento => ("forward", "documentboundary"),
        // Cima/Baixo atravessam blocos e não podem sair da
        // `Selection.modify`, que não deixa o host de edição.
        Movimento::Cima | Movimento::Baixo | Movimento::LinhaInteira => return None,
    })
}

/// Aplica um movimento `vezes` vezes. `estender` alarga a seleção em vez
/// de mover o cursor — é o que separa o modo Normal do Visual.
pub fn aplicar_movimento(mov: Movimento, vezes: u32, estender: bool) -> bool {
    let acao = if estender { "extend" } else { "move" };
    match mov {
        Movimento::Cima | Movimento::Baixo => {
            let frente = mov == Movimento::Baixo;
            let mut andou = false;
            for _ in 0..vezes.max(1) {
                andou |= mover_linha(frente);
            }
            andou
        }
        Movimento::LinhaInteira => false,
        _ => {
            let Some((dir, gran)) = granularidade(mov) else {
                return false;
            };
            let Some(sel) = web_sys::window()
                .and_then(|w| w.get_selection().ok())
                .flatten()
            else {
                return false;
            };
            // O documento inteiro é um salto só: repetir não faz sentido
            // e ainda custaria `vezes` chamadas à toa.
            let repeticoes = if matches!(
                mov,
                Movimento::InicioDoDocumento | Movimento::FimDoDocumento
            ) {
                1
            } else {
                vezes.max(1)
            };
            for _ in 0..repeticoes {
                let _ = sel.modify(acao, dir, gran);
            }
            true
        }
    }
}

/// Estende a seleção pelo movimento e devolve o texto abrangido, sem
/// apagar nada. É a metade comum de `d`, `c` e `y`.
pub fn selecionar_alcance(mov: Movimento, vezes: u32) -> Option<String> {
    if mov == Movimento::LinhaInteira {
        return None; // quem chama trata a linha inteira por bloco
    }
    aplicar_movimento(mov, vezes, true);
    Some(texto_selecionado())
}

/// Apaga a seleção atual pela API de `Range`, não por `execCommand`.
///
/// A regra do projeto (AGENTS.md) é preferir `Range` — o `execCommand`
/// fragmenta HTML de forma imprevisível no WebKitGTK. Aqui vale também
/// por um motivo mais simples: `delete_contents` faz exatamente uma
/// coisa, e dá pra saber se fez.
pub fn apagar_selecao() -> bool {
    let Some(sel) = web_sys::window()
        .and_then(|w| w.get_selection().ok())
        .flatten()
    else {
        return false;
    };
    if sel.is_collapsed() || sel.range_count() == 0 {
        return false;
    }
    let Ok(range) = sel.get_range_at(0) else {
        return false;
    };
    if range.delete_contents().is_err() {
        return false;
    }
    let _ = sel.remove_all_ranges();
    let _ = sel.add_range(&range);
    true
}

/// Junta o bloco seguinte no atual (`J`), com um espaço no meio.
pub fn juntar_linhas() -> bool {
    let Some(bloco) = bloco_atual() else {
        return false;
    };
    let Some(proximo) = bloco.next_element_sibling() else {
        return false;
    };
    let atual = bloco.text_content().unwrap_or_default();
    let seguinte = proximo.text_content().unwrap_or_default();
    let junto = format!("{} {}", atual.trim_end(), seguinte.trim_start());
    bloco.set_text_content(Some(junto.trim_end()));
    proximo.remove();
    true
}

/// Inverte a caixa do caractere sob o cursor e anda um pra frente (`~`).
pub fn trocar_caixa() -> bool {
    let Some(bloco) = bloco_atual() else {
        return false;
    };
    let Some(pos) = cursor().map(|c| c.coluna as usize) else {
        return false;
    };
    let texto: Vec<char> = bloco.text_content().unwrap_or_default().chars().collect();
    if pos >= texto.len() {
        return false;
    }
    let c = texto[pos];
    let trocado: String = if c.is_uppercase() {
        c.to_lowercase().collect()
    } else {
        c.to_uppercase().collect()
    };
    let novo: String = texto[..pos]
        .iter()
        .collect::<String>()
        .chars()
        .chain(trocado.chars())
        .chain(texto[pos + 1..].iter().copied())
        .collect();
    bloco.set_text_content(Some(&novo));
    let fim = (pos as u32 + 1).min(novo.chars().count() as u32);
    crate::components::selection_toolbar::selecionar_intervalo(&bloco, fim, fim);
    true
}

/// Substitui o caractere sob o cursor (`r`), sem sair do modo Normal.
pub fn substituir_caractere(novo: char) -> bool {
    let Some(bloco) = bloco_atual() else {
        return false;
    };
    let Some(pos) = cursor().map(|c| c.coluna as usize) else {
        return false;
    };
    let texto: Vec<char> = bloco.text_content().unwrap_or_default().chars().collect();
    if pos >= texto.len() {
        return false;
    }
    let mut saida: String = texto[..pos].iter().collect();
    saida.push(novo);
    saida.extend(texto[pos + 1..].iter());
    bloco.set_text_content(Some(&saida));
    crate::components::selection_toolbar::selecionar_intervalo(&bloco, pos as u32, pos as u32);
    true
}

/// Põe o cursor onde a inserção começa, e devolve se conseguiu.
pub fn posicionar_para_inserir(onde: crate::vim_comandos::Insercao) -> bool {
    use crate::vim_comandos::Insercao;
    match onde {
        Insercao::Antes => true,
        Insercao::Depois => aplicar_movimento(Movimento::Direita, 1, false),
        Insercao::InicioDaLinha => aplicar_movimento(Movimento::InicioDaLinha, 1, false),
        Insercao::FimDaLinha => aplicar_movimento(Movimento::FimDaLinha, 1, false),
        // As duas que criam bloco são tratadas por quem chama, porque
        // mexem na estrutura e precisam recalcular o markdown.
        Insercao::LinhaAbaixo | Insercao::LinhaAcima => true,
    }
}
