//! A sidebar em árvore de pastas (ciclo 299).
//!
//! Até aqui a TUI listava as páginas achatadas, e um vault com pastas
//! virava uma parede de nomes. A janela mostra a hierarquia real dos
//! diretórios sob `pages/`, com pasta que abre e fecha — e a mesma
//! ordem: **pastas primeiro, em ordem alfabética, depois as páginas**.
//!
//! O módulo é puro: monta a árvore e a achata na lista visível. Quem
//! desenha e quem lê tecla é o painel.

use std::collections::{BTreeMap, BTreeSet};

use anotadinho_ipc::PageMeta;

/// Um nó da árvore: pastas e as páginas daquele nível.
#[derive(Default, Debug)]
pub struct No {
    pastas: BTreeMap<String, No>,
    paginas: Vec<PageMeta>,
}

/// Monta a árvore a partir dos caminhos das páginas.
///
/// Só o que está sob `pages/`: `journals/` é cronológico e não tem
/// hierarquia que valha árvore — na janela ele também é lista.
pub fn arvore(paginas: &[PageMeta]) -> No {
    let mut raiz = No::default();
    for p in paginas {
        let rel = p.path.strip_prefix("pages/").unwrap_or(&p.path);
        let mut segmentos: Vec<&str> = rel.split('/').collect();
        segmentos.pop(); // o último é o arquivo
        let mut no = &mut raiz;
        for seg in segmentos {
            no = no.pastas.entry(seg.to_string()).or_default();
        }
        no.paginas.push(p.clone());
    }
    raiz
}

/// O que aparece numa linha da sidebar.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Uma pasta, com o caminho dela (`produto/specs`).
    Pasta { caminho: String, nome: String },
    /// Uma página, com o índice dela na lista original.
    Pagina { indice: usize, titulo: String },
}

/// Uma linha da sidebar.
#[derive(Debug, Clone, PartialEq)]
pub struct Linha {
    /// Profundidade — vira recuo.
    pub nivel: usize,
    /// Pasta ou página.
    pub item: Item,
}

/// Achata a árvore na lista que aparece, pulando o que está fechado.
///
/// A ordem é a da janela: pastas primeiro (alfabética, que o `BTreeMap`
/// já dá), depois as páginas. Duas telas com a mesma ordem é uma coisa
/// a menos pra pessoa reaprender.
pub fn visiveis(
    raiz: &No,
    paginas: &[PageMeta],
    fechadas: &BTreeSet<String>,
) -> Vec<Linha> {
    let mut fora = Vec::new();
    descer(raiz, paginas, fechadas, "", 0, &mut fora);
    fora
}

fn descer(
    no: &No,
    paginas: &[PageMeta],
    fechadas: &BTreeSet<String>,
    prefixo: &str,
    nivel: usize,
    fora: &mut Vec<Linha>,
) {
    for (nome, filho) in &no.pastas {
        let caminho = if prefixo.is_empty() {
            nome.clone()
        } else {
            format!("{prefixo}/{nome}")
        };
        fora.push(Linha {
            nivel,
            item: Item::Pasta {
                caminho: caminho.clone(),
                nome: nome.clone(),
            },
        });
        // Fechada: o conteúdo não entra na lista, e por isso as setas do
        // teclado o pulam sozinhas.
        if !fechadas.contains(&caminho) {
            descer(filho, paginas, fechadas, &caminho, nivel + 1, fora);
        }
    }
    for p in &no.paginas {
        if let Some(indice) = paginas.iter().position(|q| q.path == p.path) {
            fora.push(Linha {
                nivel,
                item: Item::Pagina {
                    indice,
                    titulo: p.title.clone(),
                },
            });
        }
    }
}

/// As pastas que nascem fechadas.
///
/// Todas. A janela também abre com as pastas recolhidas (ciclo 106), e
/// abrir tudo num vault grande devolve a mesma parede de nomes que a
/// árvore veio resolver.
pub fn fechadas_iniciais(raiz: &No) -> BTreeSet<String> {
    let mut fora = BTreeSet::new();
    juntar_caminhos(raiz, "", &mut fora);
    fora
}

