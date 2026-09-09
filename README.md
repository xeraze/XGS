# XGameStats

Universal Discord Rich Presence engine for singleplayer games.

## What it does

Scans running processes, reads game memory, and shows what you're playing in Discord. No game-specific plugins — just point it at an `.exe` and set the offsets.

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

Create a JSON file in `%LocalAppData%\XGameStats\`:

```json
{
  "process_name": "game.exe",
  "discord_app_id": "123456789012345",
  "scan_type": "offsets",
  "pointers": {
    "level_name": {
      "base": "game.exe+0x02AB120",
      "offsets": [64, 24]
    }
  },
  "rpc_template": {
    "details": "Playing",
    "state": "Level: {level_name}",
    "large_image": "game_logo",
    "large_image_text": "Game Name"
  },
  "requires_elevation": false
}
```

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