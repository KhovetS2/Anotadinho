//! Navegação espacial: qual item está "pra baixo" daqui.
//!
//! Existe porque nenhuma regra estrutural acerta o que `j` significa
//! dentro de um embed (ciclo 268). Tentei duas antes desta:
//!
//! - **ordem de documento** — descer um kanban passava pelos botões de
//!   editar e apagar de cada cartão, três teclas por cartão;
//! - **entre pares do mesmo nome** — melhor pros cartões, mas fazia `j`
//!   andar entre COLUNAS, que estão lado a lado. A regra não sabia onde
//!   as coisas estão na tela, e "pra baixo" é uma pergunta sobre a tela.
//!
//! Aqui a pergunta é respondida por geometria: entre os candidatos que
//! estão na direção pedida, ganha o mais próximo, com desvio lateral
//! penalizado. É o mesmo critério da navegação espacial de controle
//! remoto de TV, e não precisa que nenhum embed declare nada.
//!
//! Zero DOM: quem mede as caixas é a UI, quem escolhe é isto — e por
//! isso a escolha tem teste.

use serde::{Deserialize, Serialize};

/// Retângulo de um item na tela.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Caixa {
    pub x: f64,
    pub y: f64,
    pub largura: f64,
    pub altura: f64,
}

impl Caixa {
    pub fn nova(x: f64, y: f64, largura: f64, altura: f64) -> Self {
        Self { x, y, largura, altura }
    }
    fn centro_x(&self) -> f64 {
        self.x + self.largura / 2.0
    }
    fn centro_y(&self) -> f64 {
        self.y + self.altura / 2.0
    }
}

/// Para onde andar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direcao {
    Cima,
    Baixo,
    Esquerda,
    Direita,
}

/// Quanto um desvio lateral custa em relação ao avanço na direção
/// pedida.
///
/// Acima de 1 porque o alinhamento importa mais que a distância: entre
/// um item logo abaixo e um item um pouco mais perto mas numa coluna
/// distante, quem usa espera o primeiro. Em 2.0 um desvio lateral de
/// 100px empata com um avanço de 200px.
const PESO_DESVIO: f64 = 2.0;

/// Folga, em pixels, pra considerar que um item ESTÁ na direção pedida.
///
/// Sem ela, itens praticamente alinhados (diferença de meio pixel por
/// arredondamento de layout) entrariam como candidatos dos dois lados.
const FOLGA: f64 = 2.0;

/// O índice do vizinho na direção pedida, se houver.
pub fn vizinho(atual: &Caixa, candidatos: &[Caixa], direcao: Direcao) -> Option<usize> {
    let (cx, cy) = (atual.centro_x(), atual.centro_y());
    candidatos
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let (dx, dy) = (c.centro_x() - cx, c.centro_y() - cy);
            // Avanço na direção pedida, e o desvio no outro eixo.
            let (avanco, desvio) = match direcao {
                Direcao::Baixo => (dy, dx.abs()),
                Direcao::Cima => (-dy, dx.abs()),
                Direcao::Direita => (dx, dy.abs()),
                Direcao::Esquerda => (-dx, dy.abs()),
            };
            if avanco <= FOLGA {
                return None; // não está pra esse lado
            }
            Some((i, avanco + desvio * PESO_DESVIO))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
}

