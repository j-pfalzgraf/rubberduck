# Contributing to rubberduck

Thanks for wanting to make the duck better! This guide covers the day-to-day
workflow and the four most common "I want to add one more" tasks: a language, a
topic, a tip and a theme. The architecture is built so each of these is a small,
local change the compiler helps you get right.

## Development workflow

```sh
cargo test            # unit + integration + doc tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo run -- demo     # see the animations
```

CI runs all of the above plus a cross-platform test matrix, an MSRV check
(Rust 1.88), coverage, `typos`, `cargo-deny` and a CLI smoke test. Run
`cargo fmt`, `cargo clippy` and `cargo test` before opening a PR and you should
be green.

Guiding principles, in rough priority order:

1. **Offline always.** A debugging session must never touch the network. Only
   `self update` may, and only on explicit request.
2. **Default to English**, but never hard-code a user-facing string — every one
   lives in the `i18n` catalog so all languages stay in lock-step.
3. **Degrade cleanly.** Everything must work without a terminal, with `--quiet`,
   `--no-anim`, `--color never` or `NO_COLOR`.
4. **DRY and testable.** Animations talk only to the `Surface` trait, so they are
   tested against an in-memory buffer.

## Add a language

Every user-facing string lives in a per-language `Catalog` in
[`src/i18n.rs`](src/i18n.rs). Because a `const Catalog` literal must set *every*
field, the compiler refuses to build until a new language is fully translated —
a missing message is a compile error, not a runtime surprise.

1. Add a variant to the `Lang` enum with its `#[value(name = "xx", …)]` and
   `#[serde(rename = "xx")]` attributes, and extend `ALL`, `code`, `label`,
   `from_code` and `cat`.
2. Add a `const XX: Catalog = Catalog { … }` literal next to `EN`/`DE`/`FR`/`ES`.
   Translate every field; keep `{placeholder}` markers intact.
3. Add the bundled content files `questions.xx.yaml` and `tips.xx.yaml`, and the
   matching `include_str!` arms in
   [`src/questions.rs`](src/questions.rs) and [`src/tips.rs`](src/tips.rs).
4. `cargo test` — the i18n tests check that every placeholder still fills, every
   language ships the same topic set, and tip counts match across languages.

## Add a topic

Topics are pure data — no code change needed. Edit every `questions.<lang>.yaml`
and add the same topic key under `topics:`, in either form:

```yaml
my-topic:
  description: "Short text for the picker."
  questions:
    - "First question?"
    - "Second question?"
```

Keep the topic **key** identical across languages (only the text differs); the
`every_language_ships_the_same_topic_set` test enforces this. Reach a custom
topic with `rubberduck --topic my-topic`.

## Add a tip

Add one line to every `tips.<lang>.yaml` under `tips:`. Keep the order aligned
across languages so the counts match (a test checks this). Tips are short, single
sentences — one idea each.

## Add a theme

In [`src/ui/theme.rs`](src/ui/theme.rs): add a `pub const MYTHEME: Theme = …`,
append its name to `Theme::NAMES`, and add an arm to `Theme::by_name`. The
`themes`/`demo` previews and the `--theme` value parser pick it up automatically
from `NAMES`. New spinner styles and gradients follow the same "add to the enum /
list, the showcases follow" pattern in `ui::spinner` and `ui::gradient`.

## Tests

- **Unit tests** live next to the code in `#[cfg(test)] mod tests`.
- **Integration tests** drive the built binary in [`tests/cli.rs`](tests/cli.rs)
  with isolated `RUBBERDUCK_CONFIG_DIR`/`RUBBERDUCK_DATA_DIR` and `NO_COLOR=1`.

New behaviour should come with a test. Thanks, and happy quacking! 🦆
