# ClashPass ⚔️

**Password Conflict Resolver** — import CSV exports from multiple password managers, compare entries side-by-side, resolve conflicts, and export a clean unified list.

## Features

- **Import** CSV from Bitwarden, LastPass, 1Password, Proton Pass, Chrome, Firefox, and any standard password export
- **Auto-match** entries by title + email/username across files
- **Side-by-side** comparison with conflict highlighting (red = mismatched)
- **Click to resolve** — pick which version to keep per group
- **Export** merged conflict-free CSV
- **Dark theme** native desktop app, single binary, zero runtime deps

## Install

### Debian / Ubuntu
```bash
sudo dpkg -i dist/clashpass_0.1.0_amd64.deb
clashpass
```

### Portable tarball
```bash
tar xzf dist/clashpass-v0.1.0-linux-x86_64.tar.gz
cd clashpass-v0.1.0
./clashpass
```

### Build from source
```bash
git clone https://github.com/TharukRenuja/ClashPass.git
cd ClashPass
./build.sh          # handles noexec filesystems
# or just:
cargo run --release
```

## Usage

1. Click **📂 Import CSV** to load exported password files
2. Entries are auto-grouped by title + email/username
3. Conflicts (different passwords, URLs, dates) highlighted in **red**
4. **Click** any conflicting value or use **"Keep entry from"** buttons to pick
5. Click **💾 Export CSV** to save the merged, conflict-free list

Toggle **"Show conflicts only"** to focus on mismatches and **"Files"** to manage loaded sources.

## Build from source

Requires [Rust](https://rustup.rs/) (1.75+).

```bash
cargo build --release
./target/release/clashpass
```

## Supported formats

| Manager     | Auto-detected columns                                    |
|-------------|----------------------------------------------------------|
| Bitwarden   | name, login_username, login_password, login_uri, notes   |
| LastPass    | name, url, username, password, extra                     |
| 1Password   | title, website, username, password, notes, created, updated |
| Proton Pass | name, url, email, password, note, createTime, modifyTime |
| Chrome      | name, url, username, password                            |
| Generic     | title/name, username/email, password, url/website, notes |

## Project structure

```
ClashPass/
├── src/
│   ├── main.rs      # Entry point + icon
│   ├── app.rs       # GUI (egui)
│   ├── models.rs    # Data structures
│   ├── parser.rs    # CSV parsing
│   └── export.rs    # CSV export
├── icons/
│   ├── clashpass.svg
│   ├── clashpass_32.png
│   ├── clashpass_64.png
│   └── clashpass_256.png
├── test_data/       # Sample CSV exports
├── dist/            # Release packages
├── Cargo.toml
└── README.md
```

## License

MIT
