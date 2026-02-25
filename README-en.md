<div align="center">
	<img src="src/assets/logo.svg" alt="logo" width="200" height="200">

# Sea Lantern (海晶灯)

A Minecraft Server Manager based on Tauri 2 + Rust + Vue 3

<div style="display: flex; justify-content: center; gap: 12px; margin-bottom: 12px; flex-wrap: wrap;">
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/stargazers"><img src="https://img.shields.io/github/stars/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Stars" alt="GitHub Stars"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/network/members"><img src="https://img.shields.io/github/forks/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=Forks" alt="GitHub Forks"></a>
  <a href="https://github.com/SeaLantern-Studio/SeaLantern/releases/latest"><img src="https://img.shields.io/github/v/release/SeaLantern-Studio/SeaLantern?style=flat&logo=github&label=latest" alt="GitHub Latest"></a>
</div>

<div style="display: flex; justify-content: center; gap: 12px; flex-wrap: wrap;">
  <a href="https://gitee.com/fps_z/SeaLantern/stargazers"><img src="https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark" alt="Gitee Stars"></a>
  <a href="https://gitee.com/fps_z/SeaLantern/members"><img src="https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark" alt="Gitee Forks"></a>
</div>

<kbd>[简体中文](README.md)</kbd> <kbd>English</kbd>

Any questions? Try→[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/SeaLantern-Studio/SeaLantern)
---

</div>

![img](https://gitee.com/fps_z/markdown/raw/master/img/about2.png)

## What can it do?

- Logs and load at the control panel in real time; send commands to the server directly
- Edit server.properties graphically, without browsing directories
- Manage whitelists, bans and OPs in switches
- The server shuts down automatically when you close the app, so no saves are damaged.
- Check for and download updates in one click

## Quick Start

Download the software from [Releases](https://github.com/SeaLantern-Studio/SeaLantern/releases/latest);

Import a server .jar, choose a Java version, then click Start. It's that simple.

## Development

You'll need Node.js 20+ and Rust 1.70+.

Please also install `pnpm` and `cargo`.

**You need to first Fork the source repository, then proceed with development work in your own repository.**

If you only want to check the latest progress, you can directly fetch the source repository:

```bash
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern
```

The project's package manager was voted to switch from `npm` to `pnpm`.

Frontend and Backend:

```bash
pnpm install
pnpm run tauri dev
```

On some Linux distributions, such as Arch, running `pnpm run tauri dev` directly may not compile successfully. Please check if your dependency libraries are complete. It is recommended to use your package manager to install `Tauri` dependencies beforehand when running the above command to avoid missing dependency issues. [Click here to go to "Tauri | Prerequisites"](https://tauri.app/start/prerequisites/#linux)

Only Frontend:

```bash
pnpm dev
```

Build release:

```bash
pnpm run tauri build
```

Built binaries are in `src-tauri/target/release/bundle/`.

### Code Quality Check

Before your PR, we encourage you to run the commands below to check the code's quality:

<details><summary>For frontend</summary>

```bash
# Code Quality Check
pnpm run lint

# Fix fixable problems
pnpm run lint:fix

# Format code
pnpm run fmt

# Check format
pnpm run fmt:check
```

</details>

<details><summary>For backend</summary>

```bash
# Check format
cargo fmt --all -- --check

# Run Clippy check
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

</details>

CI automated checks are set up to ensure that all submitted code meets the standards.

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite + Pinia
- **Backend**: Rust + Tauri 2
- **Style**: CSS
- **Communicate**: Tauri invoke (The frontend calls Rust functions and receives the results)

No Electron, no Node backend, no Webpack. Launch fast, size small, RAM saved.

> We use WebView as the frontend rendering component. WebView is a built-in application in modern computer systems, with frontend and backend memory usage generally not exceeding 70MiB

### Project Structure

See [Project Structure](docs/STRUCTURE-en.md).

## Planned Features

Placeholders have been reserved for these features with existing code
skeletons—waiting for your contributions:

- Backup Management - Incremental backup and restore of save files
- Intranet Penetration - FRP integration
- Scheduled Tasks - Automatic restarts, scheduled backups, and scheduled commands
- Resource Management - Search and install plugins/mods from Modrinth & CurseForge

## Contributing

Contributions are welcome! Before you start, please read the [Contributing Guidelines](docs/CONTRIBUTING-en.md) to understand code standards and development workflows.

GUI modifications are also OK!

Colors are managed via CSS variables —
components are modular —
change any part you don't like.

Want to create a theme/skin? Go for it;
want to completely redesign the layout? That's fine!

Of course, the prerequisite for all of this is that you have sufficient reasons and abilities, and can only do it after discussing with everyone in the group, otherwise it is very likely that we will **reject the PR**

### How to Contribute

1. Fork the `dev` branch of repository
2. Create a branch and implement your changes
3. Submit a Pull Request
4. Your name will be added to the contributor wall

We have certain limitations on AI programming, namely `Vibe Coding`: only fixing, not refactoring, not making significant changes, manual review.

- Only fix: Due to the limitations of most current AI capabilities, it is unrealistic to rely entirely on AI.

- Not Refactoring: AI's contextual and abstract understanding abilities are not sufficient for AI to refactor existing content. Of course, there may be lucky ones who can still use it after refactoring, but that is just an example.

- Not much changed: **Do not let AI make unauthorized changes to any content that has a huge impact**.

- Manual review: After using AI, it is necessary to manually review whether there are any errors. If you do not know how to review, you can go to the group to find the management. Remember to politely ask questions instead of harassing the management.

Not being able to write code is also acceptable. Say what feature you want, or draw a UI sketch and send it out. As long as it is verified to be useful, it is considered a contribution.

### Add a new function

If you are going to add a "Backup Management":

#### Backend

1. Create `backup_manager.rs` under `src-tauri/src/services/`, code the logic
2. Create `backup.rs` under `src-tauri/src/commands/`, code with Tauri
3. Add `pub mod backup` in `commands/mod.rs`
4. Register the command in the `generate_handler!` macro under `lib.rs`

#### Frontend

1. Create `backup.ts` under `src/api/`, encapsulate invokes
2. Create `BackupView.vue` under `src/views/`, make the page
3. Add routes in `src/router/index.ts`
4. Add an item to the `navItems` array in `AppSidebar.vue`

The frontend and backend each have 3 files, plus one line each for the router and the sidebar.

### i18n — Internationalization Guide

Sea Lantern supports multiple languages, including Simplified Chinese, Traditional Chinese and English. See the i18n guide: [src/language/README-en.md](src/language/README-en.md)

If you want to add additional languages besides the commonly used ones, please create plugins.

## License

[GNU General Public License v3.0](LICENSE)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=SeaLantern-Studio/SeaLantern&type=Date)](https://star-history.com/#SeaLantern-Studio/SeaLantern&Date)

## Contributors

Thanks to everyone who contributed to Sea Lantern!

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)

## Acknowledgments

Sea Lantern is an open source project under the GPLv3 license.

Minecraft is a trademark of Mojang AB.
This project is not approved or associated with Mojang or Microsoft.

"We've built the framework — the soul is up to you."
