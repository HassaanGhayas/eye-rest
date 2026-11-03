# EyeRest 🌿

A quiet, lightweight Windows desktop utility that reminds you to take eye breaks based on the **20-20-20 rule**: *every 20 minutes, look at something 20 feet away for 20 seconds*.

Designed as an ultra-lightweight, native alternative to heavy Electron apps like Stretchly.

---

## 🎯 Highlights & Performance

- **Installer Size**: **1.16 MB** (NSIS installer `EyeRest_0.1.0_x64-setup.exe`)
- **Standalone Binary**: **2.81 MB** (`eyerest.exe`)
- **Idle Memory**: **~9.3 MB** private bytes / **~30.8 MB** working set with periodic Win32 memory trimming
- **CPU Usage**: **0.0%** when idle
- **No Keyboard Focus Stealing**: Overlay uses Win32 `WS_EX_NOACTIVATE` so you never lose your typing focus or cursor position.
- **Privacy First**: Zero telemetry, zero network calls, zero accounts. All settings stored locally in `%APPDATA%\EyeRest\settings.json`.

---

## ✨ Features

- **System Tray Resident**: Runs quietly in the notification area with no unnecessary taskbar clutter or persistent windows.
- **Monotonic State Machine Timer**:
  - `Working` ➔ `PreBreak` ➔ `Break` ➔ `Working`
  - Uses absolute timestamp comparison rather than tick intervals, preventing drift when Windows sleeps or hibernates.
- **Gentle 10-Second Pre-Break Toast**:
  - A subtle card floats into the corner of your active monitor 10 seconds before a break, giving you time to finish typing or snooze.
- **Calm Dimmed Break Overlay**:
  - Soft 65% dimming with meditation-inspired colors (navy charcoal dark theme, warm off-white light theme).
  - Minimal circular breathing animation and high-precision countdown.
  - Gentle 432 Hz Tibetan singing bowl chime (can be toggled in Preferences).
  - Ephemeral `Esc` key shortcut registered only during breaks to dismiss quickly without stealing focus.
- **Multi-Monitor Awareness**:
  - Automatically identifies which monitor contains the active foreground window (`MonitorFromWindow`) and displays the overlay and toast strictly on that screen.
- **Intelligent Watchers**:
  - **Idle Detection**: Resets the work timer after 2 minutes of system-wide inactivity (`GetLastInputInfo`).
  - **Fullscreen & Presentation Detection**: Automatically defers eye breaks if you are gaming, watching fullscreen video, or giving a presentation (`SHQueryUserNotificationState`).
  - **Lock / Sleep Detection**: Automatically pauses when Windows locks or sleeps, and resumes on unlock.
- **Configurable Preferences**:
  - Work interval (1–120 minutes)
  - Break duration (5–300 seconds)
  - Strict mode (hides skip button and disables Esc shortcut)
  - Sound chime toggle
  - Launch on Windows startup (managed via registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
  - Theme switcher (Dark, Light, System)

---

## 🚀 Installation & Usage

### 1. Run the Installer
Run the installer generated in:
```
target\release\bundle\nsis\EyeRest_0.1.0_x64-setup.exe
```
This installs EyeRest to your local user apps with start menu shortcuts.

### 2. Portable Mode
You can also run the standalone release executable directly without installation:
```
target\release\eyerest.exe
```

### 3. System Tray Menu
Right-click the EyeRest leaf icon in the notification area (system tray) to access:
- **Take break now**: Immediately trigger an eye break.
- **Pause / Resume**: Temporarily freeze the break timer.
- **Skip next break**: Reset back to the start of the next work cycle.
- **Snooze**: Postpone for 5 minutes, 1 hour, or until 4:00 AM tomorrow.
- **Preferences...**: Open the settings window to customize intervals, strict mode, sound, and theme.
- **Quit**: Exit the application.

---

## 🛠️ Building from Source

### Prerequisites
- Windows 10 or 11 (x64)
- Rust toolchain (`rustup`)
- Node.js & npm (for Tauri CLI bundling)

### Compile the Release Binary
From the project root:
```powershell
# Set cargo environment if custom cache is used:
$env:CARGO_HOME = "d:\EyeRest\.cargo_home"

# Build optimized release binary
Set-Location src-tauri
cargo build --release
```

### Build the NSIS Windows Installer
```powershell
npm run tauri -- build --bundles nsis
```
The output installer will be saved to:
`src-tauri\target\release\bundle\nsis\EyeRest_0.1.0_x64-setup.exe`

---

## 🧪 Running Unit Tests
EyeRest includes automated unit test suites for timer state transitions, snooze logic, idle reset, and pause/resume:
```powershell
Set-Location src-tauri
cargo test --lib -- --nocapture
```

---

## 🔒 Architecture & Security

- **Backend**: Rust 2021 with Tauri 2 and direct Win32 API calls (`windows` crate).
- **Frontend**: Lightweight vanilla HTML5, CSS3, and modern ECMAScript without heavy frameworks or virtual DOM overhead.
- **Window Management**: Uses Win32 extended window styles `WS_EX_NOACTIVATE` and `SWP_NOACTIVATE` to prevent taking keyboard focus away from active applications.
- **Memory Optimization**: Automatically invokes `SetProcessWorkingSetSize` after breaks to release unused pages back to the operating system.
