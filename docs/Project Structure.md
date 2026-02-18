# Sea Lantern Project Structure

## Project Overview

Sea Lantern is a lightweight Minecraft server management tool based on Tauri 2 + Rust + Vue 3.

## Detailed Structure

```
sea-lantern/
│
├── src/                                # Frontend code (Vue 3 + TypeScript)
│   │
│   ├── api/                           # Communication layer with Rust backend
│   │   ├── tauri.ts                  # Basic invoke wrapper, entry point for all APIs
│   │   ├── server.ts                 # Server management API (create, start, stop, logs)
│   │   ├── java.ts                   # Java environment detection API
│   │   ├── config.ts                 # Configuration file read/write API
│   │   ├── player.ts                 # Player management API (whitelist, ban, OP)
│   │   ├── settings.ts               # Application settings API
│   │   ├── system.ts                 # System info, file dialog API
│   │   ├── update.ts                 # Software update check API
│   │   └── remoteLocales.ts          # Remote locale API
│   │
│   ├── assets/                        # Static resources
│   │   ├── logo.svg                  # Application icon
│   │   └── vue.svg                   # Vue icon
│   │
│   ├── components/                    # UI components
│   │   ├── common/                   # Common components (building blocks of the project)
│   │   │   ├── BrandIcon.vue         # Brand icon component
│   │   │   ├── SLButton.vue          # Button component
│   │   │   ├── SLCard.vue            # Card container
│   │   │   ├── SLInput.vue           # Input component
│   │   │   ├── SLSelect.vue          # Dropdown select component
│   │   │   ├── SLSwitch.vue          # Switch component
│   │   │   ├── SLModal.vue           # Modal component
│   │   │   ├── SLProgress.vue        # Progress bar component
│   │   │   ├── SLBadge.vue           # Status badge component
│   │   │   ├── SLToast.vue           # Toast component
│   │   │   ├── SLTooltip.vue         # Tooltip component
│   │   │   ├── SLCheckbox.vue        # Checkbox component
│   │   │   ├── SLFormField.vue       # Form field component
│   │   │   ├── SLTextarea.vue        # Textarea component
│   │   │   ├── SLTabs.vue            # Tabs component
│   │   │   ├── SLTabPanel.vue        # Tab panel component
│   │   │   ├── SLSpinner.vue         # Loading spinner component
│   │   │   ├── SLContextMenu.vue     # Context menu component
│   │   │   ├── SLNotification.vue    # Notification component
│   │   │   ├── SLCloseDialog.vue     # Close dialog component
│   │   │   ├── UpdateModal.vue       # Update modal component
│   │   │   └── index.ts              # Component export file
│   │   │
│   │   ├── config/                   # Configuration related components
│   │   │   ├── ConfigCategories.vue  # Configuration categories component
│   │   │   ├── ConfigEntry.vue       # Configuration entry component
│   │   │   └── ConfigToolbar.vue     # Configuration toolbar component
│   │   │
│   │   ├── console/                  # Console related components
│   │   │   ├── CommandModal.vue      # Command modal component
│   │   │   ├── ConsoleCommands.vue   # Console commands component
│   │   │   ├── ConsoleInput.vue      # Console input component
│   │   │   ├── ConsoleOutput.vue     # Console output component
│   │   │   └── ConsoleToolbar.vue    # Console toolbar component
│   │   │
│   │   ├── effects/                  # Effects components
│   │   │   └── ZombieOverlay.vue     # Zombie effect overlay component
│   │   │
│   │   ├── layout/                   # Page layout components
│   │   │   ├── AppLayout.vue         # Main layout (sidebar + content area)
│   │   │   ├── AppSidebar.vue        # Side navigation bar
│   │   │   ├── AppHeader.vue         # Top title bar
│   │   │   └── index.ts              # Layout component export file
│   │   │
│   │   ├── splash/                   # Splash screen
│   │   │   └── SplashScreen.vue      # Loading animation when app starts
│   │   │
│   │   ├── JavaDownloader.vue        # Java downloader component
│   │   └── index.ts                  # Component export file
│   │
│   ├── composables/                   # Composition API functions
│   │   └── useToast.ts               # Toast component composition function
│   │
│   ├── data/                          # Static data
│   │   └── contributors.ts           # Contributors information list
│   │
│   ├── locales/                       # Internationalization resources
│   │   └── index.ts                   # Language file entry
│   │
│   ├── router/                        # Router configuration
│   │   └── index.ts                   # Route table definition
│   │
│   ├── stores/                        # Pinia state management
│   │   ├── index.ts                   # Pinia instance initialization
│   │   ├── serverStore.ts             # Server list and running status
│   │   ├── consoleStore.ts            # Console logs (persist across page changes)
│   │   ├── uiStore.ts                 # UI state (sidebar collapse, etc.)
│   │   ├── settingsStore.ts           # Application settings state
│   │   ├── i18nStore.ts               # Internationalization state
│   │   ├── updateStore.ts             # Update check state
│   │   └── contextMenuStore.ts        # Context menu state
│   │
│   ├── styles/                        # Global styles
│   │   ├── variables.css              # CSS variables (colors, spacing, rounded corners, fonts, shadows)
│   │   ├── reset.css                  # Browser style reset
│   │   ├── typography.css             # Typography styles
│   │   ├── animations.css             # Animation keyframes
│   │   └── glass.css                  # Glassmorphism effect styles
│   │
│   ├── types/                         # Type definitions
│   │   ├── common.ts                  # Common type definitions
│   │   └── server.ts                  # Server related type definitions
│   │
│   ├── utils/                         # Utility functions
│   │   ├── constants.ts               # Constant definitions
│   │   ├── errorHandler.ts            # Error handling utility
│   │   ├── logger.ts                  # Logging utility
│   │   ├── serverStatus.ts            # Server status utility
│   │   ├── tray.ts                    # System tray utility
│   │   └── version.ts                 # Version utility
│   │
│   ├── views/                         # Page views (one per route)
│   │   ├── HomeView.vue               # Home page (server list, system status)
│   │   ├── CreateServerView.vue       # Create/import server page
│   │   ├── ConsoleView.vue            # Console page (real-time logs, command input)
│   │   ├── ConfigView.vue             # Configuration edit page (server.properties)
│   │   ├── PlayerView.vue             # Player management page (whitelist, ban, OP)
│   │   ├── SettingsView.vue           # Application settings page
│   │   ├── PaintView.vue              # Personalization settings page
│   │   └── AboutView.vue              # About page (contributors wall, update check)
│   │
│   ├── App.vue                        # Root component
│   ├── main.ts                        # Application entry (initialize Vue, Pinia, Router)
│   ├── style.css                      # Style import summary
│   └── vite-env.d.ts                  # Vite environment type declarations
│
├── src-tauri/                         # Backend code (Rust + Tauri 2)
│   │
│   ├── capabilities/                  # Tauri permissions configuration
│   │   └── default.json               # Default permissions settings
│   │
│   ├── icons/                         # Application icons
│   │   ├── 32x32.png                  # 32x32 icon
│   │   ├── 64x64.png                  # 64x64 icon
│   │   ├── 128x128.png                # 128x128 icon
│   │   ├── 128x128@2x.png             # 128x128 2x icon
│   │   ├── icon.icns                  # macOS icon
│   │   ├── icon.ico                   # Windows icon
│   │   ├── icon.png                   # Generic icon
│   │   ├── source.png                 # Source icon
│   │   ├── Square30x30Logo.png        # 30x30 square icon
│   │   ├── Square44x44Logo.png        # 44x44 square icon
│   │   ├── Square71x71Logo.png        # 71x71 square icon
│   │   ├── Square89x89Logo.png        # 89x89 square icon
│   │   ├── Square107x107Logo.png      # 107x107 square icon
│   │   ├── Square142x142Logo.png      # 142x142 square icon
│   │   ├── Square150x150Logo.png      # 150x150 square icon
│   │   ├── Square284x284Logo.png      # 284x284 square icon
│   │   ├── Square310x310Logo.png      # 310x310 square icon
│   │   ├── StoreLogo.png              # Store icon
│   │   ├── android/                   # Android icons
│   │   └── ios/                       # iOS icons
│   │
│   ├── src/                           # Rust source code
│   │   ├── commands/                  # Tauri commands (APIs called by frontend invoke)
│   │   │   ├── mod.rs                 # Module export
│   │   │   ├── server.rs              # Server management commands
│   │   │   ├── java.rs                # Java detection commands
│   │   │   ├── config.rs              # Configuration file read/write commands
│   │   │   ├── player.rs              # Player management commands
│   │   │   ├── settings.rs            # Application settings commands
│   │   │   ├── system.rs              # System info, file dialog commands
│   │   │   ├── update.rs              # Software update check commands
│   │   │   ├── join.rs                # Join server commands
│   │   │   ├── mods.rs                # Mod management commands
│   │   │   └── server_id.rs           # Server ID management commands
│   │   │
│   │   ├── services/                  # Business logic layer
│   │   │   ├── mod.rs                 # Module export
│   │   │   ├── server_manager.rs      # Server process management, log reading
│   │   │   ├── java_detector.rs       # Java environment scanner
│   │   │   ├── java_installer.rs      # Java installer
│   │   │   ├── config_parser.rs       # .properties file parser
│   │   │   ├── player_manager.rs      # Player data file reading
│   │   │   ├── settings_manager.rs    # Application settings persistence
│   │   │   ├── mod_manager.rs         # Mod manager
│   │   │   ├── join_manager.rs        # Join server manager
│   │   │   ├── server_id_manager.rs   # Server ID manager
│   │   │   ├── i18n.rs                # Internationalization service
│   │   │   └── global.rs              # Global singleton manager
│   │   │
│   │   ├── models/                    # Data structure definitions
│   │   │   ├── mod.rs                 # Module export
│   │   │   ├── server.rs              # Server instance, status data structures
│   │   │   ├── config.rs              # Configuration entry data structures
│   │   │   ├── settings.rs            # Application settings data structures
│   │   │   └── dev_config.rs          # Developer configuration data structures
│   │   │
│   │   ├── utils/                     # Utility functions
│   │   │   ├── mod.rs                 # Utility module
│   │   │   ├── cli.rs                 # Command line utility
│   │   │   └── downloader.rs          # Download utility
│   │   │
│   │   ├── lib.rs                     # Tauri library entry (plugin registration, command registration)
│   │   └── main.rs                    # Application main function
│   │
│   ├── .gitignore                     # Git ignore file
│   ├── Cargo.lock                     # Rust dependency lock file
│   ├── Cargo.toml                     # Rust dependency configuration
│   ├── build.rs                       # Build script
│   └── tauri.conf.json                # Tauri configuration (window size, title, version, etc.)
│
├── lang/                              # Language files
│   ├── README.md                      # Language file description
│   ├── README-en.md                   # English language file description
│   ├── de-DE.json                     # German translation
│   ├── en-US.json                     # English translation
│   ├── es-ES.json                     # Spanish translation
│   ├── fr-FA.json                     # French translation
│   ├── ja-JP.json                     # Japanese translation
│   ├── ko-KR.json                     # Korean translation
│   ├── ru-RU.json                     # Russian translation
│   ├── vi-VN.json                     # Vietnamese translation
│   ├── zh-CN.json                     # Simplified Chinese translation
│   └── zh-TW.json                     # Traditional Chinese translation
│
├── docs/                              # Documentation
│   ├── AI_GUIDE.md                    # AI Usage Guide
│   ├── CONTRIBUTING.md                # Contribution Guide
│   ├── CONTRIBUTING-en.md             # English Contribution Guide
│   ├── i18n-help.md                   # Internationalization Help
│   ├── i18n-list.md                   # Internationalization Language List
│   ├── 新手使用教程.html              # Newbie Usage Tutorial
│   ├── 项目结构.md                    # Project Structure (Chinese)
│   └── Project Structure.md           # Project Structure (English)
│
├── .SRCINFO                           # Package information
├── .editorconfig                      # Editor configuration
├── .gitattributes                     # Git attributes configuration
├── .gitignore                         # Git ignore file
├── .oxfmtrc.json                      # Oxlint formatting configuration
├── .oxlintrc.json                     # Oxlint configuration
├── Cargo.lock                         # Rust dependency lock file
├── Cargo.toml                         # Rust dependency configuration
├── LICENSE                            # License file
├── PKGBUILD                           # Arch Linux package build file
├── README.md                          # Project description document
├── README-en.md                       # English project description document
├── index.html                         # HTML entry file
├── package-lock.json                  # Node.js dependency lock file
├── package.json                       # Node.js dependency configuration
├── rustfmt.toml                       # Rust code formatting configuration
├── sealantern.desktop                 # Linux desktop file
├── sealantern.install                 # Installation script
├── tsconfig.json                      # TypeScript configuration
├── tsconfig.node.json                 # Node.js environment TypeScript configuration
└── vite.config.ts                     # Vite build configuration
```

