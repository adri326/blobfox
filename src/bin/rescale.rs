use clap::Parser;
use std::fmt::Write;
use std::path::PathBuf;
use xmltree::{Element, XMLNode};

/// Scales everything (as best as it can) to a different user unit, without breaking the semantics
fn main() {
    let args = Args::parse();

    if args.input == args.output {
        println!("Warning: you should **not** set `output` to the same file as `input`, as this program may very well break your svg!");
    }

    let file = std::fs::File::open(&args.input).unwrap_or_else(|err| {
        panic!("Error while reading {}: {}", args.input.display(), err);
    });
    let mut element = Element::parse(file).expect("Couldn't parse SVG!");

    let view_box = get_viewbox(&element);

    let scale = if let Some(width) = args.width {
        width / view_box.2
    } else if let Some(height) = args.height {
        height / view_box.3
    } else {
        panic!("Expected either width or height as arguments");
    };

    rescale(&mut element, scale);

    *element.attributes.get_mut("viewBox").unwrap() = format!(
        "{} {} {} {}",
        view_box.0 * scale,
        view_box.1 * scale,
        view_box.2 * scale,
        view_box.3 * scale,
    );

    let mut s: Vec<u8> = Vec::new();
    element.write(&mut s).expect("Couldn't export SVG!");
    std::fs::write(&args.output, s).unwrap_or_else(|err| {
        panic!("Error while writing {}: {}", args.output.display(), err);
    });
}

fn get_viewbox(element: &Element) -> (f64, f64, f64, f64) {
    let view_box = element
        .attributes
        .get("viewBox")
        .expect("missing viewBox attribute on svg");
    let view_box: Vec<f64> = view_box
        .split(" ")
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();

    if view_box.len() != 4 {
        panic!(
            "Parse error: expected viewBox to be four space-separated numbers, got {} numbers",
            view_box.len()
        );
    }

    (view_box[0], view_box[1], view_box[2], view_box[3])
}

