mod new;
mod presentation;
use presentation::Presentation;

use clap::{Args, Command, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Generator, Shell};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;

/// (P)resentation h(elp)er that makes recurring presentations a breeze.
/// Automates conversion of Markdown to revealjs html.
/// Deals with templating and files creation.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, name = "pelp")]
struct Cli {
    /// Markdown source file
    #[arg(short, long)]
    input: Option<String>,

    /// HTML output file
    #[arg(short, long)]
    output: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    // If provided, outputs the completion file for given shell
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
}

#[derive(Args, Debug, PartialEq)]
struct NameArgs {
    /// Markdown source file or a name of a presentation series
    #[arg(value_hint = ValueHint::AnyPath)]
    name: Option<String>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    /// Generate .html file from an .md one
    #[command(visible_alias = "hint")]
    Build(NameArgs),
    /// Runs a command defined in config to deploy a presentaion
    Deploy(NameArgs),
    /// Open a Markdown source file in an editor. Useful for recurring
    /// scheduled presentations. Creates a new file if it doesn't exist for a
    /// recurring presentation
    Edit(NameArgs),
    /// Output files that are going to be used
    Print(NameArgs),
    /// Start a local web server for the presentation & monitor changes to the source .md file
    Serve(NameArgs),
    /// Creates new .md file from a template
    New,
    /// Prints pelp and build tooling versions
    Version,
}

enum SourceType {
    Argument,    // pelp <FILENAME> or <RECURRING_SERIES>
    Option,      // pelp --input <FILENAME>
    ConfigFile,  // not provided in CLI
    FoundInADir, // not provided in CLI
}

fn main() {
    let cli = Cli::parse();

    // Order of looking for .md file
    // 1. Provided as an `--input` option
    // 2. Provided recurring series as an argument
    // 3. Default recurring series
    //   1. For today if there is an occurrence today
    //   2. `next` subcommand for the next date even if there is an occurrence
    //     today. TODO: what to do with `next` in other cases?
    //   3. Otherwise for the next date

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {generator:?}...");
        print_completions(generator, &mut cmd);
    }

    let source_type: SourceType;
    let source_md = match &cli.input {
        Some(markdown_path) => {
            source_type = SourceType::Option;
            PathBuf::from(markdown_path)
        }
        None => {
            match find_md_file() {
                Some(file) => {
                    source_type = SourceType::FoundInADir;
                    file
                }
                None => panic!("Unable to find a .md file"), // TODO: provide an instruction
            }
        }
    };

    let output_html = match &cli.output {
        Some(html_path) => PathBuf::from(html_path),
        None => {
            // change source file extension to .html
            match source_md.extension() {
                Some(ext) => match ext.to_str() {
                    Some("md") => {
                        let mut html_path = source_md.clone();
                        html_path.set_extension("html");
                        html_path
                    }
                    _ => panic!("Source file is not an .md file"),
                },
                None => panic!("Unable to determice an output file"), // TODO: provide an instruction
            }
        }
    };

    let presentation = Presentation::new(source_md, output_html, None);

    let _ = match &cli.command {
        Some(Commands::Build(_)) => presentation.build(),
        Some(Commands::Deploy(_)) => {
            println!("Under consctuction...");
            Ok(())
        }
        Some(Commands::Edit(_)) => {
            presentation.edit();
            Ok(())
        }
        Some(Commands::Print(_)) => {
            println!("{}", presentation);
            Ok(())
        }
        Some(Commands::Serve(_)) => {
            presentation.serve();
            Ok(())
        }
        Some(Commands::New) => {
            new::create();
            Ok(())
        }
        Some(Commands::Version) => {
            println!("Pelp build info:");
            if let Some(timestamp) = option_env!("VERGEN_BUILD_TIMESTAMP") {
                println!("\tBuild Timestamp: {timestamp}");
            }
            if let Some(describe) = option_env!("VERGEN_GIT_DESCRIBE") {
                println!("\tGit describe: {describe}");
            }
            if let Some(sha) = option_env!("VERGEN_GIT_SHA") {
                println!("\tGit sha: {sha}");
            }
            if let Some(debug) = option_env!("VERGEN_CARGO_DEBUG") {
                println!("\tDebug enabled: {debug}");
            }
            if let Some(da) = option_env!("VERGEN_CARGO_FEATURES") {
                println!("\tCargo features: {da}");
            }
            if let Some(da) = option_env!("VERGEN_CARGO_OPT_LEVEL") {
                println!("\tCargo opt level: {da}");
            }
            if let Some(da) = option_env!("VERGEN_CARGO_TARGET_TRIPLE") {
                println!("\tCargo target triple: {da}");
            }
            if let Some(da) = option_env!("VERGEN_CARGO_DEPENDENCIES") {
                println!("\tCargo Dependencies: {da}");
            }
            if let Some(da) = option_env!("VERGEN_RUSTC_CHANNEL") {
                println!("\tRustc channel: {da}");
            }
            if let Some(da) = option_env!("VERGEN_RUSTC_COMMIT_DATE") {
                println!("\tRustc commit date: {da}");
            }
            if let Some(da) = option_env!("VERGEN_RUSTC_COMMIT_HASH") {
                println!("\tRustc commit hash: {da}");
            }
            if let Some(da) = option_env!("VERGEN_RUSTC_HOST_TRIPLE") {
                println!("\tRustc host triple: {da}");
            }
            if let Some(da) = option_env!("VERGEN_RUSTC_LLVM_VERSION") {
                println!("\tLLVM version: {da}");
            }
            if let Some(da) = option_env!("VERGEN_RUSTC_SEMVER") {
                println!("\tRustc semver: {da}");
            }
            if let Some(da) = option_env!("VERGEN_SYSINFO_OS_VERSION") {
                println!("\tSysinfo OS Version: {da}");
            }
            Ok(())
        }
        None => {
            println!("help output should be here");
            Ok(())
        }
    };
}

fn find_md_file() -> Option<PathBuf> {
    let paths = fs::read_dir("./").expect("Unable to read current directory to look for .md file");
    let md_files: Vec<_> = paths
        .filter(|file| {
            match file
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(OsStr::to_str)
            {
                Some("md") => true,
                _ => false,
            }
        })
        .collect();

    if md_files.len() == 1 {
        return Some(md_files.first().unwrap().as_ref().unwrap().path());
    } else {
        return None;
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
