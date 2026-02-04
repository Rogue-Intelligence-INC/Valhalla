; ═══════════════════════════════════════════════════════════════════
; 🔮 Val-OS Fate Bootloader
; x86 Real Mode Bootloader for QEMU
; ═══════════════════════════════════════════════════════════════════

[BITS 16]
[ORG 0x7C00]

start:
    ; 设置段寄存器
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    ; 清屏
    call clear_screen
    
    ; 设置蓝色背景
    mov ah, 0x06
    xor al, al
    mov bh, 0x1F        ; 蓝底白字
    xor cx, cx
    mov dx, 0x184F
    int 0x10

    ; 显示 Val-OS Logo
    mov si, logo_top
    mov dh, 2           ; 行号
    call print_centered
    
    mov si, logo_name
    inc dh
    call print_centered
    
    mov si, logo_bottom
    inc dh
    call print_centered
    
    mov si, version
    add dh, 2
    call print_centered

    ; Fate 启动序列
    mov si, fate_wake
    mov dh, 8
    call print_at
    call delay
    
    mov si, fate_detect_cpu
    inc dh
    call print_at
    call delay
    
    mov si, fate_detect_mem
    inc dh
    call print_at
    call delay
    
    mov si, fate_detect_storage
    inc dh
    call print_at
    call delay
    
    mov si, fate_init_quattro
    inc dh
    call print_at
    call delay
    
    ; Quattro 四元池
    mov si, quattro_blackhole
    add dh, 2
    call print_at
    call short_delay
    
    mov si, quattro_meshbrain
    inc dh
    call print_at
    call short_delay
    
    mov si, quattro_multinova
    inc dh
    call print_at
    call short_delay
    
    mov si, quattro_baseforce
    inc dh
    call print_at
    call short_delay

    ; 内存布局
    mov si, memory_layout
    add dh, 2
    call print_at
    call delay
    
    ; 内核加载
    mov si, loading_kernel
    inc dh
    call print_at
    call delay
    
    ; 完成
    mov si, boot_complete
    add dh, 2
    call print_at
    
    mov si, press_key
    mov dh, 23
    call print_centered

    ; 等待按键
    xor ah, ah
    int 0x16
    
    ; 重启
    int 0x19

; ═══════════════════════════════════════════════════════════════════
; 函数
; ═══════════════════════════════════════════════════════════════════

clear_screen:
    mov ah, 0x00
    mov al, 0x03
    int 0x10
    ret

; 在指定位置打印字符串 (dh = 行, si = 字符串)
print_at:
    push ax
    push bx
    mov ah, 0x02
    xor bh, bh
    mov dl, 2           ; 列
    int 0x10
    call print_string
    pop bx
    pop ax
    ret

; 居中打印 (dh = 行, si = 字符串)
print_centered:
    push ax
    push bx
    push cx
    ; 计算字符串长度
    push si
    xor cx, cx
.count:
    lodsb
    test al, al
    jz .done_count
    inc cx
    jmp .count
.done_count:
    pop si
    ; 计算起始列 (40 - len/2)
    mov ax, 80
    sub ax, cx
    shr ax, 1
    mov dl, al
    mov ah, 0x02
    xor bh, bh
    int 0x10
    call print_string
    pop cx
    pop bx
    pop ax
    ret

print_string:
    push ax
    push bx
    mov ah, 0x0E
    mov bx, 0x001F      ; 蓝底白字
.loop:
    lodsb
    test al, al
    jz .done
    int 0x10
    jmp .loop
.done:
    pop bx
    pop ax
    ret

delay:
    push cx
    mov cx, 0x0008
.outer:
    push cx
    mov cx, 0xFFFF
.inner:
    nop
    loop .inner
    pop cx
    loop .outer
    pop cx
    ret

short_delay:
    push cx
    mov cx, 0x0002
.outer:
    push cx
    mov cx, 0xFFFF
.inner:
    nop
    loop .inner
    pop cx
    loop .outer
    pop cx
    ret

; ═══════════════════════════════════════════════════════════════════
; 数据
; ═══════════════════════════════════════════════════════════════════

logo_top:       db '======================================', 0
logo_name:      db '       Val-OS v2.3 - Fate Engine      ', 0
logo_bottom:    db '======================================', 0
version:        db 'Wave Language Self-Hosting OS', 0

fate_wake:          db '[Fate] Waking up...                        OK', 0
fate_detect_cpu:    db '[Fate] Detecting CPU: x86_64               OK', 0
fate_detect_mem:    db '[Fate] Detecting Memory: 128 MB            OK', 0
fate_detect_storage: db '[Fate] Detecting Storage: HDD              OK', 0
fate_init_quattro:  db '[Fate] Initializing Quattro Memory Pools...', 0

quattro_blackhole:  db '  -> BlackHole  (Extreme Data)       Ready', 0
quattro_meshbrain:  db '  -> MeshBrain  (Cognitive Data)     Ready', 0
quattro_multinova:  db '  -> MultiNova  (Large Scale)        Ready', 0
quattro_baseforce:  db '  -> BaseForce  (Fragment Opt)       Ready', 0

memory_layout:      db '[Fate] Memory layout decided dynamically   OK', 0
loading_kernel:     db '[Fate] Loading Val-OS kernel...            OK', 0

boot_complete:      db '*** Val-OS Boot Complete - Fate Active ***', 0
press_key:          db 'Press any key to reboot...', 0

; 填充到 510 字节
times 510-($-$$) db 0

; Boot signature
dw 0xAA55
