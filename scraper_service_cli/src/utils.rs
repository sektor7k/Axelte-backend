use std::fs::File;
use std::io::Write;

pub fn save_json(data: &serde_json::Value, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;
    file.write_all(serde_json::to_string_pretty(data)?.as_bytes())?;
    println!("✅ {} oluşturuldu.", filename);
    Ok(())
}

pub fn save_text(content: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    println!("✅ {} oluşturuldu.", filename);
    Ok(())
} 