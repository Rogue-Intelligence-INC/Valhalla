#!/bin/bash
#
# Valhalla 大规模训练数据下载脚本
# 支持断点续传，自动重试
# 
# 使用: ./download_training_data.sh
# 后台运行: nohup ./download_training_data.sh > download.log 2>&1 &
#

set -e

# ==================== 配置 ====================
DOWNLOAD_DIR="/run/media/jouly/My Book/valhalla_training_data"
LOG_FILE="/home/jouly/AI/Valhalla/download_training.log"
MAX_RETRIES=5
PARALLEL_DOWNLOADS=4

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# ==================== 函数 ====================

log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

download_with_resume() {
    local url=$1
    local output=$2
    local name=$3
    
    log "${YELLOW}下载: $name${NC}"
    
    # 优先使用 aria2c (更快，支持多线程)
    if command -v aria2c &> /dev/null; then
        aria2c -c -x 8 -s 8 -k 1M --max-tries=$MAX_RETRIES \
            --retry-wait=10 -d "$(dirname "$output")" -o "$(basename "$output")" "$url" || true
    else
        # 使用 wget 断点续传
        wget -c --tries=$MAX_RETRIES --timeout=60 -O "$output" "$url" || true
    fi
    
    if [[ -f "$output" ]]; then
        log "${GREEN}✓ 完成: $name${NC}"
        return 0
    else
        log "${RED}✗ 失败: $name${NC}"
        return 1
    fi
}

download_dataset() {
    local name=$1
    local url=$2
    local dest=$3
    local extracted_dir=$4  # 可选：解压后的目录
    
    mkdir -p "$(dirname "$dest")"
    
    # 检查文件是否已存在
    if [[ -f "$dest" ]]; then
        log "${GREEN}跳过 (已存在): $name${NC}"
        return 0
    fi
    
    # 检查解压后的目录是否已存在
    if [[ -n "$extracted_dir" ]] && [[ -d "$extracted_dir" ]]; then
        local count=$(find "$extracted_dir" -type f 2>/dev/null | wc -l)
        if [[ $count -gt 0 ]]; then
            log "${GREEN}跳过 (已解压): $name ($count 文件)${NC}"
            return 0
        fi
    fi
    
    download_with_resume "$url" "$dest" "$name"
}

extract_if_archive() {
    local file=$1
    local dest_dir=$2
    
    if [[ "$file" == *.tar.gz ]] || [[ "$file" == *.tgz ]]; then
        log "解压: $file"
        tar -xzf "$file" -C "$dest_dir" && rm "$file"
    elif [[ "$file" == *.zip ]]; then
        log "解压: $file"
        unzip -q -o "$file" -d "$dest_dir" && rm "$file"
    elif [[ "$file" == *.tar ]]; then
        log "解压: $file"
        tar -xf "$file" -C "$dest_dir" && rm "$file"
    fi
}

# ==================== 主程序 ====================

log "=========================================="
log "  Valhalla 训练数据下载 - 开始"
log "  目标目录: $DOWNLOAD_DIR"
log "=========================================="

mkdir -p "$DOWNLOAD_DIR"/{images,audio,video,text,dialogue,code}

# ==================== 1. 图片数据集 ====================
log ""
log "=== 1. 图片数据集 ==="

# COCO 2017 (训练集图片 - 18GB)
download_dataset "COCO 2017 Train Images" \
    "http://images.cocodataset.org/zips/train2017.zip" \
    "$DOWNLOAD_DIR/images/coco_train2017.zip" \
    "$DOWNLOAD_DIR/images/train2017"
[[ -f "$DOWNLOAD_DIR/images/coco_train2017.zip" ]] && extract_if_archive "$DOWNLOAD_DIR/images/coco_train2017.zip" "$DOWNLOAD_DIR/images/"

# COCO 2017 Annotations
download_dataset "COCO 2017 Annotations" \
    "http://images.cocodataset.org/annotations/annotations_trainval2017.zip" \
    "$DOWNLOAD_DIR/images/coco_annotations.zip" \
    "$DOWNLOAD_DIR/images/annotations"
[[ -f "$DOWNLOAD_DIR/images/coco_annotations.zip" ]] && extract_if_archive "$DOWNLOAD_DIR/images/coco_annotations.zip" "$DOWNLOAD_DIR/images/"

# Flickr30k (需要注册，用描述代替)
download_dataset "Flickr30k Entities" \
    "https://github.com/BryanPlummer/flickr30k_entities/raw/master/annotations.zip" \
    "$DOWNLOAD_DIR/images/flickr30k_annotations.zip"

# Open Images (子集)
download_dataset "Open Images Descriptions" \
    "https://storage.googleapis.com/openimages/web/download.html" \
    "$DOWNLOAD_DIR/images/open_images_info.txt" || true

# ==================== 2. 音频数据集 ====================
log ""
log "=== 2. 音频数据集 ==="

