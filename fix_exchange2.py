with open(r'C:\vibecode\cadcraft\crates\engine\src\cmd\exchange.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = 'Ok(json!({ message: Use\n#[cfg(test)]\nmod tests {'
new = 'Ok(json!({ "message": "Use the Multi-Plot dialog (menu File \u25b8 Multi-Plot... or type MULTIPLOT)" }))\n}\n\n#[cfg(test)]\nmod tests {'

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\engine\src\cmd\exchange.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')