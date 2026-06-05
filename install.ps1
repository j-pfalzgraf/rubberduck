<#
.SYNOPSIS
    Installer for the "rubberduck" CLI (package: rubberduck-cli) on Windows.

.DESCRIPTION
    Downloads the appropriate release archive for this machine's architecture,
    verifies its SHA-256 checksum against the published SHA256SUMS file BEFORE
    extracting, installs the "rubberduck.exe" binary into
        %LOCALAPPDATA%\Programs\rubberduck
    and (if needed) adds that directory to the *user* PATH.

    Designed to be run directly via:
        irm https://raw.githubusercontent.com/leuchtturm/rubberduck/main/install.ps1 | iex
    (no mandatory parameters).

    Version selection (in order of precedence):
        1. -Version parameter (e.g. -Version v0.1.0)
        2. $env:VERSION environment variable
        3. default: latest

    NOTE (PLACEHOLDER): The GitHub owner/repo below ("leuchtturm/rubberduck")
    is a placeholder. If you forked or renamed the project, change $Owner and
    $Repo (or the derived $BaseUrl) to point at your repository.

.LICENSE
    MIT
#>

[CmdletBinding()]
param(
    # Release version/tag to install, e.g. "v0.1.0" or "0.1.0". Defaults to latest.
    [string]$Version
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# --- Configuration (PLACEHOLDER owner/repo: change these if you forked) --------
$Owner   = 'leuchtturm'
$Repo    = 'rubberduck'
$BinName = 'rubberduck'                 # produces rubberduck.exe on Windows
$BaseUrl = "https://github.com/$Owner/$Repo"

# Frozen install location for Windows binaries.
$InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\rubberduck'

# -----------------------------------------------------------------------------

function Write-Info  { param([string]$Message) Write-Host "==> $Message" }
function Write-Warn  { param([string]$Message) Write-Warning $Message }

# Resolve the requested version: -Version param, then $env:VERSION, then latest.
if ([string]::IsNullOrWhiteSpace($Version)) {
    if (-not [string]::IsNullOrWhiteSpace($env:VERSION)) {
        $Version = $env:VERSION
    }
}

# Detect architecture and map to the frozen Rust target triple.
function Get-Target {
    $arch = $null
    try {
        $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    } catch {
        # Fall back to the environment variable on very old hosts.
        $arch = $env:PROCESSOR_ARCHITECTURE
    }

    switch -Regex ($arch) {
        '^(X64|AMD64)$'   { return 'x86_64-pc-windows-msvc' }
        '^(Arm64|ARM64)$' { return 'aarch64-pc-windows-msvc' }
        default {
            throw "Unsupported Windows architecture: '$arch'. " +
                  "Only x86_64 (X64) and aarch64 (Arm64) are supported."
        }
    }
}

$target = Get-Target

# Asset names follow the frozen Release Asset Naming Contract.
$assetName = "$BinName-$target.zip"
$sumsName  = 'SHA256SUMS'

if ([string]::IsNullOrWhiteSpace($Version)) {
    # latest channel
    $resolvedVersion = 'latest'
    $downloadBase = "$BaseUrl/releases/latest/download"
} else {
    # pinned tag: accept "0.1.0" or "v0.1.0" and normalize to a "vX.Y.Z" tag.
    $tag = $Version
    if ($tag -notmatch '^v') {
        $tag = "v$tag"
    }
    $resolvedVersion = $tag
    $downloadBase = "$BaseUrl/releases/download/$tag"
}

$assetUrl = "$downloadBase/$assetName"
$sumsUrl  = "$downloadBase/$sumsName"

# Security requirement: print resolved version + source URL BEFORE doing anything.
Write-Info "rubberduck installer"
Write-Info "Resolved version : $resolvedVersion"
Write-Info "Target           : $target"
Write-Info "Archive URL      : $assetUrl"
Write-Info "Checksums URL    : $sumsUrl"
Write-Info "Install dir      : $InstallDir"
Write-Host ""

# Work in a secure, unique temp directory; always clean it up.
$tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("rubberduck-install-" + [System.Guid]::NewGuid().ToString('N'))

try {
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    $archivePath = Join-Path $tmpDir $assetName
    $sumsPath    = Join-Path $tmpDir $sumsName

    # Windows PowerShell 5.1 negotiates TLS 1.0/1.1 by default, but GitHub/Fastly
    # require TLS 1.2+. Enable TLS 1.2 (and 1.3 where the enum exists) so the
    # downloads succeed on stock Windows. Harmless on PowerShell 7+.
    try {
        [Net.ServicePointManager]::SecurityProtocol =
            [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    } catch { }
    try {
        [Net.ServicePointManager]::SecurityProtocol =
            [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls13
    } catch { }

    # Download over HTTPS only.
    foreach ($u in @($assetUrl, $sumsUrl)) {
        if ($u -notmatch '^https://') {
            throw "Refusing to download over a non-HTTPS URL: $u"
        }
    }

    Write-Info "Downloading archive..."
    Invoke-WebRequest -Uri $assetUrl -OutFile $archivePath -UseBasicParsing

    Write-Info "Downloading checksums..."
    Invoke-WebRequest -Uri $sumsUrl -OutFile $sumsPath -UseBasicParsing

    # Compute the SHA-256 of the downloaded archive.
    $actualHash = (Get-FileHash -Algorithm SHA256 -Path $archivePath).Hash

    # Parse SHA256SUMS for this asset's expected hash.
    # Format: "<64-hex>  <asset-filename>"  (two spaces, sha256sum style).
    $expectedHash = $null
    foreach ($line in (Get-Content -Path $sumsPath)) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        # Split on whitespace; first token is the hash, last token is the filename.
        $parts = $line -split '\s+', 2
        if ($parts.Count -lt 2) { continue }
        $hash = $parts[0].Trim()
        # The filename portion may start with a '*' (binary marker) in some tools.
        $file = $parts[1].Trim().TrimStart('*')
        if ($file -eq $assetName) {
            $expectedHash = $hash
            break
        }
    }

    if ([string]::IsNullOrWhiteSpace($expectedHash)) {
        throw "Could not find an entry for '$assetName' in $sumsName. Aborting."
    }

    Write-Info "Verifying checksum..."
    Write-Info "  expected: $expectedHash"
    Write-Info "  actual  : $actualHash"

    # Compare case-insensitively; abort on mismatch BEFORE extracting.
    if (-not ($actualHash -ieq $expectedHash)) {
        throw "Checksum mismatch for '$assetName'. Refusing to install. Aborting."
    }
    Write-Info "Checksum OK."

    # Extract the verified archive.
    $extractDir = Join-Path $tmpDir 'extract'
    New-Item -ItemType Directory -Path $extractDir -Force | Out-Null
    Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force

    $exeName = "$BinName.exe"
    $extractedExe = Join-Path $extractDir $exeName
    if (-not (Test-Path -Path $extractedExe)) {
        throw "Archive did not contain '$exeName' at its root. Aborting."
    }

    # Ensure the install directory exists.
    if (-not (Test-Path -Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    $destExe = Join-Path $InstallDir $exeName
    Write-Info "Installing to $destExe"
    Copy-Item -Path $extractedExe -Destination $destExe -Force

    # Ensure the install dir is on the *user* PATH.
    # Note: reading/writing the user PATH via the .NET API can, in the rare case
    # where the existing user PATH contains literal %VAR% references, flatten the
    # value from REG_EXPAND_SZ to REG_SZ. Acceptable for the common case.
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($null -eq $userPath) { $userPath = '' }

    $onPath = $false
    foreach ($entry in ($userPath -split ';')) {
        if ([string]::IsNullOrWhiteSpace($entry)) { continue }
        if ($entry.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')) {
            $onPath = $true
            break
        }
    }

    if ($onPath) {
        Write-Info "Install dir is already on your user PATH."
    } else {
        Write-Warn "Install dir is not on your PATH. Adding it to your *user* PATH."
        Write-Info "Changing user PATH: appending '$InstallDir'"

        $newUserPath =
            if ([string]::IsNullOrWhiteSpace($userPath)) { $InstallDir }
            else { ($userPath.TrimEnd(';') + ';' + $InstallDir) }

        [Environment]::SetEnvironmentVariable('Path', $newUserPath, 'User')

        # Update the current session too, so the user can run it immediately here.
        $env:Path = ($env:Path.TrimEnd(';') + ';' + $InstallDir)

        Write-Warn "Your user PATH was modified. Restart your shell (or sign out/in) for it to take effect in new shells."
        Write-Info "If you prefer not to use PATH, run the binary directly: `"$destExe`""
    }

    Write-Host ""
    Write-Info "Successfully installed rubberduck ($resolvedVersion) to:"
    Write-Host "    $destExe"
    Write-Host ""
    Write-Info "Run it with:"
    Write-Host "    rubberduck --help"
    Write-Host ""
    Write-Info "Config dir : `$HOME\.config\rubberduck   (override with `$env:RUBBERDUCK_CONFIG_DIR)"
    Write-Info "Data dir   : `$HOME\.rubberduck          (override with `$env:RUBBERDUCK_DATA_DIR)"
    Write-Info "Update later with: rubberduck self update"
}
finally {
    if (Test-Path -Path $tmpDir) {
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
