//! The `t2048` binary: a terminal front end for the library.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

use t2048::app::App;
use t2048::theme::Theme;
use t2048::ui;

/// How often we wake up to advance the spawn highlight.
const TICK: Duration = Duration::from_millis(60);
/// How often we wake up when no animation is running. Input still wakes us
/// immediately, so this only trades idle CPU for a slower animation clock.
const IDLE_TICK: Duration = Duration::from_secs(1);

fn main() -> io::Result<()> {
    // `args_os` rather than `args`: on Windows the arguments arrive as UTF-16
    // and are not guaranteed to be valid Unicode, and `args` would panic.
    let argv = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned());
    let options = match Options::parse(argv) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n");
            eprintln!("{}", Options::usage());
            std::process::exit(2);
        }
    };

    if options.help {
        println!("{}", Options::usage());
        return Ok(());
    }

    // `ratatui::init` installs a panic hook that restores the terminal, so a
    // panic cannot leave it in raw mode; no hook of our own is needed.
    let terminal = ratatui::init();
    let result = run(terminal, &options);
    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal, options: &Options) -> io::Result<()> {
    let theme = if options.mono {
        Theme::mono()
    } else {
        Theme::detect()
    };

    let mut app = match options.seed {
        Some(seed) => App::with_seed(seed),
        None => App::new(),
    };

    while app.running() {
        // Only touch the terminal when something actually changed; otherwise a
        // mostly idle game would still redraw several times a second.
        if app.needs_redraw() {
            terminal.draw(|frame| ui::draw(frame, &app, &theme))?;
            app.mark_drawn();
        }

        // The highlight is the only animation, so when it is not running there
        // is nothing to do until input arrives.
        let timeout = if app.flash() > 0 { TICK } else { IDLE_TICK };
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
                    app.on_key(key);
                }
                // A resize invalidates the layout, so redraw at the new size.
                Event::Resize(_, _) => app.request_redraw(),
                _ => {}
            }
        }

        app.on_tick();
    }

    app.save_best();
    Ok(())
}

/// Command line options.
#[derive(Debug, Default, PartialEq, Eq)]
struct Options {
    /// Print usage and exit.
    help: bool,
    /// Use the 256-colour palette.
    mono: bool,
    /// Fix the RNG seed, for reproducible games.
    seed: Option<u64>,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => options.help = true,
                "--mono" | "--no-color" => options.mono = true,
                "--seed" => {
                    let value = args.next().ok_or("--seed needs a number")?;
                    options.seed = Some(
                        value
                            .parse()
                            .map_err(|_| format!("`{value}` is not a valid seed"))?,
                    );
                }
                other => {
                    let value = other
                        .strip_prefix("--seed=")
                        .ok_or_else(|| format!("unknown argument `{other}`"))?;
                    options.seed = Some(
                        value
                            .parse()
                            .map_err(|_| format!("`{value}` is not a valid seed"))?,
                    );
                }
            }
        }

        Ok(options)
    }

    fn usage() -> String {
        format!(
            "\
{name} {version} - a 2048 clone for the terminal

USAGE:
    {name} [OPTIONS]

OPTIONS:
    -h, --help        show this message
        --mono        use the 256-colour palette instead of truecolor
        --seed <N>    start from a fixed seed, for reproducible games

CONTROLS:
    h j k l                move the tiles (arrow keys work too)
    [count] then a motion  repeat it, e.g. 3j
    u / Ctrl-r             undo / redo
    r                      restart the game
    c                      keep playing after reaching 2048
    K, ?                   show the key list
    ZZ / ZQ                save and quit / quit without saving
    q, esc, ctrl-c         quit
",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(args.iter().map(|s| (*s).to_string()))
    }

    #[test]
    fn no_arguments_means_a_normal_game() {
        assert_eq!(parse(&[]).unwrap(), Options::default());
    }

    #[test]
    fn help_is_recognised_in_both_spellings() {
        assert!(parse(&["-h"]).unwrap().help);
        assert!(parse(&["--help"]).unwrap().help);
    }

    #[test]
    fn mono_is_recognised_in_both_spellings() {
        assert!(parse(&["--mono"]).unwrap().mono);
        assert!(parse(&["--no-color"]).unwrap().mono);
    }

    #[test]
    fn a_seed_can_be_given_either_way() {
        assert_eq!(parse(&["--seed", "42"]).unwrap().seed, Some(42));
        assert_eq!(parse(&["--seed=42"]).unwrap().seed, Some(42));
    }

    #[test]
    fn bad_arguments_are_rejected() {
        assert!(parse(&["--nope"]).is_err());
        assert!(parse(&["--seed"]).is_err(), "a missing value must fail");
        assert!(parse(&["--seed", "abc"]).is_err());
        assert!(parse(&["--seed="]).is_err());
    }

    #[test]
    fn the_usage_text_documents_every_key() {
        let usage = Options::usage();
        for key in [
            "h j k l", "count", "undo", "redo", "restart", "quit", "2048", "ZZ", "ZQ",
        ] {
            assert!(usage.contains(key), "usage does not mention `{key}`");
        }
        assert!(
            !usage.contains("wasd"),
            "wasd is no longer a binding:\n{usage}"
        );
    }
}
