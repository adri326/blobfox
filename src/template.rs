use crate::parse::{SpeciesDecl, parse_css};
use mustache::{Context, Data, MapBuilder, PartialLoader, Template};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use xmltree::{Element, XMLNode};
use css_color_parser::Color as CssColor;

#[derive(Debug, Clone)]
pub struct RenderingContext {
    species: Arc<SpeciesDecl>,

    rendered_variants: Arc<Mutex<HashMap<String, Element>>>,

    loaded_assets: Arc<Mutex<HashMap<String, Element>>>,

    parent: Option<Box<RenderingContext>>,
}

impl RenderingContext {
    pub fn new(mut species: SpeciesDecl) -> Self {
        let parent = std::mem::take(&mut species.parent).map(|parent| {
            Box::new(Self::new(*parent))
        });

        Self {
            species: Arc::new(species),
            rendered_variants: Arc::new(Mutex::new(HashMap::new())),
            loaded_assets: Arc::new(Mutex::new(HashMap::new())),
            parent
        }
    }

    pub fn compile(&self, path: impl AsRef<Path>) -> Result<Template<Self>, mustache::Error> {
        let template = std::fs::read_to_string(path)?;
        Context::with_loader(self.clone()).compile(template.chars())
    }

    fn render_to_string(
        &self,
        string: &str,
        variant_name: &str,
    ) -> Result<String, mustache::Error> {
        Context::with_loader(self.clone())
            .compile(string.chars())?
            .render_data_to_string(&self.get_data(variant_name))
    }

    pub fn get_data(&self, variant_name: &str) -> Data {
        self.get_builder(variant_name, true).build()
    }

    fn get_builder(&self, variant_name: &str, include_parent: bool) -> MapBuilder {
        let mut builder = MapBuilder::new();

        builder = builder.insert_map("variant", |mut builder| {
            for variant_name in self.species.variant_paths.keys() {
                let this = self.clone();
                let variant_name = variant_name.to_string();
                builder = builder.insert_fn(variant_name.clone(), move |selector| {
                    let svg = this.get_variant(&variant_name);
                    if let Some(svg) = svg {
                        if let Some(element) = query_selector(svg, &selector) {
                            if let Some(string) = xml_to_string(element) {
                                return string;
                            }
                        }
                    }

                    String::new()
                })
            }
            builder
        });

        for asset_name in self.species.asset_paths.keys() {
            let this = self.clone();
            let asset_name = asset_name.to_string();

            builder = builder.insert_fn(asset_name.clone(), move |selector| {
                let svg = this.get_asset(&asset_name);
                if let Some(svg) = svg {
                    if let Some(element) = query_selector(svg, &selector) {
                        if let Some(string) = xml_to_string(element) {
                            return string;
                        }
                    }
                }

                String::new()
            });
        }

        for (cb, name) in [
            (set_fill as fn(&str, &mut Element), "set-fill"),
            (set_stroke, "set-stroke")
        ] {
            let this = self.clone();
            let variant_name_owned = variant_name.to_string();

            builder = builder.insert_fn(name, move |input| {
                // Parse `color|xml`
                if let [color, xml] = input.splitn(2, '|').collect::<Vec<_>>()[..] {
                    // Render `color` and `xml`
                    if let (Ok(color), Ok(xml)) = (
                        this.render_to_string(&color, &variant_name_owned),
                        this.render_to_string(&xml, &variant_name_owned),
                    ) {
                        // Convert `xml` to XML
                        match Element::parse(xml.as_bytes()) {
                            Ok(mut xml) => {
                                cb(&color.trim(), &mut xml);

                                // Render XML to string
                                if let Some(res) = xml_to_string(xml) {
                                    res
                                } else {
                                    String::from("<!-- Error in stringifying xml -->")
                                }
                            }
                            Err(err) => {
                                format!("<!-- Error in parsing xml: {} -->", err)
                            }
                        }
                    } else {
                        String::from("<!-- Error in parsing color or element -->")
                    }
                } else {
                    String::from("<!-- Invalid syntax: expected `color|xml` -->")
                }
            });
        }

        builder = builder.insert("vars", &self.species.vars).unwrap();

        if include_parent {
            let mut this = self.clone();

            loop {
                builder = builder.insert_map(&this.species.name, |_| {
                    this.get_builder(variant_name, false)
                });

                if let Some(ref parent) = this.parent {
                    this = *parent.clone();
                } else {
                    break
                }
            }
        }

        // TODO: memoize the builder to this stage

        // Variant tags
        if let Some(tags) = self.species.variants.get(variant_name) {
            builder = builder.insert_map("tags", move |mut builder| {
                for tag in tags.iter() {
                    builder = builder.insert_bool(tag, true);
                }

                builder
            });
        }

        builder
    }

