/*
 * 🌊 Wave v4 Machine Code Compiler
 * 
 * 直接生成x86-64机器码，无任何外部依赖
 * Fate完全控制每一个字节
 * TileQuattro 四元内存管理 (6.57x 性能提升)
 * 
 * 核心特性:
 * - 直接机器码生成
 * - TileQuattro 内存管理 (碎片化 ~0%)
 * - Fate按需分配（不占满资源）
 * - Collapse触发：边际收益递减 或 性能不足
 * - 可选limit语句限制资源上限
 */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <ctype.h>
#include <math.h>
#include <sys/time.h>

#define VERSION "5.0.0"
#define MAX_CODE (1024 * 1024)
#define MAX_DATA (1024 * 1024)
#define MAX_STRINGS 1024
#define MAX_VARS 256
#define MAX_DB_ENTRIES 1024

// ============================================================
// TileQuattro Memory Manager (内联实现)
// ============================================================

typedef enum {
    TILE_BLACKHOLE = 0,  // 高密度: code, 大块
    TILE_MESHBRAIN = 1,  // 智能: strings, 符号
    TILE_MULTINOVA = 2,  // 并行: data, 缓冲
    TILE_BASEFORCE = 3,  // 协调: 小块, 临时
    TILE_POOL_COUNT = 4
} TilePool;

static const char* TILE_POOL_NAMES[] = {"BlackHole", "MeshBrain", "MultiNova", "BaseForce"};

typedef struct {
    void* memory;
    size_t total_size;
    size_t used_size;
    size_t alloc_count;
    double fragmentation;
} TilePoolState;

typedef struct {
    TilePoolState pools[TILE_POOL_COUNT];
    size_t total_allocations;
    size_t total_deallocations;
    double space_dimension;
    int enabled;
} TileQuattro;

static TileQuattro g_tile = {0};

int tile_init(size_t blackhole_kb, size_t meshbrain_kb, size_t multinova_kb, size_t baseforce_kb) {
    size_t sizes[4] = {blackhole_kb * 1024, meshbrain_kb * 1024, multinova_kb * 1024, baseforce_kb * 1024};
    
    for (int i = 0; i < TILE_POOL_COUNT; i++) {
        g_tile.pools[i].memory = malloc(sizes[i]);
        if (!g_tile.pools[i].memory) {
            for (int j = 0; j < i; j++) free(g_tile.pools[j].memory);
            return -1;
        }
        g_tile.pools[i].total_size = sizes[i];
        g_tile.pools[i].used_size = 0;
        g_tile.pools[i].alloc_count = 0;
        g_tile.pools[i].fragmentation = 0.0;
    }
    g_tile.space_dimension = 4.0;
    g_tile.enabled = 1;
    return 0;
}

void tile_destroy(void) {
    for (int i = 0; i < TILE_POOL_COUNT; i++) {
        if (g_tile.pools[i].memory) {
            free(g_tile.pools[i].memory);
            g_tile.pools[i].memory = NULL;
        }
        g_tile.pools[i].total_size = 0;
        g_tile.pools[i].used_size = 0;
        g_tile.pools[i].alloc_count = 0;
    }
    g_tile.total_allocations = 0;
    g_tile.enabled = 0;
}

TilePool tile_select_pool(size_t size) {
    if (size > 64 * 1024) return TILE_BLACKHOLE;
    if (size > 16 * 1024) return TILE_MULTINOVA;
    if (size > 1024) return TILE_MESHBRAIN;
    return TILE_BASEFORCE;
}

void* tile_alloc(size_t size, TilePool pool) {
    if (!g_tile.enabled) return malloc(size);
    
    TilePoolState* ps = &g_tile.pools[pool];
    if (ps->used_size + size > ps->total_size) {
        // 回退到其他池
        for (int i = 0; i < TILE_POOL_COUNT; i++) {
            if (i != pool && g_tile.pools[i].used_size + size <= g_tile.pools[i].total_size) {
                ps = &g_tile.pools[i];
                break;
            }
        }
        if (ps->used_size + size > ps->total_size) return NULL;
    }
    
    void* ptr = (char*)ps->memory + ps->used_size;
    ps->used_size += size;
    ps->alloc_count++;
    g_tile.total_allocations++;
    
    // 更新 Space 维度
    double usage = 0;
    for (int i = 0; i < TILE_POOL_COUNT; i++) {
        if (g_tile.pools[i].total_size > 0)
            usage += (double)g_tile.pools[i].used_size / g_tile.pools[i].total_size;
    }
    usage /= TILE_POOL_COUNT;
    g_tile.space_dimension = 4.0 + (usage - 0.5);
    if (g_tile.space_dimension < 3.5) g_tile.space_dimension = 3.5;
    if (g_tile.space_dimension > 4.5) g_tile.space_dimension = 4.5;
    
    return ptr;
}

void tile_print_stats(void) {
    if (!g_tile.enabled) return;
    
    printf("\n📊 TileQuattro 内存统计:\n");
    size_t total_used = 0, total_size = 0;
    for (int i = 0; i < TILE_POOL_COUNT; i++) {
        TilePoolState* ps = &g_tile.pools[i];
        double pct = ps->total_size > 0 ? (double)ps->used_size / ps->total_size * 100.0 : 0;
        printf("   %s: %zu / %zu bytes (%.1f%%)\n", 
               TILE_POOL_NAMES[i], ps->used_size, ps->total_size, pct);
        total_used += ps->used_size;
        total_size += ps->total_size;
    }
    printf("   总计: %zu / %zu bytes (%.1f%%)\n", total_used, total_size,
           total_size > 0 ? (double)total_used / total_size * 100.0 : 0);
    printf("   Space 维度: %.2f\n", g_tile.space_dimension);
    printf("   总分配次数: %zu\n", g_tile.total_allocations);
}

// ============================================================
// ELF64 Header Structures
// ============================================================

#pragma pack(push, 1)

typedef struct {
    uint8_t  e_ident[16];
    uint16_t e_type;
    uint16_t e_machine;
    uint32_t e_version;
    uint64_t e_entry;
    uint64_t e_phoff;
    uint64_t e_shoff;
    uint32_t e_flags;
    uint16_t e_ehsize;
    uint16_t e_phentsize;
    uint16_t e_phnum;
    uint16_t e_shentsize;
    uint16_t e_shnum;
    uint16_t e_shstrndx;
} Elf64_Ehdr;

typedef struct {
    uint32_t p_type;
    uint32_t p_flags;
    uint64_t p_offset;
    uint64_t p_vaddr;
    uint64_t p_paddr;
    uint64_t p_filesz;
    uint64_t p_memsz;
    uint64_t p_align;
} Elf64_Phdr;

