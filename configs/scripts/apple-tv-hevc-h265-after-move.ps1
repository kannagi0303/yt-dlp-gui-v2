param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$InputPath
)

$ErrorActionPreference = "Stop"

function Find-AppleTvHevcFfmpeg {
    $configsRoot = Split-Path -Parent $PSScriptRoot
    $portableRoot = Split-Path -Parent $configsRoot
    $directPortablePath = Join-Path $portableRoot "tools\ffmpeg\ffmpeg.exe"
    if (Test-Path -LiteralPath $directPortablePath -PathType Leaf) {
        return $directPortablePath
    }

    $toolsRoot = Join-Path $portableRoot "tools"
    if (Test-Path -LiteralPath $toolsRoot -PathType Container) {
        $candidate = Get-ChildItem -LiteralPath $toolsRoot -Filter "ffmpeg.exe" -Recurse -File -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($null -ne $candidate) {
            return $candidate.FullName
        }
    }

    $pathCandidate = Get-Command "ffmpeg.exe" -ErrorAction SilentlyContinue
    if ($null -ne $pathCandidate) {
        return $pathCandidate.Source
    }

    throw "ffmpeg.exe was not found. Install FFmpeg from Options or place ffmpeg.exe under .\tools\ffmpeg\."
}

function New-AppleTvHevcOutputPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SourcePath
    )

    $directory = Split-Path -Parent $SourcePath
    $stem = [System.IO.Path]::GetFileNameWithoutExtension($SourcePath)
    $extension = [System.IO.Path]::GetExtension($SourcePath)

    if ([string]::IsNullOrWhiteSpace($extension) -or $extension.ToLowerInvariant() -ne ".mp4") {
        return Join-Path $directory "$stem.mp4"
    }

    return $SourcePath
}

$resolvedInput = (Resolve-Path -LiteralPath $InputPath).Path
if (-not (Test-Path -LiteralPath $resolvedInput -PathType Leaf)) {
    throw "Input media file was not found: $InputPath"
}

$ffmpeg = Find-AppleTvHevcFfmpeg
$finalOutput = New-AppleTvHevcOutputPath -SourcePath $resolvedInput
$tempOutput = Join-Path (Split-Path -Parent $finalOutput) ([System.IO.Path]::GetFileNameWithoutExtension($finalOutput) + ".apple-tv-hevc.tmp.mp4")

if (Test-Path -LiteralPath $tempOutput) {
    Remove-Item -LiteralPath $tempOutput -Force
}

Write-Host "[apple-tv-hevc] Transcoding to HEVC/H.265: $resolvedInput"

& $ffmpeg `
    -hide_banner `
    -y `
    -i $resolvedInput `
    -map 0:v:0 `
    -map "0:a?" `
    -c:v libx265 `
    -preset medium `
    -crf 26 `
    -tag:v hvc1 `
    -c:a aac `
    -b:a 160k `
    -movflags +faststart `
    -sn `
    $tempOutput

if ($LASTEXITCODE -ne 0) {
    if (Test-Path -LiteralPath $tempOutput) {
        Remove-Item -LiteralPath $tempOutput -Force
    }
    throw "FFmpeg HEVC transcode failed with exit code $LASTEXITCODE"
}

if ($finalOutput -ieq $resolvedInput) {
    Move-Item -LiteralPath $tempOutput -Destination $resolvedInput -Force
} else {
    Move-Item -LiteralPath $tempOutput -Destination $finalOutput -Force
    Remove-Item -LiteralPath $resolvedInput -Force
}

Write-Host "[apple-tv-hevc] Finished: $finalOutput"
