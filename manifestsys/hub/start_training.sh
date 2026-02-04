#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# 🌌 Valhalla 多模态训练启动脚本
#
# 功能:
# 1. 下载/更新训练语料
# 2. 编译训练程序
# 3. 后台启动持续训练 (50% CPU)
# ═══════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="$SCRIPT_DIR/data"
LOG_DIR="$SCRIPT_DIR/logs"
PID_FILE="$SCRIPT_DIR/training.pid"

mkdir -p "$LOG_DIR"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     🌌 Valhalla 多模态训练启动器                           ║"
echo "║     时间: $(date '+%Y-%m-%d %H:%M:%S')                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# 检查是否已有训练在运行
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p $OLD_PID > /dev/null 2>&1; then
        echo "⚠️  训练已在运行 (PID: $OLD_PID)"
        echo "   使用 './stop_training.sh' 停止当前训练"
        echo "   或使用 './status_training.sh' 查看状态"
        exit 1
    else
        rm "$PID_FILE"
    fi
fi

# 步骤 1: 下载/更新语料
echo "📦 [1/3] 检查训练语料..."
if [ ! -d "$DATA_DIR/corpus" ] || [ "$1" == "--update-corpus" ]; then
    echo "   → 下载训练语料..."
    chmod +x "$DATA_DIR/download_all_corpora.sh"
    bash "$DATA_DIR/download_all_corpora.sh"
else
    echo "   ✅ 语料已存在，跳过下载"
    echo "      (使用 --update-corpus 强制更新)"
fi

# 步骤 2: 编译
echo ""
echo "🔨 [2/3] 编译训练程序..."
cd "$SCRIPT_DIR/../.."  # 回到 workspace root
cargo build --release -p valhalla-hub --bin continuous_multimodal_training 2>&1 | tail -5

# 步骤 3: 启动训练
echo ""
echo "🚀 [3/3] 启动后台训练..."

LOG_FILE="$LOG_DIR/training_$(date +%Y%m%d_%H%M%S).log"

# 设置 CPU 亲和性 (如果 taskset 可用)
if command -v taskset &> /dev/null; then
    # 获取 CPU 核心数的一半
    TOTAL_CORES=$(nproc)
    HALF_CORES=$((TOTAL_CORES / 2))
    if [ $HALF_CORES -lt 1 ]; then
        HALF_CORES=1
    fi
    
    # 创建 CPU 掩码 (使用前一半核心)
    CPU_MASK=$(printf "0x%x" $((2**HALF_CORES - 1)))
    
    echo "   CPU 限制: 使用 $HALF_CORES / $TOTAL_CORES 核心"
    
    nohup taskset $CPU_MASK \
        "$SCRIPT_DIR/../../target/release/continuous_multimodal_training" \
        > "$LOG_FILE" 2>&1 &
else
    # 没有 taskset，使用 nice 降低优先级
    echo "   使用 nice 限制优先级"
    nohup nice -n 10 \
        "$SCRIPT_DIR/../../target/release/continuous_multimodal_training" \
        > "$LOG_FILE" 2>&1 &
fi

TRAINING_PID=$!
echo $TRAINING_PID > "$PID_FILE"

sleep 2

# 检查是否成功启动
if ps -p $TRAINING_PID > /dev/null 2>&1; then
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║     ✅ 训练已在后台启动                                    ║"
    echo "╠════════════════════════════════════════════════════════════╣"
    echo "║ PID:  $TRAINING_PID                                              ║"
    echo "║ 日志: $LOG_FILE"
    echo "╠════════════════════════════════════════════════════════════╣"
    echo "║ 常用命令:                                                  ║"
    echo "║   查看日志: tail -f $LOG_FILE"
    echo "║   查看状态: ./status_training.sh                           ║"
    echo "║   停止训练: ./stop_training.sh                             ║"
    echo "╚════════════════════════════════════════════════════════════╝"
else
    echo "❌ 启动失败，查看日志: $LOG_FILE"
    rm -f "$PID_FILE"
    exit 1
fi