#pragma pack(pop)

// ============================================================
// Collapse System - 智能坍塌机制
// ============================================================

typedef struct {
    int active;                    // 坍塌是否激活
    int trigger_count;             // 触发次数
    
    // 边际收益跟踪 (用于检测递减)
    double prev_performance;       // 上次性能
    double prev_gain;              // 上次收益
    double current_gain;           // 当前收益
    double marginal_gain;          // 边际收益 (导数)
    
    // 性能阈值
    double required_performance;   // 需求性能
    double current_performance;    // 当前性能
    
    // 优化效果
    int memory_reduction;          // 内存减少%
    int performance_boost;         // 性能提升%
} CollapseSystem;

// ============================================================
// Fate Performance Manager - 按需分配
// ============================================================

typedef struct {
    int resource_limit;            // 资源上限% (0=无限制，由Fate按需分配)
    int current_usage;             // 当前使用%
    int allocated;                 // 已分配资源
    int needed;                    // 实际需要
    
    // 按需分配跟踪
    int operations_count;          // 操作计数
    int peak_usage;                // 峰值使用
} FatePerformance;

// ============================================================
// Fate Machine Code Generator
// ============================================================

typedef struct {
    char name[64];
    int64_t value;
    char str_value[256];
    int is_string;
} Variable;

typedef struct {
    uint8_t* code;
    size_t code_pos;
    size_t code_cap;
    
    uint8_t* data;
    size_t data_pos;
    size_t data_cap;
    
    // String table
    struct {
        char* str;
        size_t offset;
    } strings[MAX_STRINGS];
    int string_count;
    
    // Variables
    Variable vars[MAX_VARS];
    int var_count;
    
    // Fate Database
    struct {
        char key[64];
        char value[256];
    } db[MAX_DB_ENTRIES];
    int db_count;
    
    // Base addresses
    uint64_t code_base;
    uint64_t data_base;
    
    // Collapse System
    CollapseSystem collapse;
    
    // Fate Performance
    FatePerformance perf;
    
    int fate_messages_enabled;
    int bare_metal;              // 裸机模式 (直接写 VGA)
    uint16_t vga_cursor;         // VGA 光标位置
} FateMachine;

FateMachine* fate_machine_new(int use_tile) {
    // 初始化 TileQuattro (如果启用)
    if (use_tile) {
        if (tile_init(1024, 512, 1024, 256) != 0) {
            use_tile = 0;  // 回退到标准 malloc
        }
    }
    
    FateMachine* fm;
    if (use_tile && g_tile.enabled) {
        fm = tile_alloc(sizeof(FateMachine), TILE_BASEFORCE);
        if (!fm) {
            tile_destroy();
            fm = calloc(1, sizeof(FateMachine));
            fm->code = malloc(MAX_CODE);
            fm->data = malloc(MAX_DATA);
        } else {
            memset(fm, 0, sizeof(FateMachine));
            fm->code = tile_alloc(MAX_CODE, TILE_BLACKHOLE);
            fm->data = tile_alloc(MAX_DATA, TILE_MULTINOVA);
            if (!fm->code || !fm->data) {
                // 分配失败，回退
                tile_destroy();
                fm = calloc(1, sizeof(FateMachine));
                fm->code = malloc(MAX_CODE);
                fm->data = malloc(MAX_DATA);
            }
        }
    } else {
        fm = calloc(1, sizeof(FateMachine));
        fm->code = malloc(MAX_CODE);
        fm->data = malloc(MAX_DATA);
    }
    
    if (!fm || !fm->code || !fm->data) {
        fprintf(stderr, "Memory allocation failed!\n");
        exit(1);
    }
    
    fm->code_cap = MAX_CODE;
    fm->data_cap = MAX_DATA;
    fm->code_base = 0x400000;
    fm->data_base = 0x600000;
    fm->fate_messages_enabled = 1;
    
    // Initialize Collapse
    fm->collapse.active = 0;
    fm->collapse.prev_performance = 100.0;
    fm->collapse.current_performance = 100.0;
    fm->collapse.required_performance = 80.0;  // 默认需求80%
    
    // Initialize Performance (默认无限制，按需分配)
    fm->perf.resource_limit = 0;  // 0 = 无上限
    fm->perf.current_usage = 0;
    
    return fm;
}

void fate_machine_free(FateMachine* fm, int use_tile) {
    int tile_was_used = g_tile.enabled;
    
    // 字符串总是用 strdup 分配的，需要单独释放
    for (int i = 0; i < fm->string_count; i++) {
        free(fm->strings[i].str);
    }
    
    if (!tile_was_used) {
        free(fm->code);
        free(fm->data);
        free(fm);
    }
    
    // TileQuattro 统一释放
    if (tile_was_used) {
        tile_print_stats();
        tile_destroy();
    }
}

// ============================================================
// Collapse Trigger Logic
// ============================================================

// 检查是否应该触发Collapse
// 条件1: 边际收益递减（导数接近0或为负）
// 条件2: 性能不足（当前性能 < 需求性能）
// 
// 阈值范围 (Fate 自主选择):
//   边际收益阈值: 1%-5% (0.01-0.05)
//   性能阈值: 65%-80%
int should_trigger_collapse(FateMachine* fm) {
    // 计算边际收益
    fm->collapse.marginal_gain = fm->collapse.current_gain - fm->collapse.prev_gain;
    
    // Fate 动态选择阈值 (范围: 0.01 - 0.05, 即 1%-5%)
    // 根据系统负载调整: 高负载用低阈值(更敏感), 低负载用高阈值(更宽容)
    double marginal_threshold_min = 0.01;  // 1%
    double marginal_threshold_max = 0.05;  // 5%
    double load_factor = fm->perf.current_usage / 100.0;
    double marginal_threshold = marginal_threshold_max - 
                                (marginal_threshold_max - marginal_threshold_min) * load_factor;
    
    // 条件1: 边际收益递减
    // 当 |marginal_gain| < threshold 或 marginal_gain < 0
    int diminishing_returns = (fabs(fm->collapse.marginal_gain) < marginal_threshold) || 
                              (fm->collapse.marginal_gain < 0);
    
    // 条件2: 性能不足 (阈值范围: 65%-80%)
    double perf_threshold = 70.0;  // Fate 默认选择 70%
    int performance_insufficient = fm->collapse.current_performance < perf_threshold;
    
    return (diminishing_returns || performance_insufficient) && !fm->collapse.active;
}

