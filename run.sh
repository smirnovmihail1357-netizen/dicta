#!/usr/bin/env bash
# Запуск связки: демон + оверлей. Ctrl+C останавливает оба.
set -euo pipefail
dictad &
pid=$!
sleep 1
dicta-ui &
ui=$!
trap 'kill $pid $ui 2>/dev/null || true' EXIT INT TERM
wait -n $pid $ui
