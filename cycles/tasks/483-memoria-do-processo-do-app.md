---
id: "483"
titulo: "De que é feita a memória do processo do app"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["480", "482"]
estima_min: 180
---

# 483 — A memória do processo do app

Depois do 482, o processo do app (Rust + GTK, sem a página web) fica em
~145–158 MiB de PSS parado. É um número só, e ele junta coisas que não se
cortam do mesmo jeito. Antes de cortar qualquer coisa, separar.

## Critérios de aceite

- [x] `scripts/memoria/app.mjs`: decompõe o `smaps` do app em heap,
      buffers da janela, binário, driver de vídeo e bibliotecas, com PSS e
      USS por categoria.
- [x] A parte do Rust medida de dentro: um alocador global com contador
      (`core::memoria::AlocadorContado`) e o comando `memoria_do_rust`
      (vivos e pico). Pelo `smaps` ela é indistinguível do resto do heap.
- [x] `--referencia`: o mesmo recorte do MiniBrowser do WebKitGTK numa
      página em branco, como o piso de uma janela com WebView nesta
      máquina.
- [x] O `janela.mjs` mostra a parte do Rust antes e depois do dia, pra
      dizer se o crescimento por rodada é nosso.
- [x] Medido no app instalado. Se houver corte que seja nosso, ele vem com
      a medida de antes e depois. Se não houver, dizer isso com a medida.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib memoria::
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
node scripts/memoria/app.mjs --porta 9323 --referencia
node scripts/memoria/janela.mjs --porta 9323 --rodadas 3
```

## Não-objetivos

- Tirar o contorno do NVIDIA (`WEBKIT_DISABLE_DMABUF_RENDERER`) às cegas.
  Ele evita um travamento real. Só sai com uma medida de que o
  travamento sumiu.
- Trocar o alocador (jemalloc, mimalloc) sem a medida apontar para o
  heap do Rust.
