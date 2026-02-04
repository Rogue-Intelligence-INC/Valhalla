#!/usr/bin/env python3
"""Val-OS v5 - Full interactive system with keyboard support"""

code = bytearray()

# ═══════════════════════════════════════════════════════════════
# BOOT SECTOR (First 512 bytes)
# ═══════════════════════════════════════════════════════════════

# Jump over BPB
code.extend([0xEB, 0x3C, 0x90])  # jmp short 0x3E; nop

# Fake BPB for compatibility
code.extend(b'VALOS5.0')  # OEM name
code.extend([0] * 51)  # Rest of BPB area

# Start code at 0x3E (62)
# Set up segments
code.extend([0xFA])  # cli
code.extend([0x31, 0xC0])  # xor ax, ax
code.extend([0x8E, 0xD8])  # mov ds, ax
code.extend([0x8E, 0xC0])  # mov es, ax
code.extend([0x8E, 0xD0])  # mov ss, ax
code.extend([0xBC, 0x00, 0x7C])  # mov sp, 0x7C00
code.extend([0xFB])  # sti

# Save boot drive at 0x7DFE (just before stack)
code.extend([0x88, 0x16, 0xFE, 0x7D])  # mov [0x7DFE], dl

# Load more sectors
code.extend([0xB4, 0x02])  # mov ah, 2 (read sectors)
code.extend([0xB0, 0x20])  # mov al, 32 (32 sectors)
code.extend([0xB5, 0x00])  # mov ch, 0 (cylinder)
code.extend([0xB1, 0x02])  # mov cl, 2 (sector 2)
code.extend([0xB6, 0x00])  # mov dh, 0 (head)
code.extend([0x8A, 0x16, 0xFE, 0x7D])  # mov dl, [0x7DFE]
code.extend([0xBB, 0x00, 0x7E])  # mov bx, 0x7E00
code.extend([0xCD, 0x13])  # int 0x13

# Jump to second stage
code.extend([0xEA, 0x00, 0x7E, 0x00, 0x00])  # jmp 0x0000:0x7E00

# Pad to 510 bytes
while len(code) < 510:
    code.append(0)

# Boot signature
code.extend([0x55, 0xAA])

# ═══════════════════════════════════════════════════════════════
# SECOND STAGE (starts at 0x7E00, sector 2+)
# ═══════════════════════════════════════════════════════════════

# Set video mode (80x25 color text)
code.extend([0xB8, 0x03, 0x00])  # mov ax, 0x0003
code.extend([0xCD, 0x10])  # int 0x10

# Clear screen with blue background
code.extend([0x06])  # push es
code.extend([0xB8, 0x00, 0xB8])  # mov ax, 0xB800
code.extend([0x8E, 0xC0])  # mov es, ax
code.extend([0x31, 0xFF])  # xor di, di
code.extend([0xB8, 0x20, 0x1F])  # mov ax, 0x1F20 (white on blue)
code.extend([0xB9, 0xD0, 0x07])  # mov cx, 2000
code.extend([0xF3, 0xAB])  # rep stosw
code.extend([0x07])  # pop es

def emit_print(text, row, col, attr=0x1F):
    """Emit code to print text at position"""
    global code
    offset = (row * 80 + col) * 2
    
    code.extend([0x06])  # push es
    code.extend([0xB8, 0x00, 0xB8])  # mov ax, 0xB800
    code.extend([0x8E, 0xC0])  # mov es, ax
    code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])  # mov di, offset
    
    for ch in text:
        code.extend([0xB8, ord(ch), attr])  # mov ax, attr:char
        code.extend([0xAB])  # stosw
    
    code.extend([0x07])  # pop es

# ═══════════════════════════════════════════════════════════════
# LOGIN SCREEN
# ═══════════════════════════════════════════════════════════════

emit_print("=" * 60, 2, 10, 0x1B)
emit_print("Val-OS v5.0", 4, 34, 0x1F)
emit_print("Wave + Fate + Quattro", 5, 29, 0x1E)
emit_print("=" * 60, 7, 10, 0x1B)
emit_print("LOGIN", 9, 37, 0x1F)
emit_print("Username: admin", 11, 28, 0x1E)
emit_print("Password: ****", 12, 28, 0x1E)
emit_print(">>> Press ENTER <<<", 14, 30, 0x1A)

