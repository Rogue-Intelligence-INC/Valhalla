// Val-OS 引导扇区生成器
// 生成可在 QEMU 中运行的 512 字节引导扇区

#include <stdio.h>
#include <string.h>
#include <stdint.h>

int main() {
    uint8_t boot[512] = {0};
    int i = 0;
    
    // === 16-bit Real Mode Code ===
    
    // cli - 禁用中断
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
    
    // sti - 启用中断
    boot[i++] = 0xFB;
    
    // === 清屏 (蓝色背景) ===
    // mov ax, 0x0003 (设置 80x25 文本模式)
    boot[i++] = 0xB8; boot[i++] = 0x03; boot[i++] = 0x00;
    // int 0x10
    boot[i++] = 0xCD; boot[i++] = 0x10;
    
    // mov ah, 0x06 (滚动窗口/清屏)
    boot[i++] = 0xB4; boot[i++] = 0x06;
    // mov al, 0x00 (整个窗口)
    boot[i++] = 0xB0; boot[i++] = 0x00;
    // mov bh, 0x1F (蓝底白字)
    boot[i++] = 0xB7; boot[i++] = 0x1F;
    // mov cx, 0x0000 (左上角)
    boot[i++] = 0xB9; boot[i++] = 0x00; boot[i++] = 0x00;
    // mov dx, 0x184F (右下角 24,79)
    boot[i++] = 0xBA; boot[i++] = 0x4F; boot[i++] = 0x18;
    // int 0x10
    boot[i++] = 0xCD; boot[i++] = 0x10;
    
    // === 打印字符串 ===
    // 字符串在 0x7C00 + 0x80 = 0x7C80
    
    // mov si, 0x7C80 (字符串地址)
    boot[i++] = 0xBE; boot[i++] = 0x80; boot[i++] = 0x7C;
    
    // print_loop:
    int print_loop = i;
    // lodsb (al = [si], si++)
    boot[i++] = 0xAC;
    // test al, al
    boot[i++] = 0x84; boot[i++] = 0xC0;
    // jz done
    boot[i++] = 0x74; 
    int jz_offset = i;
    boot[i++] = 0x00; // 稍后填充
    
    // mov ah, 0x0E (teletype output)
    boot[i++] = 0xB4; boot[i++] = 0x0E;
    // mov bx, 0x001F (蓝底白字)
    boot[i++] = 0xBB; boot[i++] = 0x1F; boot[i++] = 0x00;
    // int 0x10
    boot[i++] = 0xCD; boot[i++] = 0x10;
    // jmp print_loop
    boot[i++] = 0xEB;
    boot[i++] = (uint8_t)(print_loop - i - 1);
    
    // done:
    boot[jz_offset] = (uint8_t)(i - jz_offset - 1);
    
    // === 无限循环 ===
    // hlt
    boot[i++] = 0xF4;
    // jmp $-1
    boot[i++] = 0xEB; boot[i++] = 0xFD;
    
    // === 字符串数据 (从偏移 0x80 开始, 最多到 0x1FE) ===
    const char* msg = 
        "\r\n"
        " ================================\r\n"
        "    Val-OS v2.3 - Fate Engine\r\n"
        " ================================\r\n"
        " [Fate] Init...           OK\r\n"
        " [Fate] Quattro Pools  Ready\r\n"
        " *** Val-OS Active ***\r\n"
        "\r\n";
    
    // 复制字符串到 0x80 偏移 (只有约 180 字节可用)
    int msg_len = strlen(msg);
    if (msg_len > 380) msg_len = 380;  // 防止溢出
    memcpy(boot + 0x80, msg, msg_len);
    boot[0x80 + msg_len] = 0;  // null terminator
    
    // === Boot Signature ===
    boot[510] = 0x55;
    boot[511] = 0xAA;
    
    // 写入文件
    FILE* f = fopen("valos_boot.img", "wb");
    if (!f) {
        fprintf(stderr, "Cannot create valos_boot.img\n");
        return 1;
    }
    fwrite(boot, 1, 512, f);
    
    // 填充到软盘大小 (1.44MB) 的前几个扇区
    uint8_t zero[512] = {0};
    for (int s = 1; s < 20; s++) {
        fwrite(zero, 1, 512, f);
    }
    
    fclose(f);
    
    printf("Generated valos_boot.img (10240 bytes)\n");
    printf("Run with: qemu-system-i386 -fda valos_boot.img\n");
    
    return 0;
}
