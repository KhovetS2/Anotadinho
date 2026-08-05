//! Anotadinho UI: Yew frontend que compila pra WASM.
//!
//! Entry point. Inicializa o panic hook e monta o componente raiz.

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<anotadinho_ui::App>::new().render();
}
