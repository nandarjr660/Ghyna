# Cuts a signed Ghyna release that installed copies pick up automatically.
#   .\scripts\release.ps1 1.1.2 "Apa yang baru"
# Bumps the version, builds the signed installer, writes latest.json, then
# commits, tags, pushes, and publishes the GitHub release.
# The updater signing key lives at %USERPROFILE%\.tauri\ghyna.key (never commit it).
param(
    [Parameter(Mandatory)][string]$Version,
    [string]$Notes = ""
)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot
Set-Location $root

if (git status --porcelain) { throw "Commit or stash your changes first." }
$branch = (git branch --show-current).Trim()
if ($branch -ne "main") { throw "Run this from the main branch." }
$origin = (git remote get-url origin).Trim()
if ($origin -notmatch "nandarjr660/Ghyna") { throw "origin must point to nandarjr660/Ghyna (found: $origin)" }
$keyPath = "$env:USERPROFILE\.tauri\ghyna.key"
if (-not (Test-Path $keyPath)) { throw "Missing updater signing key: $keyPath" }

# 1. Version, in every manifest and lockfile.
$replacements = @(
    @{ File = "package.json";              Pattern = '("version":\s*")[^"]+(")'; Count = 1 },
    @{ File = "package-lock.json";         Pattern = '("name": "ghyna",\s*"version": ")[^"]+(")'; Count = 2 },
    @{ File = "src-tauri/tauri.conf.json"; Pattern = '("version":\s*")[^"]+(")'; Count = 1 },
    @{ File = "src-tauri/Cargo.toml";      Pattern = '(?m)^(version = ")[^"]+(")'; Count = 1 },
    @{ File = "src-tauri/Cargo.lock";      Pattern = '(?m)^(name = "ghyna"\r?\nversion = ")[^"]+(")'; Count = 1 }
)
foreach ($r in $replacements) {
    $path = Join-Path $root $r.File
    $text = [IO.File]::ReadAllText($path)
    $regex = [regex]::new($r.Pattern)
    $new = $regex.Replace($text, "`${1}$Version`${2}", $r.Count)
    if ($new -eq $text) { throw "Version not found in $($r.File)" }
    [IO.File]::WriteAllText($path, $new)
}

# 2. Build and sign. The watchdog could restart a copy mid-build, so pause it.
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $keyPath -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
$watchdog = Get-ScheduledTask "Ghyna Watchdog" -ErrorAction SilentlyContinue
if ($watchdog) { Disable-ScheduledTask "Ghyna Watchdog" | Out-Null }
Get-Process ghyna -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "$root\src-tauri\target\release\*" } |
    Stop-Process -Force
try {
    npm run tauri build -- --no-bundle
    if ($LASTEXITCODE) { throw "Build failed." }
    # Antivirus can hold the fresh exe for a moment while NSIS packs it.
    foreach ($try in 1..5) {
        Start-Sleep -Seconds 5
        npx tauri bundle --bundles nsis
        if (-not $LASTEXITCODE) { break }
        if ($try -eq 5) { throw "Packing the installer failed." }
    }
} finally {
    if ($watchdog) { Enable-ScheduledTask "Ghyna Watchdog" | Out-Null }
}

# 3. latest.json: what installed copies check.
if (-not $Notes) { $Notes = "Ghyna $Version" }
$name = "Ghyna_${Version}_x64-setup.exe"
$setup = "$root\src-tauri\target\release\bundle\nsis\$name"
$sig = "$setup.sig"
$latest = "$root\src-tauri\target\release\bundle\nsis\latest.json"
if (-not (Test-Path $setup)) { throw "Installer not found: $setup" }
if (-not (Test-Path $sig)) { throw "Signature not found: $sig" }
$json = @{
    version   = $Version
    notes     = $Notes
    pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = (Get-Content $sig -Raw).Trim()
            url       = "https://github.com/nandarjr660/Ghyna/releases/download/v$Version/$name"
        }
    }
} | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText($latest, $json)  # no BOM

# 4. Publish.
git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "Release $Version"
git tag "v$Version"
git push origin HEAD "v$Version"
gh release create "v$Version" $setup $sig $latest --repo nandarjr660/Ghyna --title "Ghyna $Version" --notes $Notes
Write-Host "Released Ghyna v$Version. Installed copies update at start, hourly, or after waking from sleep."
