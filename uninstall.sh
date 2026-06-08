#!/usr/bin/env bash
#
# uninstall.sh – removes rubberduck along with its configuration and logs.
#
# This script is the counterpart to the (curl|sh) installer and does the same
# as `rubberduck self uninstall` from inside the app. It removes:
#   - the binary  ~/.local/bin/rubberduck
#   - the config directory  ~/.config/rubberduck   (override: RUBBERDUCK_CONFIG_DIR)
#   - the data/log directory  ~/.rubberduck         (override: RUBBERDUCK_DATA_DIR)
#
# NOTE: `rubberduck self uninstall` performs the same cleanup directly from
# within the application.

# POSIX-sh compatible: run via `curl ... | sh` (dash/busybox).
set -eu

# --- Helper functions ------------------------------------------------------

# err(): write a message to stderr (does not abort).
err() {
	printf '%s\n' "$*" >&2
}

# die(): write a message to stderr and exit with an error code.
die() {
	err "Error: $*"
	exit 1
}

# --- Argument parsing ------------------------------------------------------

ASSUME_YES=0

usage() {
	cat <<'EOF'
Usage: uninstall.sh [options]

Removes rubberduck (binary, configuration and logs).

Options:
  -y, --yes      Do not ask, remove right away (for scripts/CI).
  -h, --help     Show this help.

Environment variables honoured:
  RUBBERDUCK_CONFIG_DIR   Overrides the config directory (~/.config/rubberduck).
  RUBBERDUCK_DATA_DIR     Overrides the data/log directory (~/.rubberduck).

Note: `rubberduck self uninstall` does the same from within the app.
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
		die "Unknown option: $1 (see --help)"
		;;
	*)
		die "Unexpected argument: $1 (see --help)"
		;;
	esac
	shift
done

# --- Resolve targets -------------------------------------------------------

# HOME must be set, otherwise the paths cannot be built.
[ -n "${HOME:-}" ] || die "HOME is not set – paths cannot be resolved."

# Default install location of the binary (frozen): ~/.local/bin/rubberduck
BIN_PATH="${HOME}/.local/bin/rubberduck"

# Config and data directory, each with an env override (like the binary itself).
CONFIG_DIR="${RUBBERDUCK_CONFIG_DIR:-${HOME}/.config/rubberduck}"
DATA_DIR="${RUBBERDUCK_DATA_DIR:-${HOME}/.rubberduck}"

# Check where `command -v rubberduck` points. If the detected binary lives
# elsewhere (e.g. cargo install ~/.cargo/bin or Homebrew), we warn, because this
# script only removes the default path ~/.local/bin/rubberduck.
RESOLVED_BIN=""
if RESOLVED_BIN="$(command -v rubberduck 2>/dev/null)"; then
	# Normalise paths (symlinks etc. don't matter much here; a direct string
	# comparison is enough for the warning).
	if [ "$RESOLVED_BIN" != "$BIN_PATH" ]; then
		err "Warning: 'command -v rubberduck' points to:"
		err "    $RESOLVED_BIN"
		err "  This script only removes the default path:"
		err "    $BIN_PATH"
		err "  The other binary (e.g. via 'cargo install' or Homebrew) must be"
		err "  removed separately, for example with:"
		err "    cargo uninstall rubberduck-cli   # for a cargo installation"
		err "    brew uninstall rubberduck        # for a Homebrew installation"
		err ""
	fi
fi

# --- Overview: what actually exists? ---------------------------------------

# Count existing targets (POSIX-compatible, without bash arrays).
FOUND=0
for p in "$BIN_PATH" "$CONFIG_DIR" "$DATA_DIR"; do
	if [ -e "$p" ] || [ -L "$p" ]; then
		FOUND=$((FOUND + 1))
	fi
done

echo "rubberduck uninstall"
echo "===================="
echo
echo "The following paths are checked and – if present – removed:"
printf '  Binary:        %s%s\n' "$BIN_PATH" "$([ -e "$BIN_PATH" ] && echo '' || echo '   (not present)')"
printf '  Config:        %s%s\n' "$CONFIG_DIR" "$([ -e "$CONFIG_DIR" ] && echo '' || echo '   (not present)')"
printf '  Logs/data:     %s%s\n' "$DATA_DIR" "$([ -e "$DATA_DIR" ] && echo '' || echo '   (not present)')"
echo

# Nothing present? Then we are done.
if [ "$FOUND" -eq 0 ]; then
	echo "Nothing found that would need removing. Done."
	exit 0
fi

# --- Ask for confirmation --------------------------------------------------

if [ "$ASSUME_YES" -ne 1 ]; then
	# Non-interactive guard: no TTY and no --yes => abort with instructions.
	if [ ! -t 0 ]; then
		die "No interactive input possible (stdin is not a terminal). Re-run with '--yes' or '-y' to remove without a prompt."
	fi

	printf 'Really remove these paths? [y/N] '
	read -r reply
	case "$reply" in
	[yY] | [yY][eE][sS])
		: # confirmed – continue
		;;
	*)
		echo "Cancelled – nothing was removed."
		exit 0
		;;
	esac
fi

# --- Remove ----------------------------------------------------------------

# remove_path(): removes a file or directory, tolerating absence.
remove_path() {
	path="$1"
	label="$2"
	if [ ! -e "$path" ] && [ ! -L "$path" ]; then
		# Already gone (or never there) – not an error.
		return 0
	fi
	if [ -d "$path" ] && [ ! -L "$path" ]; then
		rm -rf -- "$path" || die "Could not remove $label: $path"
	else
		rm -f -- "$path" || die "Could not remove $label: $path"
	fi
	echo "Removed: $label  ($path)"
}

remove_path "$BIN_PATH" "Binary"
remove_path "$CONFIG_DIR" "Config"
remove_path "$DATA_DIR" "Logs/data"

echo
echo "rubberduck has been removed. Thanks for the quacks!"
