//! Very crude tool for generating snuggle emotes
use clap::Parser;
use std::fmt::Write;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use xmltree::{Element, XMLNode};
use wax::{Glob, Pattern};

use blobfox_template::{
    parse,
    template,
    export,
};

#[derive(Serialize, Deserialize, Debug)]
struct Desc {
    /// Name of the snuggle emote (eg. `snuggle`, `nom`)
    name: String,

    /// How much to move the "left" emote by, horizontally
    dx: f64,
    /// How much to move the "left" emote by, vertically
    dy: f64,
    /// How much to scale the "left" emote by, unimplemented!
    scale: Option<f64>,

    /// How much of a margin to add to the "right" emote, in SVG units
    bold: f64,

    /// Optional transform to add to the "right" emote cutout
    #[serde(default)]
    transform: String,

    /// name/filename list of emotes for the "left" emotes
    left: HashMap<String, String>,
    /// name/filename list of emotes for the "right" emotes
    right: HashMap<String, String>,
}

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the description
    #[clap(short, long, value_parser)]
    desc: PathBuf,

    /// Disable automatically resizing the SVG's viewBox, defaults to false
    #[clap(short, long, value_parser, default_value = "false")]
    no_resize: bool,

    /// Dimension to export the images as; can be specified multiple times
    #[clap(long, value_parser)]
    dim: Vec<u32>,

    /// Input directory, containing the svgs to combine
    #[clap(short, long, value_parser)]
    input_dir: Option<PathBuf>,

    /// Output directory
    #[clap(short, long, value_parser)]
    output_dir: Option<PathBuf>,

    /// A glob to filter which emotes to output; supports wildcards, like `blobfox_snuggle*`
    #[clap(value_parser)]
    glob: Option<String>,
}

impl From<Args> for export::ExportArgs {
    fn from(args: Args) -> export::ExportArgs {
        export::ExportArgs {
            no_resize: args.no_resize,
            dim: args.dim,
        }
    }
}

fn main() {
    let args = Args::parse();
    let input_dir = args.input_dir.clone().unwrap_or(PathBuf::from("output/vector/"));
    let output_dir = args.output_dir.clone().unwrap_or(PathBuf::from("output/"));

    let files = std::fs::read_dir(&input_dir).unwrap_or_else(|err| {
        panic!("Couldn't read directory {}: {}", input_dir.display(), err);
    }).filter_map(|entry| {
        std::fs::read_dir(entry.ok()?.path()).ok()
    }).flatten().filter_map(|entry| {
        let entry = entry.ok()?;
        Some((entry.path().file_stem()?.to_str()?.to_string(), entry.path()))
    }).collect::<HashMap<_, _>>();

    let desc = std::fs::read_to_string(&args.desc).unwrap_or_else(|err| {
        panic!("Couldn't open {}: {}", args.desc.display(), err);
    });
    let desc: Desc = toml::from_str(&desc).unwrap();

    let export_args: export::ExportArgs = args.clone().into();

    let glob = args.glob.as_ref().map(|s| Glob::new(s).expect("Invalid parameter glob"));

    for (left_name, left_path) in desc.left.iter() {
        if let Some(left_path) = files.get(left_path) {
            let left = std::fs::read_to_string(left_path).unwrap_or_else(|err| {
                panic!("Couldn't open {}: {}", left_path.display(), err);
            });

            for (right_name, right_path) in desc.right.iter() {
                if let Some(right_path) = files.get(right_path) {
                    let name = format!("{}_{}_{}", left_name, desc.name, right_name);
                    if let Some(ref glob) = &glob {
                        if !glob.is_match(&*name) {
                            continue // Skip this emote
                        }
                    }

                    let right = std::fs::read_to_string(&right_path).unwrap_or_else(|err| {
                        panic!("Couldn't open {}: {}", right_path.display(), err);
                    });

                    let snuggle = generate_snuggle(&left, &right, &desc);
                    let snuggle = export::xml_to_str(&snuggle).unwrap();

                    export::export(
                        snuggle,
                        &output_dir,
                        &desc.name,
                        &name,
                        &export_args
                    ).unwrap();
                }
            }
        }
    }
}

fn generate_snuggle(left: &str, right: &str, desc: &Desc) -> Element {
    let left_usvg = export::get_usvg(&left).unwrap();
    let left_bbox = left_usvg.svg_node().view_box.rect;

    // == Generate mask ==
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
    right_mask.attributes.insert("transform".to_string(), desc.transform.clone());

    let mut right_xml = Element::parse(right.as_bytes()).unwrap();
    bolden(desc.bold, &mut right_xml);
    template::set_fill("#000000", &mut right_xml);
    template::set_stroke("#000000", &mut right_xml);

    for child in right_xml.children {
        if let XMLNode::Element(child) = child {
            right_mask.children.push(XMLNode::Element(child));
        }
    }

    mask.children.push(XMLNode::Element(right_mask));

    // == Insert both emotes ==
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

    // == Fill in root element ==
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

/// Increases the `stroke-width` of any drawn element by `amount`.
/// If the element has no stroke but has a filling, then it is considered to have a zero stroke width
fn bolden(amount: f64, xml: &mut Element) {
    if let Some(stroke_width) = xml.attributes.get_mut("stroke-width") {
        if let Ok(parsed) = stroke_width.parse::<f64>() {
            *stroke_width = format!("{}", parsed + amount);
        }
    } else if xml.attributes.contains_key("fill") {
        xml.attributes.insert("stroke-width".to_string(), amount.to_string());
    }

    if let Some(style) = xml.attributes.get_mut("style") {
        let mut new_style = String::new();
        let mut stroke_width = None;
        for (name, value) in parse::parse_css(style) {
            if name == "stroke-width" {
                stroke_width = value.parse::<f64>().ok();
                continue
            }

            if name == "fill" && stroke_width.is_none() {
                stroke_width = Some(0.0);
            }

            write!(&mut new_style, "{}:{};", name, value).unwrap();
        }

        if let Some(stroke_width) = stroke_width {
            write!(&mut new_style, "stroke-width: {};", stroke_width + amount).unwrap();
        }

        *style = new_style;
    }

    for child in xml.children.iter_mut() {
        if let XMLNode::Element(ref mut child) = child {
            bolden(amount, child);
        }
    }
}
