# Usage: .\beni-run.ps1 cargo test
# All remaining arguments go to the native CLI without invoking a command-string parser.
$beniCli = if ($env:BENI_CLI) { $env:BENI_CLI } else { 'beni-buddy-cli' }
& $beniCli run -- @args
exit $LASTEXITCODE
