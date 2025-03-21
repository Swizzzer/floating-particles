use rand::Rng;
use std::f64;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    width: f64,
    height: f64,
    connection_distance: f64,
    mouse_x: f64,
    mouse_y: f64,
    mouse_radius: f64,
    mouse_force: f64,
    mouse_connections: Vec<(usize, f64)>, // 存储与鼠标连接的粒子及其强度
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub size: f64,
    pub base_vx: f64, // 基础速度，保持随机漂浮
    pub base_vy: f64,
    pub orbit_angle: f64,  // 轨道角度
    pub orbit_speed: f64,  // 轨道速度
    pub orbit_radius: f64, // 轨道半径
    pub is_orbiting: bool, // 是否在轨道上
}

#[wasm_bindgen]
impl ParticleSystem {
    #[wasm_bindgen(constructor)]
    pub fn new(
        width: f64,
        height: f64,
        num_particles: usize,
        connection_distance: f64,
    ) -> ParticleSystem {
        let mut rng = rand::thread_rng();
        let mut particles = Vec::with_capacity(num_particles);

        for _ in 0..num_particles {
            let x = rng.gen_range(0.0..width);
            let y = rng.gen_range(0.0..height);

            let base_vx = rng.gen_range(-0.4..0.4);
            let base_vy = rng.gen_range(-0.4..0.4);

            let particle = Particle {
                x,
                y,
                vx: base_vx,
                vy: base_vy,
                size: rng.gen_range(1.0..3.0),
                base_vx,
                base_vy,
                orbit_angle: rng.gen_range(0.0..std::f64::consts::PI * 2.0),
                orbit_speed: rng.gen_range(0.002..0.008),
                orbit_radius: rng.gen_range(5.0..60.0),
                is_orbiting: false,
            };
            particles.push(particle);
        }

        ParticleSystem {
            particles,
            width,
            height,
            connection_distance,
            mouse_x: -1000.0,
            mouse_y: -1000.0,
            mouse_radius: 150.0,
            mouse_force: 1.0,
            mouse_connections: Vec::new(),
        }
    }
    // TODO: 流体阻尼效果(现在的粒子会被鼠标推着走, 不太符合物理规律)
    pub fn update(&mut self) {
        let mouse_active = self.mouse_x >= 0.0
            && self.mouse_y >= 0.0
            && self.mouse_x <= self.width
            && self.mouse_y <= self.height;

        self.mouse_connections.clear();

        for (idx, particle) in self.particles.iter_mut().enumerate() {
            if mouse_active {
                let mut rng = rand::thread_rng();

                let dx = particle.x - self.mouse_x;
                let dy = particle.y - self.mouse_y;
                let direct_distance = (dx * dx + dy * dy).sqrt();

                if direct_distance < self.mouse_radius {
                    let attraction_strength = 1.0 - (direct_distance / self.mouse_radius);

                    self.mouse_connections.push((idx, attraction_strength));

                    if particle.is_orbiting {
                        let target_distance = self.mouse_radius * rng.gen_range(0.6..1.5);
                        let transition_speed = 0.05;

                        particle.orbit_radius +=
                            (target_distance - particle.orbit_radius) * transition_speed;
                        particle.orbit_angle += particle.orbit_speed;
                    } else {
                        particle.orbit_angle = dy.atan2(dx);
                        particle.orbit_radius = direct_distance;
                        particle.is_orbiting = true;
                    }

                    let target_x =
                        self.mouse_x + particle.orbit_radius * particle.orbit_angle.cos();
                    let target_y =
                        self.mouse_y + particle.orbit_radius * particle.orbit_angle.sin();

                    particle.vx = (target_x - particle.x) * 0.1 * attraction_strength
                        + particle.base_vx * (1.0 - attraction_strength);
                    particle.vy = (target_y - particle.y) * 0.1 * attraction_strength
                        + particle.base_vy * (1.0 - attraction_strength);
                } else {
                    particle.is_orbiting = false;
                    particle.vx = particle.base_vx;
                    particle.vy = particle.base_vy;
                    particle.orbit_radius *= 0.95;
                }
            } else {
                particle.is_orbiting = false;
                particle.vx = particle.base_vx;
                particle.vy = particle.base_vy;
            }

            particle.x += particle.vx;
            particle.y += particle.vy;

            if particle.x < 0.0 {
                particle.x = self.width;
                // 穿越边界时重置轨道状态
                particle.is_orbiting = false;
            } else if particle.x > self.width {
                particle.x = 0.0;
                particle.is_orbiting = false;
            }

            if particle.y < 0.0 {
                particle.y = self.height;
                particle.is_orbiting = false;
            } else if particle.y > self.height {
                particle.y = 0.0;
                particle.is_orbiting = false;
            }
        }
    }