/// O item mais próximo do canto superior esquerdo de `area`.
///
/// É por onde se ENTRA num contêiner. Ordem de documento não serve: numa
/// tabela, o primeiro item marcado é o botão "+ coluna" do cabeçalho, lá
/// na direita — entrar na tabela pousava nele em vez de na primeira
/// célula.
pub fn mais_proximo_do_inicio(area: &Caixa, candidatos: &[Caixa]) -> Option<usize> {
    candidatos
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let (dx, dy) = (c.x - area.x, c.y - area.y);
            (i, dx * dx + dy * dy)
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Um kanban: duas colunas lado a lado, cartões empilhados.
    ///
    /// 0 = coluna A, 1 = coluna B,
    /// 2,3 = cartões da A, 4 = cartão da B.
    fn kanban() -> Vec<Caixa> {
        vec![
            Caixa::nova(0.0, 0.0, 180.0, 30.0),    // 0 cabeçalho A
            Caixa::nova(200.0, 0.0, 180.0, 30.0),  // 1 cabeçalho B
            Caixa::nova(0.0, 40.0, 180.0, 50.0),   // 2 cartão A1
            Caixa::nova(0.0, 100.0, 180.0, 50.0),  // 3 cartão A2
            Caixa::nova(200.0, 40.0, 180.0, 50.0), // 4 cartão B1
        ]
    }

    #[test]
    fn no_kanban_j_desce_na_coluna_e_l_troca_de_coluna() {
        let k = kanban();
        // Do cartão A1, pra baixo é A2 — não o cabeçalho B nem o B1.
        assert_eq!(vizinho(&k[2], &k, Direcao::Baixo), Some(3));
        // Do cartão A1, pra direita é B1 — a coluna ao lado.
        assert_eq!(vizinho(&k[2], &k, Direcao::Direita), Some(4));
        // Do cabeçalho A, pra direita é o cabeçalho B.
        assert_eq!(vizinho(&k[0], &k, Direcao::Direita), Some(1));
    }

    #[test]
    fn o_alinhamento_pesa_mais_que_a_distancia() {
        // Um item logo abaixo e alinhado, contra um mais PERTO em linha
        // reta mas deslocado pro lado. Quem usa espera o alinhado.
        let atual = Caixa::nova(0.0, 0.0, 100.0, 20.0);
        let alinhado = Caixa::nova(0.0, 100.0, 100.0, 20.0);
        let torto = Caixa::nova(300.0, 40.0, 100.0, 20.0);
        assert_eq!(
            vizinho(&atual, &[alinhado, torto], Direcao::Baixo),
            Some(0),
            "escolheu o desalinhado só por estar mais perto"
        );
    }

    #[test]
    fn nao_ha_vizinho_na_borda() {
        let k = kanban();
        // Do último cartão da coluna A não há nada abaixo.
        assert_eq!(vizinho(&k[3], &k, Direcao::Baixo), None);
        // Do cabeçalho A não há nada à esquerda.
        assert_eq!(vizinho(&k[0], &k, Direcao::Esquerda), None);
    }

    #[test]
    fn o_proprio_item_nunca_e_o_vizinho() {
        let k = kanban();
        for d in [Direcao::Cima, Direcao::Baixo, Direcao::Esquerda, Direcao::Direita] {
            assert_ne!(vizinho(&k[2], &k, d), Some(2), "{d:?} devolveu a si mesmo");
        }
    }

    #[test]
    fn itens_alinhados_por_arredondamento_nao_contam_como_vizinhos() {
        // Meio pixel de diferença é ruído de layout, não uma linha
        // abaixo.
        let a = Caixa::nova(0.0, 0.0, 100.0, 20.0);
        let b = Caixa::nova(120.0, 0.5, 100.0, 20.0);
        assert_eq!(vizinho(&a, &[b], Direcao::Baixo), None);
        assert_eq!(vizinho(&a, &[b], Direcao::Direita), Some(0));
    }

    #[test]
    fn uma_lista_empilhada_anda_de_um_em_um() {
        // O caso do callout: controles empilhados.
        let itens: Vec<Caixa> = (0..4)
            .map(|i| Caixa::nova(0.0, i as f64 * 30.0, 200.0, 24.0))
            .collect();
        assert_eq!(vizinho(&itens[0], &itens, Direcao::Baixo), Some(1));
        assert_eq!(vizinho(&itens[2], &itens, Direcao::Baixo), Some(3));
        assert_eq!(vizinho(&itens[3], &itens, Direcao::Baixo), None);
        assert_eq!(vizinho(&itens[3], &itens, Direcao::Cima), Some(2));
    }

    #[test]
    fn numa_grade_desce_pra_mesma_coluna() {
        // Galeria 3×2: descer da célula do meio da linha de cima cai na
        // do meio da linha de baixo.
        let g: Vec<Caixa> = (0..6)
            .map(|i| Caixa::nova((i % 3) as f64 * 110.0, (i / 3) as f64 * 90.0, 100.0, 80.0))
            .collect();
        assert_eq!(vizinho(&g[1], &g, Direcao::Baixo), Some(4));
        assert_eq!(vizinho(&g[4], &g, Direcao::Cima), Some(1));
        assert_eq!(vizinho(&g[0], &g, Direcao::Direita), Some(1));
    }

    #[test]
    fn entra_pela_primeira_celula_e_nao_pelo_botao_do_cabecalho() {
        // O caso da tabela: em ordem de documento o primeiro item
        // marcado é o "+ coluna", no canto superior DIREITO. Quem entra
        // numa tabela espera a primeira célula.
        let area = Caixa::nova(0.0, 0.0, 600.0, 200.0);
        let mais_coluna = Caixa::nova(560.0, 4.0, 30.0, 20.0);
        let primeira_celula = Caixa::nova(0.0, 40.0, 250.0, 30.0);
        assert_eq!(
            mais_proximo_do_inicio(&area, &[mais_coluna, primeira_celula]),
            Some(1)
        );
    }

    #[test]
    fn sem_candidatos_o_inicio_tambem_devolve_nada() {
        let area = Caixa::nova(0.0, 0.0, 10.0, 10.0);
        assert_eq!(mais_proximo_do_inicio(&area, &[]), None);
    }

    #[test]
    fn sem_candidatos_devolve_nada() {
        let a = Caixa::nova(0.0, 0.0, 10.0, 10.0);
        assert_eq!(vizinho(&a, &[], Direcao::Baixo), None);
    }
}
