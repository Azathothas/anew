use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str;
use clap::Parser;
use indexmap::IndexSet;
mod utils;
//opts
#[derive(Parser, Debug)]
#[command(author = "zer0yu", version, about = "A tool for adding new lines to files, skipping duplicates", long_about = None)]
struct Options {
    #[arg(short, long, help = "Do not output new lines to stdout")]
    quiet_mode: bool,

    #[arg(short, long, help = "Sort lines (natsort)")]
    sort: bool,

    #[arg(short, long, help = "Trim whitespaces")]
    trim: bool,

    #[arg(
        short,
        long,
        help = "Rewrite existing destination file to remove duplicates"
    )]
    rewrite: bool,

    #[arg(
        short,
        long,
        help = "Do not write to file, only output what would be written"
    )]
    dry_run: bool,

    #[arg(help = "Destination file")]
    filepath: String,
}
// mkdir|touch dirs & files
fn pre_run_setup(args: &Options) -> io::Result<()> {
   // if !args.dry_run {
        let path = Path::new(&args.filepath);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if !path.exists() {
            File::create(&args.filepath)?;
        }
    //}
    Ok(())
}
// post-run cleanup on --dry-run
fn post_run_cleanup(args: &Options) -> io::Result<()> {
    if args.dry_run {
        let path = Path::new(&args.filepath);
        if path.exists() {
            let dir_path = path.parent().unwrap_or_else(|| Path::new(""));
            if path.is_file() {
                fs::remove_file(&args.filepath)?;
                if fs::read_dir(&dir_path)?.next().is_none() {
                    fs::remove_dir(&dir_path)?;
                }
            }
        }
    }
    Ok(())
}
// main
fn main() -> io::Result<()> {
    let args = Options::parse();
    pre_run_setup(&args)?;
    let mut lines = load_file(&args)?;

    if args.rewrite && !args.dry_run {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&args.filepath)?;
        let mut writer = BufWriter::new(file);

        for line in lines.iter() {
            writeln!(writer, "{}", line)?;
        }
    }

    let stdin = io::stdin();
    let file = OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(&args.filepath)?;
    let mut writer = BufWriter::new(file);

    for stdin_line in stdin.lock().lines() {
        let stdin_line = stdin_line?;

        if should_add_line(&args, &lines, &stdin_line) {
            lines.insert(stdin_line.clone());

            if !args.quiet_mode {
                println!("{}", stdin_line);
            }

            if !args.sort && !args.dry_run {
                writeln!(writer, "{}", stdin_line)?;
            }
        }
    }

    if args.sort && !args.dry_run {
        let mut sorted_lines: Vec<_> = lines.into_iter().collect();
        sorted_lines.sort_by(|a, b| utils::natsort::compare(a, b, false));

        let sort_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&args.filepath)?;
        let mut sort_writer = BufWriter::new(sort_file);

        for line in sorted_lines.iter() {
            writeln!(sort_writer, "{}", line)?;
        }
    }
    post_run_cleanup(&args)?;
    Ok(())
}
// anew
fn load_file(args: &Options) -> Result<IndexSet<String>, io::Error> {
    let file = File::open(&args.filepath)?;
    let reader = BufReader::new(file);
    let mut lines = IndexSet::new();

    for line in reader.lines() {
        let line = line?;
        if should_add_line(args, &lines, &line) {
            lines.insert(line);
        }
    }

    Ok(lines)
}
// trim
fn should_add_line(args: &Options, lines: &IndexSet<String>, line: &str) -> bool {
    let trimmed_line = if args.trim { line.trim() } else { line };
    !trimmed_line.is_empty() && !lines.contains(trimmed_line)
}
// End
