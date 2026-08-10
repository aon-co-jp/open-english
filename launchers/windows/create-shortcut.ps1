param(
    [string]$RepoRoot = $null
)

# Creates a desktop shortcut (.lnk) that opens open-english's index.html
# in the default browser. open-english is a server-free static HTML/JS
# app, so the shortcut just points at the file directly.

if (-not $RepoRoot) {
    $scriptDir = if ($PSScriptRoot) { $PSScriptRoot } elseif ($MyInvocation.MyCommand.Path) { Split-Path -Path $MyInvocation.MyCommand.Path -Parent } else { $null }
    if ($scriptDir) {
        $RepoRoot = Split-Path -Path (Split-Path -Path $scriptDir -Parent) -Parent
    } else {
        Write-Error "Could not detect the script location. Pass -RepoRoot explicitly, e.g. .\create-shortcut.ps1 -RepoRoot 'F:\runo\open-english'"
        exit 1
    }
}
$repoRoot = $RepoRoot
$indexHtml = Join-Path $repoRoot "index.html"
$iconPath = Join-Path $repoRoot "icons\open-english.ico"

if (-not (Test-Path $indexHtml)) {
    Write-Error "index.html not found at: $indexHtml"
    exit 1
}

$desktop = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktop "open-english.lnk"

$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $indexHtml
$shortcut.WorkingDirectory = $repoRoot
$shortcut.IconLocation = "$iconPath,0"
$shortcut.Description = "open-english: Maid Cafe English Trainer"
$shortcut.Save()

Write-Host "Created: $shortcutPath"
