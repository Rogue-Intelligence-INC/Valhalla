#!/usr/bin/env python3
"""Val-OS v5.0 - 双监控版本 (VGA + Serial) - 精简版"""

def main():
    code = []
    
    # === 初始化 ===
    code.extend([0x31, 0xC0])        # XOR AX, AX
    code.extend([0x8E, 0xD8])        # MOV DS, AX
    code.extend([0x8E, 0xC0])        # MOV ES, AX
    code.extend([0x8E, 0xD0])        # MOV SS, AX
    code.extend([0xBC, 0x00, 0x7C])  # MOV SP, 0x7C00
    
    # === 初始化串口 ===
    code.extend([0xB0, 0x83])        # MOV AL, 0x83 (DLAB=1)
    code.extend([0xBA, 0xFB, 0x03])  # MOV DX, 0x3FB (LCR)
    code.extend([0xEE])              # OUT DX, AL
    code.extend([0xB0, 0x01])        # MOV AL, 1 (115200 baud)
    code.extend([0xBA, 0xF8, 0x03])  # MOV DX, 0x3F8
    code.extend([0xEE])              # OUT DX, AL
    code.extend([0xB0, 0x00])        # MOV AL, 0
    code.extend([0xBA, 0xF9, 0x03])  # MOV DX, 0x3F9
    code.extend([0xEE])              # OUT DX, AL
    code.extend([0xB0, 0x03])        # MOV AL, 0x03 (8N1)
    code.extend([0xBA, 0xFB, 0x03])  # MOV DX, 0x3FB
    code.extend([0xEE])              # OUT DX, AL
    
    # 输出 "V" 到串口
    code.extend([0xB0, ord('V')])    # MOV AL, 'V'
    code.extend([0xBA, 0xF8, 0x03])  # MOV DX, 0x3F8
    code.extend([0xEE])              # OUT DX, AL
    code.extend([0xB0, ord('O')])
    code.extend([0xEE])
    code.extend([0xB0, ord('S')])
    code.extend([0xEE])
    code.extend([0xB0, ord('\r')])
    code.extend([0xEE])
    code.extend([0xB0, ord('\n')])
    code.extend([0xEE])
    
    # === VGA 模式 13h ===
    code.extend([0xB8, 0x13, 0x00])  # MOV AX, 0x0013
    code.extend([0xCD, 0x10])        # INT 0x10
    
    # 串口: "G"
    code.extend([0xB0, ord('G')])
    code.extend([0xBA, 0xF8, 0x03])
    code.extend([0xEE])
    
    # === 绘制蓝色背景 ===
    code.extend([0xB8, 0x00, 0xA0])  # MOV AX, 0xA000
    code.extend([0x8E, 0xC0])        # MOV ES, AX
    code.extend([0x31, 0xFF])        # XOR DI, DI
    code.extend([0xB9, 0x00, 0xFA])  # MOV CX, 64000
    code.extend([0xB0, 0x01])        # MOV AL, 1 (蓝色)
    code.extend([0xF3, 0xAA])        # REP STOSB
    
    # 串口: "B"
    code.extend([0xB0, ord('B')])
    code.extend([0xBA, 0xF8, 0x03])
    code.extend([0xEE])
    
    # === 绘制白色窗口 (100x60 at 110,70) ===
    # DI = 70*320+110 = 22510
    code.extend([0xBF, 0xEE, 0x57])  # MOV DI, 22510
    code.extend([0xB9, 0x3C, 0x00])  # MOV CX, 60 (行)
    # win_loop:
    win_loop = len(code)
    code.extend([0x51])              # PUSH CX
    code.extend([0xB9, 0x64, 0x00])  # MOV CX, 100
    code.extend([0xB0, 0x0F])        # MOV AL, 15 (白色)
    code.extend([0xF3, 0xAA])        # REP STOSB
    code.extend([0x81, 0xC7, 0xDC, 0x00])  # ADD DI, 220 (320-100)
    code.extend([0x59])              # POP CX
    # LOOP win_loop
    offset = win_loop - (len(code) + 2)
    code.extend([0xE2, offset & 0xFF])
    
    # 串口: "W"
    code.extend([0xB0, ord('W')])
    code.extend([0xBA, 0xF8, 0x03])
    code.extend([0xEE])
    
    # === 绘制标题栏 (蓝色) ===
    code.extend([0xBF, 0xEE, 0x57])  # MOV DI, 22510
    code.extend([0xB9, 0x10, 0x00])  # MOV CX, 16 (行)
    title_loop = len(code)
    code.extend([0x51])              # PUSH CX
    code.extend([0xB9, 0x64, 0x00])  # MOV CX, 100
    code.extend([0xB0, 0x09])        # MOV AL, 9 (亮蓝)
    code.extend([0xF3, 0xAA])        # REP STOSB
    code.extend([0x81, 0xC7, 0xDC, 0x00])
    code.extend([0x59])
    offset = title_loop - (len(code) + 2)
    code.extend([0xE2, offset & 0xFF])
    
    # 串口: "T"
    code.extend([0xB0, ord('T')])
    code.extend([0xBA, 0xF8, 0x03])
    code.extend([0xEE])
    
    # === 绘制红色图标 (10x10 at 20,30) ===
    # DI = 30*320+20 = 9620
    code.extend([0xBF, 0x94, 0x25])  # MOV DI, 9620
    code.extend([0xB9, 0x0A, 0x00])  # MOV CX, 10
    icon_loop = len(code)
    code.extend([0x51])
    code.extend([0xB9, 0x0A, 0x00])
    code.extend([0xB0, 0x04])        # MOV AL, 4 (红色)
    code.extend([0xF3, 0xAA])
    code.extend([0x81, 0xC7, 0x36, 0x01])  # ADD DI, 310
    code.extend([0x59])
    offset = icon_loop - (len(code) + 2)
    code.extend([0xE2, offset & 0xFF])
    
    # === 绘制绿色图标 (10x10 at 20,50) ===
    code.extend([0xBF, 0xF4, 0x3E])  # MOV DI, 16116
    code.extend([0xB9, 0x0A, 0x00])
    icon2_loop = len(code)
    code.extend([0x51])
    code.extend([0xB9, 0x0A, 0x00])
    code.extend([0xB0, 0x02])        # MOV AL, 2 (绿色)
    code.extend([0xF3, 0xAA])
    code.extend([0x81, 0xC7, 0x36, 0x01])
    code.extend([0x59])
    offset = icon2_loop - (len(code) + 2)
    code.extend([0xE2, offset & 0xFF])
    
    # === 绘制任务栏 ===
    code.extend([0xBF, 0x00, 0xEE])  # DI = 190*320
    code.extend([0xB9, 0x00, 0x0C])  # CX = 3200
    code.extend([0xB0, 0x08])        # AL = 8 (深灰)
    code.extend([0xF3, 0xAA])
    
    # 串口: "OK\r\n"
    code.extend([0xB0, ord('O')])
    code.extend([0xBA, 0xF8, 0x03])
    code.extend([0xEE])
    code.extend([0xB0, ord('K')])
    code.extend([0xEE])
    code.extend([0xB0, ord('\r')])
    code.extend([0xEE])
    code.extend([0xB0, ord('\n')])
    code.extend([0xEE])
    
    # === 主循环 ===
    main_loop = len(code)
    code.extend([0xB4, 0x01])        # MOV AH, 1
    code.extend([0xCD, 0x16])        # INT 0x16
    code.extend([0x74, 0x08])        # JZ no_key
    code.extend([0xB4, 0x00])        # MOV AH, 0
    code.extend([0xCD, 0x16])        # INT 0x16
    code.extend([0x3C, 0x1B])        # CMP AL, ESC
    code.extend([0x74, 0x02])        # JE exit
    # no_key:
    offset = main_loop - (len(code) + 2)
    code.extend([0xEB, offset & 0xFF])  # JMP main_loop
    
    # exit:
    code.extend([0xF4])              # HLT
    
    print(f"Code size: {len(code)} bytes")
    
    # 确保不超过 510 字节
    if len(code) > 510:
        print(f"ERROR: Code too large! ({len(code)} > 510)")
        return
    
    # 填充
    while len(code) < 510:
        code.append(0x90)
    
    # 引导签名
    code.extend([0x55, 0xAA])
    
    # 写入
    with open("val-os-v5/build/valos_dual.img", "wb") as f:
        f.write(bytes(code))
    
    print(f"✅ Created: val-os-v5/build/valos_dual.img")
    print(f"   Size: {len(code)} bytes")
    print(f"   Boot signature: {code[510]:02X} {code[511]:02X}")

if __name__ == "__main__":
    main()
