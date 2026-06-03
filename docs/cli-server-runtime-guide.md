# Sea Lantern `server` CLI 使用指南

本文档面向当前仓库已经实现的统一 `server` 子命令，覆盖本地 Java 服务端与 `itzg/minecraft-server` 容器服务端两条主线。

## 适用范围

- Java 版 Minecraft 服务端
- 本地 Java 启动模式
- Docker `itzg/minecraft-server` 运行模式
- `--web` / `--cli` / 同时启用

## 命令总览

统一入口：

```powershell
sealantern.exe server [名称] [选项]
```

统一管理入口：

```powershell
sealantern.exe server list
sealantern.exe server inspect paper-docker
sealantern.exe server status paper-docker
sealantern.exe server logs paper-docker --lines 50
sealantern.exe server logs paper-docker --follow --interval 1000
sealantern.exe server send paper-docker say server is live
sealantern.exe server start paper-docker
sealantern.exe server stop paper-docker
sealantern.exe server restart paper-docker
```

对于已经存在的服务器记录，当前也支持把统一入口直接当成管理/attach 入口：

```powershell
sealantern.exe server paper-docker
sealantern.exe server paper-docker --web 8896 --cli
sealantern.exe server cache-server --cli
```

当第一个位置参数能解析到现有服务器时：

- 如果不再带其他参数，Sea Lantern 会默认按 `server start <target> --cli` 处理
- 如果后续只包含 `--web` / `--cli` 这类 transport 参数，则会优先按“已有服务器管理启动 + attach transport”处理，而不是误判成 create flow

Docker 部署自检入口：

```powershell
sealantern.exe docker doctor
sealantern.exe docker image-check itzg/minecraft-server:latest
sealantern.exe docker pull itzg/minecraft-server:latest
```

Compose 导出入口：

```powershell
sealantern.exe compose generate paper-docker
sealantern.exe compose generate paper-docker --output E:/deploy/paper-compose.yaml
sealantern.exe compose generate paper-docker --full-stack --sealantern-data /app/data/servers --http-port 3000
```

如果未显式指定 `--web` 或 `--cli`，默认会进入 `--cli`。

如果你只想先落服务器记录、不触发首次启动，可使用：

```powershell
sealantern.exe server paper-docker --runtime docker --mc 1.21.1 --core paper --image itzg/minecraft-server --create-only
```

### 典型本地命令

```powershell
sealantern.exe server fabric-1.20.1 --mc 1.20.1 --core fabric --jar E:/srv/server.jar --java C:/Java/bin/java.exe -p:25565 -min:2G -max:4G -web:8000
```

### 典型 Docker 命令

```powershell
sealantern.exe server paper-docker --runtime docker --mc 1.21.1 --core paper --image itzg/minecraft-server --image-tag latest --web 8000 --cli
```

### 传递 itzg 高级环境变量

```powershell
sealantern.exe server paper-docker --runtime docker --mc 1.21.1 --core paper --image itzg/minecraft-server --env STOP_DURATION=180 --env DISABLE_HEALTHCHECK=true --env UID=1000 --create-only
```

### 追加 Docker 挂载

```powershell
sealantern.exe server paper-docker --runtime docker --mc 1.21.1 --core paper --image itzg/minecraft-server --mount E:/plugins:/data/plugins:ro --mount E:/mods:/data/mods --cli
```

### 追加 Docker 端口映射

```powershell
sealantern.exe server paper-docker --runtime docker --mc 1.21.1 --core paper --image itzg/minecraft-server --publish 24454:24454/udp --publish 8123:8123 --cli
```

### 接管现有目录

```powershell
sealantern.exe server cache-server --folder E:/servers/cache --cli
sealantern.exe server --folder E:/servers/cache --cli
```

### 使用自定义启动命令

```powershell
sealantern.exe server custom-local --mc 1.20.1 --core fabric --entry "java -Xmx4G -Xms4G -jar server.jar nogui" --cli
```

## 参数说明

### 基本参数

