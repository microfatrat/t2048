//! Colour palette for the board.
//!
//! Truecolor is used when the terminal is likely to support it. Setting
//! `NO_COLOR`, or passing `--mono`, switches to a 256-colour palette so the
//! board stays readable on limited terminals.

use ratatui::style::Color;

/// The dark text used on the light tiles.
const INK: Color = Color::Rgb(119, 110, 101);
/// The light text used on the saturated tiles.
const PAPER: Color = Color::Rgb(249, 246, 242);

/// Colours for every part of the board.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    colored: bool,
}

impl Theme {
    /// Truecolor unless `NO_COLOR` is set.
    pub fn detect() -> Self {
        Self {
            colored: std::env::var_os("NO_COLOR").is_none(),
        }
    }

    /// The conservative 256-colour palette.
    pub fn mono() -> Self {
        Self { colored: false }
    }

    /// Whether the truecolor palette is in use.
    pub fn is_colored(self) -> bool {
        self.colored
    }

    /// Background and foreground for a tile. `0` is an empty cell.
    pub fn tile(self, value: u32) -> (Color, Color) {
        if !self.colored {
            return indexed_tile(value);
        }
        match value {
            0 => (Color::Rgb(205, 193, 180), Color::Rgb(205, 193, 180)),
            2 => (Color::Rgb(238, 228, 218), INK),
            4 => (Color::Rgb(237, 224, 200), INK),
            8 => (Color::Rgb(242, 177, 121), PAPER),
            16 => (Color::Rgb(245, 149, 99), PAPER),
            32 => (Color::Rgb(246, 124, 95), PAPER),
            64 => (Color::Rgb(246, 94, 59), PAPER),
            128 => (Color::Rgb(237, 207, 114), PAPER),
            256 => (Color::Rgb(237, 204, 97), PAPER),
            512 => (Color::Rgb(237, 200, 80), PAPER),
            1024 => (Color::Rgb(237, 197, 63), PAPER),
            2048 => (Color::Rgb(237, 194, 46), PAPER),
            _ => (Color::Rgb(60, 58, 50), PAPER),
        }
    }

    /// The colour behind the tiles.
    pub fn board(self) -> Color {
        if self.colored {
            Color::Rgb(187, 173, 160)
        } else {
            Color::Indexed(180)
        }
    }

    /// The board outline.
    pub fn border(self) -> Color {
        if self.colored {
            Color::Rgb(160, 146, 132)
        } else {
            Color::Indexed(137)
        }
    }

    /// The background of the score boxes.
    pub fn panel(self) -> Color {
        if self.colored {
            Color::Rgb(60, 58, 50)
        } else {
            Color::Indexed(236)
        }
    }

    /// Ordinary text.
    pub fn text(self) -> Color {
        if self.colored {
            Color::Rgb(238, 228, 218)
        } else {
            Color::Indexed(253)
        }
    }

    /// Secondary text.
    pub fn dim(self) -> Color {
        if self.colored {
            Color::Rgb(146, 136, 126)
        } else {
            Color::Indexed(245)
        }
    }

    /// The highlight colour, used for the title and the best score.
    pub fn accent(self) -> Color {
        if self.colored {
            Color::Rgb(237, 194, 46)
        } else {
            Color::Indexed(178)
        }
    }
}

/// Makes a colour brighter, for the flash on a freshly spawned tile.
pub fn lighten(color: Color, by: u8) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_add(by),
            g.saturating_add(by),
            b.saturating_add(by),
        ),
        Color::Indexed(i) => Color::Indexed(lighten_indexed(i, by)),
        other => other,
    }
}

/// Brightens an xterm-256 index without leaving the indexed palette, so
/// `--mono` mode still shows the spawn highlight.
fn lighten_indexed(index: u8, by: u8) -> u8 {
    match index {
        // The eight dim system colours have conventional bright partners.
        0..=7 if by > 0 => index + 8,
        0..=7 => index,
        // The bright half is already as bright as those colours get.
        8..=15 => index,
        // The 6x6x6 colour cube: decode, step every channel up, re-encode.
        16..=231 => {
            let n = index - 16;
            let step = by / 40;
            let r = (n / 36 + step).min(5);
            let g = ((n % 36) / 6 + step).min(5);
            let b = (n % 6 + step).min(5);
            16 + r * 36 + g * 6 + b
        }
        // The grayscale ramp, which runs from nearly black to white.
        _ => index.saturating_add(by / 12),
    }
}

/// Approximations of the truecolor palette in the xterm-256 cube.
fn indexed_tile(value: u32) -> (Color, Color) {
    let ink = Color::Indexed(95);
    let paper = Color::Indexed(255);
    match value {
        0 => (Color::Indexed(187), Color::Indexed(187)),
        2 => (Color::Indexed(230), ink),
        4 => (Color::Indexed(223), ink),
        8 => (Color::Indexed(216), paper),
        16 => (Color::Indexed(209), paper),
        32 => (Color::Indexed(203), paper),
        64 => (Color::Indexed(196), paper),
        128 => (Color::Indexed(227), ink),
        256 => (Color::Indexed(221), ink),
        512 => (Color::Indexed(220), ink),
        1024 => (Color::Indexed(214), ink),
        2048 => (Color::Indexed(178), ink),
        _ => (Color::Indexed(236), paper),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tile_has_a_distinct_colour() {
        let theme = Theme::detect();
        let values = [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048];
        for pair in values.windows(2) {
            assert_ne!(
                theme.tile(pair[0]).0,
                theme.tile(pair[1]).0,
                "{} and {} look the same",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn oversized_tiles_still_have_a_colour() {
        let theme = Theme::detect();
        assert_ne!(theme.tile(4096).0, theme.tile(0).0);
        assert_ne!(Theme::mono().tile(4096).0, Theme::mono().tile(0).0);
    }

    #[test]
    fn the_fallback_palette_is_indexed() {
        let theme = Theme::mono();
        assert!(!theme.is_colored());
        for value in [0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096] {
            let (bg, fg) = theme.tile(value);
            assert!(matches!(bg, Color::Indexed(_)), "{value} bg is not indexed");
            assert!(matches!(fg, Color::Indexed(_)), "{value} fg is not indexed");
        }
    }

    #[test]
    fn lighten_saturates_instead_of_wrapping() {
        assert_eq!(lighten(Color::Rgb(250, 10, 0), 40), Color::Rgb(255, 50, 40));
        assert_eq!(lighten(Color::Rgb(0, 0, 0), 0), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn lighten_also_works_for_the_indexed_palette() {
        // A dim system colour moves to its bright partner...
        assert_eq!(lighten(Color::Indexed(1), 45), Color::Indexed(9));
        // ...a cube colour steps up within the cube...
        assert_ne!(lighten(Color::Indexed(16), 45), Color::Indexed(16));
        // ...and a grayscale entry moves along the ramp.
        assert_ne!(lighten(Color::Indexed(232), 45), Color::Indexed(232));
        // The bright half has nowhere further to go.
        assert_eq!(lighten(Color::Indexed(15), 45), Color::Indexed(15));
        // A zero amount leaves every palette alone.
        assert_eq!(lighten(Color::Indexed(1), 0), Color::Indexed(1));
        assert_eq!(lighten(Color::Indexed(16), 0), Color::Indexed(16));
        assert_eq!(lighten(Color::Indexed(232), 0), Color::Indexed(232));
    }
}
