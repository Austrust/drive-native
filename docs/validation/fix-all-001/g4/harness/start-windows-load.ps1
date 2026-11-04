param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$outputRoot = [IO.Path]::GetFullPath($OutputDirectory)
$allowedRoot = [IO.Path]::GetFullPath($PSScriptRoot)
if (-not $outputRoot.StartsWith($allowedRoot + [IO.Path]::DirectorySeparatorChar)) { throw 'New private benchmark log directory required' }
if (Test-Path -LiteralPath $outputRoot) { throw 'Never reuse a sampling directory' }
New-Item -ItemType Directory -Path $outputRoot | Out-Null
$script = Join-Path $PSScriptRoot 'sample-windows-load.ps1'
$executable = (Get-Process -Id $PID).Path
$arguments = @('-NoProfile', '-File', ('"' + $script + '"'), '-OutputDirectory', ('"' + $outputRoot + '"'), '-IntervalSeconds', '2')
$process = Start-Process -FilePath $executable -ArgumentList $arguments -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $outputRoot 'sampler.stdout') -RedirectStandardError (Join-Path $outputRoot 'sampler.stderr')
$record = [ordered]@{ process_id=$process.Id; start_utc=[DateTime]::UtcNow.ToString('o'); interval_seconds=2; script_sha256=(Get-FileHash -Algorithm SHA256 -LiteralPath $script).Hash.ToLowerInvariant() }
[IO.File]::WriteAllText((Join-Path $outputRoot 'sampler-process.json'), ($record | ConvertTo-Json) + "`n", (New-Object Text.UTF8Encoding $false))
$record | ConvertTo-Json
