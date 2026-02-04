# Wave: ELF vs Raw Binary Comparison

**Date:** 2026-01-22

---

## Two Output Formats

| Format | Wave Command | Output |
|--------|--------------|--------|
| **ELF** | `wave input.wave -o output` | Linux executable |
| **Raw Binary** | `wave input.wave -o output.img --bare-metal` | Bootable image |

---

## Size Comparison

| Metric | ELF | Raw Binary |
|--------|-----|------------|
| **Code section** | 442 bytes | 264 bytes |
| **Data section** | 292 bytes | 292 bytes |
| **Overhead** | ELF header | Boot sector (512B) |
| **Total file** | 8,484 bytes | 8,704 bytes |

**Raw binary code is 40% smaller** (264 vs 442 bytes)

---

## Why Raw Binary Code is Smaller

```
ELF Version:
  Wave code -> Linux syscall wrappers -> kernel -> hardware
  
Raw Binary:
  Wave code -> BIOS interrupts -> hardware (direct)
```

| Operation | ELF (Linux) | Raw Binary |
|-----------|-------------|------------|
| Print | syscall write (7+ instructions) | int 0x10 (3 instructions) |
| Exit | syscall exit (5 instructions) | hlt (1 instruction) |
| Memory | mmap syscall | Direct address |

---

## Portability

| Format | Runs On |
|--------|---------|
| ELF | Linux x86-64 only |
| Raw Binary | Any x86 machine (bare metal, QEMU, real hardware) |

---

## Execution Speed

| Format | Startup | Runtime |
|--------|---------|---------|
| ELF | ~1ms (kernel loads) | Native |
| Raw Binary | ~0ms (direct boot) | Native |

Both execute at **native CPU speed** - no VM, no interpreter.

---

## File Structure

### ELF
```
┌─────────────────┐
│ ELF Header      │ 64 bytes
├─────────────────┤
│ Program Headers │ ~100 bytes
├─────────────────┤
│ Code Section    │ 442 bytes
├─────────────────┤
│ Data Section    │ 292 bytes
├─────────────────┤
│ Padding         │ ~7.5 KB
└─────────────────┘
```

### Raw Binary
```
┌─────────────────┐
│ Boot Sector     │ 512 bytes (0x55AA signature)
├─────────────────┤
│ Code Section    │ 264 bytes
├─────────────────┤
│ Data Section    │ 292 bytes
├─────────────────┤
│ Padding         │ ~7.6 KB (sector alignment)
└─────────────────┘
```

---

## Hex Comparison

### ELF Header
```
7f 45 4c 46 02 01 01 00  .ELF....
```

### Raw Binary Header
```
eb 3c 90 56 41 4c 4f 53  .<.VALOS
```

---

## Use Cases

| Use Case | Best Format |
|----------|-------------|
| Linux application | ELF |
| Operating system | Raw Binary |
| Embedded system | Raw Binary |
| Cross-platform | Raw Binary |
| Quick testing | ELF |

---

## Summary

| Aspect | ELF | Raw Binary | Winner |
|--------|-----|------------|--------|
| Code size | 442 B | 264 B | **Raw** |
| Total size | 8.4 KB | 8.7 KB | ELF |
| Portability | Linux only | Any x86 | **Raw** |
| Execution | Native | Native | Tie |
| Ease of use | Easy | Needs QEMU | ELF |

**Conclusion:** 
- Use **ELF** for Linux development
- Use **Raw Binary** for OS/embedded/maximum portability

Both are **native machine code** - no VM, no interpreter, direct CPU execution.

---

*Wave v4.3.0 - Direct Machine Code Compiler*
