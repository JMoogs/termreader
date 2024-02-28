use ratatui::prelude::*;
/// helper function to create a centered rect using up a certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn centered_sized_rect(width: u16, height: u16, r: Rect) -> Rect {
    let vert_pct = (100.0 * width as f64 / r.width as f64).ceil() as u16;
    let hori_pct = (100.0 * height as f64 / r.height as f64).ceil() as u16;
    centered_rect(vert_pct, hori_pct, r)
}
