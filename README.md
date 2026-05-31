# DragonFox IDE

A terminal-based AI programming assistant powered by Rust — ask coding questions and get instant answers right from your command line.

<!-- TODO: Add a GIF/screenshot of a real session here -->
<!-- Example: ![DragonFox IDE demo](assets/demo.gif) -->

---

## Quick Start

```bash
cargo run --release
```

That's it. Enter your API key when prompted, then start asking questions.

## Features

- **AI-powered chat** — ask programming questions and get detailed answers in your terminal
- **Secure API key input** — your key is hidden while typing (no shoulder-surfing)
- **Lightweight & fast** — single binary, no runtime dependencies, no config files needed
- **Optimized builds** — release profile tuned with LTO, symbol stripping, and size optimization
- **Built in Rust** — safe, fast, and zero garbage collection pauses

## How to Run Locally

### Requirements

- **Rust** 1.85+ (edition 2024) — [install via rustup](https://rustup.rs/)
- An API key for the [Hack Club AI proxy](https://ai.hackclub.com/)

### Build & run and demo

```bash
# Clone the repo
git clone https://github.com/cyberworrier8088/DragonFoxIDE.git
cd DragonFoxIDE

# Run (debug build — fast compile)
cargo run

# Or build an optimized release binary
cargo build --release
./target/release/DragonFoxIDE
```

## How It Works

DragonFox IDE is a modular Rust CLI app with four source files:

| File | Purpose |

| `main.rs`  Entry point — startup banner, API key prompt, chat loop 
| `ai.rs`  Sends prompts to the AI API and prints responses 
| `config.rs`  Handles secure (hidden) API key input via `rpassword` 
| `input.rs`  Reads user input from stdin 

The app connects to [Hack Club's free AI proxy](https://ai.hackclub.com/), which routes requests to the `nvidia/nemotron-3-nano-omni-30b-a3b-reasoning` model using the OpenAI-compatible chat completions format. HTTP requests are handled async through `reqwest` + `tokio`, so the UI stays responsive while waiting for AI responses.

The release build is tuned for size: LTO, single codegen unit, symbol stripping, and `panic = "abort"` produce a compact standalone binary with no external runtime.


## Tech Stack

- **Language:** Rust (edition 2024)
- **Async runtime:** Tokio
- **HTTP client:** Reqwest
- **JSON:** Serde + serde_json
- **Secure input:** rpassword
- **AI model:** NVIDIA Nemotron via Hack Club proxy

## License

[MIT](LICENSE) © cyberworrier8088

---

Made by imuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuu :)
