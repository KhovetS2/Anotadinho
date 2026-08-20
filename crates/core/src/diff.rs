//! Diff linha a linha por LCS (ciclo 190).
//!
//! Existe pra a pessoa poder VER o que mudou no disco antes de decidir
//! entre recarregar e perder o que escreveu, ou manter o dela e
//! sobrescrever. Sem isso a escolha é às cegas.
//!
//! Mora no core (e não na UI) por dois motivos: é lógica pura, testável
//! sem WASM, e o `anotadinho-cli` vai querer o mesmo diff pra mostrar um
//! conflito no terminal.
//!
//! O algoritmo é o LCS clássico em matriz. Uma página de notas tem
//! centenas de linhas, não centenas de milhares — o custo O(n·m) é
//! irrelevante aqui, e a implementação simples é o que se pode ler e
//! confiar.

/// Uma linha do comparativo.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "tipo", rename_all = "lowercase")]
pub enum LinhaDiff {
    /// Igual nos dois lados.
    Igual {
        /// Conteúdo da linha.
        texto: String,
    },
    /// Só no lado de cá (o que você tem).
    Removida {
        /// Conteúdo da linha.
        texto: String,
    },
    /// Só no lado de lá (o que está no disco).
    Adicionada {
        /// Conteúdo da linha.
        texto: String,
    },
}

impl LinhaDiff {
    /// Texto da linha, seja qual for o lado.
    pub fn texto(&self) -> &str {
        match self {
            Self::Igual { texto } | Self::Removida { texto } | Self::Adicionada { texto } => texto,
        }
    }

    /// `true` pra linha que aparece só de um lado.
    pub fn mudou(&self) -> bool {
        !matches!(self, Self::Igual { .. })
    }
}

/// Compara dois textos linha a linha.
///
/// `a` é o lado de cá (o que a pessoa tem na tela), `b` o lado de lá (o
/// que está no disco).
pub fn diff_linhas(a: &str, b: &str) -> Vec<LinhaDiff> {
    let a: Vec<&str> = a.lines().collect();
    let b: Vec<&str> = b.lines().collect();

    // Matriz de comprimentos da maior subsequência comum.
    let (n, m) = (a.len(), b.len());
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    // Caminha a matriz montando o resultado. Remoção antes de adição
    // quando empata, pra um bloco trocado sair agrupado (todas as linhas
    // velhas, depois todas as novas) em vez de intercalado.
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            out.push(LinhaDiff::Igual { texto: a[i].to_string() });
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            out.push(LinhaDiff::Removida { texto: a[i].to_string() });
            i += 1;
        } else {
            out.push(LinhaDiff::Adicionada { texto: b[j].to_string() });
            j += 1;
        }
    }
    while i < n {
        out.push(LinhaDiff::Removida { texto: a[i].to_string() });
        i += 1;
    }
    while j < m {
        out.push(LinhaDiff::Adicionada { texto: b[j].to_string() });
        j += 1;
    }
    out
}

/// Quantas linhas mudaram de cada lado — o bastante pra um resumo do
/// tipo "3 linhas suas, 5 do disco" sem percorrer o diff de novo.
pub fn contar(diff: &[LinhaDiff]) -> (usize, usize) {
    diff.iter().fold((0, 0), |(r, a), l| match l {
        LinhaDiff::Removida { .. } => (r + 1, a),
        LinhaDiff::Adicionada { .. } => (r, a + 1),
        LinhaDiff::Igual { .. } => (r, a),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resumo(diff: &[LinhaDiff]) -> String {
        diff.iter()
            .map(|l| match l {
                LinhaDiff::Igual { texto } => format!(" {texto}"),
                LinhaDiff::Removida { texto } => format!("-{texto}"),
                LinhaDiff::Adicionada { texto } => format!("+{texto}"),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn textos_iguais_nao_tem_mudanca() {
        let d = diff_linhas("a\nb\nc\n", "a\nb\nc\n");
        assert!(!d.iter().any(LinhaDiff::mudou));
        assert_eq!(contar(&d), (0, 0));
    }

    #[test]
    fn linha_adicionada_no_meio() {
        let d = diff_linhas("a\nc\n", "a\nb\nc\n");
        assert_eq!(resumo(&d), " a\n+b\n c");
        assert_eq!(contar(&d), (0, 1));
    }

    #[test]
    fn linha_removida_no_meio() {
        let d = diff_linhas("a\nb\nc\n", "a\nc\n");
        assert_eq!(resumo(&d), " a\n-b\n c");
        assert_eq!(contar(&d), (1, 0));
    }

    #[test]
    fn linha_trocada_vira_remocao_mais_adicao() {
        let d = diff_linhas("a\nvelha\nc\n", "a\nnova\nc\n");
        assert_eq!(resumo(&d), " a\n-velha\n+nova\n c");
        assert_eq!(contar(&d), (1, 1));
    }

    #[test]
    fn bloco_trocado_sai_agrupado() {
        // Sem o desempate a favor da remoção, isto sairia intercalado
        // (-x +p -y +q), que é bem mais difícil de ler.
        let d = diff_linhas("topo\nx\ny\nfim\n", "topo\np\nq\nfim\n");
        assert_eq!(resumo(&d), " topo\n-x\n-y\n+p\n+q\n fim");
    }

    #[test]
    fn texto_vazio_de_um_lado() {
        assert_eq!(contar(&diff_linhas("", "a\nb\n")), (0, 2));
        assert_eq!(contar(&diff_linhas("a\nb\n", "")), (2, 0));
        assert!(diff_linhas("", "").is_empty());
    }

    #[test]
    fn preserva_a_ordem_do_conteudo() {
        let d = diff_linhas("1\n2\n3\n", "1\n3\n4\n");
        assert_eq!(resumo(&d), " 1\n-2\n 3\n+4");
    }
}
