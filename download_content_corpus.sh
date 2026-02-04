#!/bin/bash

# ═══════════════════════════════════════════════════════════════
#  Valhalla 内容语料下载器
#  下载对话、小说、文章等上下文材料
# ═══════════════════════════════════════════════════════════════

CORPUS_ROOT="/run/media/jouly/My Book/valhalla_corpus/content"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     📦 Valhalla 内容语料下载                                   ║"
echo "║     对话 / 小说 / 文章 / 问答                                  ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# 创建目录结构
mkdir -p "$CORPUS_ROOT/dialogues"      # 对话
mkdir -p "$CORPUS_ROOT/novels"         # 小说/故事
mkdir -p "$CORPUS_ROOT/articles"       # 文章/新闻
mkdir -p "$CORPUS_ROOT/qa"             # 问答对
mkdir -p "$CORPUS_ROOT/scripts"        # 剧本/字幕
mkdir -p "$CORPUS_ROOT/social"         # 社交媒体

echo ""
echo "📁 目录结构已创建"
ls -la "$CORPUS_ROOT"

# ═══════════════════════════════════════════════════════════════
# 1. 对话数据
# ═══════════════════════════════════════════════════════════════
echo ""
echo "💬 [1/6] 下载对话数据..."

# DailyDialog - 日常对话数据集
echo "  下载 DailyDialog..."
curl -sL "https://huggingface.co/datasets/daily_dialog/resolve/main/data/train.json" \
    -o "$CORPUS_ROOT/dialogues/dailydialog_train.json" &

# Persona-Chat - 个性化对话
echo "  下载 PersonaChat..."
curl -sL "https://huggingface.co/datasets/bavard/personachat_truecased/resolve/main/data/train.json" \
    -o "$CORPUS_ROOT/dialogues/personachat.json" &

# Ubuntu Dialogue Corpus (样本)
echo "  下载 Ubuntu Dialogue 样本..."
curl -sL "https://raw.githubusercontent.com/rkadlec/ubuntu-ranking-dataset-creator/master/src/sample_data.json" \
    -o "$CORPUS_ROOT/dialogues/ubuntu_sample.json" &

# ═══════════════════════════════════════════════════════════════
# 2. 小说/故事
# ═══════════════════════════════════════════════════════════════
echo ""
echo "📚 [2/6] 下载小说/故事..."

# Project Gutenberg - 经典文学
echo "  下载经典文学..."
# 爱丽丝梦游仙境
curl -sL "https://www.gutenberg.org/cache/epub/11/pg11.txt" \
    -o "$CORPUS_ROOT/novels/alice_wonderland.txt" &
# 傲慢与偏见
curl -sL "https://www.gutenberg.org/cache/epub/1342/pg1342.txt" \
    -o "$CORPUS_ROOT/novels/pride_prejudice.txt" &
# 福尔摩斯探案集
curl -sL "https://www.gutenberg.org/cache/epub/1661/pg1661.txt" \
    -o "$CORPUS_ROOT/novels/sherlock_holmes.txt" &
# 双城记
curl -sL "https://www.gutenberg.org/cache/epub/98/pg98.txt" \
    -o "$CORPUS_ROOT/novels/tale_two_cities.txt" &
# 莫比迪克
curl -sL "https://www.gutenberg.org/cache/epub/2701/pg2701.txt" \
    -o "$CORPUS_ROOT/novels/moby_dick.txt" &
# 弗兰肯斯坦
curl -sL "https://www.gutenberg.org/cache/epub/84/pg84.txt" \
    -o "$CORPUS_ROOT/novels/frankenstein.txt" &
# 德古拉
curl -sL "https://www.gutenberg.org/cache/epub/345/pg345.txt" \
    -o "$CORPUS_ROOT/novels/dracula.txt" &
# 1984 (公共版权版本)
curl -sL "https://www.gutenberg.org/cache/epub/1232/pg1232.txt" \
    -o "$CORPUS_ROOT/novels/machiavelli_prince.txt" &

