with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = """});
    });
    let print_sys_clicked = print_sys;
    let print_native_clicked = print_native;
    if should_close {"""

new = """});
    });
    let print_sys_clicked = print_sys;
    if should_close {"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')