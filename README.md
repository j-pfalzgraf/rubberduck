# 🦆 rubberduck

> Offline Rubber-Duck-Debugging fürs Terminal – eine ASCII-Ente stellt dir
> strukturierte Rückfragen, bis du den Bug selbst findest.

```text
   __
 <(o )___
  ( ._> /   "Was soll dein Code an dieser Stelle eigentlich tun?"
   `---'
```

![License](https://img.shields.io/badge/license-MIT-blue)
![Version](https://img.shields.io/badge/version-0.1.0-green)

> Die Badges sind generische [shields.io](https://shields.io)-Platzhalter, keine
> Live-Status-Badges.

> **Hinweis (Platzhalter):** `j-pfalzgraf/rubberduck` ist im gesamten Dokument ein
> Platzhalter für dein eigenes GitHub-`owner/repo`. Ersetze ihn überall, wo er in
> URLs oder Befehlen auftaucht, durch deine tatsächlichen Werte.

## Was ist das?

`rubberduck` ist ein kleiner Begleiter fürs Terminal nach dem Prinzip des
[Rubber-Duck-Debuggings](https://de.wikipedia.org/wiki/Rubber_Duck_Debugging):
Statt einem Kollegen erklärst du dein Problem einer ASCII-Ente. Die Ente stellt
dir dabei strukturierte Fragen – Schritt für Schritt –, sodass du dein Problem
durchdenkst und die Lösung oft schon beim Erklären findest.

Die Sessions laufen **komplett offline**, ohne externe KI und ohne Netzwerk. Nur
das eingebaute Selbst-Update lädt (auf ausdrücklichen Wunsch) etwas herunter.

## Features

- **Interaktiver Frage-Antwort-Dialog** – die Ente führt dich durch dein Problem.
- **ASCII-Ente mit Sprechblasen** – freundlicher Begleiter direkt im Terminal.
- **Optionales Markdown-Logbuch** – speichere die Session zum Nachlesen (`--log`).
- **Editierbare `questions.yaml`** mit Themen – ergänze eigene Fragensets und teile
  sie im Team.
- **Eingebautes Self-Update & Uninstall** – `rubberduck self update` / `self uninstall`.
- **Komplett offline** für alle Sessions – nichts verlässt deinen Rechner.

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

> **PATH-Hinweis:** Liegt das Installationsverzeichnis nicht in deinem `PATH`,
> warnen die Installer dich entsprechend. Füge das Verzeichnis dann zu deinem
> `PATH` hinzu, damit `rubberduck` direkt aufrufbar ist.

## Sicherheit

- Jedes Release liefert eine `SHA256SUMS`-Datei mit den Prüfsummen aller Assets.
- Die Installer geben **vor jeder Aktion** die aufgelöste Version und die
  Quell-URL aus, laden ausschließlich über **HTTPS** und **verifizieren die
  SHA256-Prüfsumme des Archivs gegen `SHA256SUMS`, bevor** entpackt oder
  installiert wird. Bei Abweichung wird abgebrochen.
- Wie immer bei `curl | sh`: **Lies das Skript zuerst**, bevor du es ausführst –
  öffne die URL einfach im Browser.
- `rubberduck self update` lädt das Release über **HTTPS/TLS** von GitHub. Die
  zusätzliche SHA256-Prüfung erledigen die Installer; eine Signatur-Verifikation
  für `self update` ist als nächste Härtung geplant (siehe „Geplant").

## Verwendung

| Befehl                             | Beschreibung                                            |
| ---------------------------------- | ------------------------------------------------------- |
| `rubberduck`                       | Standard-Session                                        |
| `rubberduck --topic logic`         | Themen-Fragen (`default` \| `logic` \| `perf` \| `api`) |
| `rubberduck --log`                 | Session als Markdown speichern                          |
| `rubberduck --quiet`               | ohne ASCII-Ente                                         |
| `rubberduck --version`             | Version anzeigen                                        |
| `rubberduck self update [--check]` | aktualisieren (`--check`: nur prüfen)                   |
| `rubberduck self uninstall`        | rubberduck samt Konfiguration und Logs entfernen        |

## Fragen anpassen

Der Fragen-Pool liegt in `~/.config/rubberduck/questions.yaml` und wird beim
ersten Start automatisch angelegt. Das Format ist eine Map `topics` von
Themenname auf eine Liste von Fragen (Strings):

```yaml
topics:
  default:
    - "Was soll dein Code an dieser Stelle eigentlich tun?"
    - "Was passiert stattdessen – ganz konkret?"
  logic:
    - "Stimmen deine Bedingungen wirklich (==, <, >, Klammern, Negationen)?"
  mein-team:
    - "Hast du daran gedacht, das Feature-Flag zu prüfen?"
