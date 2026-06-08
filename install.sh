#!/usr/bin/env bash
#
# install.sh — installer for "rubberduck" (rubberduck-cli)
#
# Downloads the matching release archive from GitHub, verifies its SHA256
# checksum against the SHA256SUMS file BEFORE extracting, and installs the
# binary to ~/.local/bin/rubberduck.
#
# Usage:
#   ./install.sh [VERSION]
#   VERSION=v1.0.0 ./install.sh
#   curl -fsSL https://raw.githubusercontent.com/j-pfalzgraf/rubberduck/main/install.sh | sh
#
# VERSION is optional. Without it the newest release ("latest") is installed.
# Accepted forms are e.g. "v1.0.0" or "1.0.0" (the leading "v" is added when
# missing).
#
# Note: this script is for Unix (Linux/macOS) only. Windows users should use
# install.ps1.

# Stay POSIX-sh compatible: this script is run via `curl ... | sh` (dash/busybox)
# where the shebang is ignored. `pipefail` is unavailable there; `set -eu` is
# enough, since downloads run directly (not in pipes) so `set -e` catches their
# errors.
set -eu

# ---------------------------------------------------------------------------
# !!! PLACEHOLDER — change owner/repo here when you fork !!!
# GitHub owner and repository. Used for all download URLs.
# ---------------------------------------------------------------------------
OWNER="j-pfalzgraf"
REPO="rubberduck"

# Name of the binary inside the archive and on disk.
BIN_NAME="rubberduck"

# Install directory (Unix contract): ~/.local/bin
INSTALL_DIR="${HOME}/.local/bin"

# Base URL of the repo.
BASE_URL="https://github.com/${OWNER}/${REPO}"

# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------

# err(): print a message to stderr (does not abort).
err() {
	printf 'error: %s\n' "$*" >&2
}

# die(): print an error and abort with exit code 1.
die() {
	err "$*"
	exit 1
}

# info(): normal status message on stderr (so stdout stays clean).
info() {
	printf '%s\n' "$*" >&2
}

# have(): check whether a command exists.
have() {
	command -v "$1" >/dev/null 2>&1
}

# ---------------------------------------------------------------------------
# Derive the target platform (TARGET / Rust target triple) from uname
# ---------------------------------------------------------------------------
detect_target() {
	local os arch os_suffix arch_part
	os="$(uname -s)"
	arch="$(uname -m)"

	# Operating system -> os suffix of the target triple.
	case "${os}" in
	Linux)
		os_suffix="-unknown-linux-gnu"
		;;
	Darwin)
		os_suffix="-apple-darwin"
		;;
	# Windows (MINGW/MSYS/Cygwin) is deliberately NOT supported here.
	MINGW* | MSYS* | CYGWIN* | Windows_NT)
		die "Windows is not supported by install.sh. Please use install.ps1."
		;;
	*)
		die "Unsupported operating system: '${os}'. Supported: Linux, macOS."
		;;
	esac

	# Architecture -> arch part of the target triple.
	case "${arch}" in
	x86_64 | amd64)
		arch_part="x86_64"
		;;
	aarch64 | arm64)
		arch_part="aarch64"
		;;
	*)
		die "Unsupported architecture: '${arch}'. Supported: x86_64/amd64, aarch64/arm64."
		;;
	esac

	# Assemble as "<arch>-<os-suffix>", e.g. x86_64-unknown-linux-gnu.
	printf '%s%s' "${arch_part}" "${os_suffix}"
}

# ---------------------------------------------------------------------------
# Download helper: uses curl or wget, HTTPS only.
# Usage: download <url> <destination-file>
# ---------------------------------------------------------------------------
download() {
	local url="$1" dest="$2"

	# Allow HTTPS only (security).
	case "${url}" in
	https://*) ;;
	*)
		die "Insecure or invalid URL (not HTTPS): ${url}"
		;;
	esac

	if have curl; then
		# -f: fail on HTTP errors, -L: follow redirects, -S: show errors.
		curl -fsSL --proto '=https' --tlsv1.2 -o "${dest}" "${url}"
	elif have wget; then
		wget --https-only -q -O "${dest}" "${url}"
	else
		die "Neither 'curl' nor 'wget' found. Please install one of them."
	fi
}

# ---------------------------------------------------------------------------
# Compute the SHA256 of a file (print the hex hash only).
# Uses sha256sum or 'shasum -a 256'.
# ---------------------------------------------------------------------------
sha256_of() {
	local file="$1"
	if have sha256sum; then
		sha256sum "${file}" | awk '{ print $1 }'
	elif have shasum; then
		shasum -a 256 "${file}" | awk '{ print $1 }'
	else
		die "Neither 'sha256sum' nor 'shasum' found. Cannot verify the checksum."
	fi
}

