#!/bin/bash

# 对话/小说/剧本语料下载脚本
# 目标: 对话密集的内容，不限语言

CORPUS_ROOT="/run/media/jouly/My Book/valhalla_corpus"
DIALOGUE_DIR="$CORPUS_ROOT/dialogues"
SCRIPT_DIR="$CORPUS_ROOT/scripts"
NOVEL_DIR="$CORPUS_ROOT/novels"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  📚 下载对话密集语料 (小说/剧本/对话)                         ║"
echo "╚════════════════════════════════════════════════════════════════╝"

mkdir -p "$DIALOGUE_DIR" "$SCRIPT_DIR" "$NOVEL_DIR"

# ============================================
# 1. 对话数据集
# ============================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📥 1. 对话数据集"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# DailyDialog - 日常对话
echo "  [1/10] DailyDialog (日常对话)..."
curl -L -o "$DIALOGUE_DIR/dailydialog.zip" \
    "http://yanran.li/files/ijcnlp_dailydialog.zip" 2>/dev/null &

# Cornell Movie Dialogs - 电影对话 (经典!)
echo "  [2/10] Cornell Movie Dialogs (电影对话)..."
curl -L -o "$DIALOGUE_DIR/cornell_movie_dialogs.zip" \
    "http://www.cs.cornell.edu/~cristian/data/cornell_movie_dialogs_corpus.zip" 2>/dev/null &

# Ubuntu Dialogue Corpus - 技术对话
echo "  [3/10] Ubuntu Dialogue Corpus..."
curl -L -o "$DIALOGUE_DIR/ubuntu_dialogs.tar.gz" \
    "https://github.com/rkadlec/ubuntu-ranking-dataset-creator/raw/master/src/ubuntu_data.tar.gz" 2>/dev/null &

# Persona-Chat - 角色对话
echo "  [4/10] Persona-Chat..."
curl -L -o "$DIALOGUE_DIR/personachat.tar.gz" \
    "https://parl.ai/downloads/personachat/personachat.tar.gz" 2>/dev/null &

# Empathetic Dialogues - 情感对话
echo "  [5/10] Empathetic Dialogues..."
curl -L -o "$DIALOGUE_DIR/empatheticdialogues.tar.gz" \
    "https://parl.ai/downloads/empatheticdialogues/empatheticdialogues.tar.gz" 2>/dev/null &

# MultiWOZ - 多领域对话
echo "  [6/10] MultiWOZ (多领域对话)..."
curl -L -o "$DIALOGUE_DIR/multiwoz.zip" \
    "https://github.com/budzianowski/multiwoz/raw/master/data/MultiWOZ_2.1.zip" 2>/dev/null &

# Wizard of Wikipedia - 知识对话
echo "  [7/10] Wizard of Wikipedia..."
curl -L -o "$DIALOGUE_DIR/wizard_of_wikipedia.tar.gz" \
    "https://parl.ai/downloads/wizard_of_wikipedia/wizard_of_wikipedia.tar.gz" 2>/dev/null &

# OpenSubtitles - 字幕对话 (多语言!)
echo "  [8/10] OpenSubtitles 样本 (多语言字幕)..."
mkdir -p "$DIALOGUE_DIR/opensubtitles"
# 英语
curl -L -o "$DIALOGUE_DIR/opensubtitles/en.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.en.gz" 2>/dev/null &
# 中文
curl -L -o "$DIALOGUE_DIR/opensubtitles/zh.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.zh_cn.gz" 2>/dev/null &
# 日语
curl -L -o "$DIALOGUE_DIR/opensubtitles/ja.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.ja.gz" 2>/dev/null &
# 韩语
curl -L -o "$DIALOGUE_DIR/opensubtitles/ko.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.ko.gz" 2>/dev/null &
# 法语
curl -L -o "$DIALOGUE_DIR/opensubtitles/fr.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.fr.gz" 2>/dev/null &
# 德语
curl -L -o "$DIALOGUE_DIR/opensubtitles/de.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.de.gz" 2>/dev/null &
# 西班牙语
curl -L -o "$DIALOGUE_DIR/opensubtitles/es.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.es.gz" 2>/dev/null &
# 俄语
curl -L -o "$DIALOGUE_DIR/opensubtitles/ru.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.ru.gz" 2>/dev/null &
# 阿拉伯语
curl -L -o "$DIALOGUE_DIR/opensubtitles/ar.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.ar.gz" 2>/dev/null &
# 葡萄牙语
curl -L -o "$DIALOGUE_DIR/opensubtitles/pt.txt.gz" \
    "https://opus.nlpl.eu/download.php?f=OpenSubtitles/v2018/mono/OpenSubtitles.raw.pt.gz" 2>/dev/null &

