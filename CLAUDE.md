# Blindodon - Claude Context File

## License

Blindodon is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

All source files must include the AGPL license header. When creating new files, add the appropriate header:

**For Rust files (.rs):**
```rust
// Blindodon - An accessibility-first Mastodon client
// Copyright (C) 2025 Blindodon Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
```

**For C# files (.cs):**
```csharp
// Blindodon - An accessibility-first Mastodon client
// Copyright (C) 2025 Blindodon Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
```

## Git Workflow

**IMPORTANT: Commit work immediately after completing each task or feature.**

- After making changes, commit them right away - do not batch multiple unrelated changes
- Use clear, descriptive commit messages following conventional commit style
- The repository uses GitHub Actions for automatic alpha builds on every commit

## Project Overview

Blindodon is an accessibility-first Mastodon client built with:
- **C# WPF (.NET 8)** - UI, accessibility APIs, screen reader integration
- **Rust** - Backend core, Mastodon API, streaming, caching

The primary goal is full accessibility for blind and visually impaired users, with complete keyboard navigation and screen reader support.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     C# WPF Frontend                         │
│  ┌─────────────┐ ┌──────────────┐ ┌───────────────────────┐ │
│  │ MainWindow  │ │ ViewModels   │ │ Services              │ │
│  │ (XAML/UI)   │ │ (MVVM)       │ │ - MastodonBridge      │ │
│  │             │ │              │ │ - AccessibilityManager│ │
│  │             │ │              │ │ - AudioManager        │ │
│  │             │ │              │ │ - KeybindingManager   │ │
│  └─────────────┘ └──────────────┘ └───────────────────────┘ │
└────────────────────────┬────────────────────────────────────┘
                         │ Named Pipe IPC (JSON)
┌────────────────────────┴────────────────────────────────────┐
│                     Rust Backend                            │
│  ┌─────────────┐ ┌──────────────┐ ┌───────────────────────┐ │
│  │ IPC Server  │ │ API Client   │ │ Streaming             │ │
│  │ (handler)   │ │ (megalodon)  │ │ (WebSocket)           │ │
│  └─────────────┘ └──────────────┘ └───────────────────────┘ │
│  ┌─────────────┐ ┌──────────────┐ ┌───────────────────────┐ │
│  │ Cache       │ │ Logger       │ │ Crypto (Phase 4)      │ │
│  │ (SQLite)    │ │ (tracing)    │ │ (Blindodon PM)        │ │
│  └─────────────┘ └──────────────┘ └───────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## IPC Protocol

Communication uses JSON messages over Windows named pipes (`\\.\pipe\blindodon_ipc`):

```json
{
  "id": "uuid",
  "type": "request|response|event",
  "method": "timeline.get|post.create|...",
  "params": {},
  "result": {},
  "error": { "code": -1001, "message": "..." }
}
```

Key methods defined in `mastodon-core/src/models/ipc_message.rs`.

## Development Status

### Completed (Phase 1)
- [x] Rust project structure with all modules
- [x] C# WPF project with MVVM architecture
- [x] IPC communication layer (both sides)
- [x] Mastodon API integration (OAuth, timelines)
- [x] Basic logging system
- [x] Main window with timeline view
- [x] Screen reader integration foundation
- [x] Keyboard navigation framework
- [x] Audio feedback system

### In Progress (Phase 2)
- [x] **Notifications timeline** - Full implementation complete
  - Rust: `notifications.get`, `notifications.clear`, `notifications.dismiss` IPC methods
  - Rust: Notification model, converter, API client methods
  - C#: NotificationViewModel with accessibility support
  - C#: Notifications tab in MainWindow with dedicated ListBox
  - C#: Keyboard navigation (J/K, Space to announce)
  - C#: Dismiss individual / clear all notifications
- [ ] Post composition (ComposeWindow)
- [ ] Media attachments display
- [ ] Media upload

