#!/usr/bin/env python3
"""向量余弦相似度搜索 - Python版"""
import time

DIM = 16
COUNT = 100

class RNG:
    def __init__(self, seed):
        self.seed = seed
    
    def rand(self):
        self.seed = self.seed * 1103 + 12345
        q = self.seed // 100000
        self.seed = self.seed - q * 100000
        if self.seed < 0:
            self.seed = -self.seed
        return self.seed // 1000 - 50

print("=== Vector Cosine Search - Python ===")
print(f"DIM={DIM}, COUNT={COUNT}\n")

# Query
print("Query: generating...")
rng = RNG(12345)
query = [rng.rand() for _ in range(DIM)]

# Search
print("Searching 100000 vectors...")
t0 = time.perf_counter()

best_idx = 0
best_score = -999999999

rng = RNG(42)
for i in range(COUNT):
    vec = [rng.rand() for _ in range(DIM)]
    score = sum(q * v for q, v in zip(query, vec))
    if score > best_score:
        best_score = score
        best_idx = i

t1 = time.perf_counter()
print("Search complete!")
print(f"=== Best match: idx={best_idx}, score={best_score} ===")
print(f"Time: {(t1-t0)*1000:.2f} ms")
print(f"Speed: {COUNT/(t1-t0):,.0f} vectors/sec")
