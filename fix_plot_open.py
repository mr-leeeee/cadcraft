with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = """            }
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {
                MULTIPLOT_GEN.fetch_add(1, Ordering::Relaxed);
                *open = false;
                app.ui.dialog = Some("multiplot".into());
            }
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    // The output file type follows the device.
    let ext = choices.devices.iter().find(|dv| dv.name.eq_ignore_ascii_case(&st.page.device)).map_or("pdf", |dv| dv.output.as_str());
    set_temp(ctx, "plot_state", st.clone());
    if cancel {
        *open = false;
    }
    if pick {
        // The dialog comes back with these settings and the picked window (`plot_window_resume`).
        set_temp(ctx, PICKING_WINDOW, true);
        *open = false;
        app.start("plotwindow");"""

new = """            }
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {
                MULTIPLOT_GEN.fetch_add(1, Ordering::Relaxed);
                open_local = false;
                app.ui.dialog = Some("multiplot".into());
            }
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    *open = open_local;
    // The output file type follows the device.
    let ext = choices.devices.iter().find(|dv| dv.name.eq_ignore_ascii_case(&st.page.device)).map_or("pdf", |dv| dv.output.as_str());
    set_temp(ctx, "plot_state", st.clone());
    if cancel {
        open_local = false;
    }
    if pick {
        // The dialog comes back with these settings and the picked window (`plot_window_resume`).
        set_temp(ctx, PICKING_WINDOW, true);
        open_local = false;
        app.start("plotwindow");"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')