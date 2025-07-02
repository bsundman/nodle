@echo off
rem Activate USD environment for Nodle development

set NODLE_USD_ROOT=/Users/brian/nodle-working/vendor/usd
set USD_INSTALL_ROOT=/Users/brian/nodle-working/vendor/usd
set PYTHONPATH=/Users/brian/nodle-working/vendor/usd/venv\Lib\site-packages;%PYTHONPATH%
set PATH=/Users/brian/nodle-working/vendor/usd/venv\Scripts;%PATH%

echo USD environment activated for Nodle
echo USD_INSTALL_ROOT: %USD_INSTALL_ROOT%
echo Python: 
where python