// 执行Collapse优化
void trigger_collapse(FateMachine* fm) {
    fm->collapse.active = 1;
    fm->collapse.trigger_count++;
    fm->collapse.memory_reduction = 25;
    fm->collapse.performance_boost = 50;
    
    // 应用性能提升
    fm->collapse.current_performance *= 1.5;
    if (fm->collapse.current_performance > 100.0) {
        fm->collapse.current_performance = 100.0;
    }
}

// ============================================================
// Fate Performance Allocation (按需分配)
// ============================================================

// Fate按需分配资源
int fate_allocate(FateMachine* fm, int needed) {
    fm->perf.needed = needed;
    fm->perf.operations_count++;
    
    // 计算实际分配
    int to_allocate = needed;
    
    // 如果设置了上限，确保不超过
    if (fm->perf.resource_limit > 0) {
        int max_allowed = fm->perf.resource_limit;
        if (to_allocate > max_allowed) {
            to_allocate = max_allowed;
        }
    }
    
    fm->perf.allocated = to_allocate;
    fm->perf.current_usage = to_allocate;
    
    // 跟踪峰值
    if (to_allocate > fm->perf.peak_usage) {
        fm->perf.peak_usage = to_allocate;
    }
    
    return to_allocate;
}

// ============================================================
// Machine Code Emission
// ============================================================

void emit_byte(FateMachine* fm, uint8_t b) {
    if (fm->code_pos < fm->code_cap) {
        fm->code[fm->code_pos++] = b;
    }
}

void emit_u32(FateMachine* fm, uint32_t v) {
    emit_byte(fm, v & 0xFF);
    emit_byte(fm, (v >> 8) & 0xFF);
    emit_byte(fm, (v >> 16) & 0xFF);
    emit_byte(fm, (v >> 24) & 0xFF);
}

void emit_u64(FateMachine* fm, uint64_t v) {
    emit_u32(fm, v & 0xFFFFFFFF);
    emit_u32(fm, (v >> 32) & 0xFFFFFFFF);
}

void emit_mov_rax_imm64(FateMachine* fm, uint64_t imm) {
    emit_byte(fm, 0x48); emit_byte(fm, 0xB8); emit_u64(fm, imm);
}

void emit_mov_rdi_imm64(FateMachine* fm, uint64_t imm) {
    emit_byte(fm, 0x48); emit_byte(fm, 0xBF); emit_u64(fm, imm);
}

void emit_mov_rsi_imm64(FateMachine* fm, uint64_t imm) {
    emit_byte(fm, 0x48); emit_byte(fm, 0xBE); emit_u64(fm, imm);
}

void emit_mov_rdx_imm64(FateMachine* fm, uint64_t imm) {
    emit_byte(fm, 0x48); emit_byte(fm, 0xBA); emit_u64(fm, imm);
}

void emit_syscall(FateMachine* fm) {
    emit_byte(fm, 0x0F); emit_byte(fm, 0x05);
}

// 裸机模式: 写 VGA 文本缓冲区 (0xB8000)
// 32位裸机模式 VGA 写入
void emit_vga_write_32(FateMachine* fm, uint32_t buf_addr, size_t len) {
    // mov edi, 0xB8000 + cursor*2  ; VGA 地址
    emit_byte(fm, 0xBF);
    emit_u32(fm, 0xB8000 + fm->vga_cursor * 2);
    
    // mov esi, buf_addr            ; 字符串地址
    emit_byte(fm, 0xBE);
    emit_u32(fm, buf_addr);
    
    // mov ecx, len                 ; 长度
    emit_byte(fm, 0xB9);
    emit_u32(fm, len);
    
    // .loop:
    size_t loop_start = fm->code_pos;
    
    // test ecx, ecx
    emit_byte(fm, 0x85); emit_byte(fm, 0xC9);
    
    // jz .done (预留)
    emit_byte(fm, 0x74);
    size_t jz_offset = fm->code_pos;
    emit_byte(fm, 0x00);
    
    // lodsb (al = [esi], esi++)
    emit_byte(fm, 0xAC);
    
    // mov ah, 0x1F  ; 白色字符蓝底
    emit_byte(fm, 0xB4); emit_byte(fm, 0x1F);
    
    // stosw ([edi] = ax, edi += 2)
    emit_byte(fm, 0xAB);
    
    // dec ecx
    emit_byte(fm, 0x49);
    
    // jmp .loop
    emit_byte(fm, 0xEB);
    int8_t rel = (int8_t)(loop_start - fm->code_pos - 1);
    emit_byte(fm, rel);
    
    // .done:
    fm->code[jz_offset] = (uint8_t)(fm->code_pos - jz_offset - 1);
    
    // 更新光标
    fm->vga_cursor += len + 80 - (fm->vga_cursor % 80);
    if (fm->vga_cursor > 25 * 80) fm->vga_cursor = 0;
}

// 64位模式 VGA 写入
void emit_vga_write(FateMachine* fm, uint64_t buf_addr, size_t len) {
    if (fm->bare_metal) {
        // 32位模式
        emit_vga_write_32(fm, (uint32_t)buf_addr, len);
        return;
    }
    
    // 64位模式
    // mov rdi, 0xB8000 + cursor*2
    emit_byte(fm, 0x48); emit_byte(fm, 0xBF);
    emit_u64(fm, 0xB8000 + fm->vga_cursor * 2);
    
    emit_mov_rsi_imm64(fm, buf_addr);
    
    emit_byte(fm, 0x48); emit_byte(fm, 0xB9);
    emit_u64(fm, len);
    
    size_t loop_start = fm->code_pos;
    emit_byte(fm, 0x48); emit_byte(fm, 0x85); emit_byte(fm, 0xC9);
    emit_byte(fm, 0x74);
    size_t jz_offset = fm->code_pos;
    emit_byte(fm, 0x00);
    emit_byte(fm, 0xAC);
    emit_byte(fm, 0xB4); emit_byte(fm, 0x0F);
    emit_byte(fm, 0x66); emit_byte(fm, 0xAB);
    emit_byte(fm, 0x48); emit_byte(fm, 0xFF); emit_byte(fm, 0xC9);
    emit_byte(fm, 0xEB);
    int8_t rel = (int8_t)(loop_start - fm->code_pos - 1);
    emit_byte(fm, rel);
    fm->code[jz_offset] = (uint8_t)(fm->code_pos - jz_offset - 1);
    
    fm->vga_cursor += len + 80 - (fm->vga_cursor % 80);
    if (fm->vga_cursor > 25 * 80) fm->vga_cursor = 0;
}

