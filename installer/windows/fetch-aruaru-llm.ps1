# aruaru-llm(AI応答エンジン)本体を、open-englishインストーラーの
# 「まとめてインストール」タスクから取得するスクリプト(2026-08-11新設)。
#
# 正直な開示: これはaruaru-llmの実行ファイルのみを取得する。GPT-2/
# DistilGPT-2の実モデル重み(数百MB〜数GB、初回起動時に別途ダウンロード
# または `POST /v1/models/install` で取得)・aruaru-db・PostgreSQLは
# 含まない——それぞれ別リポジトリのセットアップ手順に従うこと
# (README-INSTALLED.txt参照)。
param(
    [Parameter(Mandatory = $true)]
    [string]$DestDir
)

$ErrorActionPreference = "Stop"

try {
    New-Item -ItemType Directory -Force -Path $DestDir | Out-Null

    $apiUrl = "https://api.github.com/repos/aon-co-jp/aruaru-llm/releases/latest"
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "open-english-installer" }
    $asset = $release.assets | Where-Object { $_.name -like "*windows*x86_64*.zip" } | Select-Object -First 1

    if (-not $asset) {
        Write-Output "aruaru-llm: no Windows release asset found. Please download it manually from https://github.com/aon-co-jp/aruaru-llm/releases"
        exit 0
    }

    $zipPath = Join-Path $DestDir "aruaru-llm.zip"
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath -UseBasicParsing
    Expand-Archive -Path $zipPath -DestinationPath $DestDir -Force
    Remove-Item $zipPath -Force

    Write-Output "aruaru-llm downloaded to $DestDir. Model weights are NOT included - run aruaru-llm.exe once and use POST /v1/models/install, or see its own README."
} catch {
    # ダウンロード失敗はインストーラー全体を止めない(可用性優先、
    # 既存のaruaru-llm自体の「サービスを止めない」設計方針と同じ)。
    Write-Output "aruaru-llm download failed: $_. You can install it manually later from https://github.com/aon-co-jp/aruaru-llm/releases"
    exit 0
}
