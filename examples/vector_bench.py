#!/usr/bin/env python3
"""
向量相似度搜索基准测试 - Python版
与 Wave 版本使用相同参数进行对比
"""
import time

# 配置 (与Wave版相同)
DIM = 16
COUNT = 100
RUNS = 1000  # 重复运行次数以测量性能

# 伪随机数生成器 (与Wave相同的LCG)
class LCG:
    def __init__(self, seed):
        self.seed = seed
    
    def rand(self):
        self.seed = (self.seed * 1103515245 + 12345) & 0x7FFFFFFF
        return self.seed % 2001 - 1000

def dot_product(a, b):
    """点积"""
    return sum(x * y for x, y in zip(a, b))

def main():
    print("=" * 50)
    print("向量相似度搜索基准测试 - Python版")
    print("=" * 50)
    print(f"维度: {DIM}, 向量数: {COUNT}, 重复次数: {RUNS}")
    print()
    
    # 初始化向量库 (与Wave相同的随机种子)
    rng = LCG(42)
    db = [[rng.rand() for _ in range(DIM)] for _ in range(COUNT)]
    
    # 初始化查询向量
    rng = LCG(12345)
    query = [rng.rand() for _ in range(DIM)]
    
    print(f"查询向量前5个: {query[:5]}")
    print()
    
    # 基准测试
    print("开始基准测试...")
    t0 = time.perf_counter()
    
    for _ in range(RUNS):
        best_idx = 0
        best_score = -999999999
        
        for i, vec in enumerate(db):
            score = dot_product(query, vec)
            if score > best_score:
                best_score = score
                best_idx = i
    
    t1 = time.perf_counter()
    total_time = t1 - t0
    
    # 结果
    print()
    print("=" * 50)
    print("结果")
    print("=" * 50)
    print(f"最佳匹配索引: {best_idx}")
    print(f"最佳得分: {best_score}")
    print(f"总耗时: {total_time*1000:.2f} ms ({RUNS}次)")
    print(f"单次耗时: {total_time/RUNS*1000:.4f} ms")
    print(f"吞吐量: {RUNS/total_time:,.0f} 次搜索/秒")
    print(f"向量处理: {COUNT*RUNS/total_time:,.0f} 向量/秒")

if __name__ == "__main__":
    main()
