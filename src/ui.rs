use tui::{
    layout::Alignment,
    style::Style,
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn build_main(content: &str) -> Paragraph {
    Paragraph::new(content)
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title(Span::styled("Diary - x to skip entry (suitable for skipping files), Ctrl+c to exit, Alt+n to submit entry", Style::default())),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
}

pub fn build_file_input(content: &str) -> Paragraph {
    Paragraph::new(content)
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title(Span::styled(
                    "Enter the file directory here (Esc to cancel)",
                    Style::default(),
                )),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
}

pub fn build_message(content: &str) -> Paragraph {
    Paragraph::new(content)
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title(Span::styled("Alert (Esc to go back)", Style::default())),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
}
