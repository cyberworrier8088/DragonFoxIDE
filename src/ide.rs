use eframe::egui;
use std::fs;
use rfd::FileDialog;
use std::process::Command;
#[allow(unused_imports)]
use serde::{Serialize, Deserialize};

pub fn ide() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DragonFox IDE")
            .with_inner_size([1400.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DragonFox IDE",
        options,
        Box::new(|cc| {
            apply_custom_theme(&cc.egui_ctx);
            Ok(Box::new(MyIDE::default()))
        }),
    )
}

fn apply_custom_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    // VS Code dark theme colors
    let bg_editor = egui::Color32::from_rgb(30, 30, 30);        // #1e1e1e
    let bg_sidebar = egui::Color32::from_rgb(37, 37, 38);       // #252526
    let bg_activitybar = egui::Color32::from_rgb(51, 51, 51);   // #333333
    let border_color = egui::Color32::from_rgb(60, 60, 60);     // #3c3c3c
    let selection = egui::Color32::from_rgb(14, 99, 156);       // #0e639c
    let text_color = egui::Color32::from_rgb(212, 212, 212);    // #d4d4d4
    let text_dim = egui::Color32::from_rgb(128, 128, 128);      // #808080
    let _accent_blue = egui::Color32::from_rgb(0, 122, 204);    // #007acc

    visuals.panel_fill = bg_editor;
    visuals.window_fill = bg_sidebar;
    visuals.extreme_bg_color = egui::Color32::from_rgb(25, 25, 25);
    visuals.faint_bg_color = bg_sidebar;

    visuals.widgets.noninteractive.bg_fill = bg_sidebar;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_color);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(0.5, border_color);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(2);

    visuals.widgets.inactive.bg_fill = bg_activitybar;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_dim);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.0, border_color);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(3);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(62, 62, 65);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(0.5, egui::Color32::from_rgb(80, 80, 80));
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(3);

    visuals.widgets.active.bg_fill = selection;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(3);

    visuals.selection.bg_fill = selection;
    visuals.selection.stroke = egui::Stroke::new(0.0, egui::Color32::TRANSPARENT);

    visuals.window_corner_radius = egui::CornerRadius::same(4);
    visuals.window_stroke = egui::Stroke::new(1.0, border_color);

    style.visuals = visuals;

    // Spacing & layout
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);

    ctx.set_style(style);
}

use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
struct FileNode {
    name: String,
    path: String,
    is_dir: bool,
    children: Vec<FileNode>,
}

impl FileNode {
    fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        if self.name.to_lowercase().contains(&query.to_lowercase()) {
            return true;
        }
        if self.is_dir {
            for child in &self.children {
                if child.matches_search(query) {
                    return true;
                }
            }
        }
        false
    }
}

fn build_file_tree(folder: &str) -> Vec<FileNode> {
    let mut nodes = Vec::new();
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Skip hidden directories & target folder
            if name.starts_with('.') || name == "target" {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            let is_dir = path.is_dir();

            let children = if is_dir {
                build_file_tree(&path_str)
            } else {
                Vec::new()
            };

            nodes.push(FileNode {
                name,
                path: path_str,
                is_dir,
                children,
            });
        }
    }
    // Sort directories first, then files alphabetically
    nodes.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });
    nodes
}

fn get_flat_files(nodes: &[FileNode], list: &mut Vec<FileNode>) {
    for node in nodes {
        if node.is_dir {
            get_flat_files(&node.children, list);
        } else {
            list.push(node.clone());
        }
    }
}

fn short_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn file_language(path: &str) -> &'static str {
    if path.ends_with(".rs") { "Rust" }
    else if path.ends_with(".toml") { "TOML" }
    else if path.ends_with(".md") { "Markdown" }
    else if path.ends_with(".json") { "JSON" }
    else if path.ends_with(".html") { "HTML" }
    else if path.ends_with(".css") { "CSS" }
    else if path.ends_with(".js") { "JavaScript" }
    else if path.ends_with(".ts") { "TypeScript" }
    else if path.ends_with(".py") { "Python" }
    else { "Plain Text" }
}

fn parse_outline(code: &str, path: &str) -> Vec<SymbolInfo> {
    let mut symbols = Vec::new();
    for (line_index, line) in code.lines().enumerate() {
        let trimmed = line.trim_start();
        let prefix = if path.ends_with(".rs") {
            ["fn ", "pub fn ", "struct ", "pub struct ", "enum ", "pub enum ", "trait ", "impl "].as_slice()
        } else if path.ends_with(".py") {
            ["def ", "class "].as_slice()
        } else if path.ends_with(".js") || path.ends_with(".ts") {
            ["function ", "class ", "const ", "let "].as_slice()
        } else {
            &[]
        };

        for item in prefix {
            if let Some(rest) = trimmed.strip_prefix(item) {
                let name = rest
                    .split(|c: char| c == '(' || c == '<' || c == '{' || c == ':' || c == '=' || c.is_whitespace())
                    .find(|part| !part.is_empty())
                    .unwrap_or(rest)
                    .to_string();
                let kind = item.trim().replace("pub ", "");
                symbols.push(SymbolInfo {
                    name,
                    kind,
                    line: line_index + 1,
                });
                break;
            }
        }
    }
    symbols
}

fn parse_cargo_diagnostics(output: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut current_level = String::new();
    let mut current_message = String::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("error") {
            current_level = "error".to_string();
            current_message = rest.trim_start_matches(':').trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("warning") {
            current_level = "warning".to_string();
            current_message = rest.trim_start_matches(':').trim().to_string();
        } else if let Some(location) = trimmed.strip_prefix("-->") {
            let parts: Vec<&str> = location.trim().rsplitn(3, ':').collect();
            if parts.len() >= 3 {
                let line_num = parts[1].parse::<usize>().unwrap_or(0);
                let file = parts[2].to_string();
                diagnostics.push(Diagnostic {
                    file,
                    line: line_num,
                    level: current_level.clone(),
                    message: current_message.clone(),
                });
            }
        }
    }

    diagnostics
}

fn byte_index_for_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}

fn slice_char_range(text: &str, start: usize, end: usize) -> String {
    let start_byte = byte_index_for_char(text, start);
    let end_byte = byte_index_for_char(text, end);
    text.get(start_byte..end_byte).unwrap_or("").to_string()
}

fn is_rust_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '!'
}

fn char_index_for_line(text: &str, target_line: usize) -> usize {
    if target_line <= 1 {
        return 0;
    }

    let mut line = 1usize;
    for (index, c) in text.chars().enumerate() {
        if c == '\n' {
            line += 1;
            if line == target_line {
                return index + 1;
            }
        }
    }
    text.chars().count()
}

fn rust_hover_info() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("Vec", "Vec<T>\n\nGrowable contiguous array type."),
        ("String", "String\n\nGrowable UTF-8 encoded string type."),
        ("Option", "Option<T>\n\nRepresents either Some(T) or None."),
        ("Result", "Result<T, E>\n\nRepresents success with Ok(T) or failure with Err(E)."),
        ("println!", "println!(...)\n\nPrints formatted text to standard output with a newline."),
        ("print!", "print!(...)\n\nPrints formatted text to standard output without a newline."),
        ("eprintln!", "eprintln!(...)\n\nPrints formatted text to standard error with a newline."),
        ("fn", "fn\n\nDeclares a function item or function pointer type."),
        ("struct", "struct\n\nDefines a custom data type with named or tuple fields."),
        ("impl", "impl\n\nAdds associated functions, methods, or trait implementations."),
        ("enum", "enum\n\nDefines a type with one of several possible variants."),
        ("trait", "trait\n\nDefines shared behavior that types can implement."),
    ])
}

fn render_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    expanded_paths: &mut HashSet<String>,
    selected_file: &mut String,
    delete_file: &mut String,
    tabs: &mut Vec<String>,
    code: &mut String,
    output: &mut String,
    search_text: &str,
) {
    if !node.matches_search(search_text) {
        return;
    }

    if node.is_dir {
        let is_expanded = expanded_paths.contains(&node.path);
        let icon = if is_expanded { "▾ 📁" } else { "▸ 📁" };

        let label = egui::RichText::new(format!("{} {}", icon, node.name))
            .color(egui::Color32::from_rgb(212, 212, 212));
        if ui.selectable_label(is_expanded, label).clicked() {
            if is_expanded {
                expanded_paths.remove(&node.path);
            } else {
                expanded_paths.insert(node.path.clone());
            }
        }

        if is_expanded {
            ui.indent(node.path.clone(), |ui| {
                for child in &node.children {
                    render_node(
                        ui,
                        child,
                        expanded_paths,
                        selected_file,
                        delete_file,
                        tabs,
                        code,
                        output,
                        search_text,
                    );
                }
            });
        }
    } else {
        let is_selected = selected_file == &node.path;
        let file_icon = get_file_icon(&node.name);
        let label = egui::RichText::new(format!("{} {}", file_icon, node.name))
            .color(if is_selected {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(200, 200, 200)
            });

        if ui.selectable_label(is_selected, label).clicked() {
            *selected_file = node.path.clone();
            *delete_file = node.path.clone();
            if !tabs.contains(&node.path) {
                tabs.push(node.path.clone());
            }
            match fs::read_to_string(&node.path) {
                Ok(content) => {
                    *code = content;
                    *output = format!("Opened: {}", node.name);
                }
                Err(e) => {
                    *output = format!("Error: {}", e);
                }
            }
        }
    }
}

