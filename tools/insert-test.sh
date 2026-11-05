#!/usr/bin/env bash
# Проверка «вставка в активное окно» без микрофона и без whisper.
#
# Поднимает отдельный терминал (foot), который пишет полученный ввод в файл,
# просит пользователя кликнуть по этому окну (иначе KWin не отдаст ему фокус),
# затем демон вставляет туда текст через буфер обмена + uinput-клавиши.
set -euo pipefail

KEYS="${KEYS:-}"            # напр. KEYS=ctrl+v ./tools/insert-test.sh
TEXT="${TEXT:-Дикта: тест вставки 42}"
CAP="$(mktemp /tmp/dicta-insert-test.XXXXXX)"
APP="${APP:-foot}"

command -v foot >/dev/null || { echo "нужен foot (sudo pacman -S foot)"; exit 1; }

foot --term dicta-insert-test -e bash -c \
  "printf 'жди ввод…\n'; line=''; read -t 40 line; printf 'ПОЛУЧЕНО:[%s]\n' \"\$line\" > '$CAP'" \
  >/dev/null 2>&1 &
FOOT_PID=$!

echo "1. Появилось окно «dicta-insert-test». Кликни по нему (важно: KWin сам фокус не отдаёт)."
echo "2. Enter здесь запустит вставку."
read -r _

if [ -n "$KEYS" ]; then
    dictactl --app "$APP" inject --keys "$KEYS" "$TEXT"
else
    dictactl --app "$APP" inject "$TEXT"
fi

echo "3. Жду, пока терминал закончит чтение (до 40 с)…"
wait "$FOOT_PID" 2>/dev/null || true

echo "=== что дошло до окна ==="
cat "$CAP" 2>/dev/null || echo "ничего не получено"
rm -f "$CAP"
