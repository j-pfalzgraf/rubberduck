# 🦆 rubberduck

> Offline Rubber-Duck-Debugging fürs Terminal – eine **animierte** ASCII-Ente
> stellt dir strukturierte Rückfragen, bis du den Bug selbst findest.

```text
 ____________________________________
/ Was soll dein Code hier eigentlich \
\ tun?                               /
 ------------------------------------
    \
     \
  __
<( o)___
 (___/
```

![License](https://img.shields.io/badge/license-MIT-blue)
![Version](https://img.shields.io/badge/version-0.1.0-green)

> Die Badges sind generische [shields.io](https://shields.io)-Platzhalter.
>
> **Hinweis (Platzhalter):** `j-pfalzgraf/rubberduck` ist im gesamten Dokument
> ein Platzhalter für dein eigenes GitHub-`owner/repo`. Ersetze ihn überall, wo
> er in URLs oder Befehlen auftaucht, durch deine tatsächlichen Werte.

## Was ist das?

`rubberduck` ist ein kleiner Begleiter fürs Terminal nach dem Prinzip des
[Rubber-Duck-Debuggings](https://de.wikipedia.org/wiki/Rubber_Duck_Debugging):
Statt einem Kollegen erklärst du dein Problem einer Ente. Sie tippt dir –
sichtbar Zeichen für Zeichen – strukturierte Fragen in eine Sprechblase,
schwimmt ins Bild, blinzelt, quakt und feiert mit dir den Moment, in dem der
Groschen fällt.

Alle Sessions laufen **komplett offline**, ohne externe KI und ohne Netzwerk.
Nur das eingebaute Selbst-Update lädt – auf ausdrücklichen Wunsch – etwas herunter.

## Features

- 🦆 **Animierte ASCII-Ente** – schwimmt ins Bild, blinzelt, quakt; Tippeffekt
  für die Fragen. Degradiert sauber zu statischer/`--quiet`-Ausgabe ohne Terminal.
- 💬 **Interaktiver Frage-Dialog** – Schritt für Schritt durch dein Problem.
- 🎨 **Themes & Farben** – `classic`, `midnight`, `mono`; respektiert `NO_COLOR`.
- 🧭 **Themen-Auswahl** – interaktiver Picker oder direkt per `--topic`.
- 💡 **Aha-Moment** – tippe `!aha`, wenn du den Bug gefunden hast: Feier-Animation
  und ein Marker im Logbuch.
- 📊 **Statistik** – Zeit bis zur Lösung und pro Frage in der Zusammenfassung.
- 📝 **Markdown-Logbuch** – optional via `--log`, inkl. Statistik & Aha-Notiz.
- 🧩 **Editierbare `questions.yaml`** mit Themen & Beschreibungen.
- ⚙️ **Einstellungen** in `config.yaml` (Theme, Tempo, Farbe …).
- 🐚 **Shell-Completions** für bash, zsh, fish, PowerShell, elvish.
- 🔄 **Self-Update & Uninstall** – `rubberduck self update` / `self uninstall`.
- 🔌 **Komplett offline** für alle Sessions.

## So sieht eine Session aus

```text
 ___________________
/ Steckt an den     \
| Grenzen ein       |
| Off-by-One-Fehler |
\ (< statt <=)?     /
 -------------------
    \
     \
  __
<( o)___
 (___/
  Du: !aha index war 1 zu groß

   ✦  HEUREKA!  ✦
  \ ✨ /
  __
<( ^)___
 (___/
   \o/  \o/  \o/

──────── Zusammenfassung ────────
  • 4 / 5 Fragen beantwortet
  • Dauer: 3m 12s (Ø 38s pro Frage)
  • ✅ Bug gefunden
```

## Installation

### Schnellinstallation (curl | sh / PowerShell)

> Die `j-pfalzgraf/rubberduck`-URL ist ein **Platzhalter** – bitte durch dein
> `owner/repo` ersetzen.

**Linux / macOS:**

```sh
curl -fsSL https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/install.ps1 | iex
```

### Über Cargo

```sh
cargo install rubberduck-cli
```

(Das Crate heißt `rubberduck-cli`, das installierte Binary heißt `rubberduck`.)

### Homebrew

Ein `brew tap` ist **geplant/optional** und noch nicht verfügbar.

### Installationsverzeichnisse & PATH

| Plattform     | Installationsverzeichnis             |
| ------------- | ------------------------------------ |
| Linux / macOS | `~/.local/bin`                       |
| Windows       | `%LOCALAPPDATA%\Programs\rubberduck` |

> **PATH-Hinweis:** Liegt das Verzeichnis nicht in deinem `PATH`, weisen die
> Installer dich darauf hin.

## Sicherheit

- Jedes Release liefert eine `SHA256SUMS`-Datei mit den Prüfsummen aller Assets.
- Die Installer geben **vor jeder Aktion** Version und Quelle aus, laden nur über
  **HTTPS** und **verifizieren die SHA256-Prüfsumme, bevor** installiert wird.
- `rubberduck self update` lädt das Release über **HTTPS/TLS** von GitHub; eine
  Signatur-Verifikation für `self update` ist als Härtung geplant (siehe „Geplant“).
- Wie immer bei `curl | sh`: **Lies das Skript zuerst**, bevor du es ausführst.

## Verwendung

| Befehl                             | Beschreibung                                  |
| ---------------------------------- | --------------------------------------------- |
| `rubberduck`                       | Session starten (Themen-Picker, falls kein `--topic`) |
| `rubberduck --topic logic`         | direkt mit einem Fragenset (`default`/`logic`/`perf`/`api`) |
| `rubberduck --log`                 | Session als Markdown speichern                |
| `rubberduck --quiet`               | ohne Ente/Animation, nur knapper Text         |
| `rubberduck --no-anim`             | statische Ente (kein Tippeffekt/Schwimmen)    |
| `rubberduck --speed fast`          | Animationstempo (`slow`/`normal`/`fast`)      |
| `rubberduck --color never`         | Farbe erzwingen/abschalten (`auto`/`always`/`never`) |
| `rubberduck --theme midnight`      | Farbschema (`classic`/`midnight`/`mono`)      |
| `rubberduck topics`                | verfügbare Themen mit Beschreibung anzeigen   |
| `rubberduck completions zsh`       | Shell-Completions ausgeben                    |
| `rubberduck --version`             | Version anzeigen                              |
| `rubberduck self update [--check]` | aktualisieren (`--check`: nur prüfen)         |
| `rubberduck self uninstall`        | rubberduck samt Konfiguration und Logs entfernen |

## Der Aha-Moment

Sobald du den Bug gefunden hast, **tippe `!aha`** als Antwort (optional mit
Notiz, z. B. `!aha index war 1 zu groß`). Die Ente feiert kurz und markiert den
Moment im Logbuch. Am Ende einer Session fragt sie ohnehin nach, ob du fündig
wurdest. In der Zusammenfassung siehst du die Zeit bis zur Lösung und den
Schnitt pro Frage.

## Animationen & Aussehen

- **`--theme`** `classic` (gelbe Ente), `midnight` (dunkel, RGB) oder `mono`.
- **`--speed`** `slow` / `normal` / `fast` steuert das Tempo.
- **`--no-anim`** zeigt alles statisch (eine Ente, kein Tippeffekt) – praktisch
  über SSH oder bei langsamen Terminals.
- **`--color`** `auto` (Standard) / `always` / `never`. Ohne Terminal (Pipe, CI)
  und bei gesetztem `NO_COLOR` wird automatisch nicht eingefärbt, und Animationen
  reduzieren sich auf statische Ausgabe.

## Themen

Mit `rubberduck topics` siehst du alle Themen samt Beschreibung (`*` markiert das
Standardthema). Ohne `--topic` und in einem echten Terminal erscheint ein
interaktiver Auswahl-Picker.

## Fragen anpassen

Der Fragen-Pool liegt in `~/.config/rubberduck/questions.yaml` und wird beim
ersten Start angelegt. Pro Thema sind zwei Schreibweisen erlaubt:

```yaml
topics:
  # schlanke Form: nur eine Frageliste
  default:
    - "Was soll dein Code hier eigentlich tun?"
    - "Was passiert stattdessen – ganz konkret?"

  # reiche Form: mit Beschreibung für den Themen-Picker
  mein-team:
    description: "Hausinterne Checkliste"
    questions:
      - "Hast du das Feature-Flag geprüft?"
      - "Steht der Eintrag schon im Changelog?"
```

Eigene Themen erreichst du über `--topic <name>`, z. B. `rubberduck --topic mein-team`.

## Einstellungen (`config.yaml`)

Persistente Vorlieben kommen nach `~/.config/rubberduck/config.yaml`. CLI-Flags
haben immer Vorrang. Beispiel mit den Standardwerten:

```yaml
color: auto          # auto | always | never
theme: classic       # classic | midnight | mono
animations: true
speed: normal        # slow | normal | fast
typewriter: true
default_topic: default
```

Eine kaputte `config.yaml` legt rubberduck nicht lahm – es meldet das einmal und
nutzt die Standardwerte.

## Logbuch

Mit `--log` wird die Session als Markdown unter
`~/.rubberduck/session-<datum>.md` gespeichert (z. B. `session-2026-06-05.md`),
inklusive Thema, Dauer, Statistik, Aha-Notiz und allen Frage/Antwort-Paaren.
Mehrere Sessions am selben Tag werden angehängt.

## Shell-Completions

```sh
# bash (z. B. systemweit)
rubberduck completions bash | sudo tee /etc/bash_completion.d/rubberduck

# zsh (in einen $fpath-Ordner)
rubberduck completions zsh > ~/.zfunc/_rubberduck

# fish
rubberduck completions fish > ~/.config/fish/completions/rubberduck.fish
```

Unterstützt: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

## Update & Deinstallation

- **Update:** `rubberduck self update` (mit `--check` nur prüfen).
- **Deinstallation:** `rubberduck self uninstall` entfernt Binary, Konfiguration
  und Logs (nach Rückfrage). Alternativ als Einzeiler:

  ```sh
  curl -fsSL https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/uninstall.sh | sh
  ```

> Updates passieren **ausschließlich auf ausdrücklichen Befehl** – nie still im
> Hintergrund.

## Aus dem Quellcode bauen

```sh
cargo build --release
```

Das Binary liegt dann unter `target/release/rubberduck` (Windows: `.exe`).

- **Portable Builds:** Das Feature `vendored-openssl` baut OpenSSL statisch ein
  (`cargo build --release --features vendored-openssl`) – ideal für portable
  Linux-Binaries und Cross-Compiles. macOS/Windows brauchen das nicht (Secure
  Transport bzw. SChannel).
- **Toolchain:** Es wird **kein `cmake`** benötigt; lokal baut der Standardpfad
  gegen System-OpenSSL.

## Architektur

Klar getrennte Schichten, trait-basiert und testbar:

| Modul         | Aufgabe                                                            |
| ------------- | ----------------------------------------------------------------- |
| `ui::theme`   | Farbschemata + `Styler` (Farbe an/aus, `NO_COLOR`)                |
| `ui::surface` | `Surface`-Trait: `TermSurface` (crossterm) / `BufferSurface` (Tests) |
| `ui::animate` | `Animation`-Trait, `Player`, `Frame`, `Easing`                    |
| `ui::duck`    | DRY-Pose-Builder + Schwimm-/Quak-/Feier-Animationen               |
| `ui::scene`   | `SpeechScene`: Tippeffekt-Sprechblase über lebender Ente          |
| `ui` (`Ui`)   | Fassade: löst TTY/Farbe auf, degradiert sauber                    |
| `app`         | Controller: Themenwahl, Frage-Dialog, Aha, Statistik              |
| `questions` / `session` / `config` | Daten- und Zustandsschicht                   |
| `cli` / `selfcmd` / `paths` | Argumente, Update/Deinstallation, Pfade            |

Die Animations-Engine kennt nur das `Surface`-Trait – deshalb läuft sie in Tests
gegen einen Speicherpuffer statt gegen ein echtes Terminal.

## Konfiguration & Datenablage

| Zweck            | Pfad                                  | Inhalt                |
| ---------------- | ------------------------------------- | --------------------- |
| Fragen           | `~/.config/rubberduck/questions.yaml` | Themen & Fragen       |
| Einstellungen    | `~/.config/rubberduck/config.yaml`    | Theme, Tempo, Farbe … |
| Logs / Daten     | `~/.rubberduck/`                      | `session-<datum>.md`  |
| Override Config  | `$RUBBERDUCK_CONFIG_DIR`              | überschreibt Config-Pfad |
| Override Daten   | `$RUBBERDUCK_DATA_DIR`                | überschreibt Daten-Pfad  |

> Die Pfade sind plattformübergreifend gleich aufgebaut. Unter Windows steht `~`
> für `%USERPROFILE%`.

## Geplant

- Signatur-Verifikation (ed25519) für `rubberduck self update`
- Homebrew-Tap
- Weitere Enten-Stimmungen und Themes

## Lizenz

Lizenz: **MIT** – siehe [LICENSE](LICENSE).
