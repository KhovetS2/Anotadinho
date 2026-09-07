//! Posicionar um grafo de páginas no espaço (ciclo 300).
//!
//! O grafo da janela desenhava todos os nós num CÍRCULO
//! (`2πi/n` por índice), sem física. O próprio ciclo 120 previu o
//! limite: "reavaliar layout melhor se um vault muito grande deixar o
//! círculo ilegível".
//!
//! Foi o que aconteceu. Com 239 páginas e 162 links, todo nó está na
//! borda e **toda aresta é uma corda cruzando o meio** — o desenho vira
//! uma bola de linhas, e nada do que ele mostra é a estrutura.
//!
//! Aqui o layout é força dirigida em TRÊS dimensões: nó repele nó,
//! aresta puxa quem ela liga. O que agrupa passa a ficar junto, e o que
//! não se liga se afasta — que é a informação que o círculo escondia.
//!
//! ## Por que no núcleo
//!
//! Porque é matemática, não desenho: entra grafo, sai posição. Assim dá
//! pra testar sem navegador — que é a única forma de afirmar que dois
//! nós ligados terminam mais perto do que dois que não se ligam.
//!
//! A janela projeta e desenha; a rotação é dela.

/// Um grafo pra posicionar: quantos nós, e quem liga quem.
#[derive(Debug, Clone, Default)]
pub struct Grafo {
    /// Quantidade de nós.
    pub nos: usize,
    /// Pares de índices ligados.
    pub arestas: Vec<(usize, usize)>,
}

/// Uma posição no espaço.
pub type Ponto = [f64; 3];

/// Quantas rodadas de força. Acima disto o desenho para de mudar de
/// forma perceptível, e cada rodada custa `n²`.
const RODADAS: usize = 220;

/// Posiciona os nós por força dirigida em três dimensões.
///
/// **Determinístico**: as posições iniciais saem de um gerador semeado
/// pelo índice, não do relógio nem de `rand`. Sem isso o mesmo vault
/// desenharia diferente a cada abertura — e nenhum teste conseguiria
/// afirmar nada.
pub fn posicionar(g: &Grafo) -> Vec<Ponto> {
    if g.nos == 0 {
        return Vec::new();
    }
    let n = g.nos as f64;
    // A constante do Fruchterman-Reingold: a distância "natural" entre
    // dois nós num volume unitário.
    let k = (1.0 / n).cbrt();
    let mut pos = espalhar(g.nos);
    let mut passo: f64 = 0.1;

    for _ in 0..RODADAS {
        let mut forca = vec![[0.0f64; 3]; g.nos];

        // Repulsão: todo mundo empurra todo mundo.
        for i in 0..g.nos {
            for j in (i + 1)..g.nos {
                let d = subtrair(pos[i], pos[j]);
                let dist = norma(d).max(1e-6);
                let f = k * k / dist;
                let u = escalar(d, f / dist);
                forca[i] = somar(forca[i], u);
                forca[j] = subtrair(forca[j], u);
            }
        }
        // Atração: a aresta puxa os dois lados.
        for (a, b) in &g.arestas {
            if *a >= g.nos || *b >= g.nos || a == b {
                continue;
            }
            let d = subtrair(pos[*a], pos[*b]);
            let dist = norma(d).max(1e-6);
            let f = dist * dist / k;
            let u = escalar(d, f / dist);
            forca[*a] = subtrair(forca[*a], u);
            forca[*b] = somar(forca[*b], u);
        }

        // O passo encolhe a cada rodada: no começo o grafo se
        // desembola, no fim ele assenta. Sem isso ele oscila pra sempre
        // em vez de convergir.
        for i in 0..g.nos {
            let f = norma(forca[i]);
            if f > 1e-9 {
                let limite = passo.min(f);
                pos[i] = somar(pos[i], escalar(forca[i], limite / f));
            }
        }
        passo *= 0.97;
    }
    centralizar(&mut pos);
    pos
}

/// Posições iniciais espalhadas, sem aleatório de verdade.
///
/// Espiral de Fibonacci sobre uma esfera: dá pontos bem distribuídos e
/// é função do índice, então o resultado é o mesmo em toda execução.
/// Começar todo mundo no mesmo lugar faria a repulsão explodir na
/// primeira rodada.
fn espalhar(n: usize) -> Vec<Ponto> {
    let dourado = std::f64::consts::PI * (3.0 - 5.0f64.sqrt());
    (0..n)
        .map(|i| {
            let t = if n > 1 {
                i as f64 / (n - 1) as f64
            } else {
                0.5
            };
            let y = 1.0 - 2.0 * t;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = dourado * i as f64;
            [r * a.cos(), y, r * a.sin()]
        })
        .collect()
}

/// Põe o centro de massa na origem.
fn centralizar(pos: &mut [Ponto]) {
    if pos.is_empty() {
        return;
    }
    let n = pos.len() as f64;
    let mut c = [0.0; 3];
    for p in pos.iter() {
        c = somar(c, *p);
    }
    c = escalar(c, 1.0 / n);
    for p in pos.iter_mut() {
        *p = subtrair(*p, c);
    }
}

