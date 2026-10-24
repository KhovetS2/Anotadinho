//! O modelo dos diagramas do construtor mermaid (ciclo 487).
//!
//! `ler` transforma o código de um bloco ` ```mermaid ` num [`Diagrama`],
//! e `escrever` faz o caminho de volta. O arquivo guarda só mermaid
//! (RNF1 da spec `construtor-mermaid`); este modelo existe só enquanto o
//! modal está aberto.
//!
//! O leitor é ESTRITO de propósito: o que ele não reconhece vira
//! [`ForaDoModelo`], com a linha e o motivo, e o modal abre no código. Um
//! leitor tolerante ignoraria a linha, e o "Aplicar" seguinte a apagaria
//! do arquivo sem ninguém ver.

pub mod camadas;
pub mod estados;
pub mod fluxograma;
pub mod gantt;
pub mod sequencia;
pub mod so_ver;
pub mod texto;

use std::collections::BTreeMap;

pub use estados::Estados;
pub use fluxograma::Fluxograma;
pub use gantt::Gantt;
pub use sequencia::Sequencia;

/// Um diagrama que o construtor sabe montar.
#[derive(Debug, Clone, PartialEq)]
pub enum Diagrama {
    Fluxograma(Fluxograma),
    Sequencia(Sequencia),
    Estados(Estados),
    Gantt(Gantt),
}

/// Os tipos que o construtor monta, na ordem do seletor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Fluxograma,
    Sequencia,
    Estados,
    Gantt,
}

impl Tipo {
    pub const TODOS: [Tipo; 4] = [Tipo::Fluxograma, Tipo::Sequencia, Tipo::Estados, Tipo::Gantt];

    pub fn nome(self) -> &'static str {
        match self {
            Tipo::Fluxograma => "Fluxograma",
            Tipo::Sequencia => "Sequência",
            Tipo::Estados => "Estados",
            Tipo::Gantt => "Gantt",
        }
    }
}

impl Diagrama {
    pub fn tipo(&self) -> Tipo {
        match self {
            Diagrama::Fluxograma(_) => Tipo::Fluxograma,
            Diagrama::Sequencia(_) => Tipo::Sequencia,
            Diagrama::Estados(_) => Tipo::Estados,
            Diagrama::Gantt(_) => Tipo::Gantt,
        }
    }

    /// O ponto de partida de um diagrama novo: pequeno, mas com cada peça
    /// do tipo, pra a pessoa ver o que dá pra fazer mexendo, e não lendo.
    pub fn exemplo(tipo: Tipo) -> Diagrama {
        let codigo = match tipo {
            Tipo::Fluxograma => "graph TD\n  A[Começo]\n  B{Decide}\n  C(Faz)\n  D((Fim))\n  A --> B\n  B -->|sim| C\n  B -->|não| D\n  C --> D\n",
            Tipo::Sequencia => "sequenceDiagram\n  actor P as Pessoa\n  participant S as Sistema\n  P->>S: pede\n  S-->>P: responde\n  Note right of S: pensa\n",
            Tipo::Estados => "stateDiagram-v2\n  state \"Em andamento\" as Andamento\n  Feito\n  [*] --> Andamento\n  Andamento --> Feito : terminou\n  Feito --> [*]\n",
            Tipo::Gantt => "gantt\n  dateFormat YYYY-MM-DD\n  section Primeira\n    Planejar :a1, 2026-11-02, 3d\n    Fazer :after a1, 5d\n",
        };
        ler(codigo).expect("o exemplo é do próprio construtor")
    }
}

/// O código tem algo que o construtor não monta. `linha` é 1-based; 0 é o
/// diagrama inteiro.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", self.mensagem())]
pub struct ForaDoModelo {
    pub linha: usize,
    pub motivo: String,
}

impl ForaDoModelo {
    pub fn na(linha: usize, motivo: impl Into<String>) -> Self {
        Self { linha, motivo: motivo.into() }
    }

    /// O que o modal mostra.
    pub fn mensagem(&self) -> String {
        if self.linha == 0 {
            self.motivo.clone()
        } else {
            format!("linha {}: {}", self.linha, self.motivo)
        }
    }
}

