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

/// Um TRECHO do diff (ciclo 404): linhas mudadas vizinhas, com o
/// contexto igual em volta ficando de fora.
///
/// É a unidade de aprovação parcial: revisar proposta grande tudo-ou-nada
/// era o que fazia aceitar mudança que ninguém leu, ou recusar a proposta
/// inteira por causa de uma linha.
#[derive(Debug, Clone, PartialEq)]
pub struct Trecho {
    /// Onde ele começa na lista do diff.
    pub inicio: usize,
    /// Onde ele termina (exclusivo).
    pub fim: usize,
    /// Quantas linhas saem.
    pub removidas: usize,
    /// Quantas entram.
    pub adicionadas: usize,
}

/// Os trechos mudados de um diff, na ordem.
pub fn trechos(diff: &[LinhaDiff]) -> Vec<Trecho> {
    let mut fora: Vec<Trecho> = Vec::new();
    let mut atual: Option<Trecho> = None;
    for (i, l) in diff.iter().enumerate() {
        if l.mudou() {
            let t = atual.get_or_insert(Trecho { inicio: i, fim: i, removidas: 0, adicionadas: 0 });
            t.fim = i + 1;
            match l {
                LinhaDiff::Removida { .. } => t.removidas += 1,
                LinhaDiff::Adicionada { .. } => t.adicionadas += 1,
                LinhaDiff::Igual { .. } => {}
            }
        } else if let Some(t) = atual.take() {
            fora.push(t);
        }
    }
    if let Some(t) = atual {
        fora.push(t);
    }
    fora
}

/// Monta o texto aplicando SÓ os trechos escolhidos (ciclo 404): o que
/// ficou de fora volta como estava.
pub fn aplicar_trechos(diff: &[LinhaDiff], trechos: &[Trecho], escolhidos: &[bool]) -> String {
    let aceito = |i: usize| -> bool {
        match trechos.iter().position(|t| i >= t.inicio && i < t.fim) {
            Some(k) => escolhidos.get(k).copied().unwrap_or(true),
            // Linha igual: entra sempre.
            None => true,
        }
    };
    let mut fora: Vec<&str> = Vec::new();
    for (i, l) in diff.iter().enumerate() {
        match l {
            LinhaDiff::Igual { texto } => fora.push(texto),
            LinhaDiff::Adicionada { texto } if aceito(i) => fora.push(texto),
            LinhaDiff::Removida { texto } if !aceito(i) => fora.push(texto),
            _ => {}
        }
    }
    let mut texto = fora.join("\n");
    texto.push('\n');
    texto
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

    #[test]
    fn trechos_agrupam_mudancas_vizinhas_e_aplicam_so_o_escolhido() {
        let atual = "a\nb\nc\nd\n";
        let proposto = "a\nB1\nB2\nc\nD\n";
        let d = diff_linhas(atual, proposto);
        let ts = trechos(&d);
        assert_eq!(ts.len(), 2, "{ts:?}");
        assert_eq!((ts[0].removidas, ts[0].adicionadas), (1, 2));
        // Só o primeiro trecho: `d` fica como estava.
        assert_eq!(aplicar_trechos(&d, &ts, &[true, false]), "a\nB1\nB2\nc\nd\n");
        // Só o segundo.
        assert_eq!(aplicar_trechos(&d, &ts, &[false, true]), "a\nb\nc\nD\n");
        // Os dois é a proposta inteira; nenhum é o atual.
        assert_eq!(aplicar_trechos(&d, &ts, &[true, true]), proposto);
        assert_eq!(aplicar_trechos(&d, &ts, &[false, false]), atual);
    }
}

// ---------------------------------------------------------------------
// Realce palavra a palavra (ciclo 410)
// ---------------------------------------------------------------------

/// Um pedaço de linha no realce fino.
#[derive(Debug, Clone, PartialEq)]
pub struct Pedaco {
    pub texto: String,
    /// `true` quando este pedaço é o que difere da outra linha.
    pub mudou: bool,
}

/// Quão parecidas as linhas precisam ser pra valer o realce fino. Abaixo
/// disso são linhas DIFERENTES, não uma linha editada: pintar palavra
/// por palavra aí só produz confete.
const SEMELHANCA_MINIMA: f32 = 0.34;

/// Quebra a linha em palavras e separadores, mantendo tudo — juntar os
/// pedaços de volta devolve a linha original.
fn palavras(linha: &str) -> Vec<String> {
    let mut fora: Vec<String> = Vec::new();
    let mut atual = String::new();
    let mut em_palavra = false;
    for c in linha.chars() {
        let palavra = c.is_alphanumeric() || c == '_';
        if !atual.is_empty() && palavra != em_palavra {
            fora.push(std::mem::take(&mut atual));
        }
        em_palavra = palavra;
        atual.push(c);
    }
    if !atual.is_empty() {
        fora.push(atual);
    }
    fora
}

