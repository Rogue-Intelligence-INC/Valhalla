#!/usr/bin/env python3
"""
Val-OS v5.0 - Full GUI System
==============================
图形模式 + 鼠标 + 应用程序 + 游戏
"""

code = bytearray()
jump_patches = []  # 存储需要修补的跳转

def emit_near_jmp(target_label):
    """发出 near jump 并记录需要修补的位置"""
    global code, jump_patches
    code.extend([0xE9, 0x00, 0x00])  # jmp near (3 bytes)
    jump_patches.append((len(code) - 2, target_label))

def patch_jumps(labels):
    """修补所有跳转"""
    global code, jump_patches
    for patch_pos, label in jump_patches:
        if label in labels:
            target = labels[label]
            rel = target - patch_pos - 2
            code[patch_pos] = rel & 0xFF
            code[patch_pos + 1] = (rel >> 8) & 0xFF

labels = {}

# ═══════════════════════════════════════════════════════════════════
# BOOT SECTOR
# ═══════════════════════════════════════════════════════════════════

code.extend([0xEB, 0x3C, 0x90])
code.extend(b'VALOS5.0')
code.extend([0] * 51)

# Setup
code.extend([0xFA, 0x31, 0xC0, 0x8E, 0xD8, 0x8E, 0xC0, 0x8E, 0xD0])
code.extend([0xBC, 0x00, 0x7C, 0xFB])

# Save boot drive
code.extend([0x88, 0x16, 0xFE, 0x7D])

# Load sectors
code.extend([0xB4, 0x02, 0xB0, 0x40, 0xB5, 0x00, 0xB1, 0x02, 0xB6, 0x00])
code.extend([0x8A, 0x16, 0xFE, 0x7D])
code.extend([0xBB, 0x00, 0x7E, 0xCD, 0x13])

# Jump to stage 2
code.extend([0xEA, 0x00, 0x7E, 0x00, 0x00])

while len(code) < 510:
    code.append(0)
code.extend([0x55, 0xAA])

# ═══════════════════════════════════════════════════════════════════
# STAGE 2 - Main System
# ═══════════════════════════════════════════════════════════════════

labels['stage2'] = len(code)

# Set Mode 13h (320x200x256)
code.extend([0xB8, 0x13, 0x00, 0xCD, 0x10])

# Init mouse
code.extend([0xB8, 0x00, 0x00, 0xCD, 0x33])  # Reset
code.extend([0xB8, 0x01, 0x00, 0xCD, 0x33])  # Show

# ─────────────────────────────────────────────────────────────────
# DRAW DESKTOP
# ─────────────────────────────────────────────────────────────────

labels['draw_desktop'] = len(code)

# Draw gradient background
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0, 0x31, 0xFF])
for y in range(200):
    color = 16 + min(y // 10, 15)
    code.extend([0xB0, color, 0xB9, 0x40, 0x01, 0xF3, 0xAA])
code.extend([0x07])

# Draw taskbar (bottom 12 pixels, dark gray)
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
for y in range(188, 200):
    offset = y * 320
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x08, 0xB9, 0x40, 0x01, 0xF3, 0xAA])
code.extend([0x07])

# Draw icons (simple colored squares)
def draw_rect(x, y, w, h, color):
    global code
    code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
    for row in range(h):
        offset = (y + row) * 320 + x
        code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
        code.extend([0xB0, color, 0xB9, w, 0x00, 0xF3, 0xAA])
    code.extend([0x07])

# Desktop icons
draw_rect(10, 10, 24, 24, 0x2E)   # Files - green
draw_rect(10, 45, 24, 24, 0x1F)   # Terminal - blue
draw_rect(10, 80, 24, 24, 0x4C)   # Browser - red
draw_rect(10, 115, 24, 24, 0x3F)  # Settings - cyan
draw_rect(10, 150, 24, 24, 0x5E)  # Game - magenta

# Wave logo in taskbar
draw_rect(5, 190, 30, 8, 0x09)