/// Onde a pessoa deixou cada nó no canvas, por id. O mermaid recalcula o
/// layout e ignora isto (RNF2): é só pra o canvas reabrir arrumado.
pub type Posicoes = BTreeMap<String, [f64; 2]>;

/// O comentário que guarda as [`Posicoes`]. `%%` é comentário no mermaid,
/// então o desenho não muda, e apagar a linha só perde a arrumação.
pub const MARCA_POSICOES: &str = "%% anotadinho-posicoes:";

/// Uma instrução do código: o mermaid separa por quebra de linha OU por
/// `;` (`graph TD; A-->B` é a forma que todo exemplo usa).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Instrucao {
    Texto { linha: usize, texto: String },
    Posicoes { linha: usize, posicoes: Posicoes },
}

impl Instrucao {
    pub(crate) fn linha(&self) -> usize {
        match self {
            Instrucao::Texto { linha, .. } | Instrucao::Posicoes { linha, .. } => *linha,
        }
    }
}

pub(crate) fn instrucoes(codigo: &str) -> Result<Vec<Instrucao>, ForaDoModelo> {
    let mut fora = Vec::new();
    for (i, bruta) in codigo.lines().enumerate() {
        let linha = i + 1;
        let t = bruta.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(json) = t.strip_prefix(MARCA_POSICOES) {
            let posicoes: Posicoes = serde_json::from_str(json.trim())
                .map_err(|_| ForaDoModelo::na(linha, "as posições do construtor estão ilegíveis"))?;
            fora.push(Instrucao::Posicoes { linha, posicoes });
            continue;
        }
        if t.starts_with("%%{") {
            return Err(ForaDoModelo::na(linha, "diretiva `%%{…}%%` (configuração do mermaid)"));
        }
        if t.starts_with("%%") {
            // Reescrever perderia o comentário: melhor abrir no código.
            return Err(ForaDoModelo::na(linha, "comentário — o construtor não guarda comentários"));
        }
        if t == "---" {
            return Err(ForaDoModelo::na(linha, "cabeçalho `---` (configuração do mermaid)"));
        }
        for parte in separar_por_ponto_e_virgula(t) {
            let parte = parte.trim();
            if !parte.is_empty() {
                fora.push(Instrucao::Texto { linha, texto: parte.to_string() });
            }
        }
    }
    Ok(fora)
}

/// `;` dentro de `[…]`, `(…)`, `{…}`, `|…|` ou aspas é texto, não separador.
fn separar_por_ponto_e_virgula(linha: &str) -> Vec<&str> {
    let mut partes = Vec::new();
    let (mut nivel, mut aspas, mut barra, mut ini) = (0i32, false, false, 0);
    for (i, c) in linha.char_indices() {
        match c {
            '"' => aspas = !aspas,
            _ if aspas => {}
            '[' | '(' | '{' => nivel += 1,
            // Sem descer de zero: a seta assíncrona da sequência (`-)`)
            // fecha um parêntese que nunca abriu.
            ']' | ')' | '}' => nivel = (nivel - 1).max(0),
            '|' => barra = !barra,
            // `#59;`, `#quot;`: o `;` fecha a entidade, não a instrução.
            ';' if nivel == 0 && !barra && !fecha_entidade(&linha[..i]) => {
                partes.push(&linha[ini..i]);
                ini = i + 1;
            }
            _ => {}
        }
    }
    partes.push(&linha[ini..]);
    partes
}

fn fecha_entidade(antes: &str) -> bool {
    match antes.rfind('#') {
        Some(i) => {
            let nome = &antes[i + 1..];
            !nome.is_empty() && nome.chars().all(|c| c.is_ascii_alphanumeric())
        }
        None => false,
    }
}

/// Lê o código de um bloco mermaid.
pub fn ler(codigo: &str) -> Result<Diagrama, ForaDoModelo> {
    let todas = instrucoes(codigo)?;
    let Some(Instrucao::Texto { linha, texto }) = todas.first() else {
        return Err(ForaDoModelo::na(0, "o diagrama está vazio"));
    };
    let palavra = texto.split_whitespace().next().unwrap_or("");
    let resto = &todas[1..];
    match palavra {
        "graph" | "flowchart" => fluxograma::ler(*linha, texto, resto).map(Diagrama::Fluxograma),
        "sequenceDiagram" => sequencia::ler(*linha, texto, resto).map(Diagrama::Sequencia),
        "stateDiagram" | "stateDiagram-v2" => estados::ler(*linha, texto, resto).map(Diagrama::Estados),
        "gantt" => gantt::ler(*linha, texto, resto).map(Diagrama::Gantt),
        outro => Err(ForaDoModelo::na(
            *linha,
            format!("o tipo `{outro}` não é montado pelo construtor"),
        )),
    }
}

