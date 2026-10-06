//! Encerrar o agente inteiro, não só o processo que nasceu do spawn.
//!
//! `claude` e `codex` são wrappers: o processo que o `Command::spawn`
//! devolve arranca outro (Node, e daí o modelo), e é o neto que faz o
//! trabalho. `Child::kill` mata só o filho direto. O neto fica — segurando
//! o pipe, gastando a máquina, e às vezes ainda escrevendo — depois de a
//! pessoa clicar em interromper ou de o tempo limite estourar.
//!
//! No Linux isso costuma se resolver sozinho quando o pipe fecha, mas
//! "costuma" não é garantia, e no Windows não acontece de jeito nenhum
//! (item D1 do diagnóstico de portabilidade). Aqui a árvore inteira é
//! encerrada nos dois sistemas, cada um com o mecanismo que tem.
//!
//! As duas implementações são *best-effort* de propósito: um agente que
//! já morreu sozinho, um pid reciclado, uma permissão negada — nada
//! disso pode falhar o cancelamento, que é justamente a operação que a
//! pessoa aciona quando quer que algo pare.

/// Ajusta o comando para que a árvore do filho seja alcançável depois.
///
/// Chamado ANTES do spawn.
#[cfg(unix)]
pub fn preparar(comando: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    // `pre_exec` roda no filho, entre o fork e o exec. `setpgid(0, 0)`
    // ali faz dele líder de um grupo novo, cujo id é o seu próprio pid —
    // exatamente o número que `encerrar_arvore` vai negativar.
    //
    // Tem que ser aqui, e não depois do spawn: um `setpgid` de fora é
    // corrida com o filho, que pode já ter chamado `exec`.
    //
    // E tem que ser um grupo NOVO: sem isto, o sinal iria pro grupo
    // herdado, onde o Anotadinho também está. Cancelar o agente
    // derrubaria o app junto.
    //
    // `unsafe` porque entre fork e exec só vale chamar função
    // async-signal-safe. `setpgid` é uma delas.
    unsafe {
        comando.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                // Falhar aqui não justifica abortar a execução: o agente
                // roda igual, e o cancelamento cai no `kill` do filho
                // direto, que é o que havia antes deste módulo.
                return Ok(());
            }
            Ok(())
        });
    }
}

/// Ver `preparar` (unix).
///
/// No Windows não há o que preparar: o encerramento é por pid, e o pid
/// só existe depois do spawn.
#[cfg(windows)]
pub fn preparar(_comando: &mut std::process::Command) {}

/// Encerra o processo e tudo que ele criou.
#[cfg(unix)]
pub fn encerrar_arvore(filho: &mut std::process::Child) {
    let pid = filho.id() as i32;
    // O sinal vai pro GRUPO (pid negativo), não pro processo: é o que
    // alcança o neto. E é `SIGKILL` porque `SIGTERM` num wrapper Node
    // costuma ser tratado sem repassar aos filhos.
    unsafe {
        libc::killpg(pid, libc::SIGKILL);
    }
    // O filho direto também, para o caso de o `setpgid` não ter pegado.
    let _ = filho.kill();
}

