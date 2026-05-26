use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum};

use kebabify::{format_action, process_paths, summarize, Config, ConflictMode, NamingStyle, UnicodeMode};

#[derive(Parser, Debug)]
#[command(name = "kebabify")]
#[command(about = "CLI to rename files to kebab-case or snake_case")]
#[command(long_about = "Renames files and directories to kebab-case or snake_case, with dry-run, recursion, and conflict handling.")]
struct Cli {
    #[arg(value_name = "PATH", required = true, help = "Files or directories to process")]
    paths: Vec<PathBuf>,

    #[arg(short = 'n', long = "dry-run", help = "Show planned renames without applying changes")]
    dry_run: bool,

    #[arg(short = 'r', long = "recursive", help = "Process directories recursively")]
    recursive: bool,

    #[arg(short = 'd', long = "dirs", help = "Also rename nested directories")]
    dirs: bool,

    #[arg(long = "rename-root", help = "Rename the input directory itself")]
    rename_root: bool,

    #[arg(short = 's', long = "style", value_enum, default_value_t = StyleArg::Kebab, help = "Naming style")]
    style: StyleArg,

    #[arg(short = 'a', long = "all", help = "Include hidden files and directories")]
    all: bool,

    #[arg(long = "follow-symlinks", help = "Follow symbolic links")]
    follow_symlinks: bool,

    #[arg(long = "on-conflict", value_enum, default_value_t = ConflictArg::Suffix, help = "Conflict strategy")]
    on_conflict: ConflictArg,

    #[arg(long = "unicode", value_enum, default_value_t = UnicodeArg::Ascii, help = "Unicode handling")]
    unicode: UnicodeArg,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum StyleArg {
    Kebab,
    Snake,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ConflictArg {
    Suffix,
    Skip,
    Error,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum UnicodeArg {
    Ascii,
    Preserve,
}

impl From<StyleArg> for NamingStyle {
    fn from(value: StyleArg) -> Self {
        match value {
            StyleArg::Kebab => NamingStyle::Kebab,
            StyleArg::Snake => NamingStyle::Snake,
        }
    }
}

impl From<ConflictArg> for ConflictMode {
    fn from(value: ConflictArg) -> Self {
        match value {
            ConflictArg::Suffix => ConflictMode::Suffix,
            ConflictArg::Skip => ConflictMode::Skip,
            ConflictArg::Error => ConflictMode::Error,
        }
    }
}

impl From<UnicodeArg> for UnicodeMode {
    fn from(value: UnicodeArg) -> Self {
        match value {
            UnicodeArg::Ascii => UnicodeMode::Ascii,
            UnicodeArg::Preserve => UnicodeMode::Preserve,
        }
    }
}

fn main() {
    let mut cmd = Cli::command()
        .color(clap::ColorChoice::Always)
        .styles(help_styles())
        .help_template(
            "{before-help}{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}",
        )
        .after_help(
            "Examples:\n  kebabify \"My File.pdf\"\n  kebabify ~/Downloads -r --dirs --dry-run\n  kebabify -s snake --unicode preserve ~/papers",
        );

    let matches = cmd.get_matches_mut();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    let config = Config {
        dry_run: cli.dry_run,
        recursive: cli.recursive,
        rename_dirs: cli.dirs,
        rename_root: cli.rename_root,
        include_hidden: cli.all,
        follow_symlinks: cli.follow_symlinks,
        style: cli.style.into(),
        on_conflict: cli.on_conflict.into(),
        unicode_mode: cli.unicode.into(),
    };

    match process_paths(&cli.paths, &config) {
        Ok(actions) => {
            for action in &actions {
                if action.from != action.to {
                    println!("{}", format_action(action));
                }
            }

            let summary = summarize(&actions);
            eprintln!(
                "planned: {}, renamed: {}{}",
                summary["planned"],
                summary["renamed"],
                if config.dry_run { " (dry-run)" } else { "" }
            );
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}

fn help_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::BrightBlue.on_default())
        .valid(AnsiColor::BrightGreen.on_default())
        .invalid(AnsiColor::BrightRed.on_default() | Effects::BOLD)
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
}
