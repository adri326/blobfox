use clap::Parser;
use std::path::PathBuf;

pub mod decl;
use decl::*;

fn main() {
    let args = Args::parse();

    let mut path = args.decl_file;
    let raw = std::fs::read_to_string(&path).unwrap();
    let mut declaration: Declaration = serde_yaml::from_str(&raw).unwrap();
    declaration.canonicalize(&path);

    while let Some(parent) = std::mem::take(&mut declaration.base) {
        path = path.parent().unwrap_or(&PathBuf::from(".")).join(parent);
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|err| {
             // TODO: print the include stack
            panic!("Couldn't read {}: {}", path.display(), err);
        });

        let mut parent_decl: Declaration = serde_yaml::from_str(&raw).unwrap();
        parent_decl.canonicalize(&path);

        declaration = declaration.join(parent_decl);
    }

    println!("{:#?}", declaration);
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// A YAML file containing the declaration from which the emotes should be generated
    #[clap(short, long, value_parser)]
    decl_file: PathBuf,

    /// List of the emote names to export
    #[clap(value_parser)]
    names: Vec<String>,
}
