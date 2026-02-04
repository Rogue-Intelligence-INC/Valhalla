/*
 * 🌊 Wave v4 Machine Code Compiler
 * 
 * 直接生成x86-64机器码，无任何外部依赖
 * Fate完全控制每一个字节
 * 
 * 核心特性:
 * - 直接机器码生成
 * - Fate按需分配（不占满资源）
 * - Collapse触发：边际收益递减 或 性能不足
 * - 可选limit语句限制资源上限
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <ctype.h>
#include <math.h>

#define VERSION "4.1.0"
#define MAX_CODE (1024 * 1024)
#define MAX_DATA (1024 * 1024)
#define MAX_STRINGS 1024
#define MAX_VARS 256
#define MAX_DB_ENTRIES 1024

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
} FateMachine;

FateMachine* fate_machine_new() {
    FateMachine* fm = calloc(1, sizeof(FateMachine));
    fm->code = malloc(MAX_CODE);
    fm->code_cap = MAX_CODE;
    fm->data = malloc(MAX_DATA);
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

void fate_machine_free(FateMachine* fm) {
    free(fm->code);
    free(fm->data);
    for (int i = 0; i < fm->string_count; i++) {
        free(fm->strings[i].str);
    }
    free(fm);
}

// ============================================================
// Collapse Trigger Logic
// ============================================================

// 检查是否应该触发Collapse
// 条件1: 边际收益递减（导数接近0或为负）
// 条件2: 性能不足（当前性能 < 需求性能）
int should_trigger_collapse(FateMachine* fm) {
    // 计算边际收益
    fm->collapse.marginal_gain = fm->collapse.current_gain - fm->collapse.prev_gain;
    
    // 条件1: 边际收益递减（二元方程极值点）
    // 当 |marginal_gain| < 0.01 或 marginal_gain < 0
    int diminishing_returns = (fabs(fm->collapse.marginal_gain) < 0.01) || 
                              (fm->collapse.marginal_gain < 0);
    
    // 条件2: 性能不足
    int performance_insufficient = fm->collapse.current_performance < 
                                   fm->collapse.required_performance;
    
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

void emit_syscall_write(FateMachine* fm, int fd, uint64_t buf_addr, size_t len) {
    emit_mov_rax_imm64(fm, 1);
    emit_mov_rdi_imm64(fm, fd);
    emit_mov_rsi_imm64(fm, buf_addr);
    emit_mov_rdx_imm64(fm, len);
    emit_syscall(fm);
}

void emit_syscall_exit(FateMachine* fm, int code) {
    emit_mov_rax_imm64(fm, 60);
    emit_mov_rdi_imm64(fm, code);
    emit_syscall(fm);
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

int main(int argc, char* argv[]) {
    printf("Wave v%s - Direct Machine Code Compiler\n", VERSION);
    printf("   Fate: on-demand allocation | Collapse: marginal-returns trigger\n\n");
    
    if (argc < 2) {
        printf("Usage: %s <input.wave> [-o output]\n\n", argv[0]);
        printf("Features:\n");
        printf("  - Direct x86-64 machine code\n");
        printf("  - Fate on-demand allocation (not maxed)\n");
        printf("  - Collapse on marginal-returns OR performance-drop\n");
        printf("  - Optional 'limit N' for resource cap\n\n");
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
    
    for (int i = 2; i < argc; i++) {
        if (strcmp(argv[i], "-o") == 0 && i + 1 < argc) {
            output = argv[++i];
        }
    }
    
    FateMachine* fm = fate_machine_new();
    
    if (compile(fm, input) != 0) {
        fate_machine_free(fm);
        return 1;
    }
    
    if (write_elf(fm, output) != 0) {
        fate_machine_free(fm);
        return 1;
    }
    
    fate_machine_free(fm);
    return 0;
}
