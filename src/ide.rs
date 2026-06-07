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

    search_text: String,
    new_file_name: String,
    delete_file_path: String,
    rename_file_name: String,
    new_folder_name: String,
}

fn read_files(folder: &str, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                files.push(path.to_string_lossy().to_string());
            } else if path.is_dir() {
                read_files(&path.to_string_lossy(), files);
            }
        }
    }
}

impl eframe::App for MyIDE {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {

        

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("My Rust IDE");
        });

        egui::SidePanel::left("explorer").show(ctx, |ui| {

            
            ui.heading("Explorer");

            ui.label("Search");
            ui.text_edit_singleline(&mut self.search_text);

            // Native folder picker button
            if ui.button("📁 Open Folder").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.folder_path = path.to_string_lossy().to_string();
                    self.files.clear();
                    read_files(&self.folder_path, &mut self.files);
                }
            }

            if ui.button("📄 New File").clicked() {
                self.output = "Create file feature coming soon".to_string();
            }

            ui.separator();
            ui.label("New File:");
            ui.text_edit_singleline(&mut self.new_file_name);

            if ui.button("Create").clicked() {
                if !self.folder_path.is_empty() && !self.new_file_name.is_empty() {
                    let path = format!(
                        "{}/{}",
                        self.folder_path,
                        self.new_file_name
                    );

                    match fs::write(&path, "") {
                        Ok(_) => {
                            self.output = format!("Created {}", self.new_file_name);
                            self.files.push(path);
                            self.new_file_name.clear();
                        }
                        Err(e) => {
                            self.output = e.to_string();
                        }
                    }
                }
            }

            ui.separator();
            ui.label("New Folder:");
            ui.text_edit_singleline(&mut self.new_folder_name);

            if ui.button("📁 Create Folder").clicked() {
                if !self.folder_path.is_empty() && !self.new_folder_name.is_empty() {
                    let path = format!(
                        "{}/{}",
                        self.folder_path,
                        self.new_folder_name
                    );

                    match fs::create_dir_all(&path) {
                        Ok(_) => {
                            self.output = format!("Created folder {}", self.new_folder_name);
                            self.new_folder_name.clear();
                        }
                        Err(e) => {
                            self.output = e.to_string();
                        }
                    }
                }
            }

            ui.separator();
            ui.label("Rename File:");
            ui.text_edit_singleline(&mut self.rename_file_name);

            if ui.button("✏ Rename").clicked() {
                if !self.file_path.is_empty() && !self.rename_file_name.is_empty() {
                    let parent = std::path::Path::new(&self.file_path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let new_path = format!("{}/{}", parent, self.rename_file_name);

                    match fs::rename(&self.file_path, &new_path) {
                        Ok(_) => {
                            self.output = format!(
                                "Renamed {} to {}",
                                self.file_path, self.rename_file_name
                            );

                            // Update tabs
                            if let Some(pos) = self.tabs.iter().position(|t| t == &self.file_path) {
                                self.tabs[pos] = new_path.clone();
                            }

                            self.file_path = new_path;
                            self.rename_file_name.clear();

                            // Refresh file list
                            self.files.clear();
                            read_files(&self.folder_path, &mut self.files);
                        }
                        Err(e) => {
                            self.output = e.to_string();
                        }
                    }
                }
            }
            
            // Render files list
            for file_path in self.files.clone() {
                if !self.search_text.is_empty() && !file_path.to_lowercase().contains(&self.search_text.to_lowercase())
                {
                    continue;
                }

                let name = file_path
                    .replace(&self.folder_path, "")
                    .replace("\\", "/");
                
                if ui.button(format!("📄 {}", name)).clicked() {
                    self.file_path = file_path.clone();
                    self.delete_file_path = file_path.clone();
                    
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

                if ui.button("Delete").clicked() {
                    if !self.file_path.is_empty() {
                        match fs::remove_file(&self.file_path) {
                            Ok(_) => {
                                self.output =
                                    format!("Deleted {}", self.file_path);

                                self.files.clear();
                                read_files(
                                    &self.folder_path,
                                    &mut self.files
                                );

                                self.code.clear();
                                self.file_path.clear();
                            }
                            Err(e) => {
                                self.output = e.to_string();
                            }
                        }
                    }
                }

                if ui.button("Cargo Build").clicked() {
                    if !self.folder_path.is_empty() {
                        match Command::new("cargo")
                            .arg("build")
                            .current_dir(&self.folder_path)
                            .output()
                        {
                            Ok(out) => {
                                self.terminal_output =
                                    String::from_utf8_lossy(&out.stderr).to_string();

                                if self.terminal_output.is_empty() {
                                    self.terminal_output =
                                        String::from_utf8_lossy(&out.stdout).to_string();
                                }
                            }
                            Err(e) => {
                                self.terminal_output = e.to_string();
                            }
                        }
                    }
                }

                if ui.button("Cargo Run").clicked() {
                    if !self.folder_path.is_empty() {
                        match Command::new("cargo")
                            .arg("run")
                            .current_dir(&self.folder_path)
                            .output()
                        {
                            Ok(out) => {
                                self.terminal_output =
                                    String::from_utf8_lossy(&out.stderr).to_string();

                                if self.terminal_output.is_empty() {
                                    self.terminal_output =
                                        String::from_utf8_lossy(&out.stdout).to_string();
                                }
                            }
                            Err(e) => {
                                self.terminal_output = e.to_string();
                            }
                        }
                    }
                }
            });

            ui.separator();
                ui.label("Output:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.output)
                        .desired_rows(3)
                );

                ui.separator();
                ui.label("Terminal:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.terminal_output)
                        .desired_rows(10)
                );
            });
    }
}