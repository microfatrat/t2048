//! Application state and input handling.
//!
//! The key map follows vim wherever vim has an answer: `hjkl` to move,
//! `[count]` before a motion, `u` and `Ctrl-r` to undo and redo, `K` for help,
//! and `ZZ` / `ZQ` to leave. The one action vim has no equivalent for is
//! starting over, which keeps the game's usual `r`.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::{Direction, Game, Status};
use crate::storage;

/// How many ticks a freshly spawned tile stays highlighted.
pub const FLASH_TICKS: u8 = 4;

/// The largest `[count]` that is accepted, so a slipped finger cannot lock the
/// game up replaying thousands of moves.
pub const MAX_COUNT: u32 = 999;

/// Whether leaving should write the best score to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOnExit {
    /// `ZZ`, `q`, `Ctrl-C`: keep the best score.
    Yes,
    /// `ZQ`: leave without touching the file.
    No,
}

/// The mutually exclusive input modes. Keeping them in one enum means an
/// impossible combination like "help open while waiting for `Z`" cannot be
/// represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Ordinary play.
    Normal,
    /// The help overlay is up; the next key dismisses it.
    Help,
    /// A `Z` was typed; the next key decides between `ZZ` and `ZQ`.
    PendingZ,
}

/// Everything the UI renders, plus the rules for reacting to input.
pub struct App {
    game: Game,
    running: bool,
    /// Input mode: normal play, the help overlay, or a half-typed `ZZ` / `ZQ`.
    mode: Mode,
    flash: u8,
    needs_redraw: bool,
    saved_best: u64,
    save_on_exit: SaveOnExit,
    /// Digits typed before a motion, as in vim's `3j`.
    count: Option<u32>,
}

impl App {
    /// Starts a new game with a random seed and the stored best score.
    pub fn new() -> Self {
        let best = storage::load_best();
        Self::from_game(Game::new(best), best)
    }

    /// Starts a new game with a fixed seed, for `--seed` and for tests.
    pub fn with_seed(seed: u64) -> Self {
        let best = storage::load_best();
        Self::from_game(Game::with_seed(seed, best), best)
    }

    /// Builds an app around a specific game. Used by the rendering tests.
    #[cfg(test)]
    pub(crate) fn with_game(game: Game) -> Self {
        Self::from_game(game, 0)
    }

    fn from_game(game: Game, saved_best: u64) -> Self {
        Self {
            game,
            running: true,
            mode: Mode::Normal,
            flash: 0,
            needs_redraw: true,
            saved_best,
            save_on_exit: SaveOnExit::Yes,
            count: None,
        }
    }

    /// The game being played.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// Whether the event loop should keep going.
    pub fn running(&self) -> bool {
        self.running
    }

    /// Whether the help overlay is up.
    pub fn show_help(&self) -> bool {
        self.mode == Mode::Help
    }

    /// How much longer the spawn highlight lasts.
    pub fn flash(&self) -> u8 {
        self.flash
    }

    /// The `[count]` typed so far, if any. Shown like vim's `showcmd`.
    pub fn pending_count(&self) -> Option<u32> {
        self.count
    }

    /// Whether the best score will be written when the game ends.
    pub fn save_on_exit(&self) -> SaveOnExit {
        self.save_on_exit
    }

    /// Whether the terminal needs redrawing.
    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    /// Records that the current state has been drawn.
    pub fn mark_drawn(&mut self) {
        self.needs_redraw = false;
    }

    /// Asks for a redraw without changing any state, e.g. after a resize.
    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Writes the best score, unless the player left with `ZQ`.
    pub fn save_best(&self) {
        if self.save_on_exit == SaveOnExit::Yes && self.game.best() > self.saved_best {
            storage::save_best(self.game.best());
        }
    }

    /// Advances animations. Called after every poll timeout.
    pub fn on_tick(&mut self) {
        if self.flash > 0 {
            self.flash -= 1;
            // The highlight is a boolean on screen, so only the tick that ends
            // it changes what is drawn.
            if self.flash == 0 {
                self.needs_redraw = true;
            }
        }
    }

