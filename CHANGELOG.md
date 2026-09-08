# Changelog

All notable changes to cloudfits are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release of `cloudfits` — a FOSS toolkit to estimate LLM inference cost
  per hour and per month, and to compare/switch inference providers.
- `cloudfits cost` — per-hour and per-month cost report (local vs a cloud/API provider).
- `cloudfits providers` — list providers with pricing and switching effort.
- `cloudfits compare` — local vs every provider, sorted by cost.
- `cloudfits switch` — cost delta and migration steps between two providers.
- `cloudfits system` — dump the auto-detected hardware profile.
- `cloudfits serve` — embedded web UI + JSON API (default port 8787).
- `cloudfits-tui` — interactive terminal UI.
- Automatic hardware detection: NVIDIA (`nvidia-smi`), AMD (`rocm-smi`), Apple Silicon (unified memory).
- MIT license, CI (GitHub Actions), Docker, install script, GitHub Sponsors funding.

## [0.1.0] - 2026-09-08

- Project scaffold and initial implementation.
