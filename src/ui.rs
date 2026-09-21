//! Drawing the game with ratatui.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::app::App;
use crate::game::{self, Status};
use crate::theme::{Theme, lighten};

/// Width of a single tile, in cells.
const TILE_W: u16 = 7;
/// Height of a single tile, in cells.
const TILE_H: u16 = 3;
/// Space between two tiles.
const GAP: u16 = 1;

/// The playfield, without the outline.
const BOARD_W: u16 = game::SIZE as u16 * TILE_W + (game::SIZE as u16 - 1) * GAP;
const BOARD_H: u16 = game::SIZE as u16 * TILE_H + (game::SIZE as u16 - 1) * GAP;
/// The playfield plus its one-cell outline.
const BOARD_OUTER_W: u16 = BOARD_W + 2;
const BOARD_OUTER_H: u16 = BOARD_H + 2;

const HEADER_H: u16 = 3;
const FOOTER_H: u16 = 2;
const SPACER: u16 = 1;
/// Height of everything the game needs, excluding the centring margins.
const CONTENT_H: u16 = HEADER_H + SPACER + BOARD_OUTER_H + SPACER + FOOTER_H;

/// The narrowest terminal that can show the whole game.
pub const MIN_W: u16 = BOARD_OUTER_W;
/// The shortest terminal that can show the whole game.
pub const MIN_H: u16 = CONTENT_H;

/// Renders the whole screen.
pub fn draw(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();

    if area.width < MIN_W || area.height < MIN_H {
        draw_too_small(frame, area, theme);
        return;
    }

    // Centre the fixed-size game vertically, then lay out its three bands.
    let [_, content, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(CONTENT_H),
        Constraint::Fill(1),
    ])
    .areas(area);

    let [header_slot, _, board_slot, _, footer_slot] = Layout::vertical([
        Constraint::Length(HEADER_H),
        Constraint::Length(SPACER),
        Constraint::Length(BOARD_OUTER_H),
        Constraint::Length(SPACER),
        Constraint::Length(FOOTER_H),
    ])
    .areas(content);

    let x = content.x + content.width.saturating_sub(BOARD_OUTER_W) / 2;
    let header = Rect::new(x, header_slot.y, BOARD_OUTER_W, HEADER_H);
    let board = Rect::new(x, board_slot.y, BOARD_OUTER_W, BOARD_OUTER_H);
    // The footer spans the whole terminal rather than the board, so a wide
    // window can show the longer hint line. It is centred like the board, so
    // the two still share an axis.
    let footer = Rect::new(content.x, footer_slot.y, content.width, FOOTER_H);

    draw_header(frame, header, app, theme);
    draw_board(frame, board, app, theme);
    draw_footer(frame, footer, app, theme);

    // Overlays are centred on the board and span its full width, so they sit
    // exactly inside the frame instead of clipping one of its edges.
    match app.game().status() {
        Status::Won => draw_win(frame, board, app, theme),
        Status::Lost => draw_game_over(frame, board, app, theme),
        Status::Playing => {}
    }

    if app.show_help() {
        draw_help(frame, board, theme);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let [title_area, scores] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(23)]).areas(area);

    // The title sits on the same row as the score digits.
    let title_row = Rect::new(
        title_area.x,
        title_area.y + area.height / 2,
        title_area.width,
        1,
    );
    let title = Line::styled(
        "2048",
        Style::new().fg(theme.accent()).add_modifier(Modifier::BOLD),
    );
    frame.render_widget(Paragraph::new(title), title_row);

    let [score_area, _, best_area] = Layout::horizontal([
        Constraint::Length(11),
        Constraint::Length(1),
        Constraint::Length(11),
    ])
    .areas(scores);

    frame.render_widget(
        score_box("SCORE", app.game().score(), theme.accent(), theme),
        score_area,
    );
    frame.render_widget(
        score_box("BEST", app.game().best(), theme.accent(), theme),
        best_area,
    );
}

fn score_box<'a>(
    label: &'a str,
    value: u64,
    value_color: ratatui::style::Color,
    theme: &Theme,
) -> Paragraph<'a> {
    let block = Block::bordered()
        .title(label)
        .title_alignment(Alignment::Center)
        .title_style(Style::new().fg(theme.dim()))
        .border_style(Style::new().fg(theme.border()))
        .style(Style::new().bg(theme.panel()));

    Paragraph::new(Line::styled(
        value.to_string(),
        Style::new().fg(value_color).add_modifier(Modifier::BOLD),
    ))
    .block(block)
    .alignment(Alignment::Center)
}

