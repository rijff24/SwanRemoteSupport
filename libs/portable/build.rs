#[cfg(windows)]
mod resource_order;

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../res/icon.ico")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .set_manifest_file("../../res/manifest.xml");
        // winres 0.1.12 writes both metadata maps in randomized iteration order.
        // Order the generated source before resource compilation, never the PE.
        let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Cargo OUT_DIR is required")
        })?).join("portable-reproducible.rc");
        res.write_resource_file(&output)?;
        let generated = std::fs::read_to_string(&output)?;
        std::fs::write(&output, resource_order::stable_resource_source(&generated))?;
        let path = output.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Resource path is not UTF-8")
        })?;
        res.set_resource_file(path).compile()?;
    }
    Ok(())
}
