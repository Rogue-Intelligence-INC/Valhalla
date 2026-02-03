# 🚀 Wave-GPU

**GPU-Accelerated Version of Wave Language**

Wave-GPU extends Wave with automatic GPU acceleration. The rule system naturally maps to GPU parallel patterns, allowing the same Wave code to run on CPU or GPU without modification.

---

## Core Concept: Rules → GPU Mapping

Wave's rule combinations directly correspond to GPU parallel patterns:

| Rule Combination | GPU Operation | Parallelism |
|-----------------|---------------|-------------|
| `orbital + entropy` | Vector add/mul/sub | N threads (one per element) |
| `gravitational` | Parallel reduction | O(log N) tree reduction |
| `entropy + gravitational` | Dot product | Parallel mul → parallel reduce |
| `orbital + tension` | Parallel search | N threads + atomic max |
| `orbital × orbital × entropy` | Matrix multiply | 2D thread blocks |
| `orbital + tension` | Parallel sort | Bitonic sort |

---

## Automatic CPU/GPU Selection

```wave
# Wave code (user writes)
result = vector.dot(a, b, 1000)

# Rule analysis (compiler)
entropy(mul) + gravitational(sum) → GPU parallel reduction

# Automatic execution:
if size >= 64 && gpu.enabled:
    GPU kernel
else:
    CPU fallback
```

---

## GPU Functions

### Vector Operations

```wave
# Element-wise operations (orbital + entropy)
vector.add(a, b, result, size)    # result[i] = a[i] + b[i]
vector.mul(a, b, result, size)    # result[i] = a[i] * b[i]
vector.sub(a, b, result, size)    # result[i] = a[i] - b[i]

# Reductions (gravitational)
sum = vector.sum(data, size)      # Parallel reduction sum
max = vector.max(data, size)      # Parallel reduction max
min = vector.min(data, size)      # Parallel reduction min

# Dot product (entropy + gravitational)
dot = vector.dot(a, b, size)      # Parallel mul + reduce

# Search (orbital + tension)
idx = vector.search(query, vectors, count, dim)
```

### Matrix Operations

```wave
# Matrix multiply (orbital × orbital × entropy)
matrix.mul(A, B, C, m, n, k)      # C = A × B
```

### Sorting

```wave
# Parallel sort (orbital + tension)
vector.sort(data, size)           # Bitonic sort on GPU
```

---

## GPU Memory Management

```wave
# Allocate on GPU
addr = gpu.alloc(size)

# Transfer CPU → GPU
gpu.upload(host_addr, device_addr, size)

# Transfer GPU → CPU
gpu.download(device_addr, host_addr, size)

# Free GPU memory
gpu.free(addr, size)
```

---

## Unified Field → GPU Configuration

```wave
unified {
    i: 0.8    # High information density → more threads
    e: 0.3    # Low entropy → simple compute → small blocks
    r: 0.5    # Medium relation → moderate shared memory
}
```

| Parameter | GPU Impact |
|-----------|------------|
| `i` (information density) | Thread count, memory bandwidth |
| `e` (entropy gradient) | Compute intensity, register usage |
| `r` (relation strength) | Shared memory, sync frequency |

---

## Supported Backends

| Backend | Status | Notes |
|---------|--------|-------|
| CUDA | Planned | NVIDIA GPUs |
| OpenCL | Planned | Cross-platform |
| Vulkan Compute | Planned | Modern GPUs |
| Metal | Planned | Apple GPUs |

---

## File Structure

```
wave-gpu/
├── src/
│   ├── rules/
│   │   └── gpu.wave        # GPU rule layer
│   └── platform/
│       └── gpu.wave        # GPU platform abstraction
├── test/
│   └── gpu_rules_test.wave
└── README.md
```

---

## Example: GPU-Accelerated Vector Search

```wave
# This code automatically uses GPU when available

out "Searching 10000 vectors...\n"

best_idx = 0
best_score = -999999

i = 0
loop {
    when i >= 10000 { break }
    
    # On GPU: Each iteration runs as a separate thread
    score = vector.dot(query, vectors[i], dim)
    
    when score > best_score {
        best_score = score
        best_idx = i
    }
    
    i = i + 1
}
```

---

## Why Wave is Naturally GPU-Ready

### 1. Rules Describe Intent, Not Implementation

```wave
# Wave: Describes "what"
dot = sum(q[i] * v[i])

# CUDA: Describes "how"
__global__ void kernel(...) { ... }
```

The compiler chooses the optimal implementation based on target hardware.

### 2. No Explicit Memory Management

```wave
# Wave: Automatic memory
a = tile.alloc(1000)

# CUDA: Manual management
cudaMalloc(&d_a, 1000);
cudaMemcpy(d_a, h_a, 1000, cudaMemcpyHostToDevice);
```

The Tile system automatically decides host/device allocation based on unified parameters.

### 3. Rule Combinations = Parallel Patterns

```
orbital + entropy         = embarrassingly parallel
orbital + gravitational   = parallel reduction
orbital + tension         = parallel search with sync
connection + boundary     = task parallelism
```

---

## License

MIT License

Copyright (c) 2026 Jouly Mars (ZHUOLI MA)  
Rogue Intelligence LNC.
