use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use prompted::input;
use std::env;
use std::fs::{create_dir, write};
use std::path::PathBuf;

enum Create {
    FolderOwned(String, Vec<Create>),
    FolderStatic(&'static str, Vec<Create>),
    // FileStatic(&'static str, fn(&str) -> String),
    FileOwned(String, fn(&str) -> String),
}

pub fn new(dry_run: bool) -> Result<()> {
    let serial = input!("#️⃣ PCB serial number: ");
    let serial: u32 = serial.trim().parse().context("Invalid serial number")?;
    let board_name = format!("b{serial}");
    let short_description = input!("🆒 Short PCB function description: ");
    let short_description = short_description.to_case(Case::Snake);

    let simple_structure = vec![Create::FolderOwned(
        format!("b{serial}_{short_description}"),
        vec![
            Create::FolderStatic("spec", vec![]),
            Create::FolderStatic(
                "source",
                vec![Create::FolderOwned(format!("b{serial}a"), vec![])],
            ),
            Create::FolderStatic(
                "change",
                vec![Create::FileOwned(
                    format!("change_b{serial}a.md"),
                    change_contents,
                )],
            ),
        ],
    )];
    let working_dir = env::current_dir()?;
    create_folder_structure(working_dir, &simple_structure, &board_name, dry_run)?;
    Ok(())
}

fn create_folder_structure(
    current_dir: PathBuf,
    structure: &[Create],
    board_name: &str,
    dry_run: bool,
) -> Result<()> {
    for create in structure {
        match create {
            Create::FolderOwned(folder_name, children) => {
                let path = current_dir.join(folder_name);
                if dry_run {
                    println!("create dir: {path:?}");
                } else {
                    create_dir(&path)?;
                }
                create_folder_structure(path, children, board_name, dry_run)?;
            }
            Create::FolderStatic(folder_name, children) => {
                let path = current_dir.join(folder_name);
                if dry_run {
                    println!("create dir: {path:?}");
                } else {
                    create_dir(&path)?;
                }
                create_folder_structure(path, children, board_name, dry_run)?;
            }
            // Create::FileStatic(file_name, contents_fn) => {
            //     let path = current_dir.join(file_name);
            //     if dry_run {
            //         println!("create file: {path:?}");
            //     } else {
            //         let contents = contents_fn(board_name);
            //         write(&path, &contents)?;
            //     }
            // }
            Create::FileOwned(file_name, contents_fn) => {
                let path = current_dir.join(file_name);
                if dry_run {
                    println!("create file: {path:?}");
                } else {
                    let contents = contents_fn(board_name);
                    write(&path, &contents)?;
                }
            }
        }
    }
    Ok(())
}

fn change_contents(board_name: &str) -> String {
    format!(
        "\
# Change list for board {board_name}\n\
"
    )
}
