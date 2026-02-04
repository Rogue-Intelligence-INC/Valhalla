---
title: "Val-OS v2.3: Desktop Environment Released"
date: "2026-01-15"
excerpt: "The latest Val-OS release includes full desktop environment, multi-user support, mouse support, and an integrated app store."
author: "Rogue Intelligence Team"
tags: ["Val-OS", "Operating System", "Desktop", "Release"]
---

# Val-OS v2.3: Desktop Environment Released

We're thrilled to announce **Val-OS v2.3**, the most significant update to our experimental operating system yet.

## What's New in v2.3

### Full Desktop Environment

Val-OS now includes a complete desktop environment with:

- **Window Management**: Drag, resize, minimize, maximize
- **Taskbar**: Running applications and quick launch
- **System Tray**: Network, volume, battery indicators
- **Desktop Icons**: Quick access to applications

### System Panel (Super Key)

Press the Super key to access the new system panel:

```
┌────────────────────────────────────────────────────┐
│              ⏰ 14:32                               │
│              2026-01-15 Monday                      │
├────────────────────────────────────────────────────┤
│  [WiFi] [Bluetooth] [DND] [Theme]                  │
├────────────────────────────────────────────────────┤
│  🔊 ━━━━━━━━━━━●━━━━━━━ 70%                        │
│  🔆 ━━━━━━━━━━━━━━●━━━ 80%                         │
└────────────────────────────────────────────────────┘
```

### Multi-User Support

- Login screen with user/password authentication
- User switching
- Home directories per user
- Admin/user permission levels

### Mouse Support

Full mouse support including:
- Smooth cursor movement
- Left/right click
- Scroll wheel
- Drag and drop

### Built-in Applications

| App | Function |
|-----|----------|
| 📝 Notebook | Text editing with syntax highlighting |
| 🎵 Music Player | Playlists, progress bar |
| 🎬 Video Player | Fullscreen, subtitles |
| 💣 Minesweeper | Three difficulty levels |
| 🌐 Browser | Multi-tab, bookmarks |
| 📦 App Store | Search and install apps |

## The Hybrid Interface Advantage

What makes Val-OS unique is that all of this runs in a **hybrid terminal interface**:

- Works over SSH
- Runs on minimal hardware
- Near-zero latency
- Full 24-bit color support

## Try It Out

Val-OS v2.3 is available now. Build with the Wave compiler:

```bash
cd wave
./bin/wave ../val-os/valos.wave -o valos
./valos
```

We're excited to see what you build with Val-OS!
