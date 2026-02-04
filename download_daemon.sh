#!/bin/bash
#
# Valhalla 下载守护进程
# 无限重试，确保下载不中断
#
# 使用: nohup ./download_daemon.sh > download_daemon.log 2>&1 &
#

SCRIPT_DIR="/home/jouly/AI/Valhalla"
DOWNLOAD_SCRIPT="$SCRIPT_DIR/download_training_data.sh"
LOG_FILE="$SCRIPT_DIR/download.log"
DAEMON_LOG="$SCRIPT_DIR/download_daemon.log"
RETRY_DELAY=60  # 秒

log_daemon() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$DAEMON_LOG"
}

check_disk_mounted() {
    if [[ -d "/run/media/jouly/My Book" ]]; then
        return 0
    else
        return 1
    fi
}

run_download() {
    log_daemon "🚀 启动下载..."
    
    # 检查硬盘是否挂载
    while ! check_disk_mounted; do
        log_daemon "⚠️ 外置硬盘未挂载，等待 60 秒..."
        sleep 60
    done
    
    cd "$SCRIPT_DIR"
    
    # 运行下载脚本
    bash "$DOWNLOAD_SCRIPT" >> "$LOG_FILE" 2>&1
    local exit_code=$?
    
    if [[ $exit_code -eq 0 ]]; then
        log_daemon "✅ 下载脚本正常退出"
        return 0
    else
        log_daemon "⚠️ 下载脚本退出，代码: $exit_code"
        return 1
    fi
}

# 主循环 - 无限重试
log_daemon "═══════════════════════════════════════════"
log_daemon "🌙 Valhalla 下载守护进程启动"
log_daemon "═══════════════════════════════════════════"
log_daemon "重试延迟: ${RETRY_DELAY}秒"
log_daemon "模式: 无限重试直到完成"

attempt=0
while true; do
    ((attempt++))
    log_daemon "========== 下载尝试 #$attempt =========="
    
    if run_download; then
        log_daemon "🎉 所有下载完成!"
        log_daemon "═══════════════════════════════════════════"
        exit 0
    fi
    
    log_daemon "⏳ ${RETRY_DELAY}秒后重试..."
    sleep $RETRY_DELAY
    
    # 每10次重试清理一下可能卡住的aria2进程
    if (( attempt % 10 == 0 )); then
        log_daemon "🧹 清理可能卡住的进程..."
        pkill -9 -f "aria2c" 2>/dev/null
        sleep 5
    fi
done