fn juntar_caminhos(no: &No, prefixo: &str, fora: &mut BTreeSet<String>) {
    for (nome, filho) in &no.pastas {
        let caminho = if prefixo.is_empty() {
            nome.clone()
        } else {
            format!("{prefixo}/{nome}")
        };
        juntar_caminhos(filho, &caminho, fora);
        fora.insert(caminho);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn pagina(path: &str, title: &str) -> PageMeta {
        PageMeta {
            path: path.to_string(),
            title: title.to_string(),
            section: "pages".into(),
        }
    }

    fn vault() -> Vec<PageMeta> {
        vec![
            pagina("pages/solta.md", "solta"),
            pagina("pages/produto/missao.md", "missao"),
            pagina("pages/produto/painel.md", "painel"),
            pagina("pages/specs/uma.md", "uma"),
            pagina("pages/produto/fundo/nota.md", "nota"),
        ]
    }

    #[test]
    fn as_pastas_vem_antes_das_paginas_soltas() {
        // Mesma ordem da janela: pastas primeiro, alfabética, páginas
        // depois. Duas telas com a mesma ordem é uma coisa a menos pra
        // pessoa reaprender.
        let ps = vault();
        let a = arvore(&ps);
        let ls = visiveis(&a, &ps, &fechadas_iniciais(&a));
        let rotulos: Vec<String> = ls
            .iter()
            .map(|l| match &l.item {
                Item::Pasta { nome, .. } => format!("{nome}/"),
                Item::Pagina { titulo, .. } => titulo.clone(),
            })
            .collect();
        assert_eq!(rotulos, ["produto/", "specs/", "solta"]);
    }

    #[test]
    fn pasta_fechada_esconde_o_que_tem_dentro() {
        let ps = vault();
        let a = arvore(&ps);
        let fechadas = fechadas_iniciais(&a);
        assert_eq!(visiveis(&a, &ps, &fechadas).len(), 3);

        // Abrindo `produto`: entram as duas páginas dela e a subpasta,
        // que continua fechada.
        let mut abertas = fechadas.clone();
        abertas.remove("produto");
        let ls = visiveis(&a, &ps, &abertas);
        let rotulos: Vec<String> = ls
            .iter()
            .map(|l| match &l.item {
                Item::Pasta { nome, .. } => format!("{nome}/"),
                Item::Pagina { titulo, .. } => titulo.clone(),
            })
            .collect();
        assert_eq!(
            rotulos,
            ["produto/", "fundo/", "missao", "painel", "specs/", "solta"]
        );
    }

    #[test]
    fn o_recuo_conta_a_profundidade() {
        let ps = vault();
        let a = arvore(&ps);
        let mut abertas = fechadas_iniciais(&a);
        abertas.remove("produto");
        abertas.remove("produto/fundo");
        let ls = visiveis(&a, &ps, &abertas);
        let nota = ls
            .iter()
            .find(|l| matches!(&l.item, Item::Pagina { titulo, .. } if titulo == "nota"))
            .unwrap();
        assert_eq!(nota.nivel, 2, "a página de dentro da subpasta não recuou");
    }

    #[test]
    fn a_pagina_guarda_o_indice_da_lista_original() {
        // É por ele que o Enter sabe qual arquivo abrir.
        let ps = vault();
        let a = arvore(&ps);
        let mut abertas = fechadas_iniciais(&a);
        abertas.remove("produto");
        let ls = visiveis(&a, &ps, &abertas);
        let missao = ls
            .iter()
            .find(|l| matches!(&l.item, Item::Pagina { titulo, .. } if titulo == "missao"))
            .unwrap();
        let Item::Pagina { indice, .. } = missao.item else {
            panic!("não é página")
        };
        assert_eq!(ps[indice].path, "pages/produto/missao.md");
    }

    #[test]
    fn tudo_nasce_fechado() {
        // A janela também abre com as pastas recolhidas: abrir tudo num
        // vault grande devolve a parede de nomes que a árvore veio
        // resolver.
        let ps = vault();
        let a = arvore(&ps);
        let f = fechadas_iniciais(&a);
        assert!(f.contains("produto"));
        assert!(f.contains("produto/fundo"));
        assert!(f.contains("specs"));
    }
}
