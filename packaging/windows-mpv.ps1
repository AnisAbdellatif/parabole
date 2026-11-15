# Downloads the latest shinchiro mpv-dev build, makes an MSVC import lib (mpv.lib) and exposes it to cargo.
# Needs a VS developer environment (lib.exe) and 7z on PATH. Writes LIB/PATH to $GITHUB_ENV when running in CI.
# Sets $env:MPV_DEV_DIR in the caller session; the DLL is libmpv-2.dll in that directory.
$ErrorActionPreference = 'Stop'
$dir = Join-Path (Get-Location) 'mpv-dev'
New-Item -ItemType Directory -Force $dir | Out-Null

$headers = @{ 'User-Agent' = 'parabole-ci' }
if ($env:GITHUB_TOKEN) { $headers['Authorization'] = "Bearer $env:GITHUB_TOKEN" }
$rel = Invoke-RestMethod -Headers $headers 'https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest'
$asset = $rel.assets | Where-Object { $_.name -match '^mpv-dev-x86_64-\d{8}' } | Select-Object -First 1
if (-not $asset) { throw 'no mpv-dev-x86_64 asset in latest shinchiro release' }

$archive = Join-Path $dir $asset.name
Invoke-WebRequest -Headers $headers -Uri $asset.browser_download_url -OutFile $archive
7z x -y "-o$dir" $archive | Out-Null

if (-not (Test-Path "$dir\libmpv-2.dll")) { throw 'libmpv-2.dll missing from archive' }
# shinchiro ships a MinGW libmpv.dll.a (and no longer mpv.def); MSVC needs mpv.lib, built from a .def of the DLL's exports.
$names = dumpbin /nologo /exports "$dir\libmpv-2.dll" |
  ForEach-Object { if ($_ -match '^\s+\d+\s+[0-9A-F]+\s+[0-9A-F]{8}\s+(mpv_\w+)') { $Matches[1] } }
if (-not $names) { throw 'no mpv_* exports found in libmpv-2.dll' }
@('EXPORTS') + $names | Set-Content -Encoding ascii "$dir\mpv.def"
lib /nologo /def:"$dir\mpv.def" /name:libmpv-2.dll /out:"$dir\mpv.lib" /machine:x64
if ($LASTEXITCODE -ne 0) { throw 'lib.exe failed to build mpv.lib' }

$env:LIB = "$dir;$env:LIB"
$env:PATH = "$dir;$env:PATH"
$env:MPV_DEV_DIR = $dir
if ($env:GITHUB_ENV) {
  "LIB=$env:LIB" | Out-File -Append -Encoding utf8 $env:GITHUB_ENV
  "MPV_DEV_DIR=$dir" | Out-File -Append -Encoding utf8 $env:GITHUB_ENV
  $dir | Out-File -Append -Encoding utf8 $env:GITHUB_PATH
}