## Core Function Modules

### 1. Server Management
- Create, import, start, stop, delete servers
- Support different server startup methods (jar, bat, sh)
- Support modpack import

### 2. Console Management
- Real-time server log viewing
- Direct command sending to server
- Support custom commands

### 3. Configuration Management
- Graphical editing of server.properties
- Support for multiple configuration item types

### 4. Player Management
- Whitelist management
- Ban management
- OP management

### 5. System Monitoring
- CPU, memory, disk usage
- Real-time system resource monitoring

### 6. Internationalization
- Support for multiple languages
- Built-in multiple language packs

### 7. Update Management
- Check for updates
- One-click download of new versions

## Technical Features

1. **Lightweight**: Uses Tauri instead of Electron, small size, fast startup, low memory usage
2. **High Performance**: Rust backend handles core logic with excellent performance
3. **Cross-platform**: Supports Windows, macOS, Linux
4. **Responsive Design**: Supports different screen sizes
5. **Modular Architecture**: Clear code structure, easy to extend
6. **Internationalization Support**: Built-in multi-language support
7. **Safe and Reliable**: Automatically stops servers when closing the software to prevent data loss

## Development Guide

### Frontend Development
- Use Vue 3 Composition API
- Use TypeScript to ensure type safety
- Use Pinia for state management
- Use Vite for building

