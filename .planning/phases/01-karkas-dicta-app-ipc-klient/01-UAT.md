---
status: testing
phase: 01-karkas-dicta-app-ipc-klient
source: [01-VERIFICATION.md]
started: 2026-11-22T14:20:00
updated: 2026-11-22T14:20:00
---

# Phase 01 UAT — Каркас dicta-app + IPC-клиент

Automated evidence (9/11 must-haves) is in 01-VERIFICATION.md: 13/13 tests, analyze clean,
live SMOKE OK state=idle, cyrillic gate 0, framing/golden byte-exact, engine seam proven,
review fixes effective (675269c), security SECURED (01-SECURITY.md, threats_open: 0).

## Tests

### 1. Window + sidebar + Material You themes

expected: `cd app && flutter run -d linux` — окно открывается, сайдбар переключает
Дашборд/История/Модели/Настройки, все строки русские, тёмная/светлая тема следует
акценту KDE (или graceful fallback), баннер «демон недоступен» отсутствует при живом демоне.
result: [pending] — частичные улики от оркестратора 2026-11-22: `flutter run -d linux`
  запускался на десктопе владельца — окно «Dicta» открылось и сфокусировалось, в логе
  «dynamic_color: Accent color detected» (акцент KDE подхвачен); AT-SPI-дерево Flutter
  Linux нестабильно (известное ограничение a11y Flutter), скриншот на Wayland требует
  portal-диалога — визуальную подпись остаётся дать владельцу.
source: VERIFICATION backstop truth 1
coverage_id: APP-01

### 2. Daemon kill/restart — авто-reconnect без перезапуска приложения

expected: при запущенном приложении `systemctl --user stop dictad` → появляется баннер
«демон недоступен»; `systemctl --user start dictad` → живое состояние возвращается
в течение ~2 с, приложение НЕ перезапускается.
result: [pending]
source: VERIFICATION backstop truth 2
coverage_id: APP-02

### 3. Подпись запретов (6 must-NOT)

expected: подтверждение, что ничего из этого не нарушено: (1) код FlClash не читался/не
копировался — только архитектурные паттерны; (2) протокол демона не менялся — git-история
не содержит правок crates/dictad; (3) приложение не регистрирует глобальные хоткеи;
(4) продиктованный текст не логируется и не сохраняется; (5) исходящий JSON — только
jsonEncode; (6) валидация set-значений остаётся в демоне.
LLM-проверка чистая по всем шести (01-REVIEW.md, 01-SECURITY.md), но подпись — за владельцем.
result: [pending]
source: VERIFICATION prohibition sign-off
coverage_id: APP-01..06

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps

_None so far._

## Deferred Follow-Ups

- nyquist validate-phase и ui-review отложены осознанно (запись в 01-REVIEW-DISPOSITION.md):
  визуальный аудит — после фазы 2, когда будет настоящий UI-контент, не заглушки.
