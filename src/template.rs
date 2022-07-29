use super::*;
use mustache::{Context, Data, MapBuilder, PartialLoader, Template};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use xmltree::{Element, XMLNode};

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
        self.get_builder(variant_name).build()
    }

    fn get_builder(&self, variant_name: &str) -> MapBuilder {
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

        let this = self.clone();
        let variant_name_owned = variant_name.to_string();
        builder = builder.insert_fn("set-fill", move |input| {
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
                            set_fill(&color.trim(), &mut xml);

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

        if let Some(ref parent) = self.parent {
            let parent = parent.clone();
            let variant_name = variant_name.to_string();
            builder = builder.insert_map("parent", move |_| {
                parent.get_builder(&variant_name)
            });
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

        if let Some(ref parent) = self.parent {
            if name.starts_with("parent.") {
                return parent.load(&name[7..]);
            }
        }

        if let Some(path) = self.species.template_paths.get(name) {
            Ok(std::fs::read_to_string(path)?)
        } else {
            eprintln!("No template named {}", name);
            Err(mustache::Error::NoFilename)
        }
    }
}

fn set_fill(color: &str, xml: &mut Element) {
    // Substitute the fill color
    if let Some(style) = xml.attributes.get("style") {
        xml.attributes.insert(
            "style".to_string(),
            format!("{};fill: {};", style, color),
        );
    }
    if let Some(_fill) = xml.attributes.get("fill") {
        xml.attributes.insert("fill".to_string(), color.to_string());
    }

    for child in xml.children.iter_mut() {
        if let XMLNode::Element(ref mut child) = child {
            set_fill(color, child);
        }
    }
}

pub fn query_selector(svg: Element, pattern: &str) -> Option<Element> {
    if pattern == "" {
        return Some(svg);
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
