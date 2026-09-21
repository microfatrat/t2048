//! Terminal-agnostic 2048 rules.
//!
//! Nothing in this module knows about ratatui or the terminal, and every entry
//! point is deterministic given a seed, which keeps the rules fully testable.

use rand::rngs::StdRng;
use rand::{RngExt as _, SeedableRng as _};

/// Side length of the (square) board.
pub const SIZE: usize = 4;
/// The tile value that wins the game.
pub const WIN_VALUE: u32 = 2048;
/// A single row of the board.
pub type Row = [u32; SIZE];
/// The board, indexed as `grid[row][col]`.
pub type Grid = [Row; SIZE];

/// The direction a move pushes every tile towards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// All four directions, in a stable order.
    pub const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];
}

/// Where the current game stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Normal play.
    Playing,
    /// [`WIN_VALUE`] was reached and the player has not chosen to continue yet.
    Won,
    /// No move can change the board any more.
    Lost,
}

/// What a single move did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveOutcome {
    /// Whether the board actually changed.
    pub changed: bool,
    /// Points earned by merges during this move.
    pub gained: u64,
}

/// Maps a position within a slide line back onto the board.
///
/// `i` selects the line (row for horizontal moves, column for vertical ones)
/// and `j` walks along it starting from the edge tiles move towards, so that
/// index `0` is always the tile that has nowhere further to go.
const fn cell_at(dir: Direction, i: usize, j: usize) -> (usize, usize) {
    match dir {
        Direction::Left => (i, j),
        Direction::Right => (i, SIZE - 1 - j),
        Direction::Up => (j, i),
        Direction::Down => (SIZE - 1 - j, i),
    }
}

/// Slides and merges a single line towards index `0`.
///
/// Tiles are compacted first, then each adjacent equal pair merges once, so
/// `[2, 2, 2, 2]` becomes `[4, 4, 0, 0]` and `[4, 4, 8, 0]` becomes `[8, 8, 0, 0]`.
/// Returns the resulting line and the score earned.
pub fn slide_line(line: &Row) -> (Row, u64) {
    let mut compacted = [0u32; SIZE];
    let mut len = 0;
    for &value in line {
        if value != 0 {
            compacted[len] = value;
            len += 1;
        }
    }

    let mut out = [0u32; SIZE];
    let mut score = 0;
    let mut read = 0;
    let mut write = 0;
    while read < len {
        if read + 1 < len && compacted[read] == compacted[read + 1] {
            let merged = compacted[read] * 2;
            out[write] = merged;
            score += u64::from(merged);
            read += 2;
        } else {
            out[write] = compacted[read];
            read += 1;
        }
        write += 1;
    }

    (out, score)
}

/// Applies a move to `grid` in place, without spawning a new tile.
pub fn apply_move(grid: &mut Grid, dir: Direction) -> MoveOutcome {
    let before = *grid;
    let mut gained = 0;

    for i in 0..SIZE {
        let mut line = [0u32; SIZE];
        for (j, slot) in line.iter_mut().enumerate() {
            let (row, col) = cell_at(dir, i, j);
            *slot = grid[row][col];
        }

        let (merged, score) = slide_line(&line);
        gained += score;

        for (j, &value) in merged.iter().enumerate() {
            let (row, col) = cell_at(dir, i, j);
            grid[row][col] = value;
        }
    }

    MoveOutcome {
        changed: *grid != before,
        gained,
    }
}

/// Returns `true` if no move can change the board any more.
pub fn is_stuck(grid: &Grid) -> bool {
    for row in 0..SIZE {
        for col in 0..SIZE {
            let value = grid[row][col];
            if value == 0 {
                return false;
            }
            if col + 1 < SIZE && grid[row][col + 1] == value {
                return false;
            }
            if row + 1 < SIZE && grid[row + 1][col] == value {
                return false;
            }
        }
    }
    true
}

/// A full game: board, score and the bookkeeping the UI needs.
pub struct Game {
    grid: Grid,
    score: u64,
    best: u64,
    status: Status,
    /// Set once the player dismisses the win screen, so the win is not re-shown.
    win_acknowledged: bool,
    /// States to go back to, newest last: the undo stack.
    history: Vec<Snapshot>,
    /// States undone but not yet redone, newest last. Cleared by any new move,
    /// exactly like vim discards the redo branch once you edit again.
    future: Vec<Snapshot>,
    rng: StdRng,
    spawned: Option<(usize, usize)>,
    moves: u64,
}

/// A complete, restorable position.
#[derive(Debug, Clone, Copy)]
struct Snapshot {
    grid: Grid,
    score: u64,
    moves: u64,
}

