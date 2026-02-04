#!/usr/bin/env python3
"""
向量余弦相似度搜索 - Python版
从10万向量库中找最相似的向量
"""
import time
import math
import random

# 配置
VECTOR_DIM = 128      # 向量维度
VECTOR_COUNT = 100000 # 向量库大小

def dot_product(a, b):
    """点积"""
    return sum(x * y for x, y in zip(a, b))

def magnitude(v):
    """向量模长"""
    return math.sqrt(sum(x * x for x in v))

def cosine_similarity(a, b):
    """余弦相似度"""
    dot = dot_product(a, b)
    mag_a = magnitude(a)
    mag_b = magnitude(b)
    if mag_a == 0 or mag_b == 0:
        return 0.0
    return dot / (mag_a * mag_b)

def generate_random_vector(dim, seed=None):
    """生成随机向量"""
    if seed is not None:
        random.seed(seed)
    return [random.random() * 2 - 1 for _ in range(dim)]

def main():
    print("=" * 50)
    print("向量余弦相似度搜索 - Python版")
    print("=" * 50)
    print(f"向量维度: {VECTOR_DIM}")
    print(f"向量库大小: {VECTOR_COUNT:,}")
    print()
    
    # 生成向量库
    print("生成向量库...")
    t0 = time.time()
    random.seed(42)  # 固定种子以便对比
    vector_db = [generate_random_vector(VECTOR_DIM) for _ in range(VECTOR_COUNT)]
    t1 = time.time()
    print(f"生成完成: {t1-t0:.3f}秒")
    print()
    
    # 生成查询向量
    query = generate_random_vector(VECTOR_DIM, seed=12345)
    print(f"查询向量前5个元素: {query[:5]}")
    print()
    
    # 搜索最相似向量
    print("搜索中...")
    t2 = time.time()
    
    best_idx = -1
    best_sim = -2.0
    
    for i, vec in enumerate(vector_db):
        sim = cosine_similarity(query, vec)
        if sim > best_sim:
            best_sim = sim
            best_idx = i
    
    t3 = time.time()
    search_time = t3 - t2
    
    # 结果
    print()
    print("=" * 50)
    print("搜索结果")
    print("=" * 50)
    print(f"最相似向量索引: {best_idx}")
    print(f"余弦相似度: {best_sim:.6f}")
    print(f"搜索耗时: {search_time:.3f}秒")
    print(f"搜索速度: {VECTOR_COUNT/search_time:,.0f} 向量/秒")
    print()
    
    # 验证
    print("验证结果向量前5个元素:")
    print(f"  {vector_db[best_idx][:5]}")

if __name__ == "__main__":
    main()
