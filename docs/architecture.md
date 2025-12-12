# Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser                               │
│  ┌───────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │  index.html   │  │   style.css    │  │   app.wasm     │  │
│  │               │  │                │  │                │  │
│  │  Canvas       │  │  Layout        │  │  Rust Logic    │  │
│  │  Element      │  │  Styling       │  │  WebGL Calls   │  │
│  └───────┬───────┘  └────────────────┘  └───────┬────────┘  │
│          │                                       │           │
│          │         ┌─────────────────┐          │           │
│          └────────►│   WebGL API     │◄─────────┘           │
│                    │   (Browser)     │                      │
│                    └────────┬────────┘                      │
│                             │                               │
│                    ┌────────▼────────┐                      │
│                    │      GPU        │                      │
│                    └─────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
wasm-text-sphere/
├── Cargo.toml              # Rust project configuration
├── README.md               # Project overview
├── docs/
│   ├── architecture.md     # This file
│   ├── design.md           # Technical design details
│   ├── plan.md             # Implementation plan
│   ├── prd.md              # Product requirements
│   └── status.md           # Current status
├── src/
│   ├── lib.rs              # WASM entry point & exports
│   ├── renderer/
│   │   ├── mod.rs          # Renderer module
│   │   ├── webgl.rs        # WebGL bindings & setup
│   │   ├── shader.rs       # Shader compilation & programs
│   │   └── buffer.rs       # Vertex/index buffer management
│   ├── scene/
│   │   ├── mod.rs          # Scene module
│   │   ├── sphere.rs       # Sphere geometry generation
│   │   ├── text.rs         # Text rendering & positioning
│   │   └── camera.rs       # Camera/view matrix
│   ├── math/
│   │   ├── mod.rs          # Math utilities
│   │   ├── matrix.rs       # 4x4 matrix operations
│   │   └── vector.rs       # Vector operations
│   └── animation/
│       ├── mod.rs          # Animation module
│       └── orbit.rs        # Orbital motion calculations
└── www/
    ├── index.html          # HTML entry point
    └── style.css           # Application styles
```

## Component Architecture

### 1. WASM Entry Point (`lib.rs`)
- Exports `init()` function called from HTML
- Sets up panic hook for debugging
- Initializes WebGL context
- Starts render loop using `requestAnimationFrame`

### 2. Renderer Module
- **webgl.rs**: Raw WebGL bindings via `web-sys`
- **shader.rs**: Vertex and fragment shader management
- **buffer.rs**: GPU buffer allocation and updates

### 3. Scene Module
- **sphere.rs**: Generates sphere mesh (vertices, normals, indices)
- **text.rs**: Text billboard rendering with texture atlas
- **camera.rs**: View and projection matrix management

### 4. Math Module
- Pure Rust math without external dependencies
- 4x4 matrix operations (multiply, inverse, transpose)
- Vector3/Vector4 operations
- Trigonometric utilities for orbital calculations

### 5. Animation Module
- **orbit.rs**: Calculates orbital positions over time
- Handles timing via `performance.now()`

## Dependencies

```toml
[dependencies]
wasm-bindgen = "0.2"        # Rust/WASM bindings
web-sys = "0.3"             # Web API bindings
js-sys = "0.3"              # JavaScript type bindings

[dependencies.web-sys]
features = [
    "Window",
    "Document",
    "Element",
    "HtmlCanvasElement",
    "WebGlRenderingContext",
    "WebGlProgram",
    "WebGlShader",
    "WebGlBuffer",
    "WebGlUniformLocation",
    "Performance",
]
```

## Data Flow

1. **Initialization**
   - HTML loads WASM module
   - `init()` called, acquires canvas and WebGL context
   - Shaders compiled, buffers created
   - Scene objects (sphere, text) initialized

2. **Render Loop**
   - `requestAnimationFrame` callback invoked
   - Current time retrieved from `performance.now()`
   - Orbital positions calculated
   - View/projection matrices updated
   - Draw calls issued to WebGL
   - Loop continues

3. **Memory Management**
   - Rust manages all allocations
   - WebGL resources tracked and cleaned up
   - No JavaScript garbage collection concerns
