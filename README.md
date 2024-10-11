

# Slick Cmd

[![CI](https://github.com/johnlng/slickcmd/actions/workflows/ci.yml/badge.svg)](https://github.com/johnlng/slickcmd/actions)
![Static Badge](https://img.shields.io/badge/os-Windows-blue)

**Slick Cmd** is a lightweight utility designed to supercharge your Windows Command Prompt experience by making directory navigation, path completion, and command management more efficient. Slick Cmd runs quietly in your system tray, enabling you to access its powerful features through intuitive keyboard shortcuts.

## ✨ Features

- **Streamlined Directory Navigation:**
    - `Alt + Up` — Move to the parent directory.
    - `Alt + Down` — Display a popup with a list of subdirectories for quick access.
    - `Alt + Left` — Navigate backward to the previous directory.
    - `Alt + Right` — Navigate forward to the next directory.
    - `Alt + Home` — Jump to the home directory.
    - `Alt + End` — View and navigate to recent directories.

- **Smart Path Completion:**
    - While typing a `cd` command, a path completion list appears, helping you quickly select from available directories.
    - `Esc` — Close the path completion list.
    - `Tab` — Accept the selected path without executing `cd`.
    - `Enter` — Accept the selected path and `cd` into it immediately.

- **Command History Management:**
    - `Alt + F7` — Open the command history dialog to select from previously used commands.
    - `Enter` — Place the selected command in the command line for review or modification.
    - `Ctrl + Enter` — Execute the selected command immediately.
    - Command history persists across sessions, so you can always refer back to previous commands.

- **Auto-Correct for `cd`:**
    - Automatically adds the `/d` flag when switching to a directory on another drive, eliminating the need to remember it manually.

- **Quick Screen Clearing:**
    - `Ctrl + L` — Clear the screen with a single keystroke, running the `cls` command automatically.

## ⚙️ Requirements

* **Operating System:** Windows 10 or above, 64-bit
* **Shell Compatibility:** Classic Command Prompt(cmd.exe) or PowerShell(powershell.exe or pwsh.exe)

## 📖 Installation & Getting Started

1. **Download:** Get the latest version of Slick Cmd from the [Releases page](https://github.com/johnlng/slickcmd/releases).
2. **Install:** Extract the release zip to a preferred installation folder on your local filesystem.
3. **Launch:** Start Slick Cmd by double-clicking `slickcmd.exe`. A small icon will appear in your system tray, indicating it’s running.
4. **Boost Your Productivity:** Open a command prompt window, use the keyboard shortcuts listed above to navigate directories and manage commands effortlessly!

> [!NOTE]  
> Windows Security might flag the downloaded binaries as potential threats, but this is a false positive.
> If you're concerned, you can always build the project from the source code yourself.
> 
> If you encounter an error message like "VCRUNTIME140.dll was not found" when running slickcmd.exe, 
> please download and install the [Visual C++ Runtime Libraries](https://aka.ms/vs/17/release/vc_redist.x64.exe) to resolve the issue. 

## 🛠️ Build from source
```
git clone https://github.com/johnlng/slickcmd.git
cd slickcmd
cargo build
```
You can find the binaries in the target folder.

## 📜 License

Slick Cmd is distributed under the [MIT License](https://github.com/johnlng/slickcmd?tab=MIT-1-ov-file).

## 💡 Contribute & Support

I’d love to hear your thoughts! If you have suggestions, feature requests, or find bugs, please [open an issue on the GitHub repository](https://github.com/johnlng/slickcmd/issues).

If you like the project, please consider giving it a ⭐ and sharing it with others to show your support!

