use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, WebGlBuffer, WebGlProgram,
    WebGlRenderingContext as GL, WebGlShader, WebGlTexture,
};

// Math types
#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn normalize(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            self
        }
    }

    fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[derive(Clone, Copy)]
struct Mat4 {
    data: [f32; 16],
}

impl Mat4 {
    fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y / 2.0).tan();
        let nf = 1.0 / (near - far);
        Self {
            data: [
                f / aspect,
                0.0,
                0.0,
                0.0,
                0.0,
                f,
                0.0,
                0.0,
                0.0,
                0.0,
                (far + near) * nf,
                -1.0,
                0.0,
                0.0,
                2.0 * far * near * nf,
                0.0,
            ],
        }
    }

    fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let z = eye.sub(target).normalize();
        let x = up.cross(z).normalize();
        let y = z.cross(x);

        Self {
            data: [
                x.x,
                y.x,
                z.x,
                0.0,
                x.y,
                y.y,
                z.y,
                0.0,
                x.z,
                y.z,
                z.z,
                0.0,
                -x.dot(eye),
                -y.dot(eye),
                -z.dot(eye),
                1.0,
            ],
        }
    }

    fn rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            data: [
                c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    fn billboard(position: Vec3, camera_pos: Vec3, scale: f32) -> Self {
        let forward = camera_pos.sub(position).normalize();
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let right = world_up.cross(forward).normalize();
        let up = forward.cross(right);

        Self {
            data: [
                right.x * scale,
                right.y * scale,
                right.z * scale,
                0.0,
                up.x * scale,
                up.y * scale,
                up.z * scale,
                0.0,
                forward.x * scale,
                forward.y * scale,
                forward.z * scale,
                0.0,
                position.x,
                position.y,
                position.z,
                1.0,
            ],
        }
    }
}

// Sphere geometry
fn generate_sphere(
    radius: f32,
    lat_segments: u32,
    lon_segments: u32,
) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
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

            vertices.extend_from_slice(&[x * radius, y * radius, z * radius]);
            normals.extend_from_slice(&[x, y, z]);
        }
    }

    for lat in 0..lat_segments {
        for lon in 0..lon_segments {
            let first = lat * (lon_segments + 1) + lon;
            let second = first + lon_segments + 1;

            indices.extend_from_slice(&[
                first as u16,
                second as u16,
                (first + 1) as u16,
                second as u16,
                (second + 1) as u16,
                (first + 1) as u16,
            ]);
        }
    }

    (vertices, normals, indices)
}

// Textured quad for billboards
fn generate_quad() -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let vertices = vec![
        -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, 0.5, 0.0, -0.5, 0.5, 0.0,
    ];
    let uvs = vec![0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (vertices, uvs, indices)
}

// Create text texture using Canvas 2D with color
fn create_text_texture(
    gl: &GL,
    document: &web_sys::Document,
    letter: &str,
    color: &str,
) -> Result<WebGlTexture, String> {
    let canvas = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "Failed to cast to canvas")?;

    let size = 128u32;
    canvas.set_width(size);
    canvas.set_height(size);

    let ctx = canvas
        .get_context("2d")
        .map_err(|_| "Failed to get 2d context")?
        .ok_or("No 2d context")?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| "Failed to cast to 2d context")?;

    // Clear with transparent background
    ctx.clear_rect(0.0, 0.0, size as f64, size as f64);

    // Draw text with color
    ctx.set_font("bold 90px 'Outfit', sans-serif");
    ctx.set_fill_style_str(color);
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    ctx.fill_text(letter, size as f64 / 2.0, size as f64 / 2.0)
        .map_err(|_| "Failed to draw text")?;

    // Create WebGL texture
    let texture = gl.create_texture().ok_or("Failed to create texture")?;
    gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

    gl.tex_image_2d_with_u32_and_u32_and_canvas(
        GL::TEXTURE_2D,
        0,
        GL::RGBA as i32,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        &canvas,
    )
    .map_err(|_| "Failed to upload texture")?;

    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

    Ok(texture)
}

