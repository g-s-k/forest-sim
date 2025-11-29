use egui::Widget as _;
use rand::{Rng as _, SeedableRng as _};

pub const SIZE_X: usize = 500;
pub const SIZE_Y: usize = 500;

fn image(pixels: &[egui::Color32]) -> egui::ColorImage {
    egui::ColorImage {
        size: [SIZE_X, SIZE_Y],
        source_size: egui::Vec2 {
            x: SIZE_X as f32,
            y: SIZE_Y as f32,
        },
        pixels: pixels.to_vec(),
    }
}

fn render_log(value: f64, _: std::ops::RangeInclusive<usize>) -> String {
    format!("1e{:.03}", value.log10())
}

fn spread_chance(color: egui::Color32) -> f64 {
    if color == egui::Color32::WHITE {
        1.0
    } else if color == egui::Color32::YELLOW {
        0.5
    } else if color == egui::Color32::ORANGE {
        0.25
    } else if color == egui::Color32::RED {
        0.125
    } else if color == egui::Color32::RED.gamma_multiply(0.5) {
        0.0625
    } else if color == egui::Color32::RED.gamma_multiply(0.25) {
        0.03125
    } else {
        0.0
    }
}

pub struct App {
    speed: u8,
    counter: u8,
    grow_chance: f64,
    strike_chance: f64,
    spread_chance: f64,
    texture: egui::TextureHandle,
    pixels: Box<[egui::Color32]>,
    scratch: Box<[egui::Color32]>,
    rng: rand_xoshiro::Xoshiro128Plus,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let pixels: Box<[_]> = (0..SIZE_X * SIZE_Y)
            .map(|_| egui::Color32::TRANSPARENT)
            .collect();

        Self {
            speed: 1,
            counter: 0,
            grow_chance: 1e-4,
            strike_chance: 0.5e-6,
            spread_chance: 0.75,
            texture: cc.egui_ctx.load_texture(
                "simulation",
                image(&pixels),
                egui::TextureOptions::NEAREST,
            ),
            scratch: pixels.clone(),
            pixels,
            rng: rand_xoshiro::Xoshiro128Plus::from_os_rng(),
        }
    }

    fn clear(&mut self) {
        self.pixels.fill(egui::Color32::TRANSPARENT);
    }

    fn step(&mut self) {
        self.scratch.copy_from_slice(&self.pixels);

        let green = egui::Color32::GREEN;
        let half_green = green.gamma_multiply(0.5);
        let quarter_green = green.gamma_multiply(0.25);

        let inv_sqrt_2 = 1.0 / std::f64::consts::SQRT_2;

        // propagate fire
        let (mut x, mut y) = (0, 0);
        while {
            x += 1;
            if x >= SIZE_X {
                x = 0;
                y += 1;

                y < SIZE_Y
            } else {
                true
            }
        } {
            let row_start = y * SIZE_X;
            let center = row_start + x;
            let pixel = self.pixels[center];

            if pixel == green || pixel == half_green || pixel == quarter_green {
                let mut chance = 0f64;

                if x > 0 {
                    chance = chance.max(spread_chance(self.pixels[center - 1]));
                }

                if x + 1 < SIZE_X {
                    chance = chance.max(spread_chance(self.pixels[center + 1]));
                }

                if y > 0 {
                    let top_row_center = center - SIZE_X;

                    chance = chance.max(spread_chance(self.pixels[top_row_center]));

                    if x > 0 {
                        chance =
                            chance.max(spread_chance(self.pixels[top_row_center - 1]) * inv_sqrt_2);
                    }

                    if x + 1 < SIZE_X {
                        chance =
                            chance.max(spread_chance(self.pixels[top_row_center + 1]) * inv_sqrt_2);
                    }
                }

                if y + 1 < SIZE_Y {
                    let bottom_row_center = center + SIZE_X;

                    chance = chance.max(spread_chance(self.pixels[bottom_row_center]));

                    if x > 0 {
                        chance = chance
                            .max(spread_chance(self.pixels[bottom_row_center - 1]) * inv_sqrt_2);
                    }

                    if x + 1 < SIZE_X {
                        chance = chance
                            .max(spread_chance(self.pixels[bottom_row_center + 1]) * inv_sqrt_2);
                    }
                }

                if pixel == half_green {
                    chance *= 0.5;
                } else if pixel == quarter_green {
                    chance *= 0.25;
                }

                if self.rng.random_bool(self.spread_chance * chance)
                    || self.rng.random_bool(self.strike_chance)
                {
                    self.scratch[center] = egui::Color32::WHITE;
                } else if pixel == half_green && self.rng.random_bool(0.25) {
                    self.scratch[center] = green;
                } else if pixel == quarter_green && self.rng.random_bool(0.25) {
                    self.scratch[center] = half_green;
                }
            } else if pixel == egui::Color32::BLACK && self.rng.random_bool(0.25) {
                self.scratch[center] = egui::Color32::TRANSPARENT;
            } else if pixel == egui::Color32::GRAY && self.rng.random_bool(0.25) {
                self.scratch[center] = egui::Color32::BLACK;
            } else if pixel == egui::Color32::RED.gamma_multiply(0.25) {
                self.scratch[center] = egui::Color32::GRAY;
            } else if pixel == egui::Color32::RED.gamma_multiply(0.5) {
                self.scratch[center] = egui::Color32::RED.gamma_multiply(0.25);
            } else if pixel == egui::Color32::RED {
                self.scratch[center] = egui::Color32::RED.gamma_multiply(0.5);
            } else if pixel == egui::Color32::ORANGE {
                self.scratch[center] = egui::Color32::RED;
            } else if pixel == egui::Color32::YELLOW {
                self.scratch[center] = egui::Color32::ORANGE;
            } else if pixel == egui::Color32::WHITE {
                self.scratch[center] = egui::Color32::YELLOW;
            } else if pixel == egui::Color32::TRANSPARENT && self.rng.random_bool(self.grow_chance)
            {
                self.scratch[center] = quarter_green;
            }
        }

        std::mem::swap(&mut self.scratch, &mut self.pixels);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("settings1").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::Slider::new(&mut self.grow_chance, 1e-7..=1e-3)
                    .logarithmic(true)
                    .custom_formatter(render_log)
                    .text("grow")
                    .ui(ui);
                egui::widgets::Slider::new(&mut self.strike_chance, 1e-12..=1e-4)
                    .logarithmic(true)
                    .custom_formatter(render_log)
                    .text("strike")
                    .ui(ui);
            });
        });

        egui::TopBottomPanel::top("settings2").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::Slider::new(&mut self.spread_chance, 0.5..=0.999999)
                    .logarithmic(true)
                    .custom_formatter(|value, _| format!("{value:.06}"))
                    .text("spread")
                    .ui(ui);
                #[allow(clippy::reversed_empty_ranges)]
                egui::widgets::Slider::new(&mut self.speed, 10..=1)
                    .custom_formatter(|value, _| format!("1/{value}"))
                    .text("speed")
                    .ui(ui);

                if ui.button("clear").clicked() {
                    self.clear();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.texture
                .set(image(&self.pixels), egui::TextureOptions::NEAREST);

            let size = self.texture.size_vec2();
            let sized_texture = egui::load::SizedTexture::new(&self.texture, size);
            ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));

            self.counter += 1;
            if self.counter >= self.speed {
                self.counter = 0;
                self.step();
            }

            ui.ctx().request_repaint();
        });
    }
}
