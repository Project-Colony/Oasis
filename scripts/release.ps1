param(
  [string]$Target
)

$ErrorActionPreference = 'Stop'

$packageName = 'oasis-weather-notify'
$versionLine = Select-String -Path Cargo.toml -Pattern '^version\s*=' | Select-Object -First 1
$version = $versionLine.Line.Split('"')[1]

if ($Target) {
  cargo build --release --target $Target
  $targetDir = "target/$Target/release"
  $archiveTarget = $Target
} else {
  cargo build --release
  $targetDir = "target/release"
  $archiveTarget = (rustc -vV | Select-String -Pattern '^host:').Line.Split(' ')[1]
}

$binaryPath = Join-Path $targetDir "$packageName.exe"
if (-not (Test-Path $binaryPath)) {
  throw "Binary not found at $binaryPath"
}

$distDir = 'dist'
New-Item -ItemType Directory -Force -Path $distDir | Out-Null
$archiveName = "$packageName-$version-$archiveTarget.zip"
$archivePath = Join-Path $distDir $archiveName

if (Test-Path $archivePath) {
  Remove-Item $archivePath
}

Compress-Archive -Path $binaryPath -DestinationPath $archivePath
Write-Output "Created $archivePath"
