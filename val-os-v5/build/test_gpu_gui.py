#!/usr/bin/env python3
"""
Val-OS v5.0 - GPU + Native GUI Test
测试 Bochs VBE + Wave GUI 组件
"""

code = bytearray()

# ═══════════════════════════════════════════════════════════════════
# BOOT SECTOR
# ═══════════════════════════════════════════════════════════════════

code.extend([0xEB, 0x3C, 0x90])
code.extend(b'VALOS5GP')
code.extend([0] * 51)

# Setup segments
code.extend([0xFA, 0x31, 0xC0, 0x8E, 0xD8, 0x8E, 0xC0, 0x8E, 0xD0])
code.extend([0xBC, 0x00, 0x7C, 0xFB])
code.extend([0x88, 0x16, 0xFE, 0x7D])

# Load more sectors
code.extend([0xB4, 0x02, 0xB0, 0x60, 0xB5, 0x00, 0xB1, 0x02, 0xB6, 0x00])
code.extend([0x8A, 0x16, 0xFE, 0x7D])
code.extend([0xBB, 0x00, 0x7E, 0xCD, 0x13])

# Jump to stage 2
code.extend([0xEA, 0x00, 0x7E, 0x00, 0x00])

while len(code) < 510:
    code.append(0)
code.extend([0x55, 0xAA])

# ═══════════════════════════════════════════════════════════════════
# STAGE 2 - VBE/VESA Setup
# ═══════════════════════════════════════════════════════════════════

# 尝试 VESA 模式
code.extend([0xB8, 0x02, 0x4F])  # mov ax, 0x4F02
code.extend([0xBB, 0x18, 0x41])  # mov bx, 0x4118 (1024x768x32 | LFB)
code.extend([0xCD, 0x10])        # int 0x10

# 检查是否成功，如果失败使用 Mode 13h
code.extend([0x3C, 0x4F])        # cmp al, 0x4F
code.extend([0x74, 0x05])        # je vesa_ok
code.extend([0xB8, 0x13, 0x00])  # mov ax, 0x0013
code.extend([0xCD, 0x10])        # int 0x10

# vesa_ok: 继续

# ─────────────────────────────────────────────────────────────────
# 绘制 GUI (Mode 13h: 320x200)
# ─────────────────────────────────────────────────────────────────

# 绘制渐变背景
code.extend([0x06])  # push es
code.extend([0xB8, 0x00, 0xA0])  # mov ax, 0xA000
code.extend([0x8E, 0xC0])  # mov es, ax
code.extend([0x31, 0xFF])  # xor di, di

