//! Very crude tool for generating snuggle emotes
use clap::Parser;
use std::fmt::Write;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use xmltree::{Element, XMLNode};

use blobfox_template::{
    parse,
    template,
    export,
};

#[derive(Serialize, Deserialize, Debug)]
struct Desc {
    dx: f64,
    dy: f64,

    scale: Option<f64>,

    #[serde(default)]
    transform: String,
}

fn main() {
    let args = Args::parse();

    let left = std::fs::read_to_string(&args.input_left).unwrap_or_else(|err| {
        panic!("Couldn't open {}: {}", args.input_right.display(), err);
    });
    let right = std::fs::read_to_string(&args.input_right).unwrap_or_else(|err| {
        panic!("Couldn't open {}: {}", args.input_right.display(), err);
    });

    let desc = std::fs::read_to_string(&args.desc).unwrap_or_else(|err| {
        panic!("Couldn't open {}: {}", args.desc.display(), err);
    });
    let desc: Desc = toml::from_str(&desc).unwrap();

    let snuggle = generate_snuggle(left, right, desc);
    let snuggle = export::xml_to_str(&snuggle).unwrap();

    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from("output/"));

    export::export(
        snuggle,
        &output_dir,
        args.name.clone(),
        &args.clone().into()
    ).unwrap();
}

fn generate_snuggle(left: String, right: String, desc: Desc) -> Element {
    let left_usvg = export::get_usvg(&left).unwrap();
    let left_bbox = left_usvg.svg_node().view_box.rect;

    //
    let mut mask = Element::new("mask");
    mask.attributes.insert("id".to_string(), "snuggle-mask".to_string());

    let mut rect = Element::new("rect");
    rect.attributes.insert("fill".to_string(), "white".to_string());
    // TODO: use scale?
    rect.attributes.insert("x".to_string(), (desc.dx + left_bbox.x()).to_string());
    rect.attributes.insert("y".to_string(), (desc.dy + left_bbox.y()).to_string());
    rect.attributes.insert("width".to_string(), left_bbox.width().to_string());
    rect.attributes.insert("height".to_string(), left_bbox.height().to_string());

    mask.children.push(XMLNode::Element(rect));

    let mut right_mask = Element::new("g");
    right_mask.attributes.insert("transform".to_string(), desc.transform);

    let mut right_xml = Element::parse(right.as_bytes()).unwrap();
    template::set_fill("#000000", &mut right_xml);
    template::set_stroke("#000000", &mut right_xml);

    for child in right_xml.children {
        if let XMLNode::Element(child) = child {
            right_mask.children.push(XMLNode::Element(child));
        }
    }

    mask.children.push(XMLNode::Element(right_mask));

    let mut right_xml = Element::parse(right.as_bytes()).unwrap();
    let left_xml = Element::parse(left.as_bytes()).unwrap();

    let mut left_group = Element::new("g");
    left_group.attributes.insert("transform".to_string(), format!(
        "translate({} {})",
        desc.dx,
        desc.dy
    ));
    left_group.children = left_xml.children;

    let mut left_group2 = Element::new("g");
    left_group2.attributes.insert("mask".to_string(), "url(#snuggle-mask)".to_string());
    left_group2.children.push(XMLNode::Element(left_group));

    let mut res = Element::new("svg");
    res.attributes.insert("xmlns".to_string(), "http://www.w3.org/2000/svg".to_string());
    res.attributes.insert("version".to_string(), "1.1".to_string());
    res.attributes.insert("width".to_string(), "128".to_string());
    res.attributes.insert("height".to_string(), "128".to_string());
    res.children.push(XMLNode::Element(mask));
    res.children.append(&mut right_xml.children);
    res.children.push(XMLNode::Element(left_group2));

    res
}

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    input_left: PathBuf,

    #[clap(value_parser)]
    input_right: PathBuf,

    #[clap(short, long, value_parser)]
    desc: PathBuf,

    /// Disable automatically resizing the SVG's viewBox, defaults to false
    #[clap(short, long, value_parser, default_value = "false")]
    no_resize: bool,

    #[clap(long, value_parser)]
    name: String,

    /// Dimension to export the images as; can be specified multiple times
    #[clap(long, value_parser)]
    dim: Vec<u32>,

    /// Output directory
    #[clap(short, long, value_parser)]
    output_dir: Option<PathBuf>,
}

impl From<Args> for export::ExportArgs {
    fn from(args: Args) -> export::ExportArgs {
        export::ExportArgs {
            no_resize: args.no_resize,
            dim: args.dim,
        }
    }
}
