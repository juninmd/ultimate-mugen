@echo off
cd /d "%~dp0"
if not exist "target\release\ultimate-mugen.exe" (
    echo Binario release nao encontrado. Compilando com cargo...
    cargo build --release
    if errorlevel 1 (
        echo Falha na compilacao.
        pause
        exit /b 1
    )
)
echo Iniciando Ultimate MUGEN...
"target\release\ultimate-mugen.exe" %*
