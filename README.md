# pls-reply

`pls` provides fast, concise answers to CLI-related queries. Responses are
streamed as they arrive and copied to the clipboard.

<p align="center">
  <img src="assets/pls-reply.gif" alt="pls-reply demo">
</p>

## Installation

With mise:

    mise use -g github:rtuszik/pls-reply@latest

From source:

    cargo install --git https://github.com/rtuszik/pls-reply

Building from source requires Rust 1.85 or later.

## Configuration

On first run, `pls` creates `$XDG_CONFIG_HOME/pls/pls.toml`, usually
`~/.config/pls/pls.toml`.

Minimal configuration:

    [model]
    provider = "openai"
    name = "gpt-5.6-luna"
    api_key_env = "OPENAI_API_KEY"

    [prompt]
    system = """
    You are a terminal assistant running on a {os} unix system.
    Reply with only the single most direct shell command or answer.
    """

Supports most LLM providers and model routers through native or
OpenAI-compatible APIs. See [`pls.example.toml`](pls.example.toml) for all
settings.

## Usage

    pls git command to show first commit

Queries can also be piped in:

    echo 'find files named `.prek.toml` or .pre-commit-config.yaml' | pls

Run `pls` without arguments for an interactive prompt.

    -m, --model <NAME>   Override the configured model
        --no-copy        Do not copy the response
        --stats          Print latency and token statistics

Clipboard support requires `pbcopy` on macOS or `wl-copy`, `xclip`, or `xsel`
on Linux.

## License

> This program is free software. It comes without any warranty, to
> the extent permitted by applicable law. You can redistribute it
> and/or modify it under the terms of the Do What The Fuck You Want
> To Public License, Version 2, as published by Sam Hocevar. See
> http://www.wtfpl.net/ for more details.
