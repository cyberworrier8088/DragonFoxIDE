use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};

pub fn highlight(code: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font_id = FontId::monospace(13.0);
    
    // VS Code-style color palette
    let color_keyword = Color32::from_rgb(86, 156, 214); // Blue
    let color_string = Color32::from_rgb(206, 145, 120);  // Orange/brown
    let color_comment = Color32::from_rgb(106, 153, 85);  // Green
    let color_number = Color32::from_rgb(181, 206, 168);  // Light green
    let color_type = Color32::from_rgb(78, 201, 176);    // Teal
    let color_default = Color32::from_rgb(220, 220, 220); // Off-white
    
    let mut chars = code.chars().peekable();
    
    while let Some(&c) = chars.peek() {
        if c == '/' {
            chars.next();
            if let Some(&'/') = chars.peek() {
                // Line comment
                chars.next();
                let mut comment = String::from("//");
                while let Some(&nc) = chars.peek() {
                    if nc == '\n' {
                        break;
                    }
                    comment.push(nc);
                    chars.next();
                }
                job.append(&comment, 0.0, TextFormat {
                    font_id: font_id.clone(),
                    color: color_comment,
                    ..Default::default()
                });
            } else {
                // Slash symbol
                job.append("/", 0.0, TextFormat {
                    font_id: font_id.clone(),
                    color: color_default,
                    ..Default::default()
                });
            }
        } else if c == '"' {
            // String literal
            chars.next();
            let mut string_lit = String::from("\"");
            let mut escaped = false;
            while let Some(&nc) = chars.peek() {
                string_lit.push(nc);
                chars.next();
                if escaped {
                    escaped = false;
                } else if nc == '\\' {
                    escaped = true;
                } else if nc == '"' {
                    break;
                }
            }
            job.append(&string_lit, 0.0, TextFormat {
                font_id: font_id.clone(),
                color: color_string,
                ..Default::default()
            });
        } else if c.is_ascii_digit() {
            // Number literal
            let mut number = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' || nc == 'x' || nc == 'o' || nc == 'b' || nc == '_' || nc.is_ascii_alphabetic() {
                    number.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            job.append(&number, 0.0, TextFormat {
                font_id: font_id.clone(),
                color: color_number,
                ..Default::default()
            });
        } else if c.is_alphabetic() || c == '_' {
            // Identifier or keyword
            let mut ident = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    ident.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            
            let format = match ident.as_str() {
                "fn" | "let" | "struct" | "impl" | "match" | "if" | "else" | "use" | "pub" |
                "mod" | "return" | "true" | "false" | "loop" | "while" | "for" | "in" |
                "static" | "const" | "mut" | "ref" | "self" | "Self" | "as" | "break" |
                "continue" | "crate" | "extern" | "enum" | "super" | "trait" | "type" |
                "unsafe" | "where" => TextFormat {
                    font_id: font_id.clone(),
                    color: color_keyword,
                    ..Default::default()
                },
                "String" | "Option" | "Result" | "Vec" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "f32" | "f64" | "bool" | "char" | "str" => TextFormat {
                    font_id: font_id.clone(),
                    color: color_type,
                    ..Default::default()
                },
                _ => TextFormat {
                    font_id: font_id.clone(),
                    color: color_default,
                    ..Default::default()
                }
            };
            
            job.append(&ident, 0.0, format);
        } else {
            // Symbol/whitespace
            let mut symbol = String::new();
            symbol.push(c);
            chars.next();
            job.append(&symbol, 0.0, TextFormat {
                font_id: font_id.clone(),
                color: color_default,
                ..Default::default()
            });
        }
    }
    
    job
}
