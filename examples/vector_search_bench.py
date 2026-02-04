#!/usr/bin/env python3
"""
向量余弦相似度搜索 - Python版
与 Wave 版本对比 (相同参数)
"""
import time

# 配置 (与Wave相同)
DIM = 128
COUNT = 100000

# LCG随机数 (与Wave相同)
class LCG:
    def __init__(self, seed):
        self.seed = seed
    
    def rand(self):
        self.seed = (self.seed * 1103515245 + 12345) & 0x7FFFFFFF
        r = self.seed % 2001 - 1000
        return r / 1000.0

def dot(a, b):
    return sum(x * y for x, y in zip(a, b))

def make_vec(rng, dim):
    return [rng.rand() for _ in range(dim)]

def main():
    print("=== Python Vector Search ===")
    print(f"Dim: {DIM}, Count: {COUNT}\n")
    
    print("Generating database...")
    t0 = time.perf_counter()
    rng = LCG(42)
    db = [make_vec(rng, DIM) for _ in range(COUNT)]
    t1 = time.perf_counter()
    print(f"Done. ({t1-t0:.2f}s)\n")
    
    print("Generating query...")
    rng = LCG(12345)
    query = make_vec(rng, DIM)
    print("Done.\n")
    
    print("Searching...")
    t2 = time.perf_counter()
    best_idx = 0
    best_score = -999999999.0
    for i, vec in enumerate(db):
        score = dot(query, vec)
        if score > best_score:
            best_score = score
            best_idx = i
    t3 = time.perf_counter()
    
    print("Done.\n")
    print("=== Search Complete ===")
    print(f"Best index: {best_idx}")
    print(f"Best score: {best_score:.6f}")
    print(f"Search time: {t3-t2:.3f}s")
    print(f"Speed: {COUNT/(t3-t2):,.0f} vectors/sec")

if __name__ == "__main__":
    main()
