CARGO ?= cargo
BIN_DIR ?= $(HOME)/.local/bin
IMAGE ?= dicta:local

.PHONY: build build-cpu test run install uninstall docker-build docker-run compose-up compose-down fetch-model

## сборка с Vulkan (по умолчанию)
build:
	$(CARGO) build --release --locked

## сборка без Vulkan (чистый CPU, не нужен glslangValidator)
build-cpu:
	$(CARGO) build --release --locked --no-default-features

test:
	$(CARGO) test --workspace

install: build
	install -Dm755 target/release/dictad   $(BIN_DIR)/dictad
	install -Dm755 target/release/dictactl $(BIN_DIR)/dictactl
	install -Dm755 target/release/dicta-ui $(BIN_DIR)/dicta-ui
	@echo "установлено в $(BIN_DIR); не забудь systemd/install.sh и adapters/kwin/install.sh"

uninstall:
	rm -f $(BIN_DIR)/dictad $(BIN_DIR)/dictactl $(BIN_DIR)/dicta-ui

docker-build:
	docker build -f docker/Dockerfile -t $(IMAGE) .

docker-run:
	./docker/run-host.sh

compose-up:
	docker compose up -d --build

compose-down:
	docker compose down

fetch-model:
	./tools/fetch-model.sh
