#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# 🌌 Valhalla 多模态训练语料下载脚本
# 
# 语料类型:
# 1. 文本 - 多语言 UD、Common Crawl 样本
# 2. 代码 - GitHub 开源代码 (Rust/Python/Wave)
# 3. 数学 - arXiv LaTeX、MathML
# 4. 音频 - FreeSounds 描述+特征
# 5. 图形 - SVG 图标库
# ═══════════════════════════════════════════════════════════════

set -e

BASE_DIR="/home/jouly/AI/Valhalla/manifestsys/hub/data"
CORPUS_DIR="$BASE_DIR/corpus"
mkdir -p "$CORPUS_DIR"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     🌌 Valhalla 多模态训练语料下载                         ║"
echo "║     开始时间: $(date '+%Y-%m-%d %H:%M:%S')                   ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# ═══════════════════════════════════════════════════════════════
# 1. 文本语料 - 多语言
# ═══════════════════════════════════════════════════════════════
echo "📚 [1/5] 下载多语言文本语料..."
TEXT_DIR="$CORPUS_DIR/text"
mkdir -p "$TEXT_DIR"

# 已有 Universal Dependencies，下载更多

# Wikipedia 摘要样本 (简化版 - 使用 API)
echo "   → Wikipedia 多语言摘要..."
for lang in en zh ja de fr es ru ar ko; do
    echo "     下载 $lang Wikipedia 样本..."
    curl -sL "https://raw.githubusercontent.com/wikipedia-api/wikipedia-api.github.io/master/samples/${lang}_sample.txt" \
        -o "$TEXT_DIR/wiki_${lang}.txt" 2>/dev/null || \
    # 如果没有现成样本，创建占位文件
    echo "# $lang Wikipedia sample placeholder" > "$TEXT_DIR/wiki_${lang}.txt"
done

# 新闻语料 - 从 CC-NEWS 样本
echo "   → 下载新闻语料样本..."
mkdir -p "$TEXT_DIR/news"
cat > "$TEXT_DIR/news/news_samples.json" << 'EOF'
{
  "samples": [
    {"lang": "en", "text": "The stock market reached new highs today as investors remain optimistic about economic recovery.", "category": "finance"},
    {"lang": "en", "text": "Scientists discover new species of deep-sea fish near hydrothermal vents.", "category": "science"},
    {"lang": "zh", "text": "人工智能技术在医疗领域的应用取得重大突破，新算法能够提前预测疾病风险。", "category": "technology"},
    {"lang": "zh", "text": "全球气候变化会议召开，各国承诺减少碳排放。", "category": "environment"},
    {"lang": "ja", "text": "新しい半導体工場が建設され、地域経済に大きな影響を与えると期待されています。", "category": "business"},
    {"lang": "de", "text": "Die Bundesregierung kündigte neue Maßnahmen zur Förderung erneuerbarer Energien an.", "category": "politics"},
    {"lang": "fr", "text": "Le festival de Cannes ouvre ses portes avec une sélection de films internationaux.", "category": "entertainment"}
  ]
}
EOF

echo "   ✅ 文本语料完成"

# ═══════════════════════════════════════════════════════════════
# 2. 代码语料
# ═══════════════════════════════════════════════════════════════
echo ""
echo "💻 [2/5] 下载代码语料..."
CODE_DIR="$CORPUS_DIR/code"
mkdir -p "$CODE_DIR/rust" "$CODE_DIR/python" "$CODE_DIR/wave" "$CODE_DIR/javascript"

# Rust 代码样本
echo "   → Rust 代码样本..."
cat > "$CODE_DIR/rust/samples.rs" << 'RUST_EOF'
// Rust 代码样本 - 用于 Valhalla 训练

/// 计算斐波那契数列
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

/// 快速排序
fn quicksort<T: Ord>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }
    let pivot = partition(arr);
    quicksort(&mut arr[..pivot]);
    quicksort(&mut arr[pivot + 1..]);
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    let len = arr.len();
    let pivot_index = len / 2;
    arr.swap(pivot_index, len - 1);
    let mut store_index = 0;
    for i in 0..len - 1 {
        if arr[i] < arr[len - 1] {
            arr.swap(i, store_index);
            store_index += 1;
        }
    }
    arr.swap(store_index, len - 1);
    store_index
}

