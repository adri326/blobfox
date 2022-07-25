use super::{AssetDecl, Variant};

use xmltree::{XMLNode, Element};
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug, Clone)]
pub struct Emote {
    pub assets: HashMap<String, Asset>,

    // TODO: metadata (contributors, etc.)
}

#[derive(Debug, Clone)]
pub struct Asset {
    element: Element,
}

impl Emote {
    pub fn empty() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn from_decl(variant_decl: Variant, emotes: &HashMap<String, Emote>) -> Option<Self> {
        // Load base or src
        let mut res = if let Some(src) = variant_decl.src {
            let file = File::open(src).ok()?;
            let xml = Element::parse(file).ok()?;
            Self::from_xml(xml)?
        } else if let Some(base) = variant_decl.base {
            emotes.get(&base)?.clone()
        } else {
            Emote::empty()
        };

        // Load individual assets
        for asset_decl in variant_decl.assets {
            let name = asset_decl.name.clone();
            res.assets.insert(name, Asset::from_decl(asset_decl)?);
        }

        // Apply overwrites (TODO)

        Some(res)
    }

    /// Loads an emote from an svg file.
    /// Top-level `<g>` elements are ignored
    pub fn from_xml(root: Element) -> Option<Self> {
        let iter = root.children.into_iter()
            .map(|elem| -> Box<dyn Iterator<Item=XMLNode>> {
                if let XMLNode::Element(elem2) = elem {
                    if elem2.name == "g" {
                        Box::new(elem2.children.into_iter())
                    } else {
                        Box::new(Some(XMLNode::Element(elem2)).into_iter())
                    }
                } else {
                    Box::new(None.into_iter())
                }
            })
            .flatten()
            .filter_map(|elem| {
                match elem {
                    XMLNode::Element(elem) => Some(elem),
                    _ => None
                }
            })
            .filter(filter_drawable);

        let mut assets = HashMap::new();

        for elem in iter {
            if let Some(name) = elem.attributes.get("label").or(elem.attributes.get("id")).cloned() {
                let asset = Asset::new(elem);
                assets.insert(name, asset);
            }
        }

        Some(Self {
            assets
        })
    }
}

impl Asset {
    pub fn new(element: Element) -> Self {
        Self {
            element
        }
    }

    // TODO: allow loading assets from other variants (requires a more thorough dependency management)
    // TODO: allow searching for a specific element
    pub fn from_decl(declaration: AssetDecl) -> Option<Self> {
        let mut element = if let Some(src) = declaration.src {
            let file = File::open(src).ok()?;
            Element::parse(file).ok()?
        } else {
            unimplemented!()
        };

        Some(Self {
            element
        })
    }
}

// NOTE: This isn't really viable,
fn filter_drawable(elem: &Element) -> bool {
    match elem.name.as_str() {
        "title" | "sodipodi:namedview" | "image" | "metadata" => false,
        _ => true,
    }
}
