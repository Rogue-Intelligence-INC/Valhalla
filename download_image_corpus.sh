#!/bin/bash
# 🖼️ 下载图片描述数据集 (用于多模态训练)

CORPUS="/run/media/jouly/My Book/valhalla_corpus"
mkdir -p "$CORPUS/images"

echo "═══════════════════════════════════════════════════════════════"
echo "  🖼️  下载图片描述数据集 (Captions)"
echo "═══════════════════════════════════════════════════════════════"

cd "$CORPUS/images"

# 1. COCO Captions (MS COCO 2017) - 最大最全的图片描述
echo ""
echo "📥 1/4 下载 COCO Captions annotations..."
mkdir -p coco
cd coco
# COCO annotations (captions)
wget -c "http://images.cocodataset.org/annotations/annotations_trainval2017.zip" -O annotations.zip 2>/dev/null
if [ -f annotations.zip ]; then
    unzip -o annotations.zip 2>/dev/null
    echo "✅ COCO annotations 下载完成"
else
    # 备用: 使用 Hugging Face
    echo "⚠️ 主源失败，尝试 Hugging Face..."
    wget -c "https://huggingface.co/datasets/HuggingFaceM4/COCO/resolve/main/annotations/captions_train2017.json" 2>/dev/null
    wget -c "https://huggingface.co/datasets/HuggingFaceM4/COCO/resolve/main/annotations/captions_val2017.json" 2>/dev/null
fi
cd ..

# 2. Flickr30k Captions
echo ""
echo "📥 2/4 下载 Flickr30k Captions..."
mkdir -p flickr30k
cd flickr30k
# Flickr30k captions from Kaggle alternative
wget -c "https://raw.githubusercontent.com/BryanPlummer/flickr30k_entities/master/annotations.zip" -O flickr30k_annotations.zip 2>/dev/null
if [ -f flickr30k_annotations.zip ]; then
    unzip -o flickr30k_annotations.zip 2>/dev/null
    echo "✅ Flickr30k annotations 下载完成"
fi
# Alternative: Flickr30k sentences
wget -c "https://raw.githubusercontent.com/Li-Wenhui/HughLi/master/flickr30k_sentences.txt" 2>/dev/null
cd ..

# 3. Conceptual Captions (Google) - 更大规模
echo ""
echo "📥 3/4 下载 Conceptual Captions (sample)..."
mkdir -p conceptual_captions
cd conceptual_captions
# 下载验证集 captions (TSV 格式)
wget -c "https://storage.googleapis.com/gcc-data/Validation/GCC-1.1.0-Validation.tsv" 2>/dev/null
if [ -f "GCC-1.1.0-Validation.tsv" ]; then
    echo "✅ Conceptual Captions 下载完成"
fi
cd ..

# 4. TextCaps - 带有 OCR 的图片描述
echo ""
echo "📥 4/4 下载 TextCaps..."
mkdir -p textcaps
cd textcaps
wget -c "https://dl.fbaipublicfiles.com/textvqa/data/textcaps/TextCaps_0.1_train.json" 2>/dev/null
wget -c "https://dl.fbaipublicfiles.com/textvqa/data/textcaps/TextCaps_0.1_val.json" 2>/dev/null
if [ -f "TextCaps_0.1_train.json" ]; then
    echo "✅ TextCaps 下载完成"
fi
cd ..

# 统计下载结果
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  📊 下载统计"
echo "═══════════════════════════════════════════════════════════════"

total_size=$(du -sh "$CORPUS/images" 2>/dev/null | cut -f1)
echo "总大小: $total_size"

# 统计 captions 数量
coco_count=0
if [ -f "coco/annotations/captions_train2017.json" ]; then
    coco_count=$(grep -c '"caption"' coco/annotations/captions_train2017.json 2>/dev/null || echo 0)
fi
echo "COCO Captions: ~$coco_count 条"

flickr_count=0
if [ -f "flickr30k/flickr30k_sentences.txt" ]; then
    flickr_count=$(wc -l < flickr30k/flickr30k_sentences.txt 2>/dev/null || echo 0)
fi
echo "Flickr30k: ~$flickr_count 条"

cc_count=0
if [ -f "conceptual_captions/GCC-1.1.0-Validation.tsv" ]; then
    cc_count=$(wc -l < conceptual_captions/GCC-1.1.0-Validation.tsv 2>/dev/null || echo 0)
fi
echo "Conceptual Captions: ~$cc_count 条"

textcaps_count=0
if [ -f "textcaps/TextCaps_0.1_train.json" ]; then
    textcaps_count=$(grep -c '"caption"' textcaps/TextCaps_0.1_train.json 2>/dev/null || echo 0)
fi
echo "TextCaps: ~$textcaps_count 条"

echo ""
echo "✅ 图片描述数据集下载完成！"
