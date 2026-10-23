//! Sequência: participantes e, em ordem, mensagens e notas.

use super::{desescapar, escapar, id_valido, sem_posicoes, tomar_id, ForaDoModelo, Instrucao};

#[derive(Debug, Clone, PartialEq)]
pub struct Participante {
    pub id: String,
    /// O nome mostrado; igual ao id quando não há `as`.
    pub nome: String,
    /// `actor` (bonequinho) em vez de `participant` (caixa).
    pub ator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seta {
    /// `->>` pedido
    Sincrona,
    /// `-->>` resposta
    Resposta,
    /// `-)` sem esperar
    Assincrona,
    /// `--)`
    AssincronaPontilhada,
    /// `->`
    Linha,
    /// `-->`
    LinhaPontilhada,
    /// `-x` falhou
    Cruz,
    /// `--x`
    CruzPontilhada,
}

impl Seta {
    pub const TODAS: [Seta; 8] = [
        Seta::Sincrona,
        Seta::Resposta,
        Seta::Assincrona,
        Seta::AssincronaPontilhada,
        Seta::Linha,
        Seta::LinhaPontilhada,
        Seta::Cruz,
        Seta::CruzPontilhada,
    ];

    pub fn token(self) -> &'static str {
        match self {
            Seta::Sincrona => "->>",
            Seta::Resposta => "-->>",
            Seta::Assincrona => "-)",
            Seta::AssincronaPontilhada => "--)",
            Seta::Linha => "->",
            Seta::LinhaPontilhada => "-->",
            Seta::Cruz => "-x",
            Seta::CruzPontilhada => "--x",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lado {
    Direita,
    Esquerda,
    Sobre,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Mensagem { de: String, para: String, seta: Seta, texto: String },
    /// `alvos` tem um participante, ou dois quando `Sobre` cobre os dois.
    Nota { lado: Lado, alvos: Vec<String>, texto: String },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sequencia {
    /// Na ordem em que aparecem: é a ordem das colunas no desenho.
    pub participantes: Vec<Participante>,
    pub itens: Vec<Item>,
}

impl Sequencia {
    pub fn escrever(&self) -> String {
        let mut s = String::from("sequenceDiagram\n");
        for p in &self.participantes {
            let palavra = if p.ator { "actor" } else { "participant" };
            if p.nome == p.id {
                s.push_str(&format!("  {palavra} {}\n", p.id));
            } else {
                s.push_str(&format!("  {palavra} {} as {}\n", p.id, escapar(&p.nome)));
            }
        }
        for item in &self.itens {
            match item {
                Item::Mensagem { de, para, seta, texto } => {
                    s.push_str(&format!("  {de}{}{para}: {}\n", seta.token(), escapar(texto)));
                }
                Item::Nota { lado, alvos, texto } => {
                    let onde = match lado {
                        Lado::Direita => "right of",
                        Lado::Esquerda => "left of",
                        Lado::Sobre => "over",
                    };
                    s.push_str(&format!("  Note {onde} {}: {}\n", alvos.join(","), escapar(texto)));
                }
            }
        }
        s
    }

    fn citar(&mut self, id: &str) {
        if !self.participantes.iter().any(|p| p.id == id) {
            self.participantes.push(Participante { id: id.to_string(), nome: id.to_string(), ator: false });
        }
    }
}

pub(crate) fn ler(linha: usize, cabecalho: &str, resto: &[Instrucao]) -> Result<Sequencia, ForaDoModelo> {
    if cabecalho.trim() != "sequenceDiagram" {
        return Err(ForaDoModelo::na(linha, "cabeçalho com mais coisa além de `sequenceDiagram`"));
    }
    sem_posicoes(resto)?;
    let mut s = Sequencia::default();
    for inst in resto {
        if let Instrucao::Texto { linha, texto } = inst {
            ler_instrucao(&mut s, *linha, texto)?;
        }
    }
    Ok(s)
}

fn ler_instrucao(s: &mut Sequencia, linha: usize, texto: &str) -> Result<(), ForaDoModelo> {
    let primeira = texto.split_whitespace().next().unwrap_or("");
    match primeira {
        "participant" | "actor" => return ler_participante(s, linha, primeira, texto),
        "loop" | "alt" | "else" | "opt" | "par" | "and" | "critical" | "break" | "rect" | "end"
        | "activate" | "deactivate" | "autonumber" | "box" | "create" | "destroy" | "links" | "link"
        | "title" | "accTitle" | "accDescr" => {
            return Err(ForaDoModelo::na(linha, format!("`{primeira}` não é montado pelo construtor")));
        }
        _ => {}
    }
    if primeira.eq_ignore_ascii_case("note") {
        return ler_nota(s, linha, texto);
    }
    ler_mensagem(s, linha, texto)
}

fn ler_participante(s: &mut Sequencia, linha: usize, palavra: &str, texto: &str) -> Result<(), ForaDoModelo> {
    let resto = texto[palavra.len()..].trim();
    let (id, depois) = tomar_id(resto);
    if !id_valido(id) {
        return Err(ForaDoModelo::na(linha, format!("`{resto}` não serve de id de participante")));
    }
    let depois = depois.trim();
    let nome = if depois.is_empty() {
        id.to_string()
    } else if let Some(n) = depois.strip_prefix("as ") {
        desescapar(n.trim())
    } else {
        return Err(ForaDoModelo::na(linha, format!("não entendi `{depois}` no participante")));
    };
    if s.participantes.iter().any(|p| p.id == id) {
        return Err(ForaDoModelo::na(linha, format!("`{id}` declarado depois de já aparecer")));
    }
    s.participantes.push(Participante { id: id.to_string(), nome, ator: palavra == "actor" });
    Ok(())
}

fn ler_mensagem(s: &mut Sequencia, linha: usize, texto: &str) -> Result<(), ForaDoModelo> {
    let (de, resto) = tomar_id(texto);
    if !id_valido(de) {
        return Err(ForaDoModelo::na(linha, format!("não entendi `{}`", texto.chars().take(30).collect::<String>())));
    }
    let resto = resto.trim_start();
    // Mais longo primeiro: `-->>` antes de `-->` e de `->>`.
    let mut setas = Seta::TODAS;
    setas.sort_by_key(|s| std::cmp::Reverse(s.token().len()));
    let Some(seta) = setas.into_iter().find(|s| resto.starts_with(s.token())) else {
        return Err(ForaDoModelo::na(linha, format!("seta `{}` não é montada pelo construtor", resto.split_whitespace().next().unwrap_or(resto))));
    };
    let resto = resto[seta.token().len()..].trim_start();
    if resto.starts_with(['+', '-']) {
        return Err(ForaDoModelo::na(linha, "ativação (`+`/`-` na seta) não é montada pelo construtor"));
    }
    let (para, depois) = tomar_id(resto);
    if !id_valido(para) {
        return Err(ForaDoModelo::na(linha, format!("`{resto}` não serve de destino da mensagem")));
    }
    let depois = depois.trim_start();
    let texto = match depois.strip_prefix(':') {
        Some(t) => desescapar(t.trim()),
        None if depois.is_empty() => String::new(),
        None => return Err(ForaDoModelo::na(linha, format!("não entendi `{depois}` depois do destino"))),
    };
    s.citar(de);
    s.citar(para);
    s.itens.push(Item::Mensagem { de: de.to_string(), para: para.to_string(), seta, texto });
    Ok(())
}

fn ler_nota(s: &mut Sequencia, linha: usize, texto: &str) -> Result<(), ForaDoModelo> {
    let resto = texto[4..].trim_start();
    let (lado, resto) = if let Some(r) = resto.strip_prefix("right of ") {
        (Lado::Direita, r)
    } else if let Some(r) = resto.strip_prefix("left of ") {
        (Lado::Esquerda, r)
    } else if let Some(r) = resto.strip_prefix("over ") {
        (Lado::Sobre, r)
    } else {
        return Err(ForaDoModelo::na(linha, "nota sem `right of`, `left of` ou `over`"));
    };
    let (alvos, texto) = resto.split_once(':').ok_or_else(|| ForaDoModelo::na(linha, "nota sem `:` antes do texto"))?;
    let alvos: Vec<String> = alvos.split(',').map(|a| a.trim().to_string()).collect();
    if alvos.iter().any(|a| !id_valido(a)) || alvos.len() > 2 || (alvos.len() == 2 && lado != Lado::Sobre) {
        return Err(ForaDoModelo::na(linha, "participantes da nota não são montados pelo construtor"));
    }
    for a in &alvos {
        s.citar(a);
    }
    s.itens.push(Item::Nota { lado, alvos, texto: desescapar(texto.trim()) });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{escrever, ler, Diagrama};
    use super::*;

    fn seq(codigo: &str) -> Sequencia {
        match ler(codigo) {
            Ok(Diagrama::Sequencia(s)) => s,
            outro => panic!("não leu como sequência: {outro:?}"),
        }
    }

    const CANONICO: &str = concat!(
        "sequenceDiagram\n",
        "  actor P as Pessoa\n",
        "  participant A as Agente\n",
        "  participant V\n",
        "  P->>A: pede uma mudança\n",
        "  A-->>P: entendi\n",
        "  A-)V: propor\n",
        "  A--xV: falhou\n",
        "  Note right of V: fila\n",
        "  Note over P,A: conversa\n",
    );

    #[test]
    fn canonico_volta_igual() {
        let s = seq(CANONICO);
        assert!(s.participantes[0].ator);
        assert_eq!(s.participantes[1].nome, "Agente");
        assert!(matches!(s.itens[2], Item::Mensagem { seta: Seta::Assincrona, .. }));
        assert!(matches!(s.itens[3], Item::Mensagem { seta: Seta::CruzPontilhada, .. }));
        assert_eq!(escrever(&Diagrama::Sequencia(s)), CANONICO);
    }

    #[test]
    fn participante_implicito_entra_na_ordem_em_que_aparece() {
        let s = seq("sequenceDiagram\n  participant B\n  A->>B: oi\n  B-->>A: oi");
        let ids: Vec<_> = s.participantes.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["B", "A"]);
        assert_eq!(seq(&escrever(&Diagrama::Sequencia(s.clone()))), s);
    }

    #[test]
    fn texto_com_ponto_e_virgula_e_dois_pontos_volta_igual() {
        let mut s = Sequencia::default();
        s.participantes.push(Participante { id: "A".into(), nome: "Fulano; o \"chefe\"".into(), ator: false });
        s.participantes.push(Participante { id: "B".into(), nome: "B".into(), ator: false });
        s.itens.push(Item::Mensagem { de: "A".into(), para: "B".into(), seta: Seta::Sincrona, texto: "hora: 10h; já".into() });
        let d = Diagrama::Sequencia(s);
        assert_eq!(ler(&escrever(&d)).unwrap(), d);
    }

    #[test]
    fn o_que_nao_monta_diz_a_linha() {
        let casos = [
            ("sequenceDiagram\n  A->>B: x\n  loop todo dia\n  A->>B: y\n  end", 3, "loop"),
            ("sequenceDiagram\n  A->>+B: x", 2, "ativação"),
            ("sequenceDiagram\n  autonumber", 2, "autonumber"),
            ("sequenceDiagram\n  A<<->>B: x", 2, "seta"),
            ("sequenceDiagram\n  A->>B: x\n  participant A", 3, "declarado depois"),
            ("sequenceDiagram\n  A->>B: x\n  %% anotadinho-posicoes: {}", 3, "sem canvas"),
        ];
        for (codigo, linha, trecho) in casos {
            let e = ler(codigo).expect_err(codigo);
            assert_eq!(e.linha, linha, "{codigo}: {e}");
            assert!(e.motivo.contains(trecho), "{codigo}: {e}");
        }
    }
}
