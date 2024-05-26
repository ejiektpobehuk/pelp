use clap;
use enum_iterator::Sequence;
use std::fmt;

#[derive(Debug, PartialEq, Sequence, Clone, clap::ValueEnum)]
pub enum ProjectType {
    SingleFile,
    OneShotProject,
    RecurringProject,
}

// implement Display for ProjectType
impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProjectType::SingleFile => write!(f, "Single File"),
            ProjectType::OneShotProject => write!(f, "One Shot Project"),
            ProjectType::RecurringProject => write!(f, "Recurring Project"),
        }
    }
}

impl ProjectType {
    pub fn describe(&self) -> &str {
        match self {
            ProjectType::SingleFile => "when you don't need any local files in your presentation",
            ProjectType::OneShotProject => "creates a directory structure to place your assest in",
            ProjectType::RecurringProject => {
                "creates a directory to store and manage multiple presentations"
            }
        }
    }
}
