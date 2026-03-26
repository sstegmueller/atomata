# atomata

A GPU-accelerated 3D cellular automaton simulator written in Rust that explores emergent behavior in particle systems. Particles interact via gravitational-like forces with configurable attraction, repulsion, and neutral interactions across multiple particle types.

Runs natively on desktop (OpenGL) and in the browser (WebAssembly).

## Features

- **Interactive mode** -- Real-time 3D visualization with egui parameter controls (particle count, friction, gravity, masses, etc.)
- **Search mode** -- Headless parallel parameter space exploration (3,600 configurations) with SQLite persistence
- **Emergence metrics** -- Information-theoretic measures (Shannon entropy, emergence, self-organization, complexity) computed over discretized state distributions
- **Analysis pipeline** -- Jupyter notebooks with XGBoost/SHAP for feature importance analysis across parameter space

## How it works

The simulation models N-body gravitational dynamics between three particle types with different masses. Each pair of types interacts via one of three modes:

| | Type 0 (light) | Type 1 (medium) | Type 2 (heavy) |
|---|---|---|---|
| **Type 0** | Repulsion | Attraction | Attraction |
| **Type 1** | | Repulsion | Attraction |
| **Type 2** | | | Neutral |

At each timestep, pairwise forces are computed (O(n²)), velocities are updated with friction damping, and positions are integrated. Particles reflect off a spherical boundary.

In search mode, particle states are discretized into 6D buckets (position + velocity) and visitation frequencies are accumulated over 10,000 steps. These empirical distributions feed into complexity metrics defined as:

- **Emergence** = Shannon entropy of the state distribution
- **Self-organization** = 1 - entropy
- **Complexity** = emergence * self-organization * 4 (bounded to [0, 1])

## Setup

Enable the Nix development environment:
```
nix develop
```

Run the application:
```
cargo run
```

Run in search mode (headless parameter sweep):
```
cargo run -- -s
```

Build for WebAssembly and serve locally:
```
cd web && sh build.sh && npm run serve
```

Analyze search results in Jupyter:
```
jupyter-lab
```

## Project structure

```
src/
  lib.rs            # Core orchestration, render loop, search mode
  bin/main.rs       # CLI entry point
  particle.rs       # Particle dynamics, state bucketing
  parameters.rs     # Configuration, interaction matrix, parameter space
  persistence.rs    # SQLite schema & migrations, state persistence
  sphere.rs         # 3D sphere rendering
python/
  metric.py         # Emergence, complexity, self-organization metrics
  repository.py     # SQLite query utilities
web/                # WASM build & webpack config
results.ipynb       # Analysis notebook (SHAP, distributions)
```

## Tech stack

- **Rust** (2021 edition) -- simulation core
- **three-d** -- GPU-accelerated 3D rendering (desktop + WASM)
- **Rayon** -- parallel parameter space search
- **rusqlite** -- state persistence
- **Python / Jupyter** -- analysis (pandas, XGBoost, SHAP, matplotlib)
- **Nix Flakes** -- reproducible dev environment