    pub fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn set_mouse_force(&mut self, force: f64) {
        self.mouse_force = force;
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;

        for particle in &mut self.particles {
            if particle.x > width {
                particle.x = width;
            }
            if particle.y > height {
                particle.y = height;
            }
        }
    }

    pub fn get_particles(&self) -> js_sys::Float64Array {
        let array_len = self.particles.len() * 3;
        let result = js_sys::Float64Array::new_with_length(array_len as u32);

        for (i, particle) in self.particles.iter().enumerate() {
            let base_idx = i * 3;
            result.set_index(base_idx as u32, particle.x);
            result.set_index((base_idx + 1) as u32, particle.y);
            result.set_index((base_idx + 2) as u32, particle.size);
        }

        result
    }

    pub fn get_mouse_connections(&self) -> js_sys::Float64Array {
        let mut connections = Vec::new();

        for (particle_idx, strength) in &self.mouse_connections {
            let particle = self.particles[*particle_idx];

            let dx = (particle.x - self.mouse_x).abs();
            let dy = (particle.y - self.mouse_y).abs();

            if dx > self.width / 2.0 || dy > self.height / 2.0 {
                continue;
            }

            connections.push(particle.x);
            connections.push(particle.y);
            connections.push(*strength);
        }

        let result = js_sys::Float64Array::new_with_length(connections.len() as u32);
        for (i, value) in connections.iter().enumerate() {
            result.set_index(i as u32, *value);
        }

        result
    }

    pub fn calculate_connections(&self) -> js_sys::Float64Array {
        let mut connections = Vec::new();

        for i in 0..self.particles.len() {
            let p1 = self.particles[i];

            for j in (i + 1)..self.particles.len() {
                let p2 = self.particles[j];

                let dx = (p1.x - p2.x).abs();
                let dy = (p1.y - p2.y).abs();
                // 跳过屏幕两端的粒子
                if dx > self.width / 2.0 || dy > self.height / 2.0 {
                    continue;
                }

                let distance = (dx * dx + dy * dy).sqrt();

                if distance < self.connection_distance {
                    let opacity = 1.0 - (distance / self.connection_distance);

                    let d1 = ((p1.x - self.mouse_x).powi(2) + (p1.y - self.mouse_y).powi(2)).sqrt();
                    let d2 = ((p2.x - self.mouse_x).powi(2) + (p2.y - self.mouse_y).powi(2)).sqrt();

                    let mut final_opacity = opacity;
                    if d1 < self.mouse_radius || d2 < self.mouse_radius {
                        final_opacity *= 1.3; // 稍微增强鼠标附近的连接线
                    }

                    connections.push(p1.x);
                    connections.push(p1.y);
                    connections.push(p2.x);
                    connections.push(p2.y);
                    connections.push(final_opacity);
                }
            }
        }

        let result = js_sys::Float64Array::new_with_length(connections.len() as u32);
        for (i, value) in connections.iter().enumerate() {
            result.set_index(i as u32, *value);
        }

        result
    }
}
