# WASM Text Sphere

A 3D web application rendering a sphere with orbiting text, built entirely in Rust and compiled to WebAssembly. No JavaScript, TypeScript, Python, or Rust UI frameworks.

## Features

- 3D sphere rendered with WebGL via Rust/WASM
- Text billboards orbiting the sphere
- Smooth 60 FPS animation
- Pure static file deployment (HTML, CSS, WASM)

## Tech Stack

- **Rust** - Core application logic
- **WebAssembly** - Browser execution target
- **wasm-bindgen** - Rust/JS interop bindings
- **web-sys** - Web API bindings (WebGL, DOM)
- **WebGL 1.0** - 3D rendering

## Documentation

| Document | Description |
|----------|-------------|
| [PRD](docs/prd.md) | Product requirements and goals |
| [Architecture](docs/architecture.md) | System design and structure |
| [Design](docs/design.md) | Technical implementation details |
| [Plan](docs/plan.md) | Implementation phases and tasks |
| [Status](docs/status.md) | Current progress and updates |

## Quick Start

### Prerequisites

- Rust (stable)
- wasm-pack

### Build

```bash
# Install wasm-pack if not present
cargo install wasm-pack

# Build the WASM module
wasm-pack build --target web --out-dir www/pkg

# Serve the www directory
# Any static file server works, e.g.:
python3 -m http.server -d www 8080
# Or use a Rust-based server like miniserve
```

### Run

Open `http://localhost:8080` in your browser.

## Project Structure

```
wasm-text-sphere/
├── Cargo.toml          # Rust dependencies
├── README.md           # This file
├── docs/               # Documentation
│   ├── architecture.md
│   ├── design.md
│   ├── plan.md
│   ├── prd.md
│   └── status.md
├── src/                # Rust source code
│   └── lib.rs          # WASM entry point
└── www/                # Static web files
    ├── index.html
    └── style.css
```

## Browser Support

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## License

MIT
