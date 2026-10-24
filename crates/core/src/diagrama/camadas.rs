//! A arrumação em camadas de um grafo (fluxograma e estados).
//!
//! Era do canvas do construtor (`ui/.../canvas.rs::completar`, ciclo 490) e
//! desceu pro núcleo no 497: o desenho em caracteres da TUI precisa da
//! mesma arrumação, e duas cópias da regra divergiriam na primeira
//! correção feita numa só.

use std::collections::VecDeque;

/// A camada de cada nó: busca em largura a partir de quem não tem
/// entrada, e depois de quem sobrou (ciclos, soltos). Uma ligação que
/// volta não puxa o nó de cima pra baixo: a camada é a da PRIMEIRA vez que
/// a busca chega nele. Laço (`a → a`) não conta como entrada.
pub fn niveis(n: usize, arestas: &[(usize, usize)]) -> Vec<usize> {
    let arestas: Vec<(usize, usize)> = arestas.iter().copied().filter(|(a, b)| a != b && *a < n && *b < n).collect();
    let tem_entrada: Vec<bool> = (0..n).map(|i| arestas.iter().any(|(_, b)| *b == i)).collect();
    let ordem = (0..n).filter(|i| !tem_entrada[*i]).chain((0..n).filter(|i| tem_entrada[*i]));
    let mut nivel: Vec<Option<usize>> = vec![None; n];
    for raiz in ordem {
        if nivel[raiz].is_some() {
            continue;
        }
        nivel[raiz] = Some(0);
        let mut fila = VecDeque::from([raiz]);
        while let Some(u) = fila.pop_front() {
            let nu = nivel[u].unwrap_or(0);
            for &(a, b) in &arestas {
                if a == u && nivel[b].is_none() {
                    nivel[b] = Some(nu + 1);
                    fila.push_back(b);
                }
            }
        }
    }
    nivel.into_iter().map(|n| n.unwrap_or(0)).collect()
}

/// Os nós de cada camada, numa ordem que cruza menos: cada nó vai pra
/// perto da média da posição de quem aponta pra ele na camada de cima (uma
/// passada só, de cima pra baixo). Quem não tem ninguém em cima fica na
/// ordem em que apareceu.
pub fn camadas(n: usize, arestas: &[(usize, usize)], niveis: &[usize]) -> Vec<Vec<usize>> {
    let total = niveis.iter().copied().max().map(|m| m + 1).unwrap_or(0);
    let mut fora: Vec<Vec<usize>> = vec![Vec::new(); total];
    for i in 0..n {
        fora[niveis[i]].push(i);
    }
    for k in 1..total {
        let (acima, resto) = fora.split_at_mut(k);
        let acima = &acima[k - 1];
        let posicao = |i: usize| acima.iter().position(|x| *x == i);
        let chave = |i: usize| -> Option<f64> {
            let ps: Vec<f64> = arestas
                .iter()
                .filter(|(a, b)| *b == i && niveis[*a] + 1 == k)
                .filter_map(|(a, _)| posicao(*a).map(|p| p as f64))
                .collect();
            (!ps.is_empty()).then(|| ps.iter().sum::<f64>() / ps.len() as f64)
        };
        let camada = &mut resto[0];
        let original: Vec<usize> = camada.clone();
        // Ordem estável: empate e "sem ninguém em cima" mantêm a ordem do
        // código, que é a que a pessoa escreveu.
        camada.sort_by(|a, b| {
            let (ka, kb) = (chave(*a), chave(*b));
            match (ka, kb) {
                (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
                _ => {
                    let ia = original.iter().position(|v| v == a);
                    let ib = original.iter().position(|v| v == b);
                    ia.cmp(&ib)
                }
            }
        });
    }
    fora
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camadas_seguem_as_ligacoes_e_a_volta_nao_puxa() {
        // a → b → c, b → d, c → a (volta).
        let n = niveis(4, &[(0, 1), (1, 2), (1, 3), (2, 0)]);
        assert_eq!(n, [0, 1, 2, 2]);
    }

    #[test]
    fn soltos_e_lacos_ficam_na_primeira() {
        assert_eq!(niveis(3, &[(0, 0), (1, 2)]), [0, 0, 1]);
        assert_eq!(niveis(0, &[]), Vec::<usize>::new());
    }

    #[test]
    fn so_ciclo_comeca_pelo_primeiro() {
        assert_eq!(niveis(3, &[(0, 1), (1, 2), (2, 0)]), [0, 1, 2]);
    }

    #[test]
    fn baricentro_desembaraça_a_camada() {
        // a e b em cima; embaixo, x vem de b e y de a. Na ordem do código
        // (x, y) as ligações se cruzam; pela média, y vai pra esquerda.
        let arestas = [(0, 3), (1, 2)];
        let nv = niveis(4, &arestas);
        assert_eq!(camadas(4, &arestas, &nv), vec![vec![0, 1], vec![3, 2]]);
    }
}
