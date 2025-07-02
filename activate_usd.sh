#!/bin/bash
# Activate USD environment for Nodle development

export NODLE_USD_ROOT="/Users/brian/nodle-working/vendor/usd"
export USD_INSTALL_ROOT="/Users/brian/nodle-working/vendor/usd"
export PYTHONPATH="/Users/brian/nodle-working/vendor/usd/venv/lib/python3.9/site-packages:$PYTHONPATH"
export PATH="/Users/brian/nodle-working/vendor/usd/venv/bin:$PATH"

echo "USD environment activated for Nodle"
echo "USD_INSTALL_ROOT: $USD_INSTALL_ROOT"
echo "Python: $(which python)"
