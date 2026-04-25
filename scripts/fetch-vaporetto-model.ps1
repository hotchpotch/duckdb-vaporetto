param(
    [string]$ModelName = $(if ($env:MODEL_NAME) { $env:MODEL_NAME } else { "bccwj-suw_c0.003" }),
    [string]$ModelVersion = $(if ($env:MODEL_VERSION) { $env:MODEL_VERSION } else { "v0.5.0" }),
    [string]$Root = $(if ($env:MODEL_TMP_DIR) { $env:MODEL_TMP_DIR } else { ".tmp\models" })
)

$ErrorActionPreference = "Stop"

$archive = Join-Path ".tmp" "$ModelName.tar.xz"
$modelDir = Join-Path $Root $ModelName
$modelFile = Join-Path $modelDir "$ModelName.model.zst"
$url = "https://github.com/daac-tools/vaporetto-models/releases/download/$ModelVersion/$ModelName.tar.xz"

New-Item -ItemType Directory -Force -Path ".tmp", $modelDir | Out-Null

if (-not (Test-Path $modelFile)) {
    Invoke-WebRequest -Uri $url -OutFile $archive
    tar -xJf $archive -C $modelDir --strip-components=1
}

if (-not (Test-Path $modelFile)) {
    throw "model file was not found: $modelFile"
}

$modelPath = (Resolve-Path $modelFile).Path
if ($env:GITHUB_OUTPUT) {
    "model_path=$modelPath" | Out-File -FilePath $env:GITHUB_OUTPUT -Append -Encoding utf8
}

Write-Host "model_path=$modelPath"

