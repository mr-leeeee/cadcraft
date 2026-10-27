with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = 'layout: match space { Space::Model => Model.to_string(), Space::Paper(n) => n },'
new = '        "layout": match space { Space::Model => "Model".to_string(), Space::Paper(n) => n },'

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')