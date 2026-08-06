//! Sistema de diálogo próprio do Anotadinho — substitui os diálogos nativos
//! do navegador/webview (`window.prompt`/`confirm`/`alert`, expostos via
//! `gloo_dialogs`) por um modal com a identidade visual do app.
//!
//! Uso: um único `pending_dialog: UseStateHandle<Option<PendingDialog>>`
//! vive em `app.rs`; um `open_dialog: Callback<PendingDialog>` é passado
//! como prop pra baixo na árvore (mesmo padrão de `vault_path` etc — sem
//! `Context`, seguindo a convenção do projeto). Um único `DialogHost` no
//! topo da árvore renderiza o modal quando há um diálogo pendente.

use yew::prelude::*;

/// Um diálogo pendente de resposta do usuário.
#[derive(Clone, PartialEq)]
pub enum PendingDialog {
    /// Mensagem informativa, só um botão "OK".
    Alert {
        /// Texto da mensagem.
        message: String,
    },
    /// Pergunta sim/não, com callback disparado só se confirmado.
    Confirm {
        /// Texto da pergunta.
        message: String,
        /// Texto do botão de confirmação (ex: "Excluir").
        confirm_label: String,
        /// Disparado se o usuário confirmar.
        on_confirm: Callback<()>,
    },
    /// Pede um texto livre.
    Prompt {
        /// Título do modal.
        title: String,
        /// Valor inicial do campo.
        default: String,
        /// Disparado com o texto digitado (já sem espaços nas pontas, não
        /// vazio) quando o usuário confirma.
        on_submit: Callback<String>,
    },
}