- `--name`, `--n <server-name>`
  显式指定服务器名。省略时，会回退到位置参数、`--tag`，或 `--folder` 的目录名。

- `--folder`, `--f`, `--fd <server-folder-location>`
  在 local 模式下会先判断这是不是现有服务端目录：
  如果能识别到启动脚本或服务端 JAR，就按“接管已有目录”处理；
  如果识别不到，则把它当作新建服务器的目标工作目录，并要求本地 `--jar` / 非 custom `--entry` 落在该目录根下。
  在 docker 模式下，如果未显式传 `--data-dir`，也会把它当作容器数据目录挂载源，并尽量从目录名推断核心类型和 Minecraft 版本。

- `--runtime`, `--r <local|docker|auto>`
  默认 `auto`。当存在 `--image`、`--image-tag`、`--data-dir`、`--container-name`、`--docker-backend`、`--command-mode` 等 Docker 提示时，会自动切到 Docker。

- `--web[:port]` / `--web <port>`
  启用 Web 控制台，默认端口 `8888`。

- `--cli`
  启用当前终端的交互 CLI 会话。

- `-p[:port]` / `-p <port>` / `--port[=port]`
  Minecraft 游戏端口，默认 `25565`。

- `--jar <jar_file_path>`
  本地模式下指定 JAR 或启动文件路径。支持相对路径。

- `--java`, `--j <java_path>`
  显式指定 Java 可执行文件路径。
  也支持直接写 `%env:JAVA_HOME%` 或 `%env:Path%`，让 Sea Lantern 按对应环境变量解析 Java。

- `--J`
  仅从环境变量查找 Java。优先 `JAVA_HOME`，找不到再查 `PATH`。

- `--tag`, `--t <name>`
  服务器标记名，可作为名称兜底。

- `--alias <name>`
  添加别名，可重复。

- `--min <2G|512M>` / `--max <4G|2048M>`
  控制内存上下界。

- `--entry <script-or-command>`
  指定启动脚本路径，或完整启动命令。
  如果是现有文件路径，会自动按 `bat/sh/ps1/jar` 处理。
  如果不是路径，则按 `custom` 启动命令处理。
  对于常见的 `java -jar ...` 自定义命令，Sea Lantern 会尽量提取其中的 JAR 路径，用来补全服务器工作目录与元数据推断。
  如果 `--entry` 指向现有脚本文件，Sea Lantern 也会尝试从脚本所在目录及其旁边的服务端 JAR 推断 `--core` 与 `--mc`。
  本地 create 场景下的工作目录优先级为：现有脚本所在目录，其次显式 `--jar` 所在目录，最后才是从自定义命令中提取出的路径提示目录。

- `--startup <jar|bat|sh|ps1|starter|custom>`
  显式指定本地启动方式。
  如果显式指定了 `bat/sh/ps1/jar`，则 `--entry` 需要是可解析的对应文件路径；
  如果显式指定 `custom`，则 `--entry` 会被当作自定义命令文本保留，即使它恰好指向现有脚本文件。

### Docker 扩展参数

- `--image <镜像>`
  默认 `itzg/minecraft-server`。

- `--image-tag <标签>`
  默认 `latest`。

- `--data-dir <目录>`
  显式指定 Docker 数据目录挂载源。

- `--container-name <容器名>`
  默认 `sealantern-<sanitized-name>`。

- `--docker-backend <cli|engine_api>`
  当前只有 `cli` 路径具备实际运行能力；`engine_api` 仍是预留位。

- `--command-mode <rcon|docker_stdio>`
  默认 `rcon`，也推荐优先使用 `rcon`。
  `docker_stdio` 依赖镜像内存在 `mc-send-to-console`，并且对 `itzg/minecraft-server` 语义来说还需要 `CREATE_CONSOLE_IN_PIPE=true`。当前 CLI 在 create flow 下会自动注入该变量。