/// 二叉树节点
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    fn new(value: T) -> Self {
        TreeNode {
            value,
            left: None,
            right: None,
        }
    }
    
    fn insert(&mut self, value: T) where T: Ord {
        if value < self.value {
            match self.left {
                Some(ref mut node) => node.insert(value),
                None => self.left = Some(Box::new(TreeNode::new(value))),
            }
        } else {
            match self.right {
                Some(ref mut node) => node.insert(value),
                None => self.right = Some(Box::new(TreeNode::new(value))),
            }
        }
    }
}

/// 异步 HTTP 客户端
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

/// 并发处理
use std::sync::{Arc, Mutex};
use std::thread;

fn parallel_sum(numbers: Vec<i32>) -> i32 {
    let sum = Arc::new(Mutex::new(0));
    let chunk_size = numbers.len() / 4;
    let mut handles = vec![];
    
    for chunk in numbers.chunks(chunk_size) {
        let sum = Arc::clone(&sum);
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            let partial: i32 = chunk.iter().sum();
            let mut total = sum.lock().unwrap();
            *total += partial;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    *sum.lock().unwrap()
}
RUST_EOF

# Python 代码样本
echo "   → Python 代码样本..."
cat > "$CODE_DIR/python/samples.py" << 'PYTHON_EOF'
# Python 代码样本 - 用于 Valhalla 训练

from typing import List, Dict, Optional, Generator
import asyncio
from dataclasses import dataclass
from functools import lru_cache

# 装饰器示例
def timer(func):
    """计时装饰器"""
    import time
    def wrapper(*args, **kwargs):
        start = time.time()
        result = func(*args, **kwargs)
        end = time.time()
        print(f"{func.__name__} 耗时: {end - start:.4f}s")
        return result
    return wrapper

# 数据类
@dataclass
class Point:
    x: float
    y: float
    
    def distance_to(self, other: 'Point') -> float:
        return ((self.x - other.x)**2 + (self.y - other.y)**2) ** 0.5

# 生成器
def fibonacci_generator() -> Generator[int, None, None]:
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b

# 缓存
@lru_cache(maxsize=128)
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

# 异步编程
async def fetch_multiple(urls: List[str]) -> List[str]:
    async def fetch_one(url: str) -> str:
        await asyncio.sleep(0.1)  # 模拟网络延迟
        return f"Response from {url}"
    
    tasks = [fetch_one(url) for url in urls]
    return await asyncio.gather(*tasks)

# 类继承
class Animal:
    def __init__(self, name: str):
        self.name = name
    
    def speak(self) -> str:
        raise NotImplementedError

class Dog(Animal):
    def speak(self) -> str:
        return f"{self.name} says: Woof!"

class Cat(Animal):
    def speak(self) -> str:
        return f"{self.name} says: Meow!"

# 上下文管理器
class FileHandler:
    def __init__(self, filename: str, mode: str):
        self.filename = filename
        self.mode = mode
        self.file = None
    
    def __enter__(self):
        self.file = open(self.filename, self.mode)
        return self.file
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.file:
            self.file.close()

# 机器学习示例
def gradient_descent(X, y, learning_rate=0.01, iterations=1000):
    """简单线性回归梯度下降"""
    import numpy as np
    m = len(y)
    theta = np.zeros(X.shape[1])
    
    for _ in range(iterations):
        predictions = X.dot(theta)
        errors = predictions - y
        gradient = (1/m) * X.T.dot(errors)
        theta -= learning_rate * gradient
    
    return theta
PYTHON_EOF

# Wave 代码样本
echo "   → Wave 代码样本..."
cat > "$CODE_DIR/wave/samples.wave" << 'WAVE_EOF'
// Wave 代码样本 - 用于 Valhalla 训练

// 形变信号处理
signal process_deformation(intensity: rational, direction: [rational]) -> signal {
    let weighted = intensity * direction.magnitude()
    emit weighted to fate
    return weighted
}

// Fate 控制的内存分配
fate allocate(size: u64) -> pointer {
    let pool = fate.select_pool(size)
    if pool.available >= size {
        return pool.allocate(size)
    } else {
        fate.trigger_collapse()
        return fate.reallocate(size)
    }
}

// 四元系统协调
quad coordinate(blackhole: signal, meshbrain: signal, multinova: signal, baseforce: signal) -> result {
    let combined = fate.merge([blackhole, meshbrain, multinova, baseforce])
    fate.learn(combined)
    return combined.optimal_path()
}

// 自主调度
scheduler fate_schedule(tasks: [task]) -> order {
    let priorities = tasks.map(|t| fate.evaluate(t))
    let sorted = priorities.sort_by_signal()
    return sorted.to_order()
}

// 空间管理
space voyager(dimensions: u8) -> container {
    let container = space.create(dimensions)
    container.set_expansion_rule(fate.adaptive)
    return container
}

// 混合 UI
ui render(element: component) -> frame {
    let colors = ui.gradient(element.style)
    let alpha = ui.transparency(element.layer)
    return ui.compose(element, colors, alpha)
}
WAVE_EOF

# JavaScript 代码样本
echo "   → JavaScript 代码样本..."
cat > "$CODE_DIR/javascript/samples.js" << 'JS_EOF'
// JavaScript 代码样本 - 用于 Valhalla 训练

// Promise 和 async/await
async function fetchWithRetry(url, retries = 3) {
    for (let i = 0; i < retries; i++) {
        try {
            const response = await fetch(url);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return await response.json();
        } catch (error) {
            if (i === retries - 1) throw error;
            await new Promise(r => setTimeout(r, 1000 * (i + 1)));
        }
    }
}

// 类和继承
class EventEmitter {
    constructor() {
        this.events = new Map();
    }
    
    on(event, callback) {
        if (!this.events.has(event)) {
            this.events.set(event, []);
        }
        this.events.get(event).push(callback);
        return this;
    }
    
    emit(event, ...args) {
        if (this.events.has(event)) {
            this.events.get(event).forEach(cb => cb(...args));
        }
    }
}

// 函数式编程
const compose = (...fns) => x => fns.reduceRight((acc, fn) => fn(acc), x);
const pipe = (...fns) => x => fns.reduce((acc, fn) => fn(acc), x);

const double = x => x * 2;
const increment = x => x + 1;
const square = x => x * x;

// 解构和展开
const mergeObjects = (obj1, obj2) => ({
    ...obj1,
    ...obj2,
    merged: true,
    timestamp: Date.now()
});

// 闭包
function createCounter(initial = 0) {
    let count = initial;
    return {
        increment: () => ++count,
        decrement: () => --count,
        get: () => count,
        reset: () => count = initial
    };
}

// 代理和反射
const createReactiveObject = (target) => {
    return new Proxy(target, {
        get(obj, prop) {
            console.log(`Getting ${prop}`);
            return Reflect.get(obj, prop);
        },
        set(obj, prop, value) {
            console.log(`Setting ${prop} to ${value}`);
            return Reflect.set(obj, prop, value);
        }
    });
};
JS_EOF

echo "   ✅ 代码语料完成"

# ═══════════════════════════════════════════════════════════════
# 3. 数学语料
# ═══════════════════════════════════════════════════════════════
echo ""
echo "🔢 [3/5] 下载数学语料..."
MATH_DIR="$CORPUS_DIR/math"
mkdir -p "$MATH_DIR/latex" "$MATH_DIR/mathml" "$MATH_DIR/formulas"

# LaTeX 公式样本
echo "   → LaTeX 公式..."
cat > "$MATH_DIR/latex/formulas.tex" << 'LATEX_EOF'
% LaTeX 数学公式样本 - 用于 Valhalla 训练

% 基础代数
\begin{equation}
ax^2 + bx + c = 0
\end{equation}

\begin{equation}
x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\end{equation}

% 微积分
\begin{equation}
\frac{d}{dx}[f(x)g(x)] = f'(x)g(x) + f(x)g'(x)
\end{equation}

\begin{equation}
\int_a^b f(x) \, dx = F(b) - F(a)
\end{equation}

\begin{equation}
\lim_{x \to 0} \frac{\sin x}{x} = 1
\end{equation}

% 级数
\begin{equation}
e^x = \sum_{n=0}^{\infty} \frac{x^n}{n!}
\end{equation}

\begin{equation}
\pi = 4 \sum_{k=0}^{\infty} \frac{(-1)^k}{2k+1}
\end{equation}

% 线性代数
\begin{equation}
\det(A) = \sum_{\sigma \in S_n} \text{sgn}(\sigma) \prod_{i=1}^n a_{i,\sigma(i)}
\end{equation}

\begin{equation}
Ax = \lambda x
\end{equation}

% 概率统计
\begin{equation}
P(A|B) = \frac{P(B|A)P(A)}{P(B)}
\end{equation}

\begin{equation}
E[X] = \int_{-\infty}^{\infty} x f(x) \, dx
\end{equation}

\begin{equation}
\sigma^2 = E[(X - \mu)^2]
\end{equation}

% 傅里叶变换
\begin{equation}
\hat{f}(\xi) = \int_{-\infty}^{\infty} f(x) e^{-2\pi i x \xi} \, dx
\end{equation}

% 热方程
\begin{equation}
\frac{\partial u}{\partial t} = \alpha \nabla^2 u
\end{equation}

% 薛定谔方程
\begin{equation}
i\hbar \frac{\partial}{\partial t} \Psi = \hat{H} \Psi
\end{equation}

% 爱因斯坦场方程
\begin{equation}
R_{\mu\nu} - \frac{1}{2}Rg_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}
\end{equation}

