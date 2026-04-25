param(
    [Parameter(Mandatory = $true)][string]$DuckdbCli,
    [Parameter(Mandatory = $true)][string]$Extension,
    [Parameter(Mandatory = $true)][string]$Model
)

$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path ".tmp" | Out-Null
$tmpSql = Join-Path ".tmp" ("scalar.{0}.sql" -f ([System.IO.Path]::GetRandomFileName()))
try {
    $extensionForSql = $Extension -replace '\\', '/'
    (Get-Content "tests/scalar.sql" -Raw).Replace("EXT_PATH", $extensionForSql) | Set-Content -Path $tmpSql -Encoding utf8
    $tmpSqlForDuckdb = (Resolve-Path $tmpSql).Path -replace '\\', '/'

    $env:DUCKDB_VAPORETTO_MODEL = $Model
    $output = & $DuckdbCli "-unsigned" ":memory:" ".read $tmpSqlForDuckdb" 2>&1
    $exitCode = $LASTEXITCODE

    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        throw "duckdb exited with $exitCode"
    }

    $joined = ($output -join "`n")
    @(
        "東京/特許/許可/局",
        '"東京" AND "特許" AND "許可" AND "局"',
        "hello/hello",
        "Hello/HELLO",
        "東京/検索/エンジン/実験",
        '"東京" AND "検索" AND "エンジン" AND "実験"'
    ) | ForEach-Object {
        if ($joined -notmatch [regex]::Escape($_)) {
            throw "expected output was not found: $_"
        }
    }
}
finally {
    Remove-Item -Force $tmpSql -ErrorAction SilentlyContinue
}
