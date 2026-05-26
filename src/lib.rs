use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use unicode_normalization::UnicodeNormalization;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingStyle {
    Kebab,
    Snake,
}

impl NamingStyle {
    fn separator(self) -> char {
        match self {
            NamingStyle::Kebab => '-',
            NamingStyle::Snake => '_',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictMode {
    Suffix,
    Skip,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeMode {
    Ascii,
    Preserve,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dry_run: bool,
    pub recursive: bool,
    pub rename_dirs: bool,
    pub rename_root: bool,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub style: NamingStyle,
    pub on_conflict: ConflictMode,
    pub unicode_mode: UnicodeMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameAction {
    pub from: PathBuf,
    pub to: PathBuf,
}

pub fn process_paths(paths: &[PathBuf], config: &Config) -> Result<Vec<RenameAction>, String> {
    let mut actions = Vec::new();

    for path in paths {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        if path.is_file() {
            process_single_file(path, config, &mut actions)?;
            continue;
        }

        if path.is_dir() {
            process_directory(path, config, &mut actions)?;
            continue;
        }

        return Err(format!("Unsupported path type: {}", path.display()));
    }

    apply_conflicts(&mut actions, config.on_conflict)?;

    if !config.dry_run {
        execute_actions(&actions)?;
    }

    Ok(actions)
}

fn process_single_file(path: &Path, config: &Config, actions: &mut Vec<RenameAction>) -> Result<(), String> {
    if !config.include_hidden && is_hidden(path) {
        return Ok(());
    }

    if let Some(action) = maybe_rename_path(path, config.style, config.unicode_mode) {
        actions.push(action);
    }

    Ok(())
}

fn process_directory(root: &Path, config: &Config, actions: &mut Vec<RenameAction>) -> Result<(), String> {
    if config.rename_root {
        if let Some(action) = maybe_rename_path(root, config.style, config.unicode_mode) {
            actions.push(action);
        }
    }

    if config.recursive {
        let walker = WalkDir::new(root).follow_links(config.follow_symlinks);
        let entries: Result<Vec<_>, _> = walker.into_iter().collect();
        let mut entries = entries.map_err(|e| format!("Walk error in {}: {e}", root.display()))?;
        entries.sort_by_key(|e| depth_sort_key(e));

        for entry in entries {
            let path = entry.path();
            if path == root {
                continue;
            }

            if !config.include_hidden && has_hidden_component(path, root) {
                continue;
            }

            if entry.file_type().is_symlink() && !config.follow_symlinks {
                continue;
            }

            if entry.file_type().is_file() {
                if let Some(action) = maybe_rename_path(path, config.style, config.unicode_mode) {
                    actions.push(action);
                }
            } else if entry.file_type().is_dir() && config.rename_dirs {
                if let Some(action) = maybe_rename_path(path, config.style, config.unicode_mode) {
                    actions.push(action);
                }
            }
        }
    } else {
        let rd = fs::read_dir(root).map_err(|e| format!("Read dir error in {}: {e}", root.display()))?;
        let mut entries: Vec<_> = rd.collect::<Result<_, _>>()
            .map_err(|e| format!("Read dir entry error in {}: {e}", root.display()))?;
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().map_err(|e| format!("File type error {}: {e}", path.display()))?;

            if !config.include_hidden && is_hidden(&path) {
                continue;
            }

            if file_type.is_symlink() && !config.follow_symlinks {
                continue;
            }

            if file_type.is_file() {
                if let Some(action) = maybe_rename_path(&path, config.style, config.unicode_mode) {
                    actions.push(action);
                }
            } else if file_type.is_dir() && config.rename_dirs {
                if let Some(action) = maybe_rename_path(&path, config.style, config.unicode_mode) {
                    actions.push(action);
                }
            }
        }
    }

    Ok(())
}

fn depth_sort_key(entry: &DirEntry) -> (usize, PathBuf) {
    (usize::MAX - entry.depth(), entry.path().to_path_buf())
}

fn has_hidden_component(path: &Path, root: &Path) -> bool {
    if let Ok(rel) = path.strip_prefix(root) {
        return rel.components().any(|c| {
            let name = c.as_os_str().to_string_lossy();
            name.starts_with('.')
        });
    }
    false
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn maybe_rename_path(path: &Path, style: NamingStyle, unicode_mode: UnicodeMode) -> Option<RenameAction> {
    let file_name = path.file_name()?.to_string_lossy();
    let new_name = if path.is_file() {
        rename_file_name(&file_name, style, unicode_mode)
    } else {
        to_case(&file_name, style, unicode_mode)
    };

    if new_name.is_empty() || new_name == file_name {
        return None;
    }

    let parent = path.parent()?;
    Some(RenameAction {
        from: path.to_path_buf(),
        to: parent.join(new_name),
    })
}

fn rename_file_name(name: &str, style: NamingStyle, unicode_mode: UnicodeMode) -> String {
    let (stem, ext) = split_extension(name);
    let base = to_case(stem, style, unicode_mode);
    if base.is_empty() {
        return name.to_string();
    }

    match ext {
        Some(ext) if !ext.is_empty() => format!("{base}.{ext}"),
        _ => base,
    }
}

fn split_extension(name: &str) -> (&str, Option<&str>) {
    if name.starts_with('.') && !name[1..].contains('.') {
        return (name, None);
    }

    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem, Some(ext)),
        _ => (name, None),
    }
}

pub fn to_case(input: &str, style: NamingStyle, unicode_mode: UnicodeMode) -> String {
    let sep = style.separator();
    let normalized = normalize_unicode(input, unicode_mode);
    let mut out = String::new();
    let mut pending_sep = false;

    for ch in normalized.chars() {
        if ch.is_ascii_alphanumeric() || (unicode_mode == UnicodeMode::Preserve && ch.is_alphanumeric()) {
            if pending_sep && !out.is_empty() {
                out.push(sep);
            }
            pending_sep = false;
            out.extend(ch.to_lowercase());
        } else {
            pending_sep = true;
        }
    }

    out.trim_matches(sep).to_string()
}

fn normalize_unicode(input: &str, unicode_mode: UnicodeMode) -> String {
    match unicode_mode {
        UnicodeMode::Preserve => input.to_string(),
        UnicodeMode::Ascii => input
            .nfd()
            .filter(|c| !is_combining_mark(*c))
            .collect::<String>(),
    }
}

fn is_combining_mark(c: char) -> bool {
    matches!(
        c as u32,
        0x0300..=0x036F
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x20D0..=0x20FF
            | 0xFE20..=0xFE2F
    )
}

fn apply_conflicts(actions: &mut [RenameAction], mode: ConflictMode) -> Result<(), String> {
    let mut occupied: HashSet<PathBuf> = HashSet::new();
    let froms: HashSet<PathBuf> = actions.iter().map(|a| a.from.clone()).collect();

    for action in actions.iter_mut() {
        if action.from == action.to {
            continue;
        }

        let mut target = action.to.clone();

        loop {
            let taken_by_action = occupied.contains(&target);
            let exists_on_fs = target.exists() && !froms.contains(&target);

            if !taken_by_action && !exists_on_fs {
                occupied.insert(target.clone());
                action.to = target;
                break;
            }

            match mode {
                ConflictMode::Skip => {
                    action.to = action.from.clone();
                    break;
                }
                ConflictMode::Error => {
                    return Err(format!("Conflict detected for target: {}", target.display()));
                }
                ConflictMode::Suffix => {
                    target = next_suffixed_name(&target);
                }
            }
        }
    }

    Ok(())
}

fn next_suffixed_name(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();

    let (stem, ext) = split_extension(&name);
    let mut index = 1;

    loop {
        let candidate = if let Some(ext) = ext {
            format!("{stem}-{index}.{ext}")
        } else {
            format!("{stem}-{index}")
        };

        let full = parent.join(candidate);
        if !full.exists() {
            return full;
        }

        index += 1;
    }
}

fn execute_actions(actions: &[RenameAction]) -> Result<(), String> {
    let mut by_depth: Vec<_> = actions.iter().collect();
    by_depth.sort_by_key(|a| usize::MAX - depth_of(&a.from));

    for action in by_depth {
        if action.from == action.to {
            continue;
        }

        fs::rename(&action.from, &action.to).map_err(|e| {
            format!(
                "Failed to rename {} -> {}: {e}",
                action.from.display(),
                action.to.display()
            )
        })?;
    }

    Ok(())
}

fn depth_of(path: &Path) -> usize {
    path.components().count()
}

pub fn format_action(action: &RenameAction) -> String {
    format!("{} -> {}", action.from.display(), action.to.display())
}

pub fn summarize(actions: &[RenameAction]) -> HashMap<&'static str, usize> {
    let renamed = actions.iter().filter(|a| a.from != a.to).count();
    HashMap::from([("planned", actions.len()), ("renamed", renamed)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_kebab_ascii() {
        assert_eq!(to_case("Meu Arquivo", NamingStyle::Kebab, UnicodeMode::Ascii), "meu-arquivo");
    }

    #[test]
    fn converts_to_snake_preserve_unicode() {
        assert_eq!(to_case("Olá Mundo", NamingStyle::Snake, UnicodeMode::Preserve), "olá_mundo");
    }

    #[test]
    fn splits_extension() {
        assert_eq!(split_extension("file.txt"), ("file", Some("txt")));
        assert_eq!(split_extension("archive.tar.gz"), ("archive.tar", Some("gz")));
        assert_eq!(split_extension(".bashrc"), (".bashrc", None));
    }
}