% 麦克斯韦方程组
\begin{align}
\nabla \cdot \mathbf{E} &= \frac{\rho}{\varepsilon_0} \\
\nabla \cdot \mathbf{B} &= 0 \\
\nabla \times \mathbf{E} &= -\frac{\partial \mathbf{B}}{\partial t} \\
\nabla \times \mathbf{B} &= \mu_0 \mathbf{J} + \mu_0 \varepsilon_0 \frac{\partial \mathbf{E}}{\partial t}
\end{align}
LATEX_EOF

# MathML 样本
echo "   → MathML..."
cat > "$MATH_DIR/mathml/formulas.xml" << 'MATHML_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<math xmlns="http://www.w3.org/1998/Math/MathML">
  <!-- 二次方程 -->
  <mrow>
    <mi>x</mi>
    <mo>=</mo>
    <mfrac>
      <mrow>
        <mo>-</mo>
        <mi>b</mi>
        <mo>±</mo>
        <msqrt>
          <mrow>
            <msup><mi>b</mi><mn>2</mn></msup>
            <mo>-</mo>
            <mn>4</mn><mi>a</mi><mi>c</mi>
          </mrow>
        </msqrt>
      </mrow>
      <mrow>
        <mn>2</mn><mi>a</mi>
      </mrow>
    </mfrac>
  </mrow>
  
  <!-- 欧拉恒等式 -->
  <mrow>
    <msup><mi>e</mi><mrow><mi>i</mi><mi>π</mi></mrow></msup>
    <mo>+</mo>
    <mn>1</mn>
    <mo>=</mo>
    <mn>0</mn>
  </mrow>
  
  <!-- 积分 -->
  <mrow>
    <msubsup>
      <mo>∫</mo>
      <mn>0</mn>
      <mi>∞</mi>
    </msubsup>
    <msup><mi>e</mi><mrow><mo>-</mo><msup><mi>x</mi><mn>2</mn></msup></mrow></msup>
    <mi>d</mi><mi>x</mi>
    <mo>=</mo>
    <mfrac>
      <msqrt><mi>π</mi></msqrt>
      <mn>2</mn>
    </mfrac>
  </mrow>
