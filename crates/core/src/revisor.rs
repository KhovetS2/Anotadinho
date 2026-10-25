//! O revisor automático de tarefas (ciclo 522).
//!
//! A pessoa é a PM, o revisor é o tech lead e o executor é o júnior: o
//! revisor lê o que o executor fez — a tarefa, a verificação e o diff — e
//! responde **aprovada** ou **ajustes**. Ele não edita nada: quem conserta
//! é o executor, na rodada seguinte, lendo os ajustes no comentário.
//!
//! Este módulo é só texto: a pergunta, a leitura do veredito e o estado no
//! frontmatter. Quem roda o laço é o `ipc`.
//!
//! O laço inteiro acontece com a tarefa em `executando`: o `revisao_ia`
//! diz em que ponto ele está. Assim o sistema nunca move a tarefa pra
//! `executar` (o gatilho é da pessoa: a regra "só humano avança etapa"
//! continua sem exceção), e a vaga do projeto fica com a tarefa entre as
//! rodadas — outra tarefa não entra no branch no meio da revisão.

use crate::markdown::MarkdownCodec;
use crate::projeto::Tarefa;

/// A marca da linha que o app lê pra decidir.
pub const MARCA_DE_VEREDITO: &str = "VEREDITO:";

/// O título da seção com o que mudar.
pub const SECAO_AJUSTES: &str = "## Ajustes";

/// O teto do diff no prompt. Um `package-lock.json` novo não pode empurrar
/// a tarefa pra fora da janela de contexto do revisor; o resto ele lê no
/// repositório, onde roda.
pub const TETO_DO_DIFF: usize = 60_000;

/// O `revisao_ia:` do frontmatter.
pub mod estado {
    /// A verificação passou e a revisão espera a fila.
    pub const PENDENTE: &str = "pendente";
    /// O revisor está rodando.
    pub const RODANDO: &str = "rodando";
    /// O revisor aprovou: a pessoa decide.
    pub const APROVADA: &str = "aprovada";
    /// O revisor devolveu: espera a fila dar a vez ao executor.
    pub const AJUSTES: &str = "ajustes";
    /// O executor está refazendo com os ajustes.
    pub const REFAZENDO: &str = "refazendo";
    /// Pediu ajuste no teto de rodadas: a pessoa decide.
    pub const ESGOTOU: &str = "esgotou";
    /// O revisor não rodou ou não disse nada legível.
    pub const FALHOU: &str = "falhou";
}

/// O que o revisor decidiu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Veredito {
    /// Pode ir pra pessoa.
    Aprovada,
    /// Volta pro executor com isto.
    Ajustes(String),
    /// A resposta não tem a linha, ou ela é ilegível.
    SemVeredito,
}

/// Lê o veredito: a ÚLTIMA linha com `VEREDITO:`, tolerando a marcação
/// que um modelo põe em volta (`**`, crase, `-`), como o `RESULTADO:`.
pub fn ler_veredito(resposta: &str) -> Veredito {
    for linha in resposta.lines().rev() {
        let limpa: String = linha.chars().filter(|c| !matches!(c, '*' | '`' | '_')).collect();
        let limpa = limpa.trim().trim_start_matches(['-', '>', ' ']).trim();
        // ASCII: preserva o comprimento em bytes, e o índice corta `limpa`.
        let maiusc = limpa.to_ascii_uppercase();
        let Some(pos) = maiusc.find(MARCA_DE_VEREDITO) else { continue };
        let valor = limpa[pos + MARCA_DE_VEREDITO.len()..].trim().to_lowercase();
        let valor = valor.trim_end_matches(['.', '!']);
        return match valor {
            "aprovada" | "aprovado" | "aprovar" => Veredito::Aprovada,
            "ajustes" | "ajuste" | "ajustar" | "reprovada" | "reprovado" => Veredito::Ajustes(ajustes(resposta)),
            _ => Veredito::SemVeredito,
        };
    }
    Veredito::SemVeredito
}

/// O texto da seção `## Ajustes`, até a próxima seção ou a linha do
/// veredito. Sem a seção, a resposta inteira sem o veredito: um ajuste
/// mal formatado ainda é um ajuste, e jogar fora o que o revisor disse
/// mandaria o executor de volta sem saber o quê.
fn ajustes(resposta: &str) -> String {
    let linhas: Vec<&str> = resposta.lines().collect();
    let e_veredito = |l: &str| l.to_ascii_uppercase().contains(MARCA_DE_VEREDITO);
    let titulo = |l: &str| l.trim().trim_start_matches('#').trim().eq_ignore_ascii_case("ajustes") && l.trim().starts_with('#');
    let trecho: Vec<&str> = match linhas.iter().position(|l| titulo(l)) {
        Some(ini) => linhas[ini + 1..]
            .iter()
            .take_while(|l| !l.trim_start().starts_with("## ") && !e_veredito(l))
            .copied()
            .collect(),
        None => linhas.iter().filter(|l| !e_veredito(l)).copied().collect(),
    };
    let texto = trecho.join("\n").trim().to_string();
    if texto.is_empty() {
        "o revisor pediu ajustes sem dizer quais — leia a conversa da revisão".to_string()
    } else {
        texto
    }
}

