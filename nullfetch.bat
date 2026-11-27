@echo off
setlocal
set "SCRIPT=%~dp0nullfetch.py"
where py >nul 2>nul
if %errorlevel%==0 (
  py -3 "%SCRIPT%" %*
  exit /b %errorlevel%
)
where python >nul 2>nul
if %errorlevel%==0 (
  python "%SCRIPT%" %*
  exit /b %errorlevel%
)
echo nullfetch: Python 3 not found. Install Python from https://python.org/downloads/ 1>&2
exit /b 1
