# ============================================
# SeaLantern Server Dockerfile - 多阶段构建
# 构建 sealantern-server 独立 HTTP 服务
# ============================================

# ---------- 阶段 1: 构建前端 ----------
FROM node:20-alpine AS frontend-builder

WORKDIR /app

# 安装 pnpm
RUN npm install -g pnpm@9.15.9

# 复制前端依赖文件
COPY package.json ./package.json
COPY pnpm-lock.yaml ./pnpm-lock.yaml

# 安装依赖
RUN pnpm install --frozen-lockfile

# 复制前端构建所需文件
COPY index.html ./index.html
COPY vite.config.ts ./vite.config.ts
COPY tsconfig.json ./tsconfig.json
COPY tsconfig.node.json ./tsconfig.node.json
COPY src/ ./src/

# 复制 Tauri 图标（前端构建需要引用）
COPY src-tauri/icons/ ./src-tauri/icons/

# 构建前端 (vite)
RUN npm run build

# ---------- 阶段 2: Rust 基础镜像 ----------
FROM debian:bookworm-slim AS rust-base

# 安装 Rust 工具链、cargo-chef 以及 GLib 开发依赖
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libglib2.0-dev \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . "$HOME/.cargo/env" \
    && rustup default 1.93.1 \
    && cargo install cargo-chef \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"

# ---------- 阶段 3: 准备 Rust 构建配方 ----------
FROM rust-base AS chef-preparer

WORKDIR /app

# 复制 Cargo workspace 文件（包括所有 workspace 成员）
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml ./crates/core/
COPY crates/extra/Cargo.toml ./crates/extra/
COPY crates/infra/Cargo.toml ./crates/infra/
COPY server/Cargo.toml ./server/
COPY src-tauri/Cargo.toml ./src-tauri/
COPY docker-entry/Cargo.toml ./docker-entry/

# 创建空的 src 目录用于准备配方
RUN mkdir -p crates/core/src crates/extra/src crates/infra/src server/src src-tauri/src docker-entry/src \
    && touch crates/core/src/lib.rs crates/extra/src/lib.rs crates/infra/src/lib.rs server/src/lib.rs \
    && echo "fn main() {}" > docker-entry/src/main.rs \
    && echo "#![allow(dead_code)] fn main() {}" > src-tauri/src/main.rs \
    && echo "" > src-tauri/src/lib.rs \
    && echo "fn main() {}" > src-tauri/build.rs

# 使用 cargo-chef 准备构建配方
RUN cargo chef prepare --recipe-path recipe.json

# ---------- 阶段 4: 构建 Rust 后端 ----------
FROM rust-base AS backend-builder

WORKDIR /app

# 安装系统依赖（包含 GLib）
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libglib2.0-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# 复制准备阶段的配方
COPY --from=chef-preparer /app/recipe.json recipe.json

# 使用 cargo-chef 下载并编译依赖（这一层会被缓存）
RUN cargo chef cook --release --recipe-path recipe.json

# 复制 Cargo workspace 文件
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY server/ ./server/

# 构建 sealantern-server 二进制
RUN cargo build --release -p sealantern-server

# ---------- 阶段 5: 精简运行镜像 ----------
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libglib2.0-0 \
    curl \
    wget \
    gnupg \
    apt-transport-https \
    && rm -rf /var/lib/apt/lists/*

# 添加 Temurin 官方仓库并安装 Java 8, 11, 17, 21
RUN apt-get update && apt-get install -y wget && \
    mkdir -p /etc/apt/keyrings && \
    wget -q -O - https://packages.adoptium.net/artifactory/api/gpg/key/public | gpg --dearmor -o /etc/apt/keyrings/adoptium.gpg && \
    echo "deb [signed-by=/etc/apt/keyrings/adoptium.gpg] https://packages.adoptium.net/deb bookworm main" > /etc/apt/sources.list.d/adoptium.list && \
    apt-get update && apt-get install -y \
    temurin-8-jdk \
    temurin-11-jdk \
    temurin-17-jdk \
    temurin-21-jdk \
    && rm -rf /var/lib/apt/lists/*

# 创建 Java 版本软链接
RUN mkdir -p /opt/java && \
    ln -sf /usr/lib/jvm/temurin-8-jdk-amd64 /opt/java/jdk-8 && \
    ln -sf /usr/lib/jvm/temurin-11-jdk-amd64 /opt/java/jdk-11 && \
    ln -sf /usr/lib/jvm/temurin-17-jdk-amd64 /opt/java/jdk-17 && \
    ln -sf /usr/lib/jvm/temurin-21-jdk-amd64 /opt/java/jdk-21 && \
    useradd -m -u 1000 sealantern

# 从后端构建器复制编译好的二进制程序
COPY --from=backend-builder /app/target/release/sealantern-server /app/sealantern-server

# 从前端构建器复制构建好的静态文件
COPY --from=frontend-builder /app/dist /app/dist

# 设置环境变量
ENV RUST_LOG=info
ENV STATIC_DIR=/app/dist
ENV JAVA_HOME=/opt/java/jdk-21
ENV PATH=/opt/java/jdk-21/bin:$PATH

# 创建数据目录
RUN mkdir -p /app/data && chown -R sealantern:sealantern /app

# 切换到非 root 用户
USER sealantern

# 暴露 HTTP 服务器端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 启动命令
ENTRYPOINT ["/app/sealantern-server"]