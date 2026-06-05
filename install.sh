#!/usr/bin/env bash
#
# install.sh — Installationsskript für "rubberduck" (rubberduck-cli)
#
# Lädt das passende Release-Archiv von GitHub herunter, verifiziert die
# SHA256-Prüfsumme gegen die SHA256SUMS-Datei VOR dem Entpacken und
# installiert die Binary nach ~/.local/bin/rubberduck.
#
# Nutzung:
#   ./install.sh [VERSION]
#   VERSION=v0.1.0 ./install.sh
#   curl -fsSL https://raw.githubusercontent.com/leuchtturm/rubberduck/main/install.sh | sh
#
# VERSION ist optional. Ohne Angabe wird das neueste Release ("latest")
# installiert. Erlaubt sind z. B. "v0.1.0" oder "0.1.0" (das "v" wird
# bei Bedarf ergänzt).
#
# Hinweis: Dieses Skript ist nur für Unix (Linux/macOS) gedacht.
# Windows-Nutzer verwenden bitte install.ps1.

# POSIX-sh-kompatibel halten: Dieses Skript wird per `curl ... | sh` (dash/busybox)
# ausgeführt – die Shebang wird dann ignoriert. `pipefail` gibt es dort nicht;
# `set -eu` genügt, da Downloads direkt (nicht in Pipes) laufen und `set -e`
# ihre Fehler erfasst.
set -eu

# ---------------------------------------------------------------------------
# !!! PLATZHALTER — bei einem Fork hier owner/repo anpassen !!!
# GitHub-Owner und -Repository. Wird für alle Download-URLs verwendet.
# ---------------------------------------------------------------------------
OWNER="leuchtturm"
REPO="rubberduck"

# Name der Binary im Archiv und auf der Platte.
BIN_NAME="rubberduck"

# Installationsverzeichnis (Unix-Contract): ~/.local/bin
INSTALL_DIR="${HOME}/.local/bin"

# Basis-URL des Repos.
BASE_URL="https://github.com/${OWNER}/${REPO}"

# ---------------------------------------------------------------------------
# Hilfsfunktionen
# ---------------------------------------------------------------------------

# err(): Meldung auf stderr ausgeben (kein Abbruch).
err() {
	printf 'error: %s\n' "$*" >&2
}

# die(): Fehlermeldung ausgeben und mit Exitcode 1 abbrechen.
die() {
	err "$*"
	exit 1
}

# info(): normale Statusmeldung auf stderr (damit stdout sauber bleibt).
info() {
	printf '%s\n' "$*" >&2
}

# have(): prüft, ob ein Kommando existiert.
have() {
	command -v "$1" >/dev/null 2>&1
}

# ---------------------------------------------------------------------------
# Zielplattform (TARGET / Rust-Target-Triple) aus uname ableiten
# ---------------------------------------------------------------------------
detect_target() {
	local os arch os_suffix arch_part
	os="$(uname -s)"
	arch="$(uname -m)"

	# Betriebssystem -> os-Suffix des Target-Triples.
	case "${os}" in
	Linux)
		os_suffix="-unknown-linux-gnu"
		;;
	Darwin)
		os_suffix="-apple-darwin"
		;;
	# Windows (MINGW/MSYS/Cygwin) wird hier bewusst NICHT unterstützt.
	MINGW* | MSYS* | CYGWIN* | Windows_NT)
		die "Windows wird von install.sh nicht unterstützt. Bitte install.ps1 verwenden."
		;;
	*)
		die "Nicht unterstütztes Betriebssystem: '${os}'. Unterstützt: Linux, macOS."
		;;
	esac

	# Architektur -> arch-Teil des Target-Triples.
	case "${arch}" in
	x86_64 | amd64)
		arch_part="x86_64"
		;;
	aarch64 | arm64)
		arch_part="aarch64"
		;;
	*)
		die "Nicht unterstützte Architektur: '${arch}'. Unterstützt: x86_64/amd64, aarch64/arm64."
		;;
	esac

	# Zusammensetzen als "<arch>-<os-suffix>", z. B. x86_64-unknown-linux-gnu.
	printf '%s%s' "${arch_part}" "${os_suffix}"
}

