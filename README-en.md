<div align="center">

<img src="src/assets/logo.svg" alt="Sea Lantern logo" width="200" height="200">

# Sea Lantern

A lightweight Minecraft server management tool

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/stargazers"><img src="https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars" alt="GitHub Stars"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/network/members"><img src="https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks" alt="GitHub Forks"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/releases/latest"><img src="https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Latest" alt="Latest GitHub Release"></a>
</div>

<kbd>[简体中文](README.md)</kbd> <kbd>English</kbd>

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/SeaLantern-Studio/SeaLantern)

</div>

## What It Does

- [x] Download Minecraft server software
- [x] Customize the server setup and launch experience
- [x] Edit server configuration through a clear and intuitive interface
      Currently limited to the vanilla `server.properties` file
- [x] Run frequently used console commands with ease
- [ ] JVM presets and a community sharing hub
- [ ] Run servers in containerized Docker environments

## Quick Start

| I want to...                                 | Go to                                                  |
| -------------------------------------------- | ------------------------------------------------------ |
| Download and install Sea Lantern             | [Download](https://docs.ideaflash.cn/en/download)      |
| Create or import a server for the first time | [Tutorial](https://docs.ideaflash.cn/en/tutorial)      |
| Not sure which server core to choose         | [Server Core](https://docs.ideaflash.cn/en/server-jar) |
| Run into issues or unexpected behavior       | [FAQ](https://docs.ideaflash.cn/en/faq)                |

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri 2 (Desktop host) + Axum (Web host)
- **Communication**: Tauri IPC/Event on Desktop, HTTP/WebSocket/SSE on Web, and host bridges for plugins
- **Docker**: `itzg/minecraft-server`

No Electron. No Node.js backend. No Webpack.

Sea Lantern starts quickly, has a small footprint, and keeps memory usage low.

The host-neutral `application` crate provides shared business orchestration. `src-tauri` and `server` are the Desktop and Web hosts, while `interface` contains only contracts genuinely shared by both. Reusable frontend business code goes through `src/api`; host-specific pages and features may evolve independently. See [Project Architecture and Code Organization](docs/architecture.md).

> The interface is rendered using the operating system's native WebView.

## Roadmap

- **Backup management** — Incremental backups and restoration for Minecraft worlds
- **Scheduled tasks** — Automatic restarts, scheduled backups, and timed command execution
- **Resource management** — Search for and install plugins and mods from Modrinth and CurseForge
- **NAT traversal** — Integrate FRP to provide a more stable and reliable connection for multiplayer servers

## For Developers

Make sure the following tools are installed before you begin:

| Dependency | Version |
| ---------- | ------- |
| Node.js    | 24 LTS  |
| Rust       | stable  |
| pnpm       | 9.15.9  |

For help setting up your development environment, see the [environment setup guide](https://docs.ideaflash.cn/en/dev/environment).

Clone the repository:

```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
```

Install dependencies and start the desktop development environment:

```bash
pnpm install
pnpm tauri dev
```

To preview the frontend only:

```bash
pnpm dev
```

To start only the HTTP/Docker backend:

```bash
pnpm docker:dev
```

Linux developers may need to install additional system dependencies required by Tauri. See the [Tauri prerequisites for Linux](https://tauri.app/start/prerequisites/#linux) for details.

### Code Quality Checks

Before submitting changes, we **recommend** running the following checks:

<details><summary>Frontend checks</summary>

```bash
# Run the linter
pnpm lint

# Run type checking and verify the production build
pnpm build:check

# Automatically fix supported linting issues
pnpm lint:fix

# Format the codebase
pnpm fmt

# Check code formatting
pnpm fmt:check
```

</details>

<details><summary>Backend checks</summary>

```bash
# Format Rust code
cargo fmt --all

# Check that the workspace compiles
cargo check --all-targets --workspace

# Run Clippy and treat warnings as errors
cargo clippy --all-targets --workspace -- -D warnings
```

</details>

The project includes a CI workflow that automatically runs code-quality checks on pushes and pull requests.

## Contributing

Contributions of all kinds are welcome, including code, documentation, translations, bug reports, feature requests, and UI concepts.

1. Fork the repository
2. Create a new development branch
3. Make your changes and complete the relevant checks
4. Open a pull request

For significant changes involving the overall UI, project architecture, or other core areas, please discuss the proposal in the community groups or GitHub Issues first. Large changes without sufficient justification may not be accepted.

## Community and Support

For help, feedback, or general discussion, you can reach us through the following channels:

- QQ Group 1: **293748695**
- QQ Group 2: **1085823754**
- Bug reports and feature requests: [GitHub Issues](https://github.com/SeaLantern-Studio/SeaLantern/issues)

## Contributors

Thank you to everyone who has contributed to Sea Lantern!

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)

## License

[GNU Affero General Public License v3.0](LICENSE).

## Acknowledgements

Minecraft is a registered trademark of Mojang AB. This project is not approved by, affiliated with, or associated with Mojang or Microsoft.

> We built the framework. You bring it to life.