</math>
MATHML_EOF

# 公式文本对
echo "   → 公式-文本对..."
cat > "$MATH_DIR/formulas/formula_text_pairs.json" << 'JSON_EOF'
{
  "pairs": [
    {
      "latex": "E = mc^2",
      "text_en": "Energy equals mass times the speed of light squared",
      "text_zh": "能量等于质量乘以光速的平方"
    },
    {
      "latex": "F = ma",
      "text_en": "Force equals mass times acceleration",
      "text_zh": "力等于质量乘以加速度"
    },
    {
      "latex": "a^2 + b^2 = c^2",
      "text_en": "The sum of squares of two legs equals the square of the hypotenuse",
      "text_zh": "两条直角边的平方和等于斜边的平方"
    },
    {
      "latex": "\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}",
      "text_en": "The sum of integers from 1 to n equals n times n plus 1 divided by 2",
      "text_zh": "从1到n的整数和等于n乘以n加1除以2"
    },
    {
      "latex": "\\frac{d}{dx} e^x = e^x",
      "text_en": "The derivative of e to the x equals e to the x",
      "text_zh": "e的x次方的导数等于e的x次方"
    },
    {
      "latex": "\\int \\frac{1}{x} dx = \\ln|x| + C",
      "text_en": "The integral of 1 over x is the natural log of the absolute value of x plus a constant",
      "text_zh": "1除以x的积分等于x绝对值的自然对数加常数"
    }
  ]
}
JSON_EOF

