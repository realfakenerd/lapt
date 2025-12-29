#[cfg(test)]
mod tests {
    use crate::pkg::Package;
    use crate::ui_details::render_details;
    use ratatui::{backend::TestBackend, Terminal, layout::Rect};

    #[test]
    fn test_render_details_content() {
        let mut pkg = Package::from_packagekit("vim;8.2;x64;repo", "installed", "Vi IMproved");
        pkg.update_details("A great editor", "Vim", 1024, "https://vim.org");

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 20);
            render_details(f, area, Some(&pkg));
        }).unwrap();

        // Note: Exact matching might be tricky with wrapping and dynamic content, 
        // but we can check for specific substrings in the output buffer.
        
        let actual_buffer = terminal.backend().buffer();
        // Check for key fields
        let buffer_string = format!("{:?}", actual_buffer);
        assert!(buffer_string.contains("Name: vim"));
        assert!(buffer_string.contains("Version: 8.2"));
        assert!(buffer_string.contains("A great editor"));
    }
}