// Orbiting letter with texture
struct OrbitingLetter {
    texture: WebGlTexture,
    radius: f32,
    inclination: f32,
    phase: f32,
    angular_velocity: f32,
}

impl OrbitingLetter {
    fn position_at(&self, time: f32) -> Vec3 {
        let angle = self.phase + self.angular_velocity * time;
        let x = self.radius * angle.cos();
        let z = self.radius * angle.sin();
        let y = z * self.inclination.sin();
        let z = z * self.inclination.cos();
        Vec3::new(x, y, z)
    }
}

// Shaders
const SPHERE_VERTEX_SHADER: &str = r#"
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
"#;

const SPHERE_FRAGMENT_SHADER: &str = r#"
    precision mediump float;
    varying vec3 v_normal;
    varying vec3 v_position;
    uniform vec3 u_lightPos;
    uniform vec3 u_color;
    uniform vec3 u_viewPos;
    void main() {
        vec3 normal = normalize(v_normal);
        vec3 lightDir = normalize(u_lightPos - v_position);
        vec3 viewDir = normalize(u_viewPos - v_position);
        vec3 halfDir = normalize(lightDir + viewDir);
        float ambient = 0.15;
        float diff = max(dot(normal, lightDir), 0.0);
        float spec = pow(max(dot(normal, halfDir), 0.0), 32.0);
        vec3 color = (ambient + diff * 0.7 + spec * 0.3) * u_color;
        gl_FragColor = vec4(color, 1.0);
    }
"#;

const TEXT_VERTEX_SHADER: &str = r#"
    attribute vec3 a_position;
    attribute vec2 a_uv;
    uniform mat4 u_model;
    uniform mat4 u_view;
    uniform mat4 u_projection;
    varying vec2 v_uv;
    void main() {
        v_uv = a_uv;
        gl_Position = u_projection * u_view * u_model * vec4(a_position, 1.0);
    }
"#;

const TEXT_FRAGMENT_SHADER: &str = r#"
    precision mediump float;
    varying vec2 v_uv;
    uniform sampler2D u_texture;
    void main() {
        vec4 texColor = texture2D(u_texture, v_uv);
        if (texColor.a < 0.1) discard;
        gl_FragColor = texColor;
    }
"#;

// WebGL helpers
fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or("Unable to create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown shader error".into()))
    }
}

fn link_program(gl: &GL, vert: &WebGlShader, frag: &WebGlShader) -> Result<WebGlProgram, String> {
    let program = gl.create_program().ok_or("Unable to create program")?;
    gl.attach_shader(&program, vert);
    gl.attach_shader(&program, frag);
    gl.link_program(&program);
    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown program error".into()))
    }
}

fn create_buffer(gl: &GL, data: &[f32]) -> Result<WebGlBuffer, String> {
    let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
    unsafe {
        let array = js_sys::Float32Array::view(data);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &array, GL::STATIC_DRAW);
    }
    Ok(buffer)
}

fn create_index_buffer(gl: &GL, data: &[u16]) -> Result<WebGlBuffer, String> {
    let buffer = gl.create_buffer().ok_or("Failed to create index buffer")?;
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer));
    unsafe {
        let array = js_sys::Uint16Array::view(data);
        gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &array, GL::STATIC_DRAW);
    }
    Ok(buffer)
}

// Application state
struct App {
    gl: GL,
    sphere_program: WebGlProgram,
    text_program: WebGlProgram,
    sphere_vertex_buffer: WebGlBuffer,
    sphere_normal_buffer: WebGlBuffer,
    sphere_index_buffer: WebGlBuffer,
    sphere_index_count: i32,
    quad_vertex_buffer: WebGlBuffer,
    quad_uv_buffer: WebGlBuffer,
    quad_index_buffer: WebGlBuffer,
    letters: Vec<OrbitingLetter>,
    camera_pos: Vec3,
    view_matrix: Mat4,
    projection_matrix: Mat4,
}