# Wait for ENTER key
login_loop = len(code)
code.extend([0xB4, 0x00])  # mov ah, 0
code.extend([0xCD, 0x16])  # int 0x16
code.extend([0x3C, 0x0D])  # cmp al, 0x0D
offset = login_loop - len(code) - 2
code.extend([0x75, offset & 0xFF])  # jne login_loop

# ═══════════════════════════════════════════════════════════════
# DESKTOP (jump target)
# ═══════════════════════════════════════════════════════════════

desktop_start = len(code)

# Clear screen
code.extend([0x06])
code.extend([0xB8, 0x00, 0xB8])
code.extend([0x8E, 0xC0])
code.extend([0x31, 0xFF])
code.extend([0xB8, 0x20, 0x17])
code.extend([0xB9, 0xD0, 0x07])
code.extend([0xF3, 0xAB])
code.extend([0x07])

emit_print("Val-OS Desktop", 0, 33, 0x1F)
emit_print("[F] Files", 3, 2, 0x1E)
emit_print("[T] Terminal", 6, 2, 0x1A)
emit_print("[B] Browser", 9, 2, 0x1C)
emit_print("[S] Settings", 12, 2, 0x1D)
emit_print(" " * 80, 24, 0, 0x70)
emit_print("Wave | F T B S | 12:34", 24, 2, 0x70)

# Desktop key loop with FAR jumps
desktop_loop = len(code)

code.extend([0xB4, 0x00])
code.extend([0xCD, 0x16])

# Check 'B' - use near jump with placeholder
code.extend([0x3C, ord('B')])
code.extend([0x74, 0x04])  # je +4
code.extend([0x3C, ord('b')])
code.extend([0x75, 0x03])  # jne skip
code.extend([0xE9, 0x00, 0x00])  # jmp browser (will patch)
browser_jmp = len(code) - 2

# Check 'T'
code.extend([0x3C, ord('T')])
code.extend([0x74, 0x04])
code.extend([0x3C, ord('t')])
code.extend([0x75, 0x03])
code.extend([0xE9, 0x00, 0x00])
terminal_jmp = len(code) - 2

# Check 'F'
code.extend([0x3C, ord('F')])
code.extend([0x74, 0x04])
code.extend([0x3C, ord('f')])
code.extend([0x75, 0x03])
code.extend([0xE9, 0x00, 0x00])
files_jmp = len(code) - 2

# Check 'S'
code.extend([0x3C, ord('S')])
code.extend([0x74, 0x04])
code.extend([0x3C, ord('s')])
code.extend([0x75, 0x03])
code.extend([0xE9, 0x00, 0x00])
settings_jmp = len(code) - 2

# Loop back
offset = desktop_loop - len(code) - 2
code.extend([0xEB, offset & 0xFF])

# ═══════════════════════════════════════════════════════════════
# BROWSER
# ═══════════════════════════════════════════════════════════════

browser_addr = len(code)
rel = browser_addr - browser_jmp - 2
code[browser_jmp] = rel & 0xFF
code[browser_jmp + 1] = (rel >> 8) & 0xFF

emit_print("+----------------------------------------------------------+", 3, 10, 0x3F)
emit_print("| Browser - wave://home                                [X] |", 4, 10, 0x3F)
emit_print("+----------------------------------------------------------+", 5, 10, 0x3F)
emit_print("|                                                          |", 6, 10, 0x3F)
emit_print("|   Welcome to Val-OS Browser!                             |", 7, 10, 0x3E)
emit_print("|   Powered by Wave Language + Fate Engine                 |", 9, 10, 0x3B)
emit_print("|                                                          |", 11, 10, 0x3F)
emit_print("|              Press any key to close                      |", 14, 10, 0x38)
emit_print("+----------------------------------------------------------+", 16, 10, 0x3F)

code.extend([0xB4, 0x00, 0xCD, 0x16])  # Wait key
rel = desktop_start - len(code) - 3
code.extend([0xE9, rel & 0xFF, (rel >> 8) & 0xFF])  # jmp desktop

