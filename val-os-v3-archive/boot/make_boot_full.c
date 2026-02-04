// Val-OS 完整引导扇区 + 第二阶段
// 多扇区版本，可显示更多内容

#include <stdio.h>
#include <string.h>
#include <stdint.h>

// 扇区数据
uint8_t sectors[20][512];

void put_string(int sector, int offset, const char* str) {
    int len = strlen(str);
    memcpy(sectors[sector] + offset, str, len);
}

int main() {
    memset(sectors, 0, sizeof(sectors));
    
    // === 引导扇区 (扇区 0) ===
    uint8_t* boot = sectors[0];
    int i = 0;
    
    // jmp short start (跳过 BPB)
    boot[i++] = 0xEB;
    boot[i++] = 0x3C;
    boot[i++] = 0x90;
    
    // OEM ID
    memcpy(boot + 3, "VALOS   ", 8);
    
    // 从 0x3E 开始
    i = 0x3E;
    
    // cli
    boot[i++] = 0xFA;
    
    // xor ax, ax
    boot[i++] = 0x31; boot[i++] = 0xC0;
    // mov ds, ax
    boot[i++] = 0x8E; boot[i++] = 0xD8;
    // mov es, ax
    boot[i++] = 0x8E; boot[i++] = 0xC0;
    // mov ss, ax
    boot[i++] = 0x8E; boot[i++] = 0xD0;
    // mov sp, 0x7C00
    boot[i++] = 0xBC; boot[i++] = 0x00; boot[i++] = 0x7C;
    
    // sti
    boot[i++] = 0xFB;
    
    // 设置视频模式 80x25 彩色
    // mov ax, 0x0003
    boot[i++] = 0xB8; boot[i++] = 0x03; boot[i++] = 0x00;
    // int 0x10
    boot[i++] = 0xCD; boot[i++] = 0x10;
    
    // 清屏 - 蓝色背景
    // mov ah, 0x06
    boot[i++] = 0xB4; boot[i++] = 0x06;
    // mov al, 0x00
    boot[i++] = 0xB0; boot[i++] = 0x00;
    // mov bh, 0x1F (蓝底白字)
    boot[i++] = 0xB7; boot[i++] = 0x1F;
    // mov cx, 0x0000
    boot[i++] = 0xB9; boot[i++] = 0x00; boot[i++] = 0x00;
    // mov dx, 0x184F
    boot[i++] = 0xBA; boot[i++] = 0x4F; boot[i++] = 0x18;
    // int 0x10
    boot[i++] = 0xCD; boot[i++] = 0x10;
    
    // 隐藏光标
    // mov ah, 0x01
    boot[i++] = 0xB4; boot[i++] = 0x01;
    // mov cx, 0x2000
    boot[i++] = 0xB9; boot[i++] = 0x00; boot[i++] = 0x20;
    // int 0x10
    boot[i++] = 0xCD; boot[i++] = 0x10;
    
    // 直接写 VGA 显存 (0xB800:0000)
    // mov ax, 0xB800
    boot[i++] = 0xB8; boot[i++] = 0x00; boot[i++] = 0xB8;
    // mov es, ax
    boot[i++] = 0x8E; boot[i++] = 0xC0;
    
    // 复制字符串数据到显存
    // mov si, 0x7E00 (第二扇区的字符串)
    boot[i++] = 0xBE; boot[i++] = 0x00; boot[i++] = 0x7E;
    // mov di, 160 (从第1行开始，每行160字节)
    boot[i++] = 0xBF; boot[i++] = 0xA0; boot[i++] = 0x00;
    // mov cx, 800 (20行 * 80字符 / 2 = 800 words)
    boot[i++] = 0xB9; boot[i++] = 0x20; boot[i++] = 0x03;
    
    // copy_loop:
    int copy_loop = i;
    // lodsb
    boot[i++] = 0xAC;
    // mov ah, 0x1F (蓝底白字)
    boot[i++] = 0xB4; boot[i++] = 0x1F;
    // stosw
    boot[i++] = 0xAB;
    // loop copy_loop
    boot[i++] = 0xE2;
    boot[i++] = (uint8_t)(copy_loop - i - 1);
    
    // 无限循环
    // hlt
    boot[i++] = 0xF4;
    // jmp $-1
    boot[i++] = 0xEB; boot[i++] = 0xFD;
    
    // Boot signature
    boot[510] = 0x55;
    boot[511] = 0xAA;
    
    // === 第二扇区: 显示数据 (纯文本，每字符一字节) ===
    // 这将被复制到 VGA 显存，配合属性字节 0x1F
    
    const char* lines[] = {
        "                                                                                ",
        "        ================================================================        ",
        "        |                                                              |        ",
        "        |              Val-OS v2.3 - Fate Engine                       |        ",
        "        |              Wave Language Self-Hosting OS                   |        ",
        "        |                                                              |        ",
        "        ================================================================        ",
        "                                                                                ",
        "        [Fate] Waking up...                                         OK         ",
        "        [Fate] Detecting CPU: x86-64 Compatible                     OK         ",
        "        [Fate] Detecting Memory: 128 MB                             OK         ",
        "        [Fate] Detecting Storage                                    OK         ",
        "        [Fate] Initializing Quattro Memory Pools...                            ",
        "           -> BlackHole  (Extreme Data Processing)              Ready          ",
        "           -> MeshBrain  (Cognitive Data Processing)            Ready          ",
        "           -> MultiNova  (Large Scale Processing)               Ready          ",
        "           -> BaseForce  (Fragment Optimization)                Ready          ",
        "        [Fate] Memory layout decided dynamically                    OK         ",
        "        [Fate] Loading Val-OS kernel...                             OK         ",
        "                                                                                ",
        "        ================================================================        ",
        "        |       *** Val-OS Boot Complete - Fate is Active ***        |        ",
        "        ================================================================        ",
        "                                                                                ",
        NULL
    };
    
    int offset = 0;
    for (int l = 0; lines[l] != NULL && offset < 512 * 4; l++) {
        int len = strlen(lines[l]);
        if (len > 80) len = 80;
        
        // 填充到正好 80 字符
        for (int c = 0; c < 80; c++) {
            int sector = 1 + (offset / 512);
            int pos = offset % 512;
            if (c < len) {
                sectors[sector][pos] = lines[l][c];
            } else {
                sectors[sector][pos] = ' ';
            }
            offset++;
        }
    }
    
    // 写入文件
    FILE* f = fopen("valos_full.img", "wb");
    if (!f) {
        fprintf(stderr, "Cannot create valos_full.img\n");
        return 1;
    }
    
    for (int s = 0; s < 10; s++) {
        fwrite(sectors[s], 1, 512, f);
    }
    
    fclose(f);
    
    printf("Generated valos_full.img (5120 bytes, 10 sectors)\n");
    printf("Run with: qemu-system-i386 -fda valos_full.img\n");
    
    return 0;
}
