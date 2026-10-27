with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix the missing closing brace for the Apply button
old = """            if ui.button(label).on_hover_text(hover).clicked() {
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
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {"""

new = """            if ui.button(label).on_hover_text(hover).clicked() {
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
            // Multi-Plot button
            if ui.button("Multi-Plot").on_hover_text("Open batch plot dialog for multiple sheets").clicked() {"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')