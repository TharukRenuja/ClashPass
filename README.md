<p align="center">
  <img src="icons/clashpass_256.png" width="128" alt="ClashPass Logo">
</p>

<h1 align="center">ClashPass</h1>

<p align="center">
  <strong>Password Conflict Resolver</strong><br>
  Import CSV exports from multiple password managers, compare entries side-by-side, resolve conflicts, and export a clean unified list.
</p>

<p align="center">
  <a href="https://github.com/TharukRenuja/ClashPass/releases/latest">
    <img src="https://img.shields.io/github/v/release/TharukRenuja/ClashPass?style=flat-square&color=blue" alt="Version">
  </a>
  <a href="https://github.com/TharukRenuja/ClashPass/releases/latest">
    <img src="https://img.shields.io/github/repo-size/TharukRenuja/ClashPass?style=flat-square" alt="Repo Size">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License">
  </a>
  <a href="https://github.com/TharukRenuja/ClashPass/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/TharukRenuja/ClashPass/release.yml?style=flat-square&label=build" alt="Build">
  </a>
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20windows%20%7C%20macos-lightgrey?style=flat-square" alt="Platform">
</p>

---

## Features

- **Import** CSV from Bitwarden, LastPass, 1Password, Proton Pass, Chrome, Firefox, and any standard password export
- **Auto-match** entries by title + email/username across files
- **Side-by-side** comparison with conflict highlighting (red = mismatched)
- **Click to resolve**, pick which version to keep per group
- **Export** merged conflict-free CSV
- **Dark theme** native desktop app, single binary, minimal runtime deps

## Usage

1. Click **Import CSV** to load exported password files
2. Entries are auto-grouped by title + email/username
3. Conflicts (different passwords, URLs, dates) highlighted in **red**
4. **Click** any conflicting value or use **"Keep entry from"** buttons to pick
5. Click **Export CSV** to save the merged, conflict-free list

Toggle **"Show conflicts only"** to focus on mismatches and **"Files"** to manage loaded sources.

## Supported formats

| Manager     | Auto-detected columns                                    |
|-------------|----------------------------------------------------------|
| Bitwarden   | name, login_username, login_password, login_uri, notes   |
| LastPass    | name, url, username, password, extra                     |
| 1Password   | title, website, username, password, notes, created, updated |
| Proton Pass | name, url, email, password, note, createTime, modifyTime |
| Chrome      | name, url, username, password                            |
| Generic     | title/name, username/email, password, url/website, notes |

## Install

### Linux

Download the latest binary from [Releases](https://github.com/TharukRenuja/ClashPass/releases/latest), then:

```bash
chmod +x clashpass-v*-linux-amd64
./clashpass-v*-linux-amd64          # Auto-installs to ~/.local/bin/ and adds to start menu
```

That's it. The binary installs itself — desktop entry, icon, and PATH. After first run, just launch `clashpass` from your app launcher.

Other commands:
```bash
./clashpass --install     # Manual reinstall
./clashpass --uninstall   # Remove binary, desktop entry, and icon
```

### Windows

Download `clashpass-v*-windows-amd64.exe` from [Releases](https://github.com/TharukRenuja/ClashPass/releases/latest) and run it.

### macOS

Download `clashpass-v*-macos-*.dmg` from [Releases](https://github.com/TharukRenuja/ClashPass/releases/latest), open the DMG, and drag to Applications.

Or use the portable `.app` bundle — no install needed.

### Build from source

Requires [Rust](https://rustup.rs/) (1.75+).

```bash
git clone https://github.com/TharukRenuja/ClashPass.git
cd ClashPass
cargo build --release
./target/release/clashpass
```

## Project structure

```
ClashPass/
├── src/
│   ├── main.rs      # Entry point + self-installer
│   ├── app.rs       # GUI (egui)
│   ├── models.rs    # Data structures
│   ├── parser.rs    # CSV parsing
│   └── export.rs    # CSV export
├── icons/
│   ├── clashpass.svg
│   ├── clashpass_32.png
│   ├── clashpass_64.png
│   └── clashpass_256.png
├── Cargo.toml
├── LICENSE          # MIT
└── README.md
```
