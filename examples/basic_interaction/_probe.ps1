param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Rest)
& .\_probe.exe @Rest
exit $LASTEXITCODE