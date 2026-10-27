with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Move print_sys declaration OUTSIDE the ui.horizontal closure
old = """            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {
                MULTIPLOT_GEN.fetch_add(1, Ordering::Relaxed);
                should_close = true;
                app.ui.dialog = Some("multiplot".into());
            }
            let mut print_sys = false;
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            print_sys = ui.button("프린터 출력").on_hover_text("시스템 프린터로 직접 출력 (PDF 생성 후 OS 프린트 명령 위임)").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    let print_sys_clicked = print_sys;"""

new = """            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {
                MULTIPLOT_GEN.fetch_add(1, Ordering::Relaxed);
                should_close = true;
                app.ui.dialog = Some("multiplot".into());
            }
            let mut print_sys = false;
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            print_sys = ui.button("프린터 출력").on_hover_text("시스템 프린터로 직접 출력 (PDF 생성 후 OS 프린트 명령 위임)").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    // print_sys is now in scope here
    let print_sys_clicked = print_sys;"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')