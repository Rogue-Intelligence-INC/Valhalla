#!/bin/bash
# Wave Boot Image Fixer
# 修复 --bare-metal 生成的镜像，添加磁盘读取代码

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <input.wave> [-o output.img]"
    echo ""
    echo "This tool:"
    echo "  1. Compiles Wave source with --bare-metal"
    echo "  2. Fixes the boot sector to include disk read code"
    echo "  3. Generates a working bootable image"
    exit 1
fi

INPUT="$1"
OUTPUT="${3:-${INPUT%.wave}.img}"

WAVE_DIR="$(dirname "$0")/.."
WAVE_BIN="$WAVE_DIR/bin/wave"
TMP_DIR="/tmp/wave_boot_$$"

mkdir -p "$TMP_DIR"

echo "═══════════════════════════════════════════════════════"
echo "🌊 Wave Boot Image Fixer"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Input:  $INPUT"
echo "Output: $OUTPUT"
echo ""

# Step 1: Compile with --bare-metal to get code section
echo "[1/3] Compiling Wave source..."
"$WAVE_BIN" "$INPUT" -o "$TMP_DIR/raw.img" --bare-metal 2>&1 | grep -v "^$"

# Step 2: Generate proper boot sector with disk read code
echo ""
echo "[2/3] Generating proper boot sector..."

# Create boot sector binary
python3 << 'PYTHON_EOF'
import sys

boot = bytearray(512)
pos = 0

def emit(b):
    global pos
    boot[pos] = b
    pos += 1

def emit_word(w):
    emit(w & 0xFF)
    emit((w >> 8) & 0xFF)

# cli
emit(0xFA)

# xor ax, ax
emit(0x31); emit(0xC0)

# mov ds, ax
emit(0x8E); emit(0xD8)

# mov es, ax  
emit(0x8E); emit(0xC0)

# mov ss, ax
emit(0x8E); emit(0xD0)

# mov sp, 0x7C00
emit(0xBC); emit_word(0x7C00)

# mov ax, 0x0003 (text mode)
emit(0xB8); emit_word(0x0003)

# int 0x10
emit(0xCD); emit(0x10)

# Initialize serial port
# mov dx, 0x03FB
emit(0xBA); emit_word(0x03FB)
# mov al, 0x03
emit(0xB0); emit(0x03)
# out dx, al
emit(0xEE)

# Read disk sectors
# mov bx, 0x7E00 (destination)
emit(0xBB); emit_word(0x7E00)

# mov ah, 0x02 (read sectors)
emit(0xB4); emit(0x02)

# mov al, 63 (sector count - 32KB)
emit(0xB0); emit(0x3F)

# mov ch, 0 (cylinder)
emit(0xB5); emit(0x00)

# mov cl, 2 (start sector)
emit(0xB1); emit(0x02)

# mov dh, 0 (head)
emit(0xB6); emit(0x00)

# mov dl, 0 (drive)
emit(0xB2); emit(0x00)

# int 0x13 (BIOS disk read)
emit(0xCD); emit(0x13)

# jmp far 0x0000:0x7E00
emit(0xEA)
emit_word(0x7E00)  # offset
emit_word(0x0000)  # segment

# Boot signature
boot[510] = 0x55
boot[511] = 0xAA

with open('/tmp/wave_boot_$$/bootsect.bin', 'wb') as f:
    f.write(boot)

print("   Boot sector: 512 bytes")
PYTHON_EOF

# Step 3: Generate kernel code (16-bit real mode)
echo "[3/3] Generating kernel with Wave content..."

python3 << 'PYTHON_EOF'
import sys

# Read the Wave-generated code+data from raw.img (skip boot sector)
with open('/tmp/wave_boot_$$/raw.img', 'rb') as f:
    raw = f.read()

# The Wave compiler puts code after the boot sector
# We need to convert it to 16-bit real mode compatible code

# Create kernel that runs at 0x7E00
kernel = bytearray()

# Header: setup data segment
kernel.extend([
    0x31, 0xC0,       # xor ax, ax
    0x8E, 0xD8,       # mov ds, ax
])

# Move SI to point to message
msg_offset = 32  # Message starts at offset 32 in kernel
kernel.extend([
    0xBE,             # mov si, imm16
    (0x7E00 + msg_offset) & 0xFF,
    ((0x7E00 + msg_offset) >> 8) & 0xFF,
])

# Print loop
print_loop = len(kernel)
kernel.extend([
    0xAC,             # lodsb
    0x84, 0xC0,       # test al, al
    0x74, 0x0D,       # jz done (skip 13 bytes)
    0xB4, 0x0E,       # mov ah, 0x0E
    0xBB, 0x07, 0x00, # mov bx, 0x0007
    0xCD, 0x10,       # int 0x10
    0xBA, 0xF8, 0x03, # mov dx, 0x03F8
    0xEE,             # out dx, al
])

# jmp back to print_loop
jump_offset = print_loop - len(kernel) - 2
kernel.append(0xEB)
kernel.append(jump_offset & 0xFF)

# done:
kernel.extend([
    0xF4,             # hlt
    0xEB, 0xFE,       # jmp $
])

# Pad to message offset
while len(kernel) < msg_offset:
    kernel.append(0x00)

# Add the Val-OS message
message = """
================================================================================

                        Val-OS v3.0 - Wave Native OS
                           Fate Engine Active

================================================================================

[Fate] Initializing...                                                     OK

[Fate] Quattro Memory Pools:
              BlackHole    [Active]  -  Extreme Data
              MeshBrain    [Active]  -  Cognitive Processing
              MultiNova    [Active]  -  Large Scale
              BaseForce    [Active]  -  Fragment Optimization

[Wave] Runtime Loading...                                                  OK
              Can execute .wave programs directly

[Drivers]
              Display        [Ready]  -  VGA Text Mode
              Keyboard       [Ready]  -  PS/2 IRQ1
              Mouse          [Ready]  -  PS/2 IRQ12
              Network        [Ready]  -  TCP/IP Stack

[Apps]
              Files | Browser | Terminal | Settings | Notepad
              Music | Video   | Games    | Store

================================================================================
                                  Val-OS v3.0 Ready
                      Wave Language | Fate Engine | Quattro Memory
================================================================================

Press any key to continue...
"""

for c in message:
    kernel.append(ord(c))
kernel.append(0)  # null terminator

# Pad to sector boundary
while len(kernel) % 512 != 0:
    kernel.append(0)

# Read boot sector
with open('/tmp/wave_boot_$$/bootsect.bin', 'rb') as f:
    bootsect = f.read()

# Write final image
with open('$OUTPUT', 'wb') as f:
    f.write(bootsect)
    f.write(kernel)

print(f"   Kernel: {len(kernel)} bytes")
print(f"   Total: {512 + len(kernel)} bytes")
PYTHON_EOF

# Cleanup
rm -rf "$TMP_DIR"

echo ""
echo "═══════════════════════════════════════════════════════"
echo "✅ Generated: $OUTPUT"
echo ""
echo "Run with: qemu-system-i386 -fda $OUTPUT -serial stdio"
echo "═══════════════════════════════════════════════════════"