# ============================================
# 2. 剧本数据
# ============================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📥 2. 剧本数据"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# IMSDB 电影剧本
echo "  [9/10] 电影剧本集合..."
mkdir -p "$SCRIPT_DIR/movies"

# 克隆著名剧本集合
git clone --depth 1 https://github.com/j2kun/movie-script-database.git "$SCRIPT_DIR/movie_scripts_db" 2>/dev/null &

# TV剧本
echo "  [10/10] TV剧本..."
mkdir -p "$SCRIPT_DIR/tv"

# ============================================
# 3. 小说/文学作品 (对话密集)
# ============================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📥 3. 小说/文学 (Project Gutenberg)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p "$NOVEL_DIR/gutenberg"

# 下载对话密集的经典小说
echo "  下载经典文学作品..."

# 英语小说 (对话密集)
# Pride and Prejudice - Jane Austen
curl -L -o "$NOVEL_DIR/gutenberg/pride_prejudice.txt" \
    "https://www.gutenberg.org/cache/epub/1342/pg1342.txt" 2>/dev/null &
# Alice in Wonderland
curl -L -o "$NOVEL_DIR/gutenberg/alice_wonderland.txt" \
    "https://www.gutenberg.org/cache/epub/11/pg11.txt" 2>/dev/null &
# The Great Gatsby
curl -L -o "$NOVEL_DIR/gutenberg/great_gatsby.txt" \
    "https://www.gutenberg.org/cache/epub/64317/pg64317.txt" 2>/dev/null &
# Sherlock Holmes
curl -L -o "$NOVEL_DIR/gutenberg/sherlock_holmes.txt" \
    "https://www.gutenberg.org/cache/epub/1661/pg1661.txt" 2>/dev/null &
# Jane Eyre
curl -L -o "$NOVEL_DIR/gutenberg/jane_eyre.txt" \
    "https://www.gutenberg.org/cache/epub/1260/pg1260.txt" 2>/dev/null &
# Dracula
curl -L -o "$NOVEL_DIR/gutenberg/dracula.txt" \
    "https://www.gutenberg.org/cache/epub/345/pg345.txt" 2>/dev/null &
# Frankenstein
curl -L -o "$NOVEL_DIR/gutenberg/frankenstein.txt" \
    "https://www.gutenberg.org/cache/epub/84/pg84.txt" 2>/dev/null &
# Emma - Jane Austen
curl -L -o "$NOVEL_DIR/gutenberg/emma.txt" \
    "https://www.gutenberg.org/cache/epub/158/pg158.txt" 2>/dev/null &
# Wuthering Heights
curl -L -o "$NOVEL_DIR/gutenberg/wuthering_heights.txt" \
    "https://www.gutenberg.org/cache/epub/768/pg768.txt" 2>/dev/null &
# The Picture of Dorian Gray
curl -L -o "$NOVEL_DIR/gutenberg/dorian_gray.txt" \
    "https://www.gutenberg.org/cache/epub/174/pg174.txt" 2>/dev/null &
# A Tale of Two Cities
curl -L -o "$NOVEL_DIR/gutenberg/two_cities.txt" \
    "https://www.gutenberg.org/cache/epub/98/pg98.txt" 2>/dev/null &
# Great Expectations
curl -L -o "$NOVEL_DIR/gutenberg/great_expectations.txt" \
    "https://www.gutenberg.org/cache/epub/1400/pg1400.txt" 2>/dev/null &
