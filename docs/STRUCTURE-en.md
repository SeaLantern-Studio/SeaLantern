# Sea Lantern Project Structure

## Project Overview

Sea Lantern is a lightweight Minecraft server management tool based on Tauri 2 + Rust + Vue 3 stack.

## Detailed Structure

```
sea-lantern/
│
├── src/                                # Frontend code (Vue 3 + TypeScript)
│   │
│   ├── api/                           # Communication layer with Rust backend
│   │   ├── tauri.ts                  # Base invoke wrapper, entry point for all APIs
│   │   ├── server.ts                 # Server management API (create, start, stop, logs)
│   │   ├── java.ts                   # Java environment detection API
│   │   ├── config.ts                 # Config file read/write API
│   │   ├── player.ts                 # Player management API (whitelist, ban, OP)
│   │   ├── settings.ts               # Application settings API
│   │   ├── system.ts                 # System info, file dialog API
│   │   ├── update.ts                 # Software update check API
│   │   └── remoteLocales.ts          # Remote language pack API
│   │
│   ├── assets/                        # Static resources
│   │   ├── logo.svg                  # Application icon
│   │   └── vue.svg                   # Vue icon
│   │
│   ├── components/                    # UI Components
│   │   ├── common/                   # Common components (building blocks)
│   │   │   ├── BrandIcon.vue         # Brand icon component
│   │   │   ├── SLButton.vue          # Button component
│   │   │   ├── SLCard.vue            # Card container
│   │   │   ├── SLInput.vue           # Input component
│   │   │   ├── SLSelect.vue          # Select component
│   │   │   ├── SLSwitch.vue          # Switch component
│   │   │   ├── SLModal.vue           # Modal component
│   │   │   ├── SLProgress.vue        # Progress component
│   │   │   ├── SLBadge.vue           # Badge component
│   │   │   ├── SLToast.vue           # Toast component
│   │   │   ├── SLTooltip.vue         # Tooltip component
│   │   │   ├── SLCheckbox.vue        # Checkbox component
│   │   │   ├── SLFormField.vue       # Form field component
│   │   │   ├── SLTextarea.vue        # Textarea component
│   │   │   ├── SLTabs.vue            # Tabs component
│   │   │   ├── SLTabPanel.vue        # Tab panel component
│   │   │   ├── SLSpinner.vue         # Spinner component
│   │   │   ├── SLContextMenu.vue     # Context menu component
│   │   │   ├── SLNotification.vue    # Notification component
│   │   │   ├── SLCloseDialog.vue     # Close dialog component
│   │   │   ├── UpdateModal.vue       # Update modal component
│   │   │   └── index.ts              # Component exports
│   │   │
│   │   ├── config/                   # Config related components
│   │   │   ├── ConfigCategories.vue  # Config categories component
│   │   │   ├── ConfigEntry.vue       # Config entry component
│   │   │   └── ConfigToolbar.vue     # Config toolbar component
│   │   │
│   │   ├── console/                  # Console related components
│   │   │   ├── CommandModal.vue      # Command modal component
│   │   │   ├── LogViewer.vue         # Log viewer component
│   │   │   └── PlayerList.vue        # Player list component
│   │   │
│   │   └── layout/                   # Layout components
│   │       ├── AppHeader.vue         # App header
│   │       ├── AppSidebar.vue        # Sidebar navigation
│   │       └── AppLayout.vue         # Main layout
│   │
│   ├── composables/                   # Vue composables
│   │   ├── useMessage.ts             # Message/notification composable
│   │   ├── useAsync.ts               # Async operation composable
│   │   └── useTheme.ts               # Theme management composable
│   │
│   ├── src/language/                  # Internationalization (source code)
│   │   ├── index.ts                  # i18n setup
│   │   ├── README.md                 # Documentation (CN)
│   │   ├── README-en.md              # Documentation (EN)
│   │   └── locales/                  # Translation files
│   │       ├── zh-CN.json            # Simplified Chinese
│   │       └── en-US.json            # English
│   │
│   ├── router/                        # Vue Router
│   │   └── index.ts                  # Route definitions
│   │
│   ├── stores/                        # Pinia stores
│   │   ├── serverStore.ts            # Server state management
│   │   └── consoleStore.ts           # Console state management
│   │
│   ├── styles/                        # Global styles
│   │   ├── variables.css             # CSS variables
│   │   ├── base.css                  # Base styles
│   │   └── components.css            # Component styles
│   │
│   ├── themes/                        # Theme system
│   │   ├── index.ts                  # Theme management
│   │   └── README.md                 # Theme documentation
│   │
│   ├── utils/                         # Utility functions
│   │   ├── format.ts                 # Format utilities
│   │   └── serverStatus.ts           # Server status utilities
│   │
│   ├── views/                         # Page views
│   │   ├── HomeView.vue              # Home page
│   │   ├── ConsoleView.vue           # Console page
│   │   ├── ConfigView.vue            # Config page
│   │   ├── CreateView.vue            # Create server page
│   │   ├── PlayersView.vue           # Player management page
│   │   └── SettingsView.vue          # Settings page
│   │
│   ├── App.vue                        # Root component
│   └── main.ts                        # Entry point
│
├── src-tauri/                         # Backend code (Rust)
│   ├── src/                           # Rust source
│   │   ├── main.rs                    # Entry point
│   │   ├── lib.rs                     # Library exports
│   │   ├── commands/                  # Command handlers
│   │   │   ├── server.rs              # Server commands
│   │   │   ├── java.rs                # Java commands
│   │   │   ├── config.rs              # Config commands
│   │   │   ├── player.rs              # Player commands
│   │   │   ├── settings.rs            # Settings commands
│   │   │   ├── system.rs              # System commands
│   │   │   └── update.rs              # Update commands
│   │   │
│   │   ├── models/                    # Data models
│   │   │   ├── server.rs              # Server model
│   │   │   └── player.rs              # Player model
│   │   │
│   │   ├── services/                  # Business logic
│   │   │   ├── server_service.rs      # Server service
│   │   │   └── player_service.rs      # Player service
│   │   │
│   │   └── utils/                     # Utilities
│   │       └── ...
│   │
│   ├── Cargo.toml                     # Rust dependencies
│   └── tauri.conf.json                # Tauri configuration
│
├── docs/                              # Documentation
│   ├── STRUCTURE.md                   # Chinese version
│   ├── STRUCTURE-en.md                # This file (English version)
│   ├── CONTRIBUTING.md                # Contributing guide (CN)
│   └── CONTRIBUTING-en.md             # Contributing guide (EN)
│
├── public/                            # Static public assets
├── package.json                       # Node dependencies
├── tsconfig.json                      # TypeScript config
├── vite.config.ts                     # Vite config
└── README.md                          # Project readme
```

## Key Design Principles

1. **Frontend-Backend Separation**: Frontend (Vue) and backend (Rust) communicate via Tauri invoke
2. **Component-Based Architecture**: Reusable UI components in `components/common/`
3. **State Management**: Pinia stores for global state
4. **Type Safety**: Full TypeScript support on frontend
5. **Rust Safety**: Memory-safe backend operations

## Development Workflow

1. Frontend development: `npm run tauri dev`
2. Backend development: Rust code in `src-tauri/src/`
3. Build release: `npm run tauri build`
