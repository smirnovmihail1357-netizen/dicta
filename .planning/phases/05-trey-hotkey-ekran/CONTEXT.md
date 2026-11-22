# Phase 5 Context: Трей + хоткей-экран

## Goal
Приложение живёт в фоне: запись и окно доступны из трея (SNI), хоткей-экран честно показывает, как устроен глобальный хоткей демона. Requirements: TRAY-01, TRAY-02, HK-01, HK-02.

## Locked decisions (наследие вехи, обсуждению не подлежат)

- Протокол демона и Rust-часть (crates/, adapters/, systemd/) ЗАМОРОЖЕНЫ: никаких новых команд, правок dictad/DictaEngine-контракта семантически не вводить. Трей использует ТОЛЬКО существующие команды start/stop/cancel/status/set.
- HK-02 = запрет: приложение НЕ регистрирует глобальный хоткей записи на Linux (он уже в демоне; дубль = гонка двойного stop). Это must-NOT с гейтом.
- Кириллица в lib/ вне generated — только через .arb-ключи (гейт cyrillic_gate).
- Форма ответа демона фиксирована: {"base":..., "panel":bool, "settings":...}; SetReply{note,error,settings} из фазы 4 — note IS DATA.
- UI-SPEC не создаётся: ui-plan-gate вернул block:false (решение оркестратора, автономный режим).
- Декомиссия dicta-settings — фаза 8; его код в этой фазе не трогаем.
- Нулевые РАБОЧИЕ зависимости не обязательны для плагин-слоя: tray-функциональность требует Flutter-плагин tray_manager (выбор STACK-исследования: tray_manager 0.9.0, leanflutter nativeapi-ядро). Это СПЛАНИРОВАННАЯ зависимость вехи, не нарушение; версию закрепить точечно в pubspec. Иные новые пакеты — запрещены.

## Проверенные факты (живой код, сверено)

- Хоткей демона: `hotkey` в config.toml (dicta-core/src/config.rs:49, default "off" :90) — evdev-слушатель `/dev/input/event*` (dicta-core/src/hotkey.rs), требует группы `input`/udev ACL. `hotkey` НЕ входит в EDITABLE_KEYS (17 ключей set) — сменить через протокол НЕЛЬЗЯ, только правкой config.toml + рестарт демона → отсюда предупреждение HK-01 «применится после рестарта демона».
- DE-путь: hotkey навешивает DE (README.md:223-234): KWin script регистрирует `Alt+V` (Meta+V занят историей буфера Plasma), Hyprland/Sway — include conf, GNOME — apply.sh `<Ctrl><Alt>v`; цепочка DE hotkey → systemd oneshot → `dictactl toggle --app <focused window>`. Комбинации адаптеров статичны и известны заранее.
- Путь конфига: `DICTA_CONFIG` env → иначе `~/.config/dicta/config.toml` (README.md:117). Приложение может ЧИТАТЬ config.toml (dart:io File, read-only) для показа текущего `hotkey` — протокол не несёт хоткей, чтение файла не нарушает заморозку.
- Окно: window_manager 0.7.0 уже в pubspec (используется: ensureInitialized, legacy-импорт в main.dart). Close-to-tray = setPreventClose(true) + onWindowClose → hide(); честный выход — только пункт меню трея (windowManager.destroy()).
- Состояние для иконки/меню: DictaStatusNotifier (100 мс полл, states idle/recording/transcribing, disconnected-ветка) уже жив; команды start/stop/cancel — DictaControlNotifier (commandInFlight-сериализация).
- Существующие экраны: features/{dashboard,history,models,settings,shell}; слот «Настройки» в NavigationRail. HK-01 «экран хоткея» ожидаемо = карточка/секция в существующей вкладке «Настройки» (нового пункта в rail не вводить без нужды — решение за планёром, но rail у нас 4 пункта и зоны нет).
- Ассетов в app/ сегодня нет (нет assets/ в pubspec) — иконки трея потребуют 2-3 PNG-ассета (idle/recording, возможно transcribing). Источник PNG — вопрос research: сгенерировать минимальные программно (инструментом в tool/ или закоммитить маленькие заготовки) — БЕЗ системных зависимостей.
- tray_manager в тестах: plugin-вызовы — platform-channel; в widget-тестах недоступны → нужен шов (интерфейс TrayService/WindowService с фейком), как DownloadEngine/HfClient в фазе 4.

