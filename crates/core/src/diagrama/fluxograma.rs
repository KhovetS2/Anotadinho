//! Fluxograma: `graph`/`flowchart`, nós com formato e ligações.

use super::{desescapar, escapar, id_valido, linha_de_posicoes, tomar_id, ForaDoModelo, Instrucao, Posicoes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Palavra {
    Graph,
    Flowchart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direcao {
    TD,
    TB,
    LR,
    RL,
    BT,
}

impl Direcao {
    pub const TODAS: [Direcao; 5] = [Direcao::TD, Direcao::TB, Direcao::LR, Direcao::RL, Direcao::BT];

    pub fn token(self) -> &'static str {
        match self {
            Direcao::TD => "TD",
            Direcao::TB => "TB",
            Direcao::LR => "LR",
            Direcao::RL => "RL",
            Direcao::BT => "BT",
        }
    }

    fn de(s: &str) -> Option<Self> {
        Self::TODAS.into_iter().find(|d| d.token() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forma {
    Caixa,
    Arredondado,
    Decisao,
    Circulo,
}

impl Forma {
    pub const TODAS: [Forma; 4] = [Forma::Caixa, Forma::Arredondado, Forma::Decisao, Forma::Circulo];

    fn delimitadores(self) -> (&'static str, &'static str) {
        match self {
            Forma::Caixa => ("[", "]"),
            Forma::Arredondado => ("(", ")"),
            Forma::Decisao => ("{", "}"),
            Forma::Circulo => ("((", "))"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoLigacao {
    /// `-->`
    Seta,
    /// `---`
    Linha,
    /// `-.->`
    Pontilhada,
    /// `==>`
    Grossa,
}

impl TipoLigacao {
    pub const TODAS: [TipoLigacao; 4] =
        [TipoLigacao::Seta, TipoLigacao::Linha, TipoLigacao::Pontilhada, TipoLigacao::Grossa];

    pub fn token(self) -> &'static str {
        match self {
            TipoLigacao::Seta => "-->",
            TipoLigacao::Linha => "---",
            TipoLigacao::Pontilhada => "-.->",
            TipoLigacao::Grossa => "==>",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct No {
    pub id: String,
    pub texto: String,
    pub forma: Forma,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ligacao {
    pub de: String,
    pub para: String,
    pub tipo: TipoLigacao,
    pub rotulo: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fluxograma {
    pub palavra: Palavra,
    pub direcao: Direcao,
    /// Na ordem em que aparecem no código.
    pub nos: Vec<No>,
    pub ligacoes: Vec<Ligacao>,
    pub posicoes: Posicoes,
}

impl Fluxograma {
    pub fn novo() -> Self {
        Self { palavra: Palavra::Graph, direcao: Direcao::TD, nos: Vec::new(), ligacoes: Vec::new(), posicoes: Posicoes::new() }
    }

    pub fn escrever(&self) -> String {
        let palavra = match self.palavra {
            Palavra::Graph => "graph",
            Palavra::Flowchart => "flowchart",
        };
        let mut s = format!("{palavra} {}\n", self.direcao.token());
        // Todo nó é declarado, na ordem: só assim a ordem volta igual na
        // leitura (um nó citado só na ligação viria DEPOIS dos declarados).
        for no in &self.nos {
            if no.texto == no.id && no.forma == Forma::Caixa {
                s.push_str(&format!("  {}\n", no.id));
            } else {
                let (a, f) = no.forma.delimitadores();
                s.push_str(&format!("  {}{a}{}{f}\n", no.id, texto_de_no(&no.texto)));
            }
        }
        for l in &self.ligacoes {
            if l.rotulo.is_empty() {
                s.push_str(&format!("  {} {} {}\n", l.de, l.tipo.token(), l.para));
            } else {
                s.push_str(&format!("  {} {}|{}| {}\n", l.de, l.tipo.token(), texto_de_rotulo(&l.rotulo), l.para));
            }
        }
        if let Some(p) = linha_de_posicoes(&self.posicoes) {
            s.push_str(&p);
            s.push('\n');
        }
        s
    }

    fn registrar(&mut self, no: No, declarado: bool) {
        match self.nos.iter_mut().find(|n| n.id == no.id) {
            // Redeclarar vale a última, como no mermaid.
            Some(n) if declarado => *n = no,
            Some(_) => {}
            None => self.nos.push(no),
        }
    }
}

/// Texto que precisa de aspas: o que fecharia a forma ou é sintaxe.
fn texto_de_no(t: &str) -> String {
    let e = escapar(t);
    if e.chars().any(|c| "[](){}|<>".contains(c)) || e.trim() != e || e.is_empty() {
        format!("\"{e}\"")
    } else {
        e
    }
}

fn texto_de_rotulo(t: &str) -> String {
    let e = escapar(t);
    if e.contains('|') {
        format!("\"{e}\"")
    } else {
        e
    }
}

pub(crate) fn ler(linha: usize, cabecalho: &str, resto: &[Instrucao]) -> Result<Fluxograma, ForaDoModelo> {
    let mut partes = cabecalho.split_whitespace();
    let palavra = match partes.next() {
        Some("flowchart") => Palavra::Flowchart,
        _ => Palavra::Graph,
    };
    let direcao = match partes.next() {
        // Sem direção, o mermaid usa TD.
        None => Direcao::TD,
        Some(d) => Direcao::de(d).ok_or_else(|| ForaDoModelo::na(linha, format!("direção `{d}`")))?,
    };
    if partes.next().is_some() {
        return Err(ForaDoModelo::na(linha, "cabeçalho com mais coisa além da direção"));
    }
    let mut f = Fluxograma { palavra, direcao, ..Fluxograma::novo() };
    for inst in resto {
        match inst {
            Instrucao::Posicoes { posicoes, .. } => f.posicoes = posicoes.clone(),
            Instrucao::Texto { linha, texto } => ler_instrucao(&mut f, *linha, texto)?,
        }
    }
    Ok(f)
}

fn ler_instrucao(f: &mut Fluxograma, linha: usize, texto: &str) -> Result<(), ForaDoModelo> {
    let primeira = texto.split_whitespace().next().unwrap_or("");
    for palavra in ["subgraph", "end", "style", "classDef", "class", "click", "linkStyle", "direction"] {
        if primeira == palavra {
            return Err(ForaDoModelo::na(linha, format!("`{palavra}` não é montado pelo construtor")));
        }
    }
    let (no, mut resto) = ler_no(linha, texto)?;
    let mut anterior = no.0.id.clone();
    f.registrar(no.0, no.1);
    loop {
        resto = resto.trim_start();
        if resto.is_empty() {
            return Ok(());
        }
        let (tipo, rotulo, depois) = ler_ligacao(linha, resto)?;
        let (no, depois) = ler_no(linha, depois.trim_start())?;
        let para = no.0.id.clone();
        f.registrar(no.0, no.1);
        f.ligacoes.push(Ligacao { de: anterior, para: para.clone(), tipo, rotulo });
        anterior = para;
        resto = depois;
    }
}

/// Um nó: o id e, se houver, a forma com o texto. O `bool` diz se o nó
/// foi DECLARADO aqui (tinha forma) ou só citado.
fn ler_no(linha: usize, s: &str) -> Result<((No, bool), &str), ForaDoModelo> {
    let (id, resto) = tomar_id(s);
    if id.is_empty() {
        return Err(ForaDoModelo::na(linha, format!("não entendi `{}`", s.chars().take(30).collect::<String>())));
    }
    if !id_valido(id) {
        return Err(ForaDoModelo::na(linha, format!("`{id}` não serve de id de nó")));
    }
    for raro in ["([", "[[", "[(", "[/", "[\\", "{{", ">", "(((", "@{"] {
        if resto.starts_with(raro) {
            return Err(ForaDoModelo::na(linha, format!("forma `{raro}…` não é montada pelo construtor")));
        }
    }
    // `((` antes de `(`: o círculo começa com o arredondado.
    for forma in [Forma::Circulo, Forma::Caixa, Forma::Arredondado, Forma::Decisao] {
        let (abre, fecha) = forma.delimitadores();
        if let Some(dentro) = resto.strip_prefix(abre) {
            let (texto, depois) = ler_texto(linha, dentro, fecha)?;
            return Ok(((No { id: id.to_string(), texto, forma }, true), depois));
        }
    }
    Ok(((No { id: id.to_string(), texto: id.to_string(), forma: Forma::Caixa }, false), resto))
}

/// Texto até `fecha`, com ou sem aspas.
fn ler_texto<'a>(linha: usize, s: &'a str, fecha: &str) -> Result<(String, &'a str), ForaDoModelo> {
    if let Some(dentro) = s.strip_prefix('"') {
        let fim = dentro.find('"').ok_or_else(|| ForaDoModelo::na(linha, "aspas sem fechar"))?;
        let depois = dentro[fim + 1..]
            .strip_prefix(fecha)
            .ok_or_else(|| ForaDoModelo::na(linha, format!("esperava `{fecha}` depois das aspas")))?;
        return Ok((desescapar(&dentro[..fim]), depois));
    }
    let fim = s.find(fecha).ok_or_else(|| ForaDoModelo::na(linha, format!("`{fecha}` sem fechar")))?;
    let texto = &s[..fim];
    if texto.contains(['[', '(', '{', '"']) {
        return Err(ForaDoModelo::na(linha, "texto de nó com delimitador dentro"));
    }
    Ok((desescapar(texto.trim()), &s[fim + fecha.len()..]))
}

fn ler_ligacao(linha: usize, s: &str) -> Result<(TipoLigacao, String, &str), ForaDoModelo> {
    // `-- texto -->` e `-- texto ---`: o rótulo no meio da seta.
    if let Some(meio) = s.strip_prefix("-- ") {
        for tipo in [TipoLigacao::Seta, TipoLigacao::Linha] {
            let fim = format!(" {}", tipo.token());
            if let Some(i) = meio.find(&fim) {
                return Ok((tipo, desescapar(meio[..i].trim()), &meio[i + fim.len()..]));
            }
        }
    }
    // Mais longo primeiro: `-.->` antes de qualquer coisa com `-`.
    for tipo in [TipoLigacao::Pontilhada, TipoLigacao::Seta, TipoLigacao::Linha, TipoLigacao::Grossa] {
        let Some(depois) = s.strip_prefix(tipo.token()) else {
            continue;
        };
        // `--->`, `-->o`, `--x`… são outras setas: o próximo caractere
        // ainda é seta, ou é ponta.
        if depois.starts_with(['-', '>', '=', '.']) || (depois.starts_with(['o', 'x']) && !depois[1..].starts_with(|c: char| c.is_alphanumeric() || c == '_')) {
            break;
        }
        let depois = depois.trim_start();
        if let Some(rot) = depois.strip_prefix('|') {
            let (texto, apos) = if let Some(q) = rot.strip_prefix('"') {
                let fim = q.find('"').ok_or_else(|| ForaDoModelo::na(linha, "aspas sem fechar no rótulo"))?;
                let apos = q[fim + 1..].strip_prefix('|').ok_or_else(|| ForaDoModelo::na(linha, "rótulo sem `|` no fim"))?;
                (q[..fim].to_string(), apos)
            } else {
                let fim = rot.find('|').ok_or_else(|| ForaDoModelo::na(linha, "rótulo sem `|` no fim"))?;
                (rot[..fim].trim().to_string(), &rot[fim + 1..])
            };
            return Ok((tipo, desescapar(&texto), apos));
        }
        return Ok((tipo, String::new(), depois));
    }
    if s.starts_with('&') {
        return Err(ForaDoModelo::na(linha, "`&` (vários nós numa ligação) não é montado pelo construtor"));
    }
    Err(ForaDoModelo::na(
        linha,
        format!("ligação `{}` não é montada pelo construtor", s.split_whitespace().next().unwrap_or(s)),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::{escrever, ler, Diagrama};
    use super::*;

    fn fluxo(codigo: &str) -> Fluxograma {
        match ler(codigo) {
            Ok(Diagrama::Fluxograma(f)) => f,
            outro => panic!("não leu como fluxograma: {outro:?}"),
        }
    }

    const CANONICO: &str = concat!(
        "graph TD\n",
        "  A[Ideia]\n",
        "  B(Detalhando)\n",
        "  C{Executar?}\n",
        "  D((Fim))\n",
        "  Solto\n",
        "  E\n",
        "  A --> B\n",
        "  B -->|pronta| C\n",
        "  C -.-> D\n",
        "  C ==>|não| A\n",
        "  D --- E\n",
        "  %% anotadinho-posicoes: {\"A\":[10.0,20.0],\"B\":[30.5,40.0]}\n",
    );

    #[test]
    fn canonico_volta_igual() {
        let f = fluxo(CANONICO);
        assert_eq!(f.nos.len(), 6, "{:?}", f.nos);
        assert_eq!(f.nos[2].forma, Forma::Decisao);
        assert_eq!(f.nos[3].forma, Forma::Circulo);
        assert_eq!(f.ligacoes[1].rotulo, "pronta");
        assert_eq!(f.ligacoes[2].tipo, TipoLigacao::Pontilhada);
        assert_eq!(f.posicoes["B"], [30.5, 40.0]);
        assert_eq!(escrever(&Diagrama::Fluxograma(f)), CANONICO);
    }

    #[test]
    fn a_mao_reescreve_no_mesmo_modelo() {
        let a_mao = fluxo("graph TD; A-->B; B -- sim --> C[Fim];");
        let reescrito = fluxo(&escrever(&Diagrama::Fluxograma(a_mao.clone())));
        assert_eq!(a_mao, reescrito);
        assert_eq!(a_mao.ligacoes[1].rotulo, "sim");
        assert_eq!(a_mao.nos[2].texto, "Fim");
    }

    #[test]
    fn cadeia_e_no_declarado_na_ligacao() {
        let f = fluxo("flowchart LR\n  A[Um] --> B{Dois} --> C");
        assert_eq!(f.palavra, Palavra::Flowchart);
        assert_eq!(f.direcao, Direcao::LR);
        assert_eq!(f.ligacoes.len(), 2);
        assert_eq!((f.ligacoes[1].de.as_str(), f.ligacoes[1].para.as_str()), ("B", "C"));
        assert_eq!(f.nos[1].forma, Forma::Decisao);
    }

    #[test]
    fn texto_com_sintaxe_vai_entre_aspas_e_volta() {
        let mut f = Fluxograma::novo();
        f.nos.push(No { id: "A".into(), texto: "pergunta (sim; não) \"x\"".into(), forma: Forma::Caixa });
        f.nos.push(No { id: "B".into(), texto: "B".into(), forma: Forma::Caixa });
        f.ligacoes.push(Ligacao { de: "A".into(), para: "B".into(), tipo: TipoLigacao::Seta, rotulo: "a|b; c".into() });
        let d = Diagrama::Fluxograma(f);
        let codigo = escrever(&d);
        assert_eq!(ler(&codigo).unwrap(), d, "{codigo}");
    }

    #[test]
    fn redeclarar_vale_a_ultima() {
        let f = fluxo("graph TD\n  A[um]\n  A --> B\n  A[dois]");
        assert_eq!(f.nos[0].texto, "dois");
    }

    #[test]
    fn o_que_nao_monta_diz_a_linha() {
        let casos = [
            ("graph TD\n  subgraph X\n  A\n  end", 2, "subgraph"),
            ("graph TD\n  A --> B\n  style A fill:#f00", 3, "style"),
            ("graph TD\n  A([estádio])", 2, "forma"),
            ("graph TD\n  A --o B", 2, "ligação"),
            ("graph TD\n  A ---> B", 2, "ligação"),
            ("graph TD\n  A & B --> C", 2, "&"),
            ("graph XY\n  A", 1, "direção"),
            ("graph TD\n  end --> A", 2, "end"),
        ];
        for (codigo, linha, trecho) in casos {
            let e = ler(codigo).expect_err(codigo);
            assert_eq!(e.linha, linha, "{codigo}: {e}");
            assert!(e.motivo.contains(trecho), "{codigo}: {e}");
        }
    }

    #[test]
    fn no_com_o_ou_x_no_inicio_nao_vira_ponta() {
        // `A --> ok` é um nó chamado `ok`, não a ponta `o`.
        let f = fluxo("graph TD\n  A --> ok\n  A --> xis");
        assert_eq!(f.ligacoes[0].para, "ok");
        assert_eq!(f.ligacoes[1].para, "xis");
    }
}
