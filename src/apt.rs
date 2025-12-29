use anyhow::Result;
use std::process::Command;

#[derive(Debug, Default, PartialEq)]
pub struct AptDetails {
    pub description: String,
    pub license: String,
    pub size: u64,
    pub url: String,
}

pub fn get_package_details(package_name: &str) -> Result<AptDetails> {
    let output = Command::new("apt")
        .args(&["show", package_name])
        .output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    Ok(parse_apt_show(&stdout))
}

fn parse_apt_show(output: &str) -> AptDetails {
    let mut details = AptDetails::default();
    let mut in_description = false;

    for line in output.lines() {
        if line.starts_with("Homepage: ") {
            details.url = line.trim_start_matches("Homepage: ").trim().to_string();
            in_description = false;
        } else if line.starts_with("Installed-Size: ") {
             let size_str = line.trim_start_matches("Installed-Size: ").trim();
             details.size = parse_size(size_str);
             in_description = false;
        } else if line.starts_with("Description: ") {
            details.description = line.trim_start_matches("Description: ").trim().to_string();
            in_description = true;
        } else if in_description && line.starts_with(" ") {
            if !details.description.is_empty() {
                details.description.push('\n');
            }
            details.description.push_str(line.trim());
        } else if !line.starts_with(" ") && !line.is_empty() && in_description {
            in_description = false;
        }
    }
    
    if details.license.is_empty() {
         details.license = "Unknown".to_string();
    }

    details
}

fn parse_size(size_str: &str) -> u64 {
    let parts: Vec<&str> = size_str.split_whitespace().collect();
    if parts.is_empty() { return 0; }
    
    let val: f64 = parts[0].parse().unwrap_or(0.0);
    let unit = if parts.len() > 1 { parts[1] } else { "B" };
    
    let multiplier = match unit {
        "kB" | "KB" => 1024.0,
        "MB" => 1024.0 * 1024.0,
        "GB" => 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    
    (val * multiplier) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_apt_show() {
        let output = r#"Package: vim
Version: 2:9.1.0016-1ubuntu7.9
Installed-Size: 4230 kB
Homepage: https://www.vim.org/
Description: Vi IMproved - enhanced vi editor
 Vim is an almost compatible version of the UNIX editor Vi.
 .
 Many new features have been added.
"#;
        let details = parse_apt_show(output);
        assert_eq!(details.url, "https://www.vim.org/");
        assert_eq!(details.size, 4331520); // 4230 * 1024
        assert!(details.description.contains("Vi IMproved - enhanced vi editor"));
        assert!(details.description.contains("Many new features have been added"));
    }
}