## Входы из аудитов (обязательства этой фазы)

1. UI-наследие фазы 4 (04-UI-REVIEW.md, погасить одним задачей по канону T8 фазы 4): (а) MergeSemantics-связь лейбл→контрол для слайдеров/сегментов настроек; (б) локализация собственных ошибок (delete failed:/HTTP NNN → .arb) + единый error-snackbar-скелет; (в) in-flight индикаторы на сетевых кнопках + VRAM-readout live на drag; (г) заголовок поиска отдельный .arb-ключ + вес активной модели; (д) liveRegion на fetch-ошибках + excludeSemantics на МБ-счётчике; (е) СВЁРТКА ДЛИННЫХ ЗАПИСЕЙ (history screen — перенос из фазы 3); (ж) ГБ/МБ-литералы → arb-hygiene (IN-01 wontfix фазы 4 — здесь дешёвой частью); (з) ритм формы (12 внутрь / 20-24 межгрупповые) — по мере касания.
2. daemon_engine.dart:61 печатает `$error` (SocketException OS-строка) — перевести на error.runtimeType (residual 04-SECURITY, informational).
3. Точка входа трей-инициализации должна учитывать тесты: buildFakeApp/buildTestApp не должны дёргать plugin-каналы.

## Open questions (researcher разрешает и помечает RESOLVED)

- Точная интеграция tray_manager 0.9.0 на Linux: API TrayManager.instance + setMenuItem? ( TrayManagerListener: onTrayIconMouseDown → show/hide; setIcon из asset-пути), минимальный рабочий сниппет и подводные камни (путь к иконке из assets — копирование в temp?); альтернатива — window_manager только + tray_manager-версия, проверенная с Flutter 3.50.
- Откуда берутся PNG иконки (микро-скрипт в tool/, порождающий валидные PNG байтами — без ImageMagick/inkscape), где хранить (app/assets/tray/*.png), и как резолвить путь иконки в рантайме (assets недоступны по FS-пути напрямую — rootBundle.load + temp-файл, или использовать asset-путь через AssetBundle-извлечение).
- Тест-стратегия: TrayService-шов (interface + fake), что фейкается (iconSet, menu clicks, window hide/show/destroy), как покрыть close-to-tray логику (setPreventClose + onWindowClose handler) юнит-уровнем.
- Чтение config.toml в тестах: resolveConfigPath (DICTA_CONFIG → ~/.config/dicta/config.toml) — чистая функция + fixture; TOML-парсинг НЕ тянуть (!новая зависимость запрещена) — хоткей-строку доставать минимальным regex/поиском строки `hotkey = "..."` в тексте (формат известен; файл конфига пишется dictad). Секция [xxx]? Проверить живой config.toml формат.

## Must-NOT (гейты)

1. НЕ регистрировать глобальный хоткей в приложении (никаких hotkey_manager-вызовов на Linux; пакет НЕ добавлять в pubspec) — HK-02.
2. НЕ менять протокол/демона/Rust-часть (контракт байт-в-байт к phase-base).
3. НЕ писать config.toml (только чтение; редактирование хоткея — вне приложения, инструкция на экране).
4. НЕ трогать dicta-settings crate.
5. Никаких print/логов payload в lib.
6. Кириллица в lib только через .arb.
7. Без новых пакетов, кроме tray_manager (если research не докажет иное — тогда снова к оркестратору).
8. Без новых rail-пунктов без обоснования (4 пункта закреплены фазой 1).

## Оценка объёма для планёра

Фаза меньше фазы 4: шов трея + close-to-tray + меню (T1-T2), HK-карточка с чтением config.toml (T3), UI-наследие одним задачей (T4), финальный гейт. Ожидаемо 5-6 задач, один план, ~100-120k estimate med. Waves: 0 (фейк/шов) → 1 (tracer: трей живой с живым демоном) → 2 (меню/иконки/HK-карточка) → 3 (UI-наследие) → final.
