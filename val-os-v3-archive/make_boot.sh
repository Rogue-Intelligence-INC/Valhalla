#!/bin/bash
# Val-OS 裸机镜像生成器
# Wave 风格：组合工具，Fate 决定

set -e

INPUT="${1:-simple_boot}"
OUTPUT="${2:-valos.img}"

echo "🌊 Val-OS Boot Image Generator"
echo "   Input: $INPUT"
echo "   Output: $OUTPUT"

# 检查输入
if [ ! -f "$INPUT" ]; then
    echo "Error: $INPUT not found"
    exit 1
fi

# Wave ELF 布局是固定的: 代码在 0x1000, 数据在 0x2000
CODE_OFFSET="1000"
DATA_OFFSET="2000"
# 从文件大小计算
FILE_SIZE=$(stat -c%s "$INPUT")
CODE_SIZE=$(printf '%x' $((0x1000)))  # 到数据段的距离
DATA_SIZE=$(printf '%x' $((FILE_SIZE - 0x2000)))

echo "   Code: offset=0x$CODE_OFFSET size=0x$CODE_SIZE"
echo "   Data: offset=0x$DATA_OFFSET size=0x$DATA_SIZE"

# 创建引导扇区
BOOTSECT=$(mktemp)
dd if=/dev/zero of="$BOOTSECT" bs=512 count=1 2>/dev/null

# 写入引导扇区代码 (手工构造的 16-bit → 32-bit 跳转)
printf '\xEB\x3C\x90VALOS   ' | dd of="$BOOTSECT" bs=1 conv=notrunc 2>/dev/null

# 16-bit 启动代码 at 0x3E
# cli; xor ax,ax; mov ds,ax; mov es,ax; mov ss,ax; mov sp,0x7C00
# lgdt [0x7C80]; mov eax,cr0; or al,1; mov cr0,eax; jmp 0x08:0x1000
printf '\xFA\x31\xC0\x8E\xD8\x8E\xC0\x8E\xD0\xBC\x00\x7C' | dd of="$BOOTSECT" bs=1 seek=$((0x3E)) conv=notrunc 2>/dev/null
printf '\x0F\x01\x16\x80\x7C' | dd of="$BOOTSECT" bs=1 seek=$((0x4A)) conv=notrunc 2>/dev/null
printf '\x0F\x20\xC0\x0C\x01\x0F\x22\xC0' | dd of="$BOOTSECT" bs=1 seek=$((0x4F)) conv=notrunc 2>/dev/null
printf '\xEA\x00\x10\x08\x00' | dd of="$BOOTSECT" bs=1 seek=$((0x57)) conv=notrunc 2>/dev/null

# GDT 描述符 at 0x80
printf '\x17\x00\x90\x7C\x00\x00' | dd of="$BOOTSECT" bs=1 seek=$((0x80)) conv=notrunc 2>/dev/null

# GDT 表 at 0x90: null, code, data
printf '\x00\x00\x00\x00\x00\x00\x00\x00' | dd of="$BOOTSECT" bs=1 seek=$((0x90)) conv=notrunc 2>/dev/null
printf '\xFF\xFF\x00\x00\x00\x9A\xCF\x00' | dd of="$BOOTSECT" bs=1 seek=$((0x98)) conv=notrunc 2>/dev/null
printf '\xFF\xFF\x00\x00\x00\x92\xCF\x00' | dd of="$BOOTSECT" bs=1 seek=$((0xA0)) conv=notrunc 2>/dev/null

# Boot signature
printf '\x55\xAA' | dd of="$BOOTSECT" bs=1 seek=510 conv=notrunc 2>/dev/null

# 创建输出镜像
dd if=/dev/zero of="$OUTPUT" bs=512 count=40 2>/dev/null

# 写入引导扇区
dd if="$BOOTSECT" of="$OUTPUT" bs=512 conv=notrunc 2>/dev/null

# 32-bit 保护模式头 at 0x1000
# mov eax,0x10; mov ds,ax; mov es,ax; mov fs,ax; mov gs,ax; mov ss,ax; mov esp,0x90000
PM_HEADER=$(mktemp)
printf '\xB8\x10\x00\x00\x00\x8E\xD8\x8E\xC0\x8E\xE0\x8E\xE8\x8E\xD0\xBC\x00\x00\x09\x00' > "$PM_HEADER"
dd if="$PM_HEADER" of="$OUTPUT" bs=1 seek=$((0x1000)) conv=notrunc 2>/dev/null

# 提取并写入代码 (需要转译 64→32)
dd if="$INPUT" bs=1 skip=$((0x$CODE_OFFSET)) count=$((0x$CODE_SIZE)) 2>/dev/null | \
    # 简单转译：移除 REX 前缀 (0x48, 0x49, 0x4C, 0x4D)，转换 syscall 为 hlt
    xxd -p | sed 's/480f05/f4ebfe/g; s/48//g; s/49//g; s/4c//g; s/4d//g' | xxd -r -p | \
    dd of="$OUTPUT" bs=1 seek=$((0x1000 + 20)) conv=notrunc 2>/dev/null

# 写入数据段 at 0x2000
dd if="$INPUT" bs=1 skip=$((0x$DATA_OFFSET)) count=$((0x$DATA_SIZE)) of="$OUTPUT" seek=$((0x2000)) conv=notrunc 2>/dev/null

rm -f "$BOOTSECT" "$PM_HEADER"

echo ""
echo "✅ Generated: $OUTPUT"
ls -la "$OUTPUT"
echo ""
echo "Run: qemu-system-i386 -fda $OUTPUT"