# Print text using BIOS
def print_text(text, row, col, attr=0x0F):
    global code
    code.extend([0xB4, 0x02, 0xB7, 0x00, 0xB6, row, 0xB2, col, 0xCD, 0x10])
    for ch in text:
        code.extend([0xB4, 0x09, 0xB0, ord(ch), 0xB3, attr, 0xB9, 0x01, 0x00, 0xCD, 0x10])
        col += 1
        code.extend([0xB4, 0x02, 0xB7, 0x00, 0xB6, row, 0xB2, col, 0xCD, 0x10])

print_text("Val-OS", 0, 17, 0x1F)
print_text("Files", 5, 1, 0x0A)
print_text("Term", 8, 1, 0x0B)
print_text("Web", 11, 1, 0x0C)
print_text("Set", 14, 1, 0x0E)
print_text("Game", 19, 1, 0x0D)
print_text("Wave", 24, 1, 0x09)
print_text("L=Menu B=Web G=Game Q=Exit", 24, 8, 0x07)

# ─────────────────────────────────────────────────────────────────
# MAIN LOOP
# ─────────────────────────────────────────────────────────────────

labels['main_loop'] = len(code)

# Check keyboard
code.extend([0xB4, 0x01, 0xCD, 0x16])
code.extend([0x74, 0x30])  # jz check_mouse

# Get key
code.extend([0xB4, 0x00, 0xCD, 0x16])

# 'Q' quit
code.extend([0x3C, ord('q'), 0x74, 0x00])
quit_patch = len(code) - 1

code.extend([0x3C, ord('Q'), 0x74, 0x00])
quit_patch2 = len(code) - 1

# 'G' game
code.extend([0x3C, ord('g'), 0x74, 0x00])
game_patch = len(code) - 1

code.extend([0x3C, ord('G'), 0x74, 0x00])
game_patch2 = len(code) - 1

# 'B' browser
code.extend([0x3C, ord('b'), 0x74, 0x00])
browser_patch = len(code) - 1

code.extend([0x3C, ord('B'), 0x74, 0x00])
browser_patch2 = len(code) - 1

# 'L' launcher
code.extend([0x3C, ord('l'), 0x74, 0x00])
launcher_patch = len(code) - 1

code.extend([0x3C, ord('L'), 0x74, 0x00])
launcher_patch2 = len(code) - 1

# check_mouse:
labels['check_mouse'] = len(code)

# Get mouse status
code.extend([0xB8, 0x03, 0x00, 0xCD, 0x33])
# CX=X, DX=Y, BX=buttons

# Store mouse position
code.extend([0x89, 0x0E, 0x00, 0x80])  # [0x8000] = X
code.extend([0x89, 0x16, 0x02, 0x80])  # [0x8002] = Y

# Draw cursor (white pixel)
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
# Calculate offset: Y * 320 + X
code.extend([0x89, 0xD0])  # mov ax, dx (Y)
code.extend([0xBB, 0x40, 0x01])  # mov bx, 320
code.extend([0xF7, 0xE3])  # mul bx
code.extend([0x01, 0xC8])  # add ax, cx (X)
code.extend([0x89, 0xC7])  # mov di, ax
code.extend([0xB0, 0x0F])  # white
code.extend([0x26, 0x88, 0x05])  # mov [es:di], al
code.extend([0x07])

# Delay
code.extend([0xB9, 0xFF, 0x0F, 0xE2, 0xFE])

# Loop back
offset = labels['main_loop'] - len(code) - 2
code.extend([0xEB, offset & 0xFF])

# ═══════════════════════════════════════════════════════════════════
# GAME - Mario Style
# ═══════════════════════════════════════════════════════════════════

labels['game'] = len(code)

# Sky blue background
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0, 0x31, 0xFF])
code.extend([0xB0, 0x3B, 0xB9, 0x00, 0xFA, 0xF3, 0xAA])
code.extend([0x07])

# Green ground
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
for y in range(180, 200):
    offset = y * 320
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x02, 0xB9, 0x40, 0x01, 0xF3, 0xAA])
code.extend([0x07])

# Brick platforms
for (x, y, w) in [(80, 140, 40), (150, 120, 40), (220, 100, 40)]:
    code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
    for row in range(8):
        offset = (y + row) * 320 + x
        code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
        code.extend([0xB0, 0x06, 0xB9, w, 0x00, 0xF3, 0xAA])
    code.extend([0x07])

