param(
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [ValidateRange(1,60)][int]$IntervalSeconds = 2
)
$ErrorActionPreference = 'Stop'
$outputRoot = [IO.Path]::GetFullPath($OutputDirectory)
$allowedRoot = [IO.Path]::GetFullPath($PSScriptRoot)
if (-not $outputRoot.StartsWith($allowedRoot + [IO.Path]::DirectorySeparatorChar)) { throw 'Private benchmark log directory required' }
if (-not (Test-Path -LiteralPath $outputRoot -PathType Container)) { throw 'Launcher must create new log directory first' }
$samplePath = Join-Path $outputRoot 'windows-load.samples.jsonl'
$stopPath = Join-Path $outputRoot 'stop.requested'
$summaryPath = Join-Path $outputRoot 'windows-load.summary.json'
if (Test-Path -LiteralPath $samplePath) { throw 'Never overwrite sampling evidence' }
$encoding = New-Object Text.UTF8Encoding $false
$writer = New-Object IO.StreamWriter($samplePath, $false, $encoding)
$writer.AutoFlush = $true
$sampleCount = 0
$errorCount = 0
$totalCost = 0.0
$maxCost = 0.0
$startUtc = [DateTime]::UtcNow.ToString('o')
try {
    while (-not (Test-Path -LiteralPath $stopPath)) {
        $timer = [Diagnostics.Stopwatch]::StartNew()
        $row = [ordered]@{ utc = [DateTime]::UtcNow.ToString('o'); source = 'Windows host CIM'; interval_seconds = $IntervalSeconds }
        try {
            $processor = Get-CimInstance -ClassName Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'"
            $memory = Get-CimInstance -ClassName Win32_PerfFormattedData_PerfOS_Memory
            $row.cpu_percent = $processor.PercentProcessorTime
            $row.memory_available_mib = $memory.AvailableMBytes
            $row.memory_committed_bytes = $memory.CommittedBytes
            $row.memory_commit_limit_bytes = $memory.CommitLimit
            $row.memory_pages_per_second = $memory.PagesPerSec
        } catch {
            $errorCount += 1
            $row.error_type = $_.Exception.GetType().FullName
            $row.error = $_.Exception.Message
        }
        $timer.Stop()
        $row.collection_ms = $timer.Elapsed.TotalMilliseconds
        $totalCost += $row.collection_ms
        $maxCost = [Math]::Max($maxCost, $row.collection_ms)
        $writer.WriteLine(($row | ConvertTo-Json -Compress -Depth 4))
        $sampleCount += 1
        $remaining = [Math]::Max(1, $IntervalSeconds * 1000 - [int]$timer.Elapsed.TotalMilliseconds)
        Start-Sleep -Milliseconds $remaining
    }
} finally {
    $writer.Dispose()
    $summary = [ordered]@{
        start_utc=$startUtc; end_utc=[DateTime]::UtcNow.ToString('o'); samples=$sampleCount; failed_samples=$errorCount
        interval_seconds=$IntervalSeconds; total_collection_ms=$totalCost; max_collection_ms=$maxCost
        scope='Windows host total CPU and memory; low-frequency CIM sampling adds overhead; WSL guest counters are separate.'
        command='Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor (_Total), Win32_PerfFormattedData_PerfOS_Memory'
        sample_sha256=(Get-FileHash -Algorithm SHA256 -LiteralPath $samplePath).Hash.ToLowerInvariant()
    }
    [IO.File]::WriteAllText($summaryPath, ($summary | ConvertTo-Json -Depth 4) + "`n", $encoding)
}
