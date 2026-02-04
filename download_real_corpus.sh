#!/bin/bash
# 下载真正的训练语料到 My Book

CORPUS_DIR="/run/media/jouly/My Book/valhalla_corpus"
cd "$CORPUS_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     📦 下载 Valhalla 训练语料                                  ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# ========== 文本语料 ==========
echo -e "\n📚 下载文本语料..."

# Wikipedia 简单版本 (小文件，可靠)
echo "  下载 Simple Wikipedia..."
cd text
curl -L -o simplewiki.xml.bz2 "https://dumps.wikimedia.org/simplewiki/latest/simplewiki-latest-pages-articles.xml.bz2" 2>/dev/null &

# 中文维基摘要
echo "  下载中文维基摘要..."
curl -L -o zhwiki-abstract.xml.gz "https://dumps.wikimedia.org/zhwiki/latest/zhwiki-latest-abstract.xml.gz" 2>/dev/null &

# OpenWebText 样本 (小)
echo "  下载 OpenWebText 样本..."
curl -L -o openwebtext_sample.tar.gz "https://huggingface.co/datasets/Skylion007/openwebtext/resolve/main/subsets/urlsf_subset00-1_data.tar.gz" 2>/dev/null &

cd ..

# ========== 代码语料 ==========
echo -e "\n💻 下载代码语料..."
cd code

# The Stack 样本
echo "  下载代码样本..."
curl -L -o rust_samples.jsonl "https://huggingface.co/datasets/bigcode/the-stack/resolve/main/data/rust/train-00000-of-00008.parquet" 2>/dev/null &
curl -L -o python_samples.jsonl "https://huggingface.co/datasets/bigcode/the-stack/resolve/main/data/python/train-00000-of-00152.parquet" 2>/dev/null &

cd ..

# ========== 数学语料 ==========
echo -e "\n🔢 下载数学语料..."
cd math

# MATH 数据集
echo "  下载数学数据集..."
curl -L -o math_dataset.tar.gz "https://github.com/hendrycks/math/archive/refs/heads/main.zip" 2>/dev/null &

cd ..

# ========== 等待下载 ==========
echo -e "\n⏳ 等待下载完成..."
wait

echo -e "\n✅ 下载完成！"
ls -lh "$CORPUS_DIR"/*
