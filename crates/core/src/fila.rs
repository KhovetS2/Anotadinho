//! A fila de execuções do agente (ciclo 408; veio pro núcleo no 429).
//!
//! Mora aqui porque a janela precisa da MESMA política que a TUI: o
//! limite de paralelismo não pode depender de por onde se mandou a
//! pergunta. Quem dispara processo continua sendo cada UI.
//!
//! Desde o ciclo 340 dá pra ter uma execução por conversa, e várias
//! conversas ao mesmo tempo — só que sem limite nenhum. Abrir cinco
//! conversas e mandar em todas subia cinco processos de modelo de uma
//! vez: a máquina engasgava, e quem paga por token levava cinco cobranças
//! simultâneas sem ter pedido isso.
//!
//! Aqui está a política, separada do IO pra dar pra testar: quantas
//! rodam juntas, quem espera, em que ordem, e o que acontece quando uma
//! vaga abre. Quem dispara processo é o `main`; quem decide se agora é a
//! vez é isto.
//!
//! A ordem é de chegada (FIFO). Não há prioridade: a surpresa de uma
//! execução "furar" a fila custaria mais do que o ganho.

/// Um envio esperando vaga.
///
/// O que ele CARREGA muda com a UI — a TUI guarda a pergunta e os anexos
/// (o prompt é montado na hora do disparo), a janela guarda o prompt já
/// montado e o adaptador. Por isso o tipo é genérico: a política de fila
/// é a mesma, o payload não.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Espera<T> {
    /// O vault de onde saiu (a pessoa pode trocar de vault antes da vez
    /// chegar — ciclo 396).
    pub vault: String,
    pub conversa: String,
    /// O que aquela UI precisa pra disparar quando a vez chegar.
    pub carga: T,
}

/// Quantas execuções em paralelo por padrão. Duas: uma que você olha e
/// uma que trabalha no fundo.
pub const LIMITE_PADRAO: usize = 2;

/// A fila e o limite.
#[derive(Debug, Default)]
pub struct Fila<T> {
    /// Máximo rodando ao mesmo tempo. `0` = sem limite (a fila nunca
    /// retém nada).
    pub limite: usize,
    espera: Vec<Espera<T>>,
}

impl<T> Fila<T> {
    pub fn nova(limite: usize) -> Self {
        Self { limite, espera: Vec::new() }
    }

    /// Cabe mais uma agora, com `rodando` em andamento?
    pub fn tem_vaga(&self, rodando: usize) -> bool {
        self.limite == 0 || rodando < self.limite
    }

    /// Guarda pra depois. Devolve a posição na fila (1 = próxima).
    pub fn enfileirar(&mut self, e: Espera<T>) -> usize {
        self.espera.push(e);
        self.espera.len()
    }

    /// A próxima da fila, se há vaga. Tira da fila ao devolver.
    pub fn proxima(&mut self, rodando: usize) -> Option<Espera<T>> {
        if !self.tem_vaga(rodando) || self.espera.is_empty() {
            return None;
        }
        Some(self.espera.remove(0))
    }

    /// Em que posição está esta conversa (1 = próxima), se está.
    pub fn posicao(&self, vault: &str, conversa: &str) -> Option<usize> {
        self.espera
            .iter()
            .position(|e| e.vault == vault && e.conversa == conversa)
            .map(|i| i + 1)
    }

    /// Já tem envio desta conversa esperando? Dois envios na mesma
    /// conversa se atropelariam — a segunda pergunta iria pro histórico
    /// antes da primeira resposta.
    pub fn ja_espera(&self, vault: &str, conversa: &str) -> bool {
        self.posicao(vault, conversa).is_some()
    }

    /// Tira esta conversa da fila. `true` se estava.
    pub fn desistir(&mut self, vault: &str, conversa: &str) -> bool {
        let antes = self.espera.len();
        self.espera.retain(|e| !(e.vault == vault && e.conversa == conversa));
        self.espera.len() < antes
    }

    /// Esvazia a fila. Devolve quantas desistiram.
    pub fn limpar(&mut self) -> usize {
        let n = self.espera.len();
        self.espera.clear();
        n
    }

    /// Quem está esperando, na ordem.
    pub fn esperando(&self) -> &[Espera<T>] {
        &self.espera
    }

    pub fn len(&self) -> usize {
        self.espera.len()
    }

    pub fn is_empty(&self) -> bool {
        self.espera.is_empty()
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn espera(conversa: &str) -> Espera<String> {
        Espera { vault: "/v".into(), conversa: conversa.into(), carga: "e aí".into() }
    }

    #[test]
    fn o_limite_decide_quem_roda_agora() {
        let f: Fila<String> = Fila::nova(2);
        assert!(f.tem_vaga(0) && f.tem_vaga(1));
        assert!(!f.tem_vaga(2), "no limite não cabe mais");
        assert!(!f.tem_vaga(5), "nem acima dele (o limite pode ter caído)");
        // Limite 0 é "sem limite": a fila nunca retém.
        let sem: Fila<String> = Fila::nova(0);
        assert!(sem.tem_vaga(0) && sem.tem_vaga(99));
    }

    #[test]
    fn a_fila_e_de_chegada_e_a_vaga_chama_a_proxima() {
        let mut f: Fila<String> = Fila::nova(1);
        assert_eq!(f.enfileirar(espera("a.md")), 1);
        assert_eq!(f.enfileirar(espera("b.md")), 2);
        assert_eq!(f.posicao("/v", "b.md"), Some(2));
        // Com uma rodando não sai ninguém.
        assert_eq!(f.proxima(1), None);
        // A vaga abriu: sai a primeira, e a outra sobe pra 1ª.
        assert_eq!(f.proxima(0).unwrap().conversa, "a.md");
        assert_eq!(f.posicao("/v", "a.md"), None);
        assert_eq!(f.posicao("/v", "b.md"), Some(1));
        assert_eq!(f.proxima(0).unwrap().conversa, "b.md");
        assert_eq!(f.proxima(0), None, "fila vazia");
    }

    #[test]
    fn a_mesma_conversa_nao_entra_duas_vezes_e_da_pra_desistir() {
        let mut f: Fila<String> = Fila::nova(1);
        f.enfileirar(espera("a.md"));
        assert!(f.ja_espera("/v", "a.md"));
        assert!(!f.ja_espera("/v", "z.md"));
        // A conversa é por vault: o mesmo nome noutro vault é outra.
        let mut outra = espera("a.md");
        outra.vault = "/w".into();
        f.enfileirar(outra);
        assert_eq!(f.len(), 2);
        assert!(f.desistir("/v", "a.md"));
        assert!(!f.desistir("/v", "a.md"), "desistir de quem não está é falso");
        assert_eq!(f.posicao("/w", "a.md"), Some(1));
    }

    #[test]
    fn limpar_devolve_quantas_desistiram() {
        let mut f: Fila<String> = Fila::nova(1);
        f.enfileirar(espera("a.md"));
        f.enfileirar(espera("b.md"));
        assert_eq!(f.limpar(), 2);
        assert!(f.is_empty() && f.limpar() == 0);
    }

    #[test]
    fn o_que_espera_sai_na_ordem_pra_tela() {
        let mut f: Fila<String> = Fila::nova(1);
        f.enfileirar(espera("a.md"));
        f.enfileirar(espera("b.md"));
        let nomes: Vec<&str> = f.esperando().iter().map(|e| e.conversa.as_str()).collect();
        assert_eq!(nomes, ["a.md", "b.md"]);
    }
}
