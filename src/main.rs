mod config;
mod creation_wizard;
mod file_access_logs;
mod finder;
mod presentation;
mod project_type;
mod serve;

use file_access_logs::FileAccessLog;
use finder::look_for_project_file;
use presentation::Presentation;

use clap::{Args, Command, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Generator, Shell};

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::thread;

/// (P)resentation h(elp)er that makes recurring presentations a breeze.
/// Automates conversion of Markdown to revealjs html.
/// Deals with templating and files creation.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, name = "pelp")]
struct Cli {
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
struct EditArgs {
    /// Markdown source file or a name of a presentation series
    #[arg(value_hint = ValueHint::AnyPath)]
    name: Option<String>,
    /// Serve it in the background
    #[arg(short, long)]
    serve: bool,
}

#[derive(Args, Debug, PartialEq)]
struct CreationArgs {
    /// Markdown source file or a name of a presentation series
    #[arg(value_hint = ValueHint::AnyPath)]
    name: Option<String>,

    #[arg(short = 't', long)]
    project_type: Option<project_type::ProjectType>,
}

#[derive(Args, Debug, PartialEq)]
struct CompletionArgs {
    /// Available shells
    shell: Shell,
}

#[derive(Args, Debug, PartialEq)]
struct ListArgs {
    #[arg(short = 't', long)]
    project_type: Option<project_type::ProjectType>,
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
    Edit(EditArgs),
    /// Output files that are going to be used
    Print(NameArgs),
    /// List files fond in current directory, recent presentations and recurring series
    List(ListArgs),
    /// Start a local web server for the presentation & monitor changes to the source .md file
    Serve(NameArgs),
    /// Creates new .md file or a project directory from a template
    New(CreationArgs),
    /// Prints pelp and build tooling versions
    Version,
    /// Generate a shell completion
    GenerateCompletion(CompletionArgs),
}

fn main() {
    let cli = Cli::parse();

    //let presentation = Presentation::new(source_md, output_html, None);

    config::find_user_config();

    match &cli.command {
        Commands::GenerateCompletion(shell_arg) => {
            let mut cmd = Cli::command();
            let shell = shell_arg.shell;
            eprintln!("Generating completion file for {shell:?}...");
            print_completions(shell, &mut cmd);
        }
        Commands::Build(args) => {
            let source_md = get_source(&args.name);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md.clone(), output_html, None);
            presentation.embed_build();
            file_access_logs::log(source_md);
        }
        Commands::Deploy(_) => {
            println!("🛑 Just an idea, not even under construction... 🛑");
        }
        Commands::Edit(args) => {
            let source_md = get_source(&args.name);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md.clone(), output_html, None);
            let presentation_clone = presentation.clone();
            if args.serve {
                thread::spawn(move || {
                    presentation_clone.serve();
                });
            }
            // TODO: do something with random port cuz it's not visible in an editor
            presentation.edit();
            file_access_logs::log(source_md);
        }
        Commands::Print(args) => {
            let source_md = get_source(&args.name);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md, output_html, None);
            println!("{}", presentation);
        }
        Commands::List(_args) => {
            finder::global_search();
            // Look for pelp.toml
            // Look for .md files in current directory
            // Print entries from recent db
            // Print entries from registered projects db
        }
        Commands::Serve(args) => {
            let source_md = get_source(&args.name);
            let output_html = get_output(&cli.output, &source_md);
            let presentation = Presentation::new(source_md.clone(), output_html, None);
            file_access_logs::log(source_md);
            presentation.serve();
        }
        Commands::New(args) => {
            let source_md = creation_wizard::create(args);
            file_access_logs::log(source_md);
        }
        Commands::Version => {
            print_version();
        }
    }
}

fn get_source(name_arg: &Option<String>) -> PathBuf {
    match name_arg {
        Some(markdown_path) => PathBuf::from(markdown_path),
        None => {
            match look_for_project_file() {
                Some(project_file) => {
                    // try opening file specified in a project
                    PathBuf::from("presentation.md")
                }
                None => {
                    let md_files = finder::look_for_md_files();
                    match md_files.len() {
                        0 => panic!("No source markdown files found"),
                        1 => md_files.first().unwrap().clone(),
                        _ => md_files.first().unwrap().clone(),
                    }
                }
            }
        }
    }
    .canonicalize()
    .unwrap()
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

fn print_version() {
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