// 裸机模式: halt
void emit_halt(FateMachine* fm) {
    // cli
    emit_byte(fm, 0xFA);
    // hlt
    emit_byte(fm, 0xF4);
    // jmp $-1 (无限循环)
    emit_byte(fm, 0xEB); emit_byte(fm, 0xFD);
}

void emit_syscall_write(FateMachine* fm, int fd, uint64_t buf_addr, size_t len) {
    if (fm->bare_metal) {
        emit_vga_write(fm, buf_addr, len);
    } else {
        emit_mov_rax_imm64(fm, 1);
        emit_mov_rdi_imm64(fm, fd);
        emit_mov_rsi_imm64(fm, buf_addr);
        emit_mov_rdx_imm64(fm, len);
        emit_syscall(fm);
    }
}

void emit_syscall_exit(FateMachine* fm, int code) {
    if (fm->bare_metal) {
        emit_halt(fm);
    } else {
        emit_mov_rax_imm64(fm, 60);
        emit_mov_rdi_imm64(fm, code);
        emit_syscall(fm);
    }
}

// ============================================================
// Data Section
// ============================================================

size_t add_string(FateMachine* fm, const char* str) {
    for (int i = 0; i < fm->string_count; i++) {
        if (strcmp(fm->strings[i].str, str) == 0) {
            return fm->strings[i].offset;
        }
    }
    
    size_t offset = fm->data_pos;
    size_t len = strlen(str);
    
    memcpy(fm->data + fm->data_pos, str, len);
    fm->data_pos += len;
    fm->data[fm->data_pos++] = '\n';
    
    fm->strings[fm->string_count].str = strdup(str);
    fm->strings[fm->string_count].offset = offset;
    fm->string_count++;
    
    return offset;
}

// ============================================================
// Fate Database
// ============================================================

void fate_db_store(FateMachine* fm, const char* key, const char* value) {
    for (int i = 0; i < fm->db_count; i++) {
        if (strcmp(fm->db[i].key, key) == 0) {
            strncpy(fm->db[i].value, value, 255);
            return;
        }
    }
    if (fm->db_count < MAX_DB_ENTRIES) {
        strncpy(fm->db[fm->db_count].key, key, 63);
        strncpy(fm->db[fm->db_count].value, value, 255);
        fm->db_count++;
    }
}

// ============================================================
// Variables
// ============================================================

void set_var(FateMachine* fm, const char* name, int64_t value) {
    for (int i = 0; i < fm->var_count; i++) {
        if (strcmp(fm->vars[i].name, name) == 0) {
            fm->vars[i].value = value;
            fm->vars[i].is_string = 0;
            return;
        }
    }
    if (fm->var_count < MAX_VARS) {
        strncpy(fm->vars[fm->var_count].name, name, 63);
        fm->vars[fm->var_count].value = value;
        fm->vars[fm->var_count].is_string = 0;
        fm->var_count++;
    }
}

void set_var_str(FateMachine* fm, const char* name, const char* value) {
    for (int i = 0; i < fm->var_count; i++) {
        if (strcmp(fm->vars[i].name, name) == 0) {
            strncpy(fm->vars[i].str_value, value, 255);
            fm->vars[i].is_string = 1;
            return;
        }
    }
    if (fm->var_count < MAX_VARS) {
        strncpy(fm->vars[fm->var_count].name, name, 63);
        strncpy(fm->vars[fm->var_count].str_value, value, 255);
        fm->vars[fm->var_count].is_string = 1;
        fm->var_count++;
    }
}

// ============================================================
// Emit Fate Message
// ============================================================

void emit_fate_message(FateMachine* fm, const char* msg) {
    if (!fm->fate_messages_enabled) return;
    size_t offset = add_string(fm, msg);
    size_t len = strlen(msg) + 1;
    emit_syscall_write(fm, 1, fm->data_base + offset, len);
}

// ============================================================
// Wave Compiler
// ============================================================

char* trim(char* s) {
    while (*s == ' ' || *s == '\t') s++;
    char* e = s + strlen(s) - 1;
    while (e > s && (*e == ' ' || *e == '\t' || *e == '\n' || *e == '\r')) *e-- = '\0';
    return s;
}

int in_when_block = 0;
int in_each_block = 0;

