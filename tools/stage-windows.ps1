param([Parameter(Mandatory)][string]$Sdk, [Parameter(Mandatory)][string]$Target, [string]$Out = "$PSScriptRoot/../dist/windows-x64")
$ErrorActionPreference='Stop'
New-Item -Type Directory -Force $Out | Out-Null
foreach ($item in (Get-ChildItem -LiteralPath $Sdk -File)) {
    if ($item.Extension -in '.dll','.pak','.bin','.dat' -or $item.Name -in 'vk_swiftshader_icd.json','CREDITS.html') {
        Copy-Item -LiteralPath $item.FullName -Destination $Out
    }
}
Copy-Item -LiteralPath "$Sdk/locales" -Destination $Out -Recurse -Force
Copy-Item -LiteralPath "$Sdk/bootstrap.exe" -Destination "$Out/desktop_browser.exe"
Copy-Item -LiteralPath "$Target/release/desktop_browser.dll" -Destination $Out
foreach($name in 'LICENSE','NOTICE','THIRD_PARTY.md','notices'){
    Copy-Item -LiteralPath "$PSScriptRoot/../$name" -Destination $Out -Recurse -Force
}
Write-Output (Resolve-Path "$Out/desktop_browser.exe")
