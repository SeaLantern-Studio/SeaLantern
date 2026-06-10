<div align="center">
	<img src="../../frontend/src/assets/logo.svg" alt="logo" width="200" height="200">

# Sea Lantern

A lightweight Minecraft server management tool built with Tauri 2 + Rust + Vue 3

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/stargazers"><img src="https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars" alt="GitHub Stars"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/network/members"><img src="https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks" alt="GitHub Forks"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/releases/latest"><img src="https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Latest" alt="GitHub Latest"></a>
</div>

<div style="display: flex; justify-content: center; gap: 12px; flex-wrap: wrap;">
  <a href="https://gitee.com/fps_z/SeaLantern/stargazers"><img src="https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark" alt="Gitee Stars"></a>
  <a href="https://gitee.com/fps_z/SeaLantern/members"><img src="https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark" alt="Gitee Forks"></a>
</div>

<kbd>[Simplified Chinese](../../README.md)</kbd> <kbd>English</kbd>

## Questions? Try → [![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/SeaLantern-Studio/SeaLantern)

</div>

## Quick Start

> Tip: We also have a documentation site. It is a more convenient place to browse project docs. Visit [here](https://docs.ideaflash.cn/en/intro).

Download the [stable release](https://github.com/SeaLantern-Studio/SeaLantern/releases/latest)

Download the [preview release](https://github.com/SeaLantern-Studio/SeaLantern-Preview/releases/latest)

## Development

You need `Node.js 22+` and `Rust 1.70+`.

Please also install `pnpm` and `cargo`.

**You need to fork the source repository's `beta` branch first, then do development work in your own repository.**

If you only want to check the latest progress, you can clone the upstream repository directly:

```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
git switch beta
```

The project switched its package manager from `npm` to `pnpm` after a team vote.

Frontend and backend:

```bash
pnpm --dir frontend install
pnpm --dir frontend run tauri:dev
```

On some Linux distributions such as Arch, running `pnpm --dir frontend run tauri:dev` directly may fail if required dependencies are missing. Install Tauri prerequisites with your package manager before running the command. See [Tauri prerequisites for Linux](https://tauri.app/start/prerequisites/#linux).

Frontend only:

```bash
pnpm --dir frontend run dev
```

### Code Quality Checks

Before submitting code, we recommend running these checks:

<details><summary>Frontend checks</summary>

```bash
# Code quality checks
pnpm --dir frontend run lint

# Type check and validate the production build
pnpm --dir frontend run build:check

# Auto-fix problems that can be fixed
pnpm --dir frontend run lint:fix

# Format code
pnpm --dir frontend run fmt

# Check code formatting
pnpm --dir frontend run fmt:check
```

</details>

<details><summary>Backend checks</summary>

```bash
# Check code formatting
cargo fmt --all -- --check

# Compile check
cargo check --workspace

# Run Clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

</details>

The project already has CI checks configured to ensure submitted code meets project standards.

### Commit Checks

CI validates code quality and related rules on every PR and push.

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri 2
- **Communication**: Tauri invoke
- **Docker**: itzg/minecraft-server

No Electron, no Node backend, no Webpack. Faster startup, smaller size, lower memory use.

> We use WebView for frontend rendering. WebView is built into modern operating systems, and combined frontend/backend memory usage usually stays under 70 MiB.

### Project Structure

See [Project Structure](../STRUCTURE.md).

### CLI Server Entry

This repository now provides a unified `sealantern server ...` CLI entry that supports both local Java servers and `itzg/minecraft-server` Docker-based runs.

See the [CLI server runtime guide](../cli-server-runtime-guide.md).

## Planned Features

These features already have reserved locations and code scaffolding. They are waiting for implementation:

- Backup management: incremental backup and restore for world saves
- Intranet tunneling: FRP integration
- Scheduled tasks: automatic restarts, scheduled backups, and scheduled commands
- Resource management: search and install plugins and mods from Modrinth and CurseForge

## Community

QQ group: **293748695**

## Contributing

Contributions are welcome. Before you start, read the [Contributing Guide](../CONTRIBUTING.md) for coding standards and development workflow.

The UI is included too. Colors live in CSS variables, and components are independent.
If you do not like something, change it.
Want to build a theme? Do it.
Want to replace the whole layout? That is also possible.

That said, you need solid reasoning and enough implementation ability, and major changes should be discussed with the community first. Otherwise, your PR may be rejected.

### How to Contribute

1. Fork the `beta` branch of this repository
2. Create a branch and write your code
3. Submit a pull request
4. Your name will appear on the contributor wall in the About page

You do not need to be able to code to contribute. If you have a useful feature request or a UI sketch, that still counts.

### i18n Internationalization Guide

Sea Lantern supports multiple languages, including Simplified Chinese, Traditional Chinese, and English. See the [i18n guide](../language-system.md).

If you want to add languages beyond the currently supported common ones, please do it through plugins.

## License

[GNU General Public License v3.0](../../LICENSE)

## Star History

<a href="https://www.star-history.com/#SeaLantern-Studio/SeaLantern&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=date&legend=top-left" />
 </picture>
</a>

## Contributors

Thanks to everyone who has contributed to Sea Lantern.

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)

## Acknowledgments

Sea Lantern is an open source project licensed under GPLv3.

Minecraft is a registered trademark of Mojang AB.
This project is not approved by, endorsed by, or affiliated with Mojang or Microsoft.

"We built the framework. The soul belongs to you."
