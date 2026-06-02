param(
    [Parameter(Position = 0)]
    [ValidateSet("run", "build", "check", "test")]
    [string]$Command = "run"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

switch ($Command) {
    "run"   { cargo run -p hyperspace-shell }
    "build" { cargo build --workspace }
    "check" { cargo check --workspace }
    "test"  { cargo test --workspace }
}
