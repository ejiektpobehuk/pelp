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
    command: Commands,
}

#[derive(Args, Debug, PartialEq)]
struct NameArgs {
    /// Markdown source file or a name of a presentation series
    #[arg(value_hint = ValueHint::AnyPath)]
    name: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
struct CompletionArgs {
    /// Available shells
    shell: Shell,
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
    /// Creates new .md file or a project directory from a template
    New,
    /// Prints pelp and build tooling versions
    Version,
    /// Generate a shell completion
    GenerateCompletion(CompletionArgs),
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

    //let presentation = Presentation::new(source_md, output_html, None);

    match &cli.command {
        Commands::GenerateCompletion(shell_arg) => {
            let mut cmd = Cli::command();
            let shell = shell_arg.shell;
            eprintln!("Generating completion file for {shell:?}...");
            print_completions(shell, &mut cmd);
        }
        Commands::Build(_) => {
            let (source_type, source_md) = get_source(&cli.input);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md, output_html, None);
            presentation.build();
        }
        Commands::Deploy(_) => {
            println!("Under consctuction...");
        }
        Commands::Edit(_) => {
            let (source_type, source_md) = get_source(&cli.input);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md, output_html, None);
            presentation.edit();
        }
        Commands::Print(_) => {
            let (source_type, source_md) = get_source(&cli.input);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md, output_html, None);
            println!("{}", presentation);
        }
        Commands::Serve(_) => {
            let (source_type, source_md) = get_source(&cli.input);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md, output_html, None);
            presentation.serve();
        }
        Commands::New => {
            new::create();
        }
        Commands::Version => {
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
        }
    }
}

fn get_source(input_arg: &Option<String>) -> (SourceType, PathBuf) {
    match input_arg {
        Some(markdown_path) => (SourceType::Option, PathBuf::from(markdown_path)),
        None => {
            match find_md_file() {
                Some(file) => (SourceType::FoundInADir, file),
                None => panic!("Unable to find a .md file"), // TODO: provide an instruction
            }
        }
    }
}

fn get_output(output_arg: &Option<String>, source_md: &PathBuf) -> PathBuf {
    match output_arg {
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
    }
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
