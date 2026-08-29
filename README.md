# pls-reply

pls-reply asks a large language model for a short, direct answer to a
terminal question and prints it. It is written in Rust and installs a
single binary, `pls`. The reply is streamed to standard output as it
arrives and the final text is copied to the clipboard.

It is intended for one-line lookups such as "git command to show the
first commit", where the useful reply is a single command and nothing
else.

<p align="center">
  <img src="assets/pls-reply.gif">
</p>

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

### mise

```
mise use -g github:rtuszik/pls-reply
```

### cargo

Install a released version with cargo, replacing the tag with the
release you want:

    cargo install --git https://github.com/rtuszik/pls-reply --tag v0.1.0

## Configuration

pls reads `$XDG_CONFIG_HOME/pls/pls.toml`, falling back to
`~/.config/pls/pls.toml`. This file will be created one first run.

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

Run pls with no arguments for an interactive prompt.

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