/// O código mermaid do diagrama, na forma canônica do construtor.
pub fn escrever(d: &Diagrama) -> String {
    match d {
        Diagrama::Fluxograma(f) => f.escrever(),
        Diagrama::Sequencia(s) => s.escrever(),
        Diagrama::Estados(e) => e.escrever(),
        Diagrama::Gantt(g) => g.escrever(),
    }
}

// ── comum aos tipos ────────────────────────────────────────────────

/// Id de nó, estado ou participante: letras, dígitos e `_`.
pub fn id_valido(id: &str) -> bool {
    !id.is_empty()
        && id.chars().all(|c| c.is_alphanumeric() || c == '_')
        // `end` encerra bloco no mermaid: como id, quebra o diagrama todo.
        && !id.eq_ignore_ascii_case("end")
}

/// O prefixo de `s` que é id, e o resto.
pub(crate) fn tomar_id(s: &str) -> (&str, &str) {
    let fim = s
        .char_indices()
        .find(|(_, c)| !(c.is_alphanumeric() || *c == '_'))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    (&s[..fim], &s[fim..])
}

/// Texto do modelo → texto no código. O mermaid lê `;` como fim de
/// instrução e `"` como aspas; as entidades dele são o escape.
pub(crate) fn escapar(texto: &str) -> String {
    // `;` primeiro: as entidades também terminam em `;`.
    texto.replace(';', "#59;").replace('"', "#quot;")
}

pub(crate) fn desescapar(texto: &str) -> String {
    texto.replace("#quot;", "\"").replace("#59;", ";").replace("#58;", ":")
}

pub(crate) fn linha_de_posicoes(p: &Posicoes) -> Option<String> {
    if p.is_empty() {
        return None;
    }
    let json = serde_json::to_string(p).ok()?;
    Some(format!("  {MARCA_POSICOES} {json}"))
}

