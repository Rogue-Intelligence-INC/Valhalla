#!/bin/bash
#
# Valhalla f64 持久化多模态训练
# 支持断点续训 | 70% 资源 | 全量数据
#

cd /home/jouly/AI/Valhalla

LOG_FILE="persistent_train.log"
CHECKPOINT="/run/media/jouly/My Book/valhalla_training.valhalla"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     🚀 Valhalla f64 持久化多模态训练                          ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# 检查checkpoint
if [ -f "$CHECKPOINT" ]; then
    echo "📂 发现checkpoint: $CHECKPOINT"
    echo "   将从上次状态继续训练"
else
    echo "📝 无checkpoint，从头开始训练"
fi

echo ""
echo "📊 数据源:"
echo "   旧语料库: /run/media/jouly/My Book/valhalla_corpus/"
echo "   新数据:   /run/media/jouly/My Book/valhalla_training_data/"
echo ""
echo "📝 日志: $LOG_FILE"
echo "💾 Checkpoint: $CHECKPOINT"
echo "⚡ 资源: 70% CPU"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  按 Ctrl+C 可安全停止（会自动保存状态）"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# 运行训练
./target/release/valhalla_persistent_train 2>&1 | tee -a "$LOG_FILE"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "训练结束! Checkpoint已保存到: $CHECKPOINT"