fn get_file_icon(name: &str) -> &'static str {
    if name.ends_with(".rs") { "🦀" }
    else if name.ends_with(".toml") { "⚙" }
    else if name.ends_with(".md") { "📝" }
    else if name.ends_with(".json") { "📋" }
    else if name.ends_with(".html") { "🌐" }
    else if name.ends_with(".css") { "🎨" }
    else if name.ends_with(".js") || name.ends_with(".ts") { "📜" }
    else if name.ends_with(".py") { "🐍" }
    else if name.ends_with(".lock") { "🔒" }
    else if name.ends_with(".gitignore") { "👁" }
    else { "📄" }
}

/// Extract code blocks from AI markdown response
fn extract_code_blocks(response: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_block = String::new();

    for line in response.lines() {
        if line.trim().starts_with("```") {
            if in_block {
                // End of block
                blocks.push(current_block.clone());
                current_block.clear();
                in_block = false;
            } else {
                // Start of block
                in_block = true;
                current_block.clear();
            }
        } else if in_block {
            if !current_block.is_empty() {
                current_block.push('\n');
            }
            current_block.push_str(line);
        }
    }

    blocks
}

#[derive(PartialEq, Clone, Copy)]
enum SidebarTab {
    Explorer,
    Search,
    Outline,
    AI,
    Git,
    Settings,
}

#[derive(PartialEq, Clone, Copy)]
enum BottomTab {
    Output,
    Terminal,
    Problems,
}

#[derive(Clone, Default)]
struct Diagnostic {
    file: String,
    line: usize,
    level: String,
    message: String,
}

#[derive(Clone, Default)]
struct SymbolInfo {
    name: String,
    kind: String,
    line: usize,
}

#[derive(Clone)]
struct CompletionItem {
    label: String,
    insert: String,
}

#[derive(Clone, Copy, PartialEq)]
enum AiApplyMode {
    Chat,
    ReplaceEditor,
}

#[derive(Default)]
struct MyIDE {
    code: String,
    file_path: String,
    output: String,
    folder_path: String,
    file_tree: Vec<FileNode>,
    expanded_paths: HashSet<String>,

    tabs: Vec<String>,
    current_tab: usize,

    terminal_output: String,
    terminal_command: String,
    diagnostics: Vec<Diagnostic>,
    git_output: String,
    git_commit_message: String,

    search_text: String,
    new_file_name: String,
    delete_file_path: String,
    rename_file_name: String,
    new_folder_name: String,
    show_full_paths: bool,

    is_running_cargo: bool,
    cargo_receiver: Option<std::sync::mpsc::Receiver<String>>,

    ai_prompt: String,
    ai_response: String,
    ai_api_key: String,
    is_asking_ai: bool,
    ai_receiver: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    ai_apply_mode: AiApplyMode,
    ai_context_bundle: String,

    // VS Code layout state
    active_sidebar_tab: Option<SidebarTab>,
    sidebar_visible: bool,
    active_bottom_tab: BottomTab,
    bottom_panel_visible: bool,

    // Cursor tracking
    cursor_row: usize,
    cursor_col: usize,
    cursor_index: usize,

    // AI model
    ai_model: String,

    // Global search
    global_search_query: String,
    global_search_results: Vec<(String, String)>, // (file_path, matched_line)

    // Editor intelligence
    saved_code: String,
    outline_symbols: Vec<SymbolInfo>,
    command_palette_open: bool,
    command_palette_query: String,
    selected_text: String,
    completion_visible: bool,
    pending_cursor_index: Option<usize>,
}

impl Default for AiApplyMode {
    fn default() -> Self {
        AiApplyMode::Chat
    }
}

impl Default for SidebarTab {
    fn default() -> Self {
        SidebarTab::Explorer
    }
}

impl Default for BottomTab {
    fn default() -> Self {
        BottomTab::Output
    }
}

impl MyIDE {
    fn default() -> Self {
        MyIDE {
            sidebar_visible: true,
            active_sidebar_tab: Some(SidebarTab::Explorer),
            bottom_panel_visible: true,
            active_bottom_tab: BottomTab::Output,
            ai_model: "qwen/qwen3-32b".to_string(),
            cursor_row: 1,
            cursor_col: 1,
            cursor_index: 0,
            ..Default::default()
        }
    }
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatCompletionChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

fn call_ai_api(api_key: &str, prompt: &str, model: &str) -> Result<String, String> {
    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let url = "https://ai.hackclub.com/proxy/v1/chat/completions";
    let auth_header = format!("Bearer {}", api_key);

    let response = ureq::post(url)
        .set("Authorization", &auth_header)
        .set("Content-Type", "application/json")
        .send_json(serde_json::to_value(request_body).map_err(|e| e.to_string())?);

    match response {
        Ok(res) => {
            let res_json: ChatCompletionResponse = res.into_json().map_err(|e| e.to_string())?;
            if let Some(choice) = res_json.choices.first() {
                Ok(choice.message.content.clone())
            } else {
                Err("No choices returned from AI completions API".to_string())
            }
        }
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            Err(format!("HTTP Error Code {}: {}", code, body))
        }
        Err(e) => Err(e.to_string()),
    }
}

impl MyIDE {
    fn open_file(&mut self, path: String) {
        let name = short_path(&path);
        self.file_path = path.clone();
        self.delete_file_path = path.clone();
        if !self.tabs.contains(&path) {
            self.tabs.push(path.clone());
        }
        self.current_tab = self.tabs.iter().position(|tab| tab == &path).unwrap_or(0);
        match fs::read_to_string(&path) {
            Ok(content) => {
                self.code = content.clone();
                self.saved_code = content;
                self.outline_symbols = parse_outline(&self.code, &self.file_path);
                self.output = format!("Opened: {}", name);
            }
            Err(e) => {
                self.output = format!("Error: {}", e);
            }
        }
    }

    fn save_current_file(&mut self) {
        if self.file_path.is_empty() {
            self.output = "No file is open.".to_string();
            return;
        }
        match fs::write(&self.file_path, &self.code) {
            Ok(_) => {
                self.saved_code = self.code.clone();
                self.outline_symbols = parse_outline(&self.code, &self.file_path);
                self.output = "File saved successfully.".to_string();
            }
            Err(e) => self.output = format!("Error: {}", e),
        }
    }

    fn refresh_workspace(&mut self) {
        if !self.folder_path.is_empty() {
            self.file_tree = build_file_tree(&self.folder_path);
            self.output = "Workspace refreshed.".to_string();
        }
    }

    fn active_ai_context(&self, task: &str) -> String {
        let selected = if self.file_path.is_empty() { "(no file selected)" } else { &self.file_path };
        format!(
            "You are DragonFox IDE's coding assistant.\nTask: {}\nWorkspace: {}\nActive file: {}\nLanguage: {}\nExtra user-pinned context:\n{}\nTerminal output:\n```\n{}\n```\n\nActive file contents:\n```{}\n{}\n```",
            task,
            if self.folder_path.is_empty() { "(no folder open)" } else { &self.folder_path },
            selected,
            file_language(selected),
            if self.ai_context_bundle.is_empty() { "(none)" } else { &self.ai_context_bundle },
            self.terminal_output,
            file_language(selected).to_lowercase(),
            self.code
        )
    }

