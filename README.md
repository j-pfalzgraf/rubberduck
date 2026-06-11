# 🦆 rubberduck

> Offline rubber-duck debugging for your terminal — an **animated** ASCII duck
> asks you structured questions until you find the bug yourself.

```text
 ____________________________________
/ What is your code actually          \
\ supposed to do here?                /
 ------------------------------------
    \
     \
  __
<( o)___
 (___/
```

![CI](https://github.com/j-pfalzgraf/rubberduck/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue)
![Version](https://img.shields.io/badge/version-1.1.0-green)

> The badges are generic [shields.io](https://shields.io) placeholders.
>
> **Note (placeholder):** `j-pfalzgraf/rubberduck` is a placeholder for your own
> GitHub `owner/repo` throughout this document. Replace it wherever it appears in
> URLs or commands with your actual values.

## What is this?

`rubberduck` is a small terminal companion based on
[rubber-duck debugging](https://en.wikipedia.org/wiki/Rubber_duck_debugging):
instead of a colleague, you explain your problem to a duck. It types structured
questions into a speech bubble — visibly, character by character — swims in,
blinks, quacks, and celebrates the moment the penny drops with you.

Every session runs **fully offline**, with no external AI and no network. Only
the built-in self-update downloads anything, and only when you ask it to.

The interface is **internationalized and defaults to English**; German, French
and Spanish ship bundled (`--lang de` / `--lang fr` / `--lang es`).

## Features

- 🦆 **Animated ASCII duck** — swims in, blinks, quacks; typewriter effect for the
  questions, with eleven moods. Degrades cleanly to static / `--quiet` output
  without a terminal.
- 🌈 **Gradient celebration** — a rainbow confetti + EUREKA banner for the aha moment.
- 🎬 **`demo` command** — an animated tour of every effect (moods, eleven spinner
  styles, nine gradients, ten themes).
- 📈 **Stats & history** — `stats` aggregates metrics with an animated per-topic
  bar chart (volume + solve rate); `history` lists your recent sessions.
- 🌍 **Internationalized** — English by default; German, French and Spanish
  included; pick with `--lang`, `config.yaml`, or `RUBBERDUCK_LANG`. Adding a
  language is one `Catalog` literal the compiler checks for completeness.
- 💬 **Interactive question dialog** — step by step through your problem, across
  eight topics.
- 💡 **Debugging tips** — `rubberduck tip` (a random one, delivered by the duck) /
  `rubberduck tips` (the list); a closing tip ends each session.
- 🩺 **`doctor` command** — a read-only check of your environment, settings and
  bundled content.
- 🎨 **Ten themes** — `classic`, `midnight`, `mono`, `ocean`, `forest`, `candy`,
  `dracula`, `nord`, `gruvbox`, `solarized`; honours `NO_COLOR`. `rubberduck
  themes` previews them.
- 🧭 **Topic selection** — interactive picker or `--topic` directly.
- ✨ **Aha moment** — type `!aha` when you find the bug: a celebration animation
  and a marker in the log.
- 📊 **Statistics** — time to solution and per question in the summary.
- 📝 **Markdown log** — optional via `--log`, including stats & the aha note.
- 🧩 **Editable question & tips pools** with topics & descriptions, per language.
- ⚙️ **Settings** in `config.yaml` (theme, speed, colour, language …).
- 🐚 **Shell completions** for bash, zsh, fish, PowerShell, elvish.
- 🔄 **Self-update & uninstall** — `rubberduck self update` / `self uninstall`.
- 🔌 **Fully offline** for all sessions.

## What a session looks like

```text
 ___________________
/ Is there an        \
| off-by-one error   |
| at the boundaries  |
\ (< vs <=)?         /
 -------------------
    \
     \
  __
<( o)___
 (___/
  You: !aha index was one too large

   ✦  EUREKA!  ✦
  \ ✨ /
  __
<( ^)___
 (___/
   \o/  \o/  \o/

──────── Summary ────────
  • 4 / 5 questions answered
  • Duration: 3m 12s (38s avg per question)
  • ✅ Bug found
```

## Installation

### One-liners (curl | sh / PowerShell)

> The `j-pfalzgraf/rubberduck` URL is a **placeholder** — replace it with your
> `owner/repo`.

**Linux / macOS:**

```sh
curl -fsSL https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/install.ps1 | iex
```

### Via Cargo

```sh
cargo install rubberduck-cli
```

(The crate is `rubberduck-cli`, the installed binary is `rubberduck`.)

### Homebrew

A `brew tap` is **planned/optional** and not available yet.

### Install directories & PATH

| Platform      | Install directory                    |
| ------------- | ------------------------------------ |
| Linux / macOS | `~/.local/bin`                       |
| Windows       | `%LOCALAPPDATA%\Programs\rubberduck` |

> **PATH note:** if the directory is not on your `PATH`, the installers tell you.

## Security

- Every release ships a `SHA256SUMS` file with checksums for all assets.
- The installers print the version and source **before any action**, download
  over **HTTPS only**, and **verify the SHA256 checksum before** installing.
- `rubberduck self update` downloads the release over **HTTPS/TLS** from GitHub;
  signature verification for `self update` is planned (see "Planned").
- As always with `curl | sh`: **read the script first** before running it.
- Release archives carry **SLSA build provenance** (verify with
  `gh attestation verify <archive> --repo j-pfalzgraf/rubberduck`); each release
  also ships shell completions and a man page as assets.

## Usage

| Command                                | Description                                              |
| -------------------------------------- | -------------------------------------------------------- |
| `rubberduck`                           | start a session (topic picker if no `--topic`)           |
| `rubberduck --topic logic`             | a question set directly (one of eight topics)            |
| `rubberduck --lang es`                 | switch the language (`en` / `de` / `fr` / `es`)          |
| `rubberduck --log`                     | save the session as Markdown                             |
| `rubberduck --quiet`                   | no duck/animation, just concise text                     |
| `rubberduck --no-anim`                 | static duck (no typewriter/swim)                         |
| `rubberduck --speed fast`              | animation speed (`slow`/`normal`/`fast`)                 |
| `rubberduck --color never`             | force/disable colour (`auto`/`always`/`never`)           |
| `rubberduck --theme dracula`           | colour scheme (10 themes — see Appearance)               |
| `rubberduck topics`                    | show the available topics with descriptions              |
| `rubberduck languages`                 | list the interface languages                             |
| `rubberduck themes`                    | preview every colour theme                               |
| `rubberduck demo`                      | play an animated demo of every effect                    |
| `rubberduck tip`                       | a random debugging tip, delivered by the duck            |
| `rubberduck tips`                      | list every bundled debugging tip                         |
| `rubberduck stats [--reset\|--json]`   | aggregate stats (`--json` for scripts, `--reset` clears) |
| `rubberduck history [--limit N\|--json]` | your recent sessions (newest first)                    |
| `rubberduck doctor`                    | check your environment, settings and bundled content     |
| `rubberduck completions zsh`           | print shell completions                                  |
| `rubberduck man`                       | print a man page (roff)                                  |
| `rubberduck config show`               | manage settings (`init`/`show`/`path`/`set`/`reset`)     |
| `rubberduck --version`                 | print the version                                        |
| `rubberduck self update [--check]`     | update (`--check`: check only)                           |
| `rubberduck self uninstall`            | remove rubberduck along with config and logs             |

## Languages (i18n)

English is the default. German, French and Spanish are bundled. The language is
resolved as:

**`--lang` flag › `RUBBERDUCK_LANG` env › `config.yaml` › English.**

```sh
rubberduck --lang es             # Spanish for this run
RUBBERDUCK_LANG=de rubberduck    # German via environment
```

To make it permanent, set `language: fr` in `config.yaml` (see below). Each
language has its own question pool file, so you can curate questions per
language.

Internally every user-facing string lives in a per-language `Catalog` (one data
struct, one `const` per language); the `Tr` translator is a thin, `Copy`
accessor over the active catalog. Because a struct literal must set every field,
**adding a language can never silently forget a message** — the compiler refuses
to build until the new catalog is complete.

## The aha moment

Once you find the bug, **type `!aha`** as your answer (optionally with a note,
e.g. `!aha index was one too large`). The duck celebrates briefly and marks the
moment in the log. At the end of a session it also asks whether you found it. The
summary shows the time to solution and the average per question.

## Animations & appearance

- **`--theme`** one of `classic`, `midnight`, `mono`, `ocean`, `forest`, `candy`,
  `dracula`, `nord`, `gruvbox`, `solarized` (also settable for good via
  `config set theme <name>`; preview them all with `rubberduck themes`).
- **`--speed`** `slow` / `normal` / `fast` controls the pace.
- **`--no-anim`** shows everything statically (one duck, no typewriter) — handy
  over SSH or on slow terminals.
- **`--color`** `auto` (default) / `always` / `never`. Without a terminal (pipe,
  CI) and when `NO_COLOR` is set, output is not coloured and animations fall back
  to static rendering.

## Topics

`rubberduck topics` shows all topics with their descriptions (`*` marks the
default). The bundled set covers `default`, `logic`, `perf`, `api`, `build`,
`concurrency`, `memory` and `network`. Without `--topic`, and in a real terminal,
an interactive picker appears.

## Demo

`rubberduck demo` plays an animated tour of every effect — the gradient title,
the duck's fluid swim-in entrance, the typewriter speech bubble, a quack, every
spinner style (braille, dots, line, arc, bounce, moon, pulse, clock, star, pong,
breathe), all moods, every named gradient, a colour preview of every theme, and
the confetti celebration. It respects `--speed`, `--no-anim`, `--theme` and
`--color`.

## Tips

`rubberduck tip` has the duck swim in and deliver a single random debugging tip;
`rubberduck tips` lists every tip. An interactive session also ends with a gentle
parting tip. Tips are localized and live in an editable, per-language pool at
`~/.config/rubberduck/tips.<lang>.yaml` (created on first run), so your team can
curate its own.

## Doctor

`rubberduck doctor` is a quick, read-only health check: it reports the active
version and language, the resolved theme, whether colour and animations will run
in this terminal, where your config and data live, and how much content is
bundled. Handy when something looks off, or to confirm a fresh install:

```sh
rubberduck doctor
rubberduck doctor --lang es     # the same check, in Spanish
```

## Statistics & history

When enabled (the default), each finished session is appended as one JSON line to
`~/.rubberduck/history.jsonl`. `rubberduck stats` shows aggregate metrics —
sessions, solve rate, average session length and average time to solution — plus
an animated per-topic bar chart.

```sh
rubberduck stats               # show your stats (animated)
rubberduck stats --json        # machine-readable, for scripts
rubberduck stats --reset       # clear the recorded history
rubberduck history             # list recent sessions (newest first)
rubberduck history --limit 5   # only the five most recent
rubberduck history --json      # machine-readable, for scripts
rubberduck config set history off   # stop recording entirely
```

History stays **local and offline** — nothing is ever sent anywhere.

## Editing questions

The question pool lives at `~/.config/rubberduck/questions.<lang>.yaml`
(e.g. `questions.en.yaml`) and is created on first run. Each topic may use one of
two shapes:

```yaml
topics:
  # lean form: just a list of questions
  default:
    - "What is your code actually supposed to do here?"
    - "What happens instead — concretely?"

  # rich form: with a description for the topic picker
  my-team:
    description: "House checklist"
    questions:
      - "Did you check the feature flag?"
      - "Is the changelog entry there yet?"
```

Reach custom topics via `--topic <name>`, e.g. `rubberduck --topic my-team`.

## Settings (`config.yaml`)

Persistent preferences go to `~/.config/rubberduck/config.yaml`. CLI flags always
win. Manage it with `rubberduck config init|show|path|set|reset`, e.g.
`rubberduck config set theme midnight`. Example with the defaults:

```yaml
color: auto          # auto | always | never
theme: classic       # classic | midnight | mono | ocean | forest | candy | dracula | nord | gruvbox | solarized
animations: true
speed: normal        # slow | normal | fast
typewriter: true
default_topic: default
language: en          # en | de | fr | es
history: true         # record sessions for `stats`
```

A broken `config.yaml` does not take rubberduck down — it reports it once and
uses the defaults.

## Log

With `--log` the session is saved as Markdown under
`~/.rubberduck/session-<date>.md` (e.g. `session-2026-06-05.md`), including topic,
duration, statistics, the aha note and all question/answer pairs, in the active
language. Multiple sessions on the same day are appended.

## Shell completions

```sh
# bash (e.g. system-wide)
rubberduck completions bash | sudo tee /etc/bash_completion.d/rubberduck

# zsh (into a $fpath directory)
rubberduck completions zsh > ~/.zfunc/_rubberduck

# fish
rubberduck completions fish > ~/.config/fish/completions/rubberduck.fish
```

Supported: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

A man page is available too: `rubberduck man > rubberduck.1`, then view it with
`man ./rubberduck.1`.

## Update & uninstall

- **Update:** `rubberduck self update` (with `--check` to only check).
- **Uninstall:** `rubberduck self uninstall` removes the binary, configuration
  and logs (after confirmation). Or the one-liner:

  ```sh
  curl -fsSL https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/uninstall.sh | sh
  ```

> Updates happen **only on an explicit command** — never silently in the
> background.

## Building from source

```sh
cargo build --release
```

The binary is then at `target/release/rubberduck` (Windows: `.exe`).

- **Portable builds:** the `vendored-openssl` feature builds OpenSSL statically
  (`cargo build --release --features vendored-openssl`) — ideal for portable
  Linux binaries and cross-compiles. macOS/Windows don't need it (Secure
  Transport / SChannel).
- **Toolchain:** no `cmake` is required; locally the default path builds against
  the system OpenSSL.

## Architecture

Clearly separated layers, trait-based and testable:

| Module                               | Responsibility                                                       |
| ------------------------------------ | -------------------------------------------------------------------- |
| `i18n`                               | languages (`Lang`) and the `Tr` translator (all user strings)        |
| `ui::theme`                          | colour schemes + `Styler` (colour on/off, `NO_COLOR`)                |
| `ui::surface`                        | `Surface` trait: `TermSurface` (crossterm) / `BufferSurface` (tests) |
| `ui::animate`                        | `Animation` trait, `Player`, `Frame`, `Easing`                       |
| `ui::duck`                           | DRY pose builder + swim/quack/celebrate animations                   |
| `ui::scene`                          | `SpeechScene`: typewriter speech bubble over a live duck             |
| `ui::gradient`                       | RGB gradients for the banner, confetti and charts                    |
| `ui::bar`                            | reusable animated bar charts + inline meters (`GrowingBars`)         |
| `ui` (`Ui`)                          | facade: resolves TTY/colour, degrades cleanly                        |
| `app`                                | controller: topic selection, question dialog, aha                    |
| `demo` / `stats` / `history`         | the demo tour, the stats view, session history & listing             |
| `tips` / `doctor`                    | the tips pool; the environment health check                          |
| `questions` / `session` / `config`   | data and state layer                                                 |
| `cli` / `selfcmd` / `paths` / `util` | arguments, update/uninstall, paths, helpers                          |

The animation engine only knows the `Surface` trait — which is why it runs in
tests against an in-memory buffer instead of a real terminal.

## Configuration & data locations

| Purpose           | Path                                         | Contents                             |
| ----------------- | -------------------------------------------- | ------------------------------------ |
| Questions         | `~/.config/rubberduck/questions.<lang>.yaml` | topics & questions                   |
| Tips              | `~/.config/rubberduck/tips.<lang>.yaml`      | debugging tips                       |
| Settings          | `~/.config/rubberduck/config.yaml`           | theme, speed, lang …                 |
| Logs / data       | `~/.rubberduck/`                             | `session-<date>.md`, `history.jsonl` |
| Config override   | `$RUBBERDUCK_CONFIG_DIR`                     | overrides config path                |
| Data override     | `$RUBBERDUCK_DATA_DIR`                       | overrides data path                  |
| Language override | `$RUBBERDUCK_LANG`                           | `en` / `de` / `fr` / `es`            |

> The paths are laid out the same on every platform. On Windows `~` stands for
> `%USERPROFILE%`.

## Continuous integration

GitHub Actions cover the project end to end:

- **CI** (`ci.yml`): `cargo fmt`, `clippy -D warnings`, `cargo doc -D warnings`,
  tests on Linux/macOS/Windows, an MSRV check (Rust 1.88), `shellcheck`,
  `actionlint` (lints the workflows themselves and their inline shell), a
  `cargo publish --dry-run` so the `cargo install` path can't silently break,
  and a CLI smoke test that runs the built binary in English, German, French and
  Spanish and exercises every command.
- **Coverage** (`coverage.yml`): `cargo-llvm-cov` produces an LCOV report and a
  line-coverage summary on every push and PR.
- **Lint** (`lint.yml`): `typos` (spell-check) and `cargo-deny` (dependency
  licences, advisories and bans against `deny.toml`).
- **Audit** (`audit.yml`): weekly `cargo audit` against the RustSec database.
- **Release** (`release.yml`): builds the six targets and attaches the archives,
  shell completions, a man page and `SHA256SUMS`; attests SLSA build provenance
  and writes release notes — all triggered by a `vX.Y.Z` tag.
- **Labeler** (`labeler.yml`): labels pull requests by the areas they touch.
- **Dependabot** keeps Cargo and Actions dependencies current.

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to add a language, topic, tip or
theme.

## Planned

- Signature verification (ed25519) for `rubberduck self update`
- Homebrew tap
- More languages (the catalog makes each new one a single data struct)

## License

License: **MIT** — see [LICENSE](LICENSE).