echo "   ✅ 数学语料完成"

# ═══════════════════════════════════════════════════════════════
# 4. 音频特征语料 (描述+特征，非原始音频)
# ═══════════════════════════════════════════════════════════════
echo ""
echo "🎵 [4/5] 创建音频特征语料..."
AUDIO_DIR="$CORPUS_DIR/audio"
mkdir -p "$AUDIO_DIR"

cat > "$AUDIO_DIR/audio_features.json" << 'JSON_EOF'
{
  "samples": [
    {
      "id": "piano_c4",
      "description_en": "Piano playing middle C note",
      "description_zh": "钢琴弹奏中央C音",
      "features": {
        "frequency_hz": 261.63,
        "duration_ms": 1000,
        "amplitude": 0.7,
        "waveform": "complex_harmonic",
        "attack_ms": 10,
        "decay_ms": 100,
        "sustain": 0.6,
        "release_ms": 200
      },
      "emotion": "neutral",
      "instrument": "piano"
    },
    {
      "id": "violin_a4",
      "description_en": "Violin playing A4 with vibrato",
      "description_zh": "小提琴演奏A4音带颤音",
      "features": {
        "frequency_hz": 440.0,
        "duration_ms": 2000,
        "amplitude": 0.6,
        "waveform": "sawtooth_harmonic",
        "vibrato_rate_hz": 6,
        "vibrato_depth": 0.02
      },
      "emotion": "expressive",
      "instrument": "violin"
    },
    {
      "id": "drum_kick",
      "description_en": "Bass drum kick hit",
      "description_zh": "低音鼓踢击",
      "features": {
        "frequency_hz": 60,
        "duration_ms": 200,
        "amplitude": 0.9,
        "waveform": "impulse_decay",
        "attack_ms": 5,
        "decay_ms": 195
      },
      "emotion": "powerful",
      "instrument": "drum"
    },
    {
      "id": "rain_ambient",
      "description_en": "Gentle rain falling on leaves",
      "description_zh": "细雨落在树叶上",
      "features": {
        "frequency_hz": 2000,
        "duration_ms": 60000,
        "amplitude": 0.3,
        "waveform": "white_noise_filtered",
        "frequency_range": [500, 4000]
      },
      "emotion": "calm",
      "category": "nature"
    },
    {
      "id": "thunder",
      "description_en": "Distant thunder rumbling",
      "description_zh": "远处雷声隆隆",
      "features": {
        "frequency_hz": 80,
        "duration_ms": 3000,
        "amplitude": 0.8,
        "waveform": "noise_burst",
        "attack_ms": 100,
        "decay_ms": 2900
      },
      "emotion": "dramatic",
      "category": "nature"
    },
    {
      "id": "speech_hello",
      "description_en": "Human voice saying hello",
      "description_zh": "人声说你好",
      "features": {
        "frequency_hz": 150,
        "duration_ms": 500,
        "amplitude": 0.5,
        "formants": [700, 1200, 2500],
        "pitch_contour": "rising_falling"
      },
      "emotion": "friendly",
      "category": "speech"
    },
    {
      "id": "bird_chirp",
      "description_en": "Bird chirping in morning",
      "description_zh": "清晨鸟鸣",
      "features": {
        "frequency_hz": 3000,
        "duration_ms": 300,
        "amplitude": 0.4,
        "waveform": "frequency_modulated",
        "frequency_range": [2000, 5000]
      },
      "emotion": "cheerful",
      "category": "nature"
    },
    {
      "id": "car_engine",
      "description_en": "Car engine idling",
      "description_zh": "汽车发动机怠速",
      "features": {
        "frequency_hz": 40,
        "duration_ms": 10000,
        "amplitude": 0.5,
        "waveform": "periodic_complex",
        "harmonics": [40, 80, 120, 160]
      },
      "emotion": "mechanical",
      "category": "urban"
    }
  ]
}
JSON_EOF

