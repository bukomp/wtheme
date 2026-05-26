# wtheme

Tiny Rust CLI that flips Windows 11 between **light** and **dark** mode without opening Settings.

## Requirements

- Windows 10 / 11 (per-user; no admin required)
- Rust 1.75+ to build from source

## Install

On Windows:

```powershell
cargo install --path .
```

### Cross-compile from macOS / Linux

```sh
brew install mingw-w64                    # macOS; Linux: apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/wtheme.exe
```

## Usage

```sh
wtheme status          # show current apps + system theme
wtheme dark            # set both to dark
wtheme light           # set both to light
wtheme toggle          # invert the current state
```

Flags (any subcommand):

- `--apps-only`     only change the apps theme
- `--system-only`   only change the shell theme (taskbar / Start)

## How it works

Writes two `DWORD`s under `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`:
`AppsUseLightTheme` and `SystemUsesLightTheme` (`0` = dark, `1` = light). Then broadcasts
`WM_SETTINGCHANGE` with `lParam = "ImmersiveColorSet"` via `SendMessageTimeoutW` so Explorer and
other listening apps repaint immediately — the same notification the Settings app sends.

## Notes

- Per-user only (HKCU). Accent color and wallpaper are not touched.
- A few legacy apps don't listen for the broadcast and need a restart to pick up the change.

## License

MIT — see `LICENSE`.