### Pending Phases
- **Phase 3**: User/hashtag timelines, search, lists, filtering, multi-account
- **Phase 4**: Blindodon PM (end-to-end encrypted DMs)
- **Phase 5**: Performance optimization, themes, sound packs, testing

## Key Files to Know

### Rust
| Path | Purpose |
|------|---------|
| `mastodon-core/src/ipc/handler.rs` | Routes all IPC messages |
| `mastodon-core/src/api/client.rs` | Mastodon API wrapper |
| `mastodon-core/src/api/converter.rs` | Converts megalodon types to Blindodon types |
| `mastodon-core/src/models/ipc_message.rs` | IPC protocol definitions |
| `mastodon-core/src/models/timeline.rs` | Timeline types and settings |
| `mastodon-core/src/models/notification.rs` | Notification model and request/response types |

### C#
| Path | Purpose |
|------|---------|
| `Blindodon.UI/Services/MastodonBridge.cs` | IPC client |
| `Blindodon.UI/Services/AccessibilityManager.cs` | Screen reader integration |
| `Blindodon.UI/ViewModels/MainViewModel.cs` | Main application logic |
| `Blindodon.UI/ViewModels/NotificationViewModel.cs` | Notification display and accessibility |
| `Blindodon.UI/Views/MainWindow.xaml` | Primary UI |
| `Blindodon.UI/Converters/Converters.cs` | Value converters for UI bindings |

## Coding Conventions

### Rust
- Use `tracing` macros for logging (`info!`, `debug!`, `error!`)
- All API methods return `Result<T>`
- Models derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- Use `anyhow::Result` for error handling

### C#
- MVVM pattern with CommunityToolkit.Mvvm
- Use `[ObservableProperty]` for bindable properties
- Use `[RelayCommand]` for commands
- Log with Serilog (`Log.Information`, `Log.Error`)
- All UI controls must have `AutomationProperties.Name`

## Accessibility Requirements (Critical)

1. **Every interactive element** must have `AutomationProperties.Name`
2. **Focus indicators** must be visible (3px yellow border)
3. **Screen reader announcements** via `AccessibilityManager.Announce()`
4. **Keyboard shortcuts** for all actions (no mouse required)
5. **Audio feedback** for state changes via `AudioManager.Play()`
6. **Content warnings** must be announced before content

## Building & Running

```bash
# Rust backend
cd mastodon-core
cargo build

# C# frontend
cd Blindodon.UI
dotnet build

# Run (C# will auto-start Rust if in expected path)
dotnet run --project Blindodon.UI
```

## Testing Checklist

- [ ] Full keyboard navigation (Tab, J/K, Enter, Escape)
- [ ] NVDA screen reader compatibility
- [ ] JAWS screen reader compatibility
- [ ] Windows Narrator compatibility
- [ ] All sounds play correctly
- [ ] Memory usage < 150MB with 5 timelines
- [ ] OAuth flow completes successfully
- [ ] Timeline loads and refreshes
- [ ] Post actions work (boost, favorite)
- [ ] Notifications load and display correctly
- [ ] Notification dismiss/clear works
- [ ] Notification keyboard navigation (J/K, Space)

## Common Tasks

### Adding a new IPC method
1. Add method constant in `mastodon-core/src/models/ipc_message.rs`
2. Add handler in `mastodon-core/src/ipc/handler.rs`
3. Call from C# via `App.Bridge.SendRequestAsync("method.name", params)`

### Adding a new keyboard shortcut
1. Register in `KeybindingManager.LoadDefaultBindings()`
2. Register action handler in `MainWindow.SetupKeyboardNavigation()`

### Adding a new sound event
1. Add enum value in `AudioManager.SoundEvent`
2. Add file mapping in `AudioManager.LoadSoundPack()`
3. Place .wav file in `Sounds/default/` directory

## Notes for Future Sessions

- The Blindodon PM encryption (Phase 4) should use Signal Protocol principles
- Multi-account support needs careful UI consideration for switching
- Consider virtual scrolling performance for large timelines
- The streaming module is stubbed but needs reconnection logic
- Filter system should support regex patterns