/// O que vai na pergunta do revisor além da tarefa (que vai anexada).
#[derive(Debug, Clone, Default)]
pub struct Material<'a> {
    /// Esta é a revisão número tal (1 = primeira).
    pub rodada: u32,
    /// O teto de revisões.
    pub teto: u32,
    /// O comando de verificação do projeto, se há um.
    pub verificacao: Option<&'a str>,
    /// Os arquivos mudados, uma linha cada (`M  src/x.rs`).
    pub arquivos: Vec<String>,
    /// O diff desde a base da tarefa.
    pub diff: &'a str,
    /// Por que não deu pra saber o que mudou, se for o caso.
    pub aviso: Option<&'a str>,
}

/// O diff no teto, cortado numa quebra de linha e dizendo que cortou.
pub fn diff_no_teto(diff: &str) -> String {
    if diff.len() <= TETO_DO_DIFF {
        return diff.to_string();
    }
    let mut fim = TETO_DO_DIFF;
    while !diff.is_char_boundary(fim) {
        fim -= 1;
    }
    let fim = diff[..fim].rfind('\n').unwrap_or(fim);
    format!(
        "{}\n\n[diff cortado em {} de {} caracteres: leia o resto direto nos arquivos do repositório]",
        &diff[..fim],
        fim,
        diff.len()
    )
}

/// A pergunta que manda REVISAR uma tarefa.
///
/// Trava o papel — ler e julgar, nunca editar — e o formato da resposta,
/// que é o que o app lê. Os ajustes têm que ser concretos porque quem vai
/// segui-los pode ser um modelo bem menor que o revisor.
pub fn pergunta_de_revisar(tarefa: &Tarefa, m: &Material) -> String {
    let mut p = vec![
        format!(
            "Revise a execução da tarefa \"{}\" ({}), anexada. Esta é a revisão {} de no máximo {}.",
            tarefa.titulo,
            tarefa.caminho,
            m.rodada.max(1),
            m.teto.max(1)
        ),
        String::new(),
        "Você é o revisor: LEIA e JULGUE, não edite nada. Quem conserta é o executor, na rodada seguinte, \
         seguindo o que você escrever. Você está no diretório do repositório: abra os arquivos que precisar."
            .to_string(),
        "Julgue contra os critérios de aceite, o fora de escopo da tarefa e os combinados da página do projeto. \
         Não peça melhoria que a tarefa não pede: o escopo é dela."
            .to_string(),
    ];
    if let Some(cmd) = m.verificacao {
        p.push(format!(
            "A verificação do projeto (`{cmd}`) já passou: a seção Verificação da tarefa tem a saída. Procure o que ela NÃO cobre."
        ));
    }
    p.push(String::new());
    p.push("## O que mudou".to_string());
    p.push(String::new());
    if let Some(a) = m.aviso {
        p.push(format!("Aviso: {a}"));
    }
    if m.arquivos.is_empty() {
        p.push("(nenhum arquivo mudou desde a base da tarefa)".to_string());
    } else {
        p.extend(m.arquivos.iter().cloned());
    }
    if !m.diff.trim().is_empty() {
        p.push(String::new());
        p.push("```diff".to_string());
        p.push(diff_no_teto(m.diff).trim_end().to_string());
        p.push("```".to_string());
    }
    p.extend([
        String::new(),
        "## Como responder".to_string(),
        String::new(),
        "1. Um parágrafo curto: o que foi feito e se cumpre cada critério.".to_string(),
        format!(
            "2. Se precisa de ajuste, uma seção `{SECAO_AJUSTES}` com uma lista: cada item diz ONDE (arquivo, função) \
             e O QUE mudar, concreto o bastante pra um modelo menor seguir sem adivinhar."
        ),
        // A base de problemas (ciclo 523): só o que generaliza, senão a
        // base vira o diário das tarefas e ninguém mais a lê.
        format!(
            "3. Opcional: se o executor cometeu um erro que tende a se REPETIR em outras tarefas deste projeto, \
             uma seção `{}` com uma lista `- **título curto**: o que aconteceu — como evitar`. Só o que generaliza; \
             nada específico desta tarefa, e nada que a base de problemas anexada já tenha.",
            crate::problemas::SECAO
        ),
        String::new(),
        format!(
            "A ÚLTIMA linha da resposta tem que ser `{MARCA_DE_VEREDITO} aprovada` ou `{MARCA_DE_VEREDITO} ajustes`. \
             Aprove quando os critérios estão cumpridos, mesmo que você faria diferente."
        ),
    ]);
    p.join("\n")
}

