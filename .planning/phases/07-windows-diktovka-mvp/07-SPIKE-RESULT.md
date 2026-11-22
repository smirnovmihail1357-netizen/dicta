---
phase: 07-windows-diktovka-mvp
plan: "07-02"
status: scaffold  # SCM: заполняется пользователем после физического прогона спика
created: 2026-11-22
source: spike/delayed_render (протокол RESEARCH §5, пункты 1-9)
gate: "Task 2 checkpoint:human-verify → Task 3 precondition (Must-NOT 8)"
---

# 07-SPIKE-RESULT — живые замеры delayed render (Windows)

> **ЭТО SCAFFOLD.** Строки ниже содержат маркеры `<заполнить>` (заглушки).
> Пока они есть, **production-вставка `inject_win` ЗАПРЕЩЕНА** (Must-NOT 8 / locked 5):
> Task 3 останавливается с EXECUTION INCOMPLETE.
>
> Инструкция: `spike/delayed_render/README.md`. Бин печатает строки `SPIKE | ...`
> в stdout — скопируйте их в колонку «факт» **построчно**, сохраняя префикс
> `факт: `. Содержимое транскрипта НЕ копировать (приватность).

## Таблица замеров (протокол RESEARCH §5)

| № | Замер | Ожидание (RESEARCH / PITFALLS) | Факт с Windows-машины |
|---|-------|--------------------------------|------------------------|
| 1 | Baseline буфера перед promise | `GetClipboardData(CF_UNICODETEXT)` сохраняется; печатается только длина, не текст | факт: <заполнить> |
| 2 | Promise + 4 оптаут-формата | `SetClipboardData(fmt, NULL)` возвращает NULL «на успехе»; `windows-rs` даёт Err — это ожидаемо; 4 формата: ExcludeClipboardContentFromMonitorProcessing / CanIncludeInClipboardHistory / CanUploadToCloudClipboard / Clipboard Viewer Ignore | факт: <заполнить> |
| 3 | Подтверждение успеха promise | `GetLastError` протухший (НЕ индикатор); `IsClipboardFormatAvailable(CF_UNICODETEXT)` = true; `GetClipboardOwner` = hwnd спика | факт: <заполнить> |
| 4 | t1−t0 / t2 / повторные рендеры | `WM_RENDERFORMAT` приходит по факту чтения цели; t1−t0 — доли секунды; Chromium (probe-then-read) даёт ≥2 рендера | факт: <заполнить> |
| 5 | Фактический settle (5-с лог тишины) | сотни мс после рендера (PITFALLS:44 ~400 мс); потолок `clipboard_restore_ms` = 2500 мс (config.rs:93) | факт: <заполнить> |
| 6 | Кейсы-цели × 2 режима | Notepad — вставка; Chromium — ≥2 рендера; Windows Terminal — Ctrl+V / Ctrl+Shift+V; elevated-Notepad — тихий отказ (UIPI); зажатый модификатор — аккорд искажается | факт: <заполнить> |
| 7 | Тип окна владельца | message-only (`HWND_MESSAGE`) vs скрытое `WS_OVERLAPPED` без `WS_VISIBLE`: какой получает `WM_RENDERFORMAT` в течение 10 с | факт: <заполнить> |
| 8 | `WM_RENDERALLFORMATS` перед выходом | обязательно: рендер ВСЕХ 5 форматов; не отрендерить = потеря данных | факт: <заполнить> |
| 9 | PowerShell `Get-Clipboard` + Win+V | PowerShell — второй способ дернуть рендер (счётчик растёт); Win+V — транскрипта в истории НЕТ (оптауты сработали) | факт: <заполнить> |

## Выводы для inject_win (заполняются после таблицы)

- Тип окна владельца для production (из замера 7): факт: <заполнить>
- Фактический settle для потолка `clipboard_restore_ms` (из замера 5): факт: <заполнить>
- Повторные рендеры у Chromium ≥2 — да/нет (из замера 4): факт: <заполнить>
- Поведение `WM_RENDERALLFORMATS` (из замера 8): факт: <заполнить>
- PowerShell-нюанс (из замера 9): факт: <заполнить>
- Результат Win+V / облако — утечка есть или нет (из замера 9): факт: <заполнить>

## Гейт перехода к Task 3

Task 3 (`inject_win` production) читает ЭТОТ файл первым делом. Разрешение на код
даётся только когда: (а) все 9 строк таблицы содержат реальные значения,
(б) секция выводов заполнена, (в) маркеров `<заполнить>` / `TBD` / `заглушка`
не осталось, (г) есть docs-коммит `docs(07-02): spike result ...` на main.
До этого — EXECUTION INCOMPLETE со списком недостающих замеров.