```

Dein Team kann beliebig **eigene Themen ergänzen**. Eigene Themen erreichst du
über `--topic <name>`, z. B. `rubberduck --topic mein-team`.

> **Overrides:** Mit den Umgebungsvariablen `RUBBERDUCK_CONFIG_DIR` und
> `RUBBERDUCK_DATA_DIR` kannst du Konfigurations- bzw. Datenverzeichnis frei
> festlegen (praktisch für Tests oder portable Setups).

## Logbuch

Mit `--log` wird die Session als Markdown unter
`~/.rubberduck/session-<datum>.md` gespeichert (z. B. `session-2026-06-05.md`).
Mehrere Sessions am selben Tag werden an dieselbe Datei angehängt.

## Update & Deinstallation

- **Update:** `rubberduck self update` aktualisiert auf die neueste Release-Version.
  Mit `rubberduck self update --check` prüfst du nur, ob ein Update verfügbar ist,
  ohne etwas zu installieren.
- **Deinstallation:** `rubberduck self uninstall` entfernt das Binary samt
  Konfiguration und Logs. Alternativ als Einzeiler:

  ```sh
  curl -fsSL https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/uninstall.sh | sh
  ```

  (Auch hier `j-pfalzgraf/rubberduck` durch dein **owner/repo** ersetzen.)

> Updates passieren **ausschließlich auf ausdrücklichen Befehl** – nie still und
> nie automatisch im Hintergrund.

## Aus dem Quellcode bauen

```sh
cargo build --release
```

Das fertige Binary liegt anschließend unter `target/release/rubberduck`
(unter Windows `rubberduck.exe`).

- **Portable Builds:** Mit dem Feature `vendored-openssl` wird OpenSSL aus dem
  Quellcode statisch eingebaut – ideal für portable Release-Binaries und
  Cross-Compiles:

  ```sh
  cargo build --release --features vendored-openssl
  ```

- **Toolchain:** Es wird **kein `cmake`** benötigt. Lokal baut der Standardpfad
  gegen das System-OpenSSL; `vendored-openssl` braucht lediglich `perl` und
  `make`.

## Konfiguration & Datenablage

| Zweck                  | Pfad                     | Inhalt                   |
| ---------------------- | ------------------------ | ------------------------ |
| Konfiguration          | `~/.config/rubberduck`   | `questions.yaml`         |
| Logs / Daten           | `~/.rubberduck`          | `session-<datum>.md`     |
| Override Konfiguration | `$RUBBERDUCK_CONFIG_DIR` | überschreibt Config-Pfad |
| Override Daten         | `$RUBBERDUCK_DATA_DIR`   | überschreibt Daten-Pfad  |

> Die Pfade sind auf allen Plattformen gleich aufgebaut. Unter Windows steht `~`
> für `%USERPROFILE%`, also `%USERPROFILE%\.config\rubberduck` bzw.
> `%USERPROFILE%\.rubberduck`.

## Geplant

Ideen für die Zukunft (noch **nicht** enthalten):

- Performance-Statistiken zu Sessions
- „Aha"-Marker, um den Moment der Erkenntnis festzuhalten
- Homebrew-Tap
- Signatur-Verifikation (ed25519) für `rubberduck self update`

## Lizenz

Lizenz: **MIT** – siehe [LICENSE](LICENSE).
