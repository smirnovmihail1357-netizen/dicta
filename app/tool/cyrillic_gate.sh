#!/usr/bin/env bash
# Phase 1 APP-05 guard: zero non-comment Cyrillic literals in lib/**/*.dart.
# app_ru.arb is the single source of Russian UI text. The generated l10n output
# (lib/l10n/generated/) legitimately copies those literals, so exclude it.
cd "$(dirname "$0")/.." || exit 1

count=$(
  grep -rnP '[А-Яа-яЁё]' lib --include='*.dart' \
    | grep -v '^lib/l10n/generated/' \
    | grep -vE ':[0-9]+:[[:space:]]*(//|/\*|\*)' \
    | wc -l
)

if [ "$count" -gt 0 ]; then
  echo "Cyrillic gate FAILED: $count non-comment Cyrillic literal(s) in lib/**/*.dart (excluding lib/l10n/generated/)" >&2
  exit 1
fi

echo "$count"
exit 0
