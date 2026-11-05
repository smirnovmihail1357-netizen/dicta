# Релиз на GitHub

Репозиторий выложен: https://github.com/smirnovmihail1357-netizen/dicta (публичный).
Локальная копия — `/home/wardreg/projects/dicta`, `origin` смотрит на него, рабочие ветки
`main` и `container` (в `container` живёт вынесенный из релиза контейнер).

## 1. Что уже сделано

```bash
git remote add origin https://github.com/smirnovmihail1357-netizen/dicta.git
git push -u origin main
git push -u origin container
```

Два подводных камня, оба проверены вживую:

- **Токен, которым платит git, обязан иметь право `Workflows`.** Иначе push отвергается целиком:
  `remote rejected main -> main (refusing to allow a Personal Access Token to create or update
  workflow .github/workflows/ci.yml without workflow scope)`. Лечение: Edit токена -> включить
  Workflows (fine-grained: Permissions -> Actions/Workflows -> Read and write).
- **В сборочном образе нужен `libxkbcommon-dev`.** `dicta-ui` зависит от
  `smithay-client-toolkit`, чей build-скрипт ищет `xkbcommon` через pkg-config; без пакета
  падает шаг `Build (Vulkan, default)` (у cargo exit 101). Пакет прописан в `ci.yml` и
  `release.yml`; на Arch он есть по умолчанию, поэтому локально сборка проходит, а в CI — нет.

Описание и topics — Settings -> General -> About: `wayland`, `whisper`, `dictation`,
`voice-input`, `kde`, `hyprland`, `pipewire`, `speech-to-text`.

## 2. Правила веток и protected main

```bash
gh api -X PUT repos/smirnovmihail1357-netizen/dicta/branches/main/protection \
  -f 'required_status_checks[strict]=true' \
  -f 'required_status_checks[contexts][]=build + tests (Vulkan и CPU-only)' \
  -F 'required_pull_request_reviews=' \
  -f 'restrict_pushes=true'
```

(не обязательно; CI и так блокирует нерабочие сборки, если включить required check)

## 3. Выпуск версии

Версию ниже обозначаем как `0.N.0` — подставь актуальную, а не копируй пример дословно.

1. Обновить версию в `Cargo.toml` (`[workspace.package] version = "0.N.0"`),
   `cargo metadata >/dev/null` чтобы обновить `Cargo.lock`.
2. Записать раздел в `CHANGELOG.md` (из «Unreleased» в именованный).
3. Коммит + тег + push:

```bash
VERSION=0.N.0                      # актуальная версия этого релиза
git commit -am "release: $VERSION"
git tag -a "v$VERSION" -m "dicta $VERSION"
git push origin main --follow-tags
```

Тег `v*` запускает `.github/workflows/release.yml`:
- архивы `dicta-<tag>-x86_64-linux-vulkan.tar.gz` и `...-cpu.tar.gz` (+ `config.example.toml`);
- GitHub Release с `generate_release_notes: true`.

Контейнер (`docker/Dockerfile`, `compose.yaml`, `docs/CONTAINERS.md`) в релиз 0.3.0 не входит:
он лежит отдельной веткой `container` и в CI не собирается.

## 4. Что проверять перед тегом

```bash
make build && make build-cpu && cargo test --workspace
./tools/insert-test.sh                 # вставка в окно
dictactl calibrate --secs 8            # пороги уровня на живом микрофоне
dictactl atspi-dump --depth 8          # дерево доступности видно сфокусированное поле
dictactl atspi-insert "проверка"       # вставка без буфера обмена (нужен фокус в поле)
```

## 5. Честность описания релиза

Не обещать «работает везде»: подтверждено только на KDE Plasma 6 + PipeWire + AMD. Остальные
адаптеры помечать как «не проверено на живом железе».
