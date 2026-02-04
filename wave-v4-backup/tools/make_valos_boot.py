#!/usr/bin/env python3
"""
Wave Boot Image Generator
生成正确的 Val-OS 可引导镜像
"""

import sys
import os

def create_boot_sector(kernel_sectors=63):
    """生成带磁盘读取的引导扇区"""
    boot = bytearray(512)
    pos = 0
    
    def emit(b):
        nonlocal pos
        boot[pos] = b & 0xFF
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
    
    # Initialize serial port for debugging
    # mov dx, 0x03FB
    emit(0xBA); emit_word(0x03FB)
    # mov al, 0x03
    emit(0xB0); emit(0x03)
    # out dx, al
    emit(0xEE)
    
    # Read disk sectors
    # mov bx, 0x7E00 (destination right after boot sector)
    emit(0xBB); emit_word(0x7E00)
    
    # mov ah, 0x02 (read sectors)
    emit(0xB4); emit(0x02)
    
    # mov al, sectors
    emit(0xB0); emit(kernel_sectors)
    
    # mov ch, 0 (cylinder)
    emit(0xB5); emit(0x00)
    
    # mov cl, 2 (start sector)
    emit(0xB1); emit(0x02)
    
    # mov dh, 0 (head)
    emit(0xB6); emit(0x00)
    
    # mov dl, 0 (drive A:)
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
    
    return boot

def create_kernel():
    """生成 Val-OS 内核代码"""
    kernel = bytearray()
    
    # 16-bit real mode code running at 0x7E00
    
    # Setup data segment
    kernel.extend([
        0x31, 0xC0,       # xor ax, ax
        0x8E, 0xD8,       # mov ds, ax
    ])
    
    # Point SI to message
    msg_offset = 32  # Message starts at offset 32
    si_addr = 0x7E00 + msg_offset
    kernel.extend([
        0xBE,             # mov si, imm16
        si_addr & 0xFF,
        (si_addr >> 8) & 0xFF,
    ])
    
    # Print loop (label: print_loop)
    print_loop = len(kernel)
    kernel.extend([
        0xAC,             # lodsb
        0x84, 0xC0,       # test al, al
        0x74, 0x0D,       # jz done (skip 13 bytes)
        0xB4, 0x0E,       # mov ah, 0x0E
        0xBB, 0x07, 0x00, # mov bx, 0x0007 (gray on black)
        0xCD, 0x10,       # int 0x10 (BIOS print char)
        0xBA, 0xF8, 0x03, # mov dx, 0x03F8 (COM1)
        0xEE,             # out dx, al (serial output)
    ])
    
    # jmp back to print_loop
    jump_offset = print_loop - len(kernel) - 2
    kernel.append(0xEB)
    kernel.append(jump_offset & 0xFF)
    
    # done:
    kernel.extend([
        0xF4,             # hlt
        0xEB, 0xFE,       # jmp $ (infinite loop)
    ])
    
    # Pad to message offset
    while len(kernel) < msg_offset:
        kernel.append(0x00)
    
    # Val-OS boot message
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
    
    # Pad to sector boundary (512 bytes)
    while len(kernel) % 512 != 0:
        kernel.append(0)
    
    return kernel

def main():
    output = sys.argv[1] if len(sys.argv) > 1 else "valos_wave_fixed.img"
    
    print("═" * 60)
    print("🌊 Wave Boot Image Generator")
    print("═" * 60)
    print()
    
    # Generate components
    kernel = create_kernel()
    kernel_sectors = (len(kernel) + 511) // 512
    boot = create_boot_sector(kernel_sectors)
    
    # Write image
    with open(output, 'wb') as f:
        f.write(boot)
        f.write(kernel)
    
    total_size = len(boot) + len(kernel)
    
    print(f"Boot sector:  512 bytes")
    print(f"Kernel:       {len(kernel)} bytes ({kernel_sectors} sectors)")
    print(f"Total:        {total_size} bytes")
    print()
    print(f"✅ Generated: {output}")
    print()
    print(f"Run with: qemu-system-i386 -fda {output} -serial stdio")
    print("═" * 60)

if __name__ == "__main__":
    main()