/// As posições de um diagrama que não tem canvas: é sinal de código
/// editado à mão misturando tipos, e reescrever perderia a linha.
pub(crate) fn sem_posicoes(resto: &[Instrucao]) -> Result<(), ForaDoModelo> {
    match resto.iter().find(|i| matches!(i, Instrucao::Posicoes { .. })) {
        Some(i) => Err(ForaDoModelo::na(i.linha(), "posições do construtor num diagrama sem canvas")),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipo_desconhecido_diz_qual_e() {
        let e = ler("classDiagram\n  A <|-- B").unwrap_err();
        assert_eq!(e.linha, 1);
        assert!(e.motivo.contains("classDiagram"), "{e}");
    }

    #[test]
    fn vazio_comentario_e_diretiva_ficam_fora() {
        assert_eq!(ler("\n  \n").unwrap_err().linha, 0);
        assert_eq!(ler("graph TD\n  %% nota\n  A").unwrap_err().linha, 2);
        assert_eq!(ler("%%{init: {}}%%\ngraph TD\n  A").unwrap_err().linha, 1);
    }

    #[test]
    fn ponto_e_virgula_separa_fora_do_texto() {
        let v = separar_por_ponto_e_virgula("graph TD; A[a;b] --> B; C|x;y|");
        assert_eq!(v, vec!["graph TD", " A[a;b] --> B", " C|x;y|"]);
        let v = separar_por_ponto_e_virgula("A->>B: x#59; y#quot;; B->>A: z");
        assert_eq!(v, vec!["A->>B: x#59; y#quot;", " B->>A: z"]);
    }

    #[test]
    fn escape_vai_e_volta() {
        let t = r#"diz "oi"; e: tchau"#;
        assert_eq!(desescapar(&escapar(t)), t);
        // O `;` que sobra é o fim das entidades: não separa instrução.
        assert_eq!(separar_por_ponto_e_virgula(&escapar(t)).len(), 1);
        assert!(!escapar(t).contains('"'));
    }

    #[test]
    fn exemplo_de_cada_tipo_e_do_tipo_e_volta_igual() {
        for t in Tipo::TODOS {
            let d = Diagrama::exemplo(t);
            assert_eq!(d.tipo(), t);
            assert_eq!(ler(&escrever(&d)).unwrap(), d);
        }
    }

    /// Amostras pro cenário de UI que as passa pelo `mermaid.parse` de
    /// VERDADE (ciclo 488): os testes daqui só provam que o leitor
    /// entende o escritor, não que o mermaid entende.
    ///
    /// `ANOTADINHO_AMOSTRAS=/caminho.json cargo test -p anotadinho-core
    /// --lib diagrama::tests::amostras -- --ignored`
    #[test]
    #[ignore]
    fn amostras_para_o_mermaid() {
        let destino = std::env::var("ANOTADINHO_AMOSTRAS").expect("ANOTADINHO_AMOSTRAS");
        let mut codigos: Vec<String> = Tipo::TODOS.iter().map(|t| escrever(&Diagrama::exemplo(*t))).collect();
        codigos.extend(propriedade::amostras(40));
        let saida = serde_json::json!({ "codigos": codigos, "rotulos": rotulos_para_o_desenho() });
        std::fs::write(destino, saida.to_string()).unwrap();
    }

    /// Um diagrama por LUGAR onde o construtor escreve texto, com um texto
    /// difícil, e o texto que o desenho tem que mostrar. Passar no
    /// `mermaid.parse` não basta: no nome de estado entre aspas, `#58;`
    /// passava no parse e aparecia escrito no desenho (ciclo 490).
    fn rotulos_para_o_desenho() -> Vec<serde_json::Value> {
        use estados::{Estado, Estados, Ponta, Transicao};
        use fluxograma::{Fluxograma, Forma, Ligacao, No, TipoLigacao};
        use gantt::{Fim, Gantt, Inicio, Secao, Tarefa};
        use sequencia::{Item, Lado, Participante, Seta, Sequencia};
        const T: &str = "a: b; (c) [d] {e} é";
        const T_ASPAS: &str = "a: b; \"c\" é";
        let mut casos: Vec<(Diagrama, &str)> = Vec::new();
        for forma in Forma::TODAS {
            let mut f = Fluxograma::novo();
            f.nos.push(No { id: "A".into(), texto: T_ASPAS.into(), forma });
            f.nos.push(No { id: "B".into(), texto: "B".into(), forma: Forma::Caixa });
            f.ligacoes.push(Ligacao { de: "A".into(), para: "B".into(), tipo: TipoLigacao::Seta, rotulo: T.into() });
            // No nó, `"` vira `'` (ver `fluxograma::texto_de_no`).
            casos.push((Diagrama::Fluxograma(f.clone()), "a: b; 'c' é"));
            casos.push((Diagrama::Fluxograma(f), T));
        }
        let seq = Sequencia {
            participantes: vec![
                Participante { id: "P1".into(), nome: T_ASPAS.into(), ator: false },
                Participante { id: "P2".into(), nome: "Outro".into(), ator: true },
            ],
            itens: vec![
                Item::Mensagem { de: "P1".into(), para: "P2".into(), seta: Seta::Sincrona, texto: "m: x; \"y\"".into() },
                Item::Nota { lado: Lado::Sobre, alvos: vec!["P1".into(), "P2".into()], texto: "n: x; \"y\"".into() },
            ],
        };
        for t in [T_ASPAS, "m: x; \"y\"", "n: x; \"y\""] {
            casos.push((Diagrama::Sequencia(seq.clone()), t));
        }
        let mut est = Estados::novo();
        est.estados.push(Estado { id: "E1".into(), texto: estados::nome_no_codigo(T) });
        est.transicoes.push(Transicao { de: Ponta::Inicio, para: Ponta::Estado("E1".into()), rotulo: T_ASPAS.into() });
        casos.push((Diagrama::Estados(est.clone()), T));
        casos.push((Diagrama::Estados(est), T_ASPAS));
        let g = Gantt {
            titulo: Some("t: x; \"y\"".into()),
            com_formato: true,
            secoes: vec![Secao {
                nome: Some("s: x; \"y\"".into()),
                tarefas: vec![Tarefa { nome: T_ASPAS.into(), marcas: vec![], id: None, inicio: Inicio::Data("2026-01-05".into()), fim: Fim::Duracao("3d".into()) }],
            }],
        };
        for t in ["t: x; \"y\"", "s: x; \"y\"", T_ASPAS] {
            casos.push((Diagrama::Gantt(g.clone()), t));
        }
        casos.into_iter().map(|(d, t)| serde_json::json!({ "codigo": escrever(&d), "texto": t })).collect()
    }

    #[test]
    fn end_nao_e_id() {
        assert!(id_valido("A_1") && id_valido("Começo"));
        assert!(!id_valido("end") && !id_valido("a b") && !id_valido(""));
    }
}

#[cfg(test)]
mod propriedade {
    //! `ler(escrever(d)) == d` para diagramas gerados: o que o construtor
    //! produzir tem que voltar igual, com qualquer texto que a pessoa digite.

    use super::*;

    /// Gerador determinístico (LCG): sem dependência nova, e a falha se
    /// reproduz pela semente.
    struct Lcg(u64);
    impl Lcg {
        fn n(&mut self, max: usize) -> usize {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((self.0 >> 33) as usize) % max.max(1)
        }
        fn texto(&mut self) -> String {
            const PEDACOS: [&str; 14] = ["oi", "ação", " ", ";", "\"", ":", "(x)", "[y]", "{z}", "a|b", "#", "->", "é", "1"];
            let s: String = (0..1 + self.n(5)).map(|_| PEDACOS[self.n(PEDACOS.len())]).collect();
            // O construtor apara as bordas do texto; o leitor também.
            let s = s.trim().to_string();
            if s.is_empty() { "t".into() } else { s }
        }
    }

    fn gerar_fluxograma(semente: u64) -> Diagrama {
        use fluxograma::*;
        let mut r = Lcg(semente);
        let mut f = Fluxograma::novo();
        f.direcao = Direcao::TODAS[r.n(5)];
        let n = 1 + r.n(6);
        for i in 0..n {
            // `"` não entra no texto do nó (vira `'`, ver
            // `fluxograma::texto_de_no`): o gerador não o produz.
            f.nos.push(No { id: format!("n{i}"), texto: r.texto().replace('"', "'"), forma: Forma::TODAS[r.n(4)] });
        }
        for _ in 0..r.n(8) {
            let rotulo = if r.n(2) == 0 { String::new() } else { r.texto() };
            f.ligacoes.push(Ligacao { de: format!("n{}", r.n(n)), para: format!("n{}", r.n(n)), tipo: TipoLigacao::TODAS[r.n(4)], rotulo });
        }
        if r.n(2) == 0 {
            f.posicoes.insert("n0".into(), [r.n(500) as f64, r.n(500) as f64 + 0.5]);
        }
        Diagrama::Fluxograma(f)
    }

    fn gerar_sequencia(semente: u64) -> Diagrama {
        use sequencia::*;
        let mut r = Lcg(semente);
        let mut s = Sequencia::default();
        let n = 1 + r.n(4);
        for i in 0..n {
            let nome = if r.n(2) == 0 { format!("p{i}") } else { r.texto() };
            s.participantes.push(Participante { id: format!("p{i}"), nome, ator: r.n(2) == 0 });
        }
        for _ in 0..r.n(8) {
            let item = if r.n(3) == 0 {
                let lado = [Lado::Direita, Lado::Esquerda, Lado::Sobre][r.n(3)];
                let mut alvos = vec![format!("p{}", r.n(n))];
                if lado == Lado::Sobre && r.n(2) == 0 {
                    alvos.push(format!("p{}", r.n(n)));
                }
                Item::Nota { lado, alvos, texto: r.texto() }
            } else {
                Item::Mensagem { de: format!("p{}", r.n(n)), para: format!("p{}", r.n(n)), seta: Seta::TODAS[r.n(8)], texto: r.texto() }
            };
            s.itens.push(item);
        }
        Diagrama::Sequencia(s)
    }

    fn gerar_estados(semente: u64) -> Diagrama {
        use estados::*;
        let mut r = Lcg(semente);
        let mut e = Estados::novo();
        let n = 1 + r.n(5);
        for i in 0..n {
            // `"` não entra no nome de estado (vira `'`, ver
            // `estados::nome_no_codigo`): o gerador não o produz.
            let texto = if r.n(2) == 0 { format!("s{i}") } else { estados::nome_no_codigo(&r.texto()) };
            e.estados.push(Estado { id: format!("s{i}"), texto });
        }
        for _ in 0..r.n(8) {
            let de = if r.n(4) == 0 { Ponta::Inicio } else { Ponta::Estado(format!("s{}", r.n(n))) };
            let para = if r.n(4) == 0 { Ponta::Fim } else { Ponta::Estado(format!("s{}", r.n(n))) };
            // Início direto no fim não é transição que o construtor ofereça.
            if de == Ponta::Inicio && para == Ponta::Fim {
                continue;
            }
            let rotulo = if r.n(2) == 0 { String::new() } else { r.texto() };
            e.transicoes.push(Transicao { de, para, rotulo });
        }
        Diagrama::Estados(e)
    }

    fn gerar_gantt(semente: u64) -> Diagrama {
        use gantt::*;
        let mut r = Lcg(semente);
        let mut g = Gantt { titulo: (r.n(2) == 0).then(|| r.texto()), com_formato: r.n(2) == 0, secoes: Vec::new() };
        let mut ids: Vec<String> = Vec::new();
        for s in 0..1 + r.n(3) {
            let nome = if s == 0 && r.n(2) == 0 { None } else { Some(r.texto()) };
            let mut tarefas = Vec::new();
            for _ in 0..r.n(4) {
                let id = (r.n(2) == 0).then(|| format!("t{}", ids.len()));
                let inicio = match r.n(3) {
                    0 => Inicio::Anterior,
                    1 => Inicio::Data(format!("2026-{:02}-{:02}", 1 + r.n(12), 1 + r.n(28))),
                    _ if !ids.is_empty() => Inicio::Depois(vec![ids[r.n(ids.len())].clone()]),
                    _ => Inicio::Anterior,
                };
                // Sem início escrito, o `id` sozinho seria lido como
                // início: o construtor não gera essa combinação.
                let id = if inicio == Inicio::Anterior { None } else { id };
                let fim = if r.n(2) == 0 { Fim::Duracao(format!("{}d", 1 + r.n(9))) } else { Fim::Data("2026-12-31".into()) };
                let marcas = Marca::TODAS.into_iter().filter(|_| r.n(3) == 0).collect();
                if let Some(i) = &id {
                    ids.push(i.clone());
                }
                tarefas.push(Tarefa { nome: r.texto(), marcas, id, inicio, fim });
            }
            // Uma primeira seção sem nome e sem tarefa não tem como ser
            // escrita: o construtor não a cria.
            if nome.is_some() || !tarefas.is_empty() {
                g.secoes.push(Secao { nome, tarefas });
            }
        }
        Diagrama::Gantt(g)
    }

    const GERADORES: [fn(u64) -> Diagrama; 4] = [gerar_fluxograma, gerar_sequencia, gerar_estados, gerar_gantt];

    /// O código de `n` diagramas gerados de cada tipo.
    pub(super) fn amostras(n: u64) -> Vec<String> {
        GERADORES.iter().flat_map(|g| (0..n).map(move |s| escrever(&g(s)))).collect()
    }

    fn ida_e_volta(gerar: fn(u64) -> Diagrama) {
        for semente in 0..300 {
            let d = gerar(semente);
            let codigo = escrever(&d);
            assert_eq!(ler(&codigo).as_ref(), Ok(&d), "semente {semente}\n{codigo}");
        }
    }

    #[test]
    fn fluxograma_gerado_volta_igual() {
        ida_e_volta(gerar_fluxograma);
    }

    #[test]
    fn sequencia_gerada_volta_igual() {
        ida_e_volta(gerar_sequencia);
    }

    #[test]
    fn estados_gerado_volta_igual() {
        ida_e_volta(gerar_estados);
    }

    #[test]
    fn gantt_gerado_volta_igual() {
        ida_e_volta(gerar_gantt);
    }
}