/// Ver `encerrar_arvore` (unix).
///
/// O Windows tem duas vias: um *job object* com
/// `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, ou o `taskkill /T`. O job é
/// mais forte — sobrevive até a um encerramento à força do Anotadinho,
/// quando não sobra código nosso pra rodar — mas custa FFI `unsafe` que
/// **não compila nem roda em nenhuma máquina deste projeto hoje**, e
/// código não verificado no caminho do cancelamento é pior do que código
/// simples: o `taskkill` erra na direção de não matar, o FFI errado erra
/// na direção de travar o app. Quando houver uma máquina Windows para
/// medir, a troca é local a esta função.
#[cfg(windows)]
pub fn encerrar_arvore(filho: &mut std::process::Child) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // `/T` desce a árvore inteira a partir do pid; `/F` não pede
    // licença. Sem `CREATE_NO_WINDOW`, cada cancelamento pisca uma
    // janela de console na cara de quem clicou.
    let _ = std::process::Command::new("taskkill")
        .args(["/PID", &filho.id().to_string(), "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = filho.kill();
}

/// Como ligar o stdin do agente: um pipe quando há o que entregar,
/// fechado quando não há (ciclo 449).
///
/// Fechado e não herdado: herdar deixaria o agente esperando o teclado
/// de quem abriu o app — foi o "Reading additional input from stdin..."
/// do ciclo 219.
pub fn stdin_para(entrada: &Option<String>) -> std::process::Stdio {
    if entrada.is_some() {
        std::process::Stdio::piped()
    } else {
        std::process::Stdio::null()
    }
}

/// Entrega o prompt pelo stdin do agente e FECHA o pipe (ciclo 449).
///
/// Numa thread própria, porque um prompt grande enche o buffer do pipe
/// (64 KiB no Linux) e a escrita bloqueia até o agente ler. O agente,
/// por sua vez, pode estar bloqueado escrevendo na saída que ninguém lê
/// ainda — se a escrita fosse na mesma thread que depois lê a saída,
/// os dois esperariam um pelo outro pra sempre.
///
/// Fechar é o que diz ao agente que o prompt acabou: sem o EOF, ele
/// esperaria mais texto e nunca começaria.
///
/// Erro de escrita é ignorado de propósito: o agente que morre antes de
/// ler tudo dá `BrokenPipe` aqui, e o motivo que interessa à pessoa vem
/// da saída dele, não deste pipe.
pub fn entregar_entrada(filho: &mut std::process::Child, entrada: Option<String>) {
    let (Some(texto), Some(mut pipe)) = (entrada, filho.stdin.take()) else {
        return;
    };
    std::thread::spawn(move || {
        use std::io::Write;
        let _ = pipe.write_all(texto.as_bytes());
        // `pipe` sai de escopo aqui, e é isso que fecha.
    });
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    /// O neto já morreu (ou virou zumbi esperando quem o recolha, que
    /// para quem olha de fora é o mesmo que morto).
    fn morto(pid: u32) -> bool {
        match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            Err(_) => true,
            // `pid (comm) S ...`: o estado vem depois do último `)`.
            Ok(stat) => stat.rsplit(')').next().map(|r| r.trim_start().starts_with('Z')).unwrap_or(true),
        }
    }

    /// O caso que o módulo existe pra resolver: o filho cria um neto, e
    /// interromper o filho tem que levar o neto junto.
    #[test]
    fn encerrar_arvore_alcanca_o_neto() {
        let mut comando = Command::new("sh");
        comando
            .args(["-c", "sleep 30 & echo $!; wait"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        preparar(&mut comando);
        let mut filho = comando.spawn().expect("sh");
        let mut linha = String::new();
        BufReader::new(filho.stdout.take().unwrap()).read_line(&mut linha).unwrap();
        let neto: u32 = linha.trim().parse().expect("pid do neto");
        assert!(!morto(neto), "o neto devia estar vivo antes do cancelamento");

        encerrar_arvore(&mut filho);
        let _ = filho.wait();

        let prazo = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !morto(neto) && std::time::Instant::now() < prazo {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let vivo = !morto(neto);
        if vivo {
            // Não deixa o `sleep` órfão pra trás quando o teste reprova.
            unsafe { libc::kill(neto as i32, libc::SIGKILL) };
        }
        assert!(!vivo, "o neto {neto} sobreviveu ao cancelamento");
    }

    /// O prompt grande chega INTEIRO pelo stdin. 300 KiB passa do limite
    /// de um argumento no Linux (128 KiB) e do buffer do pipe (64 KiB):
    /// sem a thread de escrita, `cat` e este teste travariam um
    /// esperando o outro.
    #[test]
    fn prompt_grande_chega_inteiro_pelo_stdin() {
        use std::io::Read;
        let prompt: String = (0..300 * 1024).map(|i| if i % 80 == 79 { '\n' } else { 'a' }).collect();
        let entrada = Some(prompt.clone());
        let mut comando = Command::new("cat");
        comando.stdin(stdin_para(&entrada)).stdout(Stdio::piped()).stderr(Stdio::null());
        preparar(&mut comando);
        let mut filho = comando.spawn().expect("cat");
        entregar_entrada(&mut filho, entrada);
        let mut saida = String::new();
        filho.stdout.take().unwrap().read_to_string(&mut saida).unwrap();
        assert!(filho.wait().unwrap().success());
        assert_eq!(saida.len(), prompt.len());
        assert!(saida == prompt, "o conteúdo chegou diferente");
    }

    /// O mesmo prompt como ARGUMENTO não sai do lugar no Linux: é o
    /// limite que o stdin existe pra contornar. Se um dia este teste
    /// reprovar, o limite mudou — e o motivo do ciclo 449 no Linux
    /// também.
    #[test]
    fn o_mesmo_prompt_como_argumento_e_recusado() {
        let prompt: String = "a".repeat(300 * 1024);
        let erro = Command::new("true").arg(&prompt).spawn().err();
        assert!(erro.is_some(), "um argumento de 300 KiB passou");
    }
}