- `--env <KEY=VALUE>`
  追加 Docker 环境变量，可重复。
  当前建议优先用它传 `STOP_DURATION`、`SETUP_ONLY`、`DISABLE_HEALTHCHECK`、`UID`、`GID` 等 itzg 原生参数。
  如果当前仍使用 `--command-mode rcon`，那么单独传 `RCON_PASSWORD_FILE` 还不够，因为 Sea Lantern 需要知道自己该用什么密码去连接容器；此时请同时提供 `RCON_PASSWORD`，或者改用 `--command-mode docker_stdio`。

- `--mount <source:target[:ro|rw]>`
  追加 Docker 额外挂载，可重复。
  例如：`--mount E:/plugins:/data/plugins:ro`

- `--publish <host:container[/proto]>`
  追加 Docker 额外端口映射，可重复。
  默认协议为 `tcp`，也支持显式写 `udp`。
  当前 create flow 会拒绝与 Minecraft 游戏端口、Web 端口、Sea Lantern 自动保留的 RCON 宿主端口冲突的 `--publish` 配置。

## 端口行为

### Web 端口

- 默认 `8888`
- 如果只有 Web 端口被占用，会自动向后顺延
- 一直顺延到可用端口，直到 `65535`
- 如果整个区间耗尽，会直接报错并退出

### Minecraft 端口

- 默认 `25565`
- 如果被占用，会逐次询问是否切换到下一个端口
- 确认输入支持：`y`、`Y`、`yes`、`是`、`o`、直接回车
- 否定输入支持：`n`、`N`
- 如果 Web 与 Minecraft 初始端口都冲突，会先询问 Web 端口，再处理 Minecraft 端口

## Web / CLI 运行方式

### 只开 CLI

```powershell
sealantern.exe server fabric-1.20.1 --mc 1.20.1 --core fabric --jar E:/srv/server.jar --java C:/Java/bin/java.exe --cli
```

### 只开 Web

```powershell
sealantern.exe server fabric-1.20.1 --mc 1.20.1 --core fabric --jar E:/srv/server.jar --java C:/Java/bin/java.exe --web:8000
```

### 同时开 Web 和 CLI

```powershell
sealantern.exe server fabric-1.20.1 --mc 1.20.1 --core fabric --jar E:/srv/server.jar --java C:/Java/bin/java.exe --web:8000 --cli
```

### Web-only 的行为

即使只启用了 `--web`，命令仍会创建并启动服务器，而不是只开一个空前端。

在桌面环境中，Sea Lantern 会尽量自动打开浏览器。

如果当前是 Headless / 容器环境，则不会尝试自动打开浏览器，只会打印可手动访问的 URL。

### 对已有服务器附加 Web / CLI

除了 create flow 之外，当前也支持对已有服务器记录追加 transport：

```powershell
sealantern.exe server start paper-docker --web 8896 --cli
sealantern.exe server paper-docker --web 8896 --cli
sealantern.exe server cache-server --cli
```

当前行为是：

- 如果目标服务器已在运行态，会跳过重复启动，只附加 Web / CLI
- 如果目标服务器尚未启动，会先启动，再附加 Web / CLI
- 对于已运行中的 existing server，会复用记录里的 Minecraft 端口，只为 Web 额外准备端口，不会把服务器自己占用的游戏端口误判为冲突

### 只创建记录，不首次启动

```powershell
sealantern.exe server paper-docker --runtime docker --mc 1.21.1 --core paper --image itzg/minecraft-server --create-only
```

这个模式适合：

- 离线预配或先建档后补资源
- Docker 镜像尚未拉取完成，但希望先落服务器记录
- smoke / 脚本场景中只验证 create/attach 与持久化，不触发首次启动

`--create-only` 会与 `--detach`、`--web`、`--cli` 互斥。

## Docker 运行建议

### 推荐默认值

- 镜像：`itzg/minecraft-server`
- `--command-mode rcon`
- 让 Sea Lantern 自动注入 `ENABLE_RCON`、`RCON_PORT`、`RCON_PASSWORD`

### 为什么推荐 `rcon`

