//! O vocabulário do vim: contagem, operador e movimento (ciclo 254).
//!
//! Até aqui cada tecla do modo normal era um `else if` que fazia uma
//! coisa. Isso comporta `j` e `x`, e não comporta `3j`, `dw` ou `d3w` —
//! porque nesses a tecla não é o comando, é uma PARTE dele.
//!
//! Aqui mora a gramática, e só ela: entra tecla, sai comando ou "ainda
//! não terminou". Nada de DOM, então dá pra testar de verdade — que é o
//! oposto do que dava pra fazer com a cadeia de `else if`.
//!
//! A gramática do vim, reduzida ao que se usa todo dia:
//!
//! ```text
//! comando := [contagem] (operador [contagem] movimento | operador operador | ação)
//! ```
//!
//! Fora de escopo por decisão da spec: macros, registradores nomeados e
//! `.` (repetir). O alvo é o vocabulário do dia a dia, não emular o vim.

/// Para onde o cursor vai — ou, com um operador na frente, até onde ele
/// age.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Movimento {
    Esquerda,
    Direita,
    Cima,
    Baixo,
    PalavraFrente,
    PalavraTras,
    FimDaPalavra,
    InicioDaLinha,
    FimDaLinha,
    InicioDoDocumento,
    FimDoDocumento,
    /// A linha inteira (o que `dd`, `yy` e `cc` significam).
    LinhaInteira,
}

/// Onde o modo de inserção começa.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Insercao {
    Antes,
    Depois,
    InicioDaLinha,
    FimDaLinha,
    LinhaAbaixo,
    LinhaAcima,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Comando {
    Mover(Movimento, u32),
    Apagar(Movimento, u32),
    Copiar(Movimento, u32),
    /// Apagar e entrar em inserção.
    Mudar(Movimento, u32),
    ApagarCaractere { antes: bool, vezes: u32 },
    Colar { antes: bool },
    Entrar(Insercao),
    Substituir(char),
    JuntarLinhas,
    TrocarCaixa,
    Desfazer,
    Refazer,
    Busca,
    Visual,
    VisualLinha,
    VisualBloco,
}

/// O que a máquina ainda espera pra fechar um comando.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct Pendente {
    contagem: Option<u32>,
    operador: Option<char>,
    /// Contagem digitada DEPOIS do operador (`d3w`): multiplica a de
    /// antes, como no vim — `2d3w` apaga seis palavras.
    contagem_do_operador: Option<u32>,
    /// `g` sozinho ainda não é nada; `gg` é ir pro topo.
    g_pendente: bool,
    /// `r` esperando o caractere de substituição.
    substituindo: bool,
}

impl Pendente {
    /// Há algo digitado pela metade? A barra de status usa pra mostrar.
    pub fn em_curso(&self) -> bool {
        self.contagem.is_some() || self.operador.is_some() || self.g_pendente || self.substituindo
    }

    /// Como o que está pendente aparece na barra (o "2d3" do canto).
    pub fn rotulo(&self) -> String {
        let mut s = String::new();
        if let Some(n) = self.contagem {
            s.push_str(&n.to_string());
        }
        if let Some(op) = self.operador {
            s.push(op);
        }
        if let Some(n) = self.contagem_do_operador {
            s.push_str(&n.to_string());
        }
        if self.g_pendente {
            s.push('g');
        }
        if self.substituindo {
            s.push('r');
        }
        s
    }

    fn total(&self) -> u32 {
        self.contagem.unwrap_or(1) * self.contagem_do_operador.unwrap_or(1)
    }

    fn limpar(&mut self) {
        *self = Pendente::default();
    }
}

/// O resultado de alimentar uma tecla na máquina.
#[derive(Clone, PartialEq, Debug)]
pub enum Passo {
    /// Comando fechado.
    Pronto(Comando),
    /// A tecla foi consumida, mas o comando ainda não fechou.
    Aguardando,
    /// A tecla não significa nada aqui — quem chamou decide.
    Ignorada,
}

