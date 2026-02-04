#!/bin/bash
# 下载多语言训练语料：词汇表、语法、短语、造句

CORPUS_BASE="/run/media/jouly/My Book/valhalla_corpus/languages"
cd "$CORPUS_BASE"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     📚 下载多语言训练语料                                      ║"
echo "║     词汇 | 语法 | 短语 | 句子                                  ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# ========== 通用数据源 ==========

# Tatoeba - 多语言句子库 (短语+造句)
echo -e "\n📖 下载 Tatoeba 多语言句子库..."
mkdir -p tatoeba && cd tatoeba
curl -L -o sentences.tar.bz2 "https://downloads.tatoeba.org/exports/sentences.tar.bz2" &
curl -L -o links.tar.bz2 "https://downloads.tatoeba.org/exports/links.tar.bz2" &
curl -L -o sentences_detailed.tar.bz2 "https://downloads.tatoeba.org/exports/sentences_detailed.tar.bz2" &
cd ..

# ========== 中文 ==========
echo -e "\n🇨🇳 下载中文语料..."
cd chinese

# 词汇表
echo "  词汇表..."
curl -L -o vocabulary/cedict.txt "https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.txt.gz" &
curl -L -o vocabulary/hsk_words.txt "https://raw.githubusercontent.com/infinyte7/Anki-xiehanzi/master/HSK%20List/HSK%20Vocabulary.txt" 2>/dev/null &

# 语法 - Universal Dependencies
echo "  语法..."
curl -L -o grammar/ud_chinese.tgz "https://lindat.mff.cuni.cz/repository/xmlui/bitstream/handle/11234/1-5287/ud-treebanks-v2.13.tgz" 2>/dev/null &

# 短语+句子
echo "  句子..."
curl -L -o sentences/chinese_sentences.txt "https://downloads.tatoeba.org/exports/per_language/cmn/cmn_sentences.tsv.bz2" &

cd ..

# ========== 英文 ==========
echo -e "\n🇬🇧 下载英文语料..."
cd english

# 词汇表
echo "  词汇表..."
curl -L -o vocabulary/english_words.txt "https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt" &
curl -L -o vocabulary/frequency_list.txt "https://raw.githubusercontent.com/first20hours/google-10000-english/master/google-10000-english.txt" &
curl -L -o vocabulary/common_words.txt "https://raw.githubusercontent.com/dolph/dictionary/master/popular.txt" &

# 语法
echo "  语法..."
curl -L -o grammar/english_grammar.json "https://raw.githubusercontent.com/wikipedia2vec/wikipedia2vec/master/scripts/extract_grammar.py" 2>/dev/null &

# 短语
echo "  短语..."
curl -L -o phrases/idioms.txt "https://raw.githubusercontent.com/tokuhirom/idiomjlpt/master/idiom.txt" 2>/dev/null &

# 句子
echo "  句子..."
curl -L -o sentences/english_sentences.txt "https://downloads.tatoeba.org/exports/per_language/eng/eng_sentences.tsv.bz2" &

cd ..

# ========== 日文 ==========
echo -e "\n🇯🇵 下载日文语料..."
cd japanese

# 词汇表
echo "  词汇表..."
curl -L -o vocabulary/jmdict.gz "http://ftp.edrdg.org/pub/Nihongo/JMdict_e.gz" &
curl -L -o vocabulary/kanjidic.gz "http://ftp.edrdg.org/pub/Nihongo/kanjidic2.xml.gz" &

# 句子
echo "  句子..."
curl -L -o sentences/japanese_sentences.txt "https://downloads.tatoeba.org/exports/per_language/jpn/jpn_sentences.tsv.bz2" &

cd ..

# ========== 韩文 ==========
echo -e "\n🇰🇷 下载韩文语料..."
cd korean

# 词汇表
curl -L -o vocabulary/korean_words.txt "https://raw.githubusercontent.com/wikipedia2vec/wikipedia2vec/master/scripts/README.md" 2>/dev/null &

# 句子
curl -L -o sentences/korean_sentences.txt "https://downloads.tatoeba.org/exports/per_language/kor/kor_sentences.tsv.bz2" &

cd ..

# ========== 法文 ==========
echo -e "\n🇫🇷 下载法文语料..."
cd french
curl -L -o vocabulary/french_words.txt "https://raw.githubusercontent.com/words/french-words/master/words.txt" &
curl -L -o sentences/french_sentences.txt "https://downloads.tatoeba.org/exports/per_language/fra/fra_sentences.tsv.bz2" &
cd ..

# ========== 德文 ==========
echo -e "\n🇩🇪 下载德文语料..."
cd german
curl -L -o vocabulary/german_words.txt "https://raw.githubusercontent.com/enz/german-wordlist/master/words" &
curl -L -o sentences/german_sentences.txt "https://downloads.tatoeba.org/exports/per_language/deu/deu_sentences.tsv.bz2" &
cd ..

# ========== 西班牙文 ==========
echo -e "\n🇪🇸 下载西班牙文语料..."
cd spanish
curl -L -o vocabulary/spanish_words.txt "https://raw.githubusercontent.com/words/spanish-words/master/words.txt" &
curl -L -o sentences/spanish_sentences.txt "https://downloads.tatoeba.org/exports/per_language/spa/spa_sentences.tsv.bz2" &
cd ..

# ========== 俄文 ==========
echo -e "\n🇷🇺 下载俄文语料..."
cd russian
curl -L -o sentences/russian_sentences.txt "https://downloads.tatoeba.org/exports/per_language/rus/rus_sentences.tsv.bz2" &
cd ..

# ========== 阿拉伯文 ==========
echo -e "\n🇸🇦 下载阿拉伯文语料..."
cd arabic
curl -L -o sentences/arabic_sentences.txt "https://downloads.tatoeba.org/exports/per_language/ara/ara_sentences.tsv.bz2" &
cd ..

# ========== 葡萄牙文 ==========
echo -e "\n🇧🇷 下载葡萄牙文语料..."
cd portuguese
curl -L -o sentences/portuguese_sentences.txt "https://downloads.tatoeba.org/exports/per_language/por/por_sentences.tsv.bz2" &
cd ..

# ========== 等待所有下载 ==========
echo -e "\n⏳ 等待所有下载完成..."
wait

echo -e "\n✅ 下载完成！"
echo "=== 下载统计 ==="
du -sh "$CORPUS_BASE"/*