- 命令通道更稳定
- 不依赖持久 attach 会话
- 更适合 Headless / 容器内运行 Sea Lantern
- 与 itzg 官方默认能力一致

### `docker_stdio` 何时适合

仅当你明确知道镜像内包含 `mc-send-to-console` 时才建议使用。对于 `itzg/minecraft-server`，还要求容器以 `CREATE_CONSOLE_IN_PIPE=true` 创建，且在非 root `UID` 场景下需要按对应 `UID` 执行 `docker exec`。

如果镜像内没有这个脚本，现在会返回更明确的错误：

- 当前 Docker 镜像内未提供 `mc-send-to-console`
- `docker_stdio` command mode 不可用
- 请确认使用 `itzg/minecraft-server`，或改用 `--command-mode rcon`

如果镜像是 `itzg/minecraft-server`，但仍然无法通过 `docker_stdio` 发命令，现在还会区分这几类典型错误：

- 容器未按 `CREATE_CONSOLE_IN_PIPE=true` 创建
- 容器内的 named pipe 尚未就绪或缺失
- 容器要求按 `UID` 执行 `mc-send-to-console`

这几种情况通常意味着容器由旧配置创建、镜像入口语义不兼容、或容器尚未完成启动。此时优先建议查看 `status`/`logs`，必要时重建容器，或者直接改回 `--command-mode rcon`。

## 容器内运行 Sea Lantern 的注意事项

### CLI Web 绑定

在桌面、 Headless 和容器环境中，CLI Web 默认都只绑定到本地回环地址，例如 `127.0.0.1`。只有显式设置 `SEALANTERN_HTTP_BIND` 或兼容变量 `SEALANTERN_WEB_BIND` 时，才会监听外部地址。若当前环境不会自动打开浏览器，CLI 只会打印可手动访问的 URL。

如果你需要显式指定绑定地址，可设置：

```powershell
$env:SEALANTERN_HTTP_BIND = "192.168.1.10:3000"
```

### Docker RCON 宿主地址

当 Sea Lantern 本身跑在容器里时，Docker RCON 默认宿主地址会更偏向容器化环境，而不是写死 `127.0.0.1`。

如果你的部署环境需要手动指定，可设置：

```powershell
$env:SEALANTERN_DOCKER_RCON_HOST = "host.docker.internal"
```

或在 Linux / 编排环境中显式指定为你的实际宿主地址。

### Docker 数据目录宿主路径映射

如果 Sea Lantern 自己运行在容器里，那么 CLI 里默认生成的服务器目录很可能是容器内路径，例如：

- 容器内可见路径：`/app/data/servers/paper-docker`
- 宿主机真实路径：`/srv/sealantern/servers/paper-docker`

Docker 在创建 Minecraft 容器时需要的是“宿主机真实路径”，而不是 Sea Lantern 容器内部看到的路径。

因此当前实现支持用以下环境变量做显式映射：

```powershell
SEALANTERN_SERVERS_CONTAINER_ROOT=/app/data/servers
SEALANTERN_SERVERS_HOST_ROOT=/srv/sealantern/servers
```

启用后，Sea Lantern 在容器里生成的默认 Docker 数据目录会自动翻译为宿主机可挂载路径。

如果你已经通过 `--data-dir` 显式传入宿主机路径，则不会依赖默认映射生成逻辑。

如果 Sea Lantern 运行在容器化/Headless 环境，并且你既没有传 `--data-dir`，也没有配置上述映射变量，当前 CLI 会直接拒绝创建 Docker 服务器记录，避免把容器内路径误当成宿主机挂载路径。

### 相关环境变量

- `SEALANTERN_HEADLESS_HTTP=1`
  强制按 Headless Web 入口逻辑运行 CLI Web，但不会改变默认 loopback 绑定。

- `SEALANTERN_HTTP_BIND=0.0.0.0:3000`
  显式让 CLI Web 监听所有网卡；默认仍是 `127.0.0.1:3000`，仅在确实需要容器外/远端访问时启用。