    pub fn get_variant(&self, name: &String) -> Option<Element> {
        let rendered = self.rendered_variants.lock().unwrap().get(name).cloned();
        if let Some(rendered) = rendered {
            Some(rendered)
        } else if let Some(path) = self.species.variant_paths.get(name) {
            // TODO: log error
            let template = self.compile(path).ok()?;
            let data = self.get_data(name);
            let rendered = template.render_data_to_string(&data).ok()?;

            let parsed = Element::parse(rendered.as_bytes()).ok()?;
            self.rendered_variants
                .lock()
                .unwrap()
                .insert(name.clone(), parsed.clone());

            Some(parsed)
        } else {
            None
        }
    }

    pub fn get_asset(&self, name: &String) -> Option<Element> {
        let loaded = self.loaded_assets.lock().unwrap().get(name).cloned();
        if let Some(loaded) = loaded {
            Some(loaded)
        } else if let Some(path) = self.species.asset_paths.get(name) {
            let string = std::fs::read_to_string(path).ok()?;
            let parsed = Element::parse(string.as_bytes()).ok()?;
            self.loaded_assets
                .lock()
                .unwrap()
                .insert(name.clone(), parsed.clone());

            Some(parsed)
        } else {
            None
        }
    }

    pub fn species(&self) -> Arc<SpeciesDecl> {
        Arc::clone(&self.species)
    }
}

impl PartialLoader for RenderingContext {
    fn load(&self, name: impl AsRef<Path>) -> Result<String, mustache::Error> {
        let name = name.as_ref().to_str().ok_or(mustache::Error::InvalidStr)?;

        let components = name.split('.').collect::<Vec<_>>();

        if components.len() == 1 {
            if let Some(path) = self.species.template_paths.get(name) {
                Ok(std::fs::read_to_string(path)?)
            } else {
                eprintln!("No template named {}", name);
                Err(mustache::Error::NoFilename)
            }
        } else if components.len() == 2 {
            if components[0] == self.species.name {
                self.load(components[1])
            } else if let Some(ref parent) = self.parent {
                parent.load(name)
            } else {
                eprintln!(
                    "Cannot get template named {}: no species called {} in the inheritance tree",
                    name,
                    components[0]
                );
                Err(mustache::Error::NoFilename)
            }
        } else {
            eprintln!("Cannot get template named {}: expected `name` or `species.name`", name);
            Err(mustache::Error::NoFilename)
        }
    }
}

macro_rules! set_color {
    ( $fn_name:tt, $color_name:expr, $opacity_name:expr ) => {
        pub fn $fn_name(color: &str, xml: &mut Element) {
            let (color, opacity) = if let Ok(parsed) = color.parse::<CssColor>() {
                (format!("#{:02x}{:02x}{:02x}", parsed.r, parsed.g, parsed.b), parsed.a)
            } else {
                (color.to_string(), 1.0)
            };

            fn rec(color: &str, opacity: f32, xml: &mut Element) {
                if let Some(style) = xml.attributes.get_mut("style") {
                    let mut new_style = Vec::new();

                    for (name, value) in parse_css(style) {
                        if name != $color_name && name != $opacity_name {
                            new_style.push(format!("{}:{}", name, value));
                        }
                    }

                    new_style.push(format!(concat!($color_name, ": {};"), color));
                    new_style.push(format!(concat!($opacity_name, ": {};"), opacity));

                    *style = new_style.join(";");
                }
                if let Some(_fill) = xml.attributes.get($color_name) {
                    xml.attributes.insert($color_name.to_string(), color.to_string());
                }
                if let Some(_fill) = xml.attributes.get($opacity_name) {
                    xml.attributes.insert($opacity_name.to_string(), opacity.to_string());
                }

                for child in xml.children.iter_mut() {
                    if let XMLNode::Element(ref mut child) = child {
                        rec(color, opacity, child);
                    }
                }
            }

            rec(&color, opacity, xml)
        }
    }
}

set_color!(set_fill, "fill", "fill-opacity");
set_color!(set_stroke, "stroke", "stroke-opacity");

pub fn query_selector(svg: Element, pattern: &str) -> Option<Element> {
    if pattern == "" {
        // NOTE: it looks like having a nested svg makes resvg unhappy
        let mut group = Element::new("g");
        group.children = svg.children;
        return Some(group);
    }

    for child in svg.children {
        if let XMLNode::Element(child) = child {
            if let ("#", pattern_id) = pattern.split_at(1) {
                if child
                    .attributes
                    .get("id")
                    .map(|id| id == pattern_id)
                    .unwrap_or(false)
                {
                    return Some(child);
                } else if child.children.len() > 0 {
                    if let Some(res) = query_selector(child, pattern) {
                        return Some(res);
                    }
                }
            }
        }
    }

    None
}

pub fn xml_to_string(element: Element) -> Option<String> {
    let mut s: Vec<u8> = Vec::new();
    let mut config = xmltree::EmitterConfig::default();
    config.perform_indent = true;
    config.write_document_declaration = false;

    element.write_with_config(&mut s, config).ok()?;

    String::from_utf8(s).ok()
}
