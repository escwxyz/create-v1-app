use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::Mutex;
use tera::Tera;
use walkdir::WalkDir;

pub static TERA: Lazy<Arc<Mutex<Tera>>> = Lazy::new(|| Arc::new(Mutex::new(Tera::default())));

pub fn initialize_tera() -> Result<()> {
    let mut tera = TERA.lock().expect("Failed to lock Tera instance");

    // Walk through the templates directory and add only .tera files
    for entry in WalkDir::new("templates").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "tera")
        {
            let template_path = entry
                .path()
                .strip_prefix("templates")
                .expect("Failed to strip prefix");
            let template_name = template_path
                .to_str()
                .expect("Failed to convert template path to string");
            tera.add_template_file(entry.path(), Some(template_name))
                .map_err(|e| anyhow::anyhow!("Failed to add template file: {}", e))?;
        }
    }

    Ok(())
}
