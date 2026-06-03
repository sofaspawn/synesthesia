use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;
use std::time::Instant;

const WIDTH: usize = 900;
const HEIGHT: usize = 900;

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    size: i32,
    hue: f32,
}

fn main() {
    let mut window = Window::new("synesthesia", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let mut accumulation = vec![[0.0f32; 3]; WIDTH * HEIGHT];
    let mut framebuffer = vec![0u32; WIDTH * HEIGHT];

    let mut particles: Vec<Particle> = Vec::new();

    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let time = start.elapsed().as_secs_f32();

        for px in accumulation.iter_mut() {
            px[0] *= 0.985;
            px[1] *= 0.985;
            px[2] *= 0.985;
        }

        let mut fft_input = vec![];

        for i in 0..256 {
            let v = ((i as f32 * 0.15 + time * 3.5).sin() * 0.5 + 0.5)
                * ((time * 1.7).sin() * 0.5 + 0.5);

            fft_input.push(Complex { re: v, im: 0.0 });
        }

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(256);
        fft.process(&mut fft_input);

        let bass = fft_input[3].norm() * 0.02;
        let mids = fft_input[20].norm() * 0.01;

        let cx = WIDTH as f32 * 0.5 + (time * 0.17).sin() * 70.0 + (time * 0.041).cos() * 40.0;

        let cy = HEIGHT as f32 * 0.5 + (time * 0.13).cos() * 55.0 + (time * 0.031).sin() * 30.0;

        for i in 0..60 {
            let t = time * 0.08 + i as f32 * 0.7;

            let x = cx + (t * 0.9).sin() * 350.0;
            let y = cy + (t * 1.2).cos() * 320.0;

            let intensity = 0.01 + bass * 0.05;

            draw_glow(
                &mut accumulation,
                x as i32,
                y as i32,
                140,
                [0.03, 0.015, 0.06],
                intensity,
            );
        }

        for (i, c) in fft_input.iter().enumerate().take(128) {
            let mag = (c.norm() * 0.01).min(1.0);

            let phase = (time * 0.11).sin() * 0.8;

            let angle = i as f32 / 128.0 * PI * 2.0 + time * 0.12 + phase + mag * 0.5;

            let noise = (angle * 8.0 + time * 1.3).sin() * 25.0
                + (angle * 19.0 - time * 0.7).cos() * 15.0
                + (angle * 37.0 + time * 0.2).sin() * 8.0;

            let radius = 170.0 + mag * 260.0 + noise + bass * 120.0;

            let x = cx + angle.cos() * radius;
            let y = cy + angle.sin() * radius;

            let hue_shift = (time * 0.07 + i as f32 * 0.01).sin() * 0.2;

            let color = [0.7 + hue_shift, 0.3 + mids * 0.5, 1.0];

            draw_glow(
                &mut accumulation,
                x as i32,
                y as i32,
                24,
                color,
                mag * 4.5 + bass * 3.0,
            );

            for j in 0..8 {
                let offset = time * (0.15 + j as f32 * 0.04);

                let small_radius = radius + (j as f32 * 16.0);

                let px = cx + (angle + offset).cos() * small_radius;

                let py = cy + (angle + offset).sin() * small_radius;

                let sparkle = (time * 5.0 + j as f32).sin() * 0.5 + 0.5;

                draw_glow(
                    &mut accumulation,
                    px as i32,
                    py as i32,
                    7 + (sparkle * 6.0) as i32,
                    [1.0, 0.5 + sparkle * 0.3, 1.0],
                    mag * 0.8,
                );
            }
        }

        let mut rng = rand::thread_rng();

        for _ in 0..12 {
            let a = rng.gen::<f32>() * PI * 2.0;
            let r = rng.gen::<f32>() * 140.0;

            particles.push(Particle {
                x: cx + a.cos() * r,
                y: cy + a.sin() * r,

                vx: a.cos() * (0.5 + rng.gen::<f32>() * 1.5),
                vy: a.sin() * (0.5 + rng.gen::<f32>() * 1.5),

                life: 1.0,

                size: rng.gen_range(4..12),

                hue: rng.gen::<f32>(),
            });
        }

        particles.retain_mut(|p| {
            p.life *= 0.992;

            let flow_x = ((p.y * 0.008) + time * 0.7).sin();

            let flow_y = ((p.x * 0.008) - time * 0.5).cos();

            p.vx += flow_x * 0.03;
            p.vy += flow_y * 0.03;

            p.vx *= 0.995;
            p.vy *= 0.995;

            p.x += p.vx;
            p.y += p.vy;

            let pulse = (time * 4.0 + p.hue * 10.0).sin() * 0.5 + 0.5;

            let color = [0.8 + pulse * 0.2, 0.4 + pulse * 0.2, 1.0];

            draw_glow(
                &mut accumulation,
                p.x as i32,
                p.y as i32,
                p.size,
                color,
                p.life * 0.5,
            );

            p.life > 0.03
        });

        for y in 1..HEIGHT {
            for x in 1..WIDTH {
                let idx = y * WIDTH + x;

                let up = (y - 1) * WIDTH + x;

                accumulation[idx][0] += accumulation[up][0] * 0.02;
                accumulation[idx][1] += accumulation[up][1] * 0.02;
                accumulation[idx][2] += accumulation[up][2] * 0.02;
            }
        }

        for y in 0..HEIGHT {
            for x in 1..WIDTH - 1 {
                let idx = y * WIDTH + x;

                let left = y * WIDTH + (x - 1);
                let right = y * WIDTH + (x + 1);

                accumulation[idx][0] += (accumulation[left][0] + accumulation[right][0]) * 0.003;

                accumulation[idx][1] += (accumulation[left][1] + accumulation[right][1]) * 0.003;

                accumulation[idx][2] += (accumulation[left][2] + accumulation[right][2]) * 0.003;
            }
        }

        for i in 0..framebuffer.len() {
            let r = tone_map(accumulation[i][0]);
            let g = tone_map(accumulation[i][1]);
            let b = tone_map(accumulation[i][2]);

            framebuffer[i] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }

        window
            .update_with_buffer(&framebuffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn tone_map(v: f32) -> u8 {
    let mapped = 1.0 - (-v).exp();
    (mapped * 255.0) as u8
}

fn draw_glow(
    accumulation: &mut [[f32; 3]],
    cx: i32,
    cy: i32,
    radius: i32,
    color: [f32; 3],
    intensity: f32,
) {
    for y in -radius..=radius {
        for x in -radius..=radius {
            let px = cx + x;
            let py = cy + y;

            if px < 0 || py < 0 {
                continue;
            }

            let px = px as usize;
            let py = py as usize;

            if px >= WIDTH || py >= HEIGHT {
                continue;
            }

            let dist = ((x * x + y * y) as f32).sqrt();

            if dist > radius as f32 {
                continue;
            }

            let falloff = 1.0 - dist / radius as f32;

            let glow = falloff.powf(3.0) * intensity;

            let idx = py * WIDTH + px;

            accumulation[idx][0] += color[0] * glow;
            accumulation[idx][1] += color[1] * glow;
            accumulation[idx][2] += color[2] * glow;
        }
    }
}
