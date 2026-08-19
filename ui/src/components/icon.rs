//! Ícones SVG inline — substituem emoji/glifos Unicode usados como
//! ícone de botão (ciclo 144). Emoji dependem de fonte de
//! emoji/símbolo instalada no SO: em algumas distros Linux sem
//! `noto-emoji`, ou no Windows sem fallback correto, viravam caixa
//! vazia ("tofu") em vez do ícone. `stroke="currentColor"` herda a
//! cor do texto do elemento pai (funciona em qualquer estado de
//! hover/tema sem CSS extra); `width`/`height` em `em` deixam o ícone
//! do tamanho da fonte ao redor por padrão.

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct IconProps {
    pub name: &'static str,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(Icon)]
pub fn icon(props: &IconProps) -> Html {
    let class = classes!("icon", props.class.clone());
    html! {
        <svg
            class={class}
            viewBox="0 0 24 24"
            width="1em" height="1em"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            { icon_body(props.name) }
        </svg>
    }
}

fn icon_body(name: &str) -> Html {
    match name {
        // Três controles deslizantes — alternativa comum pra "settings"
        // que não depende de curvas de engrenagem precisas.
        "settings" => html! { <>
            <line x1="4" y1="6" x2="20" y2="6"/><circle cx="14" cy="6" r="2" fill="currentColor" stroke="none"/>
            <line x1="4" y1="12" x2="20" y2="12"/><circle cx="8" cy="12" r="2" fill="currentColor" stroke="none"/>
            <line x1="4" y1="18" x2="20" y2="18"/><circle cx="16" cy="18" r="2" fill="currentColor" stroke="none"/>
        </> },
        "more-horizontal" => html! { <>
            <circle cx="5" cy="12" r="1.5" fill="currentColor" stroke="none"/>
            <circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none"/>
            <circle cx="19" cy="12" r="1.5" fill="currentColor" stroke="none"/>
        </> },
        "x" => html! { <>
            <line x1="6" y1="6" x2="18" y2="18"/>
            <line x1="18" y1="6" x2="6" y2="18"/>
        </> },
        "edit" => html! {
            <g transform="rotate(45 12 12)">
                <rect x="10.5" y="3" width="3" height="14" rx="1"/>
                <polygon points="10.5,17 13.5,17 12,21" fill="currentColor" stroke="none"/>
            </g>
        },
        "check" => html! {
            <polyline points="5,13 10,18 19,7"/>
        },
        "square" => html! {
            <rect x="4" y="4" width="16" height="16" rx="2"/>
        },
        "check-square" => html! { <>
            <rect x="4" y="4" width="16" height="16" rx="2"/>
            <polyline points="8,12.5 11,15.5 16,9"/>
        </> },
        "home" => html! { <>
            <path d="M4 11 12 4 20 11"/>
            <path d="M6 10v9a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-9"/>
            <path d="M10 20v-6h4v6"/>
        </> },
        "folder" => html! {
            <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/>
        },
        "calendar" => html! { <>
            <rect x="3" y="5" width="18" height="16" rx="2"/>
            <line x1="3" y1="10" x2="21" y2="10"/>
            <line x1="8" y1="3" x2="8" y2="7"/>
            <line x1="16" y1="3" x2="16" y2="7"/>
        </> },
        "search" => html! { <>
            <circle cx="10" cy="10" r="6"/>
            <line x1="15" y1="15" x2="20" y2="20"/>
        </> },
        "file-text" => html! { <>
            <path d="M7 3h7l5 5v13a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"/>
            <polyline points="14,3 14,8 19,8"/>
            <line x1="9" y1="13" x2="15" y2="13"/>
            <line x1="9" y1="17" x2="15" y2="17"/>
        </> },
        "network" => html! { <>
            <circle cx="12" cy="5" r="2.2"/>
            <circle cx="5" cy="19" r="2.2"/>
            <circle cx="19" cy="19" r="2.2"/>
            <line x1="12" y1="7.2" x2="6.3" y2="16.9"/>
            <line x1="12" y1="7.2" x2="17.7" y2="16.9"/>
            <line x1="7.2" y1="19" x2="16.8" y2="19"/>
        </> },
        "link" => html! {
            <g transform="rotate(45 12 12)">
                <rect x="3" y="9" width="8" height="6" rx="3"/>
                <rect x="13" y="9" width="8" height="6" rx="3"/>
                <line x1="9" y1="12" x2="15" y2="12"/>
            </g>
        },
        "clock" => html! { <>
            <circle cx="12" cy="12" r="9"/>
            <polyline points="12,7 12,12 16,14"/>
        </> },
        "download" => html! { <>
            <line x1="12" y1="3" x2="12" y2="15"/>
            <polyline points="7,10 12,15 17,10"/>
            <line x1="4" y1="19" x2="20" y2="19"/>
        </> },
        "git-branch" => html! { <>
            <circle cx="6" cy="4" r="2"/>
            <circle cx="6" cy="20" r="2"/>
            <circle cx="18" cy="10" r="2"/>
            <line x1="6" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="10" x2="18" y2="10"/>
        </> },
        "zap" => html! {
            <polygon points="13,2 4,14 11,14 10,22 20,9 13,9"/>
        },
        "message-circle" => html! {
            <path d="M4 5h16a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H9l-4 4v-4H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Z"/>
        },
        "paperclip" => html! {
            <path d="M12 3v11a3 3 0 0 1-6 0V6a2 2 0 0 1 4 0v9"/>
        },
        "external-link" => html! { <>
            <path d="M14 4h6v6"/>
            <line x1="20" y1="4" x2="10" y2="14"/>
            <path d="M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"/>
        </> },
        "sun" => html! { <>
            <circle cx="12" cy="12" r="4"/>
            <line x1="12" y1="2" x2="12" y2="5"/>
            <line x1="12" y1="19" x2="12" y2="22"/>
            <line x1="2" y1="12" x2="5" y2="12"/>
            <line x1="19" y1="12" x2="22" y2="12"/>
            <line x1="4.9" y1="4.9" x2="7" y2="7"/>
            <line x1="17" y1="17" x2="19.1" y2="19.1"/>
            <line x1="4.9" y1="19.1" x2="7" y2="17"/>
            <line x1="17" y1="7" x2="19.1" y2="4.9"/>
        </> },
        "moon" => html! {
            <path d="M15 3a9 9 0 1 0 6 15 8 8 0 0 1-6-15Z"/>
        },
        "chevron-left" => html! {
            <polyline points="15,4 7,12 15,20"/>
        },
        "chevron-right" => html! {
            <polyline points="9,4 17,12 9,20"/>
        },
        // Ícones dos itens do menu `/` (ciclo 148) — o menu passou a
        // mostrar um ícone por item, incluindo os tipos de embed
        // gerados a partir de `EmbedKind::all()`.
        "heading" => html! { <>
            <line x1="6" y1="4" x2="6" y2="20"/>
            <line x1="18" y1="4" x2="18" y2="20"/>
            <line x1="6" y1="12" x2="18" y2="12"/>
        </> },
        "list" => html! { <>
            <circle cx="5" cy="7" r="1.3" fill="currentColor" stroke="none"/>
            <circle cx="5" cy="12" r="1.3" fill="currentColor" stroke="none"/>
            <circle cx="5" cy="17" r="1.3" fill="currentColor" stroke="none"/>
            <line x1="10" y1="7" x2="20" y2="7"/>
            <line x1="10" y1="12" x2="20" y2="12"/>
            <line x1="10" y1="17" x2="20" y2="17"/>
        </> },
        // Duas aspas de bloco, desenhadas como barras verticais grossas
        // com um "rabo" — reconhecível sem precisar de curva precisa.
        "quote" => html! { <>
            <path d="M7 6 L7 13 L4 13 L4 8 Q4 6 6 6 Z" fill="currentColor" stroke="none"/>
            <path d="M16 6 L16 13 L13 13 L13 8 Q13 6 15 6 Z" fill="currentColor" stroke="none"/>
            <line x1="4" y1="18" x2="20" y2="18"/>
        </> },
        "code" => html! { <>
            <polyline points="9,7 4,12 9,17"/>
            <polyline points="15,7 20,12 15,17"/>
        </> },
        "table" => html! { <>
            <rect x="3" y="4" width="18" height="16" rx="2"/>
            <line x1="3" y1="10" x2="21" y2="10"/>
            <line x1="3" y1="15" x2="21" y2="15"/>
            <line x1="10" y1="4" x2="10" y2="20"/>
        </> },
        "minus" => html! {
            <line x1="4" y1="12" x2="20" y2="12"/>
        },
        "image" => html! { <>
            <rect x="3" y="4" width="18" height="16" rx="2"/>
            <circle cx="8.5" cy="9.5" r="1.8"/>
            <polyline points="4,18 10,12 14,16 17,13 20,16"/>
        </> },
        "columns" => html! { <>
            <rect x="3" y="4" width="5" height="16" rx="1"/>
            <rect x="9.5" y="4" width="5" height="16" rx="1"/>
            <rect x="16" y="4" width="5" height="16" rx="1"/>
        </> },
        _ => html! {},
    }
}