- `SEALANTERN_WEB_BIND=0.0.0.0:3000`
  兼容旧变量名，效果与 `SEALANTERN_HTTP_BIND` 相同；不设置时不会改掉默认的 `127.0.0.1:3000`。

- `SEALANTERN_DOCKER_RCON_HOST=<host>`
  覆盖 Docker RCON 默认宿主地址。

- `SEALANTERN_SERVERS_CONTAINER_ROOT=<path>`
  Sea Lantern 容器内部看到的 `servers` 根目录。

- `SEALANTERN_SERVERS_HOST_ROOT=<path>`
  Docker 真正应使用的宿主机 `servers` 根目录。

- `STATIC_DIR=<path>`
  指定 Web 静态资源目录。

## `docker doctor`

如果你准备使用 `docker_itzg` 路线，尤其是 Sea Lantern 自己也运行在容器里时，建议先执行：

```powershell
sealantern.exe docker doctor
```

当前会检查的重点包括：

- Docker CLI 是否存在于 `PATH`
- `docker info` 是否可正常执行
- 当前是否按容器/Headless 逻辑运行
- CLI Web 的实际绑定地址
- Docker RCON 实际会使用的宿主地址
- `SEALANTERN_SERVERS_CONTAINER_ROOT` / `SEALANTERN_SERVERS_HOST_ROOT` 是否形成有效映射

如果出现 `FAIL`，通常代表当前 Docker 运行链路仍有阻断项，不建议直接创建 `docker_itzg` 服务器记录。

如果你想先判断镜像引用当前是“本地已缓存”“远端可解析”还是“当前网络下无法确认但仍可先 create-only / compose”，也可以执行：

```powershell
sealantern.exe docker image-check itzg/minecraft-server:latest
```

如果只是镜像尚未缓存，或当前希望先把拉取步骤前置，也可以直接执行：

```powershell
sealantern.exe docker pull itzg/minecraft-server:latest
```

这条命令会直接调用本机 Docker CLI 拉取镜像，便于后续重新执行 `sealantern.exe server ... --runtime docker` 或 `sealantern.exe server start <id>`。

## `compose generate`

如果你已经有一条 `docker_itzg` 服务器记录，可以直接导出它对应的 Compose 片段：

```powershell
sealantern.exe compose generate paper-docker
```

也可以直接写入文件：

```powershell
sealantern.exe compose generate paper-docker --output E:/deploy/paper-compose.yaml
```

当前导出内容会覆盖：

- `image` / `image_tag`
- `container_name`
- `published_game_port` 与 `extra_ports`
- `/data` 挂载和额外 `volume_mounts`
- `env`，并在缺失时自动补 `TYPE` / `VERSION`

这一能力当前只支持 `docker_itzg` 服务器记录；本地 `local` 服务器不会生成 Compose。

如果你希望直接导出“Sea Lantern 自身容器 + Minecraft 容器”的完整部署模板，可以加上：

```powershell
sealantern.exe compose generate paper-docker --full-stack --sealantern-data /app/data/servers --http-port 3000
```

`--full-stack` 当前会额外生成：

- `sealantern` 服务
- `SEALANTERN_HEADLESS_HTTP=1`
- `SEALANTERN_HTTP_BIND=127.0.0.1:3000`，保持默认仅 loopback；如需容器外访问，请再显式改成对外地址
- `SEALANTERN_SERVERS_CONTAINER_ROOT` / `SEALANTERN_SERVERS_HOST_ROOT`
- `docker.sock` 挂载
- Sea Lantern HTTP 端口映射

可选参数：

- `--sealantern-image <image>`
  覆盖 Sea Lantern 容器镜像，默认 `ghcr.io/sealantern-studio/sealantern:latest`

- `--http-port <port>`
  覆盖 Sea Lantern HTTP 暴露端口，默认 `3000`

- `--static-dir <dir>`
  显式写入 `STATIC_DIR`

- `--sealantern-data <dir>`
  指定 Sea Lantern 容器内可见的 servers 根目录，用于生成 `SEALANTERN_SERVERS_CONTAINER_ROOT`

