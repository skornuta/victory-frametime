@echo off
rem Run a title with the frametime dump on.
rem   run-frametime.cmd "D:\game\game.exe"
rem Build first: cargo build --release -p victory
rem The dump goes to <cache>\frametime.csv (default %LOCALAPPDATA%\victory\cache);
rem set VICTORY_FRAMETIME_DUMP=<path> before this script to put it elsewhere.
cd /d "%~dp0"
setlocal
if "%~1"=="" (
    echo usage: run-frametime.cmd ^<path to the title executable^>
    echo the dump goes to %%LOCALAPPDATA%%\victory\cache\frametime.csv unless VICTORY_FRAMETIME_DUMP names a path
    exit /b 1
)
if not exist "target\release\victory.exe" (
    echo no target\release\victory.exe: run "cargo build --release -p victory" first
    exit /b 1
)
if not defined VICTORY_FRAMETIME_DUMP set VICTORY_FRAMETIME_DUMP=1
"target\release\victory.exe" run "%~1" --execute
