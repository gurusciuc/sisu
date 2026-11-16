param([Parameter(Mandatory)][string]$Directory)
$ErrorActionPreference='Stop'
if (Test-Path -LiteralPath $Directory) {throw 'Choose a new output directory'}
New-Item -Type Directory $Directory|Out-Null
$dir=(Resolve-Path $Directory).Path
$zip="$dir/package.zip"
$url='https://github.com/uBlockOrigin/uBOL-home/releases/download/2026.1110.1816/uBOLite_2026.1110.1816.chromium.zip'
& curl.exe -fLsS --retry 2 -o $zip $url
if($LASTEXITCODE -or (Get-FileHash $zip).Hash -ne '85fe5909366e8ada66c5b331bbdbf69cf5100245ed6161b1e4e2f1f848ec7d03'){throw 'uBOL integrity check failed'}
New-Item -Type Directory "$dir/unpacked"|Out-Null
& tar.exe -xf $zip -C "$dir/unpacked"
if($LASTEXITCODE){throw 'Extraction failed'}
Write-Output "$dir/unpacked"
