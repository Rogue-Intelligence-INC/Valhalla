# 🌊 Wave Compiler Makefile
# Valhalla Fate Brain Architecture

# 编译器配置
CC = gcc
CFLAGS = -Wall -Wextra -std=c99 -O2
DEBUG_FLAGS = -g -DVALHALLA_DEBUG
INCLUDES = -Iinclude
SRCDIR = src
BINDIR = bin
EXAMPLEDIR = examples
TESTDIR = tests

# 目标文件
WAVE_COMPILER = $(BINDIR)/wave
WAVE_NATIVE = $(BINDIR)/wave-native
WAVE_SRC = $(SRCDIR)/wave_compiler.c
WAVE_NATIVE_SRC = $(SRCDIR)/wave_native_compiler.c

# 颜色定义
GREEN = \033[0;32m
BLUE = \033[0;34m
YELLOW = \033[1;33m
RED = \033[0;31m
NC = \033[0m # No Color

.PHONY: all clean install test examples help banner native

# 默认目标 - 构建两个编译器
all: banner $(WAVE_COMPILER) $(WAVE_NATIVE)

# 显示Banner
banner:
	@echo "$(BLUE)🌊 Wave Compiler Build System$(NC)"
	@echo "$(GREEN)   Powered by Valhalla Fate Brain$(NC)"
	@echo "$(GREEN)   Architecture: Fate-Brain-2024$(NC)"
	@echo ""

# 编译Wave编译器 (转译器版本)
$(WAVE_COMPILER): $(WAVE_SRC) | $(BINDIR)
	@echo "$(YELLOW)🔨 Building Wave Transpiler...$(NC)"
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ $<
	@echo "$(GREEN)✅ Wave Transpiler built successfully$(NC)"
	@echo ""

# 编译Wave Native编译器 (真正独立版本)
$(WAVE_NATIVE): $(WAVE_NATIVE_SRC) | $(BINDIR)
	@echo "$(YELLOW)🚀 Building Wave Native Compiler...$(NC)"
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ $< -lm
	@echo "$(GREEN)✅ Wave Native Compiler built successfully$(NC)"
	@echo "$(GREEN)   📦 This is our REAL independent compiler!$(NC)"
	@echo ""

# 只构建独立编译器
native: banner $(WAVE_NATIVE)
	@echo "$(GREEN)🎉 Independent Wave Native Compiler ready!$(NC)"

# 创建目录
$(BINDIR):
	@mkdir -p $(BINDIR)

# 调试版本
debug: CFLAGS += $(DEBUG_FLAGS)
debug: clean $(WAVE_COMPILER)
	@echo "$(YELLOW)🐛 Debug version built$(NC)"

# 测试示例程序
test: $(WAVE_COMPILER) $(WAVE_NATIVE)
	@echo "$(BLUE)🧪 Testing Wave Compilers...$(NC)"
	@echo ""
	@echo "$(YELLOW)📝 Testing with Wave Transpiler:$(NC)"
	./$(WAVE_COMPILER) -v $(EXAMPLEDIR)/hello.wave
	@echo ""
	@echo "$(YELLOW)🚀 Running hello program:$(NC)"
	./$(WAVE_COMPILER) -r $(EXAMPLEDIR)/hello.wave
	@echo ""
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(YELLOW)📝 Testing with Wave Native Compiler:$(NC)"
	./$(WAVE_NATIVE) -o hello_native $(EXAMPLEDIR)/hello.wave
	@echo ""
	@echo "$(YELLOW)🚀 Running native hello program:$(NC)"
	./hello_native
	@rm -f hello_native
	@echo ""
	@echo "$(GREEN)🎉 All tests completed!$(NC)"

# 分析示例代码
analyze: $(WAVE_COMPILER)
	@echo "$(BLUE)🧠 Fate Brain Code Analysis...$(NC)"
	@echo ""
	@echo "$(YELLOW)🔍 Analyzing hello.wave:$(NC)"
	./$(WAVE_COMPILER) -a $(EXAMPLEDIR)/hello.wave
	@echo ""
	@echo "$(YELLOW)🔍 Analyzing data_analysis.wave:$(NC)"
	./$(WAVE_COMPILER) -a $(EXAMPLEDIR)/data_analysis.wave
	@echo ""

# 基准测试 - 比较两种编译器
benchmark: $(WAVE_COMPILER) $(WAVE_NATIVE)
	@echo "$(BLUE)⚡ Wave Compiler Benchmark...$(NC)"
	@echo ""
	@echo "$(YELLOW)Creating benchmark program...$(NC)"
	@echo 'main() { print("Wave Benchmark Test"); }' > $(EXAMPLEDIR)/benchmark.wave
	@echo ""
	@echo "$(YELLOW)🔄 Wave Transpiler (using gcc backend):$(NC)"
	@time -f "  Time: %es" ./$(WAVE_COMPILER) -r $(EXAMPLEDIR)/benchmark.wave 2>&1
	@echo ""
	@echo "$(YELLOW)🚀 Wave Native (pure machine code):$(NC)"
	@time -f "  Compile: %es" ./$(WAVE_NATIVE) -o benchmark_native $(EXAMPLEDIR)/benchmark.wave 2>&1
	@echo -n "  Run: "
	@time -f "%es" ./benchmark_native 2>&1
	@rm -f $(EXAMPLEDIR)/benchmark.wave benchmark_native
	@echo ""
	@echo "$(GREEN)📊 Benchmark complete$(NC)"

# 显示独立编译器演示
demo: $(WAVE_NATIVE)
	@echo "$(BLUE)🎭 Wave Native Compiler Demo$(NC)"
	@echo "$(GREEN)=============================$(NC)"
	@echo ""
	@echo "$(YELLOW)Creating demo program...$(NC)"
	@echo 'main() { print("Hello from Wave Native!"); print("No GCC needed!"); }' > demo.wave
	@echo ""
	@echo "$(YELLOW)📝 Source code:$(NC)"
	@cat demo.wave
	@echo ""
	@echo "$(YELLOW)🚀 Compiling with our independent compiler:$(NC)"
	./$(WAVE_NATIVE) -o demo_exe demo.wave
	@echo ""
	@echo "$(YELLOW)🔍 Checking the result:$(NC)"
	@file demo_exe
	@ls -la demo_exe
	@echo ""
	@echo "$(YELLOW)🚀 Running the executable:$(NC)"
	./demo_exe
	@echo ""
	@echo "$(GREEN)🎉 Success! Pure Wave Native compilation!$(NC)"
	@rm -f demo.wave demo_exe

# 编译所有示例
examples: $(WAVE_COMPILER)
	@echo "$(BLUE)📚 Compiling all examples...$(NC)"
	@for wave_file in $(EXAMPLEDIR)/*.wave; do \
		if [ -f "$$wave_file" ]; then \
			echo "$(YELLOW)🔨 Compiling $$wave_file$(NC)"; \
			./$(WAVE_COMPILER) -v "$$wave_file"; \
		fi; \
	done
	@echo "$(GREEN)✅ All examples compiled$(NC)"

# 显示Wave编译器信息
info: $(WAVE_COMPILER)
	@echo "$(BLUE)ℹ️  Wave Compiler Information$(NC)"
	@echo ""
	./$(WAVE_COMPILER) --version
	@echo ""
	./$(WAVE_COMPILER) --help

# 安装到系统
install: $(WAVE_COMPILER)
	@echo "$(YELLOW)📦 Installing Wave Compiler...$(NC)"
	@if [ -w "/usr/local/bin" ]; then \
		cp $(WAVE_COMPILER) /usr/local/bin/wave; \
		echo "$(GREEN)✅ Wave installed to /usr/local/bin/wave$(NC)"; \
	else \
		echo "$(RED)❌ Permission denied. Try: sudo make install$(NC)"; \
		exit 1; \
	fi

# 清理构建文件
clean:
	@echo "$(YELLOW)🧹 Cleaning build files...$(NC)"
	@rm -rf $(BINDIR)
	@rm -f $(EXAMPLEDIR)/hello $(EXAMPLEDIR)/data_analysis
	@rm -f $(EXAMPLEDIR)/*.temp.c
	@rm -f $(EXAMPLEDIR)/benchmark.wave $(EXAMPLEDIR)/benchmark
	@echo "$(GREEN)✅ Clean complete$(NC)"

# 深度清理（包括缓存文件）
distclean: clean
	@echo "$(YELLOW)🗑️  Deep cleaning...$(NC)"
	@find . -name "*.o" -delete
	@find . -name "*.temp.*" -delete
	@find . -name "core" -delete
	@echo "$(GREEN)✅ Deep clean complete$(NC)"

# 显示帮助
help:
	@echo "$(BLUE)🌊 Wave Compiler Build System$(NC)"
	@echo "$(GREEN)   Powered by Valhalla Fate Brain$(NC)"
	@echo ""
	@echo "$(YELLOW)Available targets:$(NC)"
	@echo "  $(GREEN)all$(NC)        - Build both Wave compilers (default)"
	@echo "  $(GREEN)native$(NC)     - Build only Wave Native (independent) compiler"
	@echo "  $(GREEN)debug$(NC)      - Build debug versions"
	@echo "  $(GREEN)test$(NC)       - Run tests with both compilers"
	@echo "  $(GREEN)benchmark$(NC)  - Compare performance of both compilers"
	@echo "  $(GREEN)demo$(NC)       - Demo the independent native compiler"
	@echo "  $(GREEN)analyze$(NC)    - Run Fate Brain code analysis"
	@echo "  $(GREEN)examples$(NC)   - Compile all example programs"
	@echo "  $(GREEN)info$(NC)       - Show Wave compiler information"
	@echo "  $(GREEN)install$(NC)    - Install Wave to system (requires sudo)"
	@echo "  $(GREEN)clean$(NC)      - Remove build files"
	@echo "  $(GREEN)distclean$(NC)  - Deep clean including caches"
	@echo "  $(GREEN)help$(NC)       - Show this help message"
	@echo ""
	@echo "$(YELLOW)Compiler Types:$(NC)"
	@echo "  $(GREEN)wave$(NC)       - Transpiler (Wave -> C -> gcc -> executable)"
	@echo "  $(GREEN)wave-native$(NC) - Native compiler (Wave -> machine code directly)"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make native       # Build independent compiler only"
	@echo "  make demo         # See native compilation in action"
	@echo "  make benchmark    # Compare both approaches"
	@echo "  sudo make install # Install both compilers"
	@echo ""

# 文件依赖
$(WAVE_SRC): include/valhalla_core.h 