<div align="center">
<img src="https://gitee.com/fps_z/SeaLantern/raw/master/src/assets/logo.svg" alt="logo" width="200" height="200">

# Sea Lantern (海晶灯)

A Minecraft Server Manager based on Tauri 2 + Rust + Vue 3

|[![github-stars](https://img.shields.io/github/stars/FPSZ/SeaLantern?style=flat&logo=github&label=Stars)](https://github.com/FPSZ/SeaLantern/stargazers)|[![github-forks](https://img.shields.io/github/forks/FPSZ/SeaLantern?style=flat&logo=github&label=Forks)](https://github.com/FPSZ/SeaLantern/network/members)|[![github-latest](https://img.shields.io/github/v/release/FPSZ/SeaLantern?style=flat&logo=github&label=Latest%20version)](https://github.com/FPSZ/SeaLantern/releases/latest)|
|:--|:--|:--|
|[![gitee-stars](https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/stargazers)|[![gitee-forks](https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/members)|[![gitee-latest](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgitee.com%2Fapi%2Fv5%2Frepos%2FFPS_Z%2FSeaLantern%2Freleases%2Flatest&query=%24.tag_name&label=Latest%20version&color=brightgreen&logo=gitee&style=flat)](https://gitee.com/FPS_Z/SeaLantern/releases)|

<kbd>[简体中文](README.md)</kbd> <kbd>English</kbd>

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

Download the software from [Releases](https://github.com/FPSZ/SeaLantern);

Import a server .jar, choose a Java version, then click Start. It's that simple.

## Development

You'll need Node.js 20+ and Rust 1.70+.

```bash
git clone https://github.com/fps_z/SeaLantern.git
cd SeaLantern
npm install
npm run tauri dev
```

Build release:

```bash
npm run tauri build
```

Built binaries are in `src-tauri/target/release/bundle/`.

### Code Quality Check

Before your PR, we encourage you to run the commands below to check the code's quality:

- For frontend

> ```bash
> # Code Quality Check
> npm run lint
> 
> # Fix fixable problems
> npm run lint:fix
> 
> # Format code
> npm run fmt
> 
> # Check format
> npm run fmt:check
> ```

- For backend

> ```bash
> # Check format
> cargo fmt --all -- --check
> 
> # Run Clippy check
> cargo clippy --workspace -- -D warnings
> 
> # Format code
> cargo fmt --all
> ```

CI automated checks are set up to ensure that all submitted code meets the standards.

### Tech stack

- **Frontend**: Vue 3 + TypeScript + Vite + Pinia
- **Backend**: Rust + Tauri 2
- **Style**: CSS
- **Communicate**: Tauri invoke (The frontend calls Rust functions and receives the results)

No Electron, no Node backend, no Webpack. Launch fast, size small, RAM saved.

### Project Structure

See [Project Structure](docs/Project%20Structure.md).

## Planned Features

Placeholders have been reserved for these features with existing code
skeletons—waiting for your contributions:

- Download Center - Download server cores, plugins and mods
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

### How to Contribute

1. Fork the repository
2. Create a branch and implement your changes
3. Submit a Pull Request
4. Your name will be added to the contributor wall

You don't need coding skills to contribute. Just suggest new features you want or share a UI sketch — they all count as contributions!

### Add a new function

If you are going to add a "Save Management":

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

## License

[GNU General Public License v3.0](LICENSE)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=FPSZ/SeaLantern&type=Date)](https://star-history.com/#FPSZ/SeaLantern&Date)

## Contributors

Thanks to everyone who contributed to Sea Lantern!

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/FPSZ/SeaLantern/graphs/contributors)

## Acknowledgments

Sea Lantern is an open source project under the GPLv3 license.

Minecraft is a trademark of Mojang AB.
This project is not approved or associated with Mojang or Microsoft.

"We've built the framework — the soul is up to you."
