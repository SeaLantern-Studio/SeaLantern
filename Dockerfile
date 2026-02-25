# ============================================
# SeaLantern Dockerfile - 多阶段构建
# ============================================

# ---------- 阶段 1: 构建前端 ----------
FROM docker.m.daocloud.io/node:20-alpine AS frontend-builder

WORKDIR /app

# 复制 package 文件
COPY package.json package-lock.json ./

# 安装依赖
RUN npm ci

# 复制前端源代码
COPY . .

# 构建前端 (vite)
RUN npm run build

# ---------- 阶段 2: 编译 Rust 后端 ----------
# 使用与运行镜像相同的基础镜像，避免 GLIBC 版本不兼容
FROM docker.m.daocloud.io/debian:bookworm-slim AS backend-builder

# 安装 Rust 工具链
RUN { \
    unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY; \
    sed -i 's|http://deb.debian.org/debian|http://mirrors.aliyun.com/debian|g' /etc/apt/sources.list.d/debian.sources 2>/dev/null || \
    sed -i 's|http://deb.debian.org/debian|http://mirrors.aliyun.com/debian|g' /etc/apt/sources.list 2>/dev/null || true; \
    apt-get update && apt-get install -y \
    curl \
    build-essential \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . "$HOME/.cargo/env" \
    && rustup default 1.93.1 \
    && rm -rf /var/lib/apt/lists/*; \
}

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# 安装系统依赖 (使用阿里云 HTTP 镜像源，避免证书问题)
RUN { \
    unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY; \
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
    && rm -rf /var/lib/apt/lists/*; \
}

# 复制 Cargo 文件（src-tauri 使用顶层 Cargo.lock）
COPY Cargo.toml Cargo.lock ./
COPY src-tauri/Cargo.toml ./src-tauri/

# 复制源代码（cargo fetch 需要 src 目录）
COPY . .

# 预下载依赖（利用 Docker 缓存）
RUN cd src-tauri && cargo fetch

# 构建 Release 版本 (库和二进制)
WORKDIR /app/src-tauri
RUN cargo build --release --lib --bins

# ---------- 阶段 3: 精简运行镜像 ----------
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
    && rm -rf /var/lib/apt/lists/*; \
} \
&& useradd -m -u 1000 sealantern

# 从后端构建器复制编译好的二进制程序
COPY --from=backend-builder /app/target/release/docker-entry /app/docker-entry

# 从前端构建器复制构建好的静态文件
COPY --from=frontend-builder /app/dist /app/dist

# 设置环境变量
ENV STATIC_DIR=/app/dist
ENV RUST_LOG=info

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
ENTRYPOINT ["/app/docker-entry"]
