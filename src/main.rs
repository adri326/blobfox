use std::sync::Arc;
use clap::Parser;
use std::path::PathBuf;

pub mod parse;
use parse::*;

pub mod template;
use template::*;

fn main() {
    let args = Args::parse();

    let species = Arc::new(load_species(args.decl).unwrap());
    let context = RenderingContext::new(species);

    let output_dir = args.output_dir.unwrap_or(PathBuf::from("output/vector/"));

    mkdirp::mkdirp(output_dir.clone()).unwrap();

    if args.names.is_empty() {
        for name in context.species().variants.keys() {
            generate_variant(&context, name, &output_dir);
        }
    } else {
        for name in args.names.iter() {
            generate_variant(&context, name, &output_dir);
        }
    }
}

fn generate_variant(context: &RenderingContext, name: &str, output_dir: &PathBuf) {
    if let Some(path) = context.species().variants.get(name) {
        match context.compile(path).and_then(|template| {
            template.render_data_to_string(&context.get_data())
        }) {
            Ok(rendered) => {
                let output = output_dir.join(&format!("{}_{}.svg", context.species().name, name));
                std::fs::write(output, rendered).unwrap();
            }
            Err(err) => {
                eprintln!("Error while rendering {}: {}", name, err);
            }
        }
    } else {
        eprintln!("No variant named {}!", name);
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// A folder containing the declaration from which the emotes should be generated
    #[clap(short, long, value_parser)]
    decl: PathBuf,

    /// List of the emote names to export
    #[clap(value_parser)]
    names: Vec<String>,

    /// Output directory
    #[clap(short, long, value_parser)]
    output_dir: Option<PathBuf>,
}