impl App {
    fn new(gl: GL, document: &web_sys::Document, width: u32, height: u32) -> Result<Self, String> {
        // Compile shaders
        let sphere_vert = compile_shader(&gl, GL::VERTEX_SHADER, SPHERE_VERTEX_SHADER)?;
        let sphere_frag = compile_shader(&gl, GL::FRAGMENT_SHADER, SPHERE_FRAGMENT_SHADER)?;
        let sphere_program = link_program(&gl, &sphere_vert, &sphere_frag)?;

        let text_vert = compile_shader(&gl, GL::VERTEX_SHADER, TEXT_VERTEX_SHADER)?;
        let text_frag = compile_shader(&gl, GL::FRAGMENT_SHADER, TEXT_FRAGMENT_SHADER)?;
        let text_program = link_program(&gl, &text_vert, &text_frag)?;

        // Generate sphere
        let (sphere_verts, sphere_normals, sphere_indices) = generate_sphere(1.0, 32, 64);
        let sphere_vertex_buffer = create_buffer(&gl, &sphere_verts)?;
        let sphere_normal_buffer = create_buffer(&gl, &sphere_normals)?;
        let sphere_index_buffer = create_index_buffer(&gl, &sphere_indices)?;
        let sphere_index_count = sphere_indices.len() as i32;

        // Generate quad for text billboards
        let (quad_verts, quad_uvs, quad_indices) = generate_quad();
        let quad_vertex_buffer = create_buffer(&gl, &quad_verts)?;
        let quad_uv_buffer = create_buffer(&gl, &quad_uvs)?;
        let quad_index_buffer = create_index_buffer(&gl, &quad_indices)?;

        // Create letter textures and orbits for "[wasm-text-sphere]"
        // Letters orbit clockwise (negative velocity) and are evenly spaced
        let text = "[wasm-text-sphere]";
        let colors = [
            "#FF6B6B", // Red
            "#4ECDC4", // Teal
            "#45B7D1", // Blue
            "#96CEB4", // Green
            "#FFEAA7", // Yellow
            "#DDA0DD", // Plum
            "#98D8C8", // Mint
            "#F7DC6F", // Gold
            "#BB8FCE", // Purple
            "#85C1E9", // Light blue
            "#F8B500", // Orange
            "#00CED1", // Dark cyan
            "#FF69B4", // Hot pink
            "#7FFF00", // Chartreuse
            "#FFB6C1", // Light pink
            "#40E0D0", // Turquoise
            "#FF6347", // Tomato
            "#9370DB", // Medium purple
        ];

        let char_count = text.chars().count();
        let mut letters = Vec::new();

        for (i, ch) in text.chars().enumerate() {
            let phase = -PI / 2.0 - (i as f32 * 2.0 * PI / char_count as f32);
            let color = colors[i % colors.len()];
            let letter_str = ch.to_string();
            let texture = create_text_texture(&gl, document, &letter_str, color)?;

            letters.push(OrbitingLetter {
                texture,
                radius: 2.2,
                inclination: 0.12,
                phase,
                angular_velocity: 0.3,
            });
        }

        // Camera setup
        let camera_pos = Vec3::new(0.0, 0.5, 5.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let view_matrix = Mat4::look_at(camera_pos, target, up);

        let aspect = width as f32 / height as f32;
        let projection_matrix = Mat4::perspective(PI / 4.0, aspect, 0.1, 100.0);

        // WebGL state
        gl.enable(GL::DEPTH_TEST);
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        gl.clear_color(0.05, 0.05, 0.1, 1.0);

        Ok(Self {
            gl,
            sphere_program,
            text_program,
            sphere_vertex_buffer,
            sphere_normal_buffer,
            sphere_index_buffer,
            sphere_index_count,
            quad_vertex_buffer,
            quad_uv_buffer,
            quad_index_buffer,
            letters,
            camera_pos,
            view_matrix,
            projection_matrix,
        })
    }

    fn render(&self, time: f32) {
        let gl = &self.gl;

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let sphere_rotation = time * 0.1;
        let model_matrix = Mat4::rotation_y(sphere_rotation);

        // Draw sphere
        gl.use_program(Some(&self.sphere_program));

        let u_model = gl.get_uniform_location(&self.sphere_program, "u_model");
        let u_view = gl.get_uniform_location(&self.sphere_program, "u_view");
        let u_projection = gl.get_uniform_location(&self.sphere_program, "u_projection");
        let u_light_pos = gl.get_uniform_location(&self.sphere_program, "u_lightPos");
        let u_color = gl.get_uniform_location(&self.sphere_program, "u_color");
        let u_view_pos = gl.get_uniform_location(&self.sphere_program, "u_viewPos");

        gl.uniform_matrix4fv_with_f32_array(u_model.as_ref(), false, &model_matrix.data);
        gl.uniform_matrix4fv_with_f32_array(u_view.as_ref(), false, &self.view_matrix.data);
        gl.uniform_matrix4fv_with_f32_array(
            u_projection.as_ref(),
            false,
            &self.projection_matrix.data,
        );
        gl.uniform3f(u_light_pos.as_ref(), 5.0, 5.0, 5.0);
        gl.uniform3f(u_color.as_ref(), 0.25, 0.45, 0.75);
        gl.uniform3f(
            u_view_pos.as_ref(),
            self.camera_pos.x,
            self.camera_pos.y,
            self.camera_pos.z,
        );

        let a_position = gl.get_attrib_location(&self.sphere_program, "a_position") as u32;
        let a_normal = gl.get_attrib_location(&self.sphere_program, "a_normal") as u32;

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.sphere_vertex_buffer));
        gl.vertex_attrib_pointer_with_i32(a_position, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(a_position);

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.sphere_normal_buffer));
        gl.vertex_attrib_pointer_with_i32(a_normal, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(a_normal);

        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.sphere_index_buffer));
        gl.draw_elements_with_i32(
            GL::TRIANGLES,
            self.sphere_index_count,
            GL::UNSIGNED_SHORT,
            0,
        );

        // Draw orbiting letters
        gl.use_program(Some(&self.text_program));

        let u_model = gl.get_uniform_location(&self.text_program, "u_model");
        let u_view = gl.get_uniform_location(&self.text_program, "u_view");
        let u_projection = gl.get_uniform_location(&self.text_program, "u_projection");
        let u_texture = gl.get_uniform_location(&self.text_program, "u_texture");

        gl.uniform_matrix4fv_with_f32_array(u_view.as_ref(), false, &self.view_matrix.data);
        gl.uniform_matrix4fv_with_f32_array(
            u_projection.as_ref(),
            false,
            &self.projection_matrix.data,
        );
        gl.uniform1i(u_texture.as_ref(), 0);

        let a_position = gl.get_attrib_location(&self.text_program, "a_position") as u32;
        let a_uv = gl.get_attrib_location(&self.text_program, "a_uv") as u32;

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.quad_vertex_buffer));
        gl.vertex_attrib_pointer_with_i32(a_position, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(a_position);

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.quad_uv_buffer));
        gl.vertex_attrib_pointer_with_i32(a_uv, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(a_uv);

        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.quad_index_buffer));

        for letter in &self.letters {
            let pos = letter.position_at(time);
            let letter_model = Mat4::billboard(pos, self.camera_pos, 0.6);

            gl.uniform_matrix4fv_with_f32_array(u_model.as_ref(), false, &letter_model.data);

            gl.active_texture(GL::TEXTURE0);
            gl.bind_texture(GL::TEXTURE_2D, Some(&letter.texture));

            gl.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_SHORT, 0);
        }
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("No canvas")?
        .dyn_into::<HtmlCanvasElement>()?;

    let width = window.inner_width()?.as_f64().unwrap() as u32;
    let height = window.inner_height()?.as_f64().unwrap() as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    let gl = canvas
        .get_context("webgl")?
        .ok_or("WebGL not supported")?
        .dyn_into::<GL>()?;

    gl.viewport(0, 0, width as i32, height as i32);

    let app = Rc::new(RefCell::new(
        App::new(gl, &document, width, height).map_err(|e| JsValue::from_str(&e))?,
    ));

    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let app_clone = app.clone();

    *g.borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let time = (timestamp / 1000.0) as f32;
        app_clone.borrow().render(time);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