/// O texto da tarefa com o estado da revisão (`revisao_ia`) e a `rodada`.
/// `None` tira o campo: é o que zera o laço quando a pessoa manda de novo.
pub fn com_estado(texto: &str, estado: Option<&str>, rodada: Option<u32>) -> Result<String, String> {
    let (mut fm, _) = MarkdownCodec::split_frontmatter(texto).map_err(|e| format!("frontmatter ilegível: {e}"))?;
    match estado {
        Some(e) => {
            fm.extra.insert("revisao_ia".into(), serde_yaml::Value::String(e.to_string()));
        }
        None => {
            fm.extra.remove("revisao_ia");
        }
    }
    match rodada.filter(|r| *r > 0) {
        Some(r) => {
            fm.extra.insert("rodada".into(), serde_yaml::Value::Number(r.into()));
        }
        None => {
            fm.extra.remove("rodada");
        }
    }
    MarkdownCodec::substituir_frontmatter(texto, &fm).map_err(|e| e.to_string())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_o_veredito_da_ultima_linha_que_tem_a_marca() {
        assert_eq!(ler_veredito("Tudo certo.\n\n**VEREDITO: aprovada**"), Veredito::Aprovada);
        assert_eq!(ler_veredito("- `Veredito: Aprovado.`"), Veredito::Aprovada);
        assert_eq!(ler_veredito("sem linha nenhuma"), Veredito::SemVeredito);
        assert_eq!(ler_veredito("VEREDITO: talvez"), Veredito::SemVeredito);
        // A última vale: o revisor que citou o formato no meio do texto.
        let r = "Responda com VEREDITO: aprovada quando...\n\n## Ajustes\n\n- falta X\n\nVEREDITO: ajustes";
        assert_eq!(ler_veredito(r), Veredito::Ajustes("- falta X".into()));
    }

    #[test]
    fn os_ajustes_sao_a_secao_ou_a_resposta_inteira() {
        let r = "Quase.\n\n## Ajustes\n\n- `soma.js`: trate `null`\n- teste o caso vazio\n\n## Outros\n\nnada\n\nVEREDITO: ajustes";
        assert_eq!(ler_veredito(r), Veredito::Ajustes("- `soma.js`: trate `null`\n- teste o caso vazio".into()));
        // Sem a seção: o que o revisor disse, sem a linha do veredito.
        assert_eq!(ler_veredito("Falta o teste do caso vazio.\nVEREDITO: ajustes"), Veredito::Ajustes("Falta o teste do caso vazio.".into()));
        let Veredito::Ajustes(vazio) = ler_veredito("VEREDITO: ajustes") else { panic!() };
        assert!(vazio.contains("sem dizer quais"), "{vazio}");
    }

    #[test]
    fn o_diff_grande_e_cortado_dizendo_que_cortou() {
        let linha = "+ uma linha do diff com acentuação\n";
        let grande = linha.repeat(TETO_DO_DIFF / linha.len() + 50);
        let cortado = diff_no_teto(&grande);
        assert!(cortado.len() < grande.len() && cortado.contains("diff cortado"), "{}", cortado.len());
        assert_eq!(diff_no_teto("+ pequeno\n"), "+ pequeno\n");
    }

    #[test]
    fn a_pergunta_trava_o_papel_e_o_formato() {
        let t = Tarefa::ler(
            "pages/projetos/saas/tarefas/003-login.md",
            "---\ntitle: Login\ntype: tarefa\nstatus: executando\n---\n# Login\n",
        )
        .unwrap();
        let m = Material { rodada: 2, teto: 3, verificacao: Some("npm test"), arquivos: vec!["M  login.js".into()], diff: "+x", aviso: None };
        let p = pergunta_de_revisar(&t, &m);
        for esperado in ["revisão 2 de no máximo 3", "não edite", "npm test", "M  login.js", "```diff\n+x\n```", SECAO_AJUSTES, crate::problemas::SECAO, "VEREDITO: aprovada"] {
            assert!(p.contains(esperado), "falta {esperado:?} em:\n{p}");
        }
    }

    #[test]
    fn o_estado_entra_e_sai_do_frontmatter() {
        let t = "---\ntitle: T\ntype: tarefa\nstatus: executando\n---\ncorpo\n";
        let com = com_estado(t, Some(estado::PENDENTE), Some(2)).unwrap();
        assert!(com.contains("revisao_ia: pendente") && com.contains("rodada: 2") && com.ends_with("corpo\n"), "{com}");
        let sem = com_estado(&com, None, None).unwrap();
        assert!(!sem.contains("revisao_ia") && !sem.contains("rodada"), "{sem}");
    }
}
