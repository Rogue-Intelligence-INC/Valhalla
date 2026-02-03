# 🌊 Wave-Static

**Static Version of Wave Language — No Runtime Code Modification**

Wave-Static is the static compilation version of Wave. Unlike the dynamic version where Fate can modify code at runtime, Wave-Static produces fixed machine code that never changes during execution.

---

## Key Difference from Dynamic Wave

| Feature | Dynamic (wave/) | Static (wave-static/) |
|---------|-----------------|----------------------|
| Fate | Observes, learns, modifies code | Disabled |
| Code modification | Runtime optimization | None |
| Behavior | Adaptive | Deterministic |
| Use case | General purpose | Embedded, real-time |

---

## What's Changed

### Fate System (Static Version)

```wave
# Dynamic version: Fate observes and may emit learned patterns
fn fate.emit_learned key id {
    pattern = fate.learned[key + ":" + id]
    when pattern { emit pattern }  # ← Modifies code
}

# Static version: Empty implementation
fn fate.emit_learned key id {
    # No code modification
}
```

### Code Generation (Static Version)

```wave
# Dynamic version: Inserts measurement hooks
fn codegen.loop_with_fate body {
    codegen.check_fate()
    codegen.measure_begin()
    compile(body)
    codegen.measure_end()
}

# Static version: Direct compilation
fn codegen.loop_with_fate body {
    compile(body)  # No hooks
}
```

---

## Complete Rule System Preserved

All eight fundamental rules are preserved:

- ✅ `gravitational` — Attraction, aggregation
- ✅ `tension` — Comparison, branching
- ✅ `entropy` — Computation, transformation
- ✅ `connection` — Data transfer, linking
- ✅ `memory` — Storage, persistence
- ✅ `orbital` — Loops, cycles
- ✅ `boundary` — Scope, isolation
- ✅ `evolution` — Adaptation, transformation

---

## File Structure

```
wave-static/
├── src/
│   └── rules/
│       ├── unified.wave   # Unified field (fixed i,e,r = 0.5)
│       ├── core.wave      # Eight base rules + GPU mapping
│       ├── fate.wave      # Fate disabled (empty implementations)
│       └── codegen.wave   # No Fate hooks
├── test/
│   ├── simple_rules_test.wave
│   ├── vector_final.wave
│   └── bench_10k.wave
└── README.md
```

---

## Use Cases

- **Embedded Systems** — Limited resources, no runtime optimization needed
- **Real-time Applications** — Deterministic execution time required
- **Security-sensitive** — Code cannot be modified at runtime
- **Debugging/Testing** — Predictable behavior

---

## Building

```bash
# Use the same compiler
../wave/bin/wavec test/simple_rules_test.wave
./a.out
```

---

## License

MIT License

Copyright (c) 2026 Jouly Mars (ZHUOLI MA)  
Rogue Intelligence LNC.
