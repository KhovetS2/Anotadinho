//! A tabela markdown comum (`| a | b |`), a que o menu `/` → Tabela da
//! janela insere (ciclo 389). Não é o embed de tabela: é markdown puro,
//! lido e escrito aqui pra TUI desenhar e editar sem estragar o arquivo.

/// Cabeçalho e linhas de uma tabela pipe. `None` se o texto não é uma.
pub fn ler(texto: &str) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let linhas: Vec<&str> = texto.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if linhas.len() < 2 || !linhas.iter().all(|l| l.starts_with('|')) {
        return None;
    }
    let celulas = |l: &str| -> Vec<String> {
        let miolo = l.trim().trim_start_matches('|').trim_end_matches('|');
        miolo.split('|').map(|c| c.trim().to_string()).collect()
    };
    let separador = celulas(linhas[1]);
    if !separador.iter().all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':'))) {
        return None;
    }
    let cabecalho = celulas(linhas[0]);
    let n = cabecalho.len();
    let corpo = linhas[2..]
        .iter()
        .map(|l| {
            let mut c = celulas(l);
            c.resize(n, String::new());
            c
        })
        .collect();
    Some((cabecalho, corpo))
}

/// Escreve a tabela de volta, com as colunas alinhadas.
pub fn escrever(cabecalho: &[String], linhas: &[Vec<String>]) -> String {
    let n = cabecalho.len().max(1);
    let largura = |i: usize| {
        std::iter::once(cabecalho.get(i).map_or(0, |c| c.chars().count()))
            .chain(linhas.iter().map(|l| l.get(i).map_or(0, |c| c.chars().count())))
            .max()
            .unwrap_or(0)
            .max(3)
    };
    let fileira = |celulas: &[String]| -> String {
        let partes: Vec<String> = (0..n)
            .map(|i| {
                let c = celulas.get(i).map(String::as_str).unwrap_or("");
                format!("{c}{}", " ".repeat(largura(i).saturating_sub(c.chars().count())))
            })
            .collect();
        format!("| {} |", partes.join(" | "))
    };
    let mut fora = vec![fileira(cabecalho), format!("| {} |", (0..n).map(|i| "-".repeat(largura(i))).collect::<Vec<_>>().join(" | "))];
    fora.extend(linhas.iter().map(|l| fileira(l)));
    fora.join("\n")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_e_escreve_a_tabela_pipe() {
        let (c, l) = ler("| A | B | C |\n| --- | --- | --- |\n|  |  |  |").unwrap();
        assert_eq!(c, ["A", "B", "C"]);
        assert_eq!(l, [vec!["", "", ""]]);
        let (c, l) = ler("| Nome | Nota |\n|:---|---:|\n| Ana | 10 |\n| Bia |").unwrap();
        assert_eq!((c.len(), l[1].clone()), (2, vec!["Bia".to_string(), String::new()]));
        assert_eq!(escrever(&c, &l), "| Nome | Nota |\n| ---- | ---- |\n| Ana  | 10   |\n| Bia  |      |");
        assert!(ler("texto | com pipe").is_none());
        assert!(ler("| a |\n| b |").is_none());
    }
}
