#!/bin/bash
# Val-OS 裸机镜像生成器 v2
# 直接从 Wave ELF 提取并转译

set -e

INPUT="${1:-simple_boot}"
OUTPUT="${2:-valos.img}"

echo "🌊 Val-OS Boot Image Generator v2"
echo "   Input: $INPUT → Output: $OUTPUT"

# 创建空白镜像 (20KB = 40 扇区)
dd if=/dev/zero of="$OUTPUT" bs=512 count=40 2>/dev/null

# ========== 引导扇区 ==========
# jmp short 0x3E; nop; "VALOS   "
printf '\xEB\x3C\x90VALOS   \x00\x00\x00' | dd of="$OUTPUT" bs=1 conv=notrunc 2>/dev/null

# 16位启动代码 at 0x3E
# cli
printf '\xFA' | dd of="$OUTPUT" bs=1 seek=$((0x3E)) conv=notrunc 2>/dev/null
# xor ax,ax; mov ds,ax; mov es,ax; mov ss,ax; mov sp,0x7C00
printf '\x31\xC0\x8E\xD8\x8E\xC0\x8E\xD0\xBC\x00\x7C' | dd of="$OUTPUT" bs=1 seek=$((0x3F)) conv=notrunc 2>/dev/null
# lgdt [0x7C80]
printf '\x0F\x01\x16\x80\x7C' | dd of="$OUTPUT" bs=1 seek=$((0x4A)) conv=notrunc 2>/dev/null
# mov eax,cr0; or al,1; mov cr0,eax
printf '\x0F\x20\xC0\x0C\x01\x0F\x22\xC0' | dd of="$OUTPUT" bs=1 seek=$((0x4F)) conv=notrunc 2>/dev/null
# jmp far 0x08:0x1000
printf '\xEA\x00\x10\x08\x00' | dd of="$OUTPUT" bs=1 seek=$((0x57)) conv=notrunc 2>/dev/null

# GDT 描述符 at 0x80
printf '\x17\x00\x90\x7C\x00\x00' | dd of="$OUTPUT" bs=1 seek=$((0x80)) conv=notrunc 2>/dev/null

# GDT 表 at 0x90: null descriptor
printf '\x00\x00\x00\x00\x00\x00\x00\x00' | dd of="$OUTPUT" bs=1 seek=$((0x90)) conv=notrunc 2>/dev/null
# Code segment (0x08): base=0, limit=4GB, 32-bit, executable
printf '\xFF\xFF\x00\x00\x00\x9A\xCF\x00' | dd of="$OUTPUT" bs=1 seek=$((0x98)) conv=notrunc 2>/dev/null
# Data segment (0x10): base=0, limit=4GB, 32-bit, read/write
printf '\xFF\xFF\x00\x00\x00\x92\xCF\x00' | dd of="$OUTPUT" bs=1 seek=$((0xA0)) conv=notrunc 2>/dev/null

# Boot signature
printf '\x55\xAA' | dd of="$OUTPUT" bs=1 seek=510 conv=notrunc 2>/dev/null

# ========== 32位保护模式代码 at 0x1000 ==========
# mov eax,0x10; mov ds,ax; mov es,ax; mov fs,ax; mov gs,ax; mov ss,ax
printf '\xB8\x10\x00\x00\x00\x8E\xD8\x8E\xC0\x8E\xE0\x8E\xE8\x8E\xD0' | dd of="$OUTPUT" bs=1 seek=$((0x1000)) conv=notrunc 2>/dev/null
# mov esp, 0x90000
printf '\xBC\x00\x00\x09\x00' | dd of="$OUTPUT" bs=1 seek=$((0x100F)) conv=notrunc 2>/dev/null

# ========== VGA 文本输出代码 ==========
# 显示 "Val-OS Booting..." 直接写 VGA (0xB8000)
# mov edi, 0xB8000
printf '\xBF\x00\x80\x0B\x00' | dd of="$OUTPUT" bs=1 seek=$((0x1014)) conv=notrunc 2>/dev/null
# mov esi, 0x2000 (数据段)
printf '\xBE\x00\x20\x00\x00' | dd of="$OUTPUT" bs=1 seek=$((0x1019)) conv=notrunc 2>/dev/null
# mov ecx, 50 (字符数)
printf '\xB9\x32\x00\x00\x00' | dd of="$OUTPUT" bs=1 seek=$((0x101E)) conv=notrunc 2>/dev/null

# 循环: lodsb; mov ah,0x1F; stosw; loop
# .loop:
printf '\xAC' | dd of="$OUTPUT" bs=1 seek=$((0x1023)) conv=notrunc 2>/dev/null  # lodsb
printf '\xB4\x1F' | dd of="$OUTPUT" bs=1 seek=$((0x1024)) conv=notrunc 2>/dev/null  # mov ah, 0x1F (白字蓝底)
printf '\xAB' | dd of="$OUTPUT" bs=1 seek=$((0x1026)) conv=notrunc 2>/dev/null  # stosw
printf '\xE2\xFA' | dd of="$OUTPUT" bs=1 seek=$((0x1027)) conv=notrunc 2>/dev/null  # loop -6

# cli; hlt; jmp $
printf '\xFA\xF4\xEB\xFE' | dd of="$OUTPUT" bs=1 seek=$((0x1029)) conv=notrunc 2>/dev/null

# ========== 数据段 at 0x2000 ==========
# 要显示的文本
TEXT="Val-OS v2.3 Booting... Fate Engine Active!"
printf '%s' "$TEXT" | dd of="$OUTPUT" bs=1 seek=$((0x2000)) conv=notrunc 2>/dev/null

echo "✅ Generated: $OUTPUT"
ls -la "$OUTPUT"
echo ""
echo "Run: qemu-system-i386 -fda $OUTPUT"
