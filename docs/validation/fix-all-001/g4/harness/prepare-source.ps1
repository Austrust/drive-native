param(
    [Parameter(Mandatory=$true)][string]$CandidateCommit,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [string]$Repository = (Join-Path $PSScriptRoot '..\fix-7')
)
$ErrorActionPreference = 'Stop'
if ($CandidateCommit -notmatch '^[0-9a-fA-F]{7,40}$') { throw 'Explicit candidate commit required' }
$repoPath = (Resolve-Path -LiteralPath $Repository).Path
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
$allowedRoot = [IO.Path]::GetFullPath($PSScriptRoot)
if (-not $outputPath.StartsWith($allowedRoot + [IO.Path]::DirectorySeparatorChar)) {
    throw 'Preparation output must be a new child of ultra-benchmark'
}
if (Test-Path -LiteralPath $outputPath) { throw 'Preparation output already exists' }
$candidate = (& git -C $repoPath rev-parse ($CandidateCommit + '^{commit}')).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Candidate commit missing' }
$baseline = (& git -C $repoPath rev-parse 'd555944^{commit}').Trim()
if ($LASTEXITCODE -ne 0) { throw 'Baseline commit missing' }
New-Item -ItemType Directory -Path $outputPath | Out-Null
foreach ($entry in @(@('candidate',$candidate), @('baseline',$baseline))) {
    $archive = Join-Path $outputPath ($entry[0] + '.tar')
    & git -C $repoPath archive --format=tar ('--output=' + $archive) $entry[1]
    if ($LASTEXITCODE -ne 0) { throw ('git archive failed: ' + $entry[0]) }
}
$description = [ordered]@{
    candidate_commit = $candidate
    baseline_commit = $baseline
    candidate_archive_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $outputPath 'candidate.tar')).Hash.ToLowerInvariant()
    baseline_archive_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $outputPath 'baseline.tar')).Hash.ToLowerInvariant()
    fixture_overlay = 'The candidate fsearch_fixture.rs is copied byte-for-byte into the baseline archive extraction; baseline product files remain d555944.'
    executed = 'source archival only; no build, test or benchmark'
}
[IO.File]::WriteAllText((Join-Path $outputPath 'source.json'), ($description | ConvertTo-Json -Depth 5) + "`n", (New-Object Text.UTF8Encoding $false))
$description | ConvertTo-Json -Depth 5
