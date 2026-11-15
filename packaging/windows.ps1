# Builds dist\Parabole-<version>-windows-x64.zip (exe + libmpv-2.dll). Run from a VS developer shell.
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot)
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"').Matches[0].Groups[1].Value

if (-not $env:MPV_DEV_DIR) { ./packaging/windows-mpv.ps1 }
cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }

Remove-Item -Recurse -Force dist -ErrorAction SilentlyContinue
$stage = "dist\Parabole"
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item target\release\parabole.exe $stage
Copy-Item "$env:MPV_DEV_DIR\libmpv-2.dll" $stage
Compress-Archive -Path $stage -DestinationPath "dist\Parabole-$version-windows-x64.zip"
Write-Host "built dist\Parabole-$version-windows-x64.zip"
