#!/bin/bash

# 🌌 Valhalla 扩展训练语料下载
# 目标: 增加代码、数学、对话等模态数据

CORPUS_ROOT="/run/media/jouly/My Book/valhalla_corpus"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  📦 下载扩展训练语料                                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# === 1. 更多代码数据 ===
echo "💻 下载代码语料..."
mkdir -p "$CORPUS_ROOT/code/github"

# Rosetta Code (多语言编程示例)
echo "  下载 Rosetta Code 示例..."
curl -L -o "$CORPUS_ROOT/code/rosetta_code.json" \
  "https://huggingface.co/datasets/cakiki/rosetta-code/resolve/main/data/train-00000-of-00001.parquet" 2>/dev/null &

# CodeSearchNet (函数和文档)
echo "  下载 CodeSearchNet Python..."
curl -L -o "$CORPUS_ROOT/code/codesearchnet_python.jsonl.gz" \
  "https://huggingface.co/datasets/code_search_net/resolve/main/data/python/train-00000-of-00001.parquet" 2>/dev/null &

# LeetCode 风格问题
echo "  下载编程问题..."
curl -L -o "$CORPUS_ROOT/code/leetcode_solutions.json" \
  "https://huggingface.co/datasets/greengerong/leetcode/resolve/main/leetcode-train.json" 2>/dev/null &

# === 2. 更多数学数据 ===
echo ""
echo "🔢 下载数学语料..."
mkdir -p "$CORPUS_ROOT/math/formulas"

# MATH 数据集
echo "  下载 MATH 数据集..."
curl -L -o "$CORPUS_ROOT/math/math_train.json" \
  "https://huggingface.co/datasets/competition_math/resolve/main/data/train-00000-of-00001.parquet" 2>/dev/null &

# GSM8K (数学应用题)
echo "  下载 GSM8K..."
curl -L -o "$CORPUS_ROOT/math/gsm8k_train.jsonl" \
  "https://huggingface.co/datasets/gsm8k/resolve/main/data/train-00000-of-00001.parquet" 2>/dev/null &

# 数学公式集
echo "  生成数学公式样本..."
cat > "$CORPUS_ROOT/math/formulas/basic_formulas.txt" << 'MATH'
E = mc^2
F = ma
a^2 + b^2 = c^2
e^{i\pi} + 1 = 0
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}
\lim_{x \to 0} \frac{\sin x}{x} = 1
\frac{d}{dx}[f(g(x))] = f'(g(x)) \cdot g'(x)
\nabla \times \mathbf{E} = -\frac{\partial \mathbf{B}}{\partial t}
\nabla \cdot \mathbf{E} = \frac{\rho}{\epsilon_0}
PV = nRT
\Delta G = \Delta H - T\Delta S
\hat{H}\Psi = E\Psi
i\hbar\frac{\partial}{\partial t}\Psi = \hat{H}\Psi
\oint \mathbf{B} \cdot d\mathbf{l} = \mu_0 I
S = k_B \ln \Omega
F = G\frac{m_1 m_2}{r^2}
\lambda = \frac{h}{p}
\Delta x \Delta p \geq \frac{\hbar}{2}
c = \frac{1}{\sqrt{\mu_0 \epsilon_0}}
MATH

# === 3. 更多对话数据 ===
echo ""
echo "💬 下载对话语料..."
mkdir -p "$CORPUS_ROOT/content/dialogues/extra"

# DailyDialog
echo "  下载 DailyDialog..."
curl -L -o "$CORPUS_ROOT/content/dialogues/extra/dailydialog.json" \
  "https://huggingface.co/datasets/daily_dialog/resolve/main/data/train-00000-of-00001.parquet" 2>/dev/null &

# PersonaChat
echo "  下载 PersonaChat..."
curl -L -o "$CORPUS_ROOT/content/dialogues/extra/personachat.json" \
  "https://huggingface.co/datasets/bavard/personachat_truecased/resolve/main/data/train-00000-of-00001.parquet" 2>/dev/null &

# === 4. 更多叙事/故事 ===
echo ""
echo "📖 下载叙事语料..."
mkdir -p "$CORPUS_ROOT/content/stories"

# WritingPrompts
echo "  下载 WritingPrompts..."
curl -L -o "$CORPUS_ROOT/content/stories/writing_prompts.json" \
  "https://huggingface.co/datasets/euclaise/writingprompts/resolve/main/data/train-00000-of-00020.parquet" 2>/dev/null &

# === 5. 从 Valhalla 项目提取更多代码 ===
echo ""
echo "🔧 从 Valhalla 项目提取代码..."

# 提取所有 Rust 代码
find /home/jouly/AI/Valhalla -name "*.rs" -type f 2>/dev/null | while read f; do
    cat "$f" >> "$CORPUS_ROOT/code/valhalla_rust.txt"
    echo -e "\n---FILE_SEPARATOR---\n" >> "$CORPUS_ROOT/code/valhalla_rust.txt"
done
echo "  提取了 Rust 代码"

# 提取 Wave 代码
find /home/jouly/AI/Valhalla -name "*.wave" -type f 2>/dev/null | while read f; do
    cat "$f" >> "$CORPUS_ROOT/code/valhalla_wave.txt"
    echo -e "\n---FILE_SEPARATOR---\n" >> "$CORPUS_ROOT/code/valhalla_wave.txt"
done
echo "  提取了 Wave 代码"

# 提取 Cosmic 代码
find /home/jouly/AI/Valhalla -name "*.cosmic" -type f 2>/dev/null | while read f; do
    cat "$f" >> "$CORPUS_ROOT/code/valhalla_cosmic.txt"
    echo -e "\n---FILE_SEPARATOR---\n" >> "$CORPUS_ROOT/code/valhalla_cosmic.txt"
done
echo "  提取了 Cosmic 代码"

# 等待下载完成
echo ""
echo "⏳ 等待下载完成..."
wait

echo ""
echo "✅ 下载完成！"
echo ""
echo "📊 新增数据统计:"
ls -lh "$CORPUS_ROOT/code/"*.txt 2>/dev/null
ls -lh "$CORPUS_ROOT/math/formulas/" 2>/dev/null