    fn diagnostics_context(&self) -> String {
        if self.diagnostics.is_empty() {
            return "No parsed diagnostics. Use the raw terminal output if build failed.".to_string();
        }

        self.diagnostics
            .iter()
            .map(|d| format!("{}:{} [{}] {}", d.file, d.line, d.level, d.message))
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn workspace_context(&self) -> String {
        if self.folder_path.is_empty() {
            return "(no workspace open)".to_string();
        }

        let mut flat_files = Vec::new();
        get_flat_files(&self.file_tree, &mut flat_files);

        let mut context = String::new();
        let mut bytes_used = 0usize;
        let byte_limit = 80_000usize;

        for file in flat_files {
            let is_code = file.path.ends_with(".rs")
                || file.path.ends_with(".toml")
                || file.path.ends_with(".md")
                || file.path.ends_with(".js")
                || file.path.ends_with(".ts")
                || file.path.ends_with(".py");
            if !is_code {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&file.path) {
                let entry = format!("\n\n--- FILE: {} ---\n{}", file.path, content);
                bytes_used += entry.len();
                if bytes_used > byte_limit {
                    context.push_str("\n\n--- Context truncated to keep the AI request manageable. ---");
                    break;
                }
                context.push_str(&entry);
            }
        }

        if context.is_empty() {
            "(no readable source files found)".to_string()
        } else {
            context
        }
    }

    fn add_ai_context_section(&mut self, title: &str, body: String) {
        if !self.ai_context_bundle.is_empty() {
            self.ai_context_bundle.push_str("\n\n");
        }
        self.ai_context_bundle.push_str(&format!("--- {} ---\n{}", title, body));
        self.output = format!("Added {} to AI context.", title);
    }

    fn selected_or_current_code(&self) -> String {
        if !self.selected_text.trim().is_empty() {
            self.selected_text.clone()
        } else {
            self.code.clone()
        }
    }

    fn run_command_palette_action(&mut self, action: &str, ctx: &egui::Context) {
        match action {
            "Open Folder" => {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.folder_path = path.to_string_lossy().to_string();
                    self.refresh_workspace();
                }
            }
            "Save File" => self.save_current_file(),
            "Build Workspace" => self.run_cargo("build", ctx),
            "Run Workspace" => self.run_cargo("run", ctx),
            "Toggle Sidebar" => self.sidebar_visible = !self.sidebar_visible,
            "Toggle Bottom Panel" => self.bottom_panel_visible = !self.bottom_panel_visible,
            "Ask AI About File" => {
                let prompt = self.active_ai_context("Review the active file and suggest the most useful improvements.");
                self.trigger_ai_query(prompt, ctx.clone());
                self.active_sidebar_tab = Some(SidebarTab::AI);
                self.sidebar_visible = true;
            }
            "Fix Build Errors" => {
                self.fix_build_errors(ctx);
                self.active_sidebar_tab = Some(SidebarTab::AI);
                self.sidebar_visible = true;
            }
            "Project-Wide AI" => {
                let prompt = format!(
                    "{}\n\nProject-wide context:\n{}",
                    self.active_ai_context("Answer using the whole workspace context."),
                    self.workspace_context()
                );
                self.trigger_ai_query(prompt, ctx.clone());
                self.active_sidebar_tab = Some(SidebarTab::AI);
                self.sidebar_visible = true;
            }
            "Go To Definition" => self.go_to_definition(),
            _ => {}
        }
        self.command_palette_open = false;
        self.command_palette_query.clear();
    }

    fn trigger_ai_query(&mut self, prompt: String, ctx: egui::Context) {
        self.trigger_ai_query_with_mode(prompt, ctx, AiApplyMode::Chat);
    }

    fn trigger_ai_query_with_mode(&mut self, prompt: String, ctx: egui::Context, mode: AiApplyMode) {
        if prompt.is_empty() {
            self.ai_response = "Prompt is empty".to_string();
            return;
        }
        if self.ai_api_key.trim().is_empty() {
            self.ai_response = "Add your Hack Club AI proxy API key in Settings before asking AI.".to_string();
            return;
        }
        let api_key = self.ai_api_key.clone();
        let model = self.ai_model.clone();

        let (sender, receiver) = std::sync::mpsc::channel();
        self.ai_receiver = Some(receiver);
        self.is_asking_ai = true;
        self.ai_apply_mode = mode;

        std::thread::spawn(move || {
            let res = call_ai_api(&api_key, &prompt, &model);
            let _ = sender.send(res);
            ctx.request_repaint();
        });
    }

    fn run_global_search(&mut self) {
        self.global_search_results.clear();
        if self.global_search_query.is_empty() || self.folder_path.is_empty() {
            return;
        }
        let query = self.global_search_query.to_lowercase();
        let mut flat_files = Vec::new();
        get_flat_files(&self.file_tree, &mut flat_files);

        for file_node in &flat_files {
            if let Ok(content) = fs::read_to_string(&file_node.path) {
                for (i, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&query) {
                        self.global_search_results.push((
                            format!("{}:{}", file_node.path, i + 1),
                            line.trim().to_string(),
                        ));
                    }
                    if self.global_search_results.len() >= 200 {
                        return;
                    }
                }
            }
        }
    }
}

impl eframe::App for MyIDE {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // ── Poll background channels ──────────────────────
        if let Some(ref rx) = self.cargo_receiver {
            if let Ok(output) = rx.try_recv() {
                self.diagnostics = parse_cargo_diagnostics(&output);
                self.terminal_output = output;
                self.is_running_cargo = false;
                self.cargo_receiver = None;
            }
        }

        if let Some(ref rx) = self.ai_receiver {
            if let Ok(result) = rx.try_recv() {
                self.is_asking_ai = false;
                self.ai_receiver = None;
                match result {
                    Ok(response) => {
                        if self.ai_apply_mode == AiApplyMode::ReplaceEditor {
                            let blocks = extract_code_blocks(&response);
                            let replacement = blocks.first().cloned().unwrap_or_else(|| response.clone());
                            self.code = replacement;
                            self.outline_symbols = parse_outline(&self.code, &self.file_path);
                            self.ai_response = response;
                            self.output = "AI applied an inline edit to the editor buffer.".to_string();
                            self.ai_apply_mode = AiApplyMode::Chat;
                        } else {
                            self.ai_response = response;
                        }
                    }
                    Err(err) => {
                        self.ai_response = format!("Error:\n\n{}", err);
                        self.ai_apply_mode = AiApplyMode::Chat;
                    }
                }
            }
        }

        // ── Colors ────────────────────────────────────────
        let bg_activitybar = egui::Color32::from_rgb(51, 51, 51);
        let bg_sidebar = egui::Color32::from_rgb(37, 37, 38);
        let accent_blue = egui::Color32::from_rgb(0, 122, 204);
        let status_bg = egui::Color32::from_rgb(0, 122, 204);
        let text_color = egui::Color32::from_rgb(212, 212, 212);
        let text_dim = egui::Color32::from_rgb(128, 128, 128);
        let tab_active_bg = egui::Color32::from_rgb(30, 30, 30);
        let tab_inactive_bg = egui::Color32::from_rgb(45, 45, 45);
        let border_color = egui::Color32::from_rgb(60, 60, 60);

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::P)) {
            self.command_palette_open = true;
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_current_file();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::K)) {
            self.improve_code_inline(ctx);
            self.active_sidebar_tab = Some(SidebarTab::AI);
            self.sidebar_visible = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.run_cargo("run", ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            self.go_to_definition();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.command_palette_open = false;
        }

        if self.command_palette_open {
            let actions = [
                "Open Folder",
                "Save File",
                "Build Workspace",
                "Run Workspace",
                "Ask AI About File",
                "Go To Definition",
                "Fix Build Errors",
                "Project-Wide AI",
                "Toggle Sidebar",
                "Toggle Bottom Panel",
            ];
            egui::Window::new("Command Palette")
                .collapsible(false)
                .resizable(false)
                .default_width(520.0)
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 64.0))
                .show(ctx, |ui| {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.command_palette_query)
                            .hint_text("Type a command...")
                            .desired_width(f32::INFINITY)
                    );
                    response.request_focus();
                    ui.separator();

                    let query = self.command_palette_query.to_lowercase();
                    for action in actions {
                        if !query.is_empty() && !action.to_lowercase().contains(&query) {
                            continue;
                        }
                        let clicked = ui.selectable_label(false, action).clicked();
                        let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if clicked || enter {
                            self.run_command_palette_action(action, ctx);
                            break;
                        }
                    }
                });
        }

        // ══════════════════════════════════════════════════
        // STATUS BAR (bottom)
        // ══════════════════════════════════════════════════
        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(22.0)
            .frame(egui::Frame::new()
                .fill(status_bg)
                .inner_margin(egui::Margin::symmetric(8, 2)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;

                    // Branch icon
                    ui.label(egui::RichText::new("⎇ main").size(11.0).color(egui::Color32::WHITE));

                    // Cursor position
                    ui.label(egui::RichText::new(
                        format!("Ln {}, Col {}", self.cursor_row, self.cursor_col)
                    ).size(11.0).color(egui::Color32::WHITE));

                    // Spacer
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Cargo status
                        if self.is_running_cargo {
                            ui.label(egui::RichText::new("$(sync~spin) Building...")
                                .size(11.0).color(egui::Color32::from_rgb(255, 220, 100)));
                        } else {
                            ui.label(egui::RichText::new("✓ Ready")
                                .size(11.0).color(egui::Color32::WHITE));
                        }

                        ui.separator();

                        // Active file
                        if !self.file_path.is_empty() {
                            let short = std::path::Path::new(&self.file_path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy();
                            ui.label(egui::RichText::new(short.as_ref())
                                .size(11.0).color(egui::Color32::WHITE));
                        }

                        // Language
                        if self.file_path.ends_with(".rs") {
                            ui.label(egui::RichText::new("Rust")
                                .size(11.0).color(egui::Color32::WHITE));
                        }
                    });
                });
            });

        // ══════════════════════════════════════════════════
        // ACTIVITY BAR (far left, thin icon strip)
        // ══════════════════════════════════════════════════
        egui::SidePanel::left("activitybar")
            .exact_width(40.0)
            .resizable(false)
            .frame(egui::Frame::new()
                .fill(bg_activitybar)
                .inner_margin(egui::Margin::symmetric(4, 8))
                .stroke(egui::Stroke::new(0.5, border_color)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;

                    let icons: Vec<(&str, SidebarTab)> = vec![
                        ("📁", SidebarTab::Explorer),
                        ("🔍", SidebarTab::Search),
                        ("☰", SidebarTab::Outline),
                        ("🤖", SidebarTab::AI),
                        ("🌿", SidebarTab::Git),
                        ("⚙", SidebarTab::Settings),
                    ];

                    for (icon, tab) in &icons {
                        let is_active = self.active_sidebar_tab == Some(*tab);
                        let btn_text = egui::RichText::new(*icon).size(20.0);

                        let btn = egui::Button::new(btn_text)
                            .fill(if is_active { accent_blue } else { egui::Color32::TRANSPARENT })
                            .corner_radius(egui::CornerRadius::same(4))
                            .min_size(egui::vec2(32.0, 32.0));

                        if ui.add(btn).clicked() {
                            if is_active {
                                // Toggle sidebar off
                                self.sidebar_visible = !self.sidebar_visible;
                            } else {
                                self.active_sidebar_tab = Some(*tab);
                                self.sidebar_visible = true;
                            }
                        }
                    }

                    // Bottom-aligned toggle for bottom panel
                    ui.add_space(ui.available_height() - 40.0);
                    let bottom_icon = if self.bottom_panel_visible { "▾ ⌨" } else { "▸ ⌨" };
                    if ui.add(
                        egui::Button::new(egui::RichText::new(bottom_icon).size(16.0))
                            .fill(egui::Color32::TRANSPARENT)
                            .min_size(egui::vec2(32.0, 32.0))
                    ).clicked() {
                        self.bottom_panel_visible = !self.bottom_panel_visible;
                    }
                });
            });

        // ══════════════════════════════════════════════════
        // SIDEBAR (togglable, left)
        // ══════════════════════════════════════════════════
        if self.sidebar_visible {
            egui::SidePanel::left("sidebar")
                .default_width(240.0)
                .width_range(180.0..=400.0)
                .frame(egui::Frame::new()
                    .fill(bg_sidebar)
                    .inner_margin(egui::Margin::symmetric(8, 8))
                    .stroke(egui::Stroke::new(0.5, border_color)))
                .show(ctx, |ui| {
                    match self.active_sidebar_tab.unwrap_or(SidebarTab::Explorer) {
                        SidebarTab::Explorer => self.render_explorer(ui),
                        SidebarTab::Search => self.render_search(ui),
                        SidebarTab::Outline => self.render_outline(ui),
                        SidebarTab::AI => self.render_ai_panel(ui, ctx),
                        SidebarTab::Git => self.render_git_panel(ui),
                        SidebarTab::Settings => self.render_settings(ui),
                    }
                });
        }

        // ══════════════════════════════════════════════════
        // BOTTOM PANEL (tabbed: Output / Terminal)
        // ══════════════════════════════════════════════════
        if self.bottom_panel_visible {
            egui::TopBottomPanel::bottom("bottom_drawer")
                .default_height(180.0)
                .height_range(80.0..=400.0)
                .resizable(true)
                .frame(egui::Frame::new()
                    .fill(egui::Color32::from_rgb(30, 30, 30))
                    .inner_margin(egui::Margin::same(0))
                    .stroke(egui::Stroke::new(0.5, border_color)))
                .show(ctx, |ui| {
                    // Tab bar
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;

                        let output_active = self.active_bottom_tab == BottomTab::Output;
                        let terminal_active = self.active_bottom_tab == BottomTab::Terminal;
                        let problems_active = self.active_bottom_tab == BottomTab::Problems;

                        let output_btn = egui::Button::new(
                            egui::RichText::new("  OUTPUT  ").size(11.0)
                                .color(if output_active { egui::Color32::WHITE } else { text_dim })
                        ).fill(if output_active { tab_active_bg } else { tab_inactive_bg })
                         .corner_radius(egui::CornerRadius::same(0));

                        if ui.add(output_btn).clicked() {
                            self.active_bottom_tab = BottomTab::Output;
                        }

                        let term_btn = egui::Button::new(
                            egui::RichText::new("  TERMINAL  ").size(11.0)
                                .color(if terminal_active { egui::Color32::WHITE } else { text_dim })
                        ).fill(if terminal_active { tab_active_bg } else { tab_inactive_bg })
                         .corner_radius(egui::CornerRadius::same(0));

                        if ui.add(term_btn).clicked() {
                            self.active_bottom_tab = BottomTab::Terminal;
                        }

                        let problems_btn = egui::Button::new(
                            egui::RichText::new(format!("  PROBLEMS ({})  ", self.diagnostics.len())).size(11.0)
                                .color(if problems_active { egui::Color32::WHITE } else { text_dim })
                        ).fill(if problems_active { tab_active_bg } else { tab_inactive_bg })
                         .corner_radius(egui::CornerRadius::same(0));

                        if ui.add(problems_btn).clicked() {
                            self.active_bottom_tab = BottomTab::Problems;
                        }

                        // Right-align: close
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                self.bottom_panel_visible = false;
                            }
                        });
                    });

                    ui.separator();

                    // Content
                    let frame = egui::Frame::new()
                        .inner_margin(egui::Margin::same(8))
                        .fill(egui::Color32::from_rgb(30, 30, 30));
                    frame.show(ui, |ui| {
                        match self.active_bottom_tab {
                            BottomTab::Output => {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.output)
                                            .desired_width(f32::INFINITY)
                                            .font(egui::FontId::monospace(12.0))
                                            .text_color(text_color)
                                    );
                                });
                            }
                            BottomTab::Terminal => {
                                // Command input
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("❯").color(egui::Color32::from_rgb(80, 200, 120)));
                                    let response = ui.add(
                                        egui::TextEdit::singleline(&mut self.terminal_command)
                                            .desired_width(ui.available_width() - 60.0)
                                            .font(egui::FontId::monospace(12.0))
                                            .hint_text("Type a command...")
                                    );
                                    if ui.button("Run").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                                        self.run_terminal_command();
                                    }
                                });
                                ui.separator();
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.terminal_output)
                                            .desired_width(f32::INFINITY)
                                            .font(egui::FontId::monospace(12.0))
                                            .text_color(text_color)
                                    );
                                });
                            }
                            BottomTab::Problems => {
                                if ui.button(egui::RichText::new("Fix Build Errors with AI").size(11.0)).clicked() {
                                    self.fix_build_errors(ctx);
                                    self.active_sidebar_tab = Some(SidebarTab::AI);
                                    self.sidebar_visible = true;
                                }
                                ui.separator();

                                if self.diagnostics.is_empty() {
                                    ui.label(egui::RichText::new("No diagnostics yet. Run Build to populate this panel.")
                                        .size(12.0)
                                        .color(text_dim));
                                } else {
                                    let diagnostics = self.diagnostics.clone();
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        for diagnostic in diagnostics {
                                            let color = if diagnostic.level == "error" {
                                                egui::Color32::from_rgb(244, 71, 71)
                                            } else {
                                                egui::Color32::from_rgb(255, 204, 102)
                                            };
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(egui::RichText::new(diagnostic.level.to_uppercase()).color(color).size(11.0).strong());
                                                if ui.link(format!("{}:{}", short_path(&diagnostic.file), diagnostic.line)).clicked() {
                                                    if let Some(folder) = (!self.folder_path.is_empty()).then_some(self.folder_path.clone()) {
                                                        let full = std::path::Path::new(&folder).join(&diagnostic.file);
                                                        self.open_file(full.to_string_lossy().to_string());
                                                    }
                                                }
                                                ui.label(egui::RichText::new(diagnostic.message).size(11.0).color(text_color));
                                            });
                                            ui.add_space(3.0);
                                        }
                                    });
                                }
                            }
                        }
                    });
                });
        }

        // ══════════════════════════════════════════════════
        // CENTRAL PANEL (Editor with tabs)
        // ══════════════════════════════════════════════════
        egui::CentralPanel::default()
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 30, 30))
                .inner_margin(egui::Margin::same(0)))
            .show(ctx, |ui| {
                // ── Tabs bar ──────────────────────────────
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let mut tab_to_remove = None;

                    let open_tabs = self.tabs.clone();
                    for (index, tab) in open_tabs.iter().enumerate() {
                        let name = std::path::Path::new(tab)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();

                        let is_active = self.file_path == *tab;
                        let dirty = is_active && self.code != self.saved_code;
                        let icon = get_file_icon(&name);

                        let tab_bg = if is_active { tab_active_bg } else { tab_inactive_bg };
                        let tab_fg = if is_active { egui::Color32::WHITE } else { text_dim };

                        // Tab button
                        let tab_btn = egui::Button::new(
                            egui::RichText::new(format!(" {} {}{} ", icon, name, if dirty { " *" } else { "" })).size(12.0).color(tab_fg)
                        ).fill(tab_bg)
                         .corner_radius(egui::CornerRadius::same(0))
                         .stroke(egui::Stroke::new(0.0, egui::Color32::TRANSPARENT));

                        if ui.add(tab_btn).clicked() {
                            self.open_file(tab.clone());
                        }

                        // Close button
                        let close = ui.add(
                            egui::Button::new(
                                egui::RichText::new("×").size(12.0).color(text_dim)
                            ).fill(tab_bg)
                             .corner_radius(egui::CornerRadius::same(0))
                             .frame(false)
                        );
                        if close.clicked() {
                            tab_to_remove = Some(index);
                        }

                        // Separator between tabs
                        ui.add(egui::Separator::default().vertical().spacing(0.0));
                    }

                    if let Some(index) = tab_to_remove {
                        self.tabs.remove(index);
                        if self.current_tab >= self.tabs.len() {
                            self.current_tab = self.tabs.len().saturating_sub(1);
                        }
                        if !self.tabs.is_empty() {
                            let new_tab = self.tabs[self.current_tab].clone();
                            if let Ok(content) = fs::read_to_string(&new_tab) {
                                self.code = content;
                                self.file_path = new_tab;
                            }
                        } else {
                            self.code.clear();
                            self.file_path.clear();
                        }
                    }
                });

                // Breadcrumb
                if !self.file_path.is_empty() {
                    let breadcrumb = self.file_path.replace('\\', "/");
                    let parts: Vec<&str> = breadcrumb.split('/').filter(|s| !s.is_empty()).collect();
                    let display = if parts.len() > 3 {
                        parts[parts.len()-3..].join(" › ")
                    } else {
                        parts.join(" › ")
                    };
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new(display).size(11.0).color(text_dim));
                    });
                }

                ui.separator();

                // ── Toolbar ──────────────────────────────
                ui.horizontal(|ui| {
                    ui.add_space(4.0);

                    if ui.add(egui::Button::new(
                        egui::RichText::new("💾 Save").size(12.0)
                    ).corner_radius(egui::CornerRadius::same(3))).clicked() {
                        self.save_current_file();
                    }

                    if ui.add(egui::Button::new(
                        egui::RichText::new("🗑 Delete").size(12.0)
                    ).corner_radius(egui::CornerRadius::same(3))).clicked() {
                        if !self.file_path.is_empty() {
                            match fs::remove_file(&self.file_path) {
                                Ok(_) => {
                                    self.output = format!("Deleted {}", self.file_path);
                                    self.tabs.retain(|t| t != &self.file_path);
                                    self.file_tree = build_file_tree(&self.folder_path);

                                    if !self.tabs.is_empty() {
                                        self.current_tab = 0;
                                        let new_tab_path = self.tabs[0].clone();
                                        if let Ok(content) = fs::read_to_string(&new_tab_path) {
                                            self.code = content;
                                            self.file_path = new_tab_path.clone();
                                            self.delete_file_path = new_tab_path;
                                        }
                                    } else {
                                        self.code.clear();
                                        self.file_path.clear();
                                        self.delete_file_path.clear();
                                    }
                                }
                                Err(e) => self.output = e.to_string(),
                            }
                        }
                    }

                    ui.separator();

                    let build_text = if self.is_running_cargo { "⏳ Building..." } else { "🔨 Build" };
                    if ui.add_enabled(!self.is_running_cargo, egui::Button::new(
                        egui::RichText::new(build_text).size(12.0)
                    ).corner_radius(egui::CornerRadius::same(3))).clicked() {
                        self.run_cargo("build", ctx);
                    }

                    let run_text = if self.is_running_cargo { "⏳ Running..." } else { "▶ Run" };
                    if ui.add_enabled(!self.is_running_cargo, egui::Button::new(
                        egui::RichText::new(run_text).size(12.0)
                    ).corner_radius(egui::CornerRadius::same(3))).clicked() {
                        self.run_cargo("run", ctx);
                    }

                    ui.separator();

                    if ui.button(egui::RichText::new("Go To Definition").size(12.0)).clicked() {
                        self.go_to_definition();
                    }
                });

                ui.separator();

                // ── Code editor ──────────────────────────
                let editor_frame = egui::Frame::new()
                    .fill(egui::Color32::from_rgb(30, 30, 30))
                    .inner_margin(egui::Margin::symmetric(8, 4));

                editor_frame.show(ui, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                            let mut layout_job = crate::highlighter::highlight(string.as_str());
                            layout_job.wrap.max_width = wrap_width;
                            ui.fonts_mut(|f| f.layout_job(layout_job))
                        };

                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.code)
                                .desired_width(f32::INFINITY)
                                .code_editor()
                                .layouter(&mut layouter)
                        );
                        if let Some(info) = self.hover_info_for_cursor() {
                            response.clone().on_hover_text(info);
                        }
                        response.context_menu(|ui| {
                            if ui.button("Go To Definition").clicked() {
                                self.go_to_definition();
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("AI: Explain").clicked() {
                                let prompt = format!(
                                    "{}\n\nExplain the symbol or selected code:\n```{}\n{}\n```",
                                    self.active_ai_context("Explain this code action target."),
                                    file_language(&self.file_path).to_lowercase(),
                                    self.selected_or_current_code()
                                );
                                self.trigger_ai_query(prompt, ctx.clone());
                                self.active_sidebar_tab = Some(SidebarTab::AI);
                                self.sidebar_visible = true;
                                ui.close();
                            }
                            if ui.button("AI: Improve").clicked() {
                                self.improve_code_inline(ctx);
                                ui.close();
                            }
                            if ui.button("AI: Add Comments").clicked() {
                                let prompt = format!(
                                    "{}\n\nAdd useful comments to the active file. Return the complete updated file in one fenced code block.",
                                    self.active_ai_context("Add comments to clarify the code.")
                                );
                                self.trigger_ai_query_with_mode(prompt, ctx.clone(), AiApplyMode::ReplaceEditor);
                                ui.close();
                            }
                            if ui.button("AI: Refactor").clicked() {
                                let prompt = format!(
                                    "{}\n\nRefactor the active file. Return the complete updated file in one fenced code block.",
                                    self.active_ai_context("Refactor from editor context menu.")
                                );
                                self.trigger_ai_query_with_mode(prompt, ctx.clone(), AiApplyMode::ReplaceEditor);
                                ui.close();
                            }
                            if ui.button("AI: Generate Tests").clicked() {
                                let prompt = format!(
                                    "{}\n\nGenerate focused Rust tests for this code. Return the updated file or a new test module in a fenced code block.",
                                    self.active_ai_context("Generate tests for the selected code or active file.")
                                );
                                self.trigger_ai_query(prompt, ctx.clone());
                                self.active_sidebar_tab = Some(SidebarTab::AI);
                                self.sidebar_visible = true;
                                ui.close();
                            }
                        });

                        // Cursor tracking
                        if response.has_focus() {
                            if let Some(state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                                if let Some(ccursor_range) = state.cursor.char_range() {
                                    let cursor_pos = ccursor_range.primary.index;
                                    self.cursor_index = cursor_pos;
                                    let cursor_byte = byte_index_for_char(&self.code, cursor_pos);
                                    let text_before = self.code.get(..cursor_byte).unwrap_or("");
                                    self.cursor_row = text_before.lines().count().max(1);
                                    let last_line = text_before.lines().last().unwrap_or("");
                                    self.cursor_col = last_line.len() + 1;

                                    let secondary = ccursor_range.secondary.index;
                                    if secondary != cursor_pos {
                                        let start = cursor_pos.min(secondary);
                                        let end = cursor_pos.max(secondary);
                                        self.selected_text = slice_char_range(&self.code, start, end);
                                    }
                                }
                            }
                            self.completion_visible = !self.completion_suggestions().is_empty();
                        }

                        if let Some(target) = self.pending_cursor_index.take() {
                            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                                let cursor = egui::text::CCursor::new(target);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                                state.store(ui.ctx(), response.id);
                                response.request_focus();
                            }
                        }

                        if self.completion_visible {
                            let suggestions = self.completion_suggestions();
                            if !suggestions.is_empty() {
                                egui::Frame::new()
                                    .fill(egui::Color32::from_rgb(37, 37, 38))
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)))
                                    .inner_margin(egui::Margin::symmetric(8, 6))
                                    .show(ui, |ui| {
                                        ui.label(egui::RichText::new("Completions").size(11.0).color(text_dim).strong());
                                        for item in suggestions {
                                            if ui.selectable_label(false, egui::RichText::new(&item.label).size(12.0)).clicked() {
                                                self.insert_completion(&item);
                                            }
                                        }
                                    });
                            }
                        }
                    });
                });
            });
    }
}