# Question blocks (yellow)
for (x, y) in [(100, 100), (180, 80)]:
    code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
    for row in range(16):
        offset = (y + row) * 320 + x
        code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
        code.extend([0xB0, 0x0E, 0xB9, 0x10, 0x00, 0xF3, 0xAA])
    code.extend([0x07])

# Clouds (white)
for (x, y) in [(50, 15), (150, 25), (250, 10)]:
    code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
    for row in range(8):
        offset = (y + row) * 320 + x
        code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
        code.extend([0xB0, 0x0F, 0xB9, 0x20, 0x00, 0xF3, 0xAA])
    code.extend([0x07])

# Player (Mario-like)
px, py = 40, 162
# Head (skin)
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
for row in range(6):
    offset = (py + row) * 320 + px + 2
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x2C, 0xB9, 0x08, 0x00, 0xF3, 0xAA])
# Body (red)
for row in range(6, 12):
    offset = (py + row) * 320 + px
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x04, 0xB9, 0x0C, 0x00, 0xF3, 0xAA])
# Legs (blue)
for row in range(12, 16):
    offset = (py + row) * 320 + px + 1
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x01, 0xB9, 0x04, 0x00, 0xF3, 0xAA])
    offset = (py + row) * 320 + px + 7
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x01, 0xB9, 0x04, 0x00, 0xF3, 0xAA])
code.extend([0x07])

# Game title
print_text("WAVE MARIO - ESC=Exit", 0, 5, 0x0E)

# Game loop
labels['game_loop'] = len(code)
code.extend([0xB4, 0x01, 0xCD, 0x16, 0x74, 0x08])
code.extend([0xB4, 0x00, 0xCD, 0x16])
code.extend([0x3C, 0x1B, 0x74, 0x00])  # ESC
game_exit_patch = len(code) - 1

code.extend([0xB9, 0xFF, 0x1F, 0xE2, 0xFE])
offset = labels['game_loop'] - len(code) - 2
code.extend([0xEB, offset & 0xFF])

labels['game_exit'] = len(code)
# Return to desktop
code.extend([0xB8, 0x13, 0x00, 0xCD, 0x10])
emit_near_jmp('draw_desktop')

# ═══════════════════════════════════════════════════════════════════
# BROWSER
# ═══════════════════════════════════════════════════════════════════

labels['browser'] = len(code)

# Window background (white)
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
for y in range(20, 170):
    offset = y * 320 + 30
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x0F, 0xB9, 0x04, 0x01, 0xF3, 0xAA])
# Title bar (blue)
for y in range(20, 30):
    offset = y * 320 + 30
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x01, 0xB9, 0x04, 0x01, 0xF3, 0xAA])
# Address bar
for y in range(30, 38):
    offset = y * 320 + 35
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x07, 0xB9, 0xFA, 0x00, 0xF3, 0xAA])
code.extend([0x07])

print_text("Wave Browser", 3, 5, 0x1F)
print_text("wave://home", 4, 6, 0x08)
print_text("Welcome to Val-OS!", 7, 8, 0x00)
print_text("Powered by Wave+Fate", 9, 8, 0x02)
print_text("ESC = Close", 18, 10, 0x08)

# Browser loop
labels['browser_loop'] = len(code)
code.extend([0xB4, 0x00, 0xCD, 0x16])
code.extend([0x3C, 0x1B, 0x74, 0x00])
browser_exit_patch = len(code) - 1
offset = labels['browser_loop'] - len(code) - 2
code.extend([0xEB, offset & 0xFF])

labels['browser_exit'] = len(code)
code.extend([0xB8, 0x13, 0x00, 0xCD, 0x10])
emit_near_jmp('draw_desktop')

# ═══════════════════════════════════════════════════════════════════
# LAUNCHER
# ═══════════════════════════════════════════════════════════════════

labels['launcher'] = len(code)

# Dark overlay + menu box
code.extend([0x06, 0xB8, 0x00, 0xA0, 0x8E, 0xC0])
for y in range(40, 160):
    offset = y * 320 + 50
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x00, 0xB9, 0xDC, 0x00, 0xF3, 0xAA])
# Border
for y in [40, 159]:
    offset = y * 320 + 50
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
    code.extend([0xB0, 0x0F, 0xB9, 0xDC, 0x00, 0xF3, 0xAA])
code.extend([0x07])

print_text("=== APP LAUNCHER ===", 6, 10, 0x0F)
print_text("1. Files", 8, 12, 0x0A)
print_text("2. Terminal", 9, 12, 0x0B)
print_text("3. Browser", 10, 12, 0x0C)
print_text("4. Settings", 11, 12, 0x0E)
print_text("5. Game", 12, 12, 0x0D)
print_text("6. Calculator", 13, 12, 0x09)
print_text("ESC = Close", 16, 12, 0x07)

# Launcher loop
labels['launcher_loop'] = len(code)
code.extend([0xB4, 0x00, 0xCD, 0x16])

# ESC
code.extend([0x3C, 0x1B, 0x74, 0x00])
launcher_exit_patch = len(code) - 1

# '5' for game
code.extend([0x3C, ord('5'), 0x74, 0x00])
launcher_game_patch = len(code) - 1

# '3' for browser
code.extend([0x3C, ord('3'), 0x74, 0x00])
launcher_browser_patch = len(code) - 1

offset = labels['launcher_loop'] - len(code) - 2
code.extend([0xEB, offset & 0xFF])

labels['launcher_exit'] = len(code)
code.extend([0xB8, 0x13, 0x00, 0xCD, 0x10])
emit_near_jmp('draw_desktop')

labels['launcher_game'] = len(code)
emit_near_jmp('game')

labels['launcher_browser'] = len(code)
emit_near_jmp('browser')

# ═══════════════════════════════════════════════════════════════════
# QUIT
# ═══════════════════════════════════════════════════════════════════

labels['quit'] = len(code)
code.extend([0xB8, 0x03, 0x00, 0xCD, 0x10])
print_text("Val-OS Shutdown", 12, 20, 0x0F)
print_text("Thank you!", 14, 23, 0x0A)
code.extend([0xEB, 0xFE])

# ═══════════════════════════════════════════════════════════════════
# PATCH ALL JUMPS
# ═══════════════════════════════════════════════════════════════════

# Patch short jumps
def patch_short(patch_pos, target_label):
    global code, labels
    if target_label in labels:
        rel = labels[target_label] - patch_pos - 1
        if -128 <= rel <= 127:
            code[patch_pos] = rel & 0xFF
        else:
            print(f"Warning: {target_label} too far for short jump")

patch_short(quit_patch, 'quit')
patch_short(quit_patch2, 'quit')
patch_short(game_patch, 'game')
patch_short(game_patch2, 'game')
patch_short(browser_patch, 'browser')
patch_short(browser_patch2, 'browser')
patch_short(launcher_patch, 'launcher')
patch_short(launcher_patch2, 'launcher')
patch_short(game_exit_patch, 'game_exit')
patch_short(browser_exit_patch, 'browser_exit')
patch_short(launcher_exit_patch, 'launcher_exit')
patch_short(launcher_game_patch, 'launcher_game')
patch_short(launcher_browser_patch, 'launcher_browser')

# Patch near jumps
patch_jumps(labels)

# ═══════════════════════════════════════════════════════════════════
# FINALIZE
# ═══════════════════════════════════════════════════════════════════

while len(code) % 512 != 0:
    code.append(0)

while len(code) < 64 * 512:
    code.append(0)

with open('val-os-v5/build/valos_gui.img', 'wb') as f:
    f.write(bytes(code))

print(f"✅ Created val-os-v5/build/valos_gui.img")
print(f"   Size: {len(code)} bytes ({len(code)//512} sectors)")
print()
print("🎮 Features:")
print("   - 320x200x256 Graphics Mode")
print("   - Mouse cursor support")
print("   - Desktop with icons")
print("   - Taskbar")
print("   - App Launcher (L key)")
print("   - Browser (B key)")
print("   - Mario-style Game (G key)")
print("   - Quit (Q key)")