fn rescale(element: &mut Element, scale: f64) {
    if element.name == "path" {
        if let Some(path) = element.attributes.get_mut("d") {
            let mut new_path = String::new();
            for instruction in path.split_whitespace() {
                if let [Ok(left), Ok(right)] = instruction
                    .split(',')
                    .map(|s| s.parse::<f64>())
                    .collect::<Vec<_>>()[..]
                {
                    write!(&mut new_path, "{},{} ", left * scale, right * scale).unwrap();
                } else if let Ok(number) = instruction.parse::<f64>() {
                    write!(&mut new_path, "{} ", number * scale).unwrap();
                } else {
                    write!(&mut new_path, "{} ", instruction).unwrap();
                }
            }
            *path = new_path;
        }
    } else if element.name == "image" {
        if let Some(width) = element.attributes.get_mut("width") {
            if let Ok(parsed) = width.parse::<f64>() {
                *width = (parsed * scale).to_string();
            }
        }
        if let Some(height) = element.attributes.get_mut("height") {
            if let Ok(parsed) = height.parse::<f64>() {
                *height = (parsed * scale).to_string();
            }
        }
    } else if element.name == "namedview" {
        if let Some(units) = element.attributes.get_mut("document-units") {
            *units = "px".to_string();
        }
    } else if
        element.name == "ellipse"
        || element.name == "radialGradient"
        || element.name == "linearGradient"
        || element.name == "rect"
    {
        const PROPS: [&'static str; 15] = [
            "cx",
            "cy",
            "rx",
            "ry",
            "fx",
            "fy",
            "r",
            "x1",
            "x2",
            "y1",
            "y2",
            "x",
            "y",
            "width",
            "height",
        ];

        for prop in PROPS {
            if let Some(attr) = element.attributes.get_mut(prop) {
                if let Ok(parsed) = attr.parse::<f64>() {
                    *attr = (parsed * scale).to_string();
                }
            }
        }
    } else if element.name == "polygon" {
        if let Some(points) = element.attributes.get_mut("points") {
            let new_points = points.split_whitespace().map(|pair| {
                let parsed = pair.split(',').map(|x| x.parse::<f64>()).collect::<Vec<_>>();
                if let [Ok(x), Ok(y)] = parsed[..] {
                    format!("{},{}", x * scale, y * scale)
                } else if let [Ok(z)] = parsed[..] {
                    format!("{}", z * scale)
                } else {
                    pair.to_string()
                }
            }).collect::<Vec<_>>().join(" ");

            *points = new_points;
        }
    }

    if let Some(style) = element.attributes.get_mut("style") {
        let mut new_style = String::new();
        for instruction in style.split(';') {
            match instruction.split(':').map(|x| x.trim()).collect::<Vec<_>>()[..] {
                ["stroke-width", width] => {
                    let (width, unit): (String, String) =
                        width.chars().partition(|x| !x.is_ascii_alphabetic());
                    let width = width.parse::<f64>().unwrap();

                    write!(&mut new_style, "stroke-width: {}{};", width * scale, unit).unwrap();
                }
                ref any => write!(&mut new_style, "{};", any.join(":")).unwrap(),
            }
        }
        *style = new_style;
    }

    // TODO: replace with Option::or if and when Polonius
    // or another successor to the current borrow-checker gets merged
    let mut transform = element.attributes.get_mut("transform");
    if transform.is_none() {
        transform = element.attributes.get_mut("gradientTransform");
    }

    if let Some(transform) = transform {
        let mut new_transform = String::new();

        for (instruction, args) in parse_transform(transform) {
            match instruction.trim() {
                "rotate" => {
                    let parsed: Vec<f64> = args
                        .split(|c| matches!(c, ' ' | ','))
                        .filter(|s| !s.is_empty())
                        .map(|x| x.parse().unwrap())
                        .collect();
                    if parsed.len() == 3 {
                        write!(
                            &mut new_transform,
                            "rotate({} {} {})",
                            parsed[0],
                            parsed[1] * scale,
                            parsed[2] * scale
                        )
                        .unwrap();
                    } else {
                        write!(&mut new_transform, "rotate({})", args).unwrap();
                    }
                }
                "translate" => {
                    let parsed: Vec<f64> = args
                        .split(|c| matches!(c, ' ' | ','))
                        .filter(|s| !s.is_empty())
                        .map(|x| x.parse().unwrap())
                        .collect();
                    if parsed.len() == 2 {
                        write!(
                            &mut new_transform,
                            "translate({} {})",
                            parsed[0] * scale,
                            parsed[1] * scale
                        )
                        .unwrap();
                    } else {
                        write!(&mut new_transform, "translate({})", args).unwrap();
                    }
                }
                "matrix" => {
                    let parsed: Vec<f64> = args
                        .split(|c| matches!(c, ' ' | ','))
                        .filter(|s| !s.is_empty())
                        .map(|x| x.parse().unwrap())
                        .collect();
                    if parsed.len() == 6 {
                        write!(
                            &mut new_transform,
                            "matrix({} {} {} {} {} {})",
                            parsed[0],
                            parsed[1],
                            parsed[2],
                            parsed[3],
                            parsed[4] * scale,
                            parsed[5] * scale,
                        )
                        .unwrap();
                    } else {
                        write!(&mut new_transform, "matrix({})", args).unwrap();
                    }
                }
                _ => {
                    write!(&mut new_transform, "{}({})", instruction, args).unwrap();
                }
            }
            new_transform.push(' ');
        }

        // TODO: transform-origin

        *transform = new_transform;
    }

    for child in element.children.iter_mut() {
        if let XMLNode::Element(ref mut child) = child {
            rescale(child, scale);
        }
    }
}

fn parse_transform(raw: &str) -> Vec<(String, String)> {
    let mut res: Vec<(String, String)> = Vec::new();
    let mut instruction = String::new();
    let mut args = String::new();
    let mut depth = 0;

    for c in raw.chars() {
        if c == '(' {
            depth += 1;
            if depth > 1 {
                args.push(c);
            }
        } else if c == ')' {
            depth -= 1;
            assert!(depth >= 0);
            if depth >= 1 {
                args.push(c);
            }
        } else if c == ' ' {
            if depth == 0 {
                let instruction = std::mem::take(&mut instruction);
                let args = std::mem::take(&mut args);
                res.push((instruction, args));
            } else {
                args.push(c);
            }
        } else {
            if depth == 0 {
                instruction.push(c);
            } else {
                args.push(c);
            }
        }
    }

    res.push((instruction, args));

    res
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    input: PathBuf,

    #[clap(value_parser)]
    output: PathBuf,

    #[clap(short, long, value_parser)]
    width: Option<f64>,

    #[clap(short, long, value_parser)]
    height: Option<f64>,
}