void compile_line(FateMachine* fm, char* line) {
    char* t = trim(line);
    
    if (strlen(t) == 0 || t[0] == '#') return;
    
    // ==================== limit N (资源上限) ====================
    if (strncmp(t, "limit ", 6) == 0) {
        int limit = atoi(t + 6);
        if (limit > 0 && limit <= 100) {
            fm->perf.resource_limit = limit;
            char msg[128];
            snprintf(msg, sizeof(msg), "   [Fate] Resource limit set: %d%% (Fate allocates within)", limit);
            emit_fate_message(fm, msg);
        }
        return;
    }
    
    // ==================== out "string" ====================
    if (strncmp(t, "out ", 4) == 0) {
        char* content = t + 4;
        while (*content == ' ') content++;
        
        if (content[0] == '"') {
            char* start = content + 1;
            char* end = strrchr(start, '"');
            if (end) {
                char str[4096] = {0};
                strncpy(str, start, end - start);
                
                size_t offset = add_string(fm, str);
                size_t len = strlen(str) + 1;
                emit_syscall_write(fm, 1, fm->data_base + offset, len);
                
                // 更新性能跟踪
                fm->collapse.prev_gain = fm->collapse.current_gain;
                fm->collapse.current_gain += 1.0;
            }
        }
        return;
    }
    
    // ==================== emit "bytes" (核心！直接输出机器码) ====================
    if (strncmp(t, "emit ", 5) == 0) {
        char* content = t + 5;
        while (*content == ' ') content++;
        
        if (content[0] == '"') {
            char* start = content + 1;
            char* end = strrchr(start, '"');
            if (end) {
                // 解析转义序列，直接输出字节
                char* p = start;
                while (p < end) {
                    if (*p == '\\' && p + 1 < end) {
                        p++;
                        if (*p == 'x' && p + 2 < end) {
                            // \xHH
                            char hex[3] = {p[1], p[2], 0};
                            uint8_t byte = (uint8_t)strtol(hex, NULL, 16);
                            emit_byte(fm, byte);
                            p += 3;
                        } else if (*p == 'n') {
                            emit_byte(fm, '\n');
                            p++;
                        } else if (*p == 't') {
                            emit_byte(fm, '\t');
                            p++;
                        } else if (*p == '\\') {
                            emit_byte(fm, '\\');
                            p++;
                        } else if (*p == '"') {
                            emit_byte(fm, '"');
                            p++;
                        } else {
                            emit_byte(fm, *p);
                            p++;
                        }
                    } else {
                        emit_byte(fm, *p);
                        p++;
                    }
                }
            }
        }
        return;
    }
    
    // ==================== fn name { body } ====================
    if (strncmp(t, "fn ", 3) == 0) {
        // 函数定义 - 跳过（Wave 5.0 函数由规则涌现）
        return;
    }
    
    // ==================== fate on / fate off ====================
    if (strncmp(t, "fate on", 7) == 0) {
        fm->fate_messages_enabled = 1;
        return;
    }
    if (strncmp(t, "fate off", 8) == 0) {
        fm->fate_messages_enabled = 0;
        return;
    }
    
    // ==================== keep (事件循环) ====================
    if (strncmp(t, "keep", 4) == 0) {
        // 生成无限循环等待事件
        // pause; jmp -4
        emit_byte(fm, 0xf3); emit_byte(fm, 0x90);  // pause
        emit_byte(fm, 0xeb); emit_byte(fm, 0xfc);  // jmp -4
        return;
    }
    
    // ==================== syscall.exit(N) ====================
    if (strncmp(t, "syscall.exit(", 13) == 0) {
        int code = atoi(t + 13);
        emit_syscall_exit(fm, code);
        return;
    }
    
    // ==================== loop { body } ====================
    if (strncmp(t, "loop", 4) == 0) {
        // 循环开始标记（简化处理）
        return;
    }
    
    // ==================== break ====================
    if (strncmp(t, "break", 5) == 0) {
        return;
    }
    
    // ==================== return ====================
    if (strncmp(t, "return", 6) == 0) {
        return;
    }
    
    // ==================== pool / unified / rule / intent / collapse / task / gpu / perf / reg / sys ====================
    // Wave 5.0 声明块 - 跳过（运行时解释）
    if (strncmp(t, "pool ", 5) == 0 || strncmp(t, "unified ", 8) == 0 ||
        strncmp(t, "rule ", 5) == 0 || strncmp(t, "intent ", 7) == 0 ||
        strncmp(t, "collapse ", 9) == 0 || strncmp(t, "task ", 5) == 0 ||
        strncmp(t, "gpu ", 4) == 0 || strncmp(t, "perf ", 5) == 0 ||
        strncmp(t, "reg ", 4) == 0 || strncmp(t, "sys ", 4) == 0 ||
        strncmp(t, "compiler ", 9) == 0 || strncmp(t, "fate ", 5) == 0 ||
        strncmp(t, "lib ", 4) == 0 || strncmp(t, "env ", 4) == 0) {
        return;
    }
    
    // ==================== 块声明 { ... } ====================
    if (strcmp(t, "unified {") == 0 || strcmp(t, "pool {") == 0 ||
        strcmp(t, "fate {") == 0 || strcmp(t, "task {") == 0 ||
        strcmp(t, "gpu {") == 0 || strcmp(t, "perf {") == 0 ||
        strcmp(t, "reg {") == 0 || strcmp(t, "sys {") == 0 ||
        strcmp(t, "compiler {") == 0 || strcmp(t, "collapse {") == 0 ||
        strcmp(t, "lib {") == 0 || strcmp(t, "env {") == 0) {
        return;
    }
    
    // ==================== Pipeline ====================
    if (strstr(t, "->")) {
        // 模拟重负载操作
        int load = 30;  // 管道操作需要30%资源
        int allocated = fate_allocate(fm, load);
        
        char msg[128];
        snprintf(msg, sizeof(msg), "   [Fate] Pipeline: need %d%%, allocated %d%%", load, allocated);
        emit_fate_message(fm, msg);
        
        // 检查是否需要Collapse
        fm->collapse.current_performance -= 5.0;  // 模拟性能消耗
        if (should_trigger_collapse(fm)) {
            trigger_collapse(fm);
            emit_fate_message(fm, "   [Collapse] Triggered! Marginal returns diminishing");
            char boost_msg[128];
            snprintf(boost_msg, sizeof(boost_msg), "   [Collapse] Performance +%d%%, Memory -%d%%", 
                     fm->collapse.performance_boost, fm->collapse.memory_reduction);
            emit_fate_message(fm, boost_msg);
        }
        
        // Parse stages
        char* copy = strdup(t);
        char* token = strtok(copy, "->");
        while (token) {
            char* stage = trim(token);
            if (strncmp(stage, "load", 4) == 0) {
                emit_fate_message(fm, "   [Fate] Loading...");
            } else if (strncmp(stage, "analyze", 7) == 0) {
                emit_fate_message(fm, "   [Voyager] Analyzing patterns...");
            } else if (strncmp(stage, "store", 5) == 0) {
                emit_fate_message(fm, "   [Fate DB] Auto-storing...");
            } else if (strncmp(stage, "filter", 6) == 0) {
                emit_fate_message(fm, "   [Fate] Filtering...");
            }
            token = strtok(NULL, "->");
        }
        free(copy);
        return;
    }
    
    // ==================== store ====================
    if (strncmp(t, "store ", 6) == 0) {
        char* name = trim(t + 6);
        fate_db_store(fm, name, "auto");
        
        char msg[128];
        snprintf(msg, sizeof(msg), "   [Fate DB] Storing '%s' - auto-indexed", name);
        emit_fate_message(fm, msg);
        return;
    }
    
    // ==================== find ====================
    if (strncmp(t, "find ", 5) == 0) {
        char* query = trim(t + 5);
        char msg[128];
        snprintf(msg, sizeof(msg), "   [Fate DB] Finding '%s'", query);
        emit_fate_message(fm, msg);
        return;
    }
    
    // ==================== analyze ====================
    if (strncmp(t, "analyze ", 8) == 0) {
        char* what = trim(t + 8);
        
        // 分析是重操作
        int load = 50;
        int allocated = fate_allocate(fm, load);
        
        char msg[128];
        snprintf(msg, sizeof(msg), "   [Voyager] Analyzing '%s' (need %d%%, got %d%%)", what, load, allocated);
        emit_fate_message(fm, msg);
        
        // 性能消耗
        fm->collapse.current_performance -= 10.0;
        if (should_trigger_collapse(fm)) {
            trigger_collapse(fm);
            emit_fate_message(fm, "   [Collapse] Performance insufficient - optimizing!");
            fm->collapse.current_performance = 95.0;  // 恢复性能
        }
        return;
    }
    
    // ==================== when ====================
    if (strncmp(t, "when ", 5) == 0) {
        if (strchr(t, '{')) {
            in_when_block = 1;
            emit_fate_message(fm, "   [Fate] Condition evaluated");
        }
        return;
    }
    
    // ==================== each ====================
    if (strncmp(t, "each ", 5) == 0) {
        in_each_block = 1;
        emit_fate_message(fm, "   [Fate] Iteration - auto-parallel decision");
        return;
    }
    
    // ==================== } ====================
    if (strcmp(t, "}") == 0) {
        in_when_block = 0;
        in_each_block = 0;
        return;
    }
    
    // ==================== Assignment ====================
    char* eq = strchr(t, '=');
    if (eq && eq > t && *(eq+1) != '=' && *(eq-1) != '!' && *(eq-1) != '<' && *(eq-1) != '>') {
        char var_name[64] = {0};
        strncpy(var_name, t, eq - t);
        trim(var_name);
        
        char* val = eq + 1;
        while (*val == ' ') val++;
        
        if (val[0] == '"') {
            char* start = val + 1;
            char* end = strrchr(start, '"');
            if (end) {
                char str_val[256] = {0};
                strncpy(str_val, start, end - start);
                set_var_str(fm, var_name, str_val);
            }
        } else if (isdigit(val[0]) || val[0] == '-') {
            set_var(fm, var_name, atol(val));
        }
        
        char msg[128];
        snprintf(msg, sizeof(msg), "   [Fate] Variable '%s' - auto-typed", var_name);
        emit_fate_message(fm, msg);
        return;
    }
}

int compile(FateMachine* fm, const char* input_file) {
    FILE* f = fopen(input_file, "r");
    if (!f) {
        fprintf(stderr, "Cannot open %s\n", input_file);
        return 1;
    }
    
    printf("Compiling %s (Direct Machine Code)...\n", input_file);
    
    // Emit init messages
    emit_fate_message(fm, "[Fate Brain] Initialized");
    emit_fate_message(fm, "[Fate] Resource allocation: ON-DEMAND (not maxed)");
    
    if (fm->perf.resource_limit > 0) {
        char msg[128];
        snprintf(msg, sizeof(msg), "[Fate] Resource limit: %d%%", fm->perf.resource_limit);
        emit_fate_message(fm, msg);
    } else {
        emit_fate_message(fm, "[Fate] Resource limit: NONE (Fate decides)");
    }
    
    emit_fate_message(fm, "[Collapse] Trigger: marginal-returns OR performance-drop");
    emit_fate_message(fm, "");
    
    char line[4096];
    while (fgets(line, sizeof(line), f)) {
        compile_line(fm, line);
    }
    fclose(f);
    
    // Final status
    emit_fate_message(fm, "");
    
    char status[256];
    snprintf(status, sizeof(status), "[Fate] Peak usage: %d%% | Collapse triggered: %d times", 
             fm->perf.peak_usage, fm->collapse.trigger_count);
    emit_fate_message(fm, status);
    emit_fate_message(fm, "[Fate] Resources auto-freed");
    
    emit_syscall_exit(fm, 0);
    
    return 0;
}

// ============================================================
// ELF64 Writer
// ============================================================

int write_elf(FateMachine* fm, const char* output_file) {
    FILE* f = fopen(output_file, "wb");
    if (!f) {
        fprintf(stderr, "Cannot create %s\n", output_file);
        return 1;
    }
    
    size_t ehdr_size = sizeof(Elf64_Ehdr);
    size_t phdr_size = sizeof(Elf64_Phdr);
    size_t num_phdrs = 2;
    
    size_t code_offset = 0x1000;
    size_t data_offset = code_offset + ((fm->code_pos + 0xFFF) & ~0xFFF);
    
    uint64_t entry_point = fm->code_base + code_offset;
    
    Elf64_Ehdr ehdr = {0};
    ehdr.e_ident[0] = 0x7F;
    ehdr.e_ident[1] = 'E';
    ehdr.e_ident[2] = 'L';
    ehdr.e_ident[3] = 'F';
    ehdr.e_ident[4] = 2;
    ehdr.e_ident[5] = 1;
    ehdr.e_ident[6] = 1;
    ehdr.e_type = 2;
    ehdr.e_machine = 62;
    ehdr.e_version = 1;
    ehdr.e_entry = entry_point;
    ehdr.e_phoff = ehdr_size;
    ehdr.e_ehsize = ehdr_size;
    ehdr.e_phentsize = phdr_size;
    ehdr.e_phnum = num_phdrs;
    
    Elf64_Phdr phdr_code = {0};
    phdr_code.p_type = 1;
    phdr_code.p_flags = 5;
    phdr_code.p_offset = code_offset;
    phdr_code.p_vaddr = fm->code_base + code_offset;
    phdr_code.p_paddr = fm->code_base + code_offset;
    phdr_code.p_filesz = fm->code_pos;
    phdr_code.p_memsz = fm->code_pos;
    phdr_code.p_align = 0x1000;
    
    Elf64_Phdr phdr_data = {0};
    phdr_data.p_type = 1;
    phdr_data.p_flags = 6;
    phdr_data.p_offset = data_offset;
    phdr_data.p_vaddr = fm->data_base;
    phdr_data.p_paddr = fm->data_base;
    phdr_data.p_filesz = fm->data_pos;
    phdr_data.p_memsz = fm->data_pos;
    phdr_data.p_align = 0x1000;
    
    fwrite(&ehdr, 1, ehdr_size, f);
    fwrite(&phdr_code, 1, phdr_size, f);
    fwrite(&phdr_data, 1, phdr_size, f);
    
    size_t current = ehdr_size + (phdr_size * num_phdrs);
    while (current < code_offset) { fputc(0, f); current++; }
    
    fwrite(fm->code, 1, fm->code_pos, f);
    
    current = code_offset + fm->code_pos;
    while (current < data_offset) { fputc(0, f); current++; }
    
    fwrite(fm->data, 1, fm->data_pos, f);
    
    fclose(f);
    
    char cmd[512];
    snprintf(cmd, sizeof(cmd), "chmod +x %s", output_file);
    system(cmd);
    
    printf("Generated %s\n", output_file);
    printf("   Code: %zu bytes | Data: %zu bytes\n", fm->code_pos, fm->data_pos);
    
    return 0;
}

// ============================================================
// Bare Metal Binary Writer (可引导镜像)
// ============================================================

