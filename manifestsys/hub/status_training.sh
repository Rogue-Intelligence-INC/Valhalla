#!/bin/bash
# 查看 Valhalla 多模态训练状态

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$SCRIPT_DIR/training.pid"
CHECKPOINT_DIR="$SCRIPT_DIR/data/checkpoints"
LOG_DIR="$SCRIPT_DIR/logs"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     🌌 Valhalla 多模态训练状态                             ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# 检查进程状态
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p $PID > /dev/null 2>&1; then
        echo "📊 训练状态: ✅ 运行中"
        echo "   PID: $PID"
        
        # CPU 和内存使用
        CPU_MEM=$(ps -p $PID -o %cpu,%mem --no-headers 2>/dev/null || echo "N/A N/A")
        CPU=$(echo $CPU_MEM | awk '{print $1}')
        MEM=$(echo $CPU_MEM | awk '{print $2}')
        echo "   CPU: ${CPU}%"
        echo "   内存: ${MEM}%"
        
        # 运行时间
        ELAPSED=$(ps -p $PID -o etime --no-headers 2>/dev/null | tr -d ' ')
        echo "   运行时间: $ELAPSED"
    else
        echo "📊 训练状态: ❌ 已停止 (PID 文件存在但进程不存在)"
        rm -f "$PID_FILE"
    fi
else
    echo "📊 训练状态: ⏹️  未运行"
fi

# 检查最新检查点
echo ""
if [ -f "$CHECKPOINT_DIR/latest.json" ]; then
    echo "💾 最新检查点:"
    cat "$CHECKPOINT_DIR/latest.json" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    stats = data.get('stats', {})
    print(f\"   迭代: {data.get('iteration', 'N/A')}\")
    print(f\"   时间: {data.get('timestamp', 'N/A')}\")
    print(f\"   文本: {stats.get('text_processed', 0):,}\")
    print(f\"   代码: {stats.get('code_processed', 0):,}\")
    print(f\"   数学: {stats.get('math_processed', 0):,}\")
    print(f\"   音频: {stats.get('audio_processed', 0):,}\")
    print(f\"   图形: {stats.get('visual_processed', 0):,}\")
    print(f\"   总信号: {stats.get('total_signals', 0):,}\")
    print(f\"   Fate发现: {stats.get('fate_discoveries', 0):,}\")
except:
    print('   (无法解析)')
" 2>/dev/null || echo "   (无法读取)"
else
    echo "💾 检查点: 无"
fi

# 最新日志
echo ""
LATEST_LOG=$(ls -t "$LOG_DIR"/training_*.log 2>/dev/null | head -1)
if [ -n "$LATEST_LOG" ]; then
    echo "📝 最新日志: $LATEST_LOG"
    echo "   最后 5 行:"
    tail -5 "$LATEST_LOG" | sed 's/^/   /'
else
    echo "📝 日志: 无"
fi
