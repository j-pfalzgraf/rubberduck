# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-06-08

First stable release. Earlier `0.x` builds were pre-releases; this entry
describes the full feature set as shipped.

### Added

#### Core debugging flow

- Offline rubber-duck debugging companion: an animated ASCII duck asks
  structured questions until you find the bug yourself. Every session runs fully
  offline, with no external AI and no network access.
- Interactive, step-by-step question dialog driven by a curated question pool.
- Topic selection via an interactive picker, or directly with `--topic`
  (`default`, `logic`, `perf`, `api`); `rubberduck topics` lists all topics with
  descriptions.
- Editable question pool at `~/.config/rubberduck/questions.<lang>.yaml`, created
  on first run, with a lean list form and a rich form (description plus questions)
  per language, including custom topics.
- The "aha moment": type `!aha` (optionally with a note) the instant you find the
  bug to trigger a celebration and mark it in the log; at the end of a session
  rubberduck also asks whether you found it.
- End-of-session summary with statistics: questions answered, time to solution
  and average time per question.
- Optional Markdown session log via `--log`, saved under
  `~/.rubberduck/session-<date>.md`, including topic, duration, statistics, the
  aha note and all question/answer pairs in the active language; same-day
  sessions are appended.
- Session history and aggregate statistics: `rubberduck stats` shows sessions,
  solve rate, average session length and average time to solution with an
  animated per-topic bar chart; `--json` for machine-readable output and
  `--reset` to clear recorded history. History is appended locally as JSON lines
  to `~/.rubberduck/history.jsonl` and stays fully offline.

#### Animations

- Animated ASCII duck that swims in, blinks, quacks and celebrates, with moods
  including idle, thinking, listening, happy, curious, surprised, celebrating and
  sleeping.
- Typewriter speech bubble that renders questions character by character over a
  live duck.
- Gradient rainbow confetti and EUREKA celebration banner for the aha moment.
- `rubberduck demo`: an animated tour of every effect — the gradient title, the
  swim-in entrance, the typewriter bubble, a quack, every spinner style
  (braille, dots, line, arc, bounce, moon), all moods, a colour preview of every
  theme and the confetti celebration.
- Six colour themes — `classic`, `midnight`, `mono`, `ocean`, `forest`, `candy` —
  and three animation speeds (`slow`, `normal`, `fast`).
- Clean degradation: without a terminal (pipe, CI) or with `--quiet`,
  `--no-anim`, `--color never` or `NO_COLOR`, output falls back to static,
  uncoloured rendering.

#### Internationalization

- Internationalized interface, English by default, with German (`--lang de`) and
  French (`--lang fr`) bundled; `rubberduck languages` lists them.
- Language resolution order: `--lang` flag, then `RUBBERDUCK_LANG`, then
  `config.yaml`, then English.
- Per-language question pools, so questions can be curated separately per
  language.
- Compile-time-complete translation catalogs: each language is a single data
  struct the compiler checks for completeness, so adding a language can never
  silently omit a message.

#### CLI & configuration

- Presentation flags `--quiet`, `--no-anim`, `--speed`, `--color`, `--theme` and
  `--lang`, usable globally (also after a subcommand); CLI flags always override
  config.
- Persistent settings in `~/.config/rubberduck/config.yaml` (colour, theme,
  animations, speed, typewriter, default topic, language, history), managed via
  `rubberduck config init|show|path|set|reset`. A broken config is reported once
  and falls back to defaults rather than failing.
- Path overrides via `RUBBERDUCK_CONFIG_DIR`, `RUBBERDUCK_DATA_DIR` and
  `RUBBERDUCK_LANG`, laid out the same on every platform.
- Shell completions for bash, zsh, fish, PowerShell and elvish via
  `rubberduck completions <shell>`.
- Man page generation (roff) via `rubberduck man`.
- `--version` to print the version.

#### Distribution

- `curl | sh` installer for Linux/macOS and PowerShell installer for Windows,
  installing to `~/.local/bin` (Linux/macOS) or
  `%LOCALAPPDATA%\Programs\rubberduck` (Windows), with a PATH hint when needed.
- Cargo install: `cargo install rubberduck-cli` (binary installed as
  `rubberduck`).
- Self-management: `rubberduck self update` (with `--check` to only check) and
  `rubberduck self uninstall` to remove the binary, configuration and logs.
  Updates happen only on explicit command, never silently in the background.
- Portable builds via the optional `vendored-openssl` feature for static OpenSSL;
  no `cmake` required.

#### Security & supply chain

- Each release ships a `SHA256SUMS` file; the installers print the version and
  source before any action, download over HTTPS only and verify the SHA256
  checksum before installing.
- `rubberduck self update` downloads releases over HTTPS/TLS from GitHub.
  Checksum/signature verification for `self update` is not yet implemented (see
  the README "Planned" section).
- Release archives carry SLSA build provenance, verifiable with
  `gh attestation verify`.
- Each release also ships shell completions and a man page as assets.

#### CI

- CI workflow: `cargo fmt`, `clippy -D warnings`, `cargo doc -D warnings`, tests
  on Linux/macOS/Windows, an MSRV check (Rust 1.87), `shellcheck`, `actionlint`,
  a `cargo publish --dry-run`, and a CLI smoke test running the built binary in
  English, German and French.
- Audit workflow: weekly `cargo audit` against the RustSec database.
- Docs workflow: builds rustdoc with `-D warnings` and publishes to GitHub Pages
  on every push to `main`.
- Release workflow: builds six targets and attaches archives, shell completions,
  a man page and `SHA256SUMS`, attests SLSA build provenance and writes release
  notes, triggered by a `vX.Y.Z` tag.
- Dependabot keeps Cargo and Actions dependencies current.

[1.0.0]: https://github.com/j-pfalzgraf/rubberduck/releases/tag/v1.0.0
