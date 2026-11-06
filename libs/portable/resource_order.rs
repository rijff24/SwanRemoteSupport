//! Stabilize winres-generated metadata before invoking the resource compiler.
//! Keep each contiguous metadata block separate, including language/translation
//! blocks, and leave icons, manifests, strings and other source text intact.

#[derive(Clone, Copy, PartialEq, Eq)]
enum Metadata { Version, Value }

fn metadata(line: &str) -> Option<Metadata> {
    let line = line.trim_start();
    if line.starts_with("VALUE ") {
        Some(Metadata::Value)
    } else if ["FILEVERSION ", "PRODUCTVERSION ", "FILEFLAGSMASK ", "FILEFLAGS ",
        "FILEOS ", "FILETYPE ", "FILESUBTYPE "].iter().any(|prefix| line.starts_with(prefix)) {
        Some(Metadata::Version)
    } else {
        None
    }
}

pub fn stable_resource_source(source: &str) -> String {
    let lines: Vec<_> = source.lines().collect();
    let mut output = String::with_capacity(source.len());
    let mut start = 0;
    while start < lines.len() {
        let mut end = start + 1;
        if let Some(kind) = metadata(lines[start]) {
            while end < lines.len() && metadata(lines[end]) == Some(kind) {
                end += 1;
            }
            let mut group = lines[start..end].to_vec();
            group.sort_unstable();
            for line in group {
                output.push_str(line);
                output.push('\n');
            }
        } else {
            output.push_str(lines[start]);
            output.push('\n');
        }
        start = end;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::stable_resource_source;

    #[test]
    fn generated_metadata_matches_across_map_iteration_orders() {
        let first = "#pragma code_page(65001)\n1 VERSIONINFO\nPRODUCTVERSION 1, 5, 0, 0\nFILEVERSION 1, 5, 0, 0\nFILEOS 0x40004\n{\nBLOCK \"StringFileInfo\"\n{\nBLOCK \"040904b0\"\n{\nVALUE \"ProductName\", \"RustDesk\"\nVALUE \"LegalCopyright\", \"Copyright © 2026\"\nVALUE \"OriginalFilename\", \"rustdesk.exe\"\n}\n}\nBLOCK \"VarFileInfo\" {\nVALUE \"Translation\", 0x409, 0x04b0\n}\n}\n1 ICON \"../../res/icon.ico\"\n1 24 \"../../res/manifest.xml\"\n\n";
        let second = first.replace("PRODUCTVERSION 1, 5, 0, 0\nFILEVERSION 1, 5, 0, 0\nFILEOS 0x40004", "FILEOS 0x40004\nFILEVERSION 1, 5, 0, 0\nPRODUCTVERSION 1, 5, 0, 0")
            .replace("VALUE \"ProductName\", \"RustDesk\"\nVALUE \"LegalCopyright\", \"Copyright © 2026\"\nVALUE \"OriginalFilename\", \"rustdesk.exe\"", "VALUE \"OriginalFilename\", \"rustdesk.exe\"\nVALUE \"ProductName\", \"RustDesk\"\nVALUE \"LegalCopyright\", \"Copyright © 2026\"");
        assert_ne!(first, second);
        assert_eq!(stable_resource_source(first), stable_resource_source(&second));
    }

    #[test]
    fn keeps_language_and_translation_blocks_separate() {
        let input = "BLOCK \"040904b0\"\n{\nVALUE \"Z\", \"English\"\nVALUE \"A\", \"English\"\n}\nBLOCK \"040704b0\"\n{\nVALUE \"Z\", \"German\"\nVALUE \"A\", \"German\"\n}\nBLOCK \"VarFileInfo\" {\nVALUE \"Translation\", 0x409, 0x04b0\n}\n";
        let expected = "BLOCK \"040904b0\"\n{\nVALUE \"A\", \"English\"\nVALUE \"Z\", \"English\"\n}\nBLOCK \"040704b0\"\n{\nVALUE \"A\", \"German\"\nVALUE \"Z\", \"German\"\n}\nBLOCK \"VarFileInfo\" {\nVALUE \"Translation\", 0x409, 0x04b0\n}\n";
        assert_eq!(stable_resource_source(input), expected);
    }

    #[test]
    fn preserves_escaped_values_icons_and_manifest_resources() {
        let input = "#pragma code_page(65001)\nVALUE \"A\", \"quoted \\\"VALUE text\\\" ©\"\nVALUE \"Z\", \"C:\\\\test\"\n1 ICON \"../../res/icon.ico\"\n1 24\n{\n\" <assembly>VALUE untouched</assembly> \"\n}\n";
        assert_eq!(stable_resource_source(input), input);
    }

    #[test]
    fn ordering_is_idempotent() {
        let input = "VALUE \"Z\", \"z\"\nVALUE \"A\", \"a\"\n";
        let ordered = stable_resource_source(input);
        assert_eq!(stable_resource_source(&ordered), ordered);
    }
}
