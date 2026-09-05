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

/// O índice do bloco ATÔMICO que está com o foco, se for esse o caso.
///
/// Devolve `None` quando o foco está num bloco de texto — ali quem manda
/// é o `cursor()`, que sabe a coluna. Isto existe só pro caso em que não
/// há caret nenhum pra consultar.
pub fn em_bloco_atomico() -> bool {
    indice_do_bloco_focado().is_some()
}

fn indice_do_bloco_focado() -> Option<usize> {
    let doc = web_sys::window()?.document()?;
    let ativo = doc.active_element()?;
    let bloco = ativo
        .closest(&format!("[{}]", crate::nav_mode::ATTR_BLOCO_TEXTO))
        .ok()
        .flatten()?;
    if !crate::selecao_blocos::e_atomico(&bloco) {
        return None;
    }
    crate::selecao_blocos::blocos_do_documento()
        .iter()
        .position(|b| b.is_same_node(Some(&bloco)))
}

/// Põe o cursor no bloco `i`, na coluna `coluna` (ou no fim, se o bloco
/// for mais curto).
fn pousar(i: usize, coluna: u32) -> bool {
    let blocos = crate::selecao_blocos::blocos_do_documento();
    let Some(bloco) = blocos.get(i) else {
        return false;
    };
    // Embed não comporta caret (RF3 da spec): pousar nele é dar FOCO e
    // realce, não pôr cursor. Sem isto, `selecionar_intervalo` falharia
    // por não achar nó de texto e o `j` pararia no bloco anterior —
    // exatamente o "pula por cima do embed" que este ciclo corrige.
    if crate::selecao_blocos::e_atomico(bloco) {
        crate::nav_mode::focus_item(bloco);
        return true;
    }
    // Sair de um bloco atômico tem que APAGAR o realce dele. `focus_item`
    // limpa os antigos, mas ele só é chamado ao pousar num atômico —
    // então, sem isto, o embed ficava aceso depois de o cursor já ter
    // ido embora. A sondagem mostrou `marcados: 1` com o foco já num
    // parágrafo.
    crate::nav_mode::limpar_item_ativo();
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
    // Saindo de um bloco ATÔMICO não há caret pra consultar, e
    // `cursor()` devolve `None`. Sem este ramo o `j` ENTRAVA no embed e
    // não saía mais — troca pior que o defeito original, porque antes
    // pelo menos dava pra passar por ele.
    //
    // Foi o que a sondagem manual pegou depois do ciclo 263: o cenário
    // apertava `j` UMA vez e via o pouso; três vezes seguidas mostram
    // que ele não avança.
    if let Some(i) = indice_do_bloco_focado() {
        let alvo = if frente {
            i + 1
        } else {
            match i.checked_sub(1) {
                Some(a) => a,
                None => return false,
            }
        };
        return pousar(alvo, 0);
    }
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
    // `e` sai do texto, não do `Selection.modify`.
    //
    // Aqui estava o defeito e a tentativa errada de conserto. `w` e `e`
    // mapeavam os DOIS pra `("forward", "word")` do navegador — são o
    // mesmo movimento na origem, então nenhum ajuste de alcance por cima
    // podia separá-los. Somar um caractere no fim produzia `w`+1, que é
    // pior que o erro original.
    //
    // A granularidade `word` do WebKit para no INÍCIO da palavra
    // seguinte (comportamento de `w`); `e` precisa parar no ÚLTIMO
    // caractere da palavra atual, e a definição de palavra do vim
    // (`iskeyword` contra pontuação contra branco) não é a do navegador.
    // Calcular sobre o texto do bloco resolve os dois de uma vez.
    // Os TRÊS movimentos de palavra saem do texto, não do navegador.
    //
    // Comecei corrigindo só o `e`, e o cenário do harness mostrou que o
    // `w` estava errado também — para o outro lado: a granularidade
    // `word` do WebKit parava no FIM da palavra, então `dw` deixava o
    // espaço pra trás, quando no vim ele vai junto. Deixar `e` certo e
    // `w` errado seria pior que os dois errados, porque aí eles
    // discordariam sobre o que é uma palavra.
    if let Some(alvo) = alvo_de_palavra(mov, vezes) {
        return alvo;
    }
    aplicar_movimento(mov, vezes, true);
    // Movimento INCLUSIVO leva a posição final junto (`motion.txt`,
    // seção `*inclusive*`). O `Selection.modify` para sempre ANTES do
    // caractere final, que é o comportamento exclusivo.
    if mov.alcance() == crate::vim_comandos::Alcance::Inclusivo {
        if let Some(sel) = web_sys::window()
            .and_then(|w| w.get_selection().ok())
            .flatten()
        {
            let _ = sel.modify("extend", "forward", "character");
        }
    }
    Some(texto_selecionado())
}

