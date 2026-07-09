<p align="left">
  <sub><a href="README.md">简体中文</a></sub>
</p>

<br />

<p align="center">
  <img src="frontend/src/assets/logo.svg" alt="Sea Lantern" width="160" height="160">
</p>

<h1 align="center">Sea Lantern</h1>

<p align="center">
  <strong>A lightweight Minecraft server manager</strong>
</p>

<p align="center">
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/stargazers"><img src="https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars" alt="GitHub Stars"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/network/members"><img src="https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks" alt="GitHub Forks"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/releases"><img src="https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Latest" alt="Latest Release"></a>
</p>

<p align="center">
  <a href="https://docs.ideaflash.cn/en/download">Download</a>
  &nbsp;·&nbsp;
  <a href="https://ideaflash.cn">Website</a>
  &nbsp;·&nbsp;
  <a href="https://deepwiki.com/SeaLantern-Studio/SeaLantern">DeepWiki</a>
</p>

<br />

## Quick Start

| I want to... | Go to |
| --- | --- |
| Download and install Sea Lantern | [Download](https://docs.ideaflash.cn/en/download) |
| Create or import a server for the first time | [Tutorial](https://docs.ideaflash.cn/en/tutorial) |
| Not sure which server core to choose | [Server Core](https://docs.ideaflash.cn/en/server-jar) |
| Run into issues or unexpected behavior | [FAQ](https://docs.ideaflash.cn/en/faq) |

## For Developers

We mainly develop on the `beta` branch. If you want to contribute, please fork the `beta` branch first, then work in your own repository.

Before development, you need:

| Dependency | Version |
| --- | --- |
| Node.js | 22.12.0+ |
| Rust | stable |
| pnpm | 11.5.3 |

If you have not set up the development environment yet, see [Environment Setup](https://docs.ideaflash.cn/en/dev/environment).

Clone the project and switch to the `beta` branch:
```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
git switch beta
```

If you want to understand the repository structure, frontend/backend boundaries, and main modules, see [Project Structure](https://docs.ideaflash.cn/en/structure), also [Front Docs](frontend/README.md) and [Backend Docs](backend/README.md) with your translator.

Install dependencies and start the desktop development environment:
```bash
pnpm install
pnpm tauri:dev
```

Preview frontend pages only:
```bash
pnpm dev
```

Start the HTTP / Docker backend only:
```bash
pnpm dev:http
```

If you develop on Linux, you may need to install Tauri system dependencies first. See [Tauri Linux Prerequisites](https://tauri.app/start/prerequisites/#linux).

### Code Checks

Before submitting, please run the code checks:
```bash
pnpm lint
pnpm build:check
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

For more complete development instructions, see the [Contributing Guide](https://docs.ideaflash.cn/en/dev/CONTRIBUTING).

## About the Software

Sea Lantern uses:

- **Framework**: Tauri 2
- **Frontend**: Vue 3
- **Backend**: Rust
- **Container**: itzg/minecraft-server

No Electron, no Node backend, no Webpack.

Sea Lantern uses the system WebView to render its interface. It is lightweight, responsive, highly customizable, and better suited as a local desktop tool.

## Community and Feedback

If you run into issues or want to join the discussion, you can contact us through:

- QQ Group 1: **293748695**
- QQ Group 2: **1085823754**
- Issue Tracker: [GitHub Issues](https://github.com/SeaLantern-Studio/SeaLantern/issues)

Bug reports, suggestions, and usage feedback are all welcome.

## Contributing

We welcome all kinds of contributions: code, documentation, translation, bug reports, feature suggestions, or even UI sketches.

Basic workflow:

1. Fork the `beta` branch
2. Create your own development branch
3. Make your changes and pass the basic checks
4. Submit a Pull Request

Before you start, please read the [Contributing Guide](https://docs.ideaflash.cn/en/dev/CONTRIBUTING). If you want to make major changes to features, architecture, interaction, or UI, please discuss it in an Issue or our community group first to avoid working in the wrong direction.

## AI Policy

We do not oppose the use of AI-assisted development, but contributors must take responsibility for their own submissions.

As the Linux kernel AI policy says:

> *Taking full responsibility for the contribution.*

Before submitting an Issue or PR, please make sure you understand your changes and have actually run and tested the final result.

We do not accept:

- Large-scale refactors without prior discussion
- Issues / PRs mass-generated purely by AI
- Bug reports without reproduction steps or verification results
- AI-generated code that the contributor cannot explain or maintain
- Pasting raw AI output and asking maintainers to judge whether it is correct

For major changes to architecture, interaction, directory structure, or UI, please discuss them in an Issue or community group first.

Issues / PRs that clearly lack human review, or are low-quality AI-generated content known as AI Slop, will be closed directly.

## License

[GNU General Public License v3.0](LICENSE)

## Contributors

Thanks to everyone who has contributed to Sea Lantern!

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)

## Acknowledgements

Sea Lantern is an open-source project licensed under GPLv3.

Minecraft is a registered trademark of Mojang AB. This project is not approved by or associated with Mojang or Microsoft.

> We built the skeleton. The soul is yours.
