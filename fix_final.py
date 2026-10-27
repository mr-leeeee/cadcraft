with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find and fix the print_sys line (around line 1566)
for i, line in enumerate(lines):
    if 'print_sys = ui.button("프린터 출력")' in line:
        # Replace with the correct version
        lines[i] = '            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();\n'
        lines.insert(i + 1, '            let print_sys = ui.button("프린터 출력").on_hover_text("시스템 프린터로 직접 출력 (PDF 생성 후 OS 프린트 명령 위임)").clicked();\n')
        break

# Fix the if print_sys check later (around line 1601)
for i, line in enumerate(lines):
    if '    if print_sys {' in line and i > 1570:
        lines[i] = '    if print_sys_clicked {\n'
        break

# Add the print_sys_clicked variable after the window closes
# Find the line "if should_close {" after the window closes
for i, line in enumerate(lines):
    if 'if should_close {' in line and i > 1570:
        # Insert before this line
        lines.insert(i, '    let print_sys_clicked = print_sys;\n')
        break

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.writelines(lines)
print('Fixed')