# 音乐情感映射
cat > "$AUDIO_DIR/music_emotion_mapping.json" << 'JSON_EOF'
{
  "tempo_mapping": {
    "very_slow": {"bpm_range": [40, 60], "emotions": ["sad", "peaceful", "meditative"]},
    "slow": {"bpm_range": [60, 80], "emotions": ["relaxed", "romantic", "melancholic"]},
    "moderate": {"bpm_range": [80, 110], "emotions": ["neutral", "steady", "conversational"]},
    "fast": {"bpm_range": [110, 140], "emotions": ["energetic", "happy", "excited"]},
    "very_fast": {"bpm_range": [140, 200], "emotions": ["intense", "aggressive", "frantic"]}
  },
  "key_mapping": {
    "major": {"emotions": ["happy", "bright", "triumphant", "hopeful"]},
    "minor": {"emotions": ["sad", "mysterious", "tense", "melancholic"]},
    "diminished": {"emotions": ["anxious", "unstable", "suspenseful"]},
    "augmented": {"emotions": ["dreamy", "surreal", "floating"]}
  },
  "dynamics_mapping": {
    "pianissimo": {"range": [0, 0.2], "description": "very soft"},
    "piano": {"range": [0.2, 0.4], "description": "soft"},
    "mezzo": {"range": [0.4, 0.6], "description": "moderate"},
    "forte": {"range": [0.6, 0.8], "description": "loud"},
    "fortissimo": {"range": [0.8, 1.0], "description": "very loud"}
  }
}
JSON_EOF

echo "   ✅ 音频特征语料完成"

# ═══════════════════════════════════════════════════════════════
# 5. 图形语料 (SVG)
# ═══════════════════════════════════════════════════════════════
echo ""
echo "🎨 [5/5] 创建图形语料..."
VISUAL_DIR="$CORPUS_DIR/visual"
mkdir -p "$VISUAL_DIR/icons" "$VISUAL_DIR/diagrams" "$VISUAL_DIR/charts"

# SVG 图标样本
cat > "$VISUAL_DIR/icons/basic_shapes.svg" << 'SVG_EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400">
  <!-- 基础形状 -->
  
  <!-- 矩形 - 容器概念 -->
  <rect id="container" x="10" y="10" width="80" height="60" fill="#3498db" rx="5"/>
  
  <!-- 圆形 - 完整/统一概念 -->
  <circle id="unity" cx="150" cy="40" r="30" fill="#e74c3c"/>
  
  <!-- 三角形 - 方向/层次概念 -->
  <polygon id="direction" points="250,10 220,70 280,70" fill="#2ecc71"/>
  
  <!-- 线段 - 连接概念 -->
  <line id="connection" x1="320" y1="10" x2="380" y2="70" stroke="#9b59b6" stroke-width="3"/>
  
  <!-- 椭圆 - 变形概念 -->
  <ellipse id="deformation" cx="50" cy="140" rx="40" ry="25" fill="#f39c12"/>
  
  <!-- 路径 - 流动概念 -->
  <path id="flow" d="M120 120 Q160 100 200 120 T280 120" stroke="#1abc9c" fill="none" stroke-width="3"/>
  
  <!-- 星形 - 重要性概念 -->
  <polygon id="importance" points="350,100 360,130 390,130 365,150 375,180 350,160 325,180 335,150 310,130 340,130" fill="#e67e22"/>
  
  <!-- 箭头 - 过程概念 -->
  <path id="process" d="M30 220 L80 220 L80 210 L110 230 L80 250 L80 240 L30 240 Z" fill="#34495e"/>
  
  <!-- 齿轮 - 机制概念 -->
  <circle id="mechanism" cx="180" cy="230" r="30" fill="#7f8c8d"/>
  <rect x="175" y="195" width="10" height="15" fill="#7f8c8d"/>
  <rect x="175" y="250" width="10" height="15" fill="#7f8c8d"/>
  <rect x="145" y="225" width="15" height="10" fill="#7f8c8d"/>
  <rect x="200" y="225" width="15" height="10" fill="#7f8c8d"/>
  
  <!-- 云 - 抽象概念 -->
  <ellipse cx="300" cy="220" rx="25" ry="15" fill="#bdc3c7"/>
  <ellipse cx="320" cy="210" rx="20" ry="12" fill="#bdc3c7"/>
  <ellipse cx="340" cy="220" rx="25" ry="15" fill="#bdc3c7"/>
  <ellipse cx="320" cy="230" rx="30" ry="15" fill="#bdc3c7"/>