# LibriSpeech (clean-100 - 6.3GB)
download_dataset "LibriSpeech clean-100" \
    "https://www.openslr.org/resources/12/train-clean-100.tar.gz" \
    "$DOWNLOAD_DIR/audio/librispeech_clean100.tar.gz" \
    "$DOWNLOAD_DIR/audio/LibriSpeech"
[[ -f "$DOWNLOAD_DIR/audio/librispeech_clean100.tar.gz" ]] && extract_if_archive "$DOWNLOAD_DIR/audio/librispeech_clean100.tar.gz" "$DOWNLOAD_DIR/audio/"

# Common Voice (需要手动下载，提供说明)
cat > "$DOWNLOAD_DIR/audio/README_common_voice.txt" << 'EOF'
Common Voice 数据集需要手动下载:
1. 访问 https://commonvoice.mozilla.org/zh-CN/datasets
2. 选择语言 (推荐: 中文、英语、日语)
3. 下载 validated.tsv 和对应音频
4. 解压到此目录
EOF

# VoxCeleb (说话人识别)
download_dataset "VoxCeleb Dev List" \
    "https://www.robots.ox.ac.uk/~vgg/data/voxceleb/meta/vox1_dev_txt.zip" \
    "$DOWNLOAD_DIR/audio/voxceleb_dev.zip" || true

# ==================== 3. 视频数据集 ====================
log ""
log "=== 3. 视频数据集 ==="

# Kinetics-400 (需要 youtube-dl)
cat > "$DOWNLOAD_DIR/video/download_kinetics.sh" << 'EOF'
#!/bin/bash
# Kinetics-400 下载脚本
# 需要 yt-dlp 工具

# 安装 yt-dlp (如果没有)
# pip install yt-dlp

# 下载训练集 CSV
wget -c "https://storage.googleapis.com/deepmind-media/Datasets/kinetics400.tar.gz" -O kinetics400.tar.gz
tar -xzf kinetics400.tar.gz

# 使用 yt-dlp 下载视频 (示例)
# yt-dlp -f 'best[height<=480]' --download-archive downloaded.txt -a video_urls.txt
EOF
chmod +x "$DOWNLOAD_DIR/video/download_kinetics.sh"

# ActivityNet Captions
download_dataset "ActivityNet Captions" \
    "http://ec2-52-25-205-214.us-west-2.compute.amazonaws.com/files/activity_net.v1-3.min.json" \
    "$DOWNLOAD_DIR/video/activitynet_captions.json" || true

# MSR-VTT (视频描述)
download_dataset "MSR-VTT Info" \
    "https://github.com/VisionLearningGroup/caption-guided-saliency/raw/master/data/MSRVTT/videodatainfo_2017.json" \
    "$DOWNLOAD_DIR/video/msrvtt_info.json" || true

# ==================== 4. 文本数据集 ====================
log ""
log "=== 4. 文本数据集 ==="

# Wikipedia (中文)
download_dataset "Wikipedia 中文" \
    "https://dumps.wikimedia.org/zhwiki/latest/zhwiki-latest-pages-articles.xml.bz2" \
    "$DOWNLOAD_DIR/text/zhwiki-latest.xml.bz2"

# Wikipedia (英文 - 更小的版本)
download_dataset "Wikipedia 英文简介" \
    "https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-abstract.xml.gz" \
    "$DOWNLOAD_DIR/text/enwiki-abstract.xml.gz"

# BookCorpus (替代: Project Gutenberg)
download_dataset "Gutenberg 目录" \
    "https://www.gutenberg.org/cache/epub/feeds/pg_catalog.csv" \
    "$DOWNLOAD_DIR/text/gutenberg_catalog.csv"

# CC-News
download_dataset "CC-News 样本" \
    "https://commoncrawl.s3.amazonaws.com/crawl-data/CC-NEWS/2024/01/CC-NEWS-20240101001538-00000.warc.gz" \
    "$DOWNLOAD_DIR/text/ccnews_sample.warc.gz" || true

# OpenWebText (需要工具处理)
cat > "$DOWNLOAD_DIR/text/download_openwebtext.sh" << 'EOF'
#!/bin/bash
# OpenWebText2 下载脚本
# 需要 pushshift 爬虫工具

# 方案1: 使用 HuggingFace datasets
# pip install datasets
# python -c "from datasets import load_dataset; ds = load_dataset('openwebtext'); ds.save_to_disk('openwebtext')"

# 方案2: 直接下载
# wget -c "https://the-eye.eu/public/AI/pile_preliminary_components/openwebtext2.jsonl.zst.tar"
EOF

# ==================== 5. 对话数据集 ====================
log ""
log "=== 5. 对话数据集 ==="

# DailyDialog
download_dataset "DailyDialog" \
    "http://yanran.li/files/ijcnlp_dailydialog.zip" \
    "$DOWNLOAD_DIR/dialogue/dailydialog.zip"
[[ -f "$DOWNLOAD_DIR/dialogue/dailydialog.zip" ]] && extract_if_archive "$DOWNLOAD_DIR/dialogue/dailydialog.zip" "$DOWNLOAD_DIR/dialogue/"

