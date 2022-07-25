use clap::Parser;
use std::path::PathBuf;
use std::collections::HashMap;

pub mod decl;
use decl::*;

pub mod emote;
use emote::*;

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

    let mut emotes = HashMap::new();

    let to_generate = if args.names.len() > 0 {
        args.names
    } else {
        declaration.variants.keys().cloned().collect()
    };

    for name in to_generate {
        match construct_emote(&name, &declaration, &mut emotes) {
            Some(emote) => {
                emotes.insert(name, dbg!(emote));
                // generate emote
            }
            None => {
                eprintln!("Errors occured while generating emote {}, skipping!", name);
            }
        }
    }

        // if !emotes.contains_key(&name) {
        //     let emote = Emote::from_decl(variant, &mut emotes);
        //     if let Some(emote) = emote {
        //         emotes.insert(name, emote);
        //     }
        // }
    // }
}

/// Recursively constructs the emote with variant declaration named `named`.
/// If the declaration has a base, that base is generated first.
// I could unwrap the recursion into a loop with a stack, but I'm lazy :3
fn construct_emote(name: &str, declaration: &Declaration, emotes: &mut HashMap<String, Emote>) -> Option<Emote> {
    if let Some(emote_decl) = declaration.variants.get(name) {
        if let Some(base_name) = emote_decl.base.clone() {
            let base = construct_emote(&base_name, declaration, emotes)?;
            emotes.insert(base_name, base);
        }

        Emote::from_decl(emote_decl.clone(), &*emotes)
    } else {
        eprintln!("No emote named {}!", name);
        None
    }
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
