#!/usr/bin/env bash
# Скачивает ggml-модель whisper.cpp в ~/.local/share/whisper-cpp/ (или в DICTA_MODEL).
set -euo pipefail

MODEL_NAME="${1:-ggml-medium-q5_0.bin}"
BASE_URL="${BASE_URL:-https://huggingface.co/ggerganov/whisper.cpp/resolve/main}"

if [ -n "${DICTA_MODEL:-}" ]; then
    DEST="$DICTA_MODEL"
else
    DEST="${XDG_DATA_HOME:-$HOME/.local/share}/whisper-cpp/$MODEL_NAME"
fi

mkdir -p "$(dirname "$DEST")"
if [ -f "$DEST" ]; then
    echo "уже есть: $DEST ($(stat -c %s "$DEST") байт)"
    exit 0
fi

echo "скачиваю $BASE_URL/$MODEL_NAME -> $DEST"
curl -L --fail --retry 3 -o "$DEST.part" "$BASE_URL/$MODEL_NAME"
mv "$DEST.part" "$DEST"
echo "готово: $DEST"
echo "проверка распознавания: dictactl infer --timing <файл.wav>"
