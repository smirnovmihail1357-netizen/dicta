#!/usr/bin/env bash
# Запуск dicta-контейнера с доступом к живой Wayland/PipeWire-сессии.
#
# Это «прозрачный» запуск для проверки перед Portainer/compose: он монтирует
# сокет-каталог сессии, /dev/uinput и /dev/dri, чтобы демон писал в твоё окно.
set -euo pipefail

IMAGE="${IMAGE:-dicta:local}"
RUNTIME="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
WAYLAND="${WAYLAND_DISPLAY:-wayland-1}"

docker run --rm -it \
    --name dicta \
    --device /dev/uinput \
    --device /dev/dri \
    --group-add video --group-add render \
    -v "$RUNTIME:$RUNTIME" \
    -v dicta-config:/config \
    -v dicta-models:/models \
    -e XDG_RUNTIME_DIR="$RUNTIME" \
    -e WAYLAND_DISPLAY="$WAYLAND" \
    -e DICTA_CONFIG=/config/config.toml \
    -e DICTA_SOCKET=/run/dicta/dicta.sock \
    -e DICTA_MODEL=/models/ggml-medium-q5_0.bin \
    "$IMAGE" "$@"
