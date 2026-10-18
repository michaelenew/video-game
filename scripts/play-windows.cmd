<# : This first part is a Windows batch file; everything after it is PowerShell.
@echo off
setlocal
rem Double-click to install what the game needs, fetch it, build it and play.
rem The PowerShell below does the work; this part only starts it, because a
rem .cmd is the one kind of script Windows runs on a double-click.
set "LAUNCHER=%~f0"
set "LAUNCHER_ARGS=%*"
powershell -NoProfile -ExecutionPolicy Bypass -Command "Invoke-Expression ([IO.File]::ReadAllText($env:LAUNCHER))"
set "CODE=%ERRORLEVEL%"
if not "%CODE%"=="0" (
    echo.
    echo Something went wrong ^(exit code %CODE%^). The messages above say what.
    pause
)
exit /b %CODE%
#>

# Play the game on Windows, from nothing.
#
# Installs, only if missing:
#   - Visual Studio Build Tools, C++ workload: the linker and Windows SDK that
#     Rust uses on Windows. The big one: a few GB, ten minutes or more.
#   - Git for Windows, to fetch the game and its updates.
#   - Rust (rustup, cargo), which builds the game.
# Then clones the game into %USERPROFILE%\video-game (or pulls the latest if
# it is already there) and runs it with `cargo run --release -p game`.
# Arguments are passed to the game.
#
# Run it again any time: what is installed is skipped, and only what changed
# since last time is rebuilt. If this file sits inside a checkout (as it does
# in the repository's scripts\ folder), it plays that checkout instead and
# leaves its git state alone.
#
# Each installer is downloaded from its publisher and asks for administrator
# rights with the usual Windows prompt.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'   # Windows PowerShell downloads crawl with the bar on
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = 'https://github.com/michaelenew/video-game.git'
$UserDir = $env:USERPROFILE
$Downloads = Join-Path $env:TEMP 'video-game-setup'
New-Item -ItemType Directory -Force -Path $Downloads | Out-Null

$Arch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
$Arm = $Arch -eq 'ARM64'

function Say($text) { Write-Host ''; Write-Host "==> $text" -ForegroundColor Cyan }
function Fail($text) { Write-Host ''; Write-Host "FAILED: $text" -ForegroundColor Red; exit 1 }

function Refresh-Path {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = "$machine;$user;$UserDir\.cargo\bin"
}

function Download($url, $name) {
    $file = Join-Path $Downloads $name
    Write-Host "    downloading $url"
    Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $file
    return $file
}

# Runs an installer, waits for it, and shows the administrator prompt if it
# asks for one. 3010 is "done, restart later", which is fine here.
function Install($file, $arguments) {
    $p = Start-Process -FilePath $file -ArgumentList $arguments -Wait -PassThru
    if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) {
        Fail "$([IO.Path]::GetFileName($file)) exited with code $($p.ExitCode)"
    }
}

Refresh-Path

# --- 1. The C++ build tools ---------------------------------------------------
$VcComponent = if ($Arm) { 'Microsoft.VisualStudio.Component.VC.Tools.ARM64' } else { 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64' }
$Vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'

function Vs-WithCpp {
    if (-not (Test-Path $Vswhere)) { return $null }
    $found = & $Vswhere -products * -requires $VcComponent -latest -property installationPath
    if ($found) { return $found } else { return $null }
}

Say 'Checking for the C++ build tools (Visual Studio Build Tools)'
if (Vs-WithCpp) {
    Write-Host '    already installed'
} else {
    # The bootstrapper adds C++ to an existing Build Tools, or installs one
    # beside any other Visual Studio; it asks Windows for permission itself.
    Write-Host '    installing - a few GB, often ten minutes or more.'
    Write-Host '    Windows will ask for permission; the installer shows its own progress window.'
    $vs = Download 'https://aka.ms/vs/17/release/vs_BuildTools.exe' 'vs_BuildTools.exe'
    Install $vs "--add Microsoft.VisualStudio.Workload.VCTools --add $VcComponent --includeRecommended --passive --wait --norestart"
    if (-not (Vs-WithCpp)) { Fail 'the C++ build tools still are not installed. Running this again usually finishes the job.' }
    Write-Host '    installed'
}

# --- 2. Git -------------------------------------------------------------------
Say 'Checking for Git'
if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Host '    already installed'
} else {
    $release = Invoke-RestMethod -UseBasicParsing -Uri 'https://api.github.com/repos/git-for-windows/git/releases/latest'
    $pattern = if ($Arm) { '^Git-.*-arm64\.exe$' } else { '^Git-.*-64-bit\.exe$' }
    $asset = $release.assets | Where-Object { $_.name -match $pattern } | Select-Object -First 1
    if (-not $asset) { Fail 'could not find the Git for Windows installer. Install it from https://git-scm.com and run this again.' }
    $gitSetup = Download $asset.browser_download_url $asset.name
    Install $gitSetup '/VERYSILENT /NORESTART /NOCANCEL /SP- /SUPPRESSMSGBOXES'
    Refresh-Path
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) { Fail 'Git was installed but cannot be found. Restart the computer and run this again.' }
    Write-Host '    installed'
}

# --- 3. Rust ------------------------------------------------------------------
Say 'Checking for Rust'
if (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Host '    already installed'
} else {
    $rustupUrl = if ($Arm) { 'https://win.rustup.rs/aarch64' } else { 'https://win.rustup.rs/x86_64' }
    $rustupInit = Download $rustupUrl 'rustup-init.exe'
    # Its own install, in the user's folder: no administrator prompt.
    & $rustupInit -y --default-toolchain stable --profile minimal
    if ($LASTEXITCODE -ne 0) { Fail "rustup-init exited with code $LASTEXITCODE" }
    Refresh-Path
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Fail 'Rust was installed but cargo cannot be found. Restart the computer and run this again.' }
    Write-Host '    installed'
}

# --- 4. The game --------------------------------------------------------------
$LauncherDir = Split-Path -Parent $env:LAUNCHER
$Checkout = Split-Path -Parent $LauncherDir
if ((Test-Path (Join-Path $Checkout 'Cargo.toml')) -and (Test-Path (Join-Path $Checkout 'crates\game'))) {
    Say "Playing the checkout this launcher is in: $Checkout"
} else {
    $Checkout = Join-Path $UserDir 'video-game'
    if (Test-Path (Join-Path $Checkout '.git')) {
        Say "Fetching the latest version into $Checkout"
        git -C $Checkout pull --ff-only
        if ($LASTEXITCODE -ne 0) {
            Write-Host '    could not update (files changed by hand?). Playing the version already here.' -ForegroundColor Yellow
        }
    } else {
        Say "Downloading the game into $Checkout"
        git clone -c core.longpaths=true $Repo $Checkout
        if ($LASTEXITCODE -ne 0) { Fail 'git clone did not work. Check the internet connection and run this again.' }
    }
}

Say 'Building and starting the game'
Write-Host '    The first build takes a long while - often 10 to 30 minutes, and several GB of disk.'
Write-Host '    Later runs only rebuild what changed. The game opens in its own window when it is ready.'
Set-Location $Checkout
$GameArgs = @()
if ($env:LAUNCHER_ARGS) { $GameArgs = @($env:LAUNCHER_ARGS -split '\s+' | Where-Object { $_ }) }
cargo run --release -p game -- @GameArgs
exit $LASTEXITCODE
