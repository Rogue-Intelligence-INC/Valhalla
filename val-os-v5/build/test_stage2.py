#!/usr/bin/env python3
"""简化测试 - 验证第二阶段加载"""

def main():
    # === Stage 1: 引导扇区 ===
    stage1 = []
    
    # 初始化
    stage1.extend([0x31, 0xC0])        # XOR AX, AX
    stage1.extend([0x8E, 0xD8])        # MOV DS, AX
    stage1.extend([0x8E, 0xC0])        # MOV ES, AX
    stage1.extend([0x8E, 0xD0])        # MOV SS, AX
    stage1.extend([0xBC, 0x00, 0x7C])  # MOV SP, 0x7C00
    
    # 串口初始化
    stage1.extend([0xB0, 0x03])
    stage1.extend([0xBA, 0xFB, 0x03])
    stage1.extend([0xEE])
    
    # 输出 "1"
    stage1.extend([0xB0, ord('1')])
    stage1.extend([0xBA, 0xF8, 0x03])
    stage1.extend([0xEE])
    
    # 加载 Stage 2 到 0x1000:0000 = 0x10000
    stage1.extend([0xB8, 0x00, 0x10])  # MOV AX, 0x1000
    stage1.extend([0x8E, 0xC0])        # MOV ES, AX
    stage1.extend([0x31, 0xDB])        # XOR BX, BX
    stage1.extend([0xB8, 0x02, 0x08])  # MOV AX, 0x0802 (AH=02 read, AL=8 sectors)
    stage1.extend([0xB9, 0x02, 0x00])  # MOV CX, 0x0002 (sector 2, cylinder 0)
    stage1.extend([0xBA, 0x00, 0x00])  # MOV DX, 0x0000 (head 0, drive 0)
    stage1.extend([0xCD, 0x13])        # INT 0x13
    
    # 检查错误
    stage1.extend([0x72, 0x08])        # JC error (如果失败)
    
    # 输出 "2"
    stage1.extend([0xB0, ord('2')])
    stage1.extend([0xBA, 0xF8, 0x03])
    stage1.extend([0xEE])
    
    # 跳转到 Stage 2
    stage1.extend([0xEA, 0x00, 0x00, 0x00, 0x10])  # JMP FAR 0x1000:0000
    
    # error: 输出 "E"
    stage1.extend([0xB0, ord('E')])
    stage1.extend([0xBA, 0xF8, 0x03])
    stage1.extend([0xEE])
    stage1.extend([0xF4])  # HLT
    
    # 填充
    while len(stage1) < 510:
        stage1.append(0x90)
    stage1.extend([0x55, 0xAA])
    
    # === Stage 2 ===
    stage2 = []
    
    # 直接输出 "3" 到串口 (不需要段寄存器)
    stage2.extend([0xB0, ord('3')])
    stage2.extend([0xBA, 0xF8, 0x03])
    stage2.extend([0xEE])
    
    # 设置 VGA 模式 13h
    stage2.extend([0xB8, 0x13, 0x00])
    stage2.extend([0xCD, 0x10])
    
    # 输出 "4"
    stage2.extend([0xB0, ord('4')])
    stage2.extend([0xBA, 0xF8, 0x03])
    stage2.extend([0xEE])
    
    # 绘制蓝色背景
    stage2.extend([0xB8, 0x00, 0xA0])  # MOV AX, 0xA000
    stage2.extend([0x8E, 0xC0])        # MOV ES, AX
    stage2.extend([0x31, 0xFF])        # XOR DI, DI
    stage2.extend([0xB9, 0x00, 0xFA])  # MOV CX, 64000
    stage2.extend([0xB0, 0x01])        # MOV AL, 1 (blue)
    stage2.extend([0xF3, 0xAA])        # REP STOSB
    
    # 输出 "5"
    stage2.extend([0xB0, ord('5')])
    stage2.extend([0xBA, 0xF8, 0x03])
    stage2.extend([0xEE])
    
    # 绘制白色窗口
    # DI = 50*320+80 = 16080
    stage2.extend([0xBF, 0xF0, 0x3E])  # MOV DI, 16080
    stage2.extend([0xB9, 0x64, 0x00])  # MOV CX, 100 rows
    # loop:
    loop_start = len(stage2)
    stage2.extend([0x51])              # PUSH CX
    stage2.extend([0xB9, 0xA0, 0x00])  # MOV CX, 160 cols
    stage2.extend([0xB0, 0x0F])        # MOV AL, 15 (white)
    stage2.extend([0xF3, 0xAA])        # REP STOSB
    stage2.extend([0x81, 0xC7, 0xA0, 0x00])  # ADD DI, 160
    stage2.extend([0x59])              # POP CX
    offset = loop_start - (len(stage2) + 2)
    stage2.extend([0xE2, offset & 0xFF])
    
    # 输出 "OK"
    stage2.extend([0xB0, ord('O')])
    stage2.extend([0xBA, 0xF8, 0x03])
    stage2.extend([0xEE])
    stage2.extend([0xB0, ord('K')])
    stage2.extend([0xEE])
    stage2.extend([0xB0, ord('\r')])
    stage2.extend([0xEE])
    stage2.extend([0xB0, ord('\n')])
    stage2.extend([0xEE])
    
    # 主循环
    main_loop = len(stage2)
    stage2.extend([0xB4, 0x01])
    stage2.extend([0xCD, 0x16])
    stage2.extend([0x74, 0x08])
    stage2.extend([0xB4, 0x00])
    stage2.extend([0xCD, 0x16])
    stage2.extend([0x3C, 0x1B])
    stage2.extend([0x74, 0x02])
    offset = main_loop - (len(stage2) + 2)
    stage2.extend([0xEB, offset & 0xFF])
    stage2.extend([0xF4])  # HLT
    
    # 填充 stage2 到扇区边界
    while len(stage2) % 512 != 0:
        stage2.append(0x90)
    
    # 合并
    image = bytes(stage1) + bytes(stage2)
    
    # 填充到至少 8 扇区
    while len(image) < 512 * 10:
        image += b'\x00'
    
    with open("val-os-v5/build/valos_test.img", "wb") as f:
        f.write(image)
    
    print(f"✅ Created: val-os-v5/build/valos_test.img")
    print(f"   Size: {len(image)} bytes")
    print()
    print("串口输出含义:")
    print("  1 = Stage 1 开始")
    print("  2 = 加载成功，准备跳转")
    print("  3 = Stage 2 开始执行")
    print("  4 = VGA 模式设置完成")
    print("  5 = 背景绘制完成")
    print("  OK = 窗口绘制完成")
    print("  E = 加载失败")

if __name__ == "__main__":
    main()
