with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Move print_sys declaration BEFORE the Window::show call
old = """    set_temp(ctx, PLOT_NOW, false);
    let (mut cancel, mut action, mut pick, mut preview) = (false, None, false, false);
    let mut should_close = false;
    let center = ctx.content_rect().center();
    egui::Window::new("Plot").open(open).collapsible(false).resizable(false).pivot(egui::Align2::CENTER_CENTER).default_pos(center).show(ctx, |ui| {"""

new = """    set_temp(ctx, PLOT_NOW, false);
    let (mut cancel, mut action, mut pick, mut preview) = (false, None, false, false);
    let mut should_close = false;
    let mut print_sys = false;
    let center = ctx.content_rect().center();
    egui::Window::new("Plot").open(open).collapsible(false).resizable(false).pivot(egui::Align2::CENTER_CENTER).default_pos(center).show(ctx, |ui| {"""

content = content.replace(old, new)

# Also remove the duplicate declaration inside the closure
old2 = """        ui.separator();
        let mut print_sys = false;
        ui.horizontal(|ui| {"""

new2 = """        ui.separator();
        ui.horizontal(|ui| {"""

content = content.replace(old2, new2)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')