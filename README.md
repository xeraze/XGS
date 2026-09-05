# XGameStats

Turn any singleplayer game into Discord Rich Presence.

## What it does

Scans running processes, reads game memory, and shows what you're playing in Discord. No game-specific plugins — just point it at an `.exe` and set the offsets.

## Features

- Memory scanning via C++ DLL
- Process tracking and injection hooks
- Discord RPC over named pipes (no third-party SDK)
- JSON config per game
- Dark GUI with JavaFX

## Requirements

- Windows 10/11
- [JDK 21+](https://adoptium.net/)
- [Rust](https://rustup.rs/) (with `stable-x86_64-pc-windows-gnu` toolchain)
- [CMake](https://cmake.org/download/)
- [MinGW-w64](https://www.mingw-w64.org/)
- Discord running

## Build

```bash
# Install Rust target
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup target add x86_64-pc-windows-gnu

# Build C++ scanner
cd scanner && mkdir build && cd build
cmake .. -G "MinGW Makefiles"
mingw32-make

# Build C hooks
cd ../../hooks && mkdir build && cd build
cmake .. -G "MinGW Makefiles"
mingw32-make

# Build Rust core
cd ../../core
cargo +stable-x86_64-pc-windows-gnu build --release

# Build Java GUI
cd ../gui
javac --module-path javafx-sdk/lib --add-modules javafx.controls,javafx.fxml ^
    -encoding UTF-8 -d build\classes ^
    src\main\java\com\gamepresence\gui\Main.java
jar cfm xgamestats-gui.jar MANIFEST.MF -C build\classes .

# Copy outputs to root
copy ..\core\target\release\xgs.exe ..
copy ..\scanner\build\libscanner.dll ..
copy ..\hooks\build\libhooks.dll ..
```

## Usage

```
xgs.exe          # Start everything
xgs.exe engine   # Engine only
xgs.exe gui      # GUI only
```

## Adding a game

Create a JSON file in `%LocalAppData%\XGameStats\` or the `configs/` folder:

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
├── core/           # Rust engine
├── scanner/        # C++ memory scanner
├── hooks/          # C process hooks
├── gui/            # JavaFX interface
└── configs/        # Game definitions
```

## License

MIT
