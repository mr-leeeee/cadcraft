with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix missing closing brace for the "Apply to Model/Layout" button
old = """            });
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {"""

new = """            });
            }
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')