//! Remapear teclas (ciclo 359), como o "Atalhos globais" e o "Modo vim"
//! das configurações da janela.
//!
//! Dois grupos:
//!
//! - **vim**: as teclas de uma letra da gramática (andar, inserir, criar,
//!   apagar, colar, desfazer). Trocar `j` por `n` faz `n` descer e `j`
//!   deixar de fazer isso — como na janela, uma tecla por ação.
//! - **globais**: comandos da barra que ganham tecla direta (nova página,
//!   alternar tema, hoje, tags, abas…).
//!
//! A tradução acontece no modo normal, antes de tudo o mais: onde se
//! digita texto (inserção, filtros, campos), a tecla é a tecla.

use std::collections::BTreeMap;

use super::modais::{AlvoDoDetalhe, Modal, Pedido};
use super::Estado;
use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};

/// As ações de vim remapeáveis: chave, rótulo e a tecla padrão.
pub const VIM: &[(&str, &str, &str)] = &[
    ("left", "Esquerda", "h"),
    ("down", "Descer", "j"),
    ("up", "Subir", "k"),
    ("right", "Direita", "l"),
    ("word_forward", "Próxima palavra", "w"),
    ("word_backward", "Palavra anterior", "b"),
    ("line_start", "Começo da linha", "0"),
    ("line_end", "Fim da linha", "$"),
    ("doc_end", "Fim do documento", "G"),
    ("insert_before", "Inserir antes", "i"),
    ("insert_after", "Inserir depois", "a"),
    ("open_below", "Criar abaixo", "o"),
    ("open_above", "Criar acima", "O"),
    ("delete_char", "Apagar", "x"),
    ("paste", "Colar", "p"),
    ("undo", "Desfazer", "u"),
];

/// Os comandos globais: a chave do comando da barra, rótulo e a tecla
/// padrão (vazia é sem tecla).
pub const GLOBAIS: &[(&str, &str, &str)] = &[
    ("paleta", "Barra de comandos", "Ctrl+k"),
    ("nova-pagina", "Nova página", ""),
    ("nova-pasta", "Nova pasta", ""),
    ("alternar-tema", "Alternar tema", ""),
    ("alternar-sidebar", "Alternar sidebar", ""),
    ("hoje", "Ir pra Hoje", ""),
    ("ver-tags", "Ver tags", ""),
    ("ver-assets", "Ver assets", ""),
    ("aba-proxima", "Próxima aba", "Ctrl+w"),
    ("aba-anterior", "Aba anterior", "Alt+h"),
    ("aba-fechar", "Fechar aba", "Alt+q"),
];

/// A tecla de uma ação, com o remapeamento.
fn tecla_de<'a>(mapa: &'a BTreeMap<String, String>, acao: &str, padrao: &'a str) -> &'a str {
    mapa.get(acao).map(String::as_str).unwrap_or(padrao)
}