- `--docker-socket <mount>`
  覆盖 `docker.sock` 挂载，默认 `/var/run/docker.sock:/var/run/docker.sock`

## Docker 状态说明

对于 `itzg/minecraft-server`，Sea Lantern 当前会同时参考：

- 容器运行状态
- 容器 healthcheck 状态
- 日志流

因此在容器模式下：

- `healthy` 会优先映射为 `Running`
- `starting` 会映射为 `Starting`
- `unhealthy` 会映射为 `Error`

这比单纯依赖某一条特定日志更稳。

## CLI 会话内命令

- `/status` 查看当前状态
- `/logs` 查看最近日志
- `/stop` 请求停服
- `/exit` 或 `/quit` 退出 CLI 会话

## `server` 管理子命令

除了 `sealantern.exe server <name> [options]` 这种“创建/接管并拉起 transport”的入口外，当前也支持把 `server` 当作统一管理入口使用。

- `sealantern.exe server list`
  列出全部服务器记录，并显示 runtime、状态、端口。

- `sealantern.exe server inspect <id|name|alias>`
  查看完整服务器详情，包括 runtime 配置。对于 Docker 服务器，会隐藏 RCON 密码正文，只显示已脱敏长度。

- `sealantern.exe server status <id|name|alias>`
  查看当前状态、PID、uptime 和错误信息。

- `sealantern.exe server logs <id|name|alias> [--lines 50]`
  查看最近日志。

- `sealantern.exe server logs <id|name|alias> --follow [--interval 1000]`
  持续跟踪新日志。当前实现会轮询日志库增量，按 `Ctrl+C` 结束。

- `sealantern.exe server send <id|name|alias> <command...>`
  直接发送控制台命令。

- `sealantern.exe server start <id|name|alias>`
  启动已有服务器。

  当前也支持附加 transport：
  - `sealantern.exe server start <id|name|alias> --cli`
  - `sealantern.exe server start <id|name|alias> --web 8896`
  - `sealantern.exe server start <id|name|alias> --web 8896 --cli`

  同时也支持 implicit 统一入口写法：
  - `sealantern.exe server <id|name|alias> --cli`
  - `sealantern.exe server <id|name|alias> --web 8896`
  - `sealantern.exe server <id|name|alias> --web 8896 --cli`

  如果服务器已在运行态，当前会跳过重复启动，只附加 transport。

- `sealantern.exe server stop <id|name|alias>`
  请求优雅停服。

- `sealantern.exe server restart <id|name|alias>`
  先请求停服，再等待服务器进入 `stopped/error`，随后重新启动。

## 当前边界

- 当前主要支持 Java 版服务端
- Docker 首期主要面向 `itzg/minecraft-server`
- `engine_api` 仍未落地为真实运行路径
- `gui` 没有纳入当前 CLI 目标范围
- 更复杂的编排能力（如 Compose 生成、Kubernetes）还不在当前实现范围内

## 排障建议

### Java 找不到

优先顺序：

1. 显式传 `--java`
2. 用 `--J` 只走环境变量
3. 不带 `--J` 时允许 Sea Lantern 回退到默认 Java 路径、`PATH`、全局扫描

### Docker 找不到或未启动

Docker 模式会先检查：

- `docker` / `docker.exe` 是否在 `PATH`
- `docker info` 是否成功

如果失败，会直接中止，而不是创建一条不可运行的服务器记录。

### `docker_stdio` 发送命令失败

如果报错提到 `mc-send-to-console` 不存在，优先改用：

```powershell
--command-mode rcon
```

### 容器长时间处于 `Starting`

当前会优先参考 itzg 的 healthcheck 状态。如果仍长时间停留在 `Starting`，建议检查：

- 镜像是否是 `itzg/minecraft-server`
- 版本与 `TYPE` 是否匹配
- 内存是否足够
- `docker logs <container>` 与 Sea Lantern 日志中是否存在安装器或服务端启动失败信息
