# Hytale Mod Manager

A custom PySide6 client for managing Hytale resources via the CurseForge API. This tool automates the organization of mods and worlds.

This manager was built because CurseForge provides no native support for Hytale on Linux.

---

## 🚀 Setup Guide

### 1. Obtaining a CurseForge API Key
CurseForge requires an API Key to fetch mod data.
1.  Go to the [CurseForge for Studios](https://console.curseforge.com/#/) portal.
2.  Log in with your CurseForge account.
3.  Set the organization name.
4.  Once logged in, click on the **API Keys** menu from the sidebar.
5.  Copy the Api Key
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
* **Worlds:** Automatically unzipped into `UserData/Saves`

---

## 🛠 Installation & Execution

For this step you must have python3 installed, specifically 3.12.3

Follow these steps in your terminal to get the manager running:

1. **Create a virtual environment:**
   ```bash
   python3 -m venv venv
2. **Activate the environment:**
   ```bash
     venv/bin/activate
3. **Install dependencies:**
   ```bash
    pip install -r requirements.txt
4. **Run the program:**
   ```bash
    python3 main.py
