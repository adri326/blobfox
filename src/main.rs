use std::sync::Arc;

pub mod parse;
use parse::*;

pub mod template;
use template::*;

fn main() {
    let species = Arc::new(dbg!(load_species("species/blobfox")).unwrap());
    let context = RenderingContext::new(species);
    let template = context.compile("species/blobfox/variants/base.svg").unwrap();
    let rendered = template.render_data_to_string(&context.get_data()).unwrap();
    println!("{}", rendered);
    std::fs::write("./test.svg", rendered).unwrap();
}