    /// Handles a key press.
    pub fn on_key(&mut self, key: KeyEvent) {
        // Ctrl-C interrupts, here as everywhere else.
        if is_ctrl(key, 'c') {
            self.quit(SaveOnExit::Yes);
            return;
        }

        // While the help overlay is up, any key dismisses it.
        if self.mode == Mode::Help {
            if key.code == KeyCode::Char('q') {
                self.quit(SaveOnExit::Yes);
            }
            self.mode = Mode::Normal;
            self.needs_redraw = true;
            return;
        }

        // The second key of `ZZ` / `ZQ`. Anything else cancels the sequence and
        // does nothing at all, the way an unknown vim command does.
        if self.mode == Mode::PendingZ {
            self.mode = Mode::Normal;
            self.needs_redraw = true;
            match key.code {
                KeyCode::Char('Z') => self.quit(SaveOnExit::Yes),
                KeyCode::Char('Q') => self.quit(SaveOnExit::No),
                _ => {}
            }
            return;
        }

        // A `[count]`. A leading zero is not a count, so `0` on its own is
        // inert, exactly as in vim. Modified digits (Ctrl-3, Alt-3, ...) are
        // not counts.
        if let KeyCode::Char(digit @ '0'..='9') = key.code {
            if has_non_shift_modifier(key) {
                return;
            }
            if digit != '0' || self.count.is_some() {
                let value = self.count.unwrap_or(0) * 10 + (digit as u32 - '0' as u32);
                self.count = Some(value.min(MAX_COUNT));
                self.needs_redraw = true;
            }
            return;
        }

        // Esc drops a half-typed count before it means anything else.
        if key.code == KeyCode::Esc {
            if self.count.is_some() {
                self.clear_count();
            } else {
                self.quit(SaveOnExit::Yes);
            }
            return;
        }

        // Motions take the count: `5j` moves down five times.
        if let Some(direction) = direction_for(key) {
            let times = self.count.take().unwrap_or(1);
            self.slide(direction, times);
            return;
        }

        // Everything else takes the count too, as vim does: `3u` undoes three
        // times. Keys that have no use for a count simply drop it.
        let counted = self.count.take();
        let times = counted.unwrap_or(1);
        if counted.is_some() {
            // Clearing the corner display is itself a change on screen.
            self.needs_redraw = true;
        }

        match key.code {
            KeyCode::Char('Z') => {
                self.mode = Mode::PendingZ;
                self.needs_redraw = true;
            }
            // Ctrl-r first: redo, the partner of `u`.
            KeyCode::Char('r') if is_ctrl(key, 'r') => self.redo(times),
            KeyCode::Char('u') => self.undo(times),
            KeyCode::Char('r') => self.restart(),
            KeyCode::Char('c') | KeyCode::Enter | KeyCode::Char(' ') => self.continue_after_win(),
            // `K` is vim's "look it up" key; `?` is the usual TUI help key.
            KeyCode::Char('K') | KeyCode::Char('?') | KeyCode::F(1) => self.open_help(),
            KeyCode::Char('q') => self.quit(SaveOnExit::Yes),
            _ => {}
        }
    }

    /// Applies a motion `times` over, the way a vim count repeats one.
    fn slide(&mut self, direction: Direction, times: u32) {
        let mut moved = false;
        for _ in 0..times {
            // A move that changes nothing can never be repeated into a change,
            // and a finished game ignores moves entirely, so stop there.
            if !self.game.slide(direction) {
                break;
            }
            moved = true;
        }
        if moved {
            self.flash = FLASH_TICKS;
            self.needs_redraw = true;
        }
    }

    /// Undoes `times` moves, stopping early once there is nothing left.
    fn undo(&mut self, times: u32) {
        let mut changed = false;
        for _ in 0..times {
            if !self.game.undo() {
                break;
            }
            changed = true;
        }
        if changed {
            self.flash = 0;
            self.needs_redraw = true;
        }
    }

