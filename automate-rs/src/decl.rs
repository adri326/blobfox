use std::path::{PathBuf, Path};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Declaration {
    pub name: String,
    pub base: Option<String>,
    pub variants: HashMap<String, Variant>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Variant {
    pub base: Option<String>,

    pub src: Option<PathBuf>, // Loads every asset from an SVG file
    #[serde(default)]
    pub assets: Vec<Asset>, // Loads individual assets

    #[serde(default)]
    pub overwrites: Vec<Overwrite>, // Operations on assets
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Asset {
    pub src: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Overwrite {
    pub id: String, // ID of the element to modify

    pub fill: Option<u32>,
    pub stroke: Option<u32>,

    #[serde(default)]
    pub remove: bool,
}

impl Declaration {
    pub fn join(self, parent: Self) -> Self {
        let mut variants = self.variants;

        for (name, parent_variant) in parent.variants {
            if let Some(variant) = variants.get_mut(&name) {
                variant.join(parent_variant);
            } else {
                variants.insert(name, parent_variant);
            }
        }

        Self {
            name: self.name,
            base: parent.base,
            variants
        }
    }

    /// Replaces every path relative to the yaml file to paths relative to the cwd
    pub fn canonicalize(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
        for variant in self.variants.values_mut() {
            variant.canonicalize(path.as_ref());
        }
    }
}

impl Variant {
    pub fn join(&mut self, mut parent: Self) {
        self.assets.append(&mut parent.assets);
        self.overwrites.append(&mut parent.overwrites);

        if self.base.is_none() {
            self.base = parent.base;
        }

        if self.src.is_none() {
            self.src = parent.src; // TODO: handle relative paths
        }
    }

    /// Replaces every path relative to the yaml file to paths relative to the cwd
    pub fn canonicalize(&mut self, path: &Path) {
        match &mut self.src {
            Some(src_path) => *src_path = path.join(&*src_path),
            None => {}
        }

        for asset in &mut self.assets {
            asset.canonicalize(path);
        }

        // for overwrite in &mut self.overwrites {
        //     overwrite.canonicalize(path);
        // }
    }
}

impl Asset {
    /// Replaces every path relative to the yaml file to paths relative to the cwd
    pub fn canonicalize(&mut self, path: &Path) {
        match &mut self.src {
            Some(src_path) => *src_path = path.join(&*src_path),
            None => {}
        }
    }
}
