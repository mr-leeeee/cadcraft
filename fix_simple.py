with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Try a simpler approach: use a mutable variable and capture it properly
old = """            let print_sys = std::cell::Cell::new(false);
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            print_sys.set(ui.button("프린터 출력").on_hover_text("시스템 프린터로 직접 출력 (PDF 생성 후 OS 프린트 명령 위임)").clicked());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    let print_sys_clicked = print_sys.get();"""

new = """            let mut print_sys = false;
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            print_sys = ui.button("프린터 출력").on_hover_text("시스템 프린터로 직접 출력 (PDF 생성 후 OS 프린트 명령 위임)").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    let print_sys_clicked = print_sys;"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')