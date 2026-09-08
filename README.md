# cloudfits

[CI](https://github.com/niranjanaryan/cloudfits/actions) [MIT](LICENSE) [Sponsor](https://github.com/sponsors/niranjanaryan)

Estimate the **real cost of running any LLM — per hour and per month** — self-hosted or via any hosted inference provider. Compare providers and switch easily.

```sh
# Cost: local vs cloud
cloudfits cost --model llama-3.1-8b-instruct

# Compare providers
cloudfits compare --model llama-70b

# Switch: steps + cost impact
cloudfits switch --model llama-3.1-8b-instruct --from ollama --to groq

# Hardware (GPU/CPU) used for local estimates
cloudfits system

# Interactive TUI
cloudfits-tui

# Web UI + JSON API (localhost:8787)
cloudfits serve
```

## Install

```sh
# Prebuilt binaries (Linux/macOS/Windows, x86_64 + ARM64)
./install.sh

# Or from source
cargo build --release
```

Release: [v1.0.1](https://github.com/niranjanaryan/cloudfits/releases/tag/v1.0.1) · [SPONSORS.md](SPONSORS.md) · [FUNDING.yml](.github/FUNDING.yml)

MIT License — independent FOSS project (not derived from llmfit).
