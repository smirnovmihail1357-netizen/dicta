---
status: testing
phase: 04-nastroyki-brauzer-modeley
source: [04-VERIFICATION.md]
started: 2026-11-22T15:01:00
updated: 2026-11-22T15:01:00
---

# Phase 4 UAT — Настройки + браузер моделей

Automated evidence (15/15 must-haves) is in 04-VERIFICATION.md: батарея **144/144**
(перепущена верификатором живьём), analyze «No issues found!», cyrillic gate 0,
контракт заморозки `git diff 9aee17e HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` пуст,
implements DictaEngine/HfClient/DownloadEngine == 1/1/1, security SECURED 7/7
(04-SECURITY.md, threats_open: 0), review-фиксы WR-01..06 + CR-01..03 закрыты regression-тестами.

**Запуск приложения:** `cd app && flutter run -d linux` при живом демоне
(`systemctl --user start dictad`). Ниже — только живые пункты, которые невозможно
прогнать на фейках. Каждый пункт отмечай `result:` и подписью.

## Tests

### 1. Живая смена настроек на живом демоне

expected: экран «Настройки» уже заполнен из снимка; изменить несколько ключей живьём
(min_level слайдером, auto_stop переключателем, paste_delay_ms, inject_mode, n_threads) —
каждое отпускание шлёт ровно один `set`, значение в форме обновляется из снимка ответа
(без re-fetch), строка статуса показывает ответ демона нейтральным тоном (не красным).
result: [pending]
source: VERIFICATION human_verification 1
coverage_id: CONF-01

### 2. Живой отказ reload при активной диктовке

expected: начать диктовку (Alt+V), затем в «Настройках» сменить модель (ENGINE-ключ,
после confirm) — появляется вербатим-строка «идёт запись или распознавание — смену модели
отложи; заверши диктовку (Alt+V) и повтори», локальный контрол остаётся на прежнем значении;
после завершения диктовки повтор succeeds, появляется reloading-спиннер «модель загружается…»
и ENGINE-группа disabled до гасящего снимка.
result: [pending]
source: VERIFICATION human_verification 2
coverage_id: CONF-02

### 3. Живая загрузка маленькой модели из каталога

expected: «Модели» → каталог → скачать `ggml-tiny-q5_1.bin` (~57 МБ): полоса прогресса
с MB-счётчиком, `.part` появляется в dest_dir, по завершении rename → модель в списке
установленных («уже есть» в каталоге), выбор её в списке работает.
result: [pending]
source: VERIFICATION human_verification 3
coverage_id: MODEL-01

### 4. Прерывание загрузки + повтор → докачка 206

expected: во время загрузки нажать отмену (или оборвать сеть), повторить старт той же
модели — докачка продолжилась с длины `.part` (Range/206), а не с нуля.
result: [pending]
source: VERIFICATION human_verification 4
coverage_id: MODEL-01

### 5. Битая прямая ссылка

expected: вставить в поле прямой ссылки несуществующий путь на HF (404) или закрытый
репозиторий (401) — честная ошибка текстом («HTTP 404» / «репозиторий закрыт или не
найден»), без краха и без стека в UI; кнопка снова активна.
result: [pending]
source: VERIFICATION human_verification 5
coverage_id: MODEL-01

### 6. Удаление неактивной модели с confirm

expected: скачать/иметь вторую модель; удалить неактивную — confirm-диалог, файл удалён
только внутри dest_dir, snackbar «удалено», список обновился; активная модель: кнопка
удаления недоступна/заблокирована.
result: [pending]
source: VERIFICATION human_verification 6
coverage_id: MODEL-02

### 7. Подпись must-NOT (визуальный тон VRAM/confirm/спиннеры)

expected: визуальная подпись владельца, что must-NOT-запреты ощущаются честно: VRAM-блок
и «больше лимита vram» читаются как информационная блокировка, а не ошибка; confirm-диалоги
(ENGINE-set, удаление) имеют правильный тон; reloading-спиннер и строка «модель: …»
не выглядят как error; ни одно состояние не врёт.
result: [pending]
source: VERIFICATION human_verification 7 — подпись запрета, конвенция фаз 01-03
coverage_id: must-NOT / UI-тон

---

**Статус:** testing — до живой подписи владельца фаза не закрывается молча (конвенция:
`human_needed`, никогда не PASSED без UAT).
