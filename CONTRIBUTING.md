# Contributing to cloudfits

Thanks for contributing! cloudfits is a small, focused FOSS toolkit for estimating
LLM inference costs and switching providers.

## Scope

Respect the scope of the project. We want:

- Accurate, explainable cost estimates (shows the inputs, not just a number).
- Easy, low-friction provider comparison and switching.
- A clean CLI, a TUI, and a tiny web UI that all share the same core math.

We don't want:

- A full model-fitting engine (that's [llmfit](https://github.com/AlexsJones/llmfit)'s job).
- Deps heavier than needed. Prefer the crates already in the workspace.

## Development

```sh
cargo build          # build everything
cargo test           # run the test suite
cargo clippy -- -D warnings
cargo fmt -- --check
```

## Where things live

```
cloudfits-common/    Cost models, provider catalog, hardware detection (core math)
cloudfits-cli/       The `cloudfits` binary (CLI + `serve` web endpoint)
cloudfits-tui/       The `cloudfits-tui` binary (interactive terminal UI)
cloudfits-web/       Static HTML/JS served by `cloudfits serve`
```

## Adding a provider

Add an entry to `default_providers()` in `cloudfits-common/src/lib.rs`. Include the
per-million-token input/output prices, optional hourly rate, and your best-effort
`switching_effort` / `setup_effort` (0.0 = trivial .. 1.0 = very hard). Show your
sources in the PR description.

## Before submitting a PR

- Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`.
- Update the provider table in `README.md`.

## License

MIT. By contributing you agree your work is licensed under MIT.
