use clap::Parser;
use std::path::PathBuf;

pub mod parse;
use parse::*;

pub mod template;
use template::*;

pub mod export;
use export::*;

fn main() {
    let args = Args::parse();

    let species = load_species(args.decl.clone()).unwrap();
    let context = RenderingContext::new(species);

    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from("output/"));

    if args.names.is_empty() {
        for name in context.species().variant_paths.keys() {
            generate_variant(&context, name, &output_dir, &args);
        }
    } else {
        for name in args.names.iter() {
            generate_variant(&context, name, &output_dir, &args);
        }
    }
}

fn generate_variant(context: &RenderingContext, name: &str, output_dir: &PathBuf, args: &Args) {
    if let Some(path) = context.species().variant_paths.get(name) {
        match context.compile(path).and_then(|template| {
            template.render_data_to_string(&context.get_data(name))
        }) {
            Ok(svg) => {
                match export(
                    svg,
                    output_dir,
                    format!("{}_{}", context.species().name, name),
                    args
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Error while rendering {}: {:?}", name, err);
                    }
                }
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
pub struct Args {
    /// A folder containing the declaration from which the emotes should be generated
    #[clap(short, long, value_parser)]
    decl: PathBuf,

    /// List of the emote names to export
    #[clap(value_parser)]
    names: Vec<String>,

    /// Disable automatically resizing the SVG's viewBox, defaults to false
    #[clap(short, long, value_parser, default_value = "false")]
    no_resize: bool,

    /// Dimension to export the images as; can be specified multiple times
    #[clap(long, value_parser)]
    dim: Vec<u32>,

    /// Output directory
    #[clap(short, long, value_parser)]
    output_dir: Option<PathBuf>,
}
