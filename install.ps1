<#
    p2ptokens installer for Windows (PowerShell)

    Usage:
      irm https://raw.githubusercontent.com/p2ptokens/p2ptokens/main/install.ps1 | iex
      .\install.ps1

    Environment variables:
      P2PTOKENS_REPO   GitHub repo slug to download releases from.
                       Defaults to "p2ptokens/p2ptokens". Override this if the
                       real repository lives at a different owner/name, e.g.:
                         $env:P2PTOKENS_REPO = "myorg/p2ptokens"; .\install.ps1
      P2PTOKENS_BIN    Directory to install the binaries into.
                       Defaults to "$env:LOCALAPPDATA\p2ptokens\bin".

    Installs two binaries: p2ptokens.exe (the client) and p2p-coordinator.exe.
#>

$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
$Repo = if ($env:P2PTOKENS_REPO) { $env:P2PTOKENS_REPO } else { 'p2ptokens/p2ptokens' }
$InstallDir = if ($env:P2PTOKENS_BIN) { $env:P2PTOKENS_BIN } else { Join-Path $env:LOCALAPPDATA 'p2ptokens\bin' }

# ---------------------------------------------------------------------------
# Detect architecture (only x64 is shipped for Windows)
# ---------------------------------------------------------------------------
$archRaw = $env:PROCESSOR_ARCHITECTURE
switch ($archRaw) {
    'AMD64' { $Arch = 'x64' }
    'x86'   { throw "Unsupported architecture: 32-bit x86. Please build from source." }
    'ARM64' { throw "No prebuilt Windows arm64 binary is available. Please build from source." }
    default { throw "Unsupported architecture: '$archRaw'. Please build from source." }
}

$Asset = "p2ptokens-windows-$Arch.zip"
$Url = "https://github.com/$Repo/releases/latest/download/$Asset"

Write-Host "Installing p2ptokens"
Write-Host "  repo:  $Repo"
Write-Host "  arch:  $Arch"
Write-Host "  asset: $Asset"
Write-Host ""

# ---------------------------------------------------------------------------
# Temp dir with cleanup
# ---------------------------------------------------------------------------
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("p2ptokens-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

try {
    $ArchivePath = Join-Path $TmpDir $Asset

    # -----------------------------------------------------------------------
    # Download
    # -----------------------------------------------------------------------
    Write-Host "Downloading $Url ..."
    try {
        Invoke-WebRequest -Uri $Url -OutFile $ArchivePath -UseBasicParsing
    }
    catch {
        throw "Download failed. Check that a release exists at https://github.com/$Repo/releases/latest`n$($_.Exception.Message)"
    }

    # -----------------------------------------------------------------------
    # Extract
    # -----------------------------------------------------------------------
    Write-Host "Extracting ..."
    $ExtractDir = Join-Path $TmpDir 'extract'
    Expand-Archive -Path $ArchivePath -DestinationPath $ExtractDir -Force

    # -----------------------------------------------------------------------
    # Install
    # -----------------------------------------------------------------------
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    foreach ($bin in @('p2ptokens.exe', 'p2p-coordinator.exe')) {
        $src = Join-Path $ExtractDir $bin
        if (-not (Test-Path $src)) {
            throw "Expected binary '$bin' not found in archive."
        }
        Copy-Item -Path $src -Destination (Join-Path $InstallDir $bin) -Force
        Write-Host "Installed $bin -> $(Join-Path $InstallDir $bin)"
    }

    Write-Host ""
    Write-Host "Success! p2ptokens and p2p-coordinator were installed to $InstallDir"

    # -----------------------------------------------------------------------
    # PATH check
    # -----------------------------------------------------------------------
    $pathEntries = ($env:PATH -split ';') | ForEach-Object { $_.TrimEnd('\') }
    if ($pathEntries -notcontains $InstallDir.TrimEnd('\')) {
        Write-Host ""
        Write-Host "WARNING: $InstallDir is not on your PATH."
        Write-Host "Add it for your user account by running:"
        Write-Host ""
        Write-Host "    [Environment]::SetEnvironmentVariable('Path', `"`$([Environment]::GetEnvironmentVariable('Path','User'));$InstallDir`", 'User')"
        Write-Host ""
        Write-Host "Then open a new terminal for the change to take effect."
    }
    else {
        Write-Host "You can now run: p2ptokens"
    }
}
finally {
    # Clean up temp dir
    if (Test-Path $TmpDir) {
        Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
