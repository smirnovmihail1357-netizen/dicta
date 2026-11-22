---
phase: 7
plan: 07-01
review: 07-01-REVIEW.md
status: resolved
date: 2026-11-22
---

# Review Disposition — Plan 07-01 (Windows-ядро демона)

Оркестратор: GLM-5.3, автономная директива. Ревью: 0 BLOCKER / 2 WARNING / 8 INFO.

## Fixed (8 атомарных коммитов на windows, HEAD 6313014, CI 45066485292 success)

| ID | Коммит | Что сделано |
|----|--------|-------------|
| WR-01 | cad935a | `pipe_already_listening()` — connect_by_path до bind, bail при занятом пайпе (паритет unix main.rs:161-171) + unit-тест |
| WR-02 | b90623b | Стартовый лог cfg-гейт (unix=сокет, windows=пайп); дублирующий лог удалён; комментарии cfg.socket unix-only |
| IN-01 | 753fe3d | UnregisterHotKey в error-пути и на WM_QUIT — ретрай не бьётся в 1409 |
| IN-03 | 7984e0a | Честный win32-код в ошибке: HRESULT & 0xFFFF (1409 читается как HOTKEY_ALREADY_REGISTERED) |
| IN-04 | ab82cb1 | CloseHandle токена + read_unaligned для TOKEN_USER |
| IN-05 | fbaa7a9 | uinput-warm-up и совет про udev/группу input — cfg(unix) |
| IN-06 | 6d8810d | Честный комментарий: cpal 0.18 замыкает windows 0.62, сосуществование мажорных изолировано за cpal |
| IN-08 | 6313014 | pipe_roundtrip: stderr → Stdio::null; DaemonGuard удаляет temp-конфиг в Drop |

## Accepted (не фиксится, с обоснованием)

| ID | Обоснование |
|----|-------------|
| IN-02 (парсер-щели: prtsc/правые модификаторы не парсятся на Win) | Паритет MVP:РЕQUIREMENTS WIN-03 не требует полного матчера клавиш; bare «v»-поглощение — документированное поведение (README-заметка 07-02 T4 упомянет рекомендуемые модификаторы). Расширение парсера — future |
| IN-07 (имя hotkey_card_error_test.dart vs hotkey_card_test.dart плана) | План допускал новое имя файла; гейт «33 файла» пройден; doc-only |

## Верификация фиксов

- `cargo test --workspace` зелёная; `flutter test` 198; analyze 0; cyrillic 0.
- CI run 45066485292 (push 6313014) — success: rust build+tests 1m21s, flutter build 5m20s.
- Локальные изолированные кросс-чеки Win32-сниппетов (x86_64-pc-windows-gnu) — компилируются.
- Payload-дисциплина перепроверена: 0 `{text`-интерполяций в диффе.
