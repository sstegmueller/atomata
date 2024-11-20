# atomata

## Setup
- Enable nix-flake:
```
nix develop
```

- Run the application (use '-s' flag for search mode):
```
cargo run
```

- Build for wasm and serve locally
```
cd web && sh build.sh && npm run serve
```

- Analyze search mode results
```
jupyter-lab
```