# ---------------------------------------------------------------------------
# Download-Helfer: nutzt curl oder wget, ausschließlich HTTPS.
# Verwendung: download <url> <zieldatei>
# ---------------------------------------------------------------------------
download() {
	local url="$1" dest="$2"

	# Nur HTTPS zulassen (Sicherheit).
	case "${url}" in
	https://*) ;;
	*)
		die "Unsichere oder ungültige URL (kein HTTPS): ${url}"
		;;
	esac

	if have curl; then
		# -f: bei HTTP-Fehlern fehlschlagen, -L: Redirects folgen, -S: Fehler zeigen.
		curl -fsSL --proto '=https' --tlsv1.2 -o "${dest}" "${url}"
	elif have wget; then
		wget --https-only -q -O "${dest}" "${url}"
	else
		die "Weder 'curl' noch 'wget' gefunden. Bitte eines davon installieren."
	fi
}

# ---------------------------------------------------------------------------
# SHA256 einer Datei berechnen (nur den Hex-Hash ausgeben).
# Nutzt sha256sum oder 'shasum -a 256'.
# ---------------------------------------------------------------------------
sha256_of() {
	local file="$1"
	if have sha256sum; then
		sha256sum "${file}" | awk '{ print $1 }'
	elif have shasum; then
		shasum -a 256 "${file}" | awk '{ print $1 }'
	else
		die "Weder 'sha256sum' noch 'shasum' gefunden. Kann Prüfsumme nicht verifizieren."
	fi
}

# ---------------------------------------------------------------------------
# PATH-Hinweis: warnen, falls INSTALL_DIR nicht im PATH ist.
# ---------------------------------------------------------------------------
warn_path() {
	local dir="$1"
	# PATH anhand des ':'-Trenners prüfen (mit umschließenden ':' für exakte Matches).
	case ":${PATH}:" in
	*":${dir}:"*)
		# Alles gut, Verzeichnis ist im PATH.
		return 0
		;;
	esac

	info ""
	info "Hinweis: '${dir}' ist nicht in deinem PATH."
	info "Füge es hinzu, damit 'rubberduck' direkt aufrufbar ist:"
	info ""
	info "  bash:  echo 'export PATH=\"${dir}:\$PATH\"' >> ~/.bashrc"
	info "  zsh:   echo 'export PATH=\"${dir}:\$PATH\"' >> ~/.zshrc"
	info "  fish:  fish_add_path \"${dir}\""
	info ""
	info "Danach eine neue Shell öffnen oder die Konfigurationsdatei neu laden."
}