/// Classe de um caractere, na divisão que o vim usa pra decidir onde uma
/// palavra termina (`:help word`): letras/dígitos/`_` são uma coisa,
/// pontuação é outra, e branco separa as duas.
#[derive(PartialEq, Eq, Clone, Copy)]
enum Classe {
    Branco,
    Palavra,
    Pontuacao,
}

fn classe(c: char) -> Classe {
    if c.is_whitespace() {
        Classe::Branco
    } else if c.is_alphanumeric() || c == '_' {
        Classe::Palavra
    } else {
        Classe::Pontuacao
    }
}

/// Coluna do último caractere da palavra alcançada por `e`, a partir de
/// `col`, repetido `vezes` vezes.
///
/// Função pura sobre os caracteres do bloco — é o que permite testar a
/// regra sem DOM. Devolve `None` quando não há para onde ir (já no fim).
fn fim_da_palavra(chars: &[char], col: usize, vezes: u32) -> Option<usize> {
    let mut i = col;
    for _ in 0..vezes.max(1) {
        // Um passo à frente antes de procurar: parado no fim de uma
        // palavra, `e` vai pro fim da SEGUINTE, não fica onde está.
        i += 1;
        while i < chars.len() && classe(chars[i]) == Classe::Branco {
            i += 1;
        }
        if i >= chars.len() {
            return None;
        }
        let alvo = classe(chars[i]);
        while i + 1 < chars.len() && classe(chars[i + 1]) == alvo {
            i += 1;
        }
    }
    Some(i)
}

/// Coluna do início da próxima palavra (`w`), a partir de `col`.
///
/// Exclusivo: a operação vai ATÉ essa coluna sem incluí-la, que é o que
/// faz `dw` levar o branco depois da palavra e parar antes da seguinte.
///
/// Sem próxima palavra no bloco, devolve o fim do texto — é o caso de
/// `dw` na última palavra da linha, que no vim apaga até o fim e não
/// atravessa pra linha seguinte.
fn inicio_da_proxima_palavra(chars: &[char], col: usize, vezes: u32) -> Option<usize> {
    let mut i = col;
    for _ in 0..vezes.max(1) {
        if i >= chars.len() {
            return None;
        }
        // Sai da corrida de caracteres da mesma classe em que está...
        let atual = classe(chars[i]);
        if atual != Classe::Branco {
            while i < chars.len() && classe(chars[i]) == atual {
                i += 1;
            }
        }
        // ...e depois pula o branco até a próxima palavra.
        while i < chars.len() && classe(chars[i]) == Classe::Branco {
            i += 1;
        }
    }
    Some(i)
}

/// Coluna do início da palavra anterior (`b`), a partir de `col`.
///
/// Exclusivo, e simétrico do `w`: recua o branco e depois a corrida de
/// caracteres da mesma classe, parando no primeiro deles.
fn inicio_da_palavra_anterior(chars: &[char], col: usize, vezes: u32) -> Option<usize> {
    let mut i = col;
    for _ in 0..vezes.max(1) {
        if i == 0 {
            return None;
        }
        i -= 1;
        while i > 0 && classe(chars[i]) == Classe::Branco {
            i -= 1;
        }
        if classe(chars[i]) == Classe::Branco {
            return Some(0);
        }
        let atual = classe(chars[i]);
        while i > 0 && classe(chars[i - 1]) == atual {
            i -= 1;
        }
    }
    Some(i)
}

