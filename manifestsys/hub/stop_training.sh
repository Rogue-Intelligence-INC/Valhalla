#!/bin/bash
# 停止 Valhalla 多模态训练

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$SCRIPT_DIR/training.pid"

if [ ! -f "$PID_FILE" ]; then
    echo "⚠️  没有找到运行中的训练进程"
    exit 0
fi

PID=$(cat "$PID_FILE")

if ps -p $PID > /dev/null 2>&1; then
    echo "🛑 停止训练进程 (PID: $PID)..."
    kill -SIGINT $PID
    
    # 等待优雅关闭
    for i in {1..10}; do
        if ! ps -p $PID > /dev/null 2>&1; then
            break
        fi
        echo "   等待关闭... ($i/10)"
        sleep 1
    done
    
    # 如果还在运行，强制终止
    if ps -p $PID > /dev/null 2>&1; then
        echo "   强制终止..."
        kill -9 $PID
    fi
    
    rm -f "$PID_FILE"
    echo "✅ 训练已停止"
else
    echo "⚠️  进程 $PID 已不存在"
    rm -f "$PID_FILE"
fi
