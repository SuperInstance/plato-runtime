# PLATO Runtime ⚒️

A self-discovering, self-optimizing compute runtime — tiny core, grows to fill resources.

Discovers your hardware, profiles it, and schedules work optimally — then uses spare cycles to improve itself.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    PLATO RUNTIME                     │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐  │
│  │ Discovery │  │ Profiler │  │    Scheduler      │  │
│  │  Engine   │  │  Engine  │  │  (Adaptive Priority)│  │
│  └────┬─────┘  └────┬─────┘  └────────┬──────────┘  │
│       │              │                  │             │
│  ┌────▼──────────────▼──────────────────▼──────────┐ │
│  │              RESOURCE REGISTRY                   │ │
│  │  CPU-AVX512 │ CPU-AVX2 │ GPU-CUDA │ WASM │ ...  │ │
│  └──────────────────────┬──────────────────────────┘ │
│                          │                            │
│  ┌───────────────────────▼──────────────────────────┐│
│  │             WORK STEALING POOL                    ││
│  │  Task Queue │ Priority Queue │ Deferred Queue     ││
│  └──────────────────────┬──────────────────────────┘│
│                          │                            │
│  ┌───────────────────────▼──────────────────────────┐│
│  │          IDLE HARVESTER                           ││
│  │  Monitors user activity │ Ramps up/down           ││
│  │  Low-priority background work when idle           ││
│  └──────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────┘
```

## Quick Start

```bash
cargo run -- discover    # Show all discovered resources + profiles
cargo run -- profile     # Run full profiling suite
cargo run -- benchmark   # Run constraint benchmarks
cargo run -- optimize    # Run self-optimization cycle
cargo run -- monitor     # Live dashboard (refresh every 1s)
cargo run -- idle-harvest # Start idle harvester daemon
```

## Discovery Output

```
╔══════════════════════════════════════════════════╗
║         PLATO RUNTIME — Resource Discovery       ║
╚══════════════════════════════════════════════════╝

  CPU Cluster (12C/24T)
  Status: ✓ Available
  Cores: 12 / Threads: 24
  AVX-512:       ✓
    VNNI:        ✓  IFMA:        ✓  BF16:        ✓
    VBMI2:       ✓  VPOPCNTDQ:   ✓  VP2INTERSECT:✓  BITALG:      ✓
  AVX2:           ✓
  AVX-VNNI:       ✓
  L2 Cache:       1024 KB/core

  NVIDIA RTX 4050 Laptop (Ada, sm_86)
  Compute Capability: sm_86
  SM Count: 16
  VRAM: 6.4 GB
```

## Performance Profile

```
  Constraint Check INT8:   359.33 Mops/s
  Constraint Check INT32:  199.63 Mops/s
  Constraint Check FP64:   118.62 Mops/s
  Memory Bandwidth:        0.30 GB/s
  Operation Latency:       5.2 ns
  TSC Frequency:           1.97 GHz
```

## Idle Harvester

When the user is idle for >30 seconds, the runtime ramps up background tasks:
- **Self-profiling** — Re-benchmark to detect thermal throttling
- **Kernel precompilation** — Pre-warm GPU kernels
- **Optimization sweeps** — Find best kernel configs
- **PLATO tile processing** — Knowledge crunching

Smoothly transitions between active (20% resources) and idle (80% resources) over 5 seconds.

## As a Library

```rust
use plato_runtime::{PlatoRuntime, RuntimeConfig};

let mut rt = PlatoRuntime::new()?;
let resources = rt.resources();
println!("CPU cores: {:?}", resources.cpu_resources());
println!("GPU: {:?}", resources.gpu_resources());

// Run self-optimization
let result = rt.optimize();
println!("Optimal threads: {}", result.thread_count);

rt.shutdown();
```

## Design Principles

1. **Self-discovering** — Reads /proc/cpuinfo, sysfs cache topology, CUDA driver
2. **Self-profiling** — Auto-benchmarks each resource on startup
3. **Self-optimizing** — Chooses optimal precision, batch size, thread count
4. **Idle-aware** — Background work scales with user activity
5. **Minimal deps** — Only `libc` for CPU affinity. Everything else is `std`

## Requirements

- Rust 1.75+ (edition 2021)
- Linux (uses /proc, sysfs, sched_setaffinity)
- Optional: CUDA driver for GPU detection

## License

MIT
