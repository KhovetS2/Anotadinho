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
