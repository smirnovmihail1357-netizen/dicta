# syntax=docker/dockerfile:1.7
#
# dicta в контейнере. Две стадии: сборка Rust -> минимальный рантайм.
#
#   docker build -f docker/Dockerfile -t dicta:local .
#
# Важные детали, иначе «соберётся, но не будет работать»:
#   * whisper-rs с feature vulkan требует glslangValidator на этапе сборки;
#   * запись звука идёт через pw-record, поэтому в рантайме нужен pipewire;
#   * вставка текста — wl-clipboard (wl-copy/wl-paste) + /dev/uinput (монтируется при запуске);
#   * Vulkan в контейнере требует /dev/dri + mesa-vulkan-drivers; без этого поставь
#     в конфиге use_gpu = false — ggml собран с Vulkan, но возьмёт CPU.

ARG RUST_IMAGE=rust:1-bookworm

# --- сборка -----------------------------------------------------------------
FROM ${RUST_IMAGE} AS build
ARG VULKAN=1
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential pkg-config cmake clang libclang-dev \
        libvulkan-dev glslang-tools libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
# Отдельным слоем — зависимости пересобираются только при изменении манифестов.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
# Cache-маунты (--mount=type=cache) работают только с BuildKit; здесь вариант, который
# собирается и legacy-билдером, и BuildKit. При BuildKit раскомментируй маунты в
# docker/Dockerfile.buildkit.
RUN set -eux; \
    if [ "x${VULKAN}" = "x0" ]; then \
        cargo build --release --locked --workspace --no-default-features; \
    else \
        cargo build --release --locked --workspace --features dicta-core/vulkan; \
    fi; \
    cp -v target/release/dictad target/release/dictactl target/release/dicta-ui /usr/local/bin/

# --- рантайм ----------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pipewire \
        wl-clipboard \
        libvulkan1 \
        mesa-vulkan-drivers \
        fonts-dejavu-core \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/bin/dictad /usr/local/bin/dictactl /usr/local/bin/dicta-ui /usr/local/bin/
COPY docker/entrypoint.sh /usr/local/bin/dicta-entrypoint
COPY config.example.toml /usr/local/share/dicta/config.example.toml
RUN chmod +x /usr/local/bin/dicta-entrypoint

ENV DICTA_CONFIG=/config/config.toml \
    DICTA_SOCKET=/run/user/1000/dicta.sock \
    DICTA_MODEL=/models/ggml-medium-q5_0.bin \
    XDG_RUNTIME_DIR=/run/user/1000

VOLUME ["/config", "/models", "/run/user/1000"]

ENTRYPOINT ["/usr/local/bin/dicta-entrypoint"]
CMD ["--ui"]
