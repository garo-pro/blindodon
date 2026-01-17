# Blindodon

An accessibility-first Mastodon client built with C# (WPF) and Rust.

## Project Structure

```
Blindodon/
├── mastodon-core/          # Rust backend
│   ├── src/
│   │   ├── api/            # Mastodon API client
│   │   ├── cache/          # SQLite caching
│   │   ├── crypto/         # Blindodon PM encryption (Phase 4)
│   │   ├── ipc/            # Named pipe IPC server
│   │   ├── logger/         # Structured logging
│   │   ├── models/         # Data structures
│   │   ├── streaming/      # WebSocket streaming
│   │   └── main.rs
│   └── Cargo.toml
│
├── Blindodon.UI/           # C# WPF frontend
│   ├── Views/              # XAML views
│   ├── ViewModels/         # MVVM view models
│   ├── Services/           # Business logic
│   ├── Controls/           # Custom controls
│   ├── Converters/         # Value converters
│   ├── Themes/             # UI themes
│   ├── Accessibility/      # Screen reader integration
│   └── Audio/              # Sound feedback
│
└── Blindodon.sln           # Visual Studio solution
```

## Building

### Prerequisites

- .NET 8.0 SDK
- Rust toolchain (rustup)
- Visual Studio 2022 or VS Code

### Build Rust Backend

```bash
cd mastodon-core
cargo build --release
```

### Build C# Frontend

```bash
cd Blindodon.UI
dotnet build
```

### Run

1. Start the Rust backend (or it will start automatically):
   ```bash
   ./mastodon-core/target/release/mastodon-core.exe
   ```

2. Start the UI:
   ```bash
   dotnet run --project Blindodon.UI
   ```

## Accessibility Features

- Full keyboard navigation (J/K for post navigation, customizable bindings)
- Screen reader support (NVDA, JAWS, Narrator)
- SAPI fallback for audio feedback
- High contrast themes
- Configurable sound notifications
- Content warning announcements
- Alt text for media

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| J | Next post |
| K | Previous post |
| N | New post |
| R | Reply |
| B | Boost |
| F | Favorite |
| Space | Read current post |
| Ctrl+1-5 | Switch timelines |
| Ctrl+R | Refresh |
| Ctrl+Q | Quit |

## Development Status

- [x] Phase 1: Core Foundation
- [ ] Phase 2: Essential Features
- [ ] Phase 3: Advanced Features
- [ ] Phase 4: Blindodon PM (E2E Encryption)
- [ ] Phase 5: Polish & Optimization

## License

MIT
