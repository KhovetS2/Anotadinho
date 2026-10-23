//! Estados: estados, transições com rótulo, início e fim (`[*]`).

use super::{desescapar, id_valido, linha_de_posicoes, tomar_id, ForaDoModelo, Instrucao, Posicoes};

/// O rótulo da transição: o mermaid não aceita `:` cru ali (ele separa o
/// rótulo por `:`), mas troca as entidades na hora de desenhar. Achado
/// passando as amostras pelo `mermaid.parse` de verdade (ciclo 488).
fn escapar(t: &str) -> String {
    super::escapar(t).replace(':', "#58;")
}

/// O nome entre aspas (`state "…" as id`) é o contrário do rótulo: o
/// mermaid NÃO troca entidade ali (`#58;` aparecia escrito no desenho,
/// visto no ciclo 490), e aceita `:` e `;` crus. Só `"` não tem como
/// entrar: vira `'`. É a única troca com perda do construtor, e é do
/// mermaid, que não oferece outra forma.
pub fn nome_no_codigo(t: &str) -> String {
    t.replace('"', "'")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Estado {
    pub id: String,
    /// O nome mostrado; igual ao id quando não há descrição.
    pub texto: String,
}

/// Uma ponta de transição. `[*]` à esquerda é o início, à direita o fim:
/// é o mermaid que decide pelo lado, e o modelo guarda já decidido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ponta {
    Inicio,
    Fim,
    Estado(String),
}

impl Ponta {
    /// A chave em [`Posicoes`]: o início e o fim também têm lugar no canvas.
    pub fn chave(&self) -> &str {
        match self {
            Ponta::Inicio => "[*]inicio",
            Ponta::Fim => "[*]fim",
            Ponta::Estado(id) => id,
        }
    }