### Backend Development
- Use Rust language
- Use Tauri 2 as desktop application framework
- Use serde for serialization/deserialization
- Use tokio for asynchronous processing

### Code Standards
- Frontend: Use ESLint and Prettier to ensure code quality
- Backend: Use rustfmt and clippy to ensure code quality

## Build and Deployment

### Development Environment
```bash
# Install dependencies
npm install

# Start development server
npm run tauri dev
```

### Production Build
```bash
# Build release version
npm run tauri build

# Build output location
src-tauri/target/release/bundle/
```

## Extension Guide

### Adding New Features

1. **Backend**:
   - Create a new service file under `src-tauri/src/services/`
   - Create a new command file under `src-tauri/src/commands/`
   - Export the module in `commands/mod.rs`
   - Register the command in the `generate_handler!` macro in `lib.rs`

2. **Frontend**:
   - Create a new API wrapper file under `src/api/`
   - Create a new page component under `src/views/`
   - Add a route in `src/router/index.ts`
   - Create related components under `src/components/`
   - Create related state management under `src/stores/`

## Notes

1. **Data Storage**: Server data is stored in `sea_lantern_servers.json` in the executable directory
2. **Log Management**: Server logs are stored in `latest.log` in the server directory
3. **Permission Management**: Ensure the application has sufficient file system permissions
4. **Performance Optimization**: Avoid time-consuming operations in the main thread
5. **Error Handling**: Ensure all errors have appropriate handling and prompts

## Future Plans

- Download Center: Download server cores, plugins, mods
- Backup Management: Incremental backup and restoration of world saves
- Intranet Penetration: Integrate FRP
- Scheduled Tasks: Automatic restart, scheduled backup, scheduled command execution
- Resource Management: Search and install plugins and mods from Modrinth / CurseForge

---

**Note**: This document will be updated as the project develops. For the latest project structure, please refer to the code repository.