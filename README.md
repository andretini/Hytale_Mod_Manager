# Hytale Mod Manager

A tool for managing Hytale resources via the CurseForge API. Available as both a GUI (PySide6) and an APT-style CLI for servers.

This manager was built because CurseForge provides no native support for Hytale on Linux.

---

## CLI Usage (Recommended for servers)

The CLI provides an APT-style interface with minimal dependencies, perfect for Debian servers.

### Quick Start

```bash
# No dependencies required! Just run directly:
./hytale-cf --help

# Or install with pip (zero deps)
pip install -e .

# Optional: Install click+rich for prettier output
pip install -e ".[pretty]"

# Configure
hytale-cf config --api-key YOUR_CURSEFORGE_API_KEY
hytale-cf config --game-path /path/to/hytale

# Search for mods
hytale-cf search magic

# Install a mod
hytale-cf install 12345

# List installed mods
hytale-cf list

# Update all mods
hytale-cf update
```

### CLI Commands

| Command | Description |
|---------|-------------|
| `hytale-cf search <query>` | Search for mods, worlds, prefabs |
| `hytale-cf install <id>` | Install a mod by ID |
| `hytale-cf remove <id>` | Remove an installed mod |
| `hytale-cf list` | List installed mods |
| `hytale-cf info <id>` | Show mod details |
| `hytale-cf update` | Update all mods |
| `hytale-cf config` | Configure API key and game path |

### CLI Options

```bash
# Search in specific category
hytale-cf search -c worlds "adventure"
hytale-cf search -c mods -n 20 "magic"

# Skip confirmations (for scripts)
hytale-cf install -y 12345
hytale-cf remove -y 12345
hytale-cf update -y

# Verbose output
hytale-cf list -v
```

---

## GUI Usage

For desktop users who prefer a graphical interface.

### Installation

```bash
# Install GUI dependencies
pip install -r requirements.txt

# Run the GUI
python3 main.py
```

---

## Setup Guide

### 1. Obtaining a CurseForge API Key

CurseForge requires an API Key to fetch mod data.

1. Go to the [CurseForge for Studios](https://console.curseforge.com/#/) portal.
2. Log in with your CurseForge account.
3. Set the organization name.
4. Once logged in, click on the **API Keys** menu from the sidebar.
5. Copy the API Key.

**CLI (recommended - interactive):**
```bash
hytale-cf config --api-key-prompt
```

**CLI (alternative - use single quotes to avoid shell issues):**
```bash
hytale-cf config --api-key '$2a$10$YOUR_KEY_HERE'
```

**GUI:** Click "Set API Key" in the sidebar and paste your key.

### 2. Finding your Hytale Folder Path

The manager needs to know where Hytale is installed.

**If using the Hytale Launcher:**
1. Open the **Hytale Launcher**.
2. Go to **Settings** (gear icon).
3. Look for **"Open Directory"**.

**CLI:** Run `hytale-cf config --game-path /path/to/hytale`
**GUI:** Click "Game Folder" in the sidebar.

---

## How it Works (Auto-Sorting)

The manager automatically detects the resource type and installs to the correct subfolder:

| Type | Destination |
|------|-------------|
| Mods | `UserData/Mods` |
| Worlds | `UserData/Saves` (auto-extracted) |
| Prefabs | `prefabs` |
| Bootstrap | `bootstrap` |
| Translations | `translations` |

---

## Requirements

**CLI:**
- Python 3.8+ (no external dependencies!)
- Optional: click + rich for prettier output

**GUI:**
- Python 3.8+
- PySide6

**Future TUI:**
- textual (planned)

---

## Project Structure

```
.
├── hytale-cf           # CLI entry point
├── cli/                # CLI implementation
├── curseforge/         # API client (shared)
├── tui/                # Future TUI (textual)
├── ui/                 # GUI (PySide6)
├── main.py             # GUI entry point
├── requirements.txt    # GUI dependencies
└── requirements-cli.txt # CLI dependencies
```

---

## License

MIT License
