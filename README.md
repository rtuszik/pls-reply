# pls-reply

pls-reply asks a large language model for a short, direct answer to a
terminal question and prints it. It is written in Rust and installs a
single binary, `pls`. The reply is streamed to standard output as it
arrives and the final text is copied to the clipboard.

It is intended for one-line lookups such as "git command to show the
first commit", where the useful reply is a single command and nothing
else.

## Requirements

- Rust 1.85 or later (2024 edition) to build.
- An API key for a supported provider, supplied through an environment
  variable.
- A clipboard utility for the copy feature: `pbcopy` on macOS, or
  `wl-copy`, `xclip`, or `xsel` on Linux. Without one, the answer is
  still printed.

The toolchain is managed with mise; running `mise install` provisions
Rust and the auxiliary tools (opengrep, zizmor, prek).

## Installation

Install a released version with cargo, replacing the tag with the
release you want:

    cargo install --git https://github.com/rtuszik/pls-reply --tag v0.1.0

Cargo builds pls from source and installs the binary onto its install
path, `~/.cargo/bin` by default.

Alternatively, download a prebuilt binary. Each release provides
archives for Linux and macOS on x86_64 and aarch64. Fetch the archive
matching your platform, extract `pls`, and place it on your PATH:

    tag=v0.1.0
    target=x86_64-unknown-linux-gnu
    base=https://github.com/rtuszik/pls-reply/releases/download
    curl -fsSL "$base/$tag/pls-$tag-$target.tar.gz" | tar -xz
    install -m 0755 pls ~/.local/bin/pls

The available targets are x86_64-unknown-linux-gnu,
aarch64-unknown-linux-gnu, x86_64-apple-darwin, and
aarch64-apple-darwin.

To build from a local checkout instead:

    cargo build --release

The resulting binary is `target/release/pls`.

## Configuration

pls reads `$XDG_CONFIG_HOME/pls/pls.toml`, falling back to
`~/.config/pls/pls.toml`. On first run, when no configuration exists,
pls writes a commented template to that path and exits so it can be
edited.

A minimal configuration:

    [model]
    provider    = "openai"
    name        = "gpt-5.6-luna"
    api_key_env = "OPENAI_API_KEY"

    [prompt]
    system = """
    You are a terminal assistant running on a {os} unix system.
    Reply with only the single most direct shell command or answer.
    """

The `{os}` placeholder is replaced at runtime with `darwin` or `linux`.
The API key is read from the environment variable named by
`api_key_env`. Supported providers are `openai`, `anthropic`, `gemini`,
`groq`, `ollama`, `xai`, `deepseek`, `cohere`, and `custom`. For
`custom`, set `base_url` to an OpenAI-compatible endpoint.

## Usage

    pls git command to show first commit

The query may instead be read from standard input, which avoids the
shell interpreting characters such as backticks:

    echo 'find files named `.prek.toml` or .pre-commit-config.yaml' | pls

Run pls with no arguments and no piped input to type the query at an
interactive prompt.

Options:

    -m, --model <NAME>   Override the configured model for one run.
        --no-copy        Do not copy the answer to the clipboard.

## License

> This program is free software. It comes without any warranty, to
> the extent permitted by applicable law. You can redistribute it
> and/or modify it under the terms of the Do What The Fuck You Want
> To Public License, Version 2, as published by Sam Hocevar. See
> http://www.wtfpl.net/ for more details.

pls-reply is distributed under the WTFPL License. See LICENSE.
