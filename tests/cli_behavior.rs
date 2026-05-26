use std::fs;

use kebabify::{process_paths, Config, ConflictMode, NamingStyle, UnicodeMode};
use tempfile::tempdir;

fn base_config() -> Config {
    Config {
        dry_run: false,
        recursive: false,
        rename_dirs: false,
        rename_root: false,
        include_hidden: false,
        follow_symlinks: false,
        style: NamingStyle::Kebab,
        on_conflict: ConflictMode::Suffix,
        unicode_mode: UnicodeMode::Ascii,
    }
}

#[test]
fn renames_single_file() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("Meu Arquivo.pdf");
    fs::write(&src, b"x").unwrap();

    let cfg = base_config();
    let actions = process_paths(&[src.clone()], &cfg).unwrap();

    assert_eq!(actions.len(), 1);
    assert!(dir.path().join("meu-arquivo.pdf").exists());
}

#[test]
fn dry_run_does_not_change_fs() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("Meu Arquivo.pdf");
    fs::write(&src, b"x").unwrap();

    let mut cfg = base_config();
    cfg.dry_run = true;

    let actions = process_paths(&[src.clone()], &cfg).unwrap();

    assert_eq!(actions[0].to.file_name().unwrap().to_string_lossy(), "meu-arquivo.pdf");
    assert!(src.exists());
    assert!(!dir.path().join("meu-arquivo.pdf").exists());
}

#[test]
fn non_recursive_directory_only_immediate_files() {
    let dir = tempdir().unwrap();
    let top = dir.path().join("Meu Arquivo.txt");
    let subdir = dir.path().join("Sub Pasta");
    let nested = subdir.join("Outro Arquivo.txt");

    fs::write(&top, b"x").unwrap();
    fs::create_dir(&subdir).unwrap();
    fs::write(&nested, b"y").unwrap();

    let cfg = base_config();
    process_paths(&[dir.path().to_path_buf()], &cfg).unwrap();

    assert!(dir.path().join("meu-arquivo.txt").exists());
    assert!(nested.exists());
}

#[test]
fn recursive_and_dirs_renames_nested_dirs_and_files() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("Sub Pasta");
    let nested = subdir.join("Outro Arquivo.txt");

    fs::create_dir(&subdir).unwrap();
    fs::write(&nested, b"x").unwrap();

    let mut cfg = base_config();
    cfg.recursive = true;
    cfg.rename_dirs = true;

    process_paths(&[dir.path().to_path_buf()], &cfg).unwrap();

    let renamed_dir = dir.path().join("sub-pasta");
    assert!(renamed_dir.exists());
    assert!(renamed_dir.join("outro-arquivo.txt").exists());
}

#[test]
fn conflict_suffix_mode_appends_counter() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("Meu Arquivo.txt");
    let existing = dir.path().join("meu-arquivo.txt");

    fs::write(&src, b"x").unwrap();
    fs::write(&existing, b"y").unwrap();

    let cfg = base_config();
    process_paths(&[src], &cfg).unwrap();

    assert!(dir.path().join("meu-arquivo-1.txt").exists());
}

#[test]
fn hidden_entries_skipped_by_default() {
    let dir = tempdir().unwrap();
    let src = dir.path().join(".Meu Arquivo.txt");
    fs::write(&src, b"x").unwrap();

    let cfg = base_config();
    process_paths(&[src.clone()], &cfg).unwrap();

    assert!(src.exists());
}
