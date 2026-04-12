# ============================================
# SeaLantern Dockerfile - 多阶段构建 (优化版)
# 优化点：
# 1. 提取 rust-base 阶段，避免重复安装 Rust 环境
# 2. 精准 COPY - 仅通过 Dockerfile 指令隔离前端和 Rust 构建
# 3. Cargo 国内源配置移到构建阶段
# ============================================

# ---------- 阶段 1: 构建前端 ----------
FROM docker.m.daocloud.io/node:20-alpine AS frontend-builder

WORKDIR /app

# 安装 pnpm
RUN npm install -g pnpm@9.15.9

# 复制前端依赖文件
COPY package.json pnpm-lock.yaml ./

# 安装依赖
RUN pnpm install --frozen-lockfile

# 复制前端构建所需文件（精准 COPY，只复制必要的文件）
COPY index.html vite.config.ts tsconfig.json tsconfig.node.json ./
COPY src/ ./src/

# 复制 Tauri 图标（前端构建需要引用）
COPY src-tauri/icons/ ./src-tauri/icons/

# 构建前端 (vite)
RUN npm run build

# ---------- 阶段 2: Rust 基础镜像 ----------
FROM docker.m.daocloud.io/debian:bookworm-slim AS rust-base

# 安装 Rust 工具链和 cargo-chef
RUN { \
    apt-get update && apt-get install -y \
    curl \
    build-essential \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . "$HOME/.cargo/env" \
    && rustup default 1.93.1 \
    && cargo install cargo-chef \
    && rm -rf /var/lib/apt/lists/*; \
}

# 注：如需国内镜像，可在此配置 Cargo 源
# 当前使用默认 crates.io

ENV PATH="/root/.cargo/bin:${PATH}"

# ---------- 阶段 3: 准备 Rust 构建配方 ----------
FROM rust-base AS chef-preparer

WORKDIR /app

# 只复制 Rust 项目相关文件（精准 COPY，绝不复制前端文件）
COPY Cargo.toml Cargo.lock ./
COPY src-tauri/Cargo.toml ./src-tauri/
COPY src-tauri/build.rs ./src-tauri/
COPY src-tauri/src/ ./src-tauri/src/
COPY src-tauri/icons/ ./src-tauri/icons/
COPY src-tauri/tauri.conf.json ./src-tauri/
COPY src-tauri/locales/ ./src-tauri/locales/
COPY docker-entry/Cargo.toml ./docker-entry/
COPY docker-entry/src/ ./docker-entry/src/

# 使用 cargo-chef 准备构建配方
RUN cargo chef prepare --recipe-path recipe.json

# ---------- 阶段 4: 构建 Rust 后端 ----------
FROM rust-base AS backend-builder

WORKDIR /app

# 安装系统依赖（包括 git，因为 Cargo 配置使用了 git-fetch-with-cli）
RUN { \
    sed -i 's|http://deb.debian.org/debian|http://mirrors.aliyun.com/debian|g' /etc/apt/sources.list.d/debian.sources 2>/dev/null || \
    sed -i 's|http://deb.debian.org/debian|http://mirrors.aliyun.com/debian|g' /etc/apt/sources.list 2>/dev/null || true; \
    apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libxdo-dev \
    git \
    && rm -rf /var/lib/apt/lists/*; \
}

# 复制准备阶段的配方
COPY --from=chef-preparer /app/recipe.json recipe.json

# 使用 cargo-chef 下载并编译依赖（这一层会被缓存）
RUN cargo chef cook --release --recipe-path recipe.json

# 只复制 Rust 源代码（精准 COPY，绝不复制前端文件）
COPY Cargo.toml Cargo.lock ./
COPY src-tauri/Cargo.toml ./src-tauri/
COPY src-tauri/build.rs ./src-tauri/
COPY src-tauri/src/ ./src-tauri/src/
COPY src-tauri/icons/ ./src-tauri/icons/
COPY src-tauri/tauri.conf.json ./src-tauri/
COPY src-tauri/locales/ ./src-tauri/locales/
COPY docker-entry/Cargo.toml ./docker-entry/
COPY docker-entry/src/ ./docker-entry/src/

# 复制 tauri.conf.json 中 bundle.resources 引用的文件
COPY LICENSE NOTICE ./

# 构建 docker-entry 二进制
RUN cargo build --release -p docker-entry

# ---------- 阶段 5: 精简运行镜像 ----------
FROM docker.m.daocloud.io/debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖 (使用阿里云 HTTP 镜像源，避免证书问题)
RUN { \
    unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY; \
    sed -i 's|http://deb.debian.org/debian|http://mirrors.aliyun.com/debian|g' /etc/apt/sources.list.d/debian.sources 2>/dev/null || \
    sed -i 's|http://deb.debian.org/debian|http://mirrors.aliyun.com/debian|g' /etc/apt/sources.list 2>/dev/null || true; \
    apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*; \
}

# 添加Temurin官方仓库并安装Java 8, 11, 17, 21
RUN apt-get update && apt-get install -y wget apt-transport-https gnupg && \
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
COPY --from=backend-builder /app/target/release/docker-entry /app/docker-entry

# 从前端构建器复制构建好的静态文件
COPY --from=frontend-builder /app/dist /app/dist

# 设置环境变量
ENV STATIC_DIR=/app/dist
ENV RUST_LOG=info
ENV JAVA_HOME=/opt/java/jdk-21
ENV PATH=/opt/java/jdk-21/bin:$PATH

# 创建数据目录和上传目录
RUN mkdir -p /app/data /app/uploads && chown -R sealantern:sealantern /app

# 切换到非 root 用户
USER sealantern

# 暴露 HTTP 服务器端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 启动命令
ENTRYPOINT ["/app/docker-entry"]
