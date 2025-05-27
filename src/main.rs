#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::{egui, egui::Image};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
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
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    image_path: Option<String>,

    sender: Sender<Result>,
    receiver: Receiver<Result>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        MyApp {
            image_path: None,
            sender,
            receiver,
        }
    }
}

struct Result {}

fn req_image() {
    tokio::spawn(async move {
        // Perform image processing.

        // Notify GUI thread with results.
    });
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.image_path = Some(path.display().to_string());

                            req_image();
                        }
                    }
                });
            });
        });

        // Check for results from image processing.

        egui::SidePanel::left("info").show(ctx, |ui| {
            ui.heading("Image Info");

            if let Some(path) = &self.image_path {
                ui.label(format!("Path: {}", path));
                ui.label(format!("Image Size: {}x{}", 0, 0));
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(path) = &self.image_path {
                let uri = format!("file://{}", path);
                ui.image(uri);
            }
        });
    }
}
