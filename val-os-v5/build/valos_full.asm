; ═══════════════════════════════════════════════════════════════════
; Val-OS v5.0 - Full System with Keyboard/Mouse Support
; Wave-inspired bare metal OS
; ═══════════════════════════════════════════════════════════════════

[BITS 16]
[ORG 0x7C00]

; ───────────────────────────────────────────────────────────────────
; Boot Sector
; ───────────────────────────────────────────────────────────────────

start:
    ; Set up segments
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    
    ; Save boot drive
    mov [boot_drive], dl
    
    ; Load more sectors
    mov ah, 0x02        ; Read sectors
    mov al, 30          ; Number of sectors
    mov ch, 0           ; Cylinder 0
    mov cl, 2           ; Start from sector 2
    mov dh, 0           ; Head 0
    mov dl, [boot_drive]
    mov bx, 0x7E00      ; Load to 0x7E00
    int 0x13
    jc disk_error
    
    ; Jump to loaded code
    jmp 0x0000:0x7E00

disk_error:
    mov si, disk_err_msg
    call print_string
    jmp $

print_string:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp print_string
.done:
    ret

boot_drive: db 0
disk_err_msg: db "Disk error!", 0

; Padding and boot signature
times 510 - ($ - $$) db 0
dw 0xAA55

; ═══════════════════════════════════════════════════════════════════
; Second Stage - Main System (loaded at 0x7E00)
; ═══════════════════════════════════════════════════════════════════

second_stage:
    ; Set video mode - 320x200 256 colors (Mode 13h)
    mov ax, 0x0013
    int 0x10
    
    ; Draw gradient background
    call draw_background
    
    ; Draw Val-OS logo
    call draw_logo
    
    ; Draw login box
    call draw_login_box
    
    ; Initialize mouse
    call init_mouse
    
    ; Main loop
main_loop:
    ; Check keyboard
    mov ah, 0x01
    int 0x16
    jz .check_mouse
    
    ; Key pressed
    mov ah, 0x00
    int 0x16
    call handle_key
    
.check_mouse:
    ; Update mouse
    call update_mouse
    call draw_cursor
    
    ; Small delay
    mov cx, 0x0FFF
.delay:
    loop .delay
    
    jmp main_loop

; ───────────────────────────────────────────────────────────────────
; Draw Background (gradient)
; ───────────────────────────────────────────────────────────────────

draw_background:
    push es
    mov ax, 0xA000
    mov es, ax
    
    xor di, di
    mov cx, 200         ; 200 rows
    
.row_loop:
    push cx
    
    ; Calculate color based on row
    mov ax, 200
    sub ax, cx
    shr ax, 3           ; Divide by 8
    add al, 16          ; Base color
    
    mov cx, 320         ; 320 pixels per row
    rep stosb
    
    pop cx
    loop .row_loop
    
    pop es
    ret

; ───────────────────────────────────────────────────────────────────
; Draw Logo
; ───────────────────────────────────────────────────────────────────

draw_logo:
    ; Draw "Val-OS" text at top center
    mov si, logo_text
    mov dh, 3           ; Row 3
    mov dl, 15          ; Column 15
    mov bl, 0x0F        ; White
    call print_at
    
    mov si, version_text
    mov dh, 5
    mov dl, 12
    mov bl, 0x07        ; Gray
    call print_at
    
    ret

print_at:
    mov ah, 0x02
    mov bh, 0
    int 0x10
.loop:
    lodsb
    or al, al
    jz .done
    mov ah, 0x09
    mov cx, 1
    int 0x10
    
    ; Move cursor right
    mov ah, 0x02
    inc dl
    int 0x10
    jmp .loop
.done:
    ret

logo_text: db "Val-OS v5.0", 0
version_text: db "Wave * Fate * Quattro", 0

; ───────────────────────────────────────────────────────────────────
; Draw Login Box
; ───────────────────────────────────────────────────────────────────

draw_login_box:
    ; Draw box background
    mov ax, 0xA000
    mov es, ax
    
    ; Box at (100, 60) size (120, 80)
    mov di, 60 * 320 + 100
    mov cx, 80          ; Height
    