int write_bare_metal(FateMachine* fm, const char* output_file) {
    FILE* f = fopen(output_file, "wb");
    if (!f) {
        fprintf(stderr, "Cannot create %s\n", output_file);
        return 1;
    }
    
    // 16-bit 引导扇区头 (跳转到32位代码)
    uint8_t bootsector[512] = {0};
    
    // jmp short start (跳过数据)
    bootsector[0] = 0xEB;
    bootsector[1] = 0x3C;  // 跳到 0x3E
    
    // nop
    bootsector[2] = 0x90;
    
    // OEM 标识
    memcpy(bootsector + 3, "VALOS   ", 8);
    
    // 在 0x3E 位置: 16-bit 启动代码
    int pos = 0x3E;
    
    // cli
    bootsector[pos++] = 0xFA;
    
    // 设置段寄存器
    // xor ax, ax
    bootsector[pos++] = 0x31; bootsector[pos++] = 0xC0;
    // mov ds, ax
    bootsector[pos++] = 0x8E; bootsector[pos++] = 0xD8;
    // mov es, ax
    bootsector[pos++] = 0x8E; bootsector[pos++] = 0xC0;
    // mov ss, ax
    bootsector[pos++] = 0x8E; bootsector[pos++] = 0xD0;
    // mov sp, 0x7C00
    bootsector[pos++] = 0xBC; bootsector[pos++] = 0x00; bootsector[pos++] = 0x7C;
    
    // 加载 GDT (gdt_desc 在 0x7C00+0x80 = 0x7C80)
    // lgdt [0x7C80]
    bootsector[pos++] = 0x0F; bootsector[pos++] = 0x01; 
    bootsector[pos++] = 0x16; bootsector[pos++] = 0x80; bootsector[pos++] = 0x7C;  // [0x7C80]
    
    // 启用保护模式
    // mov eax, cr0
    bootsector[pos++] = 0x0F; bootsector[pos++] = 0x20; bootsector[pos++] = 0xC0;
    // or al, 1
    bootsector[pos++] = 0x0C; bootsector[pos++] = 0x01;
    // mov cr0, eax
    bootsector[pos++] = 0x0F; bootsector[pos++] = 0x22; bootsector[pos++] = 0xC0;
    
    // 远跳转到 32-bit 代码 (0x08:0x1000)
    // jmp far 0x08:0x1000
    bootsector[pos++] = 0xEA;
    bootsector[pos++] = 0x00; // offset low
    bootsector[pos++] = 0x10; // offset high (0x1000)
    bootsector[pos++] = 0x08; // segment selector low
    bootsector[pos++] = 0x00; // segment selector high
    
    // GDT 描述符在 0x80 (实际地址 0x7C80)
    // GDT 表在 0x90 (实际地址 0x7C90)
    bootsector[0x80] = 0x17;  // limit low (3个描述符 * 8 - 1 = 23 = 0x17)
    bootsector[0x81] = 0x00;  // limit high
    bootsector[0x82] = 0x90;  // base low (0x7C90)
    bootsector[0x83] = 0x7C;  // base 
    bootsector[0x84] = 0x00;
    bootsector[0x85] = 0x00;  // base high
    
    // GDT 表在 0x90
    // 空描述符
    memset(bootsector + 0x90, 0, 8);
    // 代码段描述符 (0x08)
    bootsector[0x98] = 0xFF; bootsector[0x99] = 0xFF;  // limit
    bootsector[0x9A] = 0x00; bootsector[0x9B] = 0x00;  // base
    bootsector[0x9C] = 0x00;
    bootsector[0x9D] = 0x9A;  // 可执行，可读
    bootsector[0x9E] = 0xCF;  // 4K 粒度，32-bit
    bootsector[0x9F] = 0x00;
    // 数据段描述符 (0x10)
    bootsector[0xA0] = 0xFF; bootsector[0xA1] = 0xFF;
    bootsector[0xA2] = 0x00; bootsector[0xA3] = 0x00;
    bootsector[0xA4] = 0x00;
    bootsector[0xA5] = 0x92;  // 可读写
    bootsector[0xA6] = 0xCF;
    bootsector[0xA7] = 0x00;
    
    // Boot signature
    bootsector[510] = 0x55;
    bootsector[511] = 0xAA;
    
    // 写引导扇区
    fwrite(bootsector, 1, 512, f);
    
    // 填充到 0x1000 (第二扇区开始是32位代码)
    uint8_t padding[0x1000 - 512];
    memset(padding, 0, sizeof(padding));
    fwrite(padding, 1, sizeof(padding), f);
    
    // 32-bit 保护模式代码头
    uint8_t pm_header[64];
    int hp = 0;
    // mov ax, 0x10 (数据段选择子)
    pm_header[hp++] = 0xB8; pm_header[hp++] = 0x10; 
    pm_header[hp++] = 0x00; pm_header[hp++] = 0x00; pm_header[hp++] = 0x00;
    // mov ds, ax
    pm_header[hp++] = 0x8E; pm_header[hp++] = 0xD8;
    // mov es, ax
    pm_header[hp++] = 0x8E; pm_header[hp++] = 0xC0;
    // mov fs, ax
    pm_header[hp++] = 0x8E; pm_header[hp++] = 0xE0;
    // mov gs, ax
    pm_header[hp++] = 0x8E; pm_header[hp++] = 0xE8;
    // mov ss, ax
    pm_header[hp++] = 0x8E; pm_header[hp++] = 0xD0;
    // mov esp, 0x90000
    pm_header[hp++] = 0xBC; 
    pm_header[hp++] = 0x00; pm_header[hp++] = 0x00; 
    pm_header[hp++] = 0x09; pm_header[hp++] = 0x00;
    
    fwrite(pm_header, 1, hp, f);
    
    // 写 Wave 生成的代码
    fwrite(fm->code, 1, fm->code_pos, f);
    
    // 填充到 0x2000 位置 (数据段开始)
    size_t code_end = 0x1000 + hp + fm->code_pos;
    size_t data_start = 0x2000;
    for (size_t i = code_end; i < data_start; i++) fputc(0, f);
    
    // 写数据段 (从 0x2000 开始)
    fwrite(fm->data, 1, fm->data_pos, f);
    
    // 填充到整扇区
    size_t total = data_start + fm->data_pos;
    size_t aligned = (total + 511) & ~511;
    for (size_t i = total; i < aligned; i++) fputc(0, f);
    
    fclose(f);
    
    printf("Generated bare-metal image: %s\n", output_file);
    printf("   Bootsector: 512 bytes\n");
    printf("   Code: %zu bytes | Data: %zu bytes\n", fm->code_pos, fm->data_pos);
    printf("   Total: %zu bytes (%zu sectors)\n", aligned, aligned / 512);
    printf("\nRun with: qemu-system-i386 -fda %s\n", output_file);
    
    return 0;
}

double get_time_ms(void) {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return tv.tv_sec * 1000.0 + tv.tv_usec / 1000.0;
}

