@echo off
setlocal
set "BIN=%~dp0nullfetch.exe"
if exist "%BIN%" (
  "%BIN%" %*
  exit /b %errorlevel%
)
set "BIN=%~dp0target\release\nullfetch.exe"
if exist "%BIN%" (
  "%BIN%" %*
  exit /b %errorlevel%
)
echo nullfetch: binary not found. Build with: cargo build --release 1>&2
exit /b 1
