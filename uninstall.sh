#!/usr/bin/env bash
#
# uninstall.sh – Entfernt rubberduck samt Konfiguration und Logs.
#
# Dieses Skript ist das Gegenstück zum (curl|sh) Installer und tut dasselbe
# wie `rubberduck self uninstall` von innerhalb der App. Es entfernt:
#   - das Binary  ~/.local/bin/rubberduck
#   - das Konfig-Verzeichnis  ~/.config/rubberduck   (override: RUBBERDUCK_CONFIG_DIR)
#   - das Daten-/Log-Verzeichnis  ~/.rubberduck       (override: RUBBERDUCK_DATA_DIR)
#
# HINWEIS: Dieselbe Aufräumarbeit erledigt auch `rubberduck self uninstall`
# direkt aus der Anwendung heraus.

# POSIX-sh-kompatibel: wird per `curl ... | sh` (dash/busybox) ausgeführt.
set -eu

# --- Hilfsfunktionen -------------------------------------------------------

# err(): Schreibt eine Meldung nach stderr (kein Abbruch).
err() {
	printf '%s\n' "$*" >&2
}

# die(): Meldung nach stderr und mit Fehlercode beenden.
die() {
	err "Fehler: $*"
	exit 1
}

# --- Argument-Parsing ------------------------------------------------------

ASSUME_YES=0

usage() {
	cat <<'EOF'
Verwendung: uninstall.sh [Optionen]

Entfernt rubberduck (Binary, Konfiguration und Logs).

Optionen:
  -y, --yes      Nicht nachfragen, direkt entfernen (für Skripte/CI).
  -h, --help     Diese Hilfe anzeigen.

Berücksichtigte Umgebungsvariablen:
  RUBBERDUCK_CONFIG_DIR   Überschreibt das Konfig-Verzeichnis (~/.config/rubberduck).
  RUBBERDUCK_DATA_DIR     Überschreibt das Daten-/Log-Verzeichnis (~/.rubberduck).

Hinweis: `rubberduck self uninstall` tut dasselbe aus der App heraus.
EOF
}

while [ "$#" -gt 0 ]; do
	case "$1" in
	-y | --yes)
		ASSUME_YES=1
		;;
	-h | --help)
		usage
		exit 0
		;;
	--)
		shift
		break
		;;
	-*)
		die "Unbekannte Option: $1 (siehe --help)"
		;;
	*)
		die "Unerwartetes Argument: $1 (siehe --help)"
		;;
	esac
	shift
done

# --- Ziele auflösen --------------------------------------------------------

# HOME muss gesetzt sein, sonst lassen sich die Pfade nicht bilden.
[ -n "${HOME:-}" ] || die "HOME ist nicht gesetzt – Pfade können nicht aufgelöst werden."

# Standard-Installationsort des Binaries (frozen): ~/.local/bin/rubberduck
BIN_PATH="${HOME}/.local/bin/rubberduck"

# Konfig- und Datenverzeichnis, jeweils mit Env-Override (wie das Binary selbst).
CONFIG_DIR="${RUBBERDUCK_CONFIG_DIR:-${HOME}/.config/rubberduck}"
DATA_DIR="${RUBBERDUCK_DATA_DIR:-${HOME}/.rubberduck}"

# Prüfen, wohin `command -v rubberduck` zeigt. Liegt das aufgespürte Binary
# woanders (z. B. cargo install ~/.cargo/bin oder Homebrew), warnen wir, denn
# dieses Skript entfernt nur den Standard-Pfad ~/.local/bin/rubberduck.
RESOLVED_BIN=""
if RESOLVED_BIN="$(command -v rubberduck 2>/dev/null)"; then
	# Pfade normalisieren (Symlinks etc. interessieren uns hier nicht groß;
	# ein direkter String-Vergleich reicht für die Warnung aus).
	if [ "$RESOLVED_BIN" != "$BIN_PATH" ]; then
		err "Warnung: 'command -v rubberduck' zeigt auf:"
		err "    $RESOLVED_BIN"
		err "  Dieses Skript entfernt nur den Standard-Pfad:"
		err "    $BIN_PATH"
		err "  Das andere Binary (z. B. via 'cargo install' oder Homebrew)"
		err "  musst du separat entfernen, etwa mit:"
		err "    cargo uninstall rubberduck-cli   # bei cargo-Installation"
		err "    brew uninstall rubberduck        # bei Homebrew-Installation"
		err ""
	fi
fi

# --- Übersicht: Was existiert tatsächlich? ---------------------------------

# Vorhandene Ziele zählen (POSIX-kompatibel, ohne bash-Arrays).
FOUND=0
for p in "$BIN_PATH" "$CONFIG_DIR" "$DATA_DIR"; do
	if [ -e "$p" ] || [ -L "$p" ]; then
		FOUND=$((FOUND + 1))
	fi
done

echo "rubberduck-Deinstallation"
echo "========================="
echo
echo "Folgende Pfade werden geprüft und – falls vorhanden – entfernt:"
printf '  Binary:        %s%s\n' "$BIN_PATH" "$([ -e "$BIN_PATH" ] && echo '' || echo '   (nicht vorhanden)')"
printf '  Konfiguration: %s%s\n' "$CONFIG_DIR" "$([ -e "$CONFIG_DIR" ] && echo '' || echo '   (nicht vorhanden)')"
printf '  Logs/Daten:    %s%s\n' "$DATA_DIR" "$([ -e "$DATA_DIR" ] && echo '' || echo '   (nicht vorhanden)')"
echo

# Nichts vorhanden? Dann sind wir fertig.
if [ "$FOUND" -eq 0 ]; then
	echo "Es wurde nichts gefunden, das entfernt werden müsste. Fertig."
	exit 0
fi

# --- Bestätigung einholen --------------------------------------------------

if [ "$ASSUME_YES" -ne 1 ]; then
	# Non-interactive guard: kein TTY und kein --yes => abbrechen mit Anleitung.
	if [ ! -t 0 ]; then
		die "Keine interaktive Eingabe möglich (stdin ist kein Terminal). Mit '--yes' bzw. '-y' erneut ausführen, um ohne Rückfrage zu entfernen."
	fi

	printf 'Diese Pfade wirklich entfernen? [y/N] '
	read -r reply
	case "$reply" in
	[yY] | [yY][eE][sS])
		: # bestätigt – weiter
		;;
	*)
		echo "Abgebrochen – nichts wurde entfernt."
		exit 0
		;;
	esac
fi

# --- Entfernen -------------------------------------------------------------

# remove_path(): Entfernt eine Datei oder ein Verzeichnis, toleriert Fehlen.
remove_path() {
	path="$1"
	label="$2"
	if [ ! -e "$path" ] && [ ! -L "$path" ]; then
		# Bereits weg (oder nie da gewesen) – kein Fehler.
		return 0
	fi
	if [ -d "$path" ] && [ ! -L "$path" ]; then
		rm -rf -- "$path" || die "Konnte $label nicht entfernen: $path"
	else
		rm -f -- "$path" || die "Konnte $label nicht entfernen: $path"
	fi
	echo "Entfernt: $label  ($path)"
}

remove_path "$BIN_PATH" "Binary"
remove_path "$CONFIG_DIR" "Konfiguration"
remove_path "$DATA_DIR" "Logs/Daten"

echo
echo "rubberduck wurde entfernt. Danke fürs Quaken!"
