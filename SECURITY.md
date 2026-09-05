# Security Policy

## How the project works

XGameStats reads memory of singleplayer game processes to extract game state (level name, score, etc.) and displays it as Discord Rich Presence.

## What we access

- Process memory of games you explicitly configure
- Discord IPC named pipe (localhost only)
- File system: reads config files from `%LocalAppData%\XGameStats\`

## What we do NOT access

- Network (no internet connections, no telemetry)
- Other processes besides configured games
- System files or registry
- User credentials or tokens

## Memory reading

The scanner uses `ReadProcessMemory` Windows API. This is standard for game overlays and modding tools. We only read addresses you define in config files.

## Anti-cheat

This project is for singleplayer games only. Do not use with multiplayer games or games with anti-cheat software (EAC, BattlEye, Vanguard). Doing so may result in a ban.

## Builds

All builds are produced from source in this repository. No pre-compiled binaries are distributed except through GitHub Releases on this repository.

## Reporting vulnerabilities

Open an issue on GitHub with the tag "security".