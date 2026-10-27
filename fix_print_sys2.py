with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix the print_sys issue
old = """            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {
                MULTIPLOT_GEN.fetch_add(1, Ordering::Relaxed);
                should_close = true;
                app.ui.dialog = Some("multiplot".into());
            }
            let mut print_sys = false;
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            print_sys = ui.button("\uD504\uB9B0\uD130 \uCD9C\uB825").on_hover_text("\uC2DC\uC2A4\uD15C \uD504\uB9B0\uD130\uB85C \uC9C1\uC811 \uCD9C\uB825 (PDF \uC0DD\uC131 \uD6C4 OS \uD504\uB9B0\uD130 \uBA85\uB825 \uC704\uC784)").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    if should_close {
        *open = false;
    }
    // The output file type follows the device.
    let ext = choices.devices.iter().find(|dv| dv.name.eq_ignore_ascii_case(&st.page.device)).map_or("pdf", |dv| dv.output.as_str());
    set_temp(ctx, "plot_state", st.clone());
    if cancel {
        should_close = true;
    }
    if pick {
        // The dialog comes back with these settings and the picked window (`plot_window_resume`).
        set_temp(ctx, PICKING_WINDOW, true);
        app.start("plotwindow");
        *open = false;
        return action;
    }
    if preview {
        // A sheet about 1600 pixels on its long side.
        let long_in = st.page.width_mm.max(st.page.height_mm).max(1.0) / 25.4;
        let mut info = format!("{} \uC0B4 {}", st.target, st.page.paper);
        if st.page.landscape {
            info.push_str(" \uC2DD landscape");
        }
        crate::exchange::preview_plot(app, ctx, plot_params(&st, Some((1600.0 / long_in).clamp(50.0, 300.0))), info);
        return action;
    }
    if print_sys {"""

new = """            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {
                MULTIPLOT_GEN.fetch_add(1, Ordering::Relaxed);
                should_close = true;
                app.ui.dialog = Some("multiplot".into());
            }
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            let print_sys = ui.button("\uD504\uB9B0\uD130 \uCD9C\uB825").on_hover_text("\uC2DC\uC2A4\uD15C \uD504\uB9B0\uD130\uB85C \uC9C1\uC811 \uCD9C\uB825 (PDF \uC0DD\uC131 \uD6C4 OS \uD504\uB9B0\uD130 \uBA85\uB825 \uC704\uC784)").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    
    // Handle print_sys after the window is closed
    let print_sys_clicked = print_sys;
    if should_close {
        *open = false;
    }
    // The output file type follows the device.
    let ext = choices.devices.iter().find(|dv| dv.name.eq_ignore_ascii_case(&st.page.device)).map_or("pdf", |dv| dv.output.as_str());
    set_temp(ctx, "plot_state", st.clone());
    if cancel {
        should_close = true;
    }
    if pick {
        // The dialog comes back with these settings and the picked window (`plot_window_resume`).
        set_temp(ctx, PICKING_WINDOW, true);
        app.start("plotwindow");
        *open = false;
        return action;
    }
    if preview {
        // A sheet about 1600 pixels on its long side.
        let long_in = st.page.width_mm.max(st.page.height_mm).max(1.0) / 25.4;
        let mut info = format!("{} \uC0B4 {}", st.target, st.page.paper);
        if st.page.landscape {
            info.push_str(" \uC2DD landscape");
        }
        crate::exchange::preview_plot(app, ctx, plot_params(&st, Some((1600.0 / long_in).clamp(50.0, 300.0))), info);
        return action;
    }
    if print_sys_clicked {"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')