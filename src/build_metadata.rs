/// Render managed build metadata from a recorded UTC source timestamp.
/// The caller supplies Cargo's package version and SOURCE_DATE_EPOCH; wall-clock
/// time and the machine's local timezone must not affect managed binaries.
pub fn render_version(version: &str, epoch: &str) -> Result<String, String> {
    let seconds = epoch
        .parse::<i64>()
        .map_err(|_| "SOURCE_DATE_EPOCH must be an integer Unix timestamp".to_owned())?;
    if seconds < 0 {
        return Err("SOURCE_DATE_EPOCH cannot be negative".to_owned());
    }
    let date = hbb_common::chrono::DateTime::<hbb_common::chrono::Utc>::from_timestamp(seconds, 0)
        .ok_or_else(|| "SOURCE_DATE_EPOCH is outside the supported date range".to_owned())?;
    Ok(format!(
        "pub const VERSION: &str = {version:?};\n#[allow(dead_code)]\npub const BUILD_DATE: &str = \"{}\";\n",
        date.format("%Y-%m-%d %H:%M")
    ))
}

#[cfg(test)]
mod tests {
    use super::render_version;

    #[test]
    fn source_timestamp_has_fixed_utc_date() {
        let metadata = render_version("1.5.0", "1704067200").unwrap();
        assert!(metadata.contains("VERSION: &str = \"1.5.0\""));
        assert!(metadata.contains("BUILD_DATE: &str = \"2024-01-01 00:00\""));
        assert_eq!(metadata, render_version("1.5.0", "1704067200").unwrap());
    }

    #[test]
    fn missing_invalid_and_out_of_range_epochs_are_rejected() {
        for epoch in ["", "yesterday", "-1", "9223372036854775807"] {
            assert!(render_version("1.5.0", epoch).is_err(), "{epoch}");
        }
    }

    #[test]
    fn cargo_version_is_escaped_as_a_rust_literal() {
        assert!(render_version("test\"\\\n", "0")
            .unwrap()
            .starts_with("pub const VERSION: &str = \"test\\\"\\\\\\n\";\n"));
    }
}
