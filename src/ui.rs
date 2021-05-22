use tui::{
    layout::Alignment,
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn build_main(content: &str) -> Paragraph {
    Paragraph::new(content)
                        .style(Style::default())
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .style(Style::default())
                                .title(Span::styled("Gremlin", Style::default())),
                        )
                        .alignment(Alignment::Left)
                        .wrap(Wrap {trim: false})
}

pub fn build_input(content: String) -> Paragraph<'static> {
    Paragraph::new(content)
                        .style(Style::default())
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .style(Style::default())
                                .title(Span::styled("Enter your URI/URL here", Style::default())),
                        )
                        .alignment(Alignment::Left)
                        .wrap(Wrap {trim: false})
}