impl Game {
    /// Starts a new game with a random seed.
    pub fn new(best: u64) -> Self {
        Self::start(rand::make_rng(), best)
    }

    /// Starts a new game with a fixed seed, for tests and reproducible runs.
    pub fn with_seed(seed: u64, best: u64) -> Self {
        Self::start(StdRng::seed_from_u64(seed), best)
    }

    /// Builds a game around an explicit board, without spawning anything.
    pub fn from_grid(grid: Grid, best: u64) -> Self {
        let mut game = Self::start(StdRng::seed_from_u64(0), best);
        game.grid = grid;
        game.history.clear();
        game.future.clear();
        game.spawned = None;
        game.refresh_status();
        game
    }

    fn start(rng: StdRng, best: u64) -> Self {
        let mut game = Self {
            grid: [[0; SIZE]; SIZE],
            score: 0,
            best,
            status: Status::Playing,
            win_acknowledged: false,
            history: Vec::new(),
            future: Vec::new(),
            rng,
            spawned: None,
            moves: 0,
        };
        game.reset();
        game
    }

    /// Clears the board and starts a fresh game, keeping `best` and the RNG.
    fn reset(&mut self) {
        self.grid = [[0; SIZE]; SIZE];
        self.score = 0;
        self.status = Status::Playing;
        self.win_acknowledged = false;
        self.history.clear();
        self.future.clear();
        self.spawned = None;
        self.moves = 0;
        self.spawn_tile();
        self.spawn_tile();
    }

    /// Resets the board, keeping the best score.
    pub fn restart(&mut self) {
        self.reset();
    }

    /// Pushes every tile in `dir`. Returns `true` if the board changed.
    ///
    /// Ignored unless the game is in [`Status::Playing`].
    pub fn slide(&mut self, dir: Direction) -> bool {
        if self.status != Status::Playing {
            return false;
        }

        let before = self.snapshot();
        let outcome = apply_move(&mut self.grid, dir);
        if !outcome.changed {
            // A move that changes nothing must not spawn a tile or cost an undo.
            return false;
        }

        self.history.push(before);
        // A new move abandons the redo branch, as it does in vim.
        self.future.clear();
        self.score += outcome.gained;
        self.best = self.best.max(self.score);
        self.moves += 1;
        self.spawn_tile();
        self.refresh_status();
        true
    }

