#!/bin/sh
# entrypoint контейнера dicta: готовим конфиг/модель и стартуем демон.
set -eu

CONFIG="${DICTA_CONFIG:-/config/config.toml}"
MODEL="${DICTA_MODEL:-/models/ggml-medium-q5_0.bin}"

mkdir -p "$(dirname "$CONFIG")" /models "$(dirname "${DICTA_SOCKET:-/run/dicta/dicta.sock}")"
if [ ! -f "$CONFIG" ]; then
    cp /usr/local/share/dicta/config.example.toml "$CONFIG"
    echo "dicta: создан $CONFIG из примера — правь через volume /config"
fi

if [ ! -f "$MODEL" ]; then
    echo "dicta: модель $MODEL не найдена."
    echo "       скачай: curl -L -o $MODEL https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin"
    echo "       или запусти: dicta-entrypoint fetch-model"
fi

# Демон в контейнере не может сам поднять оверлей хост-сессии без Wayland-сокетов:
# предупреждаем, но не падаем — dictad работает и без UI.
if [ -z "${WAYLAND_DISPLAY:-}" ]; then
    echo "dicta: WAYLAND_DISPLAY не задан — оверлей (dicta-ui) не поднимется."
fi

case "${1:-}" in
    fetch-model)
        curl -L --fail -o "$MODEL" \
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin"
        echo "dicta: модель скачана в $MODEL"
        ;;
    shell|sh)
        exec /bin/sh
        ;;
    *)
        exec dictad "$@"
        ;;
esac