.box_row:
    push cx
    push di
    
    mov al, 0x17        ; Dark blue
    mov cx, 120         ; Width
    rep stosb
    
    pop di
    add di, 320
    pop cx
    loop .box_row
    
    ; Draw text
    mov si, login_title
    mov dh, 9
    mov dl, 16
    mov bl, 0x0F
    call print_at
    
    mov si, user_label
    mov dh, 11
    mov dl, 14
    mov bl, 0x0B
    call print_at
    
    ; Draw input box
    call draw_input_box
    
    mov si, pass_label
    mov dh, 14
    mov dl, 14
    mov bl, 0x0B
    call print_at
    
    ; Draw password box
    mov di, 15 * 8 * 320 + 110
    call draw_small_box
    
    ; Draw login button
    call draw_login_button
    
    ret

draw_input_box:
    mov ax, 0xA000
    mov es, ax
    mov di, 12 * 8 * 320 + 110
    mov cx, 8
.ib_row:
    push cx
    push di
    mov al, 0x00        ; Black
    mov cx, 100
    rep stosb
    pop di
    add di, 320
    pop cx
    loop .ib_row
    ret

draw_small_box:
    mov ax, 0xA000
    mov es, ax
    mov cx, 8
.sb_row:
    push cx
    push di
    mov al, 0x00
    mov cx, 100
    rep stosb
    pop di
    add di, 320
    pop cx
    loop .sb_row
    ret

draw_login_button:
    mov ax, 0xA000
    mov es, ax
    mov di, 17 * 8 * 320 + 130
    mov cx, 16
.btn_row:
    push cx
    push di
    mov al, 0x02        ; Green
    mov cx, 60
    rep stosb
    pop di
    add di, 320
    pop cx
    loop .btn_row
    
    ; Button text
    mov si, login_btn
    mov dh, 18
    mov dl, 18
    mov bl, 0x0F
    call print_at
    ret

login_title: db "Login", 0
user_label: db "Username:", 0
pass_label: db "Password:", 0
login_btn: db "LOGIN", 0

; ───────────────────────────────────────────────────────────────────
; Mouse Support
; ───────────────────────────────────────────────────────────────────

mouse_x: dw 160
mouse_y: dw 100
mouse_buttons: db 0
mouse_old_pixel: db 0

init_mouse:
    ; Initialize mouse driver
    mov ax, 0x0000
    int 0x33
    
    ; Show mouse cursor
    mov ax, 0x0001
    int 0x33
    
    ; Set mouse bounds
    mov ax, 0x0007      ; Set X bounds
    mov cx, 0           ; Min X
    mov dx, 319         ; Max X
    int 0x33
    
    mov ax, 0x0008      ; Set Y bounds
    mov cx, 0           ; Min Y
    mov dx, 199         ; Max Y
    int 0x33
    
    ret

update_mouse:
    ; Get mouse status
    mov ax, 0x0003
    int 0x33
    
    ; CX = X, DX = Y, BX = buttons
    mov [mouse_buttons], bl
    
    ; Scale coordinates (mouse returns 0-639 for X)
    shr cx, 1
    mov [mouse_x], cx
    mov [mouse_y], dx
    
    ret

draw_cursor:
    ; Restore old pixel
    mov ax, 0xA000
    mov es, ax
    
    ; Calculate position
    mov ax, [mouse_y]
    mov bx, 320
    mul bx
    add ax, [mouse_x]
    mov di, ax
    
    ; Draw cursor (simple cross)
    mov al, 0x0F        ; White
    mov [es:di], al
    mov [es:di+1], al
    mov [es:di-1], al
    mov [es:di+320], al
    mov [es:di-320], al
    
    ret

; ───────────────────────────────────────────────────────────────────
; Keyboard Handler
; ───────────────────────────────────────────────────────────────────

input_buffer: times 32 db 0
input_pos: dw 0
current_field: db 0     ; 0=username, 1=password

handle_key:
    ; AL = ASCII, AH = scan code
    
    ; Enter key
    cmp al, 13
    je .enter_pressed
    
    ; Tab key - switch fields
    cmp al, 9
    je .tab_pressed
    
    ; Backspace
    cmp al, 8
    je .backspace
    
    ; Printable character
    cmp al, 32
    jl .done
    cmp al, 126
    jg .done
    
    ; Add to buffer
    mov bx, [input_pos]
    cmp bx, 30
    jge .done
    
    mov [input_buffer + bx], al
    inc word [input_pos]
    
    ; Display character
    cmp byte [current_field], 1
    je .show_asterisk
    
    ; Show actual character
    mov ah, 0x0E
    int 0x10
    jmp .done
    
.show_asterisk:
    mov al, '*'
    mov ah, 0x0E
    int 0x10
    jmp .done
    
