#!/bin/bash
#
# Valhalla f64 多模态收敛训练启动脚本
# 70% 系统资源 | 自动收敛停机
#

cd /home/jouly/AI/Valhalla

LOG_FILE="multimodal_convergence_train.log"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     🚀 启动 Valhalla f64 多模态收敛训练                        ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 训练数据:"
echo "   - COCO 2017 图片: $(ls /run/media/jouly/My\ Book/valhalla_training_data/images/train2017/ 2>/dev/null | wc -l) 张"
echo "   - LibriSpeech 音频: $(find /run/media/jouly/My\ Book/valhalla_training_data/audio/LibriSpeech -name "*.flac" 2>/dev/null | wc -l) 个"
echo "   - Wikipedia 中文: $(du -sh /run/media/jouly/My\ Book/valhalla_training_data/text/zhwiki-latest.xml.bz2 2>/dev/null | cut -f1)"
echo ""
echo "📝 日志: $LOG_FILE"
echo "⚡ 资源: 70% CPU"
echo "🎯 模式: 自动收敛停机"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""

# 运行训练
./target/release/valhalla_multimodal_convergence 2>&1 | tee -a "$LOG_FILE"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "训练结束!"
