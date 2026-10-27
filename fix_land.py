with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = '''                    // Landscape
                    let mut land = sheet.landscape;
                    if ui.checkbox(&mut land, if land { "가로" } else { "세로" }).changed() {
                        sheet.landscape = land;
                        sheet.preview_texture = None;
                    }'''

new = '''                    // Landscape
                    let mut land = sheet.landscape;
                    let label = if land { "가로" } else { "세로" };
                    if ui.checkbox(&mut land, label).changed() {
                        sheet.landscape = land;
                        sheet.preview_texture = None;
                    }'''

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')