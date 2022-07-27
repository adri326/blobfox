use xmltree::{XMLNode, Element};
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let args = Args::parse();

    for path in args.files {
        let file = std::fs::File::open(path.clone()).unwrap_or_else(|err| {
            panic!("Error while reading {}: {}", path.display(), err);
        });
        let mut element = Element::parse(file).expect("Couldn't parse SVG!");

        clean(&mut element);

        let mut s: Vec<u8> = Vec::new();
        element.write(&mut s).expect("Couldn't export SVG!");

        std::fs::write(path.clone(), s).unwrap_or_else(|err| {
            panic!("Error while writing {}: {}", path.display(), err);
        });
    }
}

fn clean(element: &mut Element) {
    let mut counts: HashMap<String, usize> = HashMap::new();

    fn count_rec(element: &Element, counts: &mut HashMap<String, usize>) {
        if let Some(label) = element.attributes.get("label") {
            if let Some(count) = counts.get_mut(label) {
                *count += 1;
            } else {
                counts.insert(label.to_string(), 1);
            }
        }

        for child in element.children.iter() {
            if let XMLNode::Element(ref child) = child {
                count_rec(child, counts);
            }
        }
    }

    count_rec(element, &mut counts);

    fn update_rec(element: &mut Element, counts: &HashMap<String, usize>) {
        if let Some(label) = element.attributes.get("label") {
            if let Some(1) = counts.get(label) {
                element.attributes.insert("id".to_string(), label.to_string());
            }
        }

        for child in element.children.iter_mut() {
            if let XMLNode::Element(ref mut child) = child {
                update_rec(child, counts);
            }
        }
    }

    update_rec(element, &counts);
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    files: Vec<PathBuf>,
}