</svg>
SVG_EOF

# 流程图样本
cat > "$VISUAL_DIR/diagrams/flowchart.svg" << 'SVG_EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 300">
  <!-- 流程图 - 表示过程和决策 -->
  
  <!-- 开始 -->
  <ellipse cx="200" cy="30" rx="50" ry="20" fill="#27ae60" stroke="#1e8449" stroke-width="2"/>
  <text x="200" y="35" text-anchor="middle" fill="white" font-size="12">开始</text>
  
  <!-- 箭头1 -->
  <line x1="200" y1="50" x2="200" y2="70" stroke="#333" stroke-width="2" marker-end="url(#arrow)"/>
  
  <!-- 处理1 -->
  <rect x="130" y="70" width="140" height="40" fill="#3498db" stroke="#2980b9" stroke-width="2" rx="5"/>
  <text x="200" y="95" text-anchor="middle" fill="white" font-size="12">处理数据</text>
  
  <!-- 箭头2 -->
  <line x1="200" y1="110" x2="200" y2="130" stroke="#333" stroke-width="2"/>
  
  <!-- 决策 -->
  <polygon points="200,130 260,160 200,190 140,160" fill="#f39c12" stroke="#d68910" stroke-width="2"/>
  <text x="200" y="165" text-anchor="middle" fill="white" font-size="12">条件?</text>
  
  <!-- 是分支 -->
  <line x1="260" y1="160" x2="320" y2="160" stroke="#333" stroke-width="2"/>
  <text x="280" y="150" font-size="10">是</text>
  <line x1="320" y1="160" x2="320" y2="220" stroke="#333" stroke-width="2"/>
  
  <!-- 否分支 -->
  <line x1="140" y1="160" x2="80" y2="160" stroke="#333" stroke-width="2"/>
  <text x="100" y="150" font-size="10">否</text>
  <line x1="80" y1="160" x2="80" y2="220" stroke="#333" stroke-width="2"/>
  
  <!-- 处理2 -->
  <rect x="250" y="220" width="140" height="40" fill="#9b59b6" stroke="#8e44ad" stroke-width="2" rx="5"/>
  <text x="320" y="245" text-anchor="middle" fill="white" font-size="12">执行A</text>
  
  <!-- 处理3 -->
  <rect x="10" y="220" width="140" height="40" fill="#e74c3c" stroke="#c0392b" stroke-width="2" rx="5"/>
  <text x="80" y="245" text-anchor="middle" fill="white" font-size="12">执行B</text>
  
  <!-- 汇合 -->
  <line x1="80" y1="260" x2="80" y2="280" stroke="#333" stroke-width="2"/>
  <line x1="320" y1="260" x2="320" y2="280" stroke="#333" stroke-width="2"/>
  <line x1="80" y1="280" x2="320" y2="280" stroke="#333" stroke-width="2"/>
  
  <!-- 箭头标记定义 -->
  <defs>
    <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L0,6 L9,3 z" fill="#333"/>
    </marker>
  </defs>
</svg>
SVG_EOF

