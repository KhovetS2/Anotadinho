//! Extração de alvos de `[[wikilink]]` fora do WASM.
//!
//! A UI tem o parser dela (`ui/src/wikilink.rs`), que trabalha com
//! POSIÇÕES no texto pra linkificar o markdown na hora de renderizar.
//! Aqui o problema é outro: só a LISTA de alvos, pro grafo de backlinks
//! e pra varredura do vault (`crate::index`) — que rodam no backend, um
//! por página, sem DOM nenhum por perto.
//!
//! Blocos de código cercados (```` ``` ````/`~~~`) são ignorados: um
//! `[[exemplo]]` dentro de um trecho de código é texto, não link.

/// Todos os `[[alvo]]` do texto, na ordem em que aparecem, COM
/// duplicatas e sem tratar alias/âncora — o alvo cru, como escrito.
/// Serve pra contagem de referências (o grafo pesa a aresta pelo número
/// de menções).
pub fn extract_wikilink_raw(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        extract_line(line, &mut out);
    }
    out
}

/// Parte o miolo de um `[[...]]` em (alvo, texto exibido).
///
/// O separador é a barra vertical, e `\|` escapa uma barra LITERAL —
/// `|` é caractere válido em nome de arquivo no POSIX (só o Windows
/// proíbe), então um vault criado no Linux pode ter
/// `estranho|nome.md` de verdade. Sem o escape não haveria como
/// referenciar esse arquivo (ciclo 192).
///
/// Só a PRIMEIRA barra não escapada separa: `[[a|b|c]]` vira alvo `a`
/// com texto `b|c`, e não um terceiro campo — texto exibido pode conter
/// barra sem precisar escapar.
pub fn split_wikilink(raw: &str) -> (String, Option<String>) {
    let bytes = raw.as_bytes();
    let mut alvo = String::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if bytes[i] == b'\\' && i + 1 < raw.len() && bytes[i + 1] == b'|' {
            alvo.push('|');
            i += 2;
            continue;
        }
        if bytes[i] == b'|' {
            let texto = desescapar_barra(&raw[i + 1..]);
            let texto = texto.trim().to_string();
            return (
                alvo.trim().to_string(),
                if texto.is_empty() { None } else { Some(texto) },
            );
        }
        let ch = raw[i..].chars().next().unwrap_or('\0');
        alvo.push(ch);
        i += ch.len_utf8();
    }
    (alvo.trim().to_string(), None)
}

/// Troca `\|` por `|`.
fn desescapar_barra(s: &str) -> String {
    s.replace("\\|", "|")
}

/// Escapa as barras de um alvo pra ele poder ser escrito dentro de
/// `[[...]]` sem virar alias. Usado por quem GERA wikilink (autocompletar,
/// "copiar referência").
pub fn escapar_barra(alvo: &str) -> String {
    alvo.replace('|', "\\|")
}

/// Alvos únicos, com alias (`[[Página|texto]]`) e âncora
/// (`[[Página#seção]]`) recortados — o que sobra é o nome da página
/// referenciada, que é o que o grafo e a varredura precisam pra casar
/// com o título de uma página de verdade.
pub fn extract_wikilink_targets(markdown: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in extract_wikilink_raw(markdown) {
        let (alvo, _) = split_wikilink(&raw);
        let target = alvo.split('#').next().unwrap_or("").trim().to_string();
        if !target.is_empty() && !out.contains(&target) {
            out.push(target);
        }
    }
    out
}

/// Alvos de TRANSCLUSÃO (`![[Página]]`), únicos, com alias/âncora
/// preservados no formato `Página#Seção` (ciclo 170) — diferente do
/// wikilink, aqui a âncora importa: ela escolhe QUE PEDAÇO da página
/// entra.
pub fn extract_transclusion_targets(markdown: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_fence = false;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let mut i = 0;
        while let Some(pos) = line[i..].find("![[") {
            let inicio = i + pos + 3;
            let Some(fim_rel) = line[inicio..].find("]]") else { break };
            let alvo = line[inicio..inicio + fim_rel].trim().to_string();
            if !alvo.is_empty() && !alvo.contains('[') && !out.contains(&alvo) {
                out.push(alvo);
            }
            i = inicio + fim_rel + 2;
        }
    }
    out
}

