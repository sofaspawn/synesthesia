use minifb::{Key, Window, WindowOptions};
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;
use std::time::Instant;

const WIDTH: usize = 900;
const HEIGHT: usize = 900;

fn main() {
    let mut window = Window::new("synesthesia", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let mut accumulation = vec![[0.0f32; 3]; WIDTH * HEIGHT];
    let mut framebuffer = vec![0u32; WIDTH * HEIGHT];

    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let time = start.elapsed().as_secs_f32();

        // fade trails
        for px in accumulation.iter_mut() {
            px[0] *= 0.982;
            px[1] *= 0.982;
            px[2] *= 0.982;
        }

        // fake audio data for now
        let mut fft_input = vec![];

        for i in 0..256 {
            let v = ((i as f32 * 0.15 + time * 4.0).sin() * 0.5 + 0.5)
                * ((time * 2.0).sin() * 0.5 + 0.5);

            fft_input.push(Complex { re: v, im: 0.0 });
        }

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(256);
        fft.process(&mut fft_input);

        let cx = WIDTH as f32 * 0.5 + (time * 0.3).sin() * 40.0;

        let cy = HEIGHT as f32 * 0.5 + (time * 0.2).cos() * 25.0;

        for (i, c) in fft_input.iter().enumerate().take(128) {
            let mag = (c.norm() * 0.01).min(1.0);

            let angle = i as f32 / 128.0 * PI * 2.0 + time * 0.2;

            let noise =
                (angle * 9.0 + time * 2.0).sin() * 18.0 + (angle * 17.0 - time * 1.3).cos() * 10.0;

            let radius = 180.0 + mag * 220.0 + noise;

            let x = cx + angle.cos() * radius;
            let y = cy + angle.sin() * radius;

            draw_glow(
                &mut accumulation,
                x as i32,
                y as i32,
                20,
                [0.7 + 0.3 * (time * 0.3).sin(), 0.4, 1.0],
                mag * 4.0,
            );
            for j in 0..6 {
                let offset = time * (0.2 + j as f32 * 0.05);

                let small_radius = radius + (j as f32 * 18.0);

                let px = cx + (angle + offset).cos() * small_radius;

                let py = cy + (angle + offset).sin() * small_radius;

                draw_glow(
                    &mut accumulation,
                    px as i32,
                    py as i32,
                    8,
                    [1.0, 0.6, 1.0],
                    mag * 0.7,
                );
            }
        }

        for i in 0..40 {
            let t = time * 0.1 + i as f32;

            let x = cx + (t * 0.7).sin() * 300.0;

            let y = cy + (t * 0.9).cos() * 300.0;

            draw_glow(
                &mut accumulation,
                x as i32,
                y as i32,
                120,
                [0.05, 0.02, 0.08],
                0.03,
            );
        }

        for y in 1..HEIGHT {
            for x in 1..WIDTH {
                let idx = y * WIDTH + x;
                let prev = (y - 1) * WIDTH + x;

                accumulation[idx][0] += accumulation[prev][0] * 0.015;
                accumulation[idx][1] += accumulation[prev][1] * 0.015;
                accumulation[idx][2] += accumulation[prev][2] * 0.015;
            }
        }

        // tone map
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
            let glow = falloff.powf(2.5) * intensity;

            let idx = py * WIDTH + px;

            accumulation[idx][0] += color[0] * glow;
            accumulation[idx][1] += color[1] * glow;
            accumulation[idx][2] += color[2] * glow;
        }
    }
}