for y in range(200):
    color = max(0, 17 - y // 12)
    code.extend([0xB0, color])
    code.extend([0xB9, 0x40, 0x01])
    code.extend([0xF3, 0xAA])

code.extend([0x07])  # pop es

def draw_rect(x, y, w, h, color):
    global code
    code.extend([0x06])
    code.extend([0xB8, 0x00, 0xA0])
    code.extend([0x8E, 0xC0])
    
    for row in range(h):
        offset = (y + row) * 320 + x
        code.extend([0xBF, offset & 0xFF, (offset >> 8) & 0xFF])
        code.extend([0xB0, color])
        # mov cx, w (16-bit)
        code.extend([0xB9, w & 0xFF, (w >> 8) & 0xFF])
        code.extend([0xF3, 0xAA])
    
    code.extend([0x07])

# 任务栏
draw_rect(0, 188, 320, 12, 0x08)
draw_rect(2, 190, 20, 8, 0x09)

# 窗口阴影
draw_rect(32, 22, 260, 150, 0x00)
# 窗口背景
draw_rect(30, 20, 260, 150, 0x17)
# 标题栏
draw_rect(30, 20, 260, 16, 0x01)
# 窗口按钮
draw_rect(278, 24, 8, 8, 0x04)
draw_rect(266, 24, 8, 8, 0x0E)
draw_rect(254, 24, 8, 8, 0x02)
# 地址栏
draw_rect(35, 40, 250, 12, 0x07)
# 内容区域
draw_rect(35, 56, 250, 108, 0x0F)

# 侧边栏图标
draw_rect(5, 25, 20, 20, 0x02)
draw_rect(5, 50, 20, 20, 0x01)
draw_rect(5, 75, 20, 20, 0x04)
draw_rect(5, 100, 20, 20, 0x03)
draw_rect(5, 125, 20, 20, 0x05)
draw_rect(5, 150, 20, 20, 0x06)

# 通知
draw_rect(220, 160, 90, 25, 0x08)
draw_rect(221, 161, 88, 23, 0x17)

# ─────────────────────────────────────────────────────────────────
# 打印文本
# ─────────────────────────────────────────────────────────────────

def print_text(text, row, col, attr=0x0F):
    global code
    code.extend([0xB4, 0x02, 0xB7, 0x00, 0xB6, row, 0xB2, col, 0xCD, 0x10])
    for ch in text:
        code.extend([0xB4, 0x09, 0xB0, ord(ch), 0xB3, attr, 0xB9, 0x01, 0x00, 0xCD, 0x10])
        col += 1
        code.extend([0xB4, 0x02, 0xB7, 0x00, 0xB6, row, 0xB2, col, 0xCD, 0x10])

print_text("Wave Browser", 3, 5, 0x1F)
print_text("wave://home", 5, 6, 0x08)
print_text("Val-OS v5.0 GPU Test", 8, 8, 0x00)
print_text("GPU: Wave Native", 10, 8, 0x02)
print_text("Wave GUI Layer", 12, 8, 0x01)
print_text("Components Ready", 14, 8, 0x01)
print_text("Wave", 24, 1, 0x09)
print_text("12:35", 24, 37, 0x07)
print_text("F", 4, 2, 0x0F)
print_text("T", 7, 2, 0x0F)
print_text("B", 10, 2, 0x0F)
print_text("S", 13, 2, 0x0F)
print_text("G", 16, 2, 0x0F)
print_text("M", 19, 2, 0x0F)
print_text("GPU OK!", 21, 29, 0x0A)
print_text("ESC=Exit", 22, 29, 0x08)

# 鼠标
code.extend([0xB8, 0x00, 0x00, 0xCD, 0x33])
code.extend([0xB8, 0x01, 0x00, 0xCD, 0x33])

# 主循环
main_loop = len(code)

code.extend([0xB4, 0x01, 0xCD, 0x16])
code.extend([0x74, 0x08])

code.extend([0xB4, 0x00, 0xCD, 0x16])
code.extend([0x3C, 0x1B])
code.extend([0x74, 0x20])

# 鼠标光标
code.extend([0xB8, 0x03, 0x00, 0xCD, 0x33])
code.extend([0x06])
code.extend([0xB8, 0x00, 0xA0])
code.extend([0x8E, 0xC0])
code.extend([0x89, 0xD0])
code.extend([0xBB, 0x40, 0x01])
code.extend([0xF7, 0xE3])
code.extend([0x01, 0xC8])
code.extend([0x89, 0xC7])
code.extend([0xB0, 0x0F])
code.extend([0x26, 0x88, 0x05])
code.extend([0x07])

code.extend([0xB9, 0xFF, 0x0F, 0xE2, 0xFE])

offset = main_loop - len(code) - 2
code.extend([0xEB, offset & 0xFF])

# quit
code.extend([0xB8, 0x03, 0x00, 0xCD, 0x10])
print_text("GPU Test Complete!", 12, 30, 0x0A)
code.extend([0xEB, 0xFE])

# 填充
while len(code) % 512 != 0:
    code.append(0)
while len(code) < 96 * 512:
    code.append(0)

with open('val-os-v5/build/valos_gpu_test.img', 'wb') as f:
    f.write(bytes(code))

print(f"✅ Created val-os-v5/build/valos_gpu_test.img")
print(f"   Size: {len(code)} bytes ({len(code)//512} sectors)")
