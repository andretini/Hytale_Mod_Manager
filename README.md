# Hytale Mod Manager

A custom Dioxus client for managing Hytale resources via the CurseForge or MODTALE API. This tool automates the organization of mods.

This manager was built because CurseForge provides no native support for Hytale on Linux.

---

## Installation Guide

### Linux
I provided two formats for Linux:

* **AppImage:**
    1.  Download the file.
    2.  Right-click it → **Properties** → **Permissions**.
    3.  Check **"Allow executing file as program"**, then double-click to run.

* **Flatpak:**
    If you prefer Flatpak, you may need to add the Flathub repository first to ensure all dependencies (runtimes) are available:
    ```bash
      # 1. Add Flathub repo
      flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
  
      # 2. Install the app
      flatpak install HytaleModManager.flatpak
    ```

### Windows
1.  Download the **Windows EXE**.
2.  **Note:** Because this app is not currently signed with a Microsoft certificate, you might see a "Windows protected your PC" popup.
    * Click **More Info** → **Run Anyway** to proceed.

### macOS
1.  Download the **DMG** file and open it.
2.  Drag the app into your **Applications** folder in Finder.
3.  **Note:** Because I do not use a paid Apple Developer account, macOS will block the app initially. To fix this:
    * Try to open the app once (it will say it cannot be opened).
    * Go to **System Settings** → **Privacy & Security**.
    * Scroll down to the Security section and click **Open Anyway** next to the Hytale Mod Manager notification.

---

## 🚀 Setup Guide

### 1. Obtaining a CurseForge API Key
CurseForge requires an API Key to fetch mod data.
1.  Go to the [CurseForge for Studios](https://console.curseforge.com/#/) portal.
2.  Log in with your CurseForge/Overwolf account.
3.  Click on **"Create New App"**.
4.  Give your app a name (e.g., `MyHytaleManager`).
5.  Once created, copy the **API Key** from the dashboard.
6.  In the Hytale Mod Manager, click **🔑 Set API Key** in the sidebar and paste your key.

### 2. Finding your Hytale Folder Path
The manager needs to know where Hytale is installed to sort your files correctly.

#### **If using the Hytale Launcher:**
1.  Open the **Hytale Launcher**.
2.  Go to **Settings** (usually a gear icon ⚙️).
3.  Look for **"Open Directory"**.

### 3. Applying the Path
1.  Open the Manager and click **⚙ Game Folder** at the bottom of the sidebar.
2.  Navigate to the path found in the step above.
3.  Ensure you select the **root Hytale folder** (the one containing the `UserData` folder).

---

## 📂 How it Works (Auto-Sorting)
The manager automatically detects the resource type and appends the correct subfolder:
* **Mods:** Sorted into `UserData/Mods`

---

## 🛠 Tech Stack
* **Language:** Rust 1.92
* **UI Framework:** Dioxus
* **API:** CurseForge | MODTALE
* **Platform:** Any, focus on Linux

# Hytale Mod Manager

A custom Dioxus client for managing Hytale resources via the CurseForge or MODTALE API. This tool automates the organization of mods.

This manager was built because CurseForge provides no native support for Hytale on Linux.

---

## 🚀 Setup Guide

### 1. Obtaining a CurseForge API Key
CurseForge requires an API Key to fetch mod data.
1.  Go to the [CurseForge for Studios](https://console.curseforge.com/#/) portal.
2.  Log in with your CurseForge/Overwolf account.
3.  Click on **"Create New App"**.
4.  Give your app a name (e.g., `MyHytaleManager`).
5.  Once created, copy the **API Key** from the dashboard.
6.  In the Hytale Mod Manager, click **🔑 Set API Key** in the sidebar and paste your key.

### 2. Finding your Hytale Folder Path
The manager needs to know where Hytale is installed to sort your files correctly.

#### **If using the Hytale Launcher:**
1.  Open the **Hytale Launcher**.
2.  Go to **Settings** (usually a gear icon ⚙️).
3.  Look for **"Open Directory"**.

### 3. Applying the Path
1.  Open the Manager and click **⚙ Game Folder** at the bottom of the sidebar.
2.  Navigate to the path found in the step above.
3.  Ensure you select the **root Hytale folder** (the one containing the `UserData` folder).

---

## 📂 How it Works (Auto-Sorting)
The manager automatically detects the resource type and appends the correct subfolder:
* **Mods:** Sorted into `UserData/Mods`

---

## 🛠 Tech Stack
* **Language:** Rust 1.92
* **UI Framework:** Dioxus
* **API:** CurseForge | MODTALE
* **Platform:** Any, focus on Linux

---

## How to Run the Manager

### 1. Install Requirements

Make sure you have:

- **Rust** (recommended via rustup)  
  ```bash
    rustc --version
    cargo --version

### 1. Run Application
  ```bash
    cargo run --release



