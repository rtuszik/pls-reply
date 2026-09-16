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
        --stats-json     Print machine-readable timing and usage to stderr
        --profile        Print a detailed latency breakdown to stderr
        --profile-json   Print a machine-readable latency profile to stderr

Clipboard support requires `pbcopy` on macOS or `wl-copy`, `xclip`, or `xsel`
on Linux.

Run `mise run bench` to build the release binary and benchmark it with Hyperfine.
It measures complete response time over ten requests using your configured model.

The stats line labels throughput as `effective tok/s`: provider-reported
completion tokens divided by request duration, from request start through stream
completion. This includes time to first output and may include reasoning tokens;
it excludes local request preparation and clipboard work. It measures effective
request throughput, not model generation speed.

Profiling is separate from statistics and can be enabled alongside either stats
format. For example:

    pls --profile --no-copy git command to show the first commit

The profile covers argument parsing, runtime setup, config loading, input,
request preparation, time to first content, streaming, stream tail, response
finalization, clipboard work, and cleanup. Input wait is included in the total
but shown separately. Total time starts at entry to `main` and ends before the
profile report; it excludes OS process startup and final process teardown.
Network and provider time are combined. Chunk write/flush time and the first
content-to-flush interval overlap the phases and must not be added to the total.
A stream with no content has no streaming-content or tail phase.

Failures after argument parsing emit a partial profile with the failing phase.
Clipboard remains best-effort; its duration does not establish copy success.
Profiles contain timings and outcomes, without prompts, answers, or credentials.
`--profile-json` emits a `pls_profile` record with `schema_version: 1`;
`--stats-json` retains its existing schema and timing boundaries.

## License

> This program is free software. It comes without any warranty, to
> the extent permitted by applicable law. You can redistribute it
> and/or modify it under the terms of the Do What The Fuck You Want
> To Public License, Version 2, as published by Sam Hocevar. See
> http://www.wtfpl.net/ for more details.
