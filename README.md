# XGameStats

Universal Discord Rich Presence engine for singleplayer games.

## What it does

Scans running processes, reads game memory, and shows what you're playing in Discord. Supports **Unity** and **Unreal Engine** games with auto-detection.

## Features

- **Auto-detect Unity/Unreal Engine** — engine-specific memory patterns applied automatically
- **Auto-find game icon** — looks for `.ico`/`.png` in game directory
- **Memory reading** — level, health, character class, player name
- **Custom RPC templates** — configure what to show in Discord
- **Discord App ID** — set globally in settings

## Installation

Download `xgs-setup.exe` and run it. The installer will download all files and set up everything.

## Requirements

- Windows 10/11
- [JDK 21+](https://adoptium.net/)
- [Rust](https://rustup.rs/) (with `stable-x86_64-pc-windows-gnu` toolchain)
- [CMake](https://cmake.org/download/)
- [MinGW-w64](https://www.mingw-w64.org/)
- Discord running

## Adding a game

### Auto-detection (Unity/Unreal Engine)

If your game on Unity or Unreal Engine, just add the process name:

```json
{
  "process_name": "game.exe"
}
```

XGS will auto-detect the engine and apply memory patterns for level, health, and character class.

### Manual configuration

For other games, create a JSON file in `%LocalAppData%\XGameStats\`:

```json
{
  "process_name": "game.exe",
  "engine": "unknown",
  "pointers": {
    "level": {
      "base": "game.exe+0x02AB120",
      "offsets": [64, 24]
    },
    "health": {
      "base": "game.exe+0x02AB120",
      "offsets": [64, 32]
    },
    "player_name": {
      "base": "game.exe+0x02AB120",
      "offsets": [64, 16]
    },
    "character_class": {
      "base": "game.exe+0x02AB120",
      "offsets": [64, 40]
    }
  },
  "rpc_template": {
    "details": "{player_name} - {character_class}",
    "state": "Level {level} | HP {health}",
    "large_image": "game_logo",
    "large_image_text": "Game Name"
  }
}
```

### RPC template placeholders

- `{player_name}` — player's name
- `{level}` — current level
- `{health}` — current health
- `{character_class}` — character class (Assassin, Tank, etc.)

### Supported engines

| Engine | Auto-detect | Memory patterns |
|--------|-------------|-----------------|
| Unity | ✅ | ✅ |
| Unreal Engine 4 | ✅ | ✅ |
| Unreal Engine 5 | ✅ | ✅ |
| Other | ❌ | Manual JSON |

## Project structure

```
XGS/
├── core/               Rust engine (RPC, process tracking)
├── scanner/            C++ memory scanner
├── hooks/              C process hooks
├── gui/                JavaFX control panel
├── installer/          JavaFX installer (GUI)
├── installer-rs/       Rust installer (console, downloads from GitHub)
├── configs/            Game definitions
├── assets/             Icons and logos
└── translations.json   EN/RU translations
```

## Supported games

Any singleplayer game that can be read from memory. Unity and Unreal Engine games work automatically. For other games, add offsets manually.

## FAQ

**Q: How do I get my Discord App ID?**
A: Go to https://discord.com/developers/applications, create an app, copy the Application ID.

**Q: Will this work with multiplayer games?**
A: No, only singleplayer games. Anti-cheat protects multiplayer games.

**Q: How do I find memory offsets?**
A: Use Cheat Engine to scan for values, then create a JSON config with the offsets.

**Q: What if my game isn't Unity or Unreal?**
A: Create a manual JSON config with pointers to the memory addresses.