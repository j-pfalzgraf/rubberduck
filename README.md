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

![License](https://img.shields.io/badge/license-MIT-blue)
![Version](https://img.shields.io/badge/version-0.1.0-green)

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

The interface is **internationalized and defaults to English**; German ships
bundled (`--lang de`).

## Features

- 🦆 **Animated ASCII duck** — swims in, blinks, quacks; typewriter effect for the
  questions. Degrades cleanly to static / `--quiet` output without a terminal.
- 🌍 **Internationalized** — English by default, German included; pick with
  `--lang`, `config.yaml`, or `RUBBERDUCK_LANG`.
- 💬 **Interactive question dialog** — step by step through your problem.
- 🎨 **Themes & colours** — `classic`, `midnight`, `mono`; honours `NO_COLOR`.
- 🧭 **Topic selection** — interactive picker or `--topic` directly.
- 💡 **Aha moment** — type `!aha` when you find the bug: a celebration animation
  and a marker in the log.
- 📊 **Statistics** — time to solution and per question in the summary.
- 📝 **Markdown log** — optional via `--log`, including stats & the aha note.
- 🧩 **Editable question pool** with topics & descriptions, per language.
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

## Usage

| Command                            | Description                                       |
| ---------------------------------- | ------------------------------------------------- |
| `rubberduck`                       | start a session (topic picker if no `--topic`)    |
| `rubberduck --topic logic`         | a question set directly (`default`/`logic`/`perf`/`api`) |
| `rubberduck --lang de`             | switch the language (`en` / `de`)                 |
| `rubberduck --log`                 | save the session as Markdown                      |
| `rubberduck --quiet`               | no duck/animation, just concise text              |
| `rubberduck --no-anim`             | static duck (no typewriter/swim)                  |
| `rubberduck --speed fast`          | animation speed (`slow`/`normal`/`fast`)          |
| `rubberduck --color never`         | force/disable colour (`auto`/`always`/`never`)    |
| `rubberduck --theme midnight`      | colour scheme (`classic`/`midnight`/`mono`)       |
| `rubberduck topics`                | show the available topics with descriptions       |
| `rubberduck completions zsh`       | print shell completions                           |
| `rubberduck config show`           | show settings (`init`/`show`/`path`)              |
| `rubberduck --version`             | print the version                                 |
| `rubberduck self update [--check]` | update (`--check`: check only)                    |
| `rubberduck self uninstall`        | remove rubberduck along with config and logs      |

## Languages (i18n)

English is the default. German is bundled. The language is resolved as:

**`--lang` flag › `RUBBERDUCK_LANG` env › `config.yaml` › English.**

```sh
rubberduck --lang de             # German for this run
RUBBERDUCK_LANG=de rubberduck    # via environment
```

To make it permanent, set `language: de` in `config.yaml` (see below). Each
language has its own question pool file, so you can curate questions per
language.

## The aha moment

Once you find the bug, **type `!aha`** as your answer (optionally with a note,
e.g. `!aha index was one too large`). The duck celebrates briefly and marks the
moment in the log. At the end of a session it also asks whether you found it. The
summary shows the time to solution and the average per question.

## Animations & appearance

- **`--theme`** `classic` (yellow duck), `midnight` (dark, RGB) or `mono`.
- **`--speed`** `slow` / `normal` / `fast` controls the pace.
- **`--no-anim`** shows everything statically (one duck, no typewriter) — handy
  over SSH or on slow terminals.
- **`--color`** `auto` (default) / `always` / `never`. Without a terminal (pipe,
  CI) and when `NO_COLOR` is set, output is not coloured and animations fall back
  to static rendering.

## Topics

`rubberduck topics` shows all topics with their descriptions (`*` marks the
default). Without `--topic`, and in a real terminal, an interactive picker
appears.

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
win. Manage it with `rubberduck config init|show|path`. Example with the defaults:

```yaml
color: auto          # auto | always | never
theme: classic       # classic | midnight | mono
animations: true
speed: normal        # slow | normal | fast
typewriter: true
default_topic: default
language: en          # en | de
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

| Module        | Responsibility                                                    |
| ------------- | ----------------------------------------------------------------- |
| `i18n`        | languages (`Lang`) and the `Tr` translator (all user strings)     |
| `ui::theme`   | colour schemes + `Styler` (colour on/off, `NO_COLOR`)             |
| `ui::surface` | `Surface` trait: `TermSurface` (crossterm) / `BufferSurface` (tests) |
| `ui::animate` | `Animation` trait, `Player`, `Frame`, `Easing`                    |
| `ui::duck`    | DRY pose builder + swim/quack/celebrate animations                |
| `ui::scene`   | `SpeechScene`: typewriter speech bubble over a live duck          |
| `ui` (`Ui`)   | facade: resolves TTY/colour, degrades cleanly                     |
| `app`         | controller: topic selection, question dialog, aha, statistics     |
| `questions` / `session` / `config` | data and state layer                         |
| `cli` / `selfcmd` / `paths` | arguments, update/uninstall, paths                 |

The animation engine only knows the `Surface` trait — which is why it runs in
tests against an in-memory buffer instead of a real terminal.

## Configuration & data locations

| Purpose          | Path                                       | Contents            |
| ---------------- | ------------------------------------------ | ------------------- |
| Questions        | `~/.config/rubberduck/questions.<lang>.yaml` | topics & questions  |
| Settings         | `~/.config/rubberduck/config.yaml`         | theme, speed, lang … |
| Logs / data      | `~/.rubberduck/`                           | `session-<date>.md` |
| Config override  | `$RUBBERDUCK_CONFIG_DIR`                   | overrides config path |
| Data override    | `$RUBBERDUCK_DATA_DIR`                     | overrides data path |
| Language override | `$RUBBERDUCK_LANG`                        | `en` / `de`         |

> The paths are laid out the same on every platform. On Windows `~` stands for
> `%USERPROFILE%`.

## Planned

- Signature verification (ed25519) for `rubberduck self update`
- Homebrew tap
- More duck moods, themes and languages

## License

License: **MIT** — see [LICENSE](LICENSE).
