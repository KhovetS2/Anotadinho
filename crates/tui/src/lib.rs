//! O Anotadinho num terminal (ciclo 286).
//!
//! Só leitura, por enquanto: abre o vault, lista as páginas, desenha uma
//! delas e anda pelos blocos. A edição já existe no núcleo (ciclo 285) e
//! entra num ciclo próprio.
//!
//! ## O que é daqui e o que é do núcleo
//!
//! Daqui: o laço de eventos, a janela que rola, o desenho em células, e
//! a tradução de tecla do crossterm.
//!
//! Do núcleo: a árvore (`analise`), para onde o cursor vai
//! (`navegacao::mover`), o que cada unidade é (`unidade::Politica`) e o
//! conteúdo dos embeds (ciclo 283). **Nenhuma regra de navegação foi
//! reescrita aqui** — se fosse preciso, seria sinal de que o passo 4 não
//! tinha terminado.

pub mod app;
pub mod componentes;
pub mod sidebar;
pub mod tela;
pub mod tema;