/// Recorta a seção de um corpo markdown a partir do título dela, até o
/// próximo heading de nível igual ou superior (ciclo 170).
///
/// `None` se não existir heading com esse texto.
pub fn extract_section<'a>(body: &'a str, heading: &str) -> Option<&'a str> {
    let alvo = heading.trim().to_lowercase();
    let mut inicio: Option<usize> = None;
    let mut nivel = 0usize;
    let mut pos = 0usize;
    for linha in body.split_inclusive('\n') {
        let comeco = pos;
        pos += linha.len();
        let t = linha.trim();
        if !t.starts_with('#') {
            continue;
        }
        let n = t.chars().take_while(|c| *c == '#').count();
        let texto = t[n..].trim().to_lowercase();
        match inicio {
            None => {
                if texto == alvo {
                    inicio = Some(comeco);
                    nivel = n;
                }
            }
            Some(i) => {
                if n <= nivel {
                    return Some(&body[i..comeco]);
                }
            }
        }
    }
    inicio.map(|i| &body[i..])
}

/// Sufixo de identificação de bloco (ciclo 176): `^abc123` no fim da
/// linha. Mesma convenção do Obsidian — a mais compatível com vault
/// existente e a que menos atrapalha a leitura do `.md` fora do app.
pub const PREFIXO_ID: char = '^';

/// Id de bloco no fim da linha, se houver (`texto ^abc123` → `abc123`).
///
/// Aceita só `[a-z0-9-]` depois do `^`, pra não confundir com um `^`
/// legítimo no meio do texto (potência, apontar pra cima etc).
pub fn extract_block_id(linha: &str) -> Option<&str> {
    let t = linha.trim_end();
    let pos = t.rfind(PREFIXO_ID)?;
    // Precisa ter espaço antes: `x^2` não é id de bloco.
    if pos == 0 || !t[..pos].ends_with(char::is_whitespace) {
        return None;
    }
    let id = &t[pos + 1..];
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return None;
    }
    Some(id)
}

/// Texto do bloco sem o id — o que é mostrado pro leitor.
pub fn strip_block_id(linha: &str) -> &str {
    match extract_block_id(linha) {
        Some(id) => linha[..linha.rfind(id).unwrap_or(linha.len()).saturating_sub(1)].trim_end(),
        None => linha,
    }
}

/// Acha o bloco de um id dentro do corpo, devolvendo a linha inteira
/// (sem o id).
pub fn find_block<'a>(body: &'a str, id: &str) -> Option<&'a str> {
    body.lines()
        .find(|l| extract_block_id(l) == Some(id))
        .map(strip_block_id)
}

/// Gera um id curto e estável o bastante pro uso (6 chars base36) a
/// partir do conteúdo da linha + um contador de desempate.
///
/// Não é aleatório de propósito: o mesmo bloco gera o mesmo id numa
/// segunda tentativa, então referenciar duas vezes não polui o arquivo
/// com ids diferentes.
pub fn gerar_block_id(conteudo: &str, tentativa: u32) -> String {
    // Hash FNV-1a: 30 linhas de dependência a menos que trazer um crate
    // de hash só pra isso, e id de bloco não precisa de resistência
    // criptográfica.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in conteudo.as_bytes().iter().chain(tentativa.to_le_bytes().iter()) {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    const ALFABETO: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut id = String::with_capacity(6);
    for _ in 0..6 {
        id.push(ALFABETO[(hash % ALFABETO.len() as u64) as usize] as char);
        hash /= ALFABETO.len() as u64;
    }
    id
}

/// Garante que a linha `alvo` (índice 0-based entre as linhas do corpo)
/// tenha um id, devolvendo `(corpo novo, id)`.
///
/// Se já tiver, devolve o corpo INTOCADO — este é o ponto central do
/// ciclo 176: id só entra no bloco que alguém referenciou, e só uma vez.
pub fn garantir_block_id(body: &str, alvo: usize) -> Option<(String, String)> {
    let linhas: Vec<&str> = body.split_inclusive('\n').collect();
    let linha = linhas.get(alvo)?;
    let sem_quebra = linha.trim_end_matches('\n');
    if let Some(id) = extract_block_id(sem_quebra) {
        return Some((body.to_string(), id.to_string()));
    }
    if sem_quebra.trim().is_empty() {
        return None;
    }

    // Desempata contra ids já usados no documento.
    let usados: Vec<&str> = body.lines().filter_map(extract_block_id).collect();
    let mut tentativa = 0;
    let id = loop {
        let candidato = gerar_block_id(sem_quebra, tentativa);
        if !usados.contains(&candidato.as_str()) {
            break candidato;
        }
        tentativa += 1;
    };

    let mut novo = String::with_capacity(body.len() + id.len() + 2);
    for (i, l) in linhas.iter().enumerate() {
        if i == alvo {
            let sem = l.trim_end_matches('\n');
            novo.push_str(sem);
            novo.push(' ');
            novo.push(PREFIXO_ID);
            novo.push_str(&id);
            if l.ends_with('\n') {
                novo.push('\n');
            }
        } else {
            novo.push_str(l);
        }
    }
    Some((novo, id))
}