    /// Redoes `times` moves, stopping early once there is nothing left.
    fn redo(&mut self, times: u32) {
        let mut changed = false;
        for _ in 0..times {
            if !self.game.redo() {
                break;
            }
            changed = true;
        }
        if changed {
            self.flash = 0;
            self.needs_redraw = true;
        }
    }

    fn restart(&mut self) {
        self.game.restart();
        self.flash = 0;
        self.count = None;
        self.mode = Mode::Normal;
        self.needs_redraw = true;
    }

    fn continue_after_win(&mut self) {
        if self.game.status() == Status::Won {
            self.game.continue_after_win();
            self.needs_redraw = true;
        }
    }

    fn open_help(&mut self) {
        self.mode = Mode::Help;
        self.needs_redraw = true;
    }

    fn quit(&mut self, save: SaveOnExit) {
        self.save_on_exit = save;
        self.running = false;
    }

    fn clear_count(&mut self) {
        if self.count.take().is_some() {
            self.needs_redraw = true;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a key to the direction it pushes the tiles.
///
/// `hjkl` are the vim motions. The arrow keys are kept as an alias for anyone
/// who would rather not leave the arrow cluster. Modified keys such as `Ctrl-h`
/// and `Alt-h` are not motions.
fn direction_for(key: KeyEvent) -> Option<Direction> {
    if has_non_shift_modifier(key) {
        return None;
    }
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => Some(Direction::Left),
        KeyCode::Char('j') | KeyCode::Down => Some(Direction::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Direction::Up),
        KeyCode::Char('l') | KeyCode::Right => Some(Direction::Right),
        _ => None,
    }
}

/// True when a key carries a modifier other than Shift, such as `Ctrl-h` or
/// `Alt-3`. Shift is allowed so `Shift-Left` keeps working like `Left`.
fn has_non_shift_modifier(key: KeyEvent) -> bool {
    key.modifiers.intersects(
        KeyModifiers::CONTROL
            | KeyModifiers::ALT
            | KeyModifiers::SUPER
            | KeyModifiers::HYPER
            | KeyModifiers::META,
    )
}

/// True for Ctrl-`letter`.
///
/// Crossterm reports control characters as the plain letter plus the CONTROL
/// modifier, so the modifier is what we actually test.
fn is_ctrl(key: KeyEvent, letter: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char(c) if c.eq_ignore_ascii_case(&letter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Grid, SIZE, WIN_VALUE};

    /// A board that always has a legal, scoring move available.
    const MERGEABLE: Grid = [[2, 2, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]];

    /// A row packed against the bottom. Only `k` can change it: it is already
    /// full with no equal neighbours, so neither `h` nor `l` can shift it.
    const BOTTOM_ROW_ONLY: Grid = [[0; SIZE], [0; SIZE], [0; SIZE], [2, 4, 2, 4]];

    fn app_with(grid: Grid) -> App {
        App::from_game(Game::from_grid(grid, 0), 0)
    }

    fn press(app: &mut App, code: KeyCode) {
        app.on_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn plain(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn press_ctrl(app: &mut App, letter: char) {
        app.on_key(KeyEvent::new(KeyCode::Char(letter), KeyModifiers::CONTROL));
    }

    fn type_keys(app: &mut App, text: &str) {
        for c in text.chars() {
            press(app, KeyCode::Char(c));
        }
    }

    // --- motions ------------------------------------------------------

    #[test]
    fn hjkl_are_the_vim_motions() {
        for (key, expected) in [
            ('h', Direction::Left),
            ('j', Direction::Down),
            ('k', Direction::Up),
            ('l', Direction::Right),
        ] {
            assert_eq!(
                direction_for(plain(KeyCode::Char(key))),
                Some(expected),
                "`{key}` should move {expected:?}"
            );
        }
    }

    #[test]
    fn arrow_keys_still_move() {
        for (key, expected) in [
            (KeyCode::Left, Direction::Left),
            (KeyCode::Down, Direction::Down),
            (KeyCode::Up, Direction::Up),
            (KeyCode::Right, Direction::Right),
        ] {
            assert_eq!(direction_for(plain(key)), Some(expected), "{key:?}");
        }
    }

    #[test]
    fn uppercase_hjkl_are_not_motions() {
        // In vim H/J/K/L are separate commands, so they must not move tiles.
        for key in ['H', 'J', 'K', 'L'] {
            assert_eq!(direction_for(plain(KeyCode::Char(key))), None, "`{key}`");
        }

        let mut app = app_with(MERGEABLE);
        for key in ['H', 'J', 'L'] {
            press(&mut app, KeyCode::Char(key));
        }
        assert_eq!(app.game().moves(), 0);
    }

    #[test]
    fn wasd_no_longer_moves() {
        // Dropped in favour of the vim motions.
        for key in ['w', 'a', 's', 'd'] {
            assert_eq!(direction_for(plain(KeyCode::Char(key))), None, "`{key}`");
        }

        let mut app = app_with(MERGEABLE);
        for key in ['w', 'a', 's', 'd'] {
            press(&mut app, KeyCode::Char(key));
        }
        assert_eq!(app.game().moves(), 0);
    }

    #[test]
    fn a_motion_moves_the_tiles_the_way_vim_says() {
        // Only `k` (up) can change this board.
        for (key, should_move) in [('j', false), ('h', false), ('l', false), ('k', true)] {
            let mut app = app_with(BOTTOM_ROW_ONLY);
            press(&mut app, KeyCode::Char(key));
            assert_eq!(
                app.game().moves() > 0,
                should_move,
                "`{key}` behaved unexpectedly"
            );
        }
    }

    #[test]
    fn modified_keys_are_not_motions_or_counts() {
        // A terminal can report Ctrl-h or Alt-h as a plain `h` plus a modifier;
        // those must not move the tiles. The same goes for Ctrl-3 and friends.
        let mut app = app_with(MERGEABLE);
        for (code, modifiers) in [
            (KeyCode::Char('h'), KeyModifiers::CONTROL),
            (KeyCode::Char('j'), KeyModifiers::ALT),
            (KeyCode::Char('3'), KeyModifiers::CONTROL),
        ] {
            app.on_key(KeyEvent::new(code, modifiers));
        }
        assert_eq!(app.game().moves(), 0);
        assert_eq!(app.pending_count(), None);

        // Shift is not a command modifier, so Shift-Left still moves.
        let mut shifted = app_with(MERGEABLE);
        shifted.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        assert_eq!(shifted.game().moves(), 1);
    }

    // --- counts -------------------------------------------------------

    #[test]
    fn a_count_repeats_a_motion_like_vim() {
        // `3j` must do exactly what three separate `j`s do.
        let mut counted = App::with_seed(4);
        let mut pressed = App::with_seed(4);

        press(&mut counted, KeyCode::Char('3'));
        press(&mut counted, KeyCode::Char('j'));
        for _ in 0..3 {
            press(&mut pressed, KeyCode::Char('j'));
        }

        assert!(counted.game().moves() > 1, "the count did nothing");
        assert_eq!(counted.game().moves(), pressed.game().moves());
        assert_eq!(counted.game().grid(), pressed.game().grid());
        assert_eq!(counted.game().score(), pressed.game().score());
    }

    #[test]
    fn a_count_is_consumed_by_the_motion() {
        let mut app = App::with_seed(4);
        press(&mut app, KeyCode::Char('2'));
        assert_eq!(app.pending_count(), Some(2));

        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.pending_count(), None, "the count should be used up");

        let before = app.game().moves();
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.game().moves(), before + 1);
    }

    #[test]
    fn multi_digit_counts_accumulate() {
        let mut app = app_with(MERGEABLE);
        type_keys(&mut app, "12");
        assert_eq!(app.pending_count(), Some(12));
        type_keys(&mut app, "3");
        assert_eq!(app.pending_count(), Some(123));
    }

    #[test]
    fn a_leading_zero_is_not_a_count() {
        // `0` is a motion in vim, never a count; here it simply does nothing.
        let mut app = app_with(MERGEABLE);
        press(&mut app, KeyCode::Char('0'));
        assert_eq!(app.pending_count(), None);
        assert_eq!(app.game().moves(), 0);

        // But it does extend a count already being typed.
        press(&mut app, KeyCode::Char('1'));
        press(&mut app, KeyCode::Char('0'));
        assert_eq!(app.pending_count(), Some(10));
    }

    #[test]
    fn counts_are_capped() {
        let mut app = app_with(MERGEABLE);
        type_keys(&mut app, "999999");
        assert_eq!(app.pending_count(), Some(MAX_COUNT));
    }

    #[test]
    fn a_non_motion_key_consumes_a_pending_count() {
        let mut app = App::with_seed(4);
        press(&mut app, KeyCode::Char('9'));
        assert_eq!(app.pending_count(), Some(9));

        press(&mut app, KeyCode::Char('u'));
        assert_eq!(app.pending_count(), None, "the count should not linger");
    }

    #[test]
    fn a_count_undoes_several_steps_like_vim() {
        // In vim `3u` is three undos, so it must be here too.
        let mut app = App::with_seed(6);
        for _ in 0..4 {
            press(&mut app, KeyCode::Char('j'));
            press(&mut app, KeyCode::Char('l'));
        }
        assert_eq!(app.game().moves(), 8);

        press(&mut app, KeyCode::Char('3'));
        press(&mut app, KeyCode::Char('u'));
        assert_eq!(app.game().moves(), 5, "`3u` should undo three times");

        // ...and `2<C-r>` puts two of them back.
        press(&mut app, KeyCode::Char('2'));
        press_ctrl(&mut app, 'r');
        assert_eq!(app.game().moves(), 7, "`2<C-r>` should redo twice");
    }

    #[test]
    fn a_count_undo_stops_at_the_start_of_the_game() {
        let mut app = App::with_seed(6);
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('l'));
        assert_eq!(app.game().moves(), 2);

        press(&mut app, KeyCode::Char('9'));
        press(&mut app, KeyCode::Char('9'));
        press(&mut app, KeyCode::Char('u'));
        assert_eq!(app.game().moves(), 0, "it should undo what exists and stop");
        assert!(!app.game().can_undo());
    }

    #[test]
    fn a_count_that_changes_nothing_still_clears_the_corner_display() {
        // Nothing to undo, but the count must not stay on screen.
        let mut app = App::with_seed(4);
        app.mark_drawn();
        press(&mut app, KeyCode::Char('5'));
        assert!(app.needs_redraw());

        app.mark_drawn();
        press(&mut app, KeyCode::Char('u'));
        assert_eq!(app.pending_count(), None);
        assert!(
            app.needs_redraw(),
            "the leftover count would be drawn forever"
        );
    }

    // --- undo / redo --------------------------------------------------

    #[test]
    fn u_undoes_and_ctrl_r_redoes() {
        let mut app = App::with_seed(6);
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('l'));
        let after = (*app.game().grid(), app.game().score());
        assert!(app.game().can_undo());
        assert!(!app.game().can_redo());

        press(&mut app, KeyCode::Char('u'));
        assert!(app.game().can_redo());
        assert_ne!((*app.game().grid(), app.game().score()), after);

        press_ctrl(&mut app, 'r');
        assert_eq!((*app.game().grid(), app.game().score()), after);
        assert!(!app.game().can_redo());
    }