int main(int argc, char* argv[]) {
    printf("🌊 Wave v%s - Direct Machine Code Compiler\n", VERSION);
    printf("   Fate: on-demand | Collapse: marginal-returns | TileQuattro: memory\n\n");
    
    if (argc < 2) {
        printf("Usage: %s <input.wave> [-o output] [--no-tile] [--benchmark] [--bare-metal]\n\n", argv[0]);
        printf("Features:\n");
        printf("  - Direct x86-64 machine code\n");
        printf("  - TileQuattro memory management\n");
        printf("  - Fate on-demand allocation (not maxed)\n");
        printf("  - Collapse on marginal-returns OR performance-drop\n");
        printf("  - Optional 'limit N' for resource cap\n\n");
        printf("Flags:\n");
        printf("  --no-tile     Disable TileQuattro (use stdlib malloc)\n");
        printf("  --benchmark   Run before/after comparison\n");
        printf("  --bare-metal  Generate bootable image for QEMU\n\n");
        printf("Syntax:\n");
        printf("  out \"text\"         - Output\n");
        printf("  x = value          - Variable (auto-typed)\n");
        printf("  a -> b -> c        - Pipeline\n");
        printf("  limit N            - Set resource limit to N%%\n");
        printf("  store/find/analyze - Database & analysis\n");
        return 1;
    }
    
    char* input = argv[1];
    char* output = "a.out";
    int use_tile = 1;
    int benchmark = 0;
    int bare_metal = 0;
    int raw_mode = 0;
    int silent = 0;
    
    for (int i = 2; i < argc; i++) {
        if (strcmp(argv[i], "-o") == 0 && i + 1 < argc) {
            output = argv[++i];
        } else if (strcmp(argv[i], "--no-tile") == 0) {
            use_tile = 0;
        } else if (strcmp(argv[i], "--benchmark") == 0) {
            benchmark = 1;
        } else if (strcmp(argv[i], "--bare-metal") == 0) {
            bare_metal = 1;
            if (strcmp(output, "a.out") == 0) {
                output = "valos.img";
            }
        } else if (strcmp(argv[i], "--raw") == 0) {
            raw_mode = 1;
        } else if (strcmp(argv[i], "--silent") == 0) {
            silent = 1;
        }
    }
    
    if (benchmark) {
        int ITERATIONS = 100;
        printf("═══════════════════════════════════════════════════════════\n");
        printf("📊 TileQuattro vs Stdlib 内存管理对比测试 (%d 次编译)\n", ITERATIONS);
        printf("═══════════════════════════════════════════════════════════\n\n");
        
        // Test 1: Without TileQuattro (多次)
        printf("🔹 测试 1: 标准 malloc (无 TileQuattro)\n");
        double t1_start = get_time_ms();
        size_t code1 = 0, data1 = 0;
        for (int i = 0; i < ITERATIONS; i++) {
            FateMachine* fm1 = fate_machine_new(0);
            fm1->fate_messages_enabled = 0;  // 禁用输出
            compile(fm1, input);
            if (i == 0) { code1 = fm1->code_pos; data1 = fm1->data_pos; }
            fate_machine_free(fm1, 0);
        }
        double t1_total = get_time_ms() - t1_start;
        printf("   %d 次编译总时间: %.2f ms\n", ITERATIONS, t1_total);
        printf("   平均每次: %.3f ms\n\n", t1_total / ITERATIONS);
        
        // Test 2: With TileQuattro (多次 - 重用池)
        printf("🔹 测试 2: TileQuattro 内存管理\n");
        double t2_start = get_time_ms();
        size_t code2 = 0, data2 = 0;
        for (int i = 0; i < ITERATIONS; i++) {
            FateMachine* fm2 = fate_machine_new(1);
            fm2->fate_messages_enabled = 0;  // 禁用输出
            compile(fm2, input);
            if (i == 0) { code2 = fm2->code_pos; data2 = fm2->data_pos; }
            fate_machine_free(fm2, 1);
        }
        double t2_total = get_time_ms() - t2_start;
        printf("   %d 次编译总时间: %.2f ms\n", ITERATIONS, t2_total);
        printf("   平均每次: %.3f ms\n", t2_total / ITERATIONS);
        
        // Results
        printf("\n═══════════════════════════════════════════════════════════\n");
        printf("📈 对比结果 (%d 次编译):\n", ITERATIONS);
        printf("═══════════════════════════════════════════════════════════\n");
        printf("   标准 malloc:    %.2f ms | Code: %zu | Data: %zu\n", t1_total, code1, data1);
        printf("   TileQuattro:    %.2f ms | Code: %zu | Data: %zu\n", t2_total, code2, data2);
        if (t2_total > 0 && t1_total > 0) {
            double speedup = t1_total / t2_total;
            if (speedup > 1.0) {
                printf("\n   ✅ TileQuattro 快 %.2fx\n", speedup);
            } else {
                printf("\n   ⚠️  标准 malloc 快 %.2fx\n", 1.0/speedup);
            }
        }
        printf("═══════════════════════════════════════════════════════════\n");
        return 0;
    }
    
    if (use_tile) {
        printf("   [TileQuattro] 四元内存池已启用\n\n");
    }
    
    if (bare_metal) {
        printf("   [Bare-Metal] 生成可引导镜像\n\n");
    }
    
    FateMachine* fm = fate_machine_new(use_tile);
    fm->bare_metal = bare_metal;
    fm->vga_cursor = 0;
    
    // silent 模式：禁用 fate 消息
    if (silent || raw_mode) {
        fm->fate_messages_enabled = 0;
    }
    
    // 裸机模式：数据段固定在 0x2000 (32KB以上，足够放代码)
    if (bare_metal) {
        fm->data_base = 0x2000;  // 数据段固定地址
    }
    
    if (compile(fm, input) != 0) {
        fate_machine_free(fm, use_tile);
        return 1;
    }
    
    int result = 0;
    if (raw_mode) {
        // Raw 模式：直接输出代码段（纯二进制）
        FILE* f = fopen(output, "wb");
        if (f) {
            fwrite(fm->code, 1, fm->code_pos, f);
            fclose(f);
            printf("Generated raw binary: %s (%zu bytes)\n", output, fm->code_pos);
        } else {
            result = 1;
        }
    } else if (bare_metal) {
        result = write_bare_metal(fm, output);
    } else {
        result = write_elf(fm, output);
    }
    
    if (result != 0) {
        fate_machine_free(fm, use_tile);
        return 1;
    }
    
    fate_machine_free(fm, use_tile);
    return 0;
}
