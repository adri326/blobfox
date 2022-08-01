use usvg::{
    Tree,
    NodeExt,
    Options,
};
use xmltree::{XMLNode, Element};
use std::path::{PathBuf};
use std::collections::HashSet;

#[derive(Debug)]
pub enum ExportError {
    Xml(xmltree::Error),
    XmlParse(xmltree::ParseError),
    Usvg(usvg::Error),
    Io(PathBuf, std::io::Error),
    NoBBox,
    Utf8(std::string::FromUtf8Error),
    Encode(png::EncodingError),
}

impl From<xmltree::ParseError> for ExportError {
    fn from(err: xmltree::ParseError) -> Self {
        Self::XmlParse(err)
    }
}

impl From<xmltree::Error> for ExportError {
    fn from(err: xmltree::Error) -> Self {
        Self::Xml(err)
    }
}

impl From<usvg::Error> for ExportError {
    fn from(err: usvg::Error) -> Self {
        Self::Usvg(err)
    }
}

impl From<std::string::FromUtf8Error> for ExportError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<png::EncodingError> for ExportError {
    fn from(err: png::EncodingError) -> Self {
        Self::Encode(err)
    }
}

fn get_new_bbox(svg: &Tree) -> Option<(f64, f64, f64, f64)> {
    let bbox = svg.root().calculate_bbox()?;

    // FIXME: remove once https://github.com/RazrFalcon/resvg/issues/528 is fixed
    const MARGIN: f64 = 2.0;

    let x = bbox.x() - MARGIN;
    let y = bbox.y() - MARGIN;
    let width = bbox.width() + MARGIN * 2.0;
    let height = bbox.height() + MARGIN * 2.0;

    if width > height {
        let y = y - (width - height) / 2.0;

        Some((x, y, width, width))
    } else {
        let x = x - (height - width) / 2.0;

        Some((x, y, height, height))
    }
}

fn get_usvg(svg_str: &str) -> Result<usvg::Tree, usvg::Error> {
    let usvg_options = Options::default();
    Tree::from_str(svg_str, &usvg_options.to_ref())
}

fn get_xml(svg_str: &str) -> Result<Element, xmltree::ParseError> {
    Element::parse(svg_str.as_bytes())
}

fn xml_to_str(svg_xml: &Element) -> Result<String, ExportError> {
    let mut s: Vec<u8> = Vec::new();

    svg_xml.write(&mut s)?;

    Ok(String::from_utf8(s)?)
}

pub fn resize(svg_str: String) -> Result<String, ExportError> {
    if let Some(new_bbox) = get_new_bbox(&get_usvg(&svg_str)?) {
        let mut svg_xml = get_xml(&svg_str)?;
        svg_xml.attributes.insert(
            "viewBox".to_string(),
            format!("{} {} {} {}", new_bbox.0, new_bbox.1, new_bbox.2, new_bbox.3),
        );

        xml_to_str(&svg_xml)
    } else {
        Err(ExportError::NoBBox)
    }
}

/// Finds all the `<defs>` in the svg and combines them all into one
pub fn combine_defs(svg_str: String) -> Result<String, ExportError> {
    let mut svg_xml = get_xml(&svg_str)?;

    let mut defs = Vec::new();

    fn collect_defs(element: &mut Element, defs: &mut Vec<Element>) {
        for child in std::mem::take(&mut element.children) {
            match child {
                XMLNode::Element(child) if child.name == "defs" => {
                    defs.push(child);
                }
                mut child => {
                    if let XMLNode::Element(ref mut child) = &mut child {
                        collect_defs(child, defs);
                    }
                    element.children.push(child);
                }
            }
        }
    }

    collect_defs(&mut svg_xml, &mut defs);

    let mut defs_element = Element::new("defs");
    defs_element.children = defs
        .into_iter()
        .map(|def| {
            def.children.into_iter().filter(|child| matches!(child, XMLNode::Element(_)))
        })
        .flatten()
        .collect::<Vec<_>>();
    defs_element.attributes.insert("id".to_string(), "defs".to_string());

    svg_xml.children.insert(0, XMLNode::Element(defs_element));

    xml_to_str(&svg_xml)
}

pub fn export(
    mut svg_str: String,
    output_dir: &PathBuf,
    output_name: String,
    args: &super::Args,
) -> Result<(), ExportError> {
    if !args.no_resize {
        svg_str = resize(svg_str)?;
    }

    svg_str = combine_defs(svg_str)?;

    mkdirp::mkdirp(output_dir.join("vector")).unwrap();

    let output = output_dir.join(&format!("vector/{}.svg", output_name));
    std::fs::write(output.clone(), svg_str.clone()).map_err(|err| ExportError::Io(output, err))?;

    let svg_usvg = get_usvg(&svg_str)?;
    for resolution in args.dim.iter().copied().filter(|r| *r != 0).collect::<HashSet<_>>() {
        mkdirp::mkdirp(output_dir.join(&format!("{}", resolution))).unwrap();
        let output = output_dir.join(&format!("{}/{}.png", resolution, output_name));

        let mut image = tiny_skia::Pixmap::new(resolution, resolution).unwrap();

        resvg::render(
            &svg_usvg,
            usvg::FitTo::Width(resolution),
            tiny_skia::Transform::identity(),
            image.as_mut()
        ).unwrap();

        image.save_png(output)?;
    }

    Ok(())
}
