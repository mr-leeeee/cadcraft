with open(r'C:\vibecode\cadcraft\crates\engine\src\cmd\exchange.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = """}
}

/// MULTIPLOT command: opens the multi-plot dialog (UI only, no-op in JSON mode).
fn run_multiplot(_s: &mut Session, _p: &Value) -> Result<Value> {
    Ok(json!({ message: Use
#[cfg(test)]
mod tests {"""

new = """}
}

/// MULTIPLOT command: opens the multi-plot dialog (UI only, no-op in JSON mode).
fn run_multiplot(_s: &mut Session, _p: &Value) -> Result<Value> {
    Ok(json!({ "message": "Use the Multi-Plot dialog (menu File ▸ Multi-Plot... or type MULTIPLOT)" }))
}

#[cfg(test)]
mod tests {"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\engine\src\cmd\exchange.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')