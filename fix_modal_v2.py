with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = """// Preview modal dialog
    let mut show_preview_modal = st.show_preview_modal;
    let mut preview_sheet = st.preview_modal_sheet;
    if show_preview_modal {
        if let Some(sheet_idx) = preview_sheet {
            if let Some(sheet) = st.sheets.get(sheet_idx).cloned() {
                let mut modal_open = show_preview_modal;
                let modal_result = egui::Window::new("미리보기 확대")
                    .open(&mut modal_open)
                    .collapsible(false)
                    .resizable(true)
                    .default_size(egui::Vec2::new(800.0, 600.0))
                    .show(ctx, |ui| {
                        if let Some(tex) = &sheet.preview_texture {
                            let available = ui.available_size();
                            let img_size = egui::Vec2::new(tex.size()[0] as f32, tex.size()[1] as f32);
                            let scale = (available.x / img_size.x).min(available.y / img_size.y).min(1.0);
                            let display_size = img_size * scale;
                            ui.add(egui::Image::new((tex.id(), display_size)));
                        } else {
                            ui.label("미리보기가 없습니다. 🔍 버튼을 눌러 생성하세요.");
                        }
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("닫기").clicked() {
                                modal_open = false;
                            }
                        });
                    });
                if modal_open.is_none() {
                    // Window was closed
                    show_preview_modal = false;
                } else {
                    show_preview_modal = modal_open.unwrap();
                }
            }
        }
    }
    st.show_preview_modal = show_preview_modal;
    st.preview_modal_sheet = preview_sheet;"""

new = """// Preview modal dialog
    let mut show_preview_modal = st.show_preview_modal;
    let mut preview_sheet = st.preview_modal_sheet;
    if show_preview_modal {
        if let Some(sheet_idx) = preview_sheet {
            if let Some(sheet) = st.sheets.get(sheet_idx).cloned() {
                let mut modal_open = show_preview_modal;
                egui::Window::new("미리보기 확대")
                    .open(&mut modal_open)
                    .collapsible(false)
                    .resizable(true)
                    .default_size(egui::Vec2::new(800.0, 600.0))
                    .show(ctx, |ui| {
                        if let Some(tex) = &sheet.preview_texture {
                            let available = ui.available_size();
                            let img_size = egui::Vec2::new(tex.size()[0] as f32, tex.size()[1] as f32);
                            let scale = (available.x / img_size.x).min(available.y / img_size.y).min(1.0);
                            let display_size = img_size * scale;
                            ui.add(egui::Image::new((tex.id(), display_size)));
                        } else {
                            ui.label("미리보기가 없습니다. 🔍 버튼을 눌러 생성하세요.");
                        }
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("닫기").clicked() {
                                modal_open = false;
                            }
                        });
                    });
                show_preview_modal = modal_open;
            }
        }
    }
    st.show_preview_modal = show_preview_modal;
    st.preview_modal_sheet = preview_sheet;"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')