fn draw_board(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::bordered()
        .border_style(Style::new().fg(theme.border()))
        .style(Style::new().bg(theme.board()));
    frame.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));
    let spawned = app.game().spawned();
    let flashing = app.flash() > 0;

    for row in 0..game::SIZE {
        for col in 0..game::SIZE {
            let tile = Rect::new(
                inner.x + col as u16 * (TILE_W + GAP),
                inner.y + row as u16 * (TILE_H + GAP),
                TILE_W,
                TILE_H,
            );
            let is_new = flashing && spawned == Some((row, col));
            draw_tile(frame, tile, app.game().tile(row, col), is_new, theme);
        }
    }
}

fn draw_tile(frame: &mut Frame, area: Rect, value: u32, flashing: bool, theme: &Theme) {
    let (background, foreground) = theme.tile(value);
    let background = if flashing {
        lighten(background, 45)
    } else {
        background
    };
    let style = Style::new().bg(background).fg(foreground);

    // Paint the whole tile with its background, including the cells the number
    // will not cover.
    {
        let buffer = frame.buffer_mut();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buffer[(x, y)].set_symbol(" ").set_style(style);
            }
        }
    }

    if value == 0 {
        return;
    }

    let style = if flashing {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    };
    let label = Rect::new(area.x, area.y + area.height / 2, area.width, 1);
    let line = Line::styled(value.to_string(), style);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), label);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let [status_row, hint_row] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

    frame.render_widget(
        Paragraph::new(Line::styled(status_text(app), Style::new().fg(theme.dim())))
            .alignment(Alignment::Center),
        status_row,
    );

    // vim puts a half-typed count at the bottom right; so do we.
    let [hint_area, count_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(COUNT_W)]).areas(hint_row);

    frame.render_widget(
        Paragraph::new(Line::styled(
            hints(hint_area.width),
            Style::new().fg(theme.dim()),
        ))
        .alignment(Alignment::Center),
        hint_area,
    );

    if let Some(count) = app.pending_count() {
        frame.render_widget(
            Paragraph::new(Line::styled(
                count.to_string(),
                Style::new().fg(theme.accent()).add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Right),
            count_area,
        );
    }
}

/// Room reserved on the right of the hint row for a pending count.
const COUNT_W: u16 = 4;

/// The one-line summary under the board.
fn status_text(app: &App) -> String {
    let game = app.game();
    summary(game.status(), game.moves(), game.max_tile(), game.score())
}

/// Formats the status line, split out so it can be tested at its widest.
fn summary(status: Status, moves: u64, top_tile: u32, score: u64) -> String {
    match status {
        Status::Playing => format!("moves {moves} · top tile {top_tile}"),
        Status::Won => "you reached 2048".to_string(),
        Status::Lost => format!("game over · score {score}"),
    }
}

/// The key hints, shortened to fit the space available.
fn hints(width: u16) -> &'static str {
    const FULL: &str = "hjkl move · u undo · Ctrl-r redo · K help · ZZ quit";
    const SHORT: &str = "hjkl move · u undo · q quit";
    if width >= FULL.chars().count() as u16 {
        FULL
    } else {
        SHORT
    }
}

fn draw_win(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // `area` is the board: overlays are centred on it, not on the screen.
    let lines = vec![
        Line::from(""),
        Line::styled(
            "You win!",
            Style::new().fg(theme.accent()).add_modifier(Modifier::BOLD),
        ),
        Line::styled(
            format!("score {}", app.game().score()),
            Style::new().fg(theme.text()),
        ),
        Line::from(""),
        Line::styled("c  keep playing", Style::new().fg(theme.text())),
        Line::styled("r  restart", Style::new().fg(theme.text())),
        Line::styled("q  quit", Style::new().fg(theme.text())),
    ];
    draw_popup(frame, area, theme, " 2048 ", lines, Alignment::Center);
}

fn draw_game_over(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // `area` is the board: overlays are centred on it, not on the screen.
    let lines = vec![
        Line::from(""),
        Line::styled(
            "Game over",
            Style::new().fg(theme.accent()).add_modifier(Modifier::BOLD),
        ),
        Line::styled(
            format!("score {}", app.game().score()),
            Style::new().fg(theme.text()),
        ),
        Line::styled(
            format!("best  {}", app.game().best()),
            Style::new().fg(theme.dim()),
        ),
        Line::from(""),
        Line::styled(
            if app.game().can_undo() {
                "r  restart      u  undo"
            } else {
                "r  restart"
            },
            Style::new().fg(theme.text()),
        ),
        Line::styled("q  quit", Style::new().fg(theme.text())),
    ];
    draw_popup(frame, area, theme, " game over ", lines, Alignment::Center);
}