# ═══════════════════════════════════════════════════════════════
# TERMINAL
# ═══════════════════════════════════════════════════════════════

terminal_addr = len(code)
rel = terminal_addr - terminal_jmp - 2
code[terminal_jmp] = rel & 0xFF
code[terminal_jmp + 1] = (rel >> 8) & 0xFF

emit_print("+----------------------------------------------------------+", 3, 10, 0x0F)
emit_print("| Terminal - Wave Shell                                [X] |", 4, 10, 0x0F)
emit_print("+----------------------------------------------------------+", 5, 10, 0x0F)
emit_print("|                                                          |", 6, 10, 0x0F)
emit_print("| Wave Shell v5.0                                          |", 7, 10, 0x0A)
emit_print("| admin@valos:~$ _                                         |", 9, 10, 0x0A)
emit_print("|                                                          |", 11, 10, 0x0F)
emit_print("|              Press any key to close                      |", 14, 10, 0x08)
emit_print("+----------------------------------------------------------+", 16, 10, 0x0F)

code.extend([0xB4, 0x00, 0xCD, 0x16])
rel = desktop_start - len(code) - 3
code.extend([0xE9, rel & 0xFF, (rel >> 8) & 0xFF])

# ═══════════════════════════════════════════════════════════════
# FILES
# ═══════════════════════════════════════════════════════════════

files_addr = len(code)
rel = files_addr - files_jmp - 2
code[files_jmp] = rel & 0xFF
code[files_jmp + 1] = (rel >> 8) & 0xFF

emit_print("+----------------------------------------------------------+", 3, 10, 0x6F)
emit_print("| Files - /home/admin                                  [X] |", 4, 10, 0x6F)
emit_print("+----------------------------------------------------------+", 5, 10, 0x6F)
emit_print("|                                                          |", 6, 10, 0x6F)
emit_print("|  [D] Documents   [D] Downloads   [D] Pictures            |", 7, 10, 0x6E)
emit_print("|  [F] hello.wave  [F] readme.md                           |", 9, 10, 0x6A)
emit_print("|                                                          |", 11, 10, 0x6F)
emit_print("|              Press any key to close                      |", 14, 10, 0x68)
emit_print("+----------------------------------------------------------+", 16, 10, 0x6F)

code.extend([0xB4, 0x00, 0xCD, 0x16])
rel = desktop_start - len(code) - 3
code.extend([0xE9, rel & 0xFF, (rel >> 8) & 0xFF])

# ═══════════════════════════════════════════════════════════════
# SETTINGS
# ═══════════════════════════════════════════════════════════════

settings_addr = len(code)
rel = settings_addr - settings_jmp - 2
code[settings_jmp] = rel & 0xFF
code[settings_jmp + 1] = (rel >> 8) & 0xFF

emit_print("+----------------------------------------------------------+", 3, 10, 0x5F)
emit_print("| Settings                                             [X] |", 4, 10, 0x5F)
emit_print("+----------------------------------------------------------+", 5, 10, 0x5F)
emit_print("|                                                          |", 6, 10, 0x5F)
emit_print("|  [>] Display   [>] Sound   [>] Network                   |", 7, 10, 0x5E)
emit_print("|  [>] Fate Engine   [>] Quattro Memory                    |", 9, 10, 0x5D)
emit_print("|                                                          |", 11, 10, 0x5F)
emit_print("|  Val-OS v5.0 - Wave Language                             |", 13, 10, 0x5B)
emit_print("|              Press any key to close                      |", 14, 10, 0x58)
emit_print("+----------------------------------------------------------+", 16, 10, 0x5F)

code.extend([0xB4, 0x00, 0xCD, 0x16])
rel = desktop_start - len(code) - 3
code.extend([0xE9, rel & 0xFF, (rel >> 8) & 0xFF])

# Pad to multiple of 512 bytes
while len(code) % 512 != 0:
    code.append(0)

# Write image
with open('val-os-v5/build/valos_interactive.img', 'wb') as f:
    f.write(bytes(code))

print(f"Created val-os-v5/build/valos_interactive.img")
print(f"Size: {len(code)} bytes ({len(code)//512} sectors)")
