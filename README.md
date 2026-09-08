# cloudfits

<p align="center">
  <a href="https://github.com/niranjanaryan/cloudfits/actions"><img src="https://github.com/niranjanaryan/cloudfits/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/niranjanaryan/cloudfits"><img src="https://img.shields.io/github/stars/niranjanaryan/cloudfits?style=social" alt="GitHub stars"></a>
  <a href="https://github.com/sponsors/niranjanaryan"><img src="https://img.shields.io/badge/❤%EF%B8%8F-Sponsor-ff69b4" alt="Sponsor"></a>
</p>

Estimates the **real cost of running an LLM — per hour and per month** — whether you
self-host it or call a hosted inference API, and makes it **easy to compare and switch
between inference providers**.

It's a FOSS companion to [llmfit](https://github.com/AlexsJones/llmfit) (which finds
which open models fit *your hardware*). `cloudfits` answers the money questions:

- How much does it cost to run model X for an hour? For a month?
- Should I self-host it or use a hosted API?
- Which provider is cheapest for my workload?
- How hard is it to switch from provider A to provider B, and what does it cost me?

## Quick start

```sh
# Build
cargo build --release

# Cost report: local vs a cloud/API provider
./target/release/cloudfits cost --model llama-3.1-8b-instruct

# Cost against a specific provider
./target/release/cloudfits cost --model llama-3.1-8b-instruct --provider groq --format json

# List all providers + pricing + switching effort
./target/release/cloudfits providers

# Compare local vs every provider for a model
./target/release/cloudfits compare --model llama-3.1-8b

# How much would switching from AWS Bedrock to Together save me?
./target/release/cloudfits switch --model llama-70b --from bedrock --to together

# Dump the detected hardware profile (drives the local cost estimate)
./target/release/cloudfits system
```

Interactive terminal UI (browse per-provider estimates):

```sh
./target/release/cloudfits-tui
```

Web UI + JSON API:

```sh
./target/release/cloudfits serve
# http://localhost:8787
```

## What it calculates

**Local (self-hosted) cost per hour** = electricity + hardware depreciation:

```
electricity/hr     = (TDP_watts × duty_cycle / 1000) × electricity_per_kWh
depreciation/hr    = hardware_value / (3 yr × 8 760 hr)
cost_per_hour      = electricity/hr + depreciation/hr
cost_per_month     = cost_per_hour × hours_per_day × days_per_month
```

**Cloud / API cost per hour** = tokens moved through the API:

```
tokens/hr         = tokens_per_second × 3600 × duty_cycle
cost/hr           = input_tokens_M × input_$/Mtok + output_tokens_M × output_$/Mtok
                    (GPU rental providers use their hourly rate as a floor)
```

`tokens_per_second` comes from a memory-bandwidth model:

```
tps = bandwidth_GBps × 1e9 / (params_billions × 1e9 × 0.55 bytes/param)
```

Hardware (CPU cores, RAM, GPU/VRAM, bandwidth, TDP) is auto-detected — NVIDIA
(`nvidia-smi`), AMD (`rocm-smi`), and Apple Silicon (unified memory).

## Providers

| Provider | Kind | In $/Mtok | Out $/Mtok | $/hr | Switch effort |
|---|---|---|---|---|---|
| Ollama | local | 0 | 0 | – | 10% |
| Fireworks | api | 0.10 | 0.10 | – | 20% |
| Together AI | api | 0.15 | 0.15 | – | 20% |
| OpenRouter | api | 0.25 | 1.25 | – | 15% |
| Google Vertex | cloud | 0.35 | 1.25 | – | 50% |
| Groq | api | 0.59 | 0.79 | – | 20% |
| Azure OpenAI | cloud | 0.60 | 1.80 | – | 60% |
| AWS Bedrock | cloud | 0.80 | 2.40 | – | 60% |
| RunPod | gpu-rental | 0 | 0 | 0.69 | 40% |
| Lambda GPU | gpu-rental | 0 | 0 | 0.99 | 40% |

"Switch effort" is our subjective 0–100% estimate of how hard it is to move a
workload to that provider (SDK changes, auth, quotas, monitoring). Lower = easier.

## Tune the scenario

Costs depend heavily on your usage. CLI flags let you adjust everything:

```sh
# higher electricity prices
cloudfits cost --model llama-70b --electricity 0.30

# run only 4 hours/day, 22 days/month
cloudfits cost --model llama-70b --hours-per-day 4 --days 22
```

## Project layout

```
cloudfits/
├── Cargo.toml                  (Cargo workspace)
├── cloudfits-common/           Shared cost models, provider catalog, hardware detection
├── cloudfits-cli/              The `cloudfits` binary (CLI + web API)
└── cloudfits-tui/              The `cloudfits-tui` binary (interactive terminal UI)
```

## Install

```sh
# From source (requires Rust)
cargo install --path cloudfits-cli --locked
cargo install --path cloudfits-tui --locked

# Or via the installer script
./install.sh
```

## Support & Sponsoring

cloudfits is free and open source (MIT). If it saves you or your team time and
money, consider supporting its maintenance and growth:

- **❤️ Sponsor the project** via [GitHub Sponsors](https://github.com/sponsors/niranjanaryan).
- See [SPONSORS.md](SPONSORS.md) for tiers, corporate/one-time options, and transparency.

## License

MIT