// ══════════════════════════════════════════════════════════
// Sidebar panel rendering methods
// ══════════════════════════════════════════════════════════
impl MyIDE {
    fn render_explorer(&mut self, ui: &mut egui::Ui) {
        let text_dim = egui::Color32::from_rgb(128, 128, 128);

        ui.label(egui::RichText::new("EXPLORER").size(11.0).color(text_dim).strong());
        ui.add_space(4.0);

        // Open folder
        if ui.add(egui::Button::new(
            egui::RichText::new("📁 Open Folder").size(12.0)
        ).min_size(egui::vec2(ui.available_width(), 24.0))).clicked() {
            if let Some(path) = FileDialog::new().pick_folder() {
                self.folder_path = path.to_string_lossy().to_string();
                self.file_tree = build_file_tree(&self.folder_path);
            }
        }

        if !self.folder_path.is_empty() {
            let folder_name = std::path::Path::new(&self.folder_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            ui.label(egui::RichText::new(format!("📂 {}", folder_name))
                .size(12.0).color(egui::Color32::from_rgb(200, 200, 200)).strong());
        }

        ui.add_space(4.0);
        ui.separator();

        // Search
        ui.add(
            egui::TextEdit::singleline(&mut self.search_text)
                .hint_text("🔍 Filter files...")
                .desired_width(f32::INFINITY)
        );
        ui.add_space(4.0);

        // File tree
        egui::ScrollArea::vertical().max_height(ui.available_height() - 180.0).show(ui, |ui| {
            if self.show_full_paths {
                let mut flat_files = Vec::new();
                get_flat_files(&self.file_tree, &mut flat_files);

                for node in flat_files {
                    if !self.search_text.is_empty() && !node.path.to_lowercase().contains(&self.search_text.to_lowercase()) {
                        continue;
                    }
                    if ui.selectable_label(self.file_path == node.path, format!("📄 {}", node.path)).clicked() {
                        self.file_path = node.path.clone();
                        self.delete_file_path = node.path.clone();
                        if !self.tabs.contains(&node.path) {
                            self.tabs.push(node.path.clone());
                        }
                        match fs::read_to_string(&node.path) {
                            Ok(content) => {
                                self.code = content;
                                self.output = format!("Opened: {}", node.name);
                            }
                            Err(e) => self.output = format!("Error: {}", e),
                        }
                    }
                }
            } else {
                // Clone to avoid borrow issues
                let tree = self.file_tree.clone();
                for node in &tree {
                    render_node(
                        ui,
                        node,
                        &mut self.expanded_paths,
                        &mut self.file_path,
                        &mut self.delete_file_path,
                        &mut self.tabs,
                        &mut self.code,
                        &mut self.output,
                        &self.search_text,
                    );
                }
            }
        });

        ui.separator();
        ui.checkbox(&mut self.show_full_paths, egui::RichText::new("Show Full Paths").size(11.0));

        ui.add_space(4.0);

        // ── File operations (collapsible) ──
        ui.collapsing(egui::RichText::new("📄 New File").size(11.0), |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.new_file_name)
                    .hint_text("filename.rs")
                    .desired_width(f32::INFINITY)
            );
            if ui.button("Create").clicked() {
                if !self.folder_path.is_empty() && !self.new_file_name.is_empty() {
                    let path = format!("{}/{}", self.folder_path, self.new_file_name);
                    match fs::write(&path, "") {
                        Ok(_) => {
                            self.output = format!("Created {}", self.new_file_name);
                            self.file_tree = build_file_tree(&self.folder_path);
                            self.new_file_name.clear();
                        }
                        Err(e) => self.output = e.to_string(),
                    }
                }
            }
        });

        ui.collapsing(egui::RichText::new("📁 New Folder").size(11.0), |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.new_folder_name)
                    .hint_text("folder_name")
                    .desired_width(f32::INFINITY)
            );
            if ui.button("Create").clicked() {
                if !self.folder_path.is_empty() && !self.new_folder_name.is_empty() {
                    let path = format!("{}/{}", self.folder_path, self.new_folder_name);
                    match fs::create_dir_all(&path) {
                        Ok(_) => {
                            self.output = format!("Created folder {}", self.new_folder_name);
                            self.file_tree = build_file_tree(&self.folder_path);
                            self.new_folder_name.clear();
                        }
                        Err(e) => self.output = e.to_string(),
                    }
                }
            }
        });

        ui.collapsing(egui::RichText::new("✏ Rename").size(11.0), |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.rename_file_name)
                    .hint_text("new_name.rs")
                    .desired_width(f32::INFINITY)
            );
            if ui.button("Rename").clicked() {
                if !self.file_path.is_empty() && !self.rename_file_name.is_empty() {
                    let parent = std::path::Path::new(&self.file_path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let new_path = format!("{}/{}", parent, self.rename_file_name);

                    match fs::rename(&self.file_path, &new_path) {
                        Ok(_) => {
                            self.output = format!("Renamed to {}", self.rename_file_name);
                            if let Some(pos) = self.tabs.iter().position(|t| t == &self.file_path) {
                                self.tabs[pos] = new_path.clone();
                            }
                            self.file_path = new_path.clone();
                            self.delete_file_path = new_path;
                            self.rename_file_name.clear();
                            self.file_tree = build_file_tree(&self.folder_path);
                        }
                        Err(e) => self.output = e.to_string(),
                    }
                }
            }
        });
    }

    fn render_search(&mut self, ui: &mut egui::Ui) {
        let text_dim = egui::Color32::from_rgb(128, 128, 128);

        ui.label(egui::RichText::new("SEARCH").size(11.0).color(text_dim).strong());
        ui.add_space(4.0);

        let search_response = ui.add(
            egui::TextEdit::singleline(&mut self.global_search_query)
                .hint_text("Search in files...")
                .desired_width(f32::INFINITY)
        );

        if search_response.changed() || (search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
            self.run_global_search();
        }

        ui.add_space(4.0);

        if !self.global_search_results.is_empty() {
            ui.label(egui::RichText::new(
                format!("{} results", self.global_search_results.len())
            ).size(11.0).color(text_dim));
            ui.separator();

            let results = self.global_search_results.clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (location, line_text) in &results {
                    let parts: Vec<&str> = location.rsplitn(2, ':').collect();
                    let line_num = parts.first().unwrap_or(&"");
                    let file_path = parts.last().unwrap_or(&"");

                    let short_name = std::path::Path::new(file_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();

                    ui.vertical(|ui| {
                        if ui.link(egui::RichText::new(
                            format!("{} :{}", short_name, line_num)
                        ).size(11.0)).clicked() {
                            self.open_file(file_path.to_string());
                        }
                        let trimmed = if line_text.len() > 80 {
                            format!("{}...", &line_text[..80])
                        } else {
                            line_text.clone()
                        };
                        ui.label(egui::RichText::new(trimmed).size(10.0).color(text_dim));
                    });
                    ui.add_space(2.0);
                }
            });
        }
    }

    fn render_outline(&mut self, ui: &mut egui::Ui) {
        let text_dim = egui::Color32::from_rgb(128, 128, 128);

        ui.label(egui::RichText::new("OUTLINE").size(11.0).color(text_dim).strong());
        ui.add_space(4.0);

        if self.file_path.is_empty() {
            ui.label(egui::RichText::new("Open a file to see symbols.").size(11.0).color(text_dim));
            return;
        }

        self.outline_symbols = parse_outline(&self.code, &self.file_path);
        ui.label(egui::RichText::new(format!("{} symbols in {}", self.outline_symbols.len(), short_path(&self.file_path)))
            .size(11.0)
            .color(text_dim));
        ui.separator();

        if self.outline_symbols.is_empty() {
            ui.label(egui::RichText::new("No symbols detected for this language.").size(11.0).color(text_dim));
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let symbols = self.outline_symbols.clone();
            for symbol in symbols {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&symbol.kind).size(10.0).color(text_dim));
                    if ui.link(egui::RichText::new(&symbol.name).size(12.0).color(egui::Color32::from_rgb(212, 212, 212))).clicked() {
                        let target = char_index_for_line(&self.code, symbol.line);
                        self.pending_cursor_index = Some(target);
                        self.cursor_index = target;
                        self.cursor_row = symbol.line;
                        self.cursor_col = 1;
                        self.output = format!("Jumped to {} {} at line {}", symbol.kind, symbol.name, symbol.line);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!(":{}", symbol.line)).size(10.0).color(text_dim));
                    });
                });
            }
        });
    }

    fn improve_code_inline(&mut self, ctx: &egui::Context) {
        let prompt = format!(
            "{}\n\nReturn only the complete improved replacement code in one fenced code block.",
            self.active_ai_context("Improve the selected code if a selection exists, otherwise improve the whole active file.")
        );
        self.trigger_ai_query_with_mode(prompt, ctx.clone(), AiApplyMode::ReplaceEditor);
    }

    fn generate_function_inline(&mut self, ctx: &egui::Context) {
        let prompt = format!(
            "{}\n\nUser request or surrounding code:\n{}\n\nReturn only the complete updated file in one fenced code block.",
            self.active_ai_context("Generate the missing function or complete the current code."),
            self.selected_or_current_code()
        );
        self.trigger_ai_query_with_mode(prompt, ctx.clone(), AiApplyMode::ReplaceEditor);
    }

    fn fix_build_errors(&mut self, ctx: &egui::Context) {
        let prompt = format!(
            "{}\n\nCompiler diagnostics:\n{}\n\nFix the code. Return a concise explanation plus a complete replacement file in a fenced code block.",
            self.active_ai_context("Fix the compiler/build errors in the active file."),
            self.diagnostics_context()
        );
        self.trigger_ai_query(prompt, ctx.clone());
    }

    fn word_bounds_at_cursor(&self) -> (usize, usize) {
        let chars: Vec<char> = self.code.chars().collect();
        let mut start = self.cursor_index.min(chars.len());
        let mut end = self.cursor_index.min(chars.len());

        while start > 0 && is_rust_ident_char(chars[start - 1]) {
            start -= 1;
        }
        while end < chars.len() && is_rust_ident_char(chars[end]) {
            end += 1;
        }

        (start, end)
    }

    fn word_at_cursor(&self) -> String {
        let (start, end) = self.word_bounds_at_cursor();
        slice_char_range(&self.code, start, end).trim_end_matches('!').to_string()
    }

    fn completion_prefix(&self) -> String {
        let cursor_byte = byte_index_for_char(&self.code, self.cursor_index);
        let before_cursor = self.code.get(..cursor_byte).unwrap_or("");
        before_cursor
            .chars()
            .rev()
            .take_while(|c| is_rust_ident_char(*c))
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    }

    fn completion_suggestions(&self) -> Vec<CompletionItem> {
        let cursor_byte = byte_index_for_char(&self.code, self.cursor_index);
        let before_cursor = self.code.get(..cursor_byte).unwrap_or("");
        let current_line = before_cursor.lines().last().unwrap_or("");
        let prefix = self.completion_prefix();
        if prefix.is_empty() {
            return Vec::new();
        }

        let mut items = vec![
            CompletionItem { label: "fn".to_string(), insert: "fn ".to_string() },
            CompletionItem { label: "fn main()".to_string(), insert: "fn main() {\n    \n}".to_string() },
            CompletionItem { label: "fn new() -> Self".to_string(), insert: "fn new() -> Self {\n    Self {\n        \n    }\n}".to_string() },
            CompletionItem { label: "fn update()".to_string(), insert: "fn update(&mut self) {\n    \n}".to_string() },
            CompletionItem { label: "struct".to_string(), insert: "struct ".to_string() },
            CompletionItem { label: "impl".to_string(), insert: "impl ".to_string() },
            CompletionItem { label: "enum".to_string(), insert: "enum ".to_string() },
            CompletionItem { label: "trait".to_string(), insert: "trait ".to_string() },
            CompletionItem { label: "println!".to_string(), insert: "println!(\"\");".to_string() },
            CompletionItem { label: "print!".to_string(), insert: "print!(\"\");".to_string() },
            CompletionItem { label: "eprintln!".to_string(), insert: "eprintln!(\"\");".to_string() },
            CompletionItem { label: "String".to_string(), insert: "String".to_string() },
            CompletionItem { label: "Vec".to_string(), insert: "Vec".to_string() },
            CompletionItem { label: "Option".to_string(), insert: "Option".to_string() },
            CompletionItem { label: "Result".to_string(), insert: "Result".to_string() },
        ];

        for symbol in parse_outline(&self.code, &self.file_path) {
            if !symbol.name.is_empty() {
                items.push(CompletionItem {
                    label: format!("{} {}", symbol.kind, symbol.name),
                    insert: symbol.name,
                });
            }
        }

        let prefix_lower = prefix.to_lowercase();
        let trimmed_line = current_line.trim_start();
        items
            .into_iter()
            .filter(|item| {
                let label = item.label.to_lowercase();
                let insert = item.insert.to_lowercase();
                label.starts_with(&prefix_lower)
                    || insert.starts_with(&prefix_lower)
                    || label.trim_end_matches('!').starts_with(&prefix_lower)
            })
            .filter(|item| !(prefix == "fn" && item.label == "fn" && trimmed_line == "fn"))
            .take(12)
            .collect()
    }

    fn insert_completion(&mut self, item: &CompletionItem) {
        let prefix_len = self.completion_prefix().chars().count();
        let start = self.cursor_index.saturating_sub(prefix_len);
        let start_byte = byte_index_for_char(&self.code, start);
        let end_byte = byte_index_for_char(&self.code, self.cursor_index);
        self.code.replace_range(start_byte..end_byte, &item.insert);
        self.cursor_index = start + item.insert.chars().count();
        self.pending_cursor_index = Some(self.cursor_index);
        self.completion_visible = false;
        self.output = format!("Inserted completion: {}", item.label);
    }

    fn hover_info_for_cursor(&self) -> Option<String> {
        let raw_word = {
            let (start, end) = self.word_bounds_at_cursor();
            slice_char_range(&self.code, start, end)
        };
        let word = if raw_word.ends_with('!') {
            raw_word
        } else {
            raw_word.trim_end_matches(|c: char| !is_rust_ident_char(c)).to_string()
        };

        rust_hover_info().get(word.as_str()).map(|info| info.to_string())
    }

    fn go_to_definition(&mut self) {
        let word = self.word_at_cursor();
        if word.is_empty() {
            self.output = "No symbol under cursor.".to_string();
            return;
        }

        let symbols = parse_outline(&self.code, &self.file_path);
        if let Some(symbol) = symbols.iter().find(|symbol| symbol.name == word) {
            let target = char_index_for_line(&self.code, symbol.line);
            self.pending_cursor_index = Some(target);
            self.cursor_index = target;
            self.cursor_row = symbol.line;
            self.cursor_col = 1;
            self.selected_text = symbol.name.clone();
            self.output = format!("Go to definition: {} {} at line {}", symbol.kind, symbol.name, symbol.line);
        } else {
            self.output = format!("Definition not found for '{}'.", word);
        }
    }

    fn run_git_command(&mut self, args: &[&str]) {
        if self.folder_path.is_empty() {
            self.git_output = "Open a workspace folder before running Git commands.".to_string();
            return;
        }

        match Command::new("git").args(args).current_dir(&self.folder_path).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                self.git_output = format!("git {}\n{}{}", args.join(" "), stdout, stderr);
            }
            Err(e) => {
                self.git_output = format!("git {}\nError: {}", args.join(" "), e);
            }
        }
    }

    fn render_git_panel(&mut self, ui: &mut egui::Ui) {
        let text_dim = egui::Color32::from_rgb(128, 128, 128);
        let accent_green = egui::Color32::from_rgb(80, 180, 120);

        ui.label(egui::RichText::new("SOURCE CONTROL").size(11.0).color(text_dim).strong());
        ui.add_space(6.0);

        if self.folder_path.is_empty() {
            ui.label(egui::RichText::new("Open a folder to use Git.").size(11.0).color(text_dim));
            return;
        }

        ui.horizontal_wrapped(|ui| {
            if ui.button(egui::RichText::new("Status").size(11.0)).clicked() {
                self.run_git_command(&["status", "--short"]);
            }
            if ui.button(egui::RichText::new("Add All").size(11.0)).clicked() {
                self.run_git_command(&["add", "."]);
            }
        });

        ui.add_space(6.0);
        ui.label(egui::RichText::new("Commit message").size(11.0).color(text_dim));
        ui.add(
            egui::TextEdit::singleline(&mut self.git_commit_message)
                .hint_text("Describe the change...")
                .desired_width(f32::INFINITY)
        );

        if ui.add_enabled(
            !self.git_commit_message.trim().is_empty(),
            egui::Button::new(egui::RichText::new("Commit").size(11.0)).fill(accent_green)
        ).clicked() {
            let message = self.git_commit_message.clone();
            self.run_git_command(&["commit", "-m", &message]);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Git Output").size(11.0).color(text_dim));

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.git_output)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::monospace(11.0))
            );
        });
    }

    fn render_ai_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let text_dim = egui::Color32::from_rgb(128, 128, 128);
        let accent_blue = egui::Color32::from_rgb(0, 122, 204);

        ui.label(egui::RichText::new("AI ASSISTANT").size(11.0).color(text_dim).strong());
        ui.add_space(4.0);

        // Prompt input
        ui.add(
            egui::TextEdit::multiline(&mut self.ai_prompt)
                .desired_rows(5)
                .desired_width(f32::INFINITY)
                .hint_text("Ask AI anything...")
                .font(egui::FontId::proportional(12.0))
        );

        ui.add_space(4.0);

        ui.collapsing(egui::RichText::new("Context (@file / @selection / @terminal)").size(11.0), |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button(egui::RichText::new("Add Current File").size(11.0)).clicked() {
                    let body = format!("Path: {}\n\n{}", self.file_path, self.code);
                    self.add_ai_context_section("@file", body);
                }
                if ui.button(egui::RichText::new("Add Selected Code").size(11.0)).clicked() {
                    self.add_ai_context_section("@selection", self.selected_or_current_code());
                }
                if ui.button(egui::RichText::new("Add Terminal Output").size(11.0)).clicked() {
                    self.add_ai_context_section("@terminal", self.terminal_output.clone());
                }
                if ui.button(egui::RichText::new("Clear Context").size(11.0)).clicked() {
                    self.ai_context_bundle.clear();
                    self.output = "Cleared AI context.".to_string();
                }
            });

            ui.label(egui::RichText::new("Selected code").size(11.0).color(text_dim));
            ui.add(
                egui::TextEdit::multiline(&mut self.selected_text)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY)
                    .hint_text("Select code in the editor or paste a snippet here...")
                    .font(egui::FontId::monospace(11.0))
            );

            if !self.ai_context_bundle.is_empty() {
                ui.label(egui::RichText::new(format!("{} chars pinned", self.ai_context_bundle.len()))
                    .size(10.0)
                    .color(text_dim));
            }
        });

        ui.add_space(4.0);

        // Action buttons
        ui.horizontal_wrapped(|ui| {
            let ask_text = if self.is_asking_ai { "⏳ Asking..." } else { "🤖 Ask" };
            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new(ask_text).size(11.0))
                    .fill(accent_blue)
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                let prompt = self.active_ai_context(&self.ai_prompt.clone());
                self.trigger_ai_query(prompt, ctx.clone());
            }

            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("📖 Explain").size(11.0))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                let prompt = self.active_ai_context("Explain the active file clearly, including the main architecture and any risky sections.");
                self.trigger_ai_query(prompt, ctx.clone());
            }

            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("🔧 Fix").size(11.0))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                let prompt = self.active_ai_context("Fix the active file using the terminal output. Return a short explanation and then a complete replacement code block if needed.");
                self.trigger_ai_query(prompt, ctx.clone());
            }

            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("♻ Refactor").size(11.0))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                let prompt = self.active_ai_context("Refactor the active file for readability, maintainability, and IDE-quality behavior.");
                self.trigger_ai_query(prompt, ctx.clone());
            }
        });

        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("Fix Build Errors").size(11.0))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                self.fix_build_errors(ctx);
            }

            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("Improve Code").size(11.0))
                    .fill(egui::Color32::from_rgb(14, 99, 156))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                self.improve_code_inline(ctx);
            }

            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("Explain Selection").size(11.0))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                let prompt = format!(
                    "{}\n\nSelection:\n```{}\n{}\n```",
                    self.active_ai_context("Explain the selected code."),
                    file_language(&self.file_path).to_lowercase(),
                    self.selected_or_current_code()
                );
                self.trigger_ai_query(prompt, ctx.clone());
            }

            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("Generate Function").size(11.0))
                    .fill(egui::Color32::from_rgb(14, 99, 156))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                self.generate_function_inline(ctx);
            }

            if ui.add_enabled(!self.is_asking_ai,
                egui::Button::new(egui::RichText::new("Project-Wide AI").size(11.0))
                    .corner_radius(egui::CornerRadius::same(3))
            ).clicked() {
                let prompt = format!(
                    "{}\n\nProject-wide context:\n{}",
                    self.active_ai_context(&self.ai_prompt.clone()),
                    self.workspace_context()
                );
                self.trigger_ai_query(prompt, ctx.clone());
            }
        });

        if self.is_asking_ai {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new("Thinking...").size(11.0).color(text_dim));
            });
        }

        ui.add_space(4.0);
        ui.separator();

        // Response area
        ui.label(egui::RichText::new("Response:").size(11.0).color(text_dim));

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.ai_response)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(12.0))
                    .interactive(true)
            );

            // Apply Code buttons for detected code blocks
            let code_blocks = extract_code_blocks(&self.ai_response);
            if !code_blocks.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(
                    format!("📋 {} code block(s) detected", code_blocks.len())
                ).size(11.0).color(egui::Color32::from_rgb(80, 200, 120)));

                for (i, block) in code_blocks.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(
                            egui::RichText::new(format!("⬇ Apply Block {}", i + 1)).size(11.0)
                        ).fill(egui::Color32::from_rgb(14, 99, 156))
                         .corner_radius(egui::CornerRadius::same(3))).clicked() {
                            self.code = block.clone();
                            self.output = format!("Applied code block {} from AI response", i + 1);
                        }

                        if ui.add(egui::Button::new(
                            egui::RichText::new("📋 Copy").size(11.0)
                        ).corner_radius(egui::CornerRadius::same(3))).clicked() {
                            ui.ctx().copy_text(block.clone());
                            self.output = format!("Copied code block {} to clipboard", i + 1);
                        }
                    });

                    // Preview (first 3 lines)
                    let preview: String = block.lines().take(3).collect::<Vec<&str>>().join("\n");
                    ui.label(egui::RichText::new(format!("  {}", preview))
                        .size(10.0).color(text_dim).font(egui::FontId::monospace(10.0)));
                    ui.add_space(2.0);
                }
            }
        });
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        let text_dim = egui::Color32::from_rgb(128, 128, 128);

        ui.label(egui::RichText::new("SETTINGS").size(11.0).color(text_dim).strong());
        ui.add_space(8.0);

        ui.label(egui::RichText::new("AI Model:").size(12.0));
        ui.add(
            egui::TextEdit::singleline(&mut self.ai_model)
                .desired_width(f32::INFINITY)
                .hint_text("qwen/qwen3-32b")
        );

        ui.add_space(4.0);

        // Quick model selection
        ui.label(egui::RichText::new("Quick Select:").size(11.0).color(text_dim));
        let models = [
            ("Qwen 3 32B", "qwen/qwen3-32b"),
            ("DeepSeek Chat", "deepseek/deepseek-chat"),
            ("Llama 4 Scout", "meta-llama/llama-4-scout-17b-16e-instruct"),
        ];

        for (label, model_id) in &models {
            let is_selected = self.ai_model == *model_id;
            if ui.selectable_label(is_selected, egui::RichText::new(*label).size(11.0)).clicked() {
                self.ai_model = model_id.to_string();
            }
        }

        ui.add_space(8.0);
        ui.separator();

        ui.label(egui::RichText::new("API Key:").size(12.0));
        ui.add(
            egui::TextEdit::singleline(&mut self.ai_api_key)
                .desired_width(f32::INFINITY)
                .hint_text("sk-...")
                .password(true)
        );

        ui.add_space(8.0);
        ui.separator();

        ui.label(egui::RichText::new("About").size(12.0).strong());
        ui.label(egui::RichText::new("DragonFox IDE v0.3.0").size(11.0).color(text_dim));
        ui.label(egui::RichText::new("An AI-powered code editor").size(11.0).color(text_dim));
    }

    fn run_cargo(&mut self, action: &str, ctx: &egui::Context) {
        if self.folder_path.is_empty() {
            return;
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        self.cargo_receiver = Some(receiver);
        self.is_running_cargo = true;

        let folder_path = self.folder_path.clone();
        let action = action.to_string();
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            let res = match Command::new("cargo")
                .arg(&action)
                .current_dir(&folder_path)
                .output()
            {
                Ok(out) => {
                    let mut output = String::from_utf8_lossy(&out.stderr).to_string();
                    if output.is_empty() {
                        output = String::from_utf8_lossy(&out.stdout).to_string();
                    }
                    output
                }
                Err(e) => e.to_string(),
            };
            let _ = sender.send(res);
            ctx.request_repaint();
        });
    }

    fn run_terminal_command(&mut self) {
        if self.terminal_command.is_empty() {
            return;
        }
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(self.terminal_command.clone());
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(self.terminal_command.clone());
            c
        };

        if !self.folder_path.is_empty() {
            cmd.current_dir(&self.folder_path);
        }

        match cmd.output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                self.terminal_output = format!("❯ {}\n{}{}", self.terminal_command, stdout, stderr);
            }
            Err(e) => {
                self.terminal_output = format!("❯ {}\nError: {}", self.terminal_command, e);
            }
        }
    }
}