/// Varre uma linha atrás de `[[...]]`. Um par com `[` ou `]` no miolo é
/// ignorado (markdown de link normal aninhado, `[[a](b)]`), mesma regra
/// do parser da UI.
fn extract_line(line: &str, out: &mut Vec<String>) {
    let mut i = 0;
    while i < line.len() {
        if line[i..].starts_with("[[") {
            if let Some(rel_end) = line[i + 2..].find("]]") {
                let title = &line[i + 2..i + 2 + rel_end];
                if !title.is_empty() && !title.contains('[') && !title.contains(']') {
                    out.push(title.to_string());
                    i = i + 2 + rel_end + 2;
                    continue;
                }
            }
        }
        let ch = line[i..].chars().next().unwrap_or('\0');
        i += ch.len_utf8();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── alias e barra literal (ciclo 192) ─────────────────────────

    #[test]
    fn split_sem_barra_nao_tem_alias() {
        assert_eq!(split_wikilink("Grafo do Vault"), ("Grafo do Vault".into(), None));
    }

    #[test]
    fn split_separa_alvo_e_texto() {
        assert_eq!(
            split_wikilink("pages/produto/grafo.md|Grafo do Vault"),
            ("pages/produto/grafo.md".into(), Some("Grafo do Vault".into()))
        );
    }

    #[test]
    fn barra_escapada_faz_parte_do_alvo() {
        // Arquivo chamado `estranho|nome` — legal no POSIX.
        assert_eq!(split_wikilink(r"estranho\|nome"), ("estranho|nome".into(), None));
    }

    #[test]
    fn so_a_primeira_barra_separa() {
        assert_eq!(
            split_wikilink("alvo|texto|com|barra"),
            ("alvo".into(), Some("texto|com|barra".into()))
        );
    }

    #[test]
    fn alvo_escapado_com_alias_depois() {
        assert_eq!(
            split_wikilink(r"estranho\|nome|o esquisito"),
            ("estranho|nome".into(), Some("o esquisito".into()))
        );
    }

    #[test]
    fn alias_vazio_e_tratado_como_ausente() {
        assert_eq!(split_wikilink("Alvo|"), ("Alvo".into(), None));
        assert_eq!(split_wikilink("Alvo|   "), ("Alvo".into(), None));
    }

    #[test]
    fn escapar_e_desfazer_round_trip() {
        let nome = "estranho|nome";
        let (volta, _) = split_wikilink(&escapar_barra(nome));
        assert_eq!(volta, nome);
    }

    #[test]
    fn alvos_extraidos_ignoram_o_alias() {
        let alvos = extract_wikilink_targets("veja [[Grafo do Vault|o grafo]] e [[Missão]]\n");
        assert_eq!(alvos, vec!["Grafo do Vault".to_string(), "Missão".to_string()]);
    }

    #[test]
    fn raw_encontra_varios_links_na_ordem() {
        assert_eq!(
            extract_wikilink_raw("[[A]] e [[B]]\ntexto [[C]]"),
            vec!["A", "B", "C"]
        );
    }

    #[test]
    fn raw_mantem_duplicatas() {
        assert_eq!(extract_wikilink_raw("[[A]] [[A]]"), vec!["A", "A"]);
    }

    #[test]
    fn raw_ignora_bloco_de_codigo_cercado() {
        let md = "[[Sim]]\n```\n[[Nao]]\n```\n~~~\n[[TambemNao]]\n~~~";
        assert_eq!(extract_wikilink_raw(md), vec!["Sim"]);
    }

    #[test]
    fn targets_removem_duplicata_alias_e_ancora() {
        let md = "[[Roadmap|o mapa]] [[Roadmap#backlog]] [[Missão]]";
        assert_eq!(extract_wikilink_targets(md), vec!["Roadmap", "Missão"]);
    }

    #[test]
    fn targets_vazio_sem_link() {
        assert!(extract_wikilink_targets("nada aqui [ ] [x]").is_empty());
        assert!(extract_wikilink_targets("[[]]").is_empty());
    }

    #[test]
    fn transclusao_e_reconhecida_e_separada_do_wikilink() {
        let md = "texto [[Link normal]]\n\n![[Missão]]\n![[Guia#Fluxo]]\n";
        assert_eq!(
            extract_transclusion_targets(md),
            vec!["Missão", "Guia#Fluxo"],
            "só o `![[..]]` conta como transclusão"
        );
        // O wikilink comum continua sendo visto como link (o `!` faz
        // parte do texto anterior, não some da varredura de links).
        assert!(extract_wikilink_targets(md).contains(&"Link normal".to_string()));
    }

    #[test]
    fn transclusao_dentro_de_fence_e_ignorada() {
        let md = "![[Vale]]\n```\n![[NaoVale]]\n```\n";
        assert_eq!(extract_transclusion_targets(md), vec!["Vale"]);
    }

    #[test]
    fn extrai_secao_ate_o_proximo_heading_do_mesmo_nivel() {
        let body = "# Um\ntexto um\n\n## Dois\ntexto dois\n\n### Tres\ntexto tres\n\n## Quatro\ntexto quatro\n";
        let dois = extract_section(body, "Dois").unwrap();
        assert!(dois.contains("texto dois"));
        assert!(dois.contains("texto tres"), "sub-seção faz parte da seção");
        assert!(!dois.contains("texto quatro"), "parou no próximo heading do mesmo nível");
        assert!(!dois.contains("texto um"));
    }

    #[test]
    fn secao_inexistente_devolve_none() {
        assert!(extract_section("# Um\ntexto\n", "Outro").is_none());
    }

    #[test]
    fn le_id_de_bloco_no_fim_da_linha() {
        assert_eq!(extract_block_id("texto qualquer ^abc123"), Some("abc123"));
        assert_eq!(extract_block_id("texto qualquer ^abc123\n"), Some("abc123"));
        assert_eq!(strip_block_id("texto qualquer ^abc123"), "texto qualquer");
    }

    #[test]
    fn circunflexo_no_meio_do_texto_nao_e_id() {
        assert_eq!(extract_block_id("x^2 é o quadrado"), None);
        assert_eq!(extract_block_id("sem id nenhum"), None);
        assert_eq!(extract_block_id("maiúsculas ^ABC"), None, "id é minúsculo");
    }

    #[test]
    fn garantir_id_escreve_so_na_linha_pedida() {
        let body = "primeira linha\nsegunda linha\nterceira linha\n";
        let (novo, id) = garantir_block_id(body, 1).unwrap();
        assert!(novo.contains(&format!("segunda linha ^{id}")), "{novo}");
        assert!(novo.contains("primeira linha\n"), "outras linhas não podem ganhar id");
        assert_eq!(novo.lines().filter(|l| extract_block_id(l).is_some()).count(), 1);
    }

    #[test]
    fn garantir_id_e_idempotente() {
        let body = "linha\n";
        let (uma, id1) = garantir_block_id(body, 0).unwrap();
        let (duas, id2) = garantir_block_id(&uma, 0).unwrap();
        assert_eq!(id1, id2, "referenciar duas vezes não pode gerar id novo");
        assert_eq!(uma, duas, "o arquivo não pode mudar na segunda vez");
    }

    #[test]
    fn garantir_id_ignora_linha_vazia() {
        assert!(garantir_block_id("texto\n\noutro\n", 1).is_none());
    }

    #[test]
    fn ids_colidindo_no_mesmo_documento_sao_desempatados() {
        // Duas linhas com o MESMO texto gerariam o mesmo hash.
        let body = "igual\nigual\n";
        let (um, id1) = garantir_block_id(body, 0).unwrap();
        let (_dois, id2) = garantir_block_id(&um, 1).unwrap();
        assert_ne!(id1, id2, "o segundo bloco precisa de id próprio");
    }

    #[test]
    fn acha_bloco_pelo_id() {
        let body = "um\ndois ^alvo1\ntres\n";
        assert_eq!(find_block(body, "alvo1"), Some("dois"));
        assert_eq!(find_block(body, "naoexiste"), None);
    }

    #[test]
    fn colchete_aninhado_nao_vira_alvo() {
        assert!(extract_wikilink_targets("[[a](b)]").is_empty());
    }
}
