#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::{
    egui::{self, Color32, ColorImage, TextureHandle, TextureOptions},
    CreationContext,
};
use image::ImageReader;
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
    u8,
};
use tokio::runtime::Runtime;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let rt = Runtime::new().expect("Failed to create runtime");
    let _guard = rt.enter();

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        })
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(cc)))
        }),
    )
}

struct MyApp {
    sender: Sender<Result>,
    receiver: Receiver<Result>,
    result: Option<Result>,
    intensity_texture: TextureHandle,
}

impl MyApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        MyApp {
            sender,
            receiver,
            result: None,
            intensity_texture: cc.egui_ctx.load_texture(
                "intensity",
                ColorImage::example(),
                TextureOptions::NEAREST,
            ),
        }
    }
}

struct Result {
    path: String,
    dim: (u32, u32),
}

/// Pattern of linear polarizers on the CCD.
///
/// +-----+-----+-----
/// | 090 | 135 | 090
/// +-----+-----+-----
/// | 045 | 000 | 045
/// +-----+-----+-----
/// | 090 | 135 | ...
///
fn req_image(path: String, sender: Sender<Result>, mut intensity_handle: TextureHandle) {
    tokio::spawn(async move {
        println!("Processing {}", path);

        // Perform image processing.
        let image = ImageReader::open(&path).expect("Failed to open image");
        let image = image.decode().expect("Failed to decode image");
        let image = image.as_luma8().expect("Failed to read image as greyscale");

        let dim = image.dimensions();
        let mut stokes: Vec<(f32, f32, f32)> = Vec::new();

        for row in (0..dim.1).filter(|x| x & 0x1 == 0) {
            for col in (0..dim.0).filter(|x| x & 0x1 == 0) {
                let i000 = image.get_pixel(col + 1, row + 1).0[0];
                let i045 = image.get_pixel(col + 0, row + 1).0[0];
                let i090 = image.get_pixel(col + 0, row + 0).0[0];
                let i135 = image.get_pixel(col + 1, row + 0).0[0];

                let norm_i000 = i000 as f32 / u8::MAX as f32;
                let norm_i045 = i045 as f32 / u8::MAX as f32;
                let norm_i090 = i090 as f32 / u8::MAX as f32;
                let norm_i135 = i135 as f32 / u8::MAX as f32;

                let s_vec = (
                    norm_i000 + norm_i090,
                    norm_i000 - norm_i090,
                    norm_i045 - norm_i135,
                );

                stokes.push(s_vec);
            }
        }

        let bytes: Vec<Color32> = stokes
            .iter()
            .map(|ref s_vec| Color32::from_gray((s_vec.0 * 255.) as u8))
            .collect();

        let width = (dim.0 / 2) as usize;
        let height = (dim.1 / 2) as usize;

        intensity_handle.set(
            ColorImage {
                size: [width, height],
                pixels: bytes.clone(),
            },
            TextureOptions::NEAREST,
        );

        // Notify GUI thread with results.
        let result = Result { path, dim };
        let _ = sender.send(result);
    });
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            req_image(
                                path.display().to_string(),
                                self.sender.clone(),
                                self.intensity_texture.clone(),
                            );
                        }
                    }
                });
            });
        });

        // Check for results from image processing.
        if let Ok(result) = self.receiver.try_recv() {
            self.result = Some(result);
        }

        egui::SidePanel::left("info").show(ctx, |ui| {
            ui.heading("Image Info");

            if let Some(result) = &self.result {
                ui.label(format!("Path: {}", result.path));
                ui.label(format!("Image Size: {:?}", result.dim));
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(result) = &self.result {
                    let uri = format!("file://{}", result.path);
                    ui.image(uri);

                    let size = self.intensity_texture.size_vec2();
                    let sized_texture =
                        egui::load::SizedTexture::new(&self.intensity_texture, size);
                    ui.add(egui::Image::new(sized_texture));
                }
            });
        });
    }
}