/// Alimenta uma tecla do modo Normal.
///
/// `tecla` é `e.key()`; `ctrl` porque `Ctrl+R` é refazer e `Ctrl+V` é o
/// visual em bloco.
pub fn tecla_normal(p: &mut Pendente, tecla: &str, ctrl: bool) -> Passo {
    // `r` esperando o substituto vence tudo: depois dele QUALQUER tecla é
    // o caractere novo, inclusive um `d` ou um dígito.
    if p.substituindo {
        let c = tecla.chars().next();
        let unico = tecla.chars().count() == 1;
        p.limpar();
        return match c {
            Some(c) if unico => Passo::Pronto(Comando::Substituir(c)),
            // Escape (ou qualquer tecla nomeada) cancela, em vez de
            // escrever "Escape" no texto.
            _ => Passo::Aguardando,
        };
    }

    if ctrl {
        return match tecla {
            "r" | "R" => {
                p.limpar();
                Passo::Pronto(Comando::Refazer)
            }
            "v" | "V" => {
                p.limpar();
                Passo::Pronto(Comando::VisualBloco)
            }
            _ => Passo::Ignorada,
        };
    }

    // Dígitos viram contagem. `0` só conta quando JÁ há contagem — senão
    // ele é o movimento "início da linha".
    if let Some(d) = tecla.chars().next().filter(|c| c.is_ascii_digit()) {
        let d = d.to_digit(10).unwrap_or(0);
        let ja_conta = p.contagem.is_some() || p.contagem_do_operador.is_some();
        if !(d == 0 && !ja_conta) {
            let alvo = if p.operador.is_some() {
                &mut p.contagem_do_operador
            } else {
                &mut p.contagem
            };
            *alvo = Some(alvo.unwrap_or(0) * 10 + d);
            return Passo::Aguardando;
        }
    }

    // `g` sozinho fica pendente; `gg` fecha.
    if p.g_pendente {
        let vezes = p.total();
        let op = p.operador;
        p.limpar();
        if tecla == "g" {
            return Passo::Pronto(com_operador(op, Movimento::InicioDoDocumento, vezes));
        }
        return Passo::Ignorada;
    }
    if tecla == "g" {
        p.g_pendente = true;
        return Passo::Aguardando;
    }

    // Operador repetido age na linha inteira: `dd`, `yy`, `cc`.
    if let Some(op) = p.operador {
        if tecla.chars().count() == 1 && tecla.starts_with(op) {
            let vezes = p.total();
            p.limpar();
            return Passo::Pronto(com_operador(Some(op), Movimento::LinhaInteira, vezes));
        }
    }

    if let Some(mov) = movimento_de(tecla) {
        let vezes = p.total();
        let op = p.operador;
        p.limpar();
        return Passo::Pronto(com_operador(op, mov, vezes));
    }

    // Um operador novo só abre se não houver outro aberto — `dc` não é
    // comando nenhum, e engolir a tecla calado seria pior que cancelar.
    if matches!(tecla, "d" | "c" | "y") {
        if p.operador.is_some() {
            p.limpar();
            return Passo::Ignorada;
        }
        p.operador = tecla.chars().next();
        return Passo::Aguardando;
    }

    // Daqui pra baixo são ações que não aceitam operador na frente.
    let vezes = p.total();
    let tinha_operador = p.operador.is_some();
    p.limpar();
    if tinha_operador {
        return Passo::Ignorada;
    }

    let comando = match tecla {
        "x" => Comando::ApagarCaractere { antes: false, vezes },
        "X" => Comando::ApagarCaractere { antes: true, vezes },
        "D" => Comando::Apagar(Movimento::FimDaLinha, 1),
        "C" => Comando::Mudar(Movimento::FimDaLinha, 1),
        "Y" => Comando::Copiar(Movimento::LinhaInteira, vezes),
        "S" => Comando::Mudar(Movimento::LinhaInteira, vezes),
        "p" => Comando::Colar { antes: false },
        "P" => Comando::Colar { antes: true },
        "i" => Comando::Entrar(Insercao::Antes),
        "a" => Comando::Entrar(Insercao::Depois),
        "I" => Comando::Entrar(Insercao::InicioDaLinha),
        "A" => Comando::Entrar(Insercao::FimDaLinha),
        "o" => Comando::Entrar(Insercao::LinhaAbaixo),
        "O" => Comando::Entrar(Insercao::LinhaAcima),
        "r" => {
            p.substituindo = true;
            return Passo::Aguardando;
        }
        "J" => Comando::JuntarLinhas,
        "~" => Comando::TrocarCaixa,
        "u" => Comando::Desfazer,
        "/" => Comando::Busca,
        "v" => Comando::Visual,
        "V" => Comando::VisualLinha,
        _ => return Passo::Ignorada,
    };
    Passo::Pronto(comando)
}

