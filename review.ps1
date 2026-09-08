# Element Review loop for TontooUI.
#
# Builds the review app once, then relaunches it after every decision:
# Ja/Skip/Zurueck continue automatically, Nein/Done/close stop the loop.
# Decisions live in temp/review/ so a restart resumes at the same element.
#
# Usage: powershell -ExecutionPolicy Bypass -File review.ps1

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location -LiteralPath $root
New-Item -ItemType Directory -Force -Path "temp/review/approved", "temp/review/rejected" | Out-Null
if (-not (Test-Path -LiteralPath "temp/review/state.json")) {
  Set-Content -LiteralPath "temp/review/state.json" -Value '{"index":0,"done":false}' -NoNewline
}
cargo build --example review
if (-not $?) { exit 1 }
while ($true) {
  $before = Get-Content -LiteralPath "temp/review/state.json" -Raw
  & ./target/debug/examples/review.exe
  $code = $LASTEXITCODE
  $after = Get-Content -LiteralPath "temp/review/state.json" -Raw
  $state = $after | ConvertFrom-Json
  if ($state.done) { Write-Host "Review fertig!" -ForegroundColor Green; break }
  if ($code -eq 4) { Write-Host "Nein markiert - gestoppt. Fix sagen, dann weiter." -ForegroundColor Yellow; break }
  if ($after -eq $before) { Write-Host "Ohne Entscheidung geschlossen - gestoppt." -ForegroundColor Yellow; break }
}
