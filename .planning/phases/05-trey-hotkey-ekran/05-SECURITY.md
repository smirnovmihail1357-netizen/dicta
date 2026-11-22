---
phase: 5
phase_name: tray-hotkey-ekran
audit_date: 2026-11-22
security_asvs_level: 1
block_on: high
threats_total: 7
threats_open: 0
verdict: SECURED
phase_base: b81178b
audited_range: 74dcce2..23dafbd
---

# Phase 5 Security Audit — Трей + хоткей-экран

Вердикт: **SECURED** — 7/7 угроз закрыты, подтверждено живым кодом (ASVS L1 + L2-проверка границ).
Диф фазы: `git diff b81178b..HEAD -- app/` (коммиты 74dcce2..23dafbd + review-фиксы WR/IN).

## Threat Register — верификация

| ID | Категория | Severity | Диспозиция | Статус | Evidence |
|----|-----------|----------|------------|--------|----------|
| T-5-01 | Tampering | medium | mitigate | CLOSED | hotkey-строка только в Text (hotkey_card.dart:123); exec/Process.run в config_path+hotkey_card == 0; set('hotkey') в lib == 0; hotkey ∉ EDITABLE_KEYS (config.rs:142-160, живой замер); extractHotkey — линейный regex read-only |
| T-5-02 | Information Disclosure | medium | mitigate | CLOSED | daemon_engine.dart:61 = `${error.runtimeType}`; grep `\$error` по lib == 0; диф daemon_engine против b81178b — только residual-фикс; residual 04-SECURITY погашен |
| T-5-03 | EoP / DoS | low | mitigate | CLOSED | команды меню только через dictaControlProvider (tray_bootstrap.dart:131-143), двойной гард WR-01 (_menuCommandInFlight + commandInFlight); engineProvider в tray_bootstrap == 0; Timer/Socket/Stopwatch == 0; destroy только из пункта «выход» (tray_bootstrap.dart:112), крестик → hide |
| T-5-04 | Tampering (supply chain) | high | mitigate | CLOSED | pin tray_manager: 0.9.0 (pubspec.yaml:36); lock-diff = ровно один блок tray_manager 0.9.0, других записей нет; legitimacy: verified publisher leanflutter.dev (05-RESEARCH); crates/adapters/systemd байт-в-байт к b81178b |
| T-5-05 | Tampering (FS) | medium | mitigate | CLOSED | writeAs/openWrite/FileMode в config_path.dart + hotkey_card.dart == 0; единственная FS-операция — File(path).readAsString() (hotkey_card.dart:30) |
| T-5-06 | Repudiation / DoS | low | mitigate | CLOSED | перегенерация dart tools/gen_tray_icons.dart → git diff --quiet по app/assets/tray/ пуст (байт-в-байт, живая проба); PNG-сигнатура валидна; генератор чистый dart:io без сети |
| T-5-SC | Tampering (supply chain) | high | mitigate | CLOSED | tray_manager в lib == 1 (только core/tray.dart), в test/ == 0; buildFakeApp null-means-no-override (fake_dicta_engine.dart:189-190,206-207); hotkey_manager в pubspec/lock/lib == 0 (HK-02); stderr-логи bootstrap — только error.runtimeType (tray_bootstrap.dart:44), пейлоады/конфиг-текст не логируются |

## Threat Flags (SUMMARY)

Секции `## Threat Flags` в 05-01-SUMMARY.md нет — новая поверхность исполнителем не задекларирована.
Post-plan коммиты (WR-01..04, IN-01..03) сверены: гарды, read-only парсинг, docs/a11y — поверхность не расширяют.
Unregistered flags: none.

## Фазовые гейты (замеры аудита, все зелёные)

- flutter test: 178/178 «All tests passed!»; flutter analyze: «No issues found!»
- bash tool/cyrillic_gate.sh → 0
- git diff --quiet b81178b HEAD -- crates/ adapters/ systemd/ → пусто
- implements DictaEngine/TrayService/WindowShell: 1 фейк в test/ + 1 реализация в lib/ на шов
- print( в lib == 0; $error в lib == 0

## Residuals (informational)

1. Process.runSync('id', ['-u']) — daemon_engine.dart:21: унаследованный код ранних фаз, дифом фазы 5 не тронут; фикс-аргумент константен.
2. control.lastError демон-ошибок в snackbar (tray_bootstrap _reportCommandError) — протокольные ошибки reply, не диктовки; канон error-карточки дашборда.
3. Принятое SNI-отклонение (левый клик = пункт меню, оркестратор 2026-11-22) — требует UAT-подписи в /gsd-verify-work 05; на код не влияет.

## История угроз фазы

- 04-SECURITY residual (daemon_engine:61 $error) — погашен в Task 1 фазы 5, статус CLOSED.