fn draw_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    // Left aligned with a small indent, so the keys line up in a column.
    let key = |k: &'static str, what: &'static str| {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{k:<8}"),
                Style::new().fg(theme.accent()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(what, Style::new().fg(theme.text())),
        ])
    };

    let lines = vec![
        Line::from(""),
        key("h j k l", "move the tiles"),
        key("←↑↓→", "move the tiles"),
        key("[count]", "repeat: 3j, 3u"),
        Line::from(""),
        key("u", "undo"),
        key("Ctrl-r", "redo"),
        key("r", "restart the game"),
        key("c", "keep playing"),
        Line::from(""),
        key("K or ?", "this help"),
        key("ZZ", "save and quit"),
        key("ZQ", "quit, discard best"),
        key("q", "quit"),
        Line::from(""),
    ];
    draw_popup(frame, area, theme, " how to play ", lines, Alignment::Left);
}

/// Draws a box over the board, sized so that no line is ever clipped.
///
/// The box spans the board's full width, so its borders line up with the
/// board's, and it is centred inside `board` vertically.
fn draw_popup(
    frame: &mut Frame,
    board: Rect,
    theme: &Theme,
    title: &str,
    lines: Vec<Line<'static>>,
    alignment: Alignment,
) {
    // Spanning the board exactly keeps the left and right edges aligned with
    // it; anything narrower would leave one border column of the board poking
    // out on one side only.
    let width = board.width;
    let height = (lines.len() as u16 + 2).min(board.height);
    let popup = centered(board, width, height);
    frame.render_widget(Clear, popup);

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(Style::new().fg(theme.accent()).add_modifier(Modifier::BOLD))
        .border_style(Style::new().fg(theme.accent()))
        .style(Style::new().bg(theme.panel()));

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(block)
            .alignment(alignment),
        popup,
    );
}

