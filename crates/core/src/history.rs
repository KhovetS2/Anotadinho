//! Histórico de desfazer/refazer por SNAPSHOT do documento inteiro.
//!
//! Guarda o markdown completo por entrada, não um diff: uma página cabe
//! folgada em memória e o snapshot elimina a classe inteira de bug de
//! "patch aplicado fora de ordem". O limite de profundidade existe só
//! pra página gigante não crescer sem fim.
//!
//! A decisão de AGRUPAR (juntar uma rajada de digitação num passo só)
//! fica com quem chama, porque depende de relógio — o histórico só
//! recebe o `agrupar: bool` já resolvido. Isso é o que mantém este
//! módulo testável fora do WASM.
//!
//! Regra que motivou o ciclo 186: mutação ESTRUTURAL (inserir, remover,
//! mover, duplicar segmento, mudar dados de embed) nunca agrupa. Antes
//! ela passava pela mesma janela de agrupamento da digitação e, se
//! caísse dentro dela, o estado anterior sumia do histórico — desfazer
//! pulava direto pra um estado bem mais antigo.

/// Profundidade padrão. Vinte passos é o que o editor já usava antes de
/// o histórico virar um tipo próprio.
pub const LIMITE_PADRAO: usize = 20;

#[derive(Debug, Clone)]
pub struct History {
    desfazer: Vec<String>,
    refazer: Vec<String>,
    atual: String,
    limite: usize,
}

impl History {
    pub fn new(inicial: impl Into<String>) -> Self {
        Self::com_limite(inicial, LIMITE_PADRAO)
    }

    pub fn com_limite(inicial: impl Into<String>, limite: usize) -> Self {
        Self {
            desfazer: Vec::new(),
            refazer: Vec::new(),
            atual: inicial.into(),
            limite: limite.max(1),
        }
    }

    /// Estado corrente — o que o editor tem na tela agora.
    pub fn atual(&self) -> &str {
        &self.atual
    }

    /// Registra um estado novo.
    ///
    /// `agrupar` junta com o passo anterior (não empilha nada, só troca
    /// o corrente): é o que faz uma rajada de digitação virar UM passo
    /// de desfazer em vez de um passo por tecla.
    ///
    /// Devolve `true` quando um ponto de desfazer novo foi criado.
    /// Conteúdo idêntico ao corrente não faz nada.
    pub fn registrar(&mut self, novo: impl Into<String>, agrupar: bool) -> bool {
        let novo = novo.into();
        if novo == self.atual {
            return false;
        }
        if agrupar && !self.desfazer.is_empty() {
            // Sem ponto nenhum no histórico ainda, agrupar deixaria o
            // primeiro estado da sessão sem volta — daí o
            // `!self.desfazer.is_empty()`.
            self.atual = novo;
            return false;
        }
        let anterior = std::mem::replace(&mut self.atual, novo);
        self.desfazer.push(anterior);
        if self.desfazer.len() > self.limite {
            self.desfazer.remove(0);
        }
        self.refazer.clear();
        true
    }

    /// Volta um passo. `None` quando não há pra onde voltar.
    pub fn desfazer(&mut self) -> Option<String> {
        let anterior = self.desfazer.pop()?;
        let corrente = std::mem::replace(&mut self.atual, anterior.clone());
        self.refazer.push(corrente);
        Some(anterior)
    }

    /// Refaz um passo desfeito. `None` quando não há.
    pub fn refazer(&mut self) -> Option<String> {
        let proximo = self.refazer.pop()?;
        let corrente = std::mem::replace(&mut self.atual, proximo.clone());
        self.desfazer.push(corrente);
        Some(proximo)
    }

    pub fn pode_desfazer(&self) -> bool {
        !self.desfazer.is_empty()
    }

    pub fn pode_refazer(&self) -> bool {
        !self.refazer.is_empty()
    }

    /// Recomeça do zero — usado ao trocar de página, pra desfazer nunca
    /// aplicar a edição de uma página em outra.
    pub fn reiniciar(&mut self, inicial: impl Into<String>) {
        self.desfazer.clear();
        self.refazer.clear();
        self.atual = inicial.into();
    }

    pub fn profundidade(&self) -> usize {
        self.desfazer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desfazer_e_refazer_percorrem_os_estados() {
        let mut h = History::new("a");
        h.registrar("b", false);
        h.registrar("c", false);
        assert_eq!(h.atual(), "c");
        assert_eq!(h.desfazer().as_deref(), Some("b"));
        assert_eq!(h.desfazer().as_deref(), Some("a"));
        assert_eq!(h.desfazer(), None);
        assert_eq!(h.refazer().as_deref(), Some("b"));
        assert_eq!(h.refazer().as_deref(), Some("c"));
        assert_eq!(h.refazer(), None);
    }

    #[test]
    fn agrupar_nao_cria_ponto_novo() {
        let mut h = History::new("");
        h.registrar("a", false);
        h.registrar("ab", true);
        h.registrar("abc", true);
        assert_eq!(h.profundidade(), 1, "a rajada toda é um passo só");
        assert_eq!(h.desfazer().as_deref(), Some(""));
    }

    #[test]
    fn estrutural_no_meio_da_rajada_nao_se_perde() {
        // O bug do ciclo 186: com o agrupamento decidido só pelo relógio,
        // inserir um embed logo depois de digitar era engolido pela
        // janela e o estado pré-inserção sumia do histórico.
        let mut h = History::new("texto");
        h.registrar("texto d", false);
        h.registrar("texto di", true);
        h.registrar("texto dig", true);
        assert!(h.registrar("texto dig\n\n{{ type: \"callout\" }}\n{{ /callout }}", false));
        assert_eq!(
            h.desfazer().as_deref(),
            Some("texto dig"),
            "desfazer tem que voltar exatamente pro estado de antes da inserção"
        );
    }

    #[test]
    fn registrar_depois_de_desfazer_descarta_o_refazer() {
        let mut h = History::new("a");
        h.registrar("b", false);
        h.desfazer();
        assert!(h.pode_refazer());
        h.registrar("c", false);
        assert!(!h.pode_refazer(), "ramo novo descarta o futuro antigo");
    }

    #[test]
    fn conteudo_igual_nao_vira_passo() {
        let mut h = History::new("a");
        assert!(!h.registrar("a", false));
        assert_eq!(h.profundidade(), 0);
    }

    #[test]
    fn agrupar_sem_historico_ainda_cria_o_primeiro_ponto() {
        // Senão o primeiro estado da sessão ficaria sem volta.
        let mut h = History::new("a");
        assert!(h.registrar("b", true));
        assert_eq!(h.desfazer().as_deref(), Some("a"));
    }

    #[test]
    fn limite_descarta_os_mais_antigos() {
        let mut h = History::com_limite("0", 3);
        for i in 1..=5 {
            h.registrar(i.to_string(), false);
        }
        assert_eq!(h.profundidade(), 3);
        assert_eq!(h.desfazer().as_deref(), Some("4"));
        assert_eq!(h.desfazer().as_deref(), Some("3"));
        assert_eq!(h.desfazer().as_deref(), Some("2"));
        assert_eq!(h.desfazer(), None, "os mais antigos caíram pelo limite");
    }

    #[test]
    fn reiniciar_limpa_tudo() {
        let mut h = History::new("a");
        h.registrar("b", false);
        h.desfazer();
        h.reiniciar("outra pagina");
        assert!(!h.pode_desfazer());
        assert!(!h.pode_refazer());
        assert_eq!(h.atual(), "outra pagina");
    }
}