.enter_pressed:
    ; Check if login
    cmp byte [current_field], 1
    jne .switch_to_pass
    
    ; Attempt login - just show desktop
    call show_desktop
    jmp .done
    
.switch_to_pass:
    mov byte [current_field], 1
    mov word [input_pos], 0
    mov dh, 15
    mov dl, 15
    mov ah, 0x02
    mov bh, 0
    int 0x10
    jmp .done
    
.tab_pressed:
    xor byte [current_field], 1
    mov word [input_pos], 0
    jmp .done
    
.backspace:
    cmp word [input_pos], 0
    je .done
    dec word [input_pos]
    
    ; Erase character
    mov ah, 0x0E
    mov al, 8
    int 0x10
    mov al, ' '
    int 0x10
    mov al, 8
    int 0x10
    
.done:
    ret

; ───────────────────────────────────────────────────────────────────
; Desktop
; ───────────────────────────────────────────────────────────────────

show_desktop:
    ; Clear screen with gradient
    call draw_background
    
    ; Draw desktop icons
    call draw_desktop_icons
    
    ; Draw taskbar
    call draw_taskbar
    
    ; Desktop loop
.desktop_loop:
    ; Check keyboard
    mov ah, 0x01
    int 0x16
    jz .check_mouse_desktop
    
    mov ah, 0x00
    int 0x16
    call handle_desktop_key
    
.check_mouse_desktop:
    call update_mouse
    call draw_cursor
    
    ; Check click
    test byte [mouse_buttons], 1
    jz .no_click
    call handle_click
    
.no_click:
    mov cx, 0x0FFF
.delay2:
    loop .delay2
    
    jmp .desktop_loop

draw_desktop_icons:
    ; Files icon
    mov si, icon_files
    mov dh, 3
    mov dl, 2
    mov bl, 0x0F
    call print_at
    
    mov si, label_files
    mov dh, 4
    mov dl, 2
    call print_at
    
    ; Terminal icon
    mov si, icon_term
    mov dh, 7
    mov dl, 2
    call print_at
    
    mov si, label_term
    mov dh, 8
    mov dl, 1
    call print_at
    
    ; Browser icon
    mov si, icon_browser
    mov dh, 11
    mov dl, 2
    call print_at
    
    mov si, label_browser
    mov dh, 12
    mov dl, 1
    call print_at
    
    ; Settings icon
    mov si, icon_settings
    mov dh, 15
    mov dl, 2
    call print_at
    
    mov si, label_settings
    mov dh, 16
    mov dl, 1
    call print_at
    
    ret

icon_files: db "[F]", 0
label_files: db "Files", 0
icon_term: db "[T]", 0
label_term: db "Terminal", 0
icon_browser: db "[B]", 0
label_browser: db "Browser", 0
icon_settings: db "[S]", 0
label_settings: db "Settings", 0

draw_taskbar:
    mov ax, 0xA000
    mov es, ax
    
    ; Taskbar at bottom
    mov di, 190 * 320
    mov cx, 10 * 320
    mov al, 0x08        ; Dark gray
    rep stosb
    
    ; Wave logo
    mov si, taskbar_logo
    mov dh, 24
    mov dl, 1
    mov bl, 0x0B
    call print_at
    
    ; Clock
    mov si, taskbar_clock
    mov dh, 24
    mov dl, 35
    mov bl, 0x0F
    call print_at
    
    ret

taskbar_logo: db "Wave", 0
taskbar_clock: db "12:34", 0

handle_desktop_key:
    ; Super key (Windows key) - show launcher
    cmp ah, 0x5B        ; Left Windows
    je show_launcher
    cmp ah, 0x5C        ; Right Windows
    je show_launcher
    
    ; F - Files
    cmp al, 'f'
    je open_files
    cmp al, 'F'
    je open_files
    
    ; T - Terminal
    cmp al, 't'
    je open_terminal
    cmp al, 'T'
    je open_terminal
    
    ; B - Browser
    cmp al, 'b'
    je open_browser
    cmp al, 'B'
    je open_browser
    
    ; S - Settings
    cmp al, 's'
    je open_settings
    cmp al, 'S'
    je open_settings
    
    ret

handle_click:
    ; Check icon positions
    mov ax, [mouse_x]
    mov bx, [mouse_y]
    
    ; Files (x < 50, y 20-50)
    cmp ax, 50
    jg .not_files
    cmp bx, 20
    jl .not_files
    cmp bx, 50
    jg .not_files
    jmp open_files
    
