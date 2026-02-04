#!/bin/bash
# 🌌 Valhalla 全系统语言核循环测试运行脚本
# 运行 10 分钟全面测试

set -e

echo "============================================================"
echo "🌌 Valhalla 全系统语言核循环测试"
echo "============================================================"
echo ""
echo "⏰ 测试时长: 10 分钟"
echo "🎯 测试内容:"
echo "  - 启动 Valhalla 全部功能系统"
echo "  - 循环处理持续演化"
echo "  - 复杂多语言逻辑表达测试"
echo ""

# 编译测试程序
echo "🔨 编译测试程序..."
cargo build --release --bin test_full_valhalla_language_core -p stem-cell

if [ $? -ne 0 ]; then
    echo "❌ 编译失败！"
    exit 1
fi

echo "✅ 编译成功！"
echo ""

# 创建输出目录
OUTPUT_DIR="ai-incubator/experiments/results"
mkdir -p "$OUTPUT_DIR"

# 生成时间戳
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_FILE="${OUTPUT_DIR}/FULL_VALHALLA_TEST_${TIMESTAMP}.log"

echo "🚀 开始测试..."
echo "📝 输出文件: $OUTPUT_FILE"
echo ""

# 运行测试，同时输出到文件和控制台
cargo run --release --bin test_full_valhalla_language_core -p stem-cell 2>&1 | tee "$OUTPUT_FILE"

echo ""
echo "============================================================"
echo "✅ 测试完成！"
echo "📊 结果已保存到: $OUTPUT_FILE"
echo "============================================================"
