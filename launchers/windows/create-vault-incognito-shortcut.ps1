param(
    [string]$RepoRoot = $null,
    [string]$VaultUrl = $null
)

# Creates a desktop shortcut (.lnk) that launches Google Chrome in
# Incognito mode, pointed at vault.html (2026-08-27 new, per user
# request "double-clicking an icon should force Chrome to open in
# incognito mode").
#
# HONEST SCOPE: a web page cannot force the browser or its mode from
# JavaScript (browsers deliberately block this for privacy reasons --
# see vault.html's own on-page disclosure). What *is* possible, and is
# exactly what this script does, is create an OS-level shortcut whose
# target is chrome.exe with the --incognito flag and the vault URL as
# an argument -- this is a normal, documented Chrome command-line
# feature, not a workaround or exploit. Double-clicking the resulting
# icon always opens a fresh Incognito window at that URL. If Chrome is
# not installed, this script says so honestly and does not silently
# fall back to a different browser's private mode (behavior differs
# enough between browsers that a silent substitution would be
# misleading).

if (-not $RepoRoot) {
    $scriptDir = if ($PSScriptRoot) { $PSScriptRoot } elseif ($MyInvocation.MyCommand.Path) { Split-Path -Path $MyInvocation.MyCommand.Path -Parent } else { $null }
    if ($scriptDir) {
        $RepoRoot = Split-Path -Path (Split-Path -Path $scriptDir -Parent) -Parent
    } else {
        Write-Error "Could not detect the script location. Pass -RepoRoot explicitly."
        exit 1
    }
}
$repoRoot = $RepoRoot

if (-not $VaultUrl) {
    # Default: assume the local server on its default port. The
    # parentOrigin query param must match whatever origin index.html is
    # actually served from -- adjust -VaultUrl if that differs.
    $VaultUrl = "http://127.0.0.1:4601/vault.html?parentOrigin=http%3A%2F%2F127.0.0.1%3A4601"
}

# Chrome's install location varies (per-machine vs per-user install).
# Check the usual places rather than assuming one.
$candidateChromePaths = @(
    (Join-Path ${env:ProgramFiles} "Google\Chrome\Application\chrome.exe"),
    (Join-Path ${env:ProgramFiles(x86)} "Google\Chrome\Application\chrome.exe"),
    (Join-Path ${env:LocalAppData} "Google\Chrome\Application\chrome.exe")
)
$chromeExe = $candidateChromePaths | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

$desktop = [Environment]::GetFolderPath("Desktop")
$shell = New-Object -ComObject WScript.Shell

if ($chromeExe) {
    $shortcutPath = Join-Path $desktop "open-english-vault-incognito.lnk"
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $chromeExe
    # --incognito must be a separate argument, not appended inside the
    # quoted exe path -- see the Chrome shortcut guides this script's
    # approach was verified against (2026-08-27 web research).
    $shortcut.Arguments = "--incognito `"$VaultUrl`""
    $shortcut.WorkingDirectory = (Split-Path -Path $chromeExe -Parent)
    $shortcut.Description = "open-english vault (Chrome Incognito, forced via shortcut) / open-english vault(Chromeシークレットモード、ショートカット経由で強制)"
    $shortcut.Save()
    Write-Host "Created: $shortcutPath"
    Write-Host "  -> Double-clicking it always opens a fresh Chrome Incognito window at:"
    Write-Host "     $VaultUrl"
} else {
    Write-Warning "Google Chrome was not found in the usual install locations."
    Write-Warning "Download it first, then re-run this script: https://www.google.com/chrome/ / Chromeが見つかりませんでした。先にダウンロードしてください: https://www.google.com/chrome/"
    Write-Warning "No shortcut was created (this script does not silently substitute another browser's private mode)."
}
