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
    <img src="https://img.shields.io/github/v/release/TharukRenuja/ClashPass?style=for-the-badge&color=e94560" alt="Version">
  </a>
  <a href="https://github.com/TharukRenuja/ClashPass/releases/latest">
    <img src="https://img.shields.io/github/repo-size/TharukRenuja/ClashPass?style=for-the-badge&color=555555" alt="Repo Size">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-5cb870?style=for-the-badge" alt="License">
  </a>
  <a href="https://github.com/TharukRenuja/ClashPass/releases">
    <img src="https://img.shields.io/github/downloads/TharukRenuja/ClashPass/total?style=for-the-badge&color=5b8fd6" alt="Downloads">
  </a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Linux-x86_%7C_arm-FCC624?style=for-the-badge&logo=linux&logoColor=white" alt="Linux">
  <img src="https://img.shields.io/badge/Windows-x86-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Windows">
  <img src="https://img.shields.io/badge/macOS-Intel_%7C_Silicon-A2AAAD?style=for-the-badge&logo=apple&logoColor=white" alt="macOS">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Built_with-Tauri_v2-FFC131?style=for-the-badge&logo=tauri&logoColor=black" alt="Tauri">
  <img src="https://img.shields.io/badge/Language-Rust-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Frontend-HTML%2FCSS%2FJS-E44D26?style=for-the-badge" alt="Frontend">
</p>

---

## Features

- **Import** CSV from Bitwarden, LastPass, 1Password, Proton Pass, Chrome, Firefox, and any standard password export
- **Auto-match** entries by title + email/username across files
- **Side-by-side** comparison with conflict highlighting (red = mismatched)
- **Click to resolve**, pick which version to keep per group
- **Export** merged conflict-free CSV
- **Dark theme** native desktop app built with Tauri v2

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

#### One-line install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/TharukRenuja/ClashPass/main/install.sh | sh
```

This detects your architecture (x86/arm), downloads the latest release, and installs the binary to `/usr/local/bin/`, desktop entry, and system icons.

#### Manual install from tarball

<details>
<summary>Install manually from tarball</summary>

Download the tarball from [Releases](https://github.com/TharukRenuja/ClashPass/releases/latest):

```bash
# x86_64
tar xzf clashpass-v*-x86-linux.tar.gz
sudo cp clashpass /usr/local/bin/
sudo chmod +x /usr/local/bin/clashpass

# Install icons
for size in 16 32 128 256; do
  sudo mkdir -p /usr/share/icons/hicolor/${size}x${size}/apps
  sudo cp icons/${size}x${size}.png /usr/share/icons/hicolor/${size}x${size}/apps/clashpass.png
done

# Create desktop entry
sudo tee /usr/share/applications/clashpass.desktop > /dev/null << EOF
[Desktop Entry]
Name=ClashPass
Comment=Password Conflict Resolver
Exec=/usr/local/bin/clashpass
Icon=clashpass
Type=Application
Categories=Utility;Security;PasswordManager;
Terminal=false
StartupNotify=true
EOF

sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
```

</details>

### Windows

Download `clashpass-v*-x86-windows-installer.exe` from [Releases](https://github.com/TharukRenuja/ClashPass/releases/latest) and run it.

### macOS

Download the appropriate DMG from [Releases](https://github.com/TharukRenuja/ClashPass/releases/latest):

- **Apple Silicon**: `clashpass-v*-silicon-macos-installer.dmg`
- **Intel**: `clashpass-v*-intel-macos-installer.dmg`

Open the DMG and drag to Applications.

### Build from source

Requires [Rust](https://rustup.rs/) (1.75+).

```bash
git clone https://github.com/TharukRenuja/ClashPass.git
cd ClashPass
cargo tauri build
```

On Wayland, if the window doesn't render:
```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 GDK_BACKEND=wayland cargo tauri dev
```
