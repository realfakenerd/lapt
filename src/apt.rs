use crate::pkg::Package;
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

pub fn list_installed() -> Result<Vec<Package>> {
    let output = Command::new("apt")
        .args(&["list", "--installed"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(parse_apt_list(&stdout, "Installed"))
}

pub fn list_upgradable() -> Result<Vec<Package>> {
    let output = Command::new("apt")
        .args(&["list", "--upgradable"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(parse_apt_list(&stdout, "Update"))
}

pub fn search_packages(query: &str) -> Result<Vec<Package>> {
    let output = Command::new("apt")
        .args(&["search", query])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(parse_apt_search(&stdout))
}

pub fn install_package(package_name: &str) -> Result<()> {
    let output = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(&["install", "-y", package_name])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("apt-get install failed: {}", err)
    }
}

pub fn remove_package(package_name: &str) -> Result<()> {
    let output = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(&["remove", "-y", package_name])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("apt-get remove failed: {}", err)
    }
}

pub fn reinstall_package(package_name: &str) -> Result<()> {
    let output = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(&["install", "--reinstall", "-y", package_name])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("apt-get reinstall failed: {}", err)
    }
}

pub fn update_repos() -> Result<()> {
    let output = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(&["update"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("apt-get update failed: {}", err)
    }
}

pub fn upgrade_system() -> Result<()> {
    let output = Command::new("apt-get")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .args(&["dist-upgrade", "-y"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("apt-get dist-upgrade failed: {}", err)
    }
}

fn parse_apt_search(output: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    let mut lines = output.lines().peekable();

    while let Some(line) = lines.next() {
        if line.is_empty()
            || line.starts_with("Sorting...")
            || line.starts_with("Full Text Search...")
            || line.starts_with("WARNING:")
        {
            continue;
        }

        // Format: package/release version architecture
        // Example: acr/noble 2.1.2-1 all
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let name_release = parts[0];
        let version = parts[1];
        let arch = parts[2];

        let name = name_release.split('/').next().unwrap_or(name_release);

        // Peek next line for summary
        let mut summary = String::new();
        if let Some(next_line) = lines.peek() {
            if next_line.starts_with("  ") {
                summary = next_line.trim().to_string();
                lines.next(); // Consume the summary line
            }
        }

        // Reconstruct ID: name;version;arch;data
        let id = format!("{};{};{};{}", name, version, arch, "apt");

        let mut pkg = Package::from_packagekit(&id, "Available", &summary);
        pkg.status = "Available".to_string();
        packages.push(pkg);
    }

    packages
}

fn parse_apt_list(output: &str, status: &str) -> Vec<Package> {
    let mut packages = Vec::new();

    for line in output.lines() {
        if line.is_empty() || line.starts_with("Listing...") || line.starts_with("WARNING:") {
            continue;
        }

        // Format: package/release version architecture [status]
        // Example: adduser/noble,now 3.137ubuntu1 all [installed,automatic]
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let name_release = parts[0];
        let version = parts[1];
        let arch = parts[2];

        let name = name_release.split('/').next().unwrap_or(name_release);

        // Reconstruct ID: name;version;arch;data
        let id = format!("{};{};{};{}", name, version, arch, "apt");

        let mut pkg = Package::from_packagekit(&id, status, "");
        pkg.status = status.to_string(); // Ensure status matches what we passed
        packages.push(pkg);
    }

    packages
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

    #[test]
    fn test_parse_apt_list() {
        let output = r#"
Listing...
adduser/noble,now 3.137ubuntu1 all [installed,automatic]
alsa-base/noble,now 1.0.25+dfsg-0ubuntu7 all [installed]
"#;
        let pkgs = parse_apt_list(output, "Installed");
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].name, "adduser");
        assert_eq!(pkgs[0].version, "3.137ubuntu1");
        assert_eq!(pkgs[0].arch, "all");
        assert_eq!(pkgs[0].status, "Installed");

        assert_eq!(pkgs[1].name, "alsa-base");
    }

    #[test]
    fn test_parse_apt_search() {
        let output = r#"
Sorting...
Full Text Search...
acr/noble 2.1.2-1 all
  autoconf like tool

aerc/noble-updates 0.17.0-1 amd64
  Pretty Good Email Client
"#;
        let pkgs = parse_apt_search(output);
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].name, "acr");
        assert_eq!(pkgs[0].summary, "autoconf like tool");
        assert_eq!(pkgs[1].name, "aerc");
        assert_eq!(pkgs[1].summary, "Pretty Good Email Client");
    }
}
