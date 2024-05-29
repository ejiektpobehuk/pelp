use std::env;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::serve;

#[derive(Clone)]
pub struct Presentation {
    source_md: PathBuf,
    output_html: PathBuf,
    base_dir: Option<PathBuf>,
}

impl Presentation {
    pub fn new(
        source_md: PathBuf,
        target_html: PathBuf,
        base_dir: Option<PathBuf>,
    ) -> Presentation {
        Presentation {
            source_md,
            output_html: target_html,
            base_dir,
        }
    }

    fn build(&self, embed: bool) -> Result<(), std::io::Error> {
        let _ = match &self.base_dir {
            Some(dir_path) => env::set_current_dir(dir_path),
            None => Ok(()),
        };
        // Check if pandoc is available
        let _pandoc = Command::new("pandoc")
            .arg("--version")
            .output()
            .expect("Failed to run pandoc");

        let mut build = Command::new("pandoc");
        build.arg("--to=revealjs").arg("--slide-level=2");
        //.arg("--css=um.css")
        if embed {
            build.arg("--embed-resources");
        };
        build
            .arg("--standalone")
            .arg(format!("--output={}", &self.output_html.display()))
            .arg(&self.source_md);
        let build = build.output().expect("Failed to build");

        if build.status.success() {
            println!(
                "{:?} successfully build into {:?}",
                &self.source_md, &self.output_html
            );
            Ok(())
        } else {
            eprintln!(
                "Failed to build {:?}. Pandoc error:\n{}",
                &self.source_md,
                String::from_utf8(build.stderr).unwrap()
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to build",
            ))
        }
    }

    pub fn embed_build(&self) -> Result<(), std::io::Error> {
        self.build(true)
    }

    fn quick_build(&self) -> Result<(), std::io::Error> {
        self.build(false)
    }

    pub fn edit(&self) {
        let _ = match &self.base_dir {
            Some(dir_path) => env::set_current_dir(dir_path),
            None => Ok(()),
        };
        let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());
        let _edit = Command::new(editor)
            .arg(&self.source_md)
            .status()
            .expect("Failed to edit");
    }

    pub fn serve(&self) {
        let _ = match &self.base_dir {
            Some(dir_path) => env::set_current_dir(dir_path),
            None => Ok(()),
        };

        // TODO: hash comparison might speed things up a bit
        let _ = self.quick_build();
        println!("Successfully built");

        serve::serve(self.source_md.clone(), self.output_html.clone());

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).unwrap();
        watcher
            .watch(&self.source_md, RecursiveMode::NonRecursive)
            .unwrap();
        for res in rx {
            match res {
                Ok(event) => {
                    if event.kind.is_modify() {
                        self.quick_build();
                    }
                }
                Err(error) => panic!("Unable to parse an FS event: {}", error),
            }
        }
    }
}

impl fmt::Display for Presentation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(
            f,
            "Markdown: {:?}\nHTML:     {:?}\nBase Dir: {:?}",
            self.source_md, self.output_html, self.base_dir
        )
    }
}
