<div align="center">
<img src="https://gitee.com/fps_z/SeaLantern/raw/master/src/assets/logo.svg" alt="logo" width="200" height="200">

# Sea Lantern(海晶灯)

Minecraft Server Manager · Tauri 2 + Rust + Vue 3

[![star](https://gitee.com/fps_z/SeaLantern/badge/star.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/stargazers)[![fork](https://gitee.com/fps_z/SeaLantern/badge/fork.svg?theme=dark)](https://gitee.com/fps_z/SeaLantern/members)
[![GitHub Repo stars](https://img.shields.io/github/stars/FPSZ/SeaLantern?style=flat&logo=github&label=stars)](https://github.com/FPSZ/SeaLantern)[![GitHub forks](https://img.shields.io/github/forks/FPSZ/SeaLantern?style=flat&logo=github&label=forks)](https://github.com/FPSZ/SeaLantern/network/members)
[![最新版本](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgitee.com%2Fapi%2Fv5%2Frepos%2FFPS_Z%2FSeaLantern%2Freleases%2Flatest&query=%24.tag_name&label=latest_version&color=brightgreen&logo=gitee&style=flat)](https://gitee.com/FPS_Z/SeaLantern/releases)[![GitHub release](https://img.shields.io/github/v/release/FPSZ/SeaLantern?style=flat&logo=github&label=latest)](https://github.com/FPSZ/SeaLantern/releases)
====

</div>

> 您正在浏览英文版的readme，点击[此处](README.md)前往简体中文版

> You are browsing the English version of the readme. Click [here](README.md) to go to the Simplified Chinese version

![img](https://gitee.com/fps_z/markdown/raw/master/img/about2.png)

## What can it do?

Import a server JAR file,choose a Java version,then click START!It's that simple.

- See the load at control panel in real-time,enter the command directly.
- server.properties GUI Editor,without change it manually.
- Whitelist,Ban,OP Manage easily.
- It will shut server down when you close the software which won't let your saves be damaged.
- Check update,update with one click

## Quick Start

- If you are a user,download the software from RELEASE

- If you are a developer,you need to download Node.js 20+ and Rust 1.70+.

```bash
git clone https://github.com/fps_z/SeaLantern.git
cd SeaLantern
npm install
npm run tauri dev
```

Build release：

```bash
npm run tauri build
```

The built things are in `src-tauri/target/release/bundle/`.

### Code Quality Check

Before you PR,we recommend you run commands below to check the code's quality：

Frontend Check：

```bash
# Code Quality Check
npm run lint

# Fix the fixable problem automatically
npm run lint:fix

# Initialize code
npm run fmt

# check format(No file changing)
npm run fmt:check
```

Backend Check：

```bash
# Check Code Format
cargo fmt --all -- --check

# Run Clippy check
cargo clippy --workspace -- -D warnings

# Format code automatically
cargo fmt --all
```

The project is set up with CI automated checks to ensure that all submitted code meets the standards.

## Tech stack

- **Frontend**: Vue 3 + TypeScript + Vite + Pinia
- **Backend**: Rust + Tauri 2
- **Style**: Only CSS
- **Communicate**: Tauri invoke(Call Rust functions from the frontend and directly use the return value)

No Electron,No Node Backend,No Webpack.Quick launch,small size,save RAM.

## Project Structure
 - See [Project Structure](docs\Project Structure.md)
## Planned Features

Placeholders have been reserved for these features with existing code
skeletons—waiting for your contributions:

- Download Center - Download server cores, plugins, and mods
- Backup Management - Incremental backup and restoration of world save files
- Intranet Penetration - FRP integration
- Scheduled Tasks - Automatic restarts, scheduled backups, and scheduled command execution
- Resource Management - Search and install plugins/mods from Modrinth / CurseForge
- Dark Theme - CSS variables are already configured; just add a dark mode value set

## Contributing

Contributions are welcome! Before you start, please read the [Contributing Guidelines](docs/CONTRIBUTING-en.md) to understand code standards and development workflows.

The UI is also fully customizable:

- Colors are managed via CSS variables
- Components are modular—replace any part you dislike
  Want to create a theme skin? Go for it.
  Want to completely redesign the layout? That's fine too.

### How to Contribute

1. Fork this repository
2. Create a branch and implement your changes (following the [Contributing Guidelines](docs/CONTRIBUTING-en.md)）
3. Submit a Pull Request
4. Your name will be added to the contributor wall on the About page

Coding skills aren't required to contribute:just sugget what new features you want,or share UI mockups/sketches,All contributions are valued.

### Add a new function

If you are going to add a「Save Management」：

**Backend**：

1. Create `backup_manager.rs` under `src-tauri/src/services/`,code the logic.
2. Create `backup.rs` under `src-tauri/src/commands/`,code the Tauri command
3. Add `pub mod backup` into `commands/mod.rs`
4. Regist the command in `lib.rs`'s `generate_handler!` macro.

**Fronted**：

1. Create `backup.ts` under `src/api/`,Encapsulate invoke calls
2. Create `BackupView.vue` under `src/views/`,draw the page
3. Add routes in src/router/index.ts
4. Add an item to the navItems array in AppSidebar.vue

Frontend/Backend each 3 files,Change one line each for the router and the sidebar。

## License

GPLv3

## Thank

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=FPSZ/SeaLantern&type=Date)](https://star-history.com/#FPSZ/SeaLantern&Date)

## Contributors

Thanks to everyone who contributed to Sea Lantern!

[![Contributors](https://sealentern-contributors.sb4893.workers.dev/)](https://github.com/FPSZ/SeaLantern/graphs/contributors)

## Acknowledgments

Sea Lantern is an OPEN SOURCE project,Complies with the GPLv3 license.

Minecraft is Mojang Studios's trademark.Not associated with Mojang.

"We built the framework, but the soul,is up to you."