# ROCStories - 短故事
echo "  下载短故事数据集..."
curl -sL "https://huggingface.co/datasets/roneneldan/TinyStories/resolve/main/data/train-00000-of-00004.parquet" \
    -o "$CORPUS_ROOT/novels/tinystories_train.parquet" &

# ═══════════════════════════════════════════════════════════════
# 3. 新闻/文章
# ═══════════════════════════════════════════════════════════════
echo ""
echo "📰 [3/6] 下载新闻/文章..."

# CNN/DailyMail 样本
echo "  下载新闻样本..."
curl -sL "https://huggingface.co/datasets/cnn_dailymail/resolve/main/data/3.0.0/train-00000-of-00003.parquet" \
    -o "$CORPUS_ROOT/articles/cnn_dailymail_train.parquet" &

# AG News
echo "  下载 AG News..."
curl -sL "https://raw.githubusercontent.com/mhjabreel/CharCnn_Keras/master/data/ag_news_csv/train.csv" \
    -o "$CORPUS_ROOT/articles/ag_news_train.csv" &

# ═══════════════════════════════════════════════════════════════
# 4. 问答数据
# ═══════════════════════════════════════════════════════════════
echo ""
echo "❓ [4/6] 下载问答数据..."

# SQuAD 2.0
echo "  下载 SQuAD..."
curl -sL "https://rajpurkar.github.io/SQuAD-explorer/dataset/train-v2.0.json" \
    -o "$CORPUS_ROOT/qa/squad_train.json" &

# Natural Questions 样本
echo "  下载 NQ 样本..."
curl -sL "https://huggingface.co/datasets/natural_questions/resolve/main/data/v1.0_simplified/nq-train-00.jsonl.gz" \
    -o "$CORPUS_ROOT/qa/nq_train_sample.jsonl.gz" &

# TriviaQA
echo "  下载 TriviaQA..."
curl -sL "http://nlp.cs.washington.edu/triviaqa/data/triviaqa-rc.tar.gz" \
    -o "$CORPUS_ROOT/qa/triviaqa.tar.gz" &

# ═══════════════════════════════════════════════════════════════
# 5. 剧本/字幕
# ═══════════════════════════════════════════════════════════════
echo ""
echo "🎬 [5/6] 下载剧本/字幕..."

# Cornell Movie Dialogs
echo "  下载电影对话..."
curl -sL "http://www.cs.cornell.edu/~cristian/data/cornell_movie_dialogs_corpus.zip" \
    -o "$CORPUS_ROOT/scripts/cornell_movie_dialogs.zip" &

# OpenSubtitles 样本 (通过 OPUS)
echo "  下载字幕样本..."
curl -sL "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/moses/en-zh_cn.txt.zip" \
    -o "$CORPUS_ROOT/scripts/opensubtitles_en_zh.zip" &

# ═══════════════════════════════════════════════════════════════
# 6. 中文内容
# ═══════════════════════════════════════════════════════════════
echo ""
echo "🇨🇳 [6/6] 下载中文内容..."

# 中文小说 (公共领域)
echo "  下载中文经典..."
# 红楼梦
curl -sL "https://raw.githubusercontent.com/blmoiern/HLM/master/chapter.txt" \
    -o "$CORPUS_ROOT/novels/hongloumeng.txt" &

# 中文对话
echo "  下载中文对话数据..."
curl -sL "https://huggingface.co/datasets/shibing624/nli_zh/resolve/main/data/train.jsonl" \
    -o "$CORPUS_ROOT/dialogues/chinese_nli.jsonl" &

# LCCC 中文对话
curl -sL "https://huggingface.co/datasets/silver/lccc/resolve/main/data/train.json" \
    -o "$CORPUS_ROOT/dialogues/lccc_chinese.json" &

# ═══════════════════════════════════════════════════════════════
# 等待完成
# ═══════════════════════════════════════════════════════════════
echo ""
echo "⏳ 等待所有下载完成..."
wait

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✅ 下载完成！"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📊 下载统计:"
du -sh "$CORPUS_ROOT"/*
echo ""
echo "📁 语料位置: $CORPUS_ROOT"
