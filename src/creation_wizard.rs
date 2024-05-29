use std::fs;
use std::path::PathBuf;
use std::process;

use crate::project_type::ProjectType;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::CreationArgs;

pub fn create(args: &CreationArgs) -> PathBuf {
    let project_path: String = if args.name.is_some() {
        args.name.clone().unwrap()
    } else {
        // TODO: Hint to a user that using `.md` leads to a single file presentation
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("presentation name")
            .interact_text()
            .unwrap()
    };
    let project_name = if project_path.contains('/') {
        PathBuf::from(&project_path).file_name().unwrap().to_str().unwrap().to_string()
        //project_path.split('/').last().unwrap().to_string()
    } else {
        project_path.clone()
    };

    let project_type = if args.project_type.is_some() {
        args.project_type.clone().unwrap()
    } else {
        if project_path.ends_with(".md") {
            println!("Single file presentation");
            ProjectType::SingleFile
        } else {
            println!("Choose a type of presentation you'd like to create.");

            for pres_type in enum_iterator::all::<ProjectType>() {
                println!("{} - {}", style(&pres_type).bold(), &pres_type.describe())
            }
            println!();

            let selections = enum_iterator::all::<ProjectType>().collect::<Vec<_>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Pick a presentation type")
                .default(0)
                .items(&selections[..])
                .interact()
                .unwrap();
            selections[selection].clone()
        }
    };

    // The creation
    match project_type {
        ProjectType::SingleFile => {
            let template = include_str!("template.md");
            let filepath = if project_path.ends_with(".md") {
                PathBuf::from(project_path)
            } else {
                let mut path = PathBuf::from(project_path);
                path.set_extension("md");
                path
            };
            if filepath.exists() {
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "File {} already exists, do you want to overwite it?",
                        filepath.to_str().unwrap()
                    ))
                    .default(false)
                    .interact()
                    .unwrap()
                {
                    create_single_file_presentation(filepath, template)
                } else {
                    println!("Aborting, no file was created");
                    process::exit(3)
                }
            } else {
                create_single_file_presentation(filepath, template)
            }
        }
        ProjectType::OneShotProject => {
            let template = include_str!("template.md");
            let path = PathBuf::from(project_path.clone());
            if path.exists() {
                eprintln!("Unable to create file, it aleady exists: {}", project_path);
                process::exit(69);
            } else {
                // temporary solution
                // I don't not how to store project templates & create projects yet
                println!("Path: {:?}", &path);
                match std::fs::create_dir(path.clone()) {
                    Ok(_) => {

                        println!("project_name: {:?}", &project_name);
                        let mut presentation_path = path.join(&project_name);
                        presentation_path.set_extension("md");
                        println!("presentation path: {:?}", &presentation_path);

                        create_single_file_presentation(presentation_path, template)
                    }
                    Err(e) => {
                        eprintln![
                            "Error while creating directory {}: {}",
                            path.to_str().unwrap(),
                            e.to_string()
                        ];
                        process::exit(2);
                    }
                }
            }
        }
        ProjectType::RecurringProject => {
            eprintln!(
                "\n{}",
                style("🚧 Recurring Projects / Presentation series are under construction 🚧")
                    .bold()
            );
            process::exit(4);
        }
    }
}

fn create_single_file_presentation(path: PathBuf, template: &str) -> PathBuf {
    match fs::write(path.clone(), template) {
        Ok(_) => {
            println!(
                "\nPresentation created 🎉\n\n\
            {} to open it in a browser\n\
            {} to open in your editor",
                style(format!("pelp serve {}", path.to_str().unwrap())).bold(),
                style(format!("pelp edit {}", path.to_str().unwrap())).bold()
            );
            path.clone()
        }
        Err(e) => {
            eprintln!("Error while creating file: {}", e);
            process::exit(69);
        }
    }
}
