# Релиз на GitHub

Репозиторий пока локальный (`/home/wardreg/projects/dicta`, ветка `main`). Порядок, чтобы
выложить его и выпускать релизы.

## 1. Создать репозиторий

Вариант через `gh` (если поставишь `pacman -S github-cli`):

```bash
cd ~/projects/dicta
git init -b main           # уже сделано: ветка переименована из master
git remote add origin https://github.com/<owner>/dicta.git
git push -u origin main
```

или:

```bash
gh repo create dicta --public --source=. --remote=origin --push \
  --description "System-wide voice dictation for Linux/Wayland (whisper.cpp + PipeWire + uinput)"
```

Дальше в Settings -> General: About (описание + topics `wayland`, `whisper`, `dictation`,
`voice-input`, `kde`, `hyprland`, `pipewire`, `speech-to-text`), включить Issues.

## 2. Правила веток иProtected main

```bash
gh api -X PUT repos/<owner>/dicta/branches/main/protection \
  -f 'required_status_checks[strict]=true' \
  -f 'required_status_checks[contexts][]=build + tests (Vulkan и CPU-only)' \
  -F 'required_pull_request_reviews=' \
  -f 'restrict_pushes=true'
```

(не обязательно; CI и так блокирует нерабочие сборки, если включить required check)

## 3. Выпуск версии

1. Обновить версию в `Cargo.toml` (`[workspace.package] version = "0.2.0"`),
   `cargo metadata >/dev/null` чтобы обновить `Cargo.lock`.
2. Записать раздел в `CHANGELOG.md` (из «Unreleased» в именованный).
3. Коммит + тег + push:

```bash
git commit -am "release: 0.2.0"
git tag -a v0.2.0 -m "dicta 0.2.0"
git push origin main --follow-tags
```

Тег `v*` запускает `.github/workflows/release.yml`:
- архивы `dicta-<tag>-x86_64-linux-vulkan.tar.gz` и `...-cpu.tar.gz` (+ `config.example.toml`);
- GitHub Release с `generate_release_notes: true`;
- образ `ghcr.io/<owner>/dicta:<tag>` и `:latest`.

## 4. Что проверять перед тегом

```bash
make build && make build-cpu && cargo test --workspace
./tools/insert-test.sh                 # вставка в окно
dictactl calibrate --secs 8            # пороги уровня на живом микрофоне
docker build -f docker/Dockerfile -t dicta:local . && docker compose up -d
docker exec dicta dictactl status
```

## 5. Честность описания релиза

Не обещать «работает везде»: подтверждено только на KDE Plasma 6 + PipeWire + AMD. Остальные
адаптеры помечать как «не проверено на живом железе».
