# syntax=docker/dockerfile:1
# DeepWhale 多架构 Docker 镜像（CLI 模式，非 GUI）
#
# 构建（多架构）：
#   docker buildx build --platform linux/amd64,linux/arm64 -t deepwhale:latest .
#
# 运行：
#   docker run --rm -it -e DEEPSEEK_API_KEY deepwhale --cli exec "hello"
#   docker run --rm -it --env-file .env deepwhale
#
# 注意：API 密钥必须在运行时传入（环境变量或 .env 文件）
# 不支持 GUI 模式（无 Tauri 桌面环境）

ARG RUST_VERSION=1.88

# ── Stage 1: Build ────────────────────────────────────────────────────
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-slim-bookworm AS builder
ARG TARGETPLATFORM
ARG TARGETARCH
ARG BUILDPLATFORM

ENV PKG_CONFIG_ALLOW_CROSS=1

RUN if [ "${TARGETARCH}" = "arm64" ] && [ "${BUILDPLATFORM}" != "${TARGETPLATFORM}" ]; then \
      dpkg --add-architecture arm64; \
    fi \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
      pkg-config libdbus-1-dev \
    && if [ "${TARGETARCH}" = "arm64" ] && [ "${BUILDPLATFORM}" != "${TARGETPLATFORM}" ]; then \
      apt-get install -y --no-install-recommends \
        gcc-aarch64-linux-gnu libc6-dev-arm64-cross libdbus-1-dev:arm64; \
    fi \
    && rm -rf /var/lib/apt/lists/*

RUN case "${TARGETPLATFORM}" in \
      linux/amd64)  echo x86_64-unknown-linux-gnu  > /rust-target ;; \
      linux/arm64)  echo aarch64-unknown-linux-gnu > /rust-target ;; \
      *)            echo "Unsupported platform: ${TARGETPLATFORM}" >&2; exit 1 ;; \
    esac

RUN rustup target add "$(cat /rust-target)"

WORKDIR /build
COPY . .

# Build CLI binary (headless, no Tauri GUI dependencies)
RUN --mount=type=cache,id=deepwhale-target-${TARGETARCH},target=/build/target,sharing=locked \
    --mount=type=cache,id=deepwhale-cargo-registry-${TARGETARCH},target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=deepwhale-cargo-git-${TARGETARCH},target=/usr/local/cargo/git,sharing=locked \
    cargo build --release --locked --target "$(cat /rust-target)" -p nyamuwhale \
    && mkdir -p /out \
    && cp target/$(cat /rust-target)/release/deepwhale /out/ \
    || cp target/$(cat /rust-target)/release/deepwhale.exe /out/ || true

# ── Stage 2: Runtime ──────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libdbus-1-3 \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 deepwhale \
    && useradd --create-home --shell /bin/bash --uid 1000 --gid 1000 deepwhale \
    && install -d -m 0700 -o deepwhale -g deepwhale /home/deepwhale/.deepwhale
USER deepwhale
WORKDIR /home/deepwhale

COPY --from=builder --chown=deepwhale:deepwhale /out/deepwhale /usr/local/bin/deepwhale

ENTRYPOINT ["deepwhale"]
CMD ["--cli"]