    fn codigo(&self) -> &str {
        match self {
            Ponta::Inicio | Ponta::Fim => "[*]",
            Ponta::Estado(id) => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transicao {
    pub de: Ponta,
    pub para: Ponta,
    pub rotulo: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Estados {
    /// `stateDiagram-v2` (o atual) ou `stateDiagram`.
    pub v2: bool,
    pub estados: Vec<Estado>,
    pub transicoes: Vec<Transicao>,
    pub posicoes: Posicoes,
}

impl Estados {
    pub fn novo() -> Self {
        Self { v2: true, estados: Vec::new(), transicoes: Vec::new(), posicoes: Posicoes::new() }
    }

    pub fn escrever(&self) -> String {
        let mut s = String::from(if self.v2 { "stateDiagram-v2\n" } else { "stateDiagram\n" });
        // Todo estado é declarado, na ordem: um estado citado só na
        // transição voltaria DEPOIS dos declarados (achado pelo teste de
        // propriedade, como no fluxograma).
        for e in &self.estados {
            if e.texto != e.id {
                s.push_str(&format!("  state \"{}\" as {}\n", nome_no_codigo(&e.texto), e.id));
            } else {
                s.push_str(&format!("  {}\n", e.id));
            }
        }
        for t in &self.transicoes {
            if t.rotulo.is_empty() {
                s.push_str(&format!("  {} --> {}\n", t.de.codigo(), t.para.codigo()));
            } else {
                s.push_str(&format!("  {} --> {} : {}\n", t.de.codigo(), t.para.codigo(), escapar(&t.rotulo)));
            }
        }
        if let Some(p) = linha_de_posicoes(&self.posicoes) {
            s.push_str(&p);
            s.push('\n');
        }
        s
    }

    fn citar(&mut self, id: &str) {
        if !self.estados.iter().any(|e| e.id == id) {
            self.estados.push(Estado { id: id.to_string(), texto: id.to_string() });
        }
    }

    fn descrever(&mut self, id: &str, texto: String) {
        match self.estados.iter_mut().find(|e| e.id == id) {
            Some(e) => e.texto = texto,
            None => self.estados.push(Estado { id: id.to_string(), texto }),
        }
    }
}

pub(crate) fn ler(linha: usize, cabecalho: &str, resto: &[Instrucao]) -> Result<Estados, ForaDoModelo> {
    let v2 = match cabecalho.trim() {
        "stateDiagram-v2" => true,
        "stateDiagram" => false,
        _ => return Err(ForaDoModelo::na(linha, "cabeçalho com mais coisa além do tipo")),
    };
    let mut e = Estados { v2, ..Estados::novo() };
    for inst in resto {
        match inst {
            Instrucao::Posicoes { posicoes, .. } => e.posicoes = posicoes.clone(),
            Instrucao::Texto { linha, texto } => ler_instrucao(&mut e, *linha, texto)?,
        }
    }
    Ok(e)
}

fn ler_instrucao(e: &mut Estados, linha: usize, texto: &str) -> Result<(), ForaDoModelo> {
    let primeira = texto.split_whitespace().next().unwrap_or("");
    if ["direction", "note", "classDef", "class", "}", "--", "accTitle", "accDescr"].contains(&primeira)
        || primeira.eq_ignore_ascii_case("note")
        || texto.contains("<<")
        || texto.ends_with('{')
    {
        return Err(ForaDoModelo::na(linha, format!("`{primeira}` não é montado pelo construtor")));
    }
    if let Some(resto) = texto.strip_prefix("state ") {
        return ler_declaracao(e, linha, resto.trim());
    }
    if let Some((esq, dir)) = texto.split_once("-->") {
        let (dir, rotulo) = match dir.split_once(':') {
            Some((d, r)) => (d, desescapar(r.trim())),
            None => (dir, String::new()),
        };
        let de = ponta(e, linha, esq.trim(), true)?;
        let para = ponta(e, linha, dir.trim(), false)?;
        if de == Ponta::Inicio && para == Ponta::Fim {
            return Err(ForaDoModelo::na(linha, "`[*] --> [*]` não é montado pelo construtor"));
        }
        e.transicoes.push(Transicao { de, para, rotulo });
        return Ok(());
    }
    // `A : descrição` ou só `A`.
    let (id, depois) = tomar_id(texto);
    if !id_valido(id) {
        return Err(ForaDoModelo::na(linha, format!("não entendi `{}`", texto.chars().take(30).collect::<String>())));
    }
    let depois = depois.trim();
    // `A:::classe` é estilo, não descrição. Só aqui: num rótulo de
    // transição, `:::` é texto (o teste de propriedade achou a recusa).
    if depois.starts_with(":::") {
        return Err(ForaDoModelo::na(linha, "classe `:::` não é montada pelo construtor"));
    }
    if depois.is_empty() {
        e.citar(id);
    } else if let Some(d) = depois.strip_prefix(':') {
        e.descrever(id, desescapar(d.trim()));
    } else {
        return Err(ForaDoModelo::na(linha, format!("não entendi `{depois}` depois de `{id}`")));
    }
    Ok(())
}

/// `state "Nome longo" as A`.
fn ler_declaracao(e: &mut Estados, linha: usize, resto: &str) -> Result<(), ForaDoModelo> {
    let Some(dentro) = resto.strip_prefix('"') else {
        return Err(ForaDoModelo::na(linha, "`state` sem `\"nome\" as id` não é montado pelo construtor"));
    };
    let fim = dentro.find('"').ok_or_else(|| ForaDoModelo::na(linha, "aspas sem fechar"))?;
    let id = dentro[fim + 1..]
        .trim()
        .strip_prefix("as ")
        .map(str::trim)
        .ok_or_else(|| ForaDoModelo::na(linha, "`state \"…\"` sem `as id`"))?;
    if !id_valido(id) {
        return Err(ForaDoModelo::na(linha, format!("`{id}` não serve de id de estado")));
    }
    e.descrever(id, desescapar(&dentro[..fim]));
    Ok(())
}

fn ponta(e: &mut Estados, linha: usize, s: &str, esquerda: bool) -> Result<Ponta, ForaDoModelo> {
    if s == "[*]" {
        return Ok(if esquerda { Ponta::Inicio } else { Ponta::Fim });
    }
    if !id_valido(s) {
        return Err(ForaDoModelo::na(linha, format!("`{s}` não serve de estado")));
    }
    e.citar(s);
    Ok(Ponta::Estado(s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::super::{escrever, ler, Diagrama};
    use super::*;

    fn est(codigo: &str) -> Estados {
        match ler(codigo) {
            Ok(Diagrama::Estados(e)) => e,
            outro => panic!("não leu como estados: {outro:?}"),
        }
    }

    const CANONICO: &str = concat!(
        "stateDiagram-v2\n",
        "  state \"Em revisão\" as Revisao\n",
        "  Solto\n",
        "  Ideia\n",
        "  [*] --> Ideia\n",
        "  Ideia --> Revisao : detalhou\n",
        "  Revisao --> Ideia : voltou\n",
        "  Revisao --> [*]\n",
        "  %% anotadinho-posicoes: {\"Ideia\":[0.0,0.0],\"[*]inicio\":[0.0,-80.0]}\n",
    );

    #[test]
    fn canonico_volta_igual() {
        let e = est(CANONICO);
        let ids: Vec<_> = e.estados.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, ["Revisao", "Solto", "Ideia"]);
        assert_eq!(e.transicoes[0].de, Ponta::Inicio);
        assert_eq!(e.transicoes[3].para, Ponta::Fim);
        assert_eq!(e.transicoes[1].rotulo, "detalhou");
        assert_eq!(escrever(&Diagrama::Estados(e)), CANONICO);
    }

    #[test]
    fn descricao_com_dois_pontos_reescreve_no_mesmo_modelo() {
        let a_mao = est("stateDiagram\n  A : Aguardando\n  [*] --> A\n  A --> B: pronto");
        assert!(!a_mao.v2);
        assert_eq!(a_mao.estados[0].texto, "Aguardando");
        assert_eq!(est(&escrever(&Diagrama::Estados(a_mao.clone()))), a_mao);
    }

    #[test]
    fn o_que_nao_monta_diz_a_linha() {
        let casos = [
            ("stateDiagram-v2\n  state Composto {\n  A --> B\n  }", 2, "state"),
            ("stateDiagram-v2\n  state escolha <<choice>>", 2, "state"),
            ("stateDiagram-v2\n  A --> B\n  note right of A : x", 3, "note"),
            ("stateDiagram-v2\n  [*] --> [*]", 2, "[*]"),
            ("stateDiagram-v2\n  direction LR", 2, "direction"),
            ("stateDiagram-v2\n  A:::quente", 2, "classe"),
            ("stateDiagram-v2\n  A:::quente --> B", 2, "não serve de estado"),
        ];
        for (codigo, linha, trecho) in casos {
            let e = ler(codigo).expect_err(codigo);
            assert_eq!(e.linha, linha, "{codigo}: {e}");
            assert!(e.motivo.contains(trecho), "{codigo}: {e}");
        }
    }
}
