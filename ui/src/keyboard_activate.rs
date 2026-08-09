//! Handler compartilhado pra cards/linhas clicáveis (kanban, calendário,
//! tabela, tags — ciclo 127) virarem operáveis por teclado, sem
//! duplicar a checagem de tecla em cada componente. Mesmo padrão já
//! validado nos nós do grafo (`graph_view.rs`, ciclo 126).

use yew::prelude::*;

/// Constrói um `onkeydown` que aciona `action` em Enter ou Espaço.
///
/// `.key() == " "` é o valor correto por spec num teclado real;
/// `.code() == "Space"` é reforço pra ferramentas de automação que
/// mandam o nome do código em vez do caractere (quirk encontrado no
/// driver MCP durante a validação do ciclo 126) — não muda nada pra um
/// usuário com teclado físico.
pub fn activate_on_enter_or_space(action: Callback<()>) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        if e.key() == "Enter" || e.key() == " " || e.code() == "Space" {
            e.prevent_default();
            action.emit(());
        }
    })
}
