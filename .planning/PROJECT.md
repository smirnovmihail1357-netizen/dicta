# dicta

## What This Is

Системный голосовой диктант для Linux/Wayland на Rust: глобальный хоткей → запись → whisper.cpp →
вставка распознанного текста в активное окно. Живёт как демон `dictad` с unix-сокетом, поверх —
layer-shell оверлей с пилюлей записи, CLI `dictactl` и окно настроек `dicta-settings`.
Считает себя (и становится) кроссплатформенным продуктом: веха v1.1 «Dicta App» добавляет
полноценное Flutter-приложение в духе FlClash и каркас Windows-версии.

## Core Value

Нажал хоткей — заговорил — текст оказался в курсоре. Быстро и предсказуемо, без фокуса на окно диктанта.

## Requirements

### Validated

- ✓ Демон dictad с unix-сокетом: команды status/settings/set/start/stop/toggle/cancel/inject/panel, валидация и запись `~/.config/dicta/config.toml` — существующее ядро
- ✓ Захват звука с уровнями, порог тишины, авто-стоп по тишине с грейс-периодом — существующее
- ✓ STT через whisper.cpp (Vulkan, gpu_device=0 — быстрое устройство, замер ~200 мс; greedy beam_size=1) — существующее
- ✓ Вставка текста в активное окно: буфер+клавиши (проверка wl-paste вместо слепого sleep, фоновое восстановление буфера), AT-SPI, режимы paste/atspi/clipboard/stdout — существующее
- ✓ Layer-shell оверлей dicta-ui: пилюля записи, быстрый старт/стоп — существующее
- ✓ Окно настроек dicta-settings (eframe/egui): слайдеры/чекбоксы с отправкой в демон, красная/зелёная строка состояния, «поверх всех» через KWin — существующее
- ✓ Браузер моделей Hugging Face в dicta-settings: каталог 6 проверенных, поиск по HF API (фильтр ASR, сортировки, пагинация, подпапки, VRAM-лимит), прямая ссылка; скачивание curl с `.part`/докачкой — существующее
- ✓ dictactl: снимок/изменение настроек, запуск окна, инжект — существующее
- ✓ Задержки вывода починены: inference в отдельном воркере, вставка ~20–40 мс после подтверждения буфера — существующее

### Active

<!-- Веха v1.1 «Dicta App» — всё, что ниже, гипотезы до поставки -->

- [ ] Полноценное Flutter-приложение `dicta-app` (Material You, тёмная/светлая тема, лэйаут в духе FlClash: сайдбар/карточки) — единый центр управления
- [ ] Дашборд записи в приложении: состояние (запись/пауза/ошибка), живой уровень сигнала, кнопка старт/стоп, распознанный текст в реальном времени
- [ ] История диктовок: журнал распознаваний, скопировать/повторно вставить
- [ ] Перенос настроек и браузера моделей в dicta-app (валидация остаётся в демоне/ядре)
- [ ] Трей и глобальные хоткеи на десктопе: свернуть в трей, запись без открытия окна
- [ ] Windows-ветка (отдельная GitHub-ветка): каркас приложения, собирающийся под Windows, с рабочей диктовкой MVP (WASAPI-захват + вставка SendInput)
- [ ] Автоматическая сборка Windows-пакетов через GitHub Actions (Flutter для Windows не кросс-компилируется с Linux — CI на виндовом раннере)
- [ ] dicta-settings (egui) упраздняется после паритета dicta-app по функциям

### Out of Scope

- Полный паритет Windows-версии с Linux (все режимы вставки, оверлей) — это следующая веха, сейчас только каркас с диктовкой
- Трогать демон, протокол сокета и оверлей dicta-ui на Linux — они работают и не должны ломаться
- Android/macOS — флаттер-приложение делает возможным, но не в этой вехе
- Монетизация — хобби-проект

## Context

- Репо: github.com/smirnovmihail1357-netizen/dicta, ветка main; 22 незапушенных коммита на момент старта вехи
- Карта кодовой базы: `.planning/codebase/` (7 документов, 2026-11-22)
- Вдохновение дизайна: FlClash (github.com/chen08209/FlClash) — Flutter/Dart UI (Material You, Riverpod), ядро отдельным процессом: Unix-сокет на Linux/macOS, именованный канал на Windows, трей + глобальные хоткеи на десктопе
- dicta уже имеет ту же архитектуру «ядро + IPC», что FlClash: демон + клиенты
- Пользователь: Mihail, машина cachyos-LLM (KDE Plasma 6 Wayland, RX 7900 XTX + 610M iGPU); есть Windows-ПК для живого теста
- Рабочий процесс: планирование — GLM-5.3, исполнение кода — субагенты (бесплатная hy4 flash free), ревью/сборка/тесты/коммиты — GLM-5.3; пользователь диктует голосом
- Известные слабые места (`.planning/codebase/CONCERNS.md`): узкая гонка двойного stop, cancel-не-отменяет-inference, слепые sleep'ы 300/20/600/1500 мс в inject.rs, три копии схемы Settings, панель оверлея — кандидат на упрощение, inject.rs без тестов

## Constraints

- **Tech stack**: Flutter (Dart) для приложения; Rust-ядро whisper.cpp остаётся; мост — FFI/IPC как у FlClash — решение пользователя по итогам изучения FlClash
- **Совместимость**: Linux-часть обязана работать поверх существующего демона dictad без изменения протокола сокета
- **Сборка**: Windows-пакеты только через CI на Windows-раннере (Flutter не кросс-компилируется с Linux); Rust-часть можно кросс-компилировать
- **Производительность**: задержка «стоп → текст в окне» уже ~20–40 мс — не ухудшать
- **Ресурсы**: исполнение бесплатной моделью hy4 — решения с большим контекстом планирования делать GLM-5.3

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Flutter (а не egui-редизайн) для нового окна | Пользователь выбрал дизайн-паритет с FlClash (Material You, темы, карточки); FlClash доказал схему «Flutter UI + ядро по IPC» | — Pending |
| Windows-ветка: каркас MVP, не полный паритет | Полный паритет — отдельная веха; сначала рабочий цикл записи на Windows (WASAPI + SendInput) | — Pending |
| CI (GitHub Actions) для Windows-сборок | Flutter для Windows не кросс-компилируется с Linux; пользователь хочет автоматическую сборку — CI даёт её без Windows-тулчейна на Linux | — Pending |
| Оверлей dicta-ui и демон не трогаем | Работают и привычны; приложение их дополняет, не заменяет (на Linux) | — Pending |
| gpu_device=0, beam_size=1 в конфигах | Замер по журналу: 0 — быстрое (~200 мс), 1 — медленное (~4 с); greedy пока устраивает по качеству | ✓ Good |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-11-22 after initialization (v1.1 «Dicta App»)*
