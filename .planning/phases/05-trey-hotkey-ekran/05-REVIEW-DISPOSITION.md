# Phase 5 Review Disposition

```yaml
phase: 5
review: 05-REVIEW.md
status: resolved
date: 2026-11-22
```

## Fixed (7 коммитов фиксера)

| Finding | Коммит | Как закрыт |
|---|---|---|
| WR-01 (трей-меню обходит сериализацию) | 331f79f | in-flight гард в TrayController (двойной клик DBus-пункта → одна команда) + snackbar при error!=null если окно видно; для скрытого окна — тихо с комментарием (у трея нет поверхности ошибок) |
| WR-02 (SNI-недоступен → краш старта) | b10e664 | bootstrapTray() try/catch → windowed-режим без трея; stderr-лог error.runtimeType с \u-эскейпами |
| WR-03 (живая смена иконки не доказана) | daa5c33 | ЖИВОЕ доказательство: IconPixmap через gdbus — idle 208 синих/0 красных → запись 0/208 → отмена 208/0. Присвоение trayIcon.icon реально перерисовывает панель; destroy+create-fallback НЕ добавлен — живой замер опроверг предпосылку (fallback плодил бы SNI-регистрации) |
| WR-04 (TOML одинарные кавычки) | cce3a7b | regex `["']([^"']*)["']` + тест-кейсы |
| IN-01 (edge-паритет resolveConfigPath) | 910fec3 | пустой DICTA_CONFIG = unset; относительный XDG_CONFIG_HOME игнорируется (паритет dirs::config_dir, config.rs:173-178) |
| IN-02 (выход при записи не документирован) | dcd1bf6 | по предложению в README.md/README.ru.md: выход не прерывает запись демона, by design |
| IN-03 (_ConfigError без liveRegion) | 23dafbd | Semantics(liveRegion:true) + тест properties.liveRegion |

## Accepted / Deferred

- **IN-04** (spinner на retry-in-flight) — косметика, побочный эффект Riverpod-отклонения исполнителя Task 4; отложено.
- **IN-05** (error-поле ответа демона отбрасывается в settingsSnapshotProvider) — паритет-долг протокольного снимка; вне фазы 5, кандидат в фазу 6+ (touch protocol-снимка требует сверки с dicta-settings эталоном).
- **Nyquist validate-phase** — отложен до UAT-раунда (канон фаз 2-3).

## Принятые девиации исполнителя (из 05-01-SUMMARY.md)

- Task 4 Riverpod AsyncLoading-гонка → config.hasError + чтение configPath из engineProvider напрямую (гонка disposal вложенного FutureProvider устранена).
- errHttp(String code) — передаётся statusCode.toString() (gen-l10n не принял int).
- Живой smoke через DBusMenu-контракт вместо физического клика (Wayland-сессия агента не имеет Mutter RemoteDesktop) — контракт тот же, что использует панель; пункты UAT 2/8 остаются на подпись пользователя.

## Итог

Батарея 178/178 (+8 к ревью-базе), analyze 0, cyrillic 0, RUST-FROZEN к b81178b, hotkey_manager 0, иконки детерминированы. Блокеров не было; все 4 варнинга закрыты, 3 из 5 info закрыты, 2 приняты с обоснованием.
