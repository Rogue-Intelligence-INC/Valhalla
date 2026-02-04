#!/bin/bash
#
# Valhalla 70% 数据训练脚本
# 持续运行，自动保存 checkpoint
#

cd /home/jouly/AI/Valhalla

LOG_FILE="train_70percent.log"
CYCLES=100  # 运行100个周期

echo "=========================================="
echo "  Valhalla 70% 数据训练 - 开始"
echo "  周期数: $CYCLES"
echo "  日志: $LOG_FILE"
echo "=========================================="

# 设置环境变量控制数据量
export VALHALLA_DATA_RATIO=0.7

# 运行训练
./target/release/valhalla_cyclic_train --cycles $CYCLES 2>&1 | tee -a "$LOG_FILE"

echo ""
echo "训练完成!"