fn somar(a: Ponto, b: Ponto) -> Ponto {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn subtrair(a: Ponto, b: Ponto) -> Ponto {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn escalar(a: Ponto, f: f64) -> Ponto {
    [a[0] * f, a[1] * f, a[2] * f]
}
fn norma(a: Ponto) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// A distância entre dois nós posicionados.
pub fn distancia(a: Ponto, b: Ponto) -> f64 {
    norma(subtrair(a, b))
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Duas panelinhas ligadas por dentro e nada entre elas.
    fn duas_panelinhas() -> Grafo {
        Grafo {
            nos: 6,
            arestas: vec![(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)],
        }
    }

    /// Teto de ORDEM, não meta de desempenho (a regra do ciclo 259).
    ///
    /// Medido no vault real — 239 páginas, 162 links — em release:
    /// **36ms**. Em debug, 440ms. O cálculo acontece UMA vez ao abrir a
    /// página, não por quadro, então 36ms não aparece.
    ///
    /// O teto folgado existe pra pegar mudança de ordem: a repulsão é
    /// `n²` por rodada, e quem trocar isso por `n³` sem perceber vê aqui.
    ///
    /// Ignorado por padrão porque mede tempo, e tempo depende de máquina:
    ///     cargo test -p anotadinho-core --release grafo -- --ignored
    #[test]
    #[ignore = "mede tempo; roda com --ignored"]
    fn o_layout_do_vault_real_cabe_no_teto() {
        let g = Grafo {
            nos: 239,
            arestas: (0..162).map(|i| (i % 239, (i * 7 + 3) % 239)).collect(),
        };
        let t0 = std::time::Instant::now();
        let p = posicionar(&g);
        let ms = t0.elapsed().as_millis();
        println!("239 nós, 162 arestas: {ms}ms");
        assert_eq!(p.len(), 239);
        assert!(ms < 300, "o layout levou {ms}ms — a ordem mudou?");
    }

    #[test]
    fn quem_se_liga_termina_mais_perto() {
        // É a afirmação que o círculo não conseguia fazer: lá a
        // distância entre dois nós era o ângulo entre eles, e o ângulo
        // vinha da ordem alfabética. Nada a ver com ligação.
        let g = duas_panelinhas();
        let p = posicionar(&g);

        let dentro = distancia(p[0], p[1]);
        let fora = distancia(p[0], p[4]);
        assert!(
            dentro < fora,
            "ligados ({dentro:.3}) não ficaram mais perto que desligados ({fora:.3})"
        );
    }

    #[test]
    fn o_resultado_e_o_mesmo_toda_vez() {
        // Determinístico de propósito: sem isso o mesmo vault desenharia
        // diferente a cada abertura, e nenhum teste afirmaria nada.
        let g = duas_panelinhas();
        assert_eq!(posicionar(&g), posicionar(&g));
    }

    #[test]
    fn ninguem_fica_no_mesmo_lugar_que_outro() {
        // Se as posições iniciais colidissem, a repulsão dividiria por
        // zero e o grafo explodiria — ou pior, ficaria sobreposto e
        // ninguém notaria.
        let p = posicionar(&Grafo { nos: 30, arestas: vec![] });
        for i in 0..p.len() {
            for j in (i + 1)..p.len() {
                assert!(
                    distancia(p[i], p[j]) > 1e-6,
                    "os nós {i} e {j} ficaram no mesmo ponto"
                );
            }
        }
    }

    #[test]
    fn usa_as_tres_dimensoes() {
        // Um layout que colapsa num plano é um layout 2D com passos a
        // mais.
        let p = posicionar(&Grafo {
            nos: 40,
            arestas: (0..39).map(|i| (i, i + 1)).collect(),
        });
        for eixo in 0..3 {
            let min = p.iter().map(|q| q[eixo]).fold(f64::MAX, f64::min);
            let max = p.iter().map(|q| q[eixo]).fold(f64::MIN, f64::max);
            assert!(
                max - min > 0.05,
                "o eixo {eixo} ficou achatado ({:.4})",
                max - min
            );
        }
    }

    #[test]
    fn o_grafo_fica_centrado_na_origem() {
        // A janela gira em volta do centro; se o centro de massa
        // estivesse longe da origem, girar jogaria o grafo pra fora da
        // tela.
        let p = posicionar(&duas_panelinhas());
        let n = p.len() as f64;
        for eixo in 0..3 {
            let media: f64 = p.iter().map(|q| q[eixo]).sum::<f64>() / n;
            assert!(media.abs() < 1e-9, "o eixo {eixo} ficou deslocado: {media}");
        }
    }

    #[test]
    fn grafo_vazio_nao_quebra() {
        assert!(posicionar(&Grafo::default()).is_empty());
        assert_eq!(posicionar(&Grafo { nos: 1, arestas: vec![] }).len(), 1);
    }

    #[test]
    fn aresta_invalida_e_ignorada() {
        // Wikilink pra página que não existe vira índice fora da faixa;
        // isso vem do vault e não pode derrubar o desenho.
        let g = Grafo {
            nos: 3,
            arestas: vec![(0, 1), (1, 99), (2, 2)],
        };
        assert_eq!(posicionar(&g).len(), 3);
    }
}
