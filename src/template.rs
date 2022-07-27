use mustache::{
    Context,
    PartialLoader,
    Template,
    MapBuilder,
    Data,
};
use super::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use xmltree::{XMLNode, Element};

#[derive(Debug, Clone)]
pub struct RenderingContext {
    species: Arc<SpeciesDecl>,

    rendered_variants: Arc<Mutex<HashMap<String, Element>>>,

    loaded_assets: Arc<Mutex<HashMap<String, Element>>>,
}

impl RenderingContext {
    pub fn new(species: Arc<SpeciesDecl>) -> Self {
        Self {
            species,
            rendered_variants: Arc::new(Mutex::new(HashMap::new())),
            loaded_assets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn compile(&self, path: impl AsRef<Path>) -> Result<Template<Self>, mustache::Error> {
        let template = std::fs::read_to_string(path)?;
        Context::with_loader(self.clone()).compile(template.chars())
    }

    pub fn get_data(&self) -> Data {
        let mut builder = MapBuilder::new();

        builder = builder.insert_map("variant", |mut builder| {
            for variant_name in self.species.variants.keys() {
                let this = self.clone();
                let variant_name = variant_name.to_string();
                builder = builder.insert_fn(variant_name.clone(), move |selector| {
                    let svg = this.get_variant(&variant_name);
                    if let Some(svg) = svg {
                        if let Some(element) = query_selector(svg, &selector) {
                            if let Some(string) = xml_to_string(element) {
                                return string
                            }
                        }
                    }

                    String::new()
                })
            }
            builder
        });

        for asset_name in self.species.assets.keys() {
            let this = self.clone();
            let asset_name = asset_name.to_string();

            builder = builder.insert_fn(asset_name.clone(), move |selector| {
                let svg = this.get_asset(&asset_name);
                if let Some(svg) = svg {
                    if let Some(element) = query_selector(svg, &selector) {
                        if let Some(string) = xml_to_string(element) {
                            return string
                        }
                    }
                }

                String::new()
            });
        }

        builder.build()
    }

    pub fn get_variant(&self, name: &String) -> Option<Element> {
        let rendered = self.rendered_variants.lock().unwrap().get(name).cloned();
        if let Some(rendered) = rendered {
            Some(rendered)
        } else if let Some(path) = self.species.variants.get(name) {
            // TODO: log error
            let template = self.compile(path).ok()?;
            let data = self.get_data();
            let rendered = template.render_data_to_string(&data).ok()?;

            let parsed = Element::parse(rendered.as_bytes()).ok()?;
            self.rendered_variants.lock().unwrap().insert(name.clone(), parsed.clone());

            Some(parsed)
        } else {
            None
        }
    }

    pub fn get_asset(&self, name: &String) -> Option<Element> {
        let loaded = self.loaded_assets.lock().unwrap().get(name).cloned();
        if let Some(loaded) = loaded {
            Some(loaded)
        } else if let Some(path) = self.species.assets.get(name) {
            let string = std::fs::read_to_string(path).ok()?;
            let parsed = Element::parse(string.as_bytes()).ok()?;
            self.loaded_assets.lock().unwrap().insert(name.clone(), parsed.clone());

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

        if let Some(path) = self.species.templates.get(name) {
            Ok(std::fs::read_to_string(path)?)
        } else {
            eprintln!("No template named {}", name);
            Err(mustache::Error::NoFilename)
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
                if child.attributes.get("id").map(|id| id == pattern_id).unwrap_or(false) {
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