.not_files:
    ; Terminal (x < 50, y 50-80)
    cmp ax, 50
    jg .not_term
    cmp bx, 50
    jl .not_term
    cmp bx, 80
    jg .not_term
    jmp open_terminal
    
.not_term:
    ; Browser (x < 50, y 80-110)
    cmp ax, 50
    jg .not_browser
    cmp bx, 80
    jl .not_browser
    cmp bx, 110
    jg .not_browser
    jmp open_browser
    
.not_browser:
    ret

; ───────────────────────────────────────────────────────────────────
; Applications
; ───────────────────────────────────────────────────────────────────

show_launcher:
    ; Draw launcher overlay
    mov ax, 0xA000
    mov es, ax
    
    mov di, 40 * 320 + 80
    mov cx, 120
.launcher_row:
    push cx
    push di
    mov al, 0x01        ; Blue
    mov cx, 160
    rep stosb
    pop di
    add di, 320
    pop cx
    loop .launcher_row
    
    ; Title
    mov si, launcher_title
    mov dh, 6
    mov dl, 14
    mov bl, 0x0F
    call print_at
    
    ; Apps list
    mov si, launcher_apps
    mov dh, 8
    mov dl, 12
    call print_at
    
    ; Wait for key
    mov ah, 0x00
    int 0x16
    
    ; Redraw desktop
    jmp show_desktop

launcher_title: db "Applications", 0
launcher_apps: db "F-Files T-Term B-Browse S-Set", 0

open_files:
    call draw_window
    mov si, files_title
    mov dh, 5
    mov dl, 12
    mov bl, 0x0F
    call print_at
    
    mov si, files_content
    mov dh, 8
    mov dl, 10
    mov bl, 0x07
    call print_at
    
    call wait_close
    jmp show_desktop

files_title: db "Files", 0
files_content: db "Documents  Downloads  Pictures", 0

open_terminal:
    call draw_window
    mov si, term_title
    mov dh, 5
    mov dl, 12
    mov bl, 0x0F
    call print_at
    
    mov si, term_prompt
    mov dh, 8
    mov dl, 10
    mov bl, 0x0A
    call print_at
    
    call wait_close
    jmp show_desktop

term_title: db "Terminal", 0
term_prompt: db "admin@valos:~$ _", 0

open_browser:
    call draw_window
    mov si, browser_title
    mov dh, 5
    mov dl, 12
    mov bl, 0x0F
    call print_at
    
    ; Address bar
    mov si, browser_url
    mov dh, 7
    mov dl, 10
    mov bl, 0x07
    call print_at
    
    ; Content
    mov si, browser_content
    mov dh, 10
    mov dl, 12
    mov bl, 0x0F
    call print_at
    
    mov si, browser_wave
    mov dh, 12
    mov dl, 10
    mov bl, 0x0B
    call print_at
    
    call wait_close
    jmp show_desktop

browser_title: db "Browser", 0
browser_url: db "[wave://home]", 0
browser_content: db "Welcome to Val-OS", 0
browser_wave: db "Wave Language + Fate Engine", 0

open_settings:
    call draw_window
    mov si, settings_title
    mov dh, 5
    mov dl, 12
    mov bl, 0x0F
    call print_at
    
    mov si, settings_opts
    mov dh, 8
    mov dl, 10
    mov bl, 0x07
    call print_at
    
    call wait_close
    jmp show_desktop

settings_title: db "Settings", 0
settings_opts: db "Display  Sound  Network  About", 0

draw_window:
    mov ax, 0xA000
    mov es, ax
    
    ; Window at (60, 30) size (200, 140)
    mov di, 30 * 320 + 60
    mov cx, 140
.win_row:
    push cx
    push di
    mov al, 0x17        ; Blue
    mov cx, 200
    rep stosb
    pop di
    add di, 320
    pop cx
    loop .win_row
    
    ; Title bar
    mov di, 30 * 320 + 60
    mov cx, 12
.title_row:
    push cx
    push di
    mov al, 0x01        ; Dark blue
    mov cx, 200
    rep stosb
    pop di
    add di, 320
    pop cx
    loop .title_row
    
    ; Close button text
    mov si, close_btn
    mov dh, 4
    mov dl, 30
    mov bl, 0x0C
    call print_at
    
    ret

close_btn: db "[X]", 0

wait_close:
    ; Wait for any key
    mov ah, 0x00
    int 0x16
    ret

; Padding
times 15360 - ($ - $$) db 0