/// Seleciona do cursor até o alvo de um movimento de palavra.
///
/// `None` quando `mov` não é de palavra — quem chama segue pro caminho
/// do `Selection.modify`.
fn alvo_de_palavra(mov: Movimento, vezes: u32) -> Option<Option<String>> {
    let (para_frente, calcular): (bool, fn(&[char], usize, u32) -> Option<usize>) = match mov {
        Movimento::FimDaPalavra => (true, fim_da_palavra),
        Movimento::PalavraFrente => (true, inicio_da_proxima_palavra),
        Movimento::PalavraTras => (false, inicio_da_palavra_anterior),
        _ => return None,
    };
    Some((|| {
        let c = cursor()?;
        let bloco = crate::selecao_blocos::blocos_do_documento()
            .into_iter()
            .nth(c.bloco)?;
        let chars: Vec<char> = bloco.text_content().unwrap_or_default().chars().collect();
        let alvo = calcular(&chars, c.coluna as usize, vezes)?;
        // `e` é o único INCLUSIVO dos três (`motion.txt`): leva o
        // caractere final junto, e o intervalo é aberto no fim.
        let fim = if mov == Movimento::FimDaPalavra { alvo + 1 } else { alvo };
        let (ini, fim) = if para_frente {
            (c.coluna, fim as u32)
        } else {
            (fim as u32, c.coluna)
        };
        crate::components::selection_toolbar::selecionar_intervalo(&bloco, ini, fim)?;
        Some(texto_selecionado())
    })())
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

/// O caractere sob o cursor é um espaço em branco?
///
/// Existe pro caso especial de `cw` (ciclo 260): o `normal.c` do Neovim
/// só mapeia `cw` pra `ce` quando o cursor NÃO está sobre espaço ou
/// tabulação — `if (n != NUL && !ascii_iswhite(n))`. Fora do texto, ou
/// no fim da linha, a resposta honesta é "não sei", e `false` mantém o
/// comportamento de `cw` como `ce`, que é o caso comum.
pub fn cursor_sobre_espaco() -> bool {
    let Some(c) = cursor() else { return false };
    let Some(bloco) = crate::selecao_blocos::blocos_do_documento()
        .into_iter()
        .nth(c.bloco)
    else {
        return false;
    };
    bloco
        .text_content()
        .unwrap_or_default()
        .chars()
        .nth(c.coluna as usize)
        .is_some_and(|ch| ch.is_whitespace())
}

#[cfg(test)]
mod testes_palavra {
    use super::*;

    fn cs(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn e_para_no_ultimo_caractere_da_palavra() {
        // "alfa um dois", cursor em 0 -> o `a` final de "alfa" é o
        // índice 3. É esta a diferença pra `w`, que iria pro 5.
        assert_eq!(fim_da_palavra(&cs("alfa um dois"), 0, 1), Some(3));
    }

    #[test]
    fn e_no_fim_de_uma_palavra_vai_pra_seguinte() {
        // Documentado no `normal.c`: "When standing on the end of a word
        // 'ce' will change until the end of the next word".
        assert_eq!(fim_da_palavra(&cs("alfa um dois"), 3, 1), Some(6));
    }

    #[test]
    fn a_contagem_repete() {
        assert_eq!(fim_da_palavra(&cs("alfa um dois"), 0, 2), Some(6));
        assert_eq!(fim_da_palavra(&cs("alfa um dois"), 0, 3), Some(11));
    }

    #[test]
    fn pontuacao_e_uma_palavra_separada() {
        // Regra do `:help word`: pontuação forma palavra por si. Em
        // "foo.bar", `e` do começo para no `o` do "foo" (índice 2), não
        // no fim do token inteiro.
        assert_eq!(fim_da_palavra(&cs("foo.bar"), 0, 1), Some(2));
        assert_eq!(fim_da_palavra(&cs("foo.bar"), 2, 1), Some(3));
        assert_eq!(fim_da_palavra(&cs("foo.bar"), 3, 1), Some(6));
    }

    #[test]
    fn branco_no_meio_e_pulado() {
        assert_eq!(fim_da_palavra(&cs("a    bb"), 0, 1), Some(6));
    }

    #[test]
    fn sem_para_onde_ir_devolve_nada() {
        assert_eq!(fim_da_palavra(&cs("alfa"), 3, 1), None);
        assert_eq!(fim_da_palavra(&cs("alfa   "), 3, 1), None);
        assert_eq!(fim_da_palavra(&cs(""), 0, 1), None);
    }

    #[test]
    fn sublinhado_faz_parte_da_palavra() {
        // `iskeyword` inclui `_` por padrão, então `meu_nome` é uma
        // palavra só.
        assert_eq!(fim_da_palavra(&cs("meu_nome x"), 0, 1), Some(7));
    }
}

#[cfg(test)]
mod testes_w_e_b {
    use super::*;

    fn cs(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn w_para_no_inicio_da_proxima_palavra() {
        // É a diferença que o harness pegou: o `w` do navegador parava
        // no FIM de "beta" (índice 4), então `dw` deixava o espaço. O
        // `w` do vim vai pro `t` de "tres" (índice 5), e o espaço sai
        // junto.
        assert_eq!(inicio_da_proxima_palavra(&cs("beta tres quatro"), 0, 1), Some(5));
        assert_eq!(inicio_da_proxima_palavra(&cs("beta tres quatro"), 5, 1), Some(10));
    }

    #[test]
    fn w_e_e_nao_param_no_mesmo_lugar() {
        // A guarda contra a regressão de origem: os dois mapeavam pro
        // mesmo `("forward", "word")` do navegador e eram indistinguíveis.
        let t = cs("beta tres quatro");
        assert_ne!(
            inicio_da_proxima_palavra(&t, 0, 1),
            fim_da_palavra(&t, 0, 1),
            "`w` e `e` voltaram a ser o mesmo movimento"
        );
    }

    #[test]
    fn w_na_ultima_palavra_vai_ate_o_fim_da_linha() {
        // `dw` na última palavra apaga até o fim e não atravessa pra
        // linha seguinte.
        assert_eq!(inicio_da_proxima_palavra(&cs("beta tres"), 5, 1), Some(9));
    }

    #[test]
    fn w_com_contagem() {
        assert_eq!(inicio_da_proxima_palavra(&cs("um dois tres quatro"), 0, 2), Some(8));
    }

    #[test]
    fn w_trata_pontuacao_como_palavra() {
        assert_eq!(inicio_da_proxima_palavra(&cs("foo.bar"), 0, 1), Some(3));
        assert_eq!(inicio_da_proxima_palavra(&cs("foo.bar"), 3, 1), Some(4));
    }

    #[test]
    fn b_volta_pro_inicio_da_palavra() {
        // Do meio de "quatro" volta pro começo dela; do começo, pra
        // palavra anterior.
        assert_eq!(inicio_da_palavra_anterior(&cs("beta tres quatro"), 12, 1), Some(10));
        assert_eq!(inicio_da_palavra_anterior(&cs("beta tres quatro"), 10, 1), Some(5));
    }

    #[test]
    fn b_com_contagem_e_no_comeco() {
        assert_eq!(inicio_da_palavra_anterior(&cs("beta tres quatro"), 10, 2), Some(0));
        assert_eq!(inicio_da_palavra_anterior(&cs("beta"), 0, 1), None);
    }

    #[test]
    fn b_pula_branco_antes_da_palavra() {
        assert_eq!(inicio_da_palavra_anterior(&cs("um    dois"), 6, 1), Some(0));
    }
}
