param(
    [string]$Destination = (Join-Path $PSScriptRoot '..\src-tauri\resources\ffmpeg.exe')
)

$ErrorActionPreference = 'Stop'

if (Test-Path $Destination) {
    Write-Host "ffmpeg already present: $Destination"
    exit 0
}

$builder = Join-Path $env:RUNNER_TEMP ([System.Guid]::NewGuid().ToString())
if (-not $env:RUNNER_TEMP) {
    $builder = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
}
New-Item -ItemType Directory -Path $builder -Force | Out-Null

try {
    $zip = Join-Path $builder 'ffmpeg.zip'
    Write-Host 'Downloading ffmpeg essentials build...'
    Invoke-WebRequest -Uri 'https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip' -OutFile $zip

    $extract = Join-Path $builder 'extract'
    Expand-Archive -Path $zip -DestinationPath $extract -Force

    $exe = Get-ChildItem -Path $extract -Recurse -Filter 'ffmpeg.exe' |
        Select-Object -First 1
    if (-not $exe) {
        throw 'ffmpeg.exe was not found in the downloaded archive'
    }

    $parent = Split-Path $Destination -Parent
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    Copy-Item $exe.FullName $Destination -Force
    Write-Host "ffmpeg saved to $Destination"
}
finally {
    if (Test-Path $builder) {
        Remove-Item $builder -Recurse -Force -ErrorAction SilentlyContinue
    }
}