fn draw_too_small(frame: &mut Frame, area: Rect, theme: &Theme) {
    let lines = vec![
        Line::styled(
            "terminal too small",
            Style::new().fg(theme.accent()).add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::styled(
            format!("need {MIN_W} x {MIN_H}"),
            Style::new().fg(theme.text()),
        ),
        Line::styled(
            format!("have {} x {}", area.width, area.height),
            Style::new().fg(theme.dim()),
        ),
    ];
    let popup = centered(area, 24.min(area.width), 4.min(area.height));
    frame.render_widget(
        Paragraph::new(Text::from(lines)).alignment(Alignment::Center),
        popup,
    );
}

/// Centres a `width` x `height` box inside `area`, never overflowing it.
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Direction, Game, SIZE};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// Renders the app and returns the screen as text, one line per row.
    fn render(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let theme = Theme::detect();
        terminal.draw(|frame| draw(frame, app, &theme)).unwrap();

        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    fn press(app: &mut App, code: KeyCode) {
        app.on_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    /// Counts all-digit words inside the board area only, so the title and the
    /// SCORE/BEST values cannot be mistaken for tiles.
    fn numbers_on(screen: &str) -> usize {
        let height = screen.lines().count() as u16;
        let content_top = height.saturating_sub(CONTENT_H) / 2;
        let board_top = (content_top + HEADER_H + SPACER) as usize;

        screen
            .lines()
            .skip(board_top)
            .take(BOARD_OUTER_H as usize)
            .flat_map(|line| line.split_whitespace())
            .filter(|word| word.chars().all(|c| c.is_ascii_digit()))
            .count()
    }

    #[test]
    fn the_board_fits_a_standard_terminal() {
        // 80x24 is the classic default; the game must not need more than that.
        const { assert!(MIN_W <= 80, "the game needs more than 80 columns") };
        const { assert!(MIN_H <= 24, "the game needs more than 24 rows") };
    }

    #[test]
    fn a_new_game_renders_its_starting_tiles() {
        let app = App::with_seed(1);
        let screen = render(&app, 80, 24);
        // Only board digits are counted, so the title and score boxes do not
        // inflate this.
        assert!(
            numbers_on(&screen) >= 2,
            "expected tiles on screen:\n{screen}"
        );
        assert!(screen.contains("2048"), "missing title:\n{screen}");
        assert!(screen.contains("SCORE"));
        assert!(screen.contains("BEST"));
    }

    #[test]
    fn the_board_grows_as_tiles_merge() {
        let mut app = App::with_seed(3);
        let mut best = numbers_on(&render(&app, 80, 24));
        for step in 0..40 {
            let keys = [KeyCode::Left, KeyCode::Down, KeyCode::Right, KeyCode::Up];
            press(&mut app, keys[step % keys.len()]);
            best = best.max(numbers_on(&render(&app, 80, 24)));
        }
        // The board fills up, so many tiles end up on screen.
        assert!(best >= 8, "board never filled up, peaked at {best} tiles");
    }

    #[test]
    fn a_small_terminal_gets_an_explanation_instead_of_a_broken_board() {
        let app = App::with_seed(1);
        let screen = render(&app, 20, 8);
        assert!(
            screen.contains("too small"),
            "expected a warning:\n{screen}"
        );
    }

    #[test]
    fn the_narrowest_supported_terminal_still_renders_the_board() {
        let app = App::with_seed(1);
        let screen = render(&app, MIN_W, MIN_H);
        assert!(!screen.contains("too small"));
        assert!(screen.contains("2048"));
        assert!(screen.contains("SCORE"));
    }

    #[test]
    fn the_win_overlay_explains_how_to_continue() {
        let mut game = Game::from_grid([[1024, 1024, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]], 0);
        assert!(game.slide(Direction::Left));
        assert_eq!(game.status(), Status::Won);

        let screen = render(&App::with_game(game), 80, 24);
        assert!(screen.contains("You win!"), "{screen}");
        assert!(screen.contains("keep playing"), "{screen}");
    }

    #[test]
    fn the_game_over_overlay_shows_the_scores() {
        let game = Game::from_grid(
            [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]],
            777,
        );
        assert_eq!(game.status(), Status::Lost);

        let screen = render(&App::with_game(game), 80, 24);
        assert!(screen.contains("Game over"), "{screen}");
        assert!(screen.contains("restart"), "{screen}");
    }

    #[test]
    fn the_help_overlay_lists_the_keys() {
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('?'));
        assert!(app.show_help());

        let screen = render(&app, 80, 24);
        assert!(screen.contains("how to play"), "{screen}");
        assert!(screen.contains("undo"), "{screen}");
        assert!(screen.contains("quit"), "{screen}");
    }

    #[test]
    fn the_help_overlay_is_not_clipped() {
        // The whole point of the overlay is that no line is cut off, including
        // the longest one, even on the smallest supported terminal.
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('?'));

        for width in [MIN_W, 40, 80, 120] {
            let screen = render(&app, width, MIN_H.max(24));
            for expected in [
                "move the tiles",
                "repeat: 3j",
                "undo",
                "redo",
                "restart the game",
                "keep playing",
                "save and quit",
                "quit, discard best",
            ] {
                assert!(
                    screen.contains(expected),
                    "`{expected}` was clipped at width {width}:\n{screen}"
                );
            }
        }
    }

    #[test]
    fn the_help_overlay_documents_the_vim_keys() {
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('K'));
        let screen = render(&app, 80, 24);

        for expected in ["h j k l", "Ctrl-r", "ZZ", "ZQ", "[count]"] {
            assert!(
                screen.contains(expected),
                "the help does not mention `{expected}`:\n{screen}"
            );
        }
        assert!(
            !screen.contains("wasd"),
            "wasd is no longer a binding:\n{screen}"
        );
    }

    #[test]
    fn the_end_screens_are_not_clipped() {
        let mut game = Game::from_grid([[1024, 1024, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]], 0);
        assert!(game.slide(Direction::Left));
        let won = render(&App::with_game(game), MIN_W, MIN_H);
        for expected in ["You win!", "score 2048", "keep playing", "restart", "quit"] {
            assert!(won.contains(expected), "`{expected}` clipped:\n{won}");
        }

        let lost = render(
            &App::with_game(Game::from_grid(
                [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]],
                123_456,
            )),
            MIN_W,
            MIN_H,
        );
        for expected in ["Game over", "score 0", "best  123456", "restart", "quit"] {
            assert!(lost.contains(expected), "`{expected}` clipped:\n{lost}");
        }
    }

    #[test]
    fn the_footer_is_never_clipped() {
        // Render at the narrowest supported width and make sure the status
        // line and the hints both survive intact.
        let app = App::with_seed(21);
        let screen = render(&app, MIN_W, MIN_H);
        assert!(screen.contains(&status_text(&app)), "{screen}");
        assert!(screen.contains(hints(MIN_W - COUNT_W)), "{screen}");

        let mut app = App::with_seed(21);
        for step in 0..60 {
            let keys = [KeyCode::Left, KeyCode::Down, KeyCode::Right, KeyCode::Up];
            press(&mut app, keys[step % keys.len()]);
            let screen = render(&app, MIN_W, MIN_H);
            let expected = status_text(&app);
            assert!(
                screen.contains(&expected),
                "clipped `{expected}`:\n{screen}"
            );
        }
    }

    #[test]
    fn the_status_line_fits_the_board_at_realistic_extremes() {
        // Even a very long game stays inside the 33-column footer. The bounds
        // below are far beyond anything reachable in practice.
        let cases = [
            (Status::Playing, 99_999_999, 131_072, 0),
            (Status::Won, 1_000_000, 4096, 1_000_000),
            (Status::Lost, 99_999_999, 131_072, 99_999_999),
        ];
        for (status, moves, top, score) in cases {
            let text = summary(status, moves, top, score);
            assert!(
                text.chars().count() <= BOARD_OUTER_W as usize,
                "`{text}` is {} cells but the footer is {BOARD_OUTER_W}",
                text.chars().count()
            );
        }
    }

    #[test]
    fn a_pending_count_is_shown_like_vims_showcmd() {
        let mut app = App::with_seed(1);
        let without = render(&app, MIN_W, MIN_H);
        assert!(!without.contains("  12"), "nothing should be pending yet");

        press(&mut app, KeyCode::Char('1'));
        press(&mut app, KeyCode::Char('2'));
        assert_eq!(app.pending_count(), Some(12));

        let screen = render(&app, MIN_W, MIN_H);
        let bottom = screen.lines().last().unwrap_or_default();
        assert!(
            bottom.trim_end().ends_with("12"),
            "the count should sit at the bottom right:\n{screen}"
        );

        // Typing a motion uses it up, so the display goes away again.
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.pending_count(), None);
        let after = render(&app, MIN_W, MIN_H);
        assert!(!after.lines().last().unwrap_or_default().contains("12"));
    }

    #[test]
    fn the_footer_still_fits_with_a_count_showing() {
        // The count steals width from the hints, so check the hint row still
        // fits at the narrowest supported terminal.
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('9'));
        let screen = render(&app, MIN_W, MIN_H);

        let hint_width = MIN_W - COUNT_W;
        assert!(
            screen.contains(hints(hint_width)),
            "hints were clipped with a count showing:\n{screen}"
        );
    }

    #[test]
    fn a_wide_terminal_gets_the_longer_hint_line() {
        // The short hints would otherwise be the only ones ever shown, since
        // they are what fits under a board-width footer.
        let screen = render(&App::with_seed(1), 100, 24);
        // Spelled out, not abbreviated: `C-r` reads like just another letter.
        assert!(
            screen.contains("Ctrl-r redo"),
            "long hints missing:\n{screen}"
        );
        assert!(screen.contains(hints(100 - COUNT_W)), "{screen}");

        // ...but the narrow one still falls back to the short line.
        let narrow = render(&App::with_seed(1), MIN_W, MIN_H);
        assert!(narrow.contains(hints(MIN_W - COUNT_W)), "{narrow}");
    }

    #[test]
    fn control_keys_are_spelled_out_everywhere() {
        // `C-r` is ambiguous next to keys like `hjkl`; every screen must use
        // the same, explicit spelling.
        let mut app = App::with_seed(1);
        let playing = render(&app, 100, 24);
        assert!(playing.contains("Ctrl-r"), "{playing}");
        assert!(!playing.contains("C-r "), "{playing}");

        press(&mut app, KeyCode::Char('K'));
        let help = render(&app, 100, 24);
        assert!(help.contains("Ctrl-r"), "{help}");
        assert!(!help.contains("C-r "), "{help}");
    }

    #[test]
    fn the_hints_always_fit_the_space_available() {
        // Whatever width we end up with, the footer must not be clipped into
        // the border of the board.
        for width in [MIN_W, 40, 60, 80, 120] {
            let text = hints(width);
            assert!(
                text.chars().count() as u16 <= width,
                "hint of {} chars does not fit {width} columns",
                text.chars().count()
            );
        }
    }

    #[test]
    fn rendering_survives_a_long_game() {
        let mut app = App::with_seed(9);
        let keys = [KeyCode::Left, KeyCode::Down, KeyCode::Right, KeyCode::Up];
        for step in 0..300 {
            press(&mut app, keys[step % keys.len()]);
            let screen = render(&app, 80, 24);
            assert!(!screen.contains("too small"));
        }
    }
}
