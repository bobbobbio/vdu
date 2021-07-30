// copyright 2021 Remi Bernotavicius

use bytesize::ByteSize;
use indicatif::{ProgressBar, ProgressStyle};
use num_format::{Locale, ToFormattedString as _};
use std::fs::Metadata;
use std::io;
use std::path::{Path, PathBuf};
use vdu_path_tree::PathTree;
use walkdir::WalkDir;

fn log_path_error(path: &Path) {
    log::warn!("cannot access path {}", path.display());
}

fn error_logging_walk(path: &Path) -> impl Iterator<Item = (PathBuf, Metadata)> {
    WalkDir::new(path)
        .same_file_system(true)
        .into_iter()
        .filter_map(|maybe_entry| {
            match maybe_entry.map(|entry| (entry.metadata(), entry.into_path())) {
                Ok((Ok(meta), path)) => return Some((path, meta)),
                Err(error) => log_path_error(error.path().unwrap()),
                Ok((_, path)) => log_path_error(&path),
            };
            None
        })
}

pub fn build_tree_from_path(path: &Path) -> io::Result<PathTree> {
    log::info!("scanning \"{}\"", path.display());

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            path.to_string_lossy(),
        ));
    }

    let prog_style = ProgressStyle::default_spinner().template("{spinner} {wide_msg}");
    let prog = ProgressBar::new_spinner();
    prog.set_style(prog_style);

    let mut path_tree = PathTree::empty();

    for (path, meta) in error_logging_walk(path) {
        let num_bytes = meta.len();
        path_tree.add_path(&path, num_bytes);

        prog.inc(1);
        let message = format!("{} files", prog.position().to_formatted_string(&Locale::en));
        prog.set_message(message);
    }

    log::info!(
        "found {} files",
        path_tree.size().to_formatted_string(&Locale::en)
    );
    log::info!("total of {} bytes", ByteSize::b(path_tree.num_bytes()));

    Ok(path_tree)
}
