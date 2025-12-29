#[cfg(test)]
mod tests {
    use crate::pkg::Package;

    #[test]
    fn test_package_new_fields() {
        let pkg = Package {
            id: "vim;8.2.1234;x86_64;updates".to_string(),
            name: "vim".to_string(),
            version: "8.2.1234".to_string(),
            arch: "x86_64".to_string(),
            data: "updates".to_string(),
            status: "installed".to_string(),
            summary: "Vi IMproved, a programmers tool that is largely compatible with Vi".to_string(),
            description: "Vim is a text editor that is upwards compatible to Vi. It can be used to edit all kinds of plain text. It is especially useful for editing programs.".to_string(),
            license: "Vim".to_string(),
            size: 1234567,
            url: "https://www.vim.org/".to_string(),
        };

        assert_eq!(pkg.summary, "Vi IMproved, a programmers tool that is largely compatible with Vi");
        assert_eq!(pkg.license, "Vim");
        assert_eq!(pkg.size, 1234567);
        assert_eq!(pkg.url, "https://www.vim.org/");
    }

    #[test]
    fn test_from_packagekit_updated() {
        let pkg = Package::from_packagekit(
            "vim;8.2.1234;x86_64;updates",
            "installed",
            "Vi IMproved"
        );

        assert_eq!(pkg.name, "vim");
        assert_eq!(pkg.summary, "Vi IMproved");
        assert_eq!(pkg.description, ""); // Should be empty by default
        assert_eq!(pkg.license, "");
        assert_eq!(pkg.size, 0);
        assert_eq!(pkg.url, "");
    }

    #[test]
    fn test_update_details() {
        let mut pkg = Package::from_packagekit(
            "vim;8.2.1234;x86_64;updates",
            "installed",
            "Vi IMproved"
        );

        pkg.update_details(
            "Full description here",
            "Vim License",
            9999,
            "https://vim.org"
        );

        assert_eq!(pkg.description, "Full description here");
        assert_eq!(pkg.license, "Vim License");
        assert_eq!(pkg.size, 9999);
        assert_eq!(pkg.url, "https://vim.org");
    }
}
