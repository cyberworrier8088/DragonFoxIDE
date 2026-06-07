use eframe::egui;
use std::fs;
use rfd::FileDialog;
use std::process::Command;

pub fn ide() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "My Rust IDE",
        options,
        Box::new(|_cc| Ok(Box::new(MyIDE::default()))),
    )
}

#[derive(Default)]
struct MyIDE {
    code: String,
    file_path: String,
    output: String,
    folder_path: String,
    files: Vec<String>,

    tabs: Vec<String>,
    current_tab: usize,


    terminal_output: String,
}

impl eframe::App for MyIDE {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {

        

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("My Rust IDE");
        });

        egui::SidePanel::left("explorer").show(ctx, |ui| {
            
            ui.heading("Explorer");

            // Native folder picker button
            if ui.button("📁 Open Folder").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.folder_path = path.to_string_lossy().to_string();
                    self.files.clear();
                    
                    if let Ok(entries) = fs::read_dir(&self.folder_path) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                self.files.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }

            ui.separator();
            
            // Render files list
            for file_path in self.files.clone() {
                let path = std::path::Path::new(&file_path);
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                
                if ui.button(format!("📄 {}", name)).clicked() {
                    self.file_path = file_path.clone();
                    
                    if !self.tabs.contains(&file_path) {
                        self.tabs.push(file_path.clone());
                    }

                    match fs::read_to_string(&file_path) {
                        Ok(content) => {
                            self.code = content;
                            self.output = format!("Opened: {}", name);
                        }
                        Err(e) => self.output = format!("Error: {}", e),
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Tabs
            let mut tab_to_remove = None;
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let name = std::path::Path::new(tab)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();

                    if ui.button(name.as_ref()).clicked() {
                        self.current_tab = index;

                        if let Ok(content) = fs::read_to_string(tab) {
                            self.code = content;
                            self.file_path = tab.clone();
                        }
                    }

                    if ui.small_button("✖").clicked() {
                        tab_to_remove = Some(index);
                    }
                }
            });

            if let Some(index) = tab_to_remove {
                self.tabs.remove(index);
                if self.current_tab >= self.tabs.len() {
                    self.current_tab = self.tabs.len().saturating_sub(1);
                }
            }

            ui.separator();

            // Editor
            ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .desired_rows(30)
                    .code_editor()
            );

            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut self.file_path);

                if ui.button("Save").clicked() {
                    match fs::write(&self.file_path, &self.code) {
                        Ok(_) => self.output = "File saved successfully!".to_string(),
                        Err(e) => self.output = format!("Error: {}", e),
                    }
                }
            });

            ui.separator();
            if ui.button("Cargo Check").clicked() {
                match Command::new("cargo").arg("check").output() {
                    Ok(out) => {
                        self.terminal_output = String::from_utf8_lossy(&out.stdout).to_string();
                    }
                    Err(e) => {
                        self.terminal_output = e.to_string();
                    }
                }
}
        });
    }
}