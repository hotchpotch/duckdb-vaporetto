param(
    [Parameter(Mandatory = $true)][string]$DuckdbCli,
    [Parameter(Mandatory = $true)][string]$Extension
)

$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path ".tmp" | Out-Null
$tmpSql = Join-Path ".tmp" ("default-model.{0}.sql" -f ([System.IO.Path]::GetRandomFileName()))
$oldModel = $env:DUCKDB_VAPORETTO_MODEL
$oldTags = $env:DUCKDB_VAPORETTO_TAGS
try {
    $extensionForSql = $Extension -replace '\\', '/'
    (Get-Content "tests/default_model.sql" -Raw).Replace("EXT_PATH", $extensionForSql) | Set-Content -Path $tmpSql -Encoding utf8

    Remove-Item Env:\DUCKDB_VAPORETTO_MODEL -ErrorAction SilentlyContinue
    Remove-Item Env:\DUCKDB_VAPORETTO_TAGS -ErrorAction SilentlyContinue
    $output = & $DuckdbCli "-unsigned" ":memory:" ".read $tmpSql" 2>&1
    $exitCode = $LASTEXITCODE

    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        throw "duckdb exited with $exitCode"
    }

    $joined = ($output -join "`n")
    @(
        "東京/特許/許可/局",
        '"東京" AND "特許" AND "許可" AND "局"'
    ) | ForEach-Object {
        if ($joined -notmatch [regex]::Escape($_)) {
            throw "expected output was not found: $_"
        }
    }
}
finally {
    if ($null -ne $oldModel) { $env:DUCKDB_VAPORETTO_MODEL = $oldModel }
    if ($null -ne $oldTags) { $env:DUCKDB_VAPORETTO_TAGS = $oldTags }
    Remove-Item -Force $tmpSql -ErrorAction SilentlyContinue
}
