#!/bin/bash
#
# Valhalla 训练数据下载 - 自动重试包装脚本
# 如果下载中断，会自动重新启动
#
# 使用: nohup ./download_auto_retry.sh > download_retry.log 2>&1 &
#

SCRIPT_DIR="/home/jouly/AI/Valhalla"
DOWNLOAD_SCRIPT="$SCRIPT_DIR/download_training_data.sh"
LOG_FILE="$SCRIPT_DIR/download.log"
RETRY_LOG="$SCRIPT_DIR/download_retry.log"
MAX_GLOBAL_RETRIES=10
RETRY_DELAY=30  # 秒

log_retry() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$RETRY_LOG"
}

check_disk_mounted() {
    if [[ -d "/run/media/jouly/My Book" ]]; then
        return 0
    else
        return 1
    fi
}

run_download() {
    local attempt=$1
    log_retry "========== 下载尝试 #$attempt =========="
    
    # 检查硬盘是否挂载
    if ! check_disk_mounted; then
        log_retry "⚠️ 外置硬盘未挂载，等待 60 秒..."
        sleep 60
        return 1
    fi
    
    cd "$SCRIPT_DIR"
    
    # 运行下载脚本
    bash "$DOWNLOAD_SCRIPT" >> "$LOG_FILE" 2>&1
    local exit_code=$?
    
    if [[ $exit_code -eq 0 ]]; then
        log_retry "✅ 下载成功完成!"
        return 0
    else
        log_retry "❌ 下载退出，代码: $exit_code"
        return 1
    fi
}

# 主循环
log_retry "🚀 启动自动重试下载器"
log_retry "最大重试次数: $MAX_GLOBAL_RETRIES"
log_retry "重试延迟: ${RETRY_DELAY}秒"

for ((i=1; i<=MAX_GLOBAL_RETRIES; i++)); do
    if run_download $i; then
        log_retry "🎉 所有下载完成!"
        exit 0
    fi
    
    if [[ $i -lt $MAX_GLOBAL_RETRIES ]]; then
        log_retry "⏳ ${RETRY_DELAY}秒后重试..."
        sleep $RETRY_DELAY
    fi
done

log_retry "❌ 达到最大重试次数 ($MAX_GLOBAL_RETRIES)，放弃"
exit 1