/// O que a tecla faz depois do remapeamento. `Traduzida` é a tecla
/// padrão a processar; `Comando` executa um comando global; `Nada` é
/// tecla cuja ação foi pra outra.
#[derive(Debug, PartialEq)]
pub(super) enum Remapeada {
    Traduzida(String),
    Comando(&'static str),
    Nada,
}

/// Traduz a tecla pelo remapeamento das preferências.
pub(super) fn traduzir(e: &Estado, tecla: &str) -> Remapeada {
    let p = &e.preferencias;
    if p.teclas_globais.is_empty() && p.teclas_vim.is_empty() {
        return Remapeada::Traduzida(tecla.to_string());
    }
    // Globais que mudaram: a nova executa; a padrão antiga fica livre
    // pro que ela fazia antes (as de aba e a barra são tratadas pelo
    // caminho de sempre, então a antiga para de funcionar abaixo).
    for (acao, _, padrao) in GLOBAIS {
        let atual = tecla_de(&p.teclas_globais, acao, padrao);
        if !atual.is_empty() && atual == tecla && atual != *padrao {
            return Remapeada::Comando(acao);
        }
        if !padrao.is_empty() && *padrao == tecla && atual != *padrao {
            return Remapeada::Nada;
        }
    }
    if e.vim.em_curso() {
        return Remapeada::Traduzida(tecla.to_string());
    }
    for (acao, _, padrao) in VIM {
        let atual = tecla_de(&p.teclas_vim, acao, padrao);
        if atual == tecla && atual != *padrao {
            return Remapeada::Traduzida(padrao.to_string());
        }
    }
    for (acao, _, padrao) in VIM {
        let atual = tecla_de(&p.teclas_vim, acao, padrao);
        if *padrao == tecla && atual != *padrao {
            return Remapeada::Nada;
        }
    }
    Remapeada::Traduzida(tecla.to_string())
}

/// O formulário "Remapear teclas".
pub fn abrir(e: &mut Estado) {
    let p = &e.preferencias;
    let mut campos: Vec<C> = VIM
        .iter()
        .map(|(acao, rotulo, padrao)| {
            C::novo(acao, format!("Vim · {rotulo}"), Valor::Texto(tecla_de(&p.teclas_vim, acao, padrao).to_string())).com_dica(*padrao)
        })
        .collect();
    campos.extend(GLOBAIS.iter().map(|(acao, rotulo, padrao)| {
        C::novo(acao, format!("Global · {rotulo}"), Valor::Texto(tecla_de(&p.teclas_globais, acao, padrao).to_string()))
            .com_dica(if padrao.is_empty() { "sem tecla — ex.: Ctrl+n, Alt+t" } else { padrao })
    }));
    let mut form = Formulario::novo(campos);
    form.botoes.push(("salvar", "Salvar".into()));
    form.botoes.push(("padrao", "Voltar ao padrão".into()));
    e.modal = Some(Modal::Detalhe { titulo: "Remapear teclas".into(), form, alvo: AlvoDoDetalhe::Teclas });
}

/// Uma tecla escrita é válida: um caractere, ou `Ctrl+x`/`Alt+x`.
fn valida(t: &str) -> bool {
    t.chars().count() == 1
        || ["Ctrl+", "Alt+"].iter().any(|p| t.strip_prefix(p).is_some_and(|r| r.chars().count() == 1))
}

/// "Salvar": confere e grava. `false` mantém o formulário, com o problema
/// no aviso.
pub(super) fn salvar(e: &mut Estado, form: &Formulario) -> bool {
    let mut vim = BTreeMap::new();
    let mut globais = BTreeMap::new();
    let mut usadas: BTreeMap<String, String> = BTreeMap::new();
    let todas = VIM.iter().map(|x| (x, true)).chain(GLOBAIS.iter().map(|x| (x, false)));
    for ((acao, rotulo, padrao), e_vim) in todas {
        let t = form.texto(acao).trim().to_string();
        if t.is_empty() && e_vim {
            e.aviso = Some(format!("{rotulo}: toda ação do vim precisa de tecla"));
            return false;
        }
        if !t.is_empty() {
            if !valida(&t) {
                e.aviso = Some(format!("{rotulo}: \"{t}\" não é tecla (use uma letra, Ctrl+x ou Alt+x)"));
                return false;
            }
            if let Some(outra) = usadas.insert(t.clone(), rotulo.to_string()) {
                e.aviso = Some(format!("\"{t}\" está em {outra} e em {rotulo}"));
                return false;
            }
        }
        if t != *padrao {
            if e_vim { vim.insert(acao.to_string(), t) } else { globais.insert(acao.to_string(), t) };
        }
    }
    e.preferencias.teclas_vim = vim;
    e.preferencias.teclas_globais = globais;
    e.pedidos.push(Pedido::GravarPreferencias);
    e.aviso = Some("teclas gravadas".into());
    true
}

/// "Voltar ao padrão".
pub(super) fn padrao(e: &mut Estado) {
    e.preferencias.teclas_vim.clear();
    e.preferencias.teclas_globais.clear();
    e.pedidos.push(Pedido::GravarPreferencias);
    e.aviso = Some("teclas de volta ao padrão".into());
}