# Cornell Movie Dialogs
download_dataset "Cornell Movie Dialogs" \
    "http://www.cs.cornell.edu/~cristian/data/cornell_movie_dialogs_corpus.zip" \
    "$DOWNLOAD_DIR/dialogue/cornell_movie.zip"
[[ -f "$DOWNLOAD_DIR/dialogue/cornell_movie.zip" ]] && extract_if_archive "$DOWNLOAD_DIR/dialogue/cornell_movie.zip" "$DOWNLOAD_DIR/dialogue/"

# Persona Chat
download_dataset "Persona Chat" \
    "https://s3.amazonaws.com/datasets.huggingface.co/personachat/personachat_self_original.json" \
    "$DOWNLOAD_DIR/dialogue/personachat.json"

# Ubuntu Dialogue Corpus
download_dataset "Ubuntu Dialogue" \
    "http://cs.mcgill.ca/~jpineau/datasets/ubuntu-corpus-1.0/ubuntu_dialogs.tgz" \
    "$DOWNLOAD_DIR/dialogue/ubuntu_dialogs.tgz" || true

# 中文对话 (LCCC)
download_dataset "LCCC 中文对话" \
    "https://github.com/thu-coai/CDial-GPT/raw/master/data/LCCC-base_train.json" \
    "$DOWNLOAD_DIR/dialogue/lccc_train.json" || true

# ==================== 6. 代码数据集 ====================
log ""
log "=== 6. 代码数据集 ==="

# CodeSearchNet
download_dataset "CodeSearchNet Python" \
    "https://s3.amazonaws.com/code-search-net/CodeSearchNet/v2/python.zip" \
    "$DOWNLOAD_DIR/code/codesearchnet_python.zip"
[[ -f "$DOWNLOAD_DIR/code/codesearchnet_python.zip" ]] && extract_if_archive "$DOWNLOAD_DIR/code/codesearchnet_python.zip" "$DOWNLOAD_DIR/code/"

download_dataset "CodeSearchNet JavaScript" \
    "https://s3.amazonaws.com/code-search-net/CodeSearchNet/v2/javascript.zip" \
    "$DOWNLOAD_DIR/code/codesearchnet_js.zip"
[[ -f "$DOWNLOAD_DIR/code/codesearchnet_js.zip" ]] && extract_if_archive "$DOWNLOAD_DIR/code/codesearchnet_js.zip" "$DOWNLOAD_DIR/code/"

# The Stack (需要 HuggingFace)
cat > "$DOWNLOAD_DIR/code/download_thestack.sh" << 'EOF'
#!/bin/bash
# The Stack 代码数据集
# 需要 HuggingFace 账号和 datasets 库

# pip install datasets huggingface_hub
# huggingface-cli login

# python << 'PYTHON'
# from datasets import load_dataset
# ds = load_dataset("bigcode/the-stack", data_dir="data/python", split="train", streaming=True)
# # 保存子集
# for i, sample in enumerate(ds):
#     if i >= 100000: break
#     # 保存 sample
# PYTHON
EOF

# ==================== 7. 多模态数据集 ====================
log ""
log "=== 7. 多模态数据集 ==="

# Visual Genome
download_dataset "Visual Genome Relationships" \
    "https://homes.cs.washington.edu/~ranjay/visualgenome/data/dataset/relationships.json.zip" \
    "$DOWNLOAD_DIR/images/visualgenome_relationships.zip" || true

# Conceptual Captions (需要脚本下载)
download_dataset "Conceptual Captions TSV" \
    "https://storage.googleapis.com/gcc-data/Train/GCC-training.tsv" \
    "$DOWNLOAD_DIR/images/gcc_training.tsv" || true

# ==================== 8. 数学数据集 ====================
log ""
log "=== 8. 数学数据集 ==="

# MATH Dataset
download_dataset "MATH Dataset" \
    "https://people.eecs.berkeley.edu/~hendrycks/MATH.tar" \
    "$DOWNLOAD_DIR/text/math_dataset.tar" || true
[[ -f "$DOWNLOAD_DIR/text/math_dataset.tar" ]] && extract_if_archive "$DOWNLOAD_DIR/text/math_dataset.tar" "$DOWNLOAD_DIR/text/"

# GSM8K
download_dataset "GSM8K Train" \
    "https://raw.githubusercontent.com/openai/grade-school-math/master/grade_school_math/data/train.jsonl" \
    "$DOWNLOAD_DIR/text/gsm8k_train.jsonl"

# ==================== 完成 ====================

log ""
log "=========================================="
log "  下载完成!"
log "=========================================="
log ""

# 统计
log "文件统计:"
for dir in images audio video text dialogue code; do
    count=$(find "$DOWNLOAD_DIR/$dir" -type f 2>/dev/null | wc -l)
    size=$(du -sh "$DOWNLOAD_DIR/$dir" 2>/dev/null | cut -f1)
    log "  $dir: $count 文件, $size"
done

total_size=$(du -sh "$DOWNLOAD_DIR" 2>/dev/null | cut -f1)
log ""
log "总计: $total_size"
log ""
log "日志文件: $LOG_FILE"
