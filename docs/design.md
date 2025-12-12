# Technical Design

## Rendering Approach

### WebGL Strategy

We use WebGL 1.0 for maximum browser compatibility. All WebGL calls are made directly from Rust via `web-sys` bindings, with no JavaScript wrapper layer.

```
Rust Code ─► wasm-bindgen ─► web-sys ─► WebGL API ─► GPU
```

### Shaders

#### Vertex Shader (Sphere)
```glsl
attribute vec3 a_position;
attribute vec3 a_normal;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

varying vec3 v_normal;
varying vec3 v_position;

void main() {
    vec4 worldPos = u_model * vec4(a_position, 1.0);
    v_position = worldPos.xyz;
    v_normal = mat3(u_model) * a_normal;
    gl_Position = u_projection * u_view * worldPos;
}
```

#### Fragment Shader (Sphere)
```glsl
precision mediump float;

varying vec3 v_normal;
varying vec3 v_position;

uniform vec3 u_lightPos;
uniform vec3 u_color;

void main() {
    vec3 normal = normalize(v_normal);
    vec3 lightDir = normalize(u_lightPos - v_position);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 ambient = 0.2 * u_color;
    vec3 diffuse = diff * u_color;
    gl_FragColor = vec4(ambient + diffuse, 1.0);
}
```

## 3D Sphere Generation

### UV Sphere Algorithm

Generate sphere using latitude/longitude subdivision:

```rust
fn generate_sphere(radius: f32, lat_segments: u32, lon_segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=lat_segments {
        let theta = lat as f32 * PI / lat_segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=lon_segments {
            let phi = lon as f32 * 2.0 * PI / lon_segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            vertices.push([x * radius, y * radius, z * radius]);
            normals.push([x, y, z]);
        }
    }

    // Generate triangle indices...
    Mesh { vertices, normals, indices }
}
```

### Sphere Parameters
- Radius: 1.0 (normalized, scaled via model matrix)
- Latitude segments: 32
- Longitude segments: 64
- Total triangles: ~4,000

## Text Rendering

### Approach: Texture Atlas Billboards

Since we cannot use JavaScript for font rendering, we use a pre-generated texture atlas:

1. **Texture Atlas**: Bitmap font texture embedded in WASM binary
2. **Billboards**: Textured quads that always face the camera
3. **Character Mapping**: ASCII lookup table for texture coordinates

### Billboard Implementation

```rust
struct TextBillboard {
    text: String,
    position: Vec3,      // Orbital position
    scale: f32,
    texture_coords: Vec<[f32; 4]>,  // Per-character UVs
}

impl TextBillboard {
    fn get_model_matrix(&self, camera_right: Vec3, camera_up: Vec3) -> Mat4 {
        // Construct matrix so quad faces camera
        Mat4::from_columns(
            camera_right.extend(0.0),
            camera_up.extend(0.0),
            (camera_right.cross(camera_up)).extend(0.0),
            self.position.extend(1.0),
        )
    }
}
```

## Orbital Animation

### Orbital Motion Model

Each text item follows a circular orbit around the sphere:

```rust
struct Orbit {
    radius: f32,           // Distance from sphere center
    inclination: f32,      // Tilt angle from equator
    phase: f32,            // Starting angle
    angular_velocity: f32, // Radians per second
}

impl Orbit {
    fn position_at(&self, time: f32) -> Vec3 {
        let angle = self.phase + self.angular_velocity * time;

        // Position in orbital plane
        let x = self.radius * angle.cos();
        let z = self.radius * angle.sin();

        // Rotate by inclination
        let y = z * self.inclination.sin();
        let z = z * self.inclination.cos();

        Vec3::new(x, y, z)
    }
}
```

### Multiple Orbits

Configure multiple text items with varied parameters:

| Text | Radius | Inclination | Angular Velocity |
|------|--------|-------------|------------------|
| "Rust" | 1.5 | 0° | 0.5 rad/s |
| "WASM" | 1.6 | 30° | -0.4 rad/s |
| "WebGL" | 1.7 | 60° | 0.3 rad/s |

## Camera System

### Fixed Camera with Perspective

```rust
struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl Camera {
    fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov, self.aspect, self.near, self.far)
    }
}
```

### Default Camera Setup
- Position: (0, 0, 5)
- Target: (0, 0, 0)
- FOV: 45°
- Near plane: 0.1
- Far plane: 100.0

## Color Scheme

| Element | Color (RGB) | Hex |
|---------|-------------|-----|
| Background | (15, 15, 25) | #0F0F19 |
| Sphere | (100, 149, 237) | #6495ED |
| Text | (255, 255, 255) | #FFFFFF |
| Light | (255, 244, 214) | #FFF4D6 |

## Performance Considerations

1. **Single Draw Call for Sphere**: All sphere geometry in one buffer
2. **Instanced Text**: Batch text billboards where possible
3. **No Allocations in Render Loop**: Pre-allocate all buffers
4. **Delta Time Animation**: Frame-rate independent movement