# The Count of Monte Cristo
curl -L -o "$NOVEL_DIR/gutenberg/monte_cristo.txt" \
    "https://www.gutenberg.org/cache/epub/1184/pg1184.txt" 2>/dev/null &
# Les Misérables (法语原版有对话)
curl -L -o "$NOVEL_DIR/gutenberg/les_miserables.txt" \
    "https://www.gutenberg.org/cache/epub/135/pg135.txt" 2>/dev/null &
# War and Peace
curl -L -o "$NOVEL_DIR/gutenberg/war_peace.txt" \
    "https://www.gutenberg.org/cache/epub/2600/pg2600.txt" 2>/dev/null &
# Anna Karenina
curl -L -o "$NOVEL_DIR/gutenberg/anna_karenina.txt" \
    "https://www.gutenberg.org/cache/epub/1399/pg1399.txt" 2>/dev/null &
# Crime and Punishment
curl -L -o "$NOVEL_DIR/gutenberg/crime_punishment.txt" \
    "https://www.gutenberg.org/cache/epub/2554/pg2554.txt" 2>/dev/null &
# The Brothers Karamazov
curl -L -o "$NOVEL_DIR/gutenberg/karamazov.txt" \
    "https://www.gutenberg.org/cache/epub/28054/pg28054.txt" 2>/dev/null &
# Don Quixote
curl -L -o "$NOVEL_DIR/gutenberg/don_quixote.txt" \
    "https://www.gutenberg.org/cache/epub/996/pg996.txt" 2>/dev/null &
# Moby Dick
curl -L -o "$NOVEL_DIR/gutenberg/moby_dick.txt" \
    "https://www.gutenberg.org/cache/epub/2701/pg2701.txt" 2>/dev/null &

# 中文小说 (从 Chinese Text Project 或其他来源)
echo "  下载中文文学..."
mkdir -p "$NOVEL_DIR/chinese"
# 红楼梦
curl -L -o "$NOVEL_DIR/chinese/hongloumeng.txt" \
    "https://raw.githubusercontent.com/caoyang2002/chinese-novels/main/红楼梦.txt" 2>/dev/null &
# 西游记  
curl -L -o "$NOVEL_DIR/chinese/xiyouji.txt" \
    "https://raw.githubusercontent.com/caoyang2002/chinese-novels/main/西游记.txt" 2>/dev/null &
# 水浒传
curl -L -o "$NOVEL_DIR/chinese/shuihuzhuan.txt" \
    "https://raw.githubusercontent.com/caoyang2002/chinese-novels/main/水浒传.txt" 2>/dev/null &
# 三国演义
curl -L -o "$NOVEL_DIR/chinese/sanguoyanyi.txt" \
    "https://raw.githubusercontent.com/caoyang2002/chinese-novels/main/三国演义.txt" 2>/dev/null &

# 日本文学
echo "  下载日本文学..."
mkdir -p "$NOVEL_DIR/japanese"
# 青空文库的一些作品
curl -L -o "$NOVEL_DIR/japanese/aozora_sample.zip" \
    "https://github.com/aozorabunko/aozorabunko/archive/refs/heads/master.zip" 2>/dev/null &

echo ""
echo "⏳ 等待所有下载完成..."
wait

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 解压文件..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 解压对话数据
cd "$DIALOGUE_DIR"
unzip -o dailydialog.zip 2>/dev/null
unzip -o cornell_movie_dialogs.zip 2>/dev/null
unzip -o multiwoz.zip 2>/dev/null
tar -xzf personachat.tar.gz 2>/dev/null
tar -xzf empatheticdialogues.tar.gz 2>/dev/null
tar -xzf wizard_of_wikipedia.tar.gz 2>/dev/null
tar -xzf ubuntu_dialogs.tar.gz 2>/dev/null

# 解压字幕
cd "$DIALOGUE_DIR/opensubtitles"
for f in *.gz; do
    gunzip -k "$f" 2>/dev/null
done

echo ""
echo "✅ 下载和解压完成！"
echo ""