fn com_operador(op: Option<char>, mov: Movimento, vezes: u32) -> Comando {
    match op {
        Some('d') => Comando::Apagar(mov, vezes),
        Some('y') => Comando::Copiar(mov, vezes),
        Some('c') => Comando::Mudar(mov, vezes),
        _ => Comando::Mover(mov, vezes),
    }
}

fn movimento_de(tecla: &str) -> Option<Movimento> {
    Some(match tecla {
        "h" | "ArrowLeft" => Movimento::Esquerda,
        "l" | "ArrowRight" => Movimento::Direita,
        "k" | "ArrowUp" => Movimento::Cima,
        "j" | "ArrowDown" => Movimento::Baixo,
        "w" => Movimento::PalavraFrente,
        "b" => Movimento::PalavraTras,
        "e" => Movimento::FimDaPalavra,
        "0" | "^" | "Home" => Movimento::InicioDaLinha,
        "$" | "End" => Movimento::FimDaLinha,
        "G" => Movimento::FimDoDocumento,
        _ => return None,
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    fn passo(p: &mut Pendente, teclas: &[&str]) -> Passo {
        let mut ultimo = Passo::Ignorada;
        for t in teclas {
            ultimo = tecla_normal(p, t, false);
        }
        ultimo
    }

    #[test]
    fn movimento_simples() {
        let mut p = Pendente::default();
        assert_eq!(
            passo(&mut p, &["j"]),
            Passo::Pronto(Comando::Mover(Movimento::Baixo, 1))
        );
        assert!(!p.em_curso(), "a máquina tem que voltar pro zero");
    }

    #[test]
    fn contagem_multiplica_o_movimento() {
        let mut p = Pendente::default();
        assert_eq!(passo(&mut p, &["3"]), Passo::Aguardando);
        assert_eq!(
            passo(&mut p, &["j"]),
            Passo::Pronto(Comando::Mover(Movimento::Baixo, 3))
        );
    }

    #[test]
    fn contagem_de_varios_digitos() {
        let mut p = Pendente::default();
        assert_eq!(
            passo(&mut p, &["1", "2", "j"]),
            Passo::Pronto(Comando::Mover(Movimento::Baixo, 12))
        );
    }

    #[test]
    fn zero_sozinho_e_movimento_nao_contagem() {
        // O caso que quase todo clone de vim erra: `0` só é dígito
        // quando já existe contagem.
        let mut p = Pendente::default();
        assert_eq!(
            passo(&mut p, &["0"]),
            Passo::Pronto(Comando::Mover(Movimento::InicioDaLinha, 1))
        );

        let mut p = Pendente::default();
        assert_eq!(
            passo(&mut p, &["1", "0", "j"]),
            Passo::Pronto(Comando::Mover(Movimento::Baixo, 10))
        );
    }

    #[test]
    fn operador_com_movimento() {
        let mut p = Pendente::default();
        assert_eq!(passo(&mut p, &["d"]), Passo::Aguardando);
        assert_eq!(
            passo(&mut p, &["w"]),
            Passo::Pronto(Comando::Apagar(Movimento::PalavraFrente, 1))
        );
    }

    #[test]
    fn operador_dobrado_age_na_linha() {
        for (op, esperado) in [
            ("d", Comando::Apagar(Movimento::LinhaInteira, 1)),
            ("y", Comando::Copiar(Movimento::LinhaInteira, 1)),
            ("c", Comando::Mudar(Movimento::LinhaInteira, 1)),
        ] {
            let mut p = Pendente::default();
            assert_eq!(passo(&mut p, &[op, op]), Passo::Pronto(esperado), "{op}{op}");
        }
    }

    #[test]
    fn as_duas_contagens_se_multiplicam() {
        // `2d3w` apaga seis palavras, como no vim.
        let mut p = Pendente::default();
        assert_eq!(
            passo(&mut p, &["2", "d", "3", "w"]),
            Passo::Pronto(Comando::Apagar(Movimento::PalavraFrente, 6))
        );
    }

    #[test]
    fn gg_vai_pro_topo_e_g_sozinho_espera() {
        let mut p = Pendente::default();
        assert_eq!(passo(&mut p, &["g"]), Passo::Aguardando);
        assert_eq!(
            passo(&mut p, &["g"]),
            Passo::Pronto(Comando::Mover(Movimento::InicioDoDocumento, 1))
        );
    }

    #[test]
    fn dgg_apaga_ate_o_topo() {
        let mut p = Pendente::default();
        assert_eq!(
            passo(&mut p, &["d", "g", "g"]),
            Passo::Pronto(Comando::Apagar(Movimento::InicioDoDocumento, 1))
        );
    }

    #[test]
    fn r_troca_o_caractere_seguinte_seja_qual_for() {
        // Inclusive um que normalmente é comando: `rd` põe um "d".
        let mut p = Pendente::default();
        assert_eq!(passo(&mut p, &["r"]), Passo::Aguardando);
        assert_eq!(passo(&mut p, &["d"]), Passo::Pronto(Comando::Substituir('d')));

        // E um dígito não vira contagem depois do `r`.
        let mut p = Pendente::default();
        assert_eq!(
            passo(&mut p, &["r", "3"]),
            Passo::Pronto(Comando::Substituir('3'))
        );
    }

    #[test]
    fn r_com_tecla_nomeada_cancela_em_vez_de_escrever() {
        let mut p = Pendente::default();
        tecla_normal(&mut p, "r", false);
        assert_eq!(tecla_normal(&mut p, "Escape", false), Passo::Aguardando);
        assert!(!p.em_curso(), "Escape tinha que ter cancelado o r");
    }

    #[test]
    fn operador_seguido_de_outro_operador_cancela() {
        let mut p = Pendente::default();
        assert_eq!(passo(&mut p, &["d", "c"]), Passo::Ignorada);
        assert!(!p.em_curso(), "`dc` não é comando; a máquina tem que zerar");
    }

    #[test]
    fn tecla_desconhecida_nao_deixa_lixo_pendente() {
        let mut p = Pendente::default();
        assert_eq!(passo(&mut p, &["3", "ç"]), Passo::Ignorada);
        assert!(!p.em_curso());
    }

    #[test]
    fn atalhos_com_ctrl() {
        let mut p = Pendente::default();
        assert_eq!(tecla_normal(&mut p, "r", true), Passo::Pronto(Comando::Refazer));
        assert_eq!(
            tecla_normal(&mut p, "v", true),
            Passo::Pronto(Comando::VisualBloco)
        );
    }

    #[test]
    fn maiusculas_de_linha() {
        let casos = [
            ("D", Comando::Apagar(Movimento::FimDaLinha, 1)),
            ("C", Comando::Mudar(Movimento::FimDaLinha, 1)),
            ("Y", Comando::Copiar(Movimento::LinhaInteira, 1)),
        ];
        for (tecla, esperado) in casos {
            let mut p = Pendente::default();
            assert_eq!(passo(&mut p, &[tecla]), Passo::Pronto(esperado), "{tecla}");
        }
    }

    #[test]
    fn rotulo_mostra_o_que_esta_pela_metade() {
        let mut p = Pendente::default();
        passo(&mut p, &["2", "d", "3"]);
        assert_eq!(p.rotulo(), "2d3");
    }

    #[test]
    fn entrar_em_insercao_pelas_seis_portas() {
        let casos = [
            ("i", Insercao::Antes),
            ("a", Insercao::Depois),
            ("I", Insercao::InicioDaLinha),
            ("A", Insercao::FimDaLinha),
            ("o", Insercao::LinhaAbaixo),
            ("O", Insercao::LinhaAcima),
        ];
        for (tecla, esperado) in casos {
            let mut p = Pendente::default();
            assert_eq!(
                passo(&mut p, &[tecla]),
                Passo::Pronto(Comando::Entrar(esperado)),
                "{tecla}"
            );
        }
    }
}