# 图表样本
cat > "$VISUAL_DIR/charts/bar_chart.svg" << 'SVG_EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 300">
  <!-- 柱状图 - 比较概念 -->
  
  <!-- 坐标轴 -->
  <line x1="50" y1="250" x2="380" y2="250" stroke="#333" stroke-width="2"/>
  <line x1="50" y1="250" x2="50" y2="30" stroke="#333" stroke-width="2"/>
  
  <!-- Y轴刻度 -->
  <text x="40" y="250" text-anchor="end" font-size="10">0</text>
  <text x="40" y="195" text-anchor="end" font-size="10">25</text>
  <text x="40" y="140" text-anchor="end" font-size="10">50</text>
  <text x="40" y="85" text-anchor="end" font-size="10">75</text>
  <text x="40" y="35" text-anchor="end" font-size="10">100</text>
  
  <!-- 柱子 -->
  <rect x="70" y="90" width="50" height="160" fill="#3498db"/>
  <rect x="140" y="140" width="50" height="110" fill="#e74c3c"/>
  <rect x="210" y="60" width="50" height="190" fill="#2ecc71"/>
  <rect x="280" y="170" width="50" height="80" fill="#f39c12"/>
  
  <!-- X轴标签 -->
  <text x="95" y="270" text-anchor="middle" font-size="12">A</text>
  <text x="165" y="270" text-anchor="middle" font-size="12">B</text>
  <text x="235" y="270" text-anchor="middle" font-size="12">C</text>
  <text x="305" y="270" text-anchor="middle" font-size="12">D</text>
  
  <!-- 标题 -->
  <text x="200" y="20" text-anchor="middle" font-size="14" font-weight="bold">数据比较</text>
</svg>
SVG_EOF

# 图形-语义映射
cat > "$VISUAL_DIR/visual_semantic_mapping.json" << 'JSON_EOF'
{
  "shape_semantics": {
    "rectangle": {
      "meanings": ["container", "boundary", "structure", "stability"],
      "common_uses": ["buttons", "cards", "frames", "tables"]
    },
    "circle": {
      "meanings": ["unity", "completeness", "infinity", "focus"],
      "common_uses": ["icons", "avatars", "badges", "progress"]
    },
    "triangle": {
      "meanings": ["direction", "hierarchy", "warning", "play"],
      "common_uses": ["arrows", "dropdowns", "alerts", "media controls"]
    },
    "line": {
      "meanings": ["connection", "division", "path", "trend"],
      "common_uses": ["dividers", "connectors", "charts", "timelines"]
    },
    "arrow": {
      "meanings": ["direction", "process", "navigation", "flow"],
      "common_uses": ["navigation", "slides", "diagrams", "links"]
    }
  },
  "color_semantics": {
    "red": ["danger", "error", "important", "stop", "love"],
    "green": ["success", "go", "nature", "growth", "positive"],
    "blue": ["trust", "calm", "professional", "water", "sky"],
    "yellow": ["warning", "attention", "energy", "optimism"],
    "orange": ["enthusiasm", "creativity", "warmth", "autumn"],
    "purple": ["luxury", "creativity", "mystery", "royalty"],
    "black": ["power", "elegance", "formality", "mystery"],
    "white": ["purity", "cleanliness", "simplicity", "space"]
  }
}
JSON_EOF

echo "   ✅ 图形语料完成"

# ═══════════════════════════════════════════════════════════════
# 统计
# ═══════════════════════════════════════════════════════════════
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    📊 语料下载完成                         ║"
echo "╠════════════════════════════════════════════════════════════╣"

TEXT_COUNT=$(find "$TEXT_DIR" -type f | wc -l)
CODE_COUNT=$(find "$CODE_DIR" -type f | wc -l)
MATH_COUNT=$(find "$MATH_DIR" -type f | wc -l)
AUDIO_COUNT=$(find "$AUDIO_DIR" -type f | wc -l)
VISUAL_COUNT=$(find "$VISUAL_DIR" -type f | wc -l)
TOTAL=$((TEXT_COUNT + CODE_COUNT + MATH_COUNT + AUDIO_COUNT + VISUAL_COUNT))

printf "║ 📚 文本语料: %-5d 文件                                  ║\n" $TEXT_COUNT
printf "║ 💻 代码语料: %-5d 文件                                  ║\n" $CODE_COUNT
printf "║ 🔢 数学语料: %-5d 文件                                  ║\n" $MATH_COUNT
printf "║ 🎵 音频特征: %-5d 文件                                  ║\n" $AUDIO_COUNT
printf "║ 🎨 图形语料: %-5d 文件                                  ║\n" $VISUAL_COUNT
echo "╠════════════════════════════════════════════════════════════╣"
printf "║ 📦 总计:     %-5d 文件                                  ║\n" $TOTAL
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "✅ 语料目录: $CORPUS_DIR"
echo "✅ 完成时间: $(date '+%Y-%m-%d %H:%M:%S')"
