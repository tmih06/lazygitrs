# lazygitrs installer for Windows — downloads the prebuilt binary from this
# repo's GitHub releases and installs it to %LOCALAPPDATA%\Programs\lazygitrs.
#
#   irm https://github.com/tmih06/lazygitrs/releases/latest/download/lazygitrs-installer.ps1 | iex
#
# Optional version (default: latest):
#   & ([scriptblock]::Create((irm ".../lazygitrs-installer.ps1"))) v0.0.38
[CmdletBinding()]
param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

$Repo = "tmih06/lazygitrs"
$BinaryName = "lazygitrs"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\lazygitrs" }

# --- platform detection -----------------------------------------------------
$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64" }
    "ARM64" { "aarch64" }
    default { throw "unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}

$asset = "$BinaryName-windows-$arch.exe"

if ($env:LAZYGITRS_BASE_URL) {
    # Test/CI override: point the installer at a local mirror of the release
    # assets instead of GitHub.
    $base = $env:LAZYGITRS_BASE_URL
} elseif ($Version -eq "latest") {
    $base = "https://github.com/$Repo/releases/latest/download"
} else {
    $tag = if ($Version.StartsWith("v")) { $Version } else { "v$Version" }
    $base = "https://github.com/$Repo/releases/download/$tag"
}

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Write-Host "-> Downloading $asset ($Version)..."
    Invoke-WebRequest -Uri "$base/$asset" -OutFile "$tmp\$BinaryName.exe" -UseBasicParsing
    try {
        Invoke-WebRequest -Uri "$base/checksums.txt" -OutFile "$tmp\checksums.txt" -UseBasicParsing
    } catch {
        # checksums are best-effort
    }

    # --- verify checksum ----------------------------------------------------
    if (Test-Path "$tmp\checksums.txt") {
        $line = Get-Content "$tmp\checksums.txt" | Where-Object { $_ -match " $asset$" } | Select-Object -First 1
        if ($line) {
            $expected = ($line -split "\s+")[0]
            $actual = (Get-FileHash "$tmp\$BinaryName.exe" -Algorithm SHA256).Hash.ToLower()
            if ($actual -ne $expected) {
                throw "checksum mismatch for $asset`n  expected: $expected`n  actual:   $actual"
            }
            Write-Host "-> Checksum verified."
        }
    }

    # --- install ------------------------------------------------------------
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Move-Item -Force "$tmp\$BinaryName.exe" "$InstallDir\$BinaryName.exe"

    Write-Host "[OK] lazygitrs installed to $InstallDir\$BinaryName.exe"

    # --- PATH ---------------------------------------------------------------
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (($userPath -split ";") -notcontains $InstallDir) {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
        Write-Host "-> Added $InstallDir to your user PATH (restart your terminal)."
    }

    Write-Host ""
    Write-Host "Run: lazygitrs"
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