    /// Undoes the last move. Returns `true` if there was one.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        self.future.push(self.snapshot());
        self.restore(previous);
        true
    }

    /// Redoes an undone move. Returns `true` if there was one.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };
        self.history.push(self.snapshot());
        self.restore(next);
        true
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            grid: self.grid,
            score: self.score,
            moves: self.moves,
        }
    }

    /// Puts the board back exactly as the snapshot recorded it.
    fn restore(&mut self, snapshot: Snapshot) {
        self.grid = snapshot.grid;
        self.score = snapshot.score;
        self.moves = snapshot.moves;
        self.spawned = None;
        // A redo can land on a finished position, so recompute rather than
        // assuming play can continue.
        self.refresh_status();
    }

    /// Keeps playing after reaching [`WIN_VALUE`].
    pub fn continue_after_win(&mut self) {
        if self.status == Status::Won {
            self.win_acknowledged = true;
            // A board can be winning and stuck at the same time when it was
            // built through [`Game::from_grid`], so re-check instead of
            // assuming play can continue.
            self.refresh_status();
        }
    }

    /// Returns `true` if the last move spawned a tile and where.
    pub fn spawned(&self) -> Option<(usize, usize)> {
        self.spawned
    }

    /// The current board.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// The value at `(row, col)`.
    pub fn tile(&self, row: usize, col: usize) -> u32 {
        self.grid[row][col]
    }

    /// Points earned in this game.
    pub fn score(&self) -> u64 {
        self.score
    }

    /// Highest score seen so far.
    pub fn best(&self) -> u64 {
        self.best
    }

    /// Number of moves that changed the board.
    pub fn moves(&self) -> u64 {
        self.moves
    }

    /// Current game status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Whether an undo is available.
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Whether a redo is available.
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// The highest tile currently on the board.
    pub fn max_tile(&self) -> u32 {
        self.grid.iter().flatten().copied().max().unwrap_or(0)
    }

    fn spawn_tile(&mut self) -> Option<(usize, usize)> {
        // The board only has 16 cells, so a stack array avoids a heap
        // allocation on every productive move.
        let mut empty = [(0usize, 0usize); SIZE * SIZE];
        let mut len = 0;
        for row in 0..SIZE {
            for col in 0..SIZE {
                if self.grid[row][col] == 0 {
                    empty[len] = (row, col);
                    len += 1;
                }
            }
        }

        if len == 0 {
            self.spawned = None;
            return None;
        }

        let (row, col) = empty[self.rng.random_range(0..len)];
        // The classic distribution: 90% twos, 10% fours.
        self.grid[row][col] = if self.rng.random_range(0..10) == 0 {
            4
        } else {
            2
        };
        self.spawned = Some((row, col));
        Some((row, col))
    }

    fn refresh_status(&mut self) {
        if !self.win_acknowledged && self.max_tile() >= WIN_VALUE {
            self.status = Status::Won;
        } else if is_stuck(&self.grid) {
            self.status = Status::Lost;
        } else {
            self.status = Status::Playing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slides_gaps_away() {
        assert_eq!(slide_line(&[0, 0, 2, 4]), ([2, 4, 0, 0], 0));
        assert_eq!(slide_line(&[0, 0, 0, 2]), ([2, 0, 0, 0], 0));
        assert_eq!(slide_line(&[0; SIZE]), ([0; SIZE], 0));
    }

    #[test]
    fn merges_equal_neighbours_once() {
        // A pair merges and scores the merged value.
        assert_eq!(slide_line(&[2, 2, 0, 0]), ([4, 0, 0, 0], 4));
        // Four equal tiles become two merged tiles, not one.
        assert_eq!(slide_line(&[2, 2, 2, 2]), ([4, 4, 0, 0], 8));
        // A freshly merged tile does not merge again in the same move, so the
        // new 4 stays next to the existing 4.
        assert_eq!(slide_line(&[2, 2, 4, 0]), ([4, 4, 0, 0], 4));
        assert_eq!(slide_line(&[4, 4, 8, 0]), ([8, 8, 0, 0], 8));
        // Only adjacent pairs merge.
        assert_eq!(slide_line(&[2, 0, 2, 4]), ([4, 4, 0, 0], 4));
    }

    #[test]
    fn moves_are_directional() {
        let grid = [[2, 2, 4, 4], [0; SIZE], [0; SIZE], [0; SIZE]];

        let mut left = grid;
        assert!(apply_move(&mut left, Direction::Left).changed);
        assert_eq!(left[0], [4, 8, 0, 0]);

        let mut right = grid;
        assert!(apply_move(&mut right, Direction::Right).changed);
        assert_eq!(right[0], [0, 0, 4, 8]);

        let mut up = [[2, 0, 0, 0], [2, 0, 0, 0], [4, 0, 0, 0], [4, 0, 0, 0]];
        assert!(apply_move(&mut up, Direction::Up).changed);
        assert_eq!(up, [[4, 0, 0, 0], [8, 0, 0, 0], [0; SIZE], [0; SIZE]]);

        let mut down = [[2, 0, 0, 0], [2, 0, 0, 0], [4, 0, 0, 0], [4, 0, 0, 0]];
        assert!(apply_move(&mut down, Direction::Down).changed);
        assert_eq!(down, [[0; SIZE], [0; SIZE], [4, 0, 0, 0], [8, 0, 0, 0]]);
    }

    #[test]
    fn a_move_conserves_the_total_of_all_tiles() {
        // Merging 2+2 into 4 keeps the sum, so sliding can never create or
        // destroy value; only the spawn after a real move does that.
        assert_eq!(slide_line(&[2, 2, 2, 2]).0.iter().sum::<u32>(), 8);

        let grid = [[2, 2, 4, 4], [8, 0, 8, 0], [0, 4, 0, 4], [2, 0, 0, 0]];
        for dir in Direction::ALL {
            let before: u32 = grid.iter().flatten().sum();
            let mut next = grid;
            apply_move(&mut next, dir);
            let after: u32 = next.iter().flatten().sum();
            assert_eq!(before, after, "sum changed moving {dir:?}");
        }
    }

    #[test]
    fn a_move_never_increases_the_number_of_tiles() {
        let grid = [[2, 2, 4, 4], [8, 0, 8, 0], [0, 4, 0, 4], [2, 0, 0, 0]];
        for dir in Direction::ALL {
            let before = grid.iter().flatten().filter(|&&v| v != 0).count();
            let mut next = grid;
            apply_move(&mut next, dir);
            let after = next.iter().flatten().filter(|&&v| v != 0).count();
            assert!(after <= before, "tile count grew moving {dir:?}");
        }
    }

    #[test]
    fn a_move_that_changes_nothing_is_rejected() {
        let mut grid = [
            [2, 4, 8, 16],
            [4, 8, 16, 32],
            [8, 16, 32, 64],
            [16, 32, 64, 128],
        ];
        let before = grid;
        let outcome = apply_move(&mut grid, Direction::Left);
        assert!(!outcome.changed);
        assert_eq!(grid, before);
        assert_eq!(outcome.gained, 0);
    }

    #[test]
    fn detects_a_stuck_board() {
        let stuck = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        assert!(is_stuck(&stuck));

        // One empty cell is enough to keep playing.
        let mut open = stuck;
        open[3][3] = 0;
        assert!(!is_stuck(&open));

        // So is one mergeable pair.
        let mut mergeable = stuck;
        mergeable[0][1] = mergeable[0][0];
        assert!(!is_stuck(&mergeable));
    }

    #[test]
    fn from_grid_recognises_a_board_that_already_won() {
        let game = Game::from_grid([[2048, 0, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]], 0);
        assert_eq!(game.status(), Status::Won);
    }

    #[test]
    fn a_stuck_winning_board_continues_into_a_loss() {
        // Fully packed with no equal neighbours, but already containing 2048:
        // `from_grid` accepts such a board, so continuing must notice the loss.
        let grid = [[2048, 2, 4, 8], [4, 8, 2, 4], [2, 4, 8, 2], [8, 2, 4, 8]];
        let mut game = Game::from_grid(grid, 0);
        assert_eq!(game.status(), Status::Won);

        game.continue_after_win();
        assert_eq!(game.status(), Status::Lost);
    }

    #[test]
    fn a_new_game_starts_with_two_tiles_and_no_score() {
        let game = Game::with_seed(42, 0);
        let filled = game.grid().iter().flatten().filter(|&&v| v != 0).count();
        assert_eq!(filled, 2);
        assert_eq!(game.score(), 0);
        assert_eq!(game.status(), Status::Playing);
        assert!(!game.can_undo());

        // Starting tiles are always 2 or 4.
        assert!(
            game.grid()
                .iter()
                .flatten()
                .all(|&v| v == 0 || v == 2 || v == 4)
        );
    }

    #[test]
    fn a_productive_move_spawns_exactly_one_tile() {
        let mut game = Game::with_seed(7, 0);
        let before = game.grid().iter().flatten().filter(|&&v| v != 0).count();
        assert!(game.slide(Direction::Left));
        let after = game.grid().iter().flatten().filter(|&&v| v != 0).count();
        // The move moved tiles around; exactly one new tile appeared.
        assert_eq!(after, before + 1);
        assert!(game.spawned().is_some());
        assert_eq!(game.moves(), 1);
    }

    #[test]
    fn redo_puts_back_exactly_what_undo_removed() {
        let mut game = Game::with_seed(11, 0);
        assert!(game.slide(Direction::Down));
        let after_move = (*game.grid(), game.score(), game.moves());
        assert!(!game.can_redo(), "nothing has been undone yet");

        assert!(game.undo());
        assert!(game.can_redo());
        assert!(game.redo());

        assert_eq!((*game.grid(), game.score(), game.moves()), after_move);
        assert!(!game.can_redo(), "the redo stack is used up");
    }

    #[test]
    fn a_new_move_discards_the_redo_branch() {
        // The same rule vim uses: once you edit again, the old redo chain is
        // gone for good.
        let mut game = Game::with_seed(11, 0);
        assert!(game.slide(Direction::Down));
        assert!(game.undo());
        assert!(game.can_redo());

        assert!(game.slide(Direction::Left));
        assert!(!game.can_redo());
        assert!(!game.redo());
    }

    #[test]
    fn undo_and_redo_can_be_walked_back_and_forth() {
        let mut game = Game::with_seed(3, 0);
        let mut states = vec![(*game.grid(), game.score())];
        for dir in [Direction::Left, Direction::Down, Direction::Right] {
            assert!(game.slide(dir));
            states.push((*game.grid(), game.score()));
        }

        // Walk all the way back...
        for expected in states.iter().rev().skip(1) {
            assert!(game.undo());
            assert_eq!((*game.grid(), game.score()), *expected);
        }
        assert!(!game.can_undo());

        // ...and all the way forward again.
        for expected in states.iter().skip(1) {
            assert!(game.redo());
            assert_eq!((*game.grid(), game.score()), *expected);
        }
        assert!(!game.can_redo());
    }

    #[test]
    fn redoing_into_a_finished_position_reports_it_as_lost() {
        // The move that ends the game must still be "lost" after undo + redo.
        let mut game = Game::from_grid([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 0]], 0);
        for step in 0..200 {
            if game.status() == Status::Lost {
                break;
            }
            game.slide(Direction::ALL[step % 4]);
        }
        assert_eq!(game.status(), Status::Lost);

        assert!(game.undo());
        assert_eq!(game.status(), Status::Playing);
        assert!(game.redo());
        assert_eq!(
            game.status(),
            Status::Lost,
            "redo must restore the finished state, not silently resume"
        );
    }

    #[test]
    fn undo_restores_board_and_score() {
        let mut game = Game::with_seed(11, 0);
        let grid = *game.grid();
        let score = game.score();

        assert!(game.slide(Direction::Down));
        assert!(game.can_undo());
        assert!(game.undo());

        assert_eq!(*game.grid(), grid);
        assert_eq!(game.score(), score);
        assert_eq!(game.moves(), 0);
        assert!(!game.can_undo());
    }

    #[test]
    fn score_is_the_sum_of_merged_tiles() {
        let mut game = Game::from_grid([[2, 2, 4, 4], [0; SIZE], [0; SIZE], [0; SIZE]], 0);
        assert!(game.slide(Direction::Left));
        assert_eq!(game.score(), 12); // 4 + 8
        assert_eq!(game.best(), 12);
    }

    #[test]
    fn best_score_is_never_lowered() {
        let mut game = Game::from_grid([[2, 2, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]], 1000);
        assert!(game.slide(Direction::Left));
        assert_eq!(game.score(), 4);
        assert_eq!(game.best(), 1000);
    }

    #[test]
    fn reaching_the_win_value_wins_once() {
        let mut game = Game::from_grid([[1024, 1024, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]], 0);
        assert!(game.slide(Direction::Left));
        assert_eq!(game.status(), Status::Won);
        assert_eq!(game.max_tile(), WIN_VALUE);

        // While the win screen is up, moves are ignored.
        assert!(!game.slide(Direction::Down));

        game.continue_after_win();
        assert_eq!(game.status(), Status::Playing);
        // Continuing does not re-trigger the win screen on the next move.
        assert!(game.slide(Direction::Down));
        assert_eq!(game.status(), Status::Playing);
    }

    #[test]
    fn a_full_board_with_no_merges_is_lost() {
        let mut game = Game::from_grid([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]], 0);
        assert_eq!(game.status(), Status::Lost);
        assert!(!game.slide(Direction::Left));
        assert!(!game.slide(Direction::Up));
    }

    #[test]
    fn status_always_agrees_with_the_board() {
        // Whatever the RNG does, the reported status must match reality.
        let mut game = Game::with_seed(31337, 0);
        for step in 0..2000 {
            if game.status() == Status::Won {
                game.continue_after_win();
            }
            if game.status() == Status::Lost {
                break;
            }
            game.slide(Direction::ALL[step % 4]);
            assert_eq!(
                game.status() == Status::Lost,
                is_stuck(game.grid()),
                "status disagrees with the board at step {step}:\n{:?}",
                game.grid()
            );
        }
        assert_eq!(
            game.status(),
            Status::Lost,
            "a badly played game must eventually end"
        );
    }

    #[test]
    fn restart_clears_progress_but_keeps_the_best() {
        let mut game = Game::from_grid([[2, 2, 0, 0], [0; SIZE], [0; SIZE], [0; SIZE]], 500);
        assert!(game.slide(Direction::Left));
        assert_eq!(game.score(), 4);

        game.restart();
        assert_eq!(game.score(), 0);
        assert_eq!(game.best(), 500);
        assert_eq!(game.moves(), 0);
        assert!(!game.can_undo());
        assert_eq!(game.grid().iter().flatten().filter(|&&v| v != 0).count(), 2);
    }

    #[test]
    fn spawning_only_ever_produces_twos_and_fours() {
        // Play a long random game and check every spawned value.
        let mut game = Game::with_seed(2024, 0);
        for step in 0..400 {
            let dir = Direction::ALL[step % Direction::ALL.len()];
            if game.status() != Status::Playing {
                break;
            }
            game.slide(dir);
            assert!(
                game.grid()
                    .iter()
                    .flatten()
                    .all(|&v| v == 0 || v.is_power_of_two()),
                "every tile must be a power of two, got {:?}",
                game.grid()
            );
        }
    }

    #[test]
    fn score_never_exceeds_a_sane_bound() {
        // Guards against runaway merging loops: a long game stays consistent.
        let mut game = Game::with_seed(99, 0);
        for step in 0..1000 {
            if game.status() != Status::Playing {
                game.restart();
            }
            game.slide(Direction::ALL[step % 4]);
        }
        assert!(game.max_tile() <= 1 << 20);
    }
}
