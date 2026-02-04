#!/bin/bash
#
# Valhalla 下载监控界面
# 使用: ./download_monitor.sh
#

clear

while true; do
    tput cup 0 0  # 移动光标到顶部
    
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║            🚀 Valhalla 训练数据下载监控                              ║"
    echo "║            $(date '+%Y-%m-%d %H:%M:%S')                                        ║"
    echo "╠══════════════════════════════════════════════════════════════════════╣"
    
    # 进程状态
    echo "║ 📡 进程状态:                                                         ║"
    if pgrep -f "download_auto_retry" > /dev/null; then
        echo "║   ✅ 自动重试器: 运行中                                              ║"
    else
        echo "║   ❌ 自动重试器: 未运行                                              ║"
    fi
    
    if pgrep -f "download_training_data" > /dev/null; then
        echo "║   ✅ 下载脚本: 运行中                                                ║"
    else
        echo "║   ❌ 下载脚本: 未运行                                                ║"
    fi
    
    if pgrep -f "aria2c" > /dev/null; then
        echo "║   ✅ aria2c: 下载中                                                  ║"
    else
        echo "║   ⏸️  aria2c: 空闲                                                    ║"
    fi
    
    echo "╠══════════════════════════════════════════════════════════════════════╣"
    
    # 磁盘使用
    echo "║ 💾 磁盘使用:                                                         ║"
    
    TOTAL=$(du -sh "/run/media/jouly/My Book/valhalla_training_data/" 2>/dev/null | cut -f1)
    printf "║   总计: %-60s ║\n" "$TOTAL"
    
    for dir in images audio video text dialogue code; do
        SIZE=$(du -sh "/run/media/jouly/My Book/valhalla_training_data/$dir/" 2>/dev/null | cut -f1)
        COUNT=$(find "/run/media/jouly/My Book/valhalla_training_data/$dir" -type f 2>/dev/null | wc -l)
        printf "║   %-10s %8s  (%6d 文件)                               ║\n" "$dir:" "$SIZE" "$COUNT"
    done
    
    echo "╠══════════════════════════════════════════════════════════════════════╣"
    
    # 最新日志
    echo "║ 📜 最新活动:                                                         ║"
    tail -5 /home/jouly/AI/Valhalla/download.log 2>/dev/null | while read line; do
        # 截断长行
        line=$(echo "$line" | sed 's/\x1b\[[0-9;]*m//g' | cut -c1-66)
        printf "║   %-66s ║\n" "$line"
    done
    
    echo "╠══════════════════════════════════════════════════════════════════════╣"
    
    # 当前下载
    CURRENT=$(ps aux | grep aria2c | grep -v grep | sed 's/.*-o //' | cut -d' ' -f1 | head -1)
    if [[ -n "$CURRENT" ]]; then
        printf "║ 📥 当前下载: %-55s ║\n" "$CURRENT"
    else
        echo "║ 📥 当前下载: (无)                                                    ║"
    fi
    
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "按 Ctrl+C 退出监控"
    
    sleep 5
done
