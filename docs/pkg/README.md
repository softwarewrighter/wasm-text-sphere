# WASM Text Sphere

A 3D web application rendering a sphere with orbiting text, built entirely in Rust (2024 Edition) and compiled to WebAssembly. No JavaScript, TypeScript, Python, or Rust UI frameworks.

![Screenshot](images/screenshot.png?ts=1734050820000)

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

## Live Demo

[**View Live Demo**](https://softwarewrighter.github.io/wasm-text-sphere/)

## Documentation

| Document | Description |
|----------|-------------|
| [PRD](documentation/prd.md) | Product requirements and goals |
| [Architecture](documentation/architecture.md) | System design and structure |
| [Design](documentation/design.md) | Technical implementation details |
| [Plan](documentation/plan.md) | Implementation phases and tasks |
| [Status](documentation/status.md) | Current progress and updates |

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

# Serve the www directory (use any static file server)
basic-http-server -a 0.0.0.0:8080 www
```

### Run

Open `http://localhost:8080` in your browser.

## Project Structure

```
wasm-text-sphere/
├── Cargo.toml          # Rust dependencies
├── README.md           # This file
├── docs/               # GitHub Pages deployment (built output)
├── documentation/      # Project documentation
│   ├── architecture.md
│   ├── design.md
│   ├── plan.md
│   ├── prd.md
│   └── status.md
├── src/                # Rust source code
│   └── lib.rs          # WASM entry point
└── www/                # Static web files (source)
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