    #[test]
    fn a_move_after_undo_drops_the_redo_branch() {
        let mut app = App::with_seed(6);
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('u'));
        assert!(app.game().can_redo());

        press(&mut app, KeyCode::Char('l'));
        assert!(!app.game().can_redo());
        press_ctrl(&mut app, 'r');
        assert!(!app.game().can_redo(), "redo should have been discarded");
    }

    #[test]
    fn ctrl_r_is_redo_but_plain_r_restarts() {
        let mut app = app_with(MERGEABLE);
        press(&mut app, KeyCode::Char('h'));
        press(&mut app, KeyCode::Char('u'));
        assert!(app.game().can_redo());

        press_ctrl(&mut app, 'r');
        assert!(!app.game().can_redo(), "Ctrl-r should have redone the move");

        press(&mut app, KeyCode::Char('r'));
        assert_eq!(app.game().moves(), 0, "plain r should restart");
    }

    #[test]
    fn ctrl_r_never_restarts_the_game() {
        // Not even when there is nothing to redo.
        let mut app = App::with_seed(6);
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('l'));
        let moves = app.game().moves();

        press_ctrl(&mut app, 'r');
        assert_eq!(app.game().moves(), moves, "Ctrl-r must not wipe the board");
    }

    // --- leaving ------------------------------------------------------

    #[test]
    fn q_quits_and_saves() {
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('q'));
        assert!(!app.running());
        assert_eq!(app.save_on_exit(), SaveOnExit::Yes);
    }

    #[test]
    fn zz_quits_and_saves() {
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('Z'));
        assert!(app.running(), "one Z is not a command on its own");
        press(&mut app, KeyCode::Char('Z'));
        assert!(!app.running());
        assert_eq!(app.save_on_exit(), SaveOnExit::Yes);
    }

    #[test]
    fn zq_quits_without_saving() {
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('Z'));
        press(&mut app, KeyCode::Char('Q'));
        assert!(!app.running());
        assert_eq!(app.save_on_exit(), SaveOnExit::No);
    }

    #[test]
    fn z_followed_by_anything_else_does_nothing() {
        // An unknown vim command is ignored rather than acted on.
        for key in ['j', 'u', 'r', 'x'] {
            let mut app = App::with_seed(1);
            press(&mut app, KeyCode::Char('Z'));
            press(&mut app, KeyCode::Char(key));
            assert!(app.running(), "Z{key} should not quit");
            assert_eq!(app.game().moves(), 0, "Z{key} should not move");
        }
    }

    #[test]
    fn escape_cancels_a_pending_count_before_it_quits() {
        let mut app = App::with_seed(4);
        press(&mut app, KeyCode::Char('5'));
        press(&mut app, KeyCode::Esc);
        assert!(app.running(), "Esc should only have dropped the count");
        assert_eq!(app.pending_count(), None);

        // With nothing pending it quits, as before.
        press(&mut app, KeyCode::Esc);
        assert!(!app.running());
    }

    #[test]
    fn ctrl_c_quits_even_with_help_open() {
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('?'));
        assert!(app.show_help());
        press_ctrl(&mut app, 'c');
        assert!(!app.running());
    }

    // --- help ---------------------------------------------------------

    #[test]
    fn help_opens_with_question_mark_or_shift_k() {
        for key in ['?', 'K'] {
            let mut app = App::with_seed(1);
            press(&mut app, KeyCode::Char(key));
            assert!(app.show_help(), "`{key}` should open the help");
        }

        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::F(1));
        assert!(app.show_help());
    }

    #[test]
    fn help_is_dismissed_by_the_next_key_without_acting_on_it() {
        let mut app = App::with_seed(1);
        press(&mut app, KeyCode::Char('K'));
        assert!(app.show_help());

        let before = *app.game().grid();
        press(&mut app, KeyCode::Char('j'));
        assert!(!app.show_help());
        assert_eq!(
            *app.game().grid(),
            before,
            "the dismissing key must not move"
        );
    }

    // --- the rest of the map ------------------------------------------

    #[test]
    fn moving_marks_the_screen_dirty() {
        let mut app = app_with(MERGEABLE);
        app.mark_drawn();
        assert!(!app.needs_redraw());

        press(&mut app, KeyCode::Char('h'));
        assert!(app.needs_redraw());
    }

    #[test]
    fn a_move_that_changes_nothing_does_not_dirty_the_screen() {
        // Already packed against the left wall with no merges available.
        let mut app = app_with([[2, 4, 8, 16], [0; SIZE], [0; SIZE], [0; SIZE]]);
        app.mark_drawn();
        press(&mut app, KeyCode::Char('h'));
        assert!(!app.needs_redraw());
        assert_eq!(app.game().moves(), 0);
    }

    #[test]
    fn restart_clears_the_score() {
        let mut app = app_with(MERGEABLE);
        press(&mut app, KeyCode::Char('h'));
        assert_eq!(app.game().score(), 4);

        press(&mut app, KeyCode::Char('r'));
        assert_eq!(app.game().score(), 0);
        assert_eq!(app.game().moves(), 0);
        assert!(!app.game().can_undo());
        assert!(!app.game().can_redo());
    }

    #[test]
    fn a_count_does_not_survive_a_restart() {
        let mut app = app_with(MERGEABLE);
        press(&mut app, KeyCode::Char('7'));
        press(&mut app, KeyCode::Char('r'));
        assert_eq!(app.pending_count(), None);
    }

    #[test]
    fn the_spawn_highlight_fades() {
        let mut app = app_with(MERGEABLE);
        press(&mut app, KeyCode::Char('h'));
        assert_eq!(app.flash(), FLASH_TICKS);

        for _ in 0..FLASH_TICKS {
            app.on_tick();
        }
        assert_eq!(app.flash(), 0);
    }

    #[test]
    fn the_spawn_highlight_only_redraws_when_it_ends() {
        let mut app = app_with(MERGEABLE);
        press(&mut app, KeyCode::Char('h'));
        app.mark_drawn();

        // The highlight is drawn as a boolean, so only the tick that clears it
        // changes the frame.
        for _ in 0..FLASH_TICKS - 1 {
            app.on_tick();
            assert!(
                !app.needs_redraw(),
                "an intermediate tick asked for a redraw"
            );
        }

        app.on_tick();
        assert!(
            app.needs_redraw(),
            "the tick that clears the highlight must redraw"
        );
    }

    #[test]
    fn a_win_can_be_continued() {
        let mut app = app_with([[1024, 1024, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]]);
        press(&mut app, KeyCode::Char('h'));
        assert_eq!(app.game().status(), Status::Won);
        assert!(app.game().max_tile() >= WIN_VALUE);

        // Moves are ignored while the win screen is up.
        let before = *app.game().grid();
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(*app.game().grid(), before);

        press(&mut app, KeyCode::Char('c'));
        assert_eq!(app.game().status(), Status::Playing);

        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.game().status(), Status::Playing);
    }

    #[test]
    fn keys_are_inert_once_the_game_is_lost() {
        let mut app = app_with([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        assert_eq!(app.game().status(), Status::Lost);

        let before = *app.game().grid();
        for key in ['h', 'j', 'k', 'l'] {
            press(&mut app, KeyCode::Char(key));
        }
        assert_eq!(*app.game().grid(), before);

        press(&mut app, KeyCode::Char('r'));
        assert_eq!(app.game().status(), Status::Playing);
        assert_eq!(app.game().moves(), 0);
    }

    #[test]
    fn long_random_play_stays_consistent() {
        let mut app = App::with_seed(20240607);
        let keys = [
            KeyCode::Char('h'),
            KeyCode::Char('j'),
            KeyCode::Char('l'),
            KeyCode::Char('k'),
            KeyCode::Char('u'),
            KeyCode::Char('3'),
        ];
        for step in 0..3000 {
            press(&mut app, keys[step % keys.len()]);
            assert!(app.game().score() <= app.game().best());
            assert_eq!(
                app.game().status() == Status::Lost,
                crate::game::is_stuck(app.game().grid()),
                "status disagrees with the board at step {step}"
            );
        }
    }
}