# ---------------------------------------------------------------------------
# Hauptprogramm
# ---------------------------------------------------------------------------
main() {
	# Version bestimmen: 1. Argument hat Vorrang vor der VERSION-Umgebungsvariable,
	# Default ist "latest".
	local version
	version="${1:-${VERSION:-latest}}"

	# Falls eine konkrete Version ohne führendes 'v' angegeben wurde (z. B. "0.1.0"),
	# das 'v' ergänzen, damit es zu den Git-Tags (v0.1.0) passt.
	if [ "${version}" != "latest" ]; then
		case "${version}" in
		v*) : ;; # bereits mit 'v' versehen
		*) version="v${version}" ;;
		esac
	fi

	# Zielplattform ermitteln.
	local target
	target="$(detect_target)"

	# Asset-Namen gemäß Release-Naming-Contract.
	local archive_name checksums_name
	archive_name="${BIN_NAME}-${target}.tar.gz"
	checksums_name="SHA256SUMS"

	# Download-URLs zusammenbauen (latest vs. gepinnte Version).
	local base_dl archive_url checksums_url
	if [ "${version}" = "latest" ]; then
		base_dl="${BASE_URL}/releases/latest/download"
	else
		base_dl="${BASE_URL}/releases/download/${version}"
	fi
	archive_url="${base_dl}/${archive_name}"
	checksums_url="${base_dl}/${checksums_name}"

	# Vor dem Download: aufgelöste Version + Quelle ausgeben (Sicherheit/Transparenz).
	info "rubberduck installer"
	info "  Version: ${version}"
	info "  Target:  ${target}"
	info "  Quelle:  ${archive_url}"
	info ""

	# Sicheres temporäres Verzeichnis anlegen und beim Beenden aufräumen.
	local tmpdir
	tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t rubberduck)"
	# shellcheck disable=SC2064
	# Absichtlich jetzt expandieren, damit der Pfad im Trap fixiert ist.
	trap "rm -rf \"${tmpdir}\"" EXIT

	local archive_path checksums_path
	archive_path="${tmpdir}/${archive_name}"
	checksums_path="${tmpdir}/${checksums_name}"

	# Archiv und Prüfsummen-Datei herunterladen.
	info "Lade Archiv herunter ..."
	download "${archive_url}" "${archive_path}"
	info "Lade SHA256SUMS herunter ..."
	download "${checksums_url}" "${checksums_path}"

	# --- Prüfsumme VOR dem Entpacken verifizieren ---
	info "Verifiziere SHA256-Prüfsumme ..."

	# Erwartete Prüfsumme aus SHA256SUMS herausfiltern.
	# Format: "<64-hex>  <asset-filename>". Wir matchen auf den Dateinamen am
	# Zeilenende, um Teil-Treffer zu vermeiden.
	local expected_line expected_hash actual_hash
	expected_line="$(grep -E "[[:space:]]\*?${archive_name}\$" "${checksums_path}" || true)"
	if [ -z "${expected_line}" ]; then
		die "Kein SHA256-Eintrag für '${archive_name}' in SHA256SUMS gefunden."
	fi
	expected_hash="$(printf '%s\n' "${expected_line}" | awk '{ print $1 }')"

	# Tatsächliche Prüfsumme des heruntergeladenen Archivs berechnen.
	actual_hash="$(sha256_of "${archive_path}")"

	# Hashes klein schreiben und vergleichen.
	expected_hash="$(printf '%s' "${expected_hash}" | tr '[:upper:]' '[:lower:]')"
	actual_hash="$(printf '%s' "${actual_hash}" | tr '[:upper:]' '[:lower:]')"

	if [ "${expected_hash}" != "${actual_hash}" ]; then
		err "SHA256-PRÜFSUMME STIMMT NICHT ÜBEREIN — Installation abgebrochen!"
		err "  erwartet:  ${expected_hash}"
		err "  berechnet: ${actual_hash}"
		die "Mögliche Manipulation oder beschädigter Download."
	fi
	info "Prüfsumme OK."

	# --- Archiv entpacken ---
	info "Entpacke Archiv ..."
	tar -xzf "${archive_path}" -C "${tmpdir}"

	# Die entpackte Binary liegt laut Contract im Archiv-Root als "rubberduck".
	local extracted_bin
	extracted_bin="${tmpdir}/${BIN_NAME}"
	if [ ! -f "${extracted_bin}" ]; then
		die "Erwartete Binary '${BIN_NAME}' wurde im Archiv nicht gefunden."
	fi

	# --- Installieren ---
	# Zielverzeichnis anlegen, falls nicht vorhanden.
	mkdir -p "${INSTALL_DIR}"

	local dest_bin
	dest_bin="${INSTALL_DIR}/${BIN_NAME}"

	# Verschieben und ausführbar machen.
	mv -f "${extracted_bin}" "${dest_bin}"
	chmod 755 "${dest_bin}"

	info ""
	info "Erfolgreich installiert: ${dest_bin}"

	# Warnen, falls das Installationsverzeichnis nicht im PATH ist.
	warn_path "${INSTALL_DIR}"

	info ""
	info "Loslegen mit:  ${BIN_NAME}"
}

main "$@"
