with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old = """    }
    #[cfg(feature = "windows-print")]
    let print_native_clicked = print_native;
    #[cfg(not(feature = "windows-print"))]
    let print_native_clicked = false;
    if print_native_clicked {
        // Print to system printer using native dialog: generate PDF bytes and send to native print dialog
        let params = plot_params(&st, None);
        let hook = io().and_then(|h| h.plot).ok_or_else(|| "plotting not available".to_string());
        let hook_fn = match hook {
            Ok(f) => f,
            Err(e) => { app.session.echo(format!("플로팅 기능 없음: {}", e)); return action; }
        };
        match app.session.doc() {
            Ok(doc) => {
                let space = match st.target.as_str() {
                    "Model" => Space::Model,
                    name => Space::Paper(name.to_string()),
                };
                match hook_fn(doc, &space, &params) {
                    Ok(pdf_bytes) => {
                        let pdf_vec: Vec<u8> = pdf_bytes.to_vec();
                        // Get selected printer from device (if it's a system printer) or use default
                        let printer_name = if st.page.device != "CADCraft PDF" && st.page.device != "CADCraft PNG" && st.page.device != "CADCraft SVG" {
                            Some(st.page.device.as_str())
                        } else {
                            None
                        };
                        match print_pdf_to_system_printer_native(&pdf_vec, printer_name) {
                            Ok(_) => app.session.echo(format!("시스템 프린터(네이티브)로 출력 전송됨: {}", st.target)),
                            Err(e) => app.session.echo(format!("네이티브 프린터 출력 실패: {}", e)),
                        }
                    }
                    Err(e) => app.session.echo(format!("PDF 생성 실패: {}", e)),
                }
            }
            Err(e) => app.session.echo(format!("도면 없음: {}", e)),
        }
        return action;
    }
    if ok {"""

new = """    }
    if ok {"""

content = content.replace(old, new)

with open(r'C:\vibecode\cadcraft\crates\ui-egui\src\managers.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed')