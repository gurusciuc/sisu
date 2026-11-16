param([string]$BuildRoot = "$env:TEMP/db-cef156", [string]$Out = "$PSScriptRoot/../dist/preview")
$ErrorActionPreference='Stop'
$repo=(Resolve-Path "$PSScriptRoot/..").Path
$null=New-Item -ItemType Directory -Force $BuildRoot
$BuildRoot=(Resolve-Path $BuildRoot).Path
$archive='cef_binary_156.0.30+g6028375+chromium-156.0.8078.96_windows64_minimal'
$expected='74fd321975e99703acd8fc7e3c1b0859292f61b388f4052a0cd7ef1fb3b29f6e'
$sdk="$BuildRoot/sdk"
if (!(Test-Path -LiteralPath "$sdk/.complete-$expected")) {
    $file="$BuildRoot/$archive.tar.bz2"
    if (!(Test-Path -LiteralPath $file)) {
        & curl.exe --fail --location --retry 2 --output "$file.partial" ('https://cef-builds.spotifycdn.com/'+$archive.Replace('+','%2B')+'.tar.bz2')
        if ($LASTEXITCODE) {throw 'CEF download failed'}
        Move-Item -LiteralPath "$file.partial" -Destination $file -Force
    }
    if ((Get-FileHash -LiteralPath $file).Hash -ne $expected){throw 'Invalid CEF archive SHA-256'}
    & tar.exe -xf $file -C $BuildRoot
    if($LASTEXITCODE){throw 'CEF extraction failed'}
    New-Item -Type Directory -Force $sdk|Out-Null
    $source="$BuildRoot/$archive"
    foreach ($name in 'include','cmake','libcef_dll','CMakeLists.txt','CREDITS.html','LICENSE.txt') {
        Copy-Item -LiteralPath "$source/$name" -Recurse -Force -Destination $sdk
    }
    Copy-Item "$source/Release/*","$source/Resources/*" -Destination $sdk -Recurse -Force
    [IO.File]::WriteAllText("$sdk/archive.json",(@{name="$archive.tar.bz2";type='minimal';sha1='1fedfc8dd0274caec9183a709ad1ed7856867b91';size=175176258;last_modified='2026-11-13T04:49:27.03Z'}|ConvertTo-Json))
    [IO.File]::WriteAllText("$sdk/.complete-$expected",'verified')
}
$vs=& "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(!$vs){throw 'MSVC x64 build tools required'}
$env:Path="$vs/Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin;$vs/Common7/IDE/CommonExtensions/Microsoft/CMake/Ninja;$env:Path"
$env:CEF_PATH=$sdk
$env:CARGO_TARGET_DIR="$BuildRoot/target"
Push-Location $repo
try {
    & cargo build --locked --release --lib
    if($LASTEXITCODE){throw 'Release build failed'}
    & "$PSScriptRoot/stage-windows.ps1" -Sdk $sdk -Target $env:CARGO_TARGET_DIR -Out $Out
} finally {Pop-Location}
