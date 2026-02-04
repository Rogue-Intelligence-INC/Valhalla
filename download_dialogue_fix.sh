#!/bin/bash

# 修复版：下载对话/小说语料
CORPUS="/run/media/jouly/My Book/valhalla_corpus"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  🔧 修复下载 - 对话/小说语料                                   ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# ============================================
# 1. 中文小说 - 使用可靠源
# ============================================
echo ""
echo "📚 下载中文经典小说..."
mkdir -p "$CORPUS/novels/chinese"
cd "$CORPUS/novels/chinese"

# 从 Wikisource 或其他可靠源下载
# 红楼梦
echo "  红楼梦..."
curl -L -o hongloumeng.txt "https://raw.githubusercontent.com/garfieldnate/traditional_chinese_literature/master/红楼梦/红楼梦.txt" 2>/dev/null || \
curl -L -o hongloumeng.txt "https://ctext.org/library/book.pl?book=hongloumeng&format=txt" 2>/dev/null &

# 尝试从另一个源下载中文文学
echo "  下载中文对话语料 (LCCC)..."
curl -L -o lccc_base.zip "https://github.com/thu-coai/CDial-GPT/raw/master/data/LCCC-base.zip" 2>/dev/null &

# ============================================
# 2. 多语言字幕 - 使用 HuggingFace
# ============================================
echo ""
echo "🎬 下载多语言字幕对话..."
mkdir -p "$CORPUS/dialogues/subtitles"
cd "$CORPUS/dialogues/subtitles"

# 使用 HuggingFace datasets 的电影对话
echo "  电影对话数据集..."
curl -L -o movie_dialog.tar.gz "https://parl.ai/downloads/md_gender/md_gender.tgz" 2>/dev/null &

# ============================================
# 3. 更多小说来源 - Wikisource / 其他
# ============================================
echo ""
echo "📖 下载更多文学作品..."
mkdir -p "$CORPUS/novels/misc"
cd "$CORPUS/novels/misc"

# Writing Prompts dataset (Reddit 写作)
echo "  WritingPrompts..."
curl -L -o writingprompts.tar.gz "https://huggingface.co/datasets/euclaise/writingprompts/resolve/main/data.tar.gz" 2>/dev/null &

# BookCorpus 样本
echo "  BookCorpus 样本..."
curl -L -o bookcorpus_sample.txt "https://huggingface.co/datasets/bookcorpus/resolve/main/plain_text/bookcorpus-train.txt" 2>/dev/null &

# ============================================
# 4. 日语对话/小说
# ============================================
echo ""
echo "🗾 下载日语对话..."
mkdir -p "$CORPUS/dialogues/japanese"
cd "$CORPUS/dialogues/japanese"

# JParaCrawl (日英平行语料，含大量对话)
echo "  日语对话语料..."
curl -L -o japanese_dialog_sample.txt "https://huggingface.co/datasets/kunishou/oasst1-89k-ja/resolve/main/data/oasst1_ja_89k.json" 2>/dev/null &

# ============================================
# 5. 韩语对话
# ============================================
echo ""
echo "🇰🇷 下载韩语对话..."
mkdir -p "$CORPUS/dialogues/korean"
cd "$CORPUS/dialogues/korean"

echo "  韩语对话语料..."
curl -L -o korean_dialog.json "https://huggingface.co/datasets/heegyu/kowiki/resolve/main/data/train-00000-of-00001.json" 2>/dev/null &

# ============================================
# 6. 电影/电视剧剧本
# ============================================
echo ""
echo "🎭 下载剧本..."
mkdir -p "$CORPUS/scripts/screenplays"
cd "$CORPUS/scripts/screenplays"

# 电影剧本数据集
echo "  电影剧本..."
# 从 Kaggle 或其他源
curl -L -o screenplays.zip "https://github.com/Aveek-Saha/Movie-Script-Database/archive/refs/heads/master.zip" 2>/dev/null &

# ============================================
# 7. 对话密集的文本 - ConvAI2, Blended Skill Talk
# ============================================
echo ""
echo "💬 下载更多对话数据..."
mkdir -p "$CORPUS/dialogues/convai"
cd "$CORPUS/dialogues/convai"

# ConvAI2
echo "  ConvAI2..."
curl -L -o convai2.tar.gz "https://parl.ai/downloads/convai2/convai2.tar.gz" 2>/dev/null &

# Blended Skill Talk
echo "  Blended Skill Talk..."
curl -L -o blended_skill_talk.tar.gz "https://parl.ai/downloads/blended_skill_talk/blended_skill_talk.tar.gz" 2>/dev/null &

echo ""
echo "⏳ 等待下载完成..."
wait

# 解压
echo ""
echo "📦 解压文件..."
cd "$CORPUS/novels/chinese" && unzip -o lccc_base.zip 2>/dev/null
cd "$CORPUS/dialogues/convai" && tar -xzf convai2.tar.gz 2>/dev/null
cd "$CORPUS/dialogues/convai" && tar -xzf blended_skill_talk.tar.gz 2>/dev/null
cd "$CORPUS/scripts/screenplays" && unzip -o screenplays.zip 2>/dev/null

echo ""
echo "✅ 修复下载完成！"
