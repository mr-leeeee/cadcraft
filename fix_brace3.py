with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix both issues: add missing } for Apply button if block, and move print_sys outside
old = """        ui.horizontal(|ui| {
            let (cmd, label, hover) = if model {
                ("modelpagesetup", "Apply to Model", "Save these settings for Model space")
            } else {
                ("pagesetup", "Apply to Layout", "Save these settings in the layout's page setup")
            };
            if ui.button(label).on_hover_text(hover).clicked() {
                let page = &st.page;
                action = Some((
                    cmd,
                    json!({
                        "layout": st.target,
                        "device": page.device,
                        "paper": page.paper,
                        "landscape": page.landscape,
                        "scaleToFit": page.scale_to_fit,
                        "lineweights": page.lineweights,
                        "plotTransparency": page.plot_transparency,
                        "margins": page.margins_mm,
                        "scale": page.scale,
                        "center": page.center,
                        "plotArea": page.plot_area,
                        "plotStyleTable": page.plot_style_table,
                        "displayPlotStyles": page.show_plot_styles,
                    }),
                ));
            let mut print_sys = false;
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {"""

new = """        ui.horizontal(|ui| {
            let (cmd, label, hover) = if model {
                ("modelpagesetup", "Apply to Model", "Save these settings for Model space")
            } else {
                ("pagesetup", "Apply to Layout", "Save these settings in the layout's page setup")
            };
            if ui.button(label).on_hover_text(hover).clicked() {
                let page = &st.page;
                action = Some((
                    cmd,
                    json!({
                        "layout": st.target,
                        "device": page.device,
                        "paper": page.paper,
                        "landscape": page.landscape,
                        "scaleToFit": page.scale_to_fit,
                        "lineweights": page.lineweights,
                        "plotTransparency": page.plot_transparency,
                        "margins": page.margins_mm,
                        "scale": page.scale,
                        "center": page.center,
                        "plotArea": page.plot_area,
                        "plotStyleTable": page.plot_style_table,
                        "displayPlotStyles": page.show_plot_styles,
                    }),
                ));
            }
            let mut print_sys = false;
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')