# ---------------------------------------------------------------------------
# PATH hint: warn if INSTALL_DIR is not on PATH.
# ---------------------------------------------------------------------------
warn_path() {
	local dir="$1"
	# Check PATH using the ':' separator (with surrounding ':' for exact matches).
	case ":${PATH}:" in
	*":${dir}:"*)
		# All good, the directory is on PATH.
		return 0
		;;
	esac

	info ""
	info "Note: '${dir}' is not on your PATH."
	info "Add it so 'rubberduck' can be called directly:"
	info ""
	info "  bash:  echo 'export PATH=\"${dir}:\$PATH\"' >> ~/.bashrc"
	info "  zsh:   echo 'export PATH=\"${dir}:\$PATH\"' >> ~/.zshrc"
	info "  fish:  fish_add_path \"${dir}\""
	info ""
	info "Then open a new shell or reload the configuration file."
}

# ---------------------------------------------------------------------------
# Main program
# ---------------------------------------------------------------------------
main() {
	# Determine the version: the 1st argument takes precedence over the VERSION
	# environment variable; the default is "latest".
	local version
	version="${1:-${VERSION:-latest}}"

	# If a concrete version without a leading 'v' was given (e.g. "1.0.0"), add
	# the 'v' so it matches the git tags (v1.0.0).
	if [ "${version}" != "latest" ]; then
		case "${version}" in
		v*) : ;; # already prefixed with 'v'
		*) version="v${version}" ;;
		esac
	fi

	# Determine the target platform.
	local target
	target="$(detect_target)"

	# Asset names per the release naming contract.
	local archive_name checksums_name
	archive_name="${BIN_NAME}-${target}.tar.gz"
	checksums_name="SHA256SUMS"

	# Build the download URLs (latest vs. pinned version).
	local base_dl archive_url checksums_url
	if [ "${version}" = "latest" ]; then
		base_dl="${BASE_URL}/releases/latest/download"
	else
		base_dl="${BASE_URL}/releases/download/${version}"
	fi
	archive_url="${base_dl}/${archive_name}"
	checksums_url="${base_dl}/${checksums_name}"

	# Before downloading: print the resolved version + source (security/transparency).
	info "rubberduck installer"
	info "  Version: ${version}"
	info "  Target:  ${target}"
	info "  Source:  ${archive_url}"
	info ""

	# Create a safe temporary directory and clean it up on exit.
	local tmpdir
	tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t rubberduck)"
	# shellcheck disable=SC2064
	# Intentionally expand now so the path is fixed in the trap.
	trap "rm -rf \"${tmpdir}\"" EXIT

	local archive_path checksums_path
	archive_path="${tmpdir}/${archive_name}"
	checksums_path="${tmpdir}/${checksums_name}"

	# Download the archive and the checksums file.
	info "Downloading archive ..."
	download "${archive_url}" "${archive_path}"
	info "Downloading SHA256SUMS ..."
	download "${checksums_url}" "${checksums_path}"

	# --- Verify the checksum BEFORE extracting ---
	info "Verifying SHA256 checksum ..."

	# Filter the expected checksum out of SHA256SUMS.
	# Format: "<64-hex>  <asset-filename>". We match the file name at the end of
	# the line to avoid partial matches.
	local expected_line expected_hash actual_hash
	expected_line="$(grep -E "[[:space:]]\*?${archive_name}\$" "${checksums_path}" || true)"
	if [ -z "${expected_line}" ]; then
		die "No SHA256 entry for '${archive_name}' found in SHA256SUMS."
	fi
	expected_hash="$(printf '%s\n' "${expected_line}" | awk '{ print $1 }')"

	# Compute the actual checksum of the downloaded archive.
	actual_hash="$(sha256_of "${archive_path}")"

	# Lowercase both hashes and compare.
	expected_hash="$(printf '%s' "${expected_hash}" | tr '[:upper:]' '[:lower:]')"
	actual_hash="$(printf '%s' "${actual_hash}" | tr '[:upper:]' '[:lower:]')"

	if [ "${expected_hash}" != "${actual_hash}" ]; then
		err "SHA256 CHECKSUM MISMATCH — installation aborted!"
		err "  expected:   ${expected_hash}"
		err "  calculated: ${actual_hash}"
		die "Possible tampering or a corrupted download."
	fi
	info "Checksum OK."

	# --- Extract the archive ---
	info "Extracting archive ..."
	tar -xzf "${archive_path}" -C "${tmpdir}"

	# Per the contract the extracted binary sits at the archive root as "rubberduck".
	local extracted_bin
	extracted_bin="${tmpdir}/${BIN_NAME}"
	if [ ! -f "${extracted_bin}" ]; then
		die "Expected binary '${BIN_NAME}' was not found in the archive."
	fi

	# --- Install ---
	# Create the target directory if it does not exist.
	mkdir -p "${INSTALL_DIR}"

	local dest_bin
	dest_bin="${INSTALL_DIR}/${BIN_NAME}"

	# Move it into place and make it executable.
	mv -f "${extracted_bin}" "${dest_bin}"
	chmod 755 "${dest_bin}"

	info ""
	info "Successfully installed: ${dest_bin}"

	# Warn if the install directory is not on PATH.
	warn_path "${INSTALL_DIR}"

	info ""
	info "Get started with:  ${BIN_NAME}"
}

main "$@"