/// Compara duas linhas palavra a palavra.
///
/// Devolve os pedaços de cada lado (o de cá e o de lá) com a marca de
/// mudou, ou `None` quando as linhas são parecidas demais (iguais) ou
/// diferentes demais pra o realce ajudar.
pub fn diff_palavras(a: &str, b: &str) -> Option<(Vec<Pedaco>, Vec<Pedaco>)> {
    if a == b {
        return None;
    }
    let (pa, pb) = (palavras(a), palavras(b));
    let (n, m) = (pa.len(), pb.len());
    if n == 0 || m == 0 {
        return None;
    }
    // LCS nas palavras, o mesmo método das linhas.
    let mut tabela = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            tabela[i][j] = if pa[i] == pb[j] {
                tabela[i + 1][j + 1] + 1
            } else {
                tabela[i + 1][j].max(tabela[i][j + 1])
            };
        }
    }
    // Só conta como semelhança o que não é espaço em branco: duas linhas
    // que só compartilham a indentação não são a mesma linha editada.
    let comuns = {
        let (mut i, mut j, mut pesa) = (0, 0, 0usize);
        while i < n && j < m {
            if pa[i] == pb[j] {
                if !pa[i].trim().is_empty() {
                    pesa += pa[i].chars().count();
                }
                i += 1;
                j += 1;
            } else if tabela[i + 1][j] >= tabela[i][j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
        pesa
    };
    let maior = a.trim().chars().count().max(b.trim().chars().count()).max(1);
    if (comuns as f32) / (maior as f32) < SEMELHANCA_MINIMA {
        return None;
    }

    let (mut lado_a, mut lado_b): (Vec<Pedaco>, Vec<Pedaco>) = (Vec::new(), Vec::new());
    let junta = |lado: &mut Vec<Pedaco>, texto: &str, mudou: bool| {
        match lado.last_mut() {
            Some(p) if p.mudou == mudou => p.texto.push_str(texto),
            _ => lado.push(Pedaco { texto: texto.to_string(), mudou }),
        };
    };
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && pa[i] == pb[j] {
            junta(&mut lado_a, &pa[i], false);
            junta(&mut lado_b, &pb[j], false);
            i += 1;
            j += 1;
        } else if j >= m || (i < n && tabela[i + 1][j] >= tabela[i][j + 1]) {
            junta(&mut lado_a, &pa[i], true);
            i += 1;
        } else {
            junta(&mut lado_b, &pb[j], true);
            j += 1;
        }
    }
    Some((lado_a, lado_b))
}

/// Os pares de linhas de um trecho (ciclo 410): a k-ésima removida com a
/// k-ésima adicionada. É a leitura natural de "esta linha virou aquela",
/// e é o que permite o realce fino.
pub fn pares_do_trecho(diff: &[LinhaDiff], trecho: &Trecho) -> Vec<(usize, usize)> {
    let saem: Vec<usize> = (trecho.inicio..trecho.fim)
        .filter(|&i| matches!(diff.get(i), Some(LinhaDiff::Removida { .. })))
        .collect();
    let entram: Vec<usize> = (trecho.inicio..trecho.fim)
        .filter(|&i| matches!(diff.get(i), Some(LinhaDiff::Adicionada { .. })))
        .collect();
    saem.into_iter().zip(entram).collect()
}

#[cfg(test)]
mod testes_de_palavras {
    use super::*;

    #[test]
    fn os_pedacos_remontam_a_linha_e_marcam_so_o_que_mudou() {
        let (a, b) = diff_palavras("o prazo é sexta", "o prazo é segunda").expect("parecidas");
        assert_eq!(a.iter().map(|p| p.texto.clone()).collect::<String>(), "o prazo é sexta");
        assert_eq!(b.iter().map(|p| p.texto.clone()).collect::<String>(), "o prazo é segunda");
        let mudou = |lado: &[Pedaco]| lado.iter().filter(|p| p.mudou).map(|p| p.texto.clone()).collect::<Vec<_>>();
        assert_eq!(mudou(&a), ["sexta"]);
        assert_eq!(mudou(&b), ["segunda"]);
    }

    #[test]
    fn linha_igual_ou_diferente_demais_nao_ganha_realce() {
        assert_eq!(diff_palavras("igual", "igual"), None);
        assert_eq!(diff_palavras("", "outra"), None);
        // Nada em comum: pintar palavra por palavra aqui atrapalha.
        assert_eq!(diff_palavras("- [ ] revisar o PR", "## Conclusões da semana"), None);
        // Só a indentação em comum também não conta.
        assert_eq!(diff_palavras("    alfa beta", "    gama delta"), None);
    }

    #[test]
    fn os_pares_do_trecho_casam_removida_com_adicionada() {
        let diff = diff_linhas("a\nb\nc\n", "a\nB\nC\nD\n");
        let t = &trechos(&diff)[0];
        let pares = pares_do_trecho(&diff, t);
        assert_eq!(pares.len(), 2, "duas saem, três entram: sobra uma sem par");
        for (i, j) in pares {
            assert!(matches!(diff[i], LinhaDiff::Removida { .. }));
            assert!(matches!(diff[j], LinhaDiff::Adicionada { .. }));
        }
    }
}

