use std::collections::HashSet;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use ignore::WalkBuilder;

use punct_converter::{
    ConvertOptions, DecodeError, DetectedEncoding, convert_text_with_options, decode, encode,
};

#[derive(Parser, Debug)]
#[command(name = "punct-converter", version, about)]
struct Args {
    /// File or directory paths (reads from stdin if '-' is given or piped)
    #[arg(value_name = "PATHS")]
    paths: Vec<PathBuf>,

    /// Recursively process subdirectories
    #[arg(short, long)]
    recursive: bool,

    /// Filter directory scans by extension (comma-separated, case-insensitive)
    #[arg(short = 'e', long, value_delimiter = ',', default_value = DEFAULT_EXTS)]
    ext: Vec<String>,

    /// Preview changes without modifying files
    #[arg(long)]
    dry_run: bool,

    /// Save a <filename>.bak backup before overwriting
    #[arg(long)]
    backup: bool,

    /// Always write output as UTF-8 (default preserves UTF-8 / GB18030)
    #[arg(long)]
    to_utf8: bool,

    /// Do not insert a trailing space after converted sentence punctuation
    #[arg(long)]
    no_space: bool,

    /// Skip converting double (“”) and single (‘’) quotes
    #[arg(long)]
    no_quotes: bool,

    /// Do not respect .gitignore rules or skip hidden files
    #[arg(long)]
    no_ignore: bool,
}

const DEFAULT_EXTS: &str = "txt,md,markdown,c,h,cpp,hpp,cc,rs,py,json,yaml,yml,toml,ini,cfg,csv,log,ts,tsx,js,jsx,java,go,cs,sh,bat,ps1,html,css,xml,rst,tex,adoc,org";

fn main() -> ExitCode {
    let args = Args::parse();

    let is_pipe_mode = (args.paths.len() == 1 && args.paths[0].as_os_str() == "-")
        || (args.paths.is_empty() && !io::stdin().is_terminal());

    let opts = ConvertOptions {
        space_after_terminal_punct: !args.no_space,
        convert_quotes: !args.no_quotes,
    };

    if is_pipe_mode {
        let mut buffer = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buffer) {
            eprintln!("Error: failed to read from standard input: {e}");
            return ExitCode::from(1);
        }
        let (text, enc) = match decode(&buffer) {
            Ok(v) => v,
            Err(DecodeError::LikelyBinary) => {
                eprintln!("Error: input appears to be binary data, skipping.");
                return ExitCode::from(1);
            }
        };
        let result = convert_text_with_options(&text, &opts);
        let out_bytes = encode(&result.text, enc, args.to_utf8);
        if let Err(e) = io::stdout().write_all(&out_bytes) {
            eprintln!("Error: failed to write to standard output: {e}");
            return ExitCode::from(1);
        }
        return ExitCode::SUCCESS;
    }

    if args.paths.is_empty() {
        eprintln!(
            "Error: no file or directory paths provided.\nRun `punct-converter --help` for usage, or pipe text via stdin: `echo '你好，世界！' | punct-converter`"
        );
        return ExitCode::from(1);
    }

    let ext_set: HashSet<String> = args
        .ext
        .iter()
        .map(|e| e.trim().trim_start_matches('.').to_lowercase())
        .filter(|e| !e.is_empty())
        .collect();

    let mut files_to_process: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut errors = 0usize;

    for path in &args.paths {
        if !path.exists() {
            errors += 1;
            eprintln!("Error: path does not exist -> {}", path.display());
            continue;
        }
        if path.is_file() {
            if seen.insert(path.clone()) {
                files_to_process.push(path.clone());
            }
            continue;
        }
        if path.is_dir() {
            let mut builder = WalkBuilder::new(path);
            builder.standard_filters(!args.no_ignore);
            if !args.recursive {
                builder.max_depth(Some(1));
            }
            if !args.no_ignore {
                builder.filter_entry(|e| {
                    if e.depth() > 0 && e.file_type().is_some_and(|ft| ft.is_dir()) {
                        !matches!(e.file_name().to_str(), Some("node_modules" | "target"))
                    } else {
                        true
                    }
                });
            }

            for result in builder.build() {
                match result {
                    Ok(entry) => {
                        if entry.file_type().is_some_and(|ft| ft.is_file())
                            && has_wanted_ext(entry.path(), &ext_set)
                        {
                            let p = entry.into_path();
                            if seen.insert(p.clone()) {
                                files_to_process.push(p);
                            }
                        }
                    }
                    Err(err) => {
                        errors += 1;
                        eprintln!("Error: {err}");
                    }
                }
            }
        }
    }

    if files_to_process.is_empty() {
        if errors == 0 {
            eprintln!("No matching files found.");
        }
        return ExitCode::from(1);
    }

    let mut changed = 0usize;
    let mut total_replacements = 0usize;
    let mut skipped_binary = 0usize;

    for file in &files_to_process {
        match process_file(file, args.dry_run, args.backup, args.to_utf8, &opts) {
            Ok(Some(n)) => {
                changed += 1;
                total_replacements += n;
            }
            Ok(None) => {}
            Err(ProcessError::Binary) => {
                skipped_binary += 1;
            }
            Err(ProcessError::Io(e)) => {
                errors += 1;
                eprintln!("Error: {} -> {}", file.display(), e);
            }
        }
    }

    println!(
        "Scanned {} file(s): {} modified ({} replacement(s)), {} suspected binary file(s) skipped{}.",
        files_to_process.len(),
        changed,
        total_replacements,
        skipped_binary,
        if args.dry_run {
            " (dry-run, no files modified)"
        } else {
            ""
        }
    );

    if errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn has_wanted_ext(path: &Path, ext_set: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext_set.contains(&ext.to_lowercase()))
}

enum ProcessError {
    Binary,
    Io(std::io::Error),
}

impl From<std::io::Error> for ProcessError {
    fn from(e: std::io::Error) -> Self {
        ProcessError::Io(e)
    }
}

fn process_file(
    path: &Path,
    dry_run: bool,
    backup: bool,
    to_utf8: bool,
    opts: &ConvertOptions,
) -> Result<Option<usize>, ProcessError> {
    let bytes = fs::read(path)?;

    let (text, enc) = match decode(&bytes) {
        Ok(v) => v,
        Err(DecodeError::LikelyBinary) => return Err(ProcessError::Binary),
    };

    let result = convert_text_with_options(&text, opts);
    let encoding_changed = to_utf8 && enc != DetectedEncoding::Utf8 { bom: false };

    if result.replacements == 0 && !encoding_changed {
        return Ok(None);
    }

    let enc_display = if encoding_changed {
        format!("{} -> UTF-8", enc.label())
    } else {
        enc.label().to_string()
    };

    if dry_run {
        println!(
            "[Preview] {} ({}, {} replacement(s))",
            path.display(),
            enc_display,
            result.replacements
        );
        return Ok(Some(result.replacements));
    }

    if backup {
        let backup_path = append_ext(path, "bak");
        fs::copy(path, backup_path)?;
    }

    let out_bytes = encode(&result.text, enc, to_utf8);
    fs::write(path, out_bytes)?;

    println!(
        "{} ({}, {} replacement(s))",
        path.display(),
        enc_display,
        result.replacements
    );

    Ok(Some(result.replacements))
}

fn append_ext(path: &Path, extra: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".");
    s.push(extra);
    PathBuf::from(s)
}
