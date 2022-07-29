use xmltree::{Element};
use serde::{Serialize, Deserialize};
use std::path::{PathBuf, Path};
use std::collections::HashMap;

/// Error returned upon failing to parse something
#[derive(Debug)]
pub enum ParseError {
    Io(PathBuf, std::io::Error),
    XmlParse(xmltree::ParseError),
    Toml(toml::de::Error),
}

impl From<xmltree::ParseError> for ParseError {
    fn from(err: xmltree::ParseError) -> Self {
        Self::XmlParse(err)
    }
}

impl From<toml::de::Error> for ParseError {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpeciesDecl {
    /// Imports xml and svg files from this folder if they aren't found
    pub base: Option<PathBuf>,

    /// The name of the species
    pub name: String,

    #[serde(default)]
    pub variants: HashMap<String, Vec<String>>,

    #[serde(skip)]
    pub template_paths: HashMap<String, PathBuf>,

    #[serde(skip)]
    pub variant_paths: HashMap<String, PathBuf>,

    #[serde(skip)]
    pub asset_paths: HashMap<String, PathBuf>,
}

/// Loads the given file as an XML tree
pub fn load_xml(path: impl AsRef<Path>) -> Result<Element, ParseError> {
    let file = std::fs::File::open(path.as_ref()).map_err(|err| {
        ParseError::Io(path.as_ref().to_path_buf(), err)
    })?;

    Ok(Element::parse(file)?)
}

/// Loads the basic description of a SpeciesDecl
pub fn load_species(path: impl AsRef<Path>) -> Result<SpeciesDecl, ParseError> {
    let declaration_path = path.as_ref().join("species.toml");
    let declaration = std::fs::read_to_string(&declaration_path).map_err(|err| {
        ParseError::Io(declaration_path, err)
    })?;

    let mut res: SpeciesDecl = toml::from_str(&declaration)?;

    if let Some(ref base) = &res.base {
        let path = path.as_ref().to_path_buf().join(base);
        let base = load_species(path)?;

        res.template_paths = base.template_paths;
        res.variant_paths = base.variant_paths;
        res.asset_paths = base.asset_paths;
    }

    // Read the `templates` directory and populate the `template_paths` field;
    // on error, ignore the directory.
    for (name, path) in read_dir_xml(path.as_ref().join("templates")) {
        res.template_paths.insert(name, path);
    }

    // Read the `variants` directory
    for (name, path) in read_dir_xml(path.as_ref().join("variants")) {
        res.variant_paths.insert(name, path);
    }

    // Read the `assets` directory
    for (name, path) in read_dir_xml(path.as_ref().join("assets")) {
        res.asset_paths.insert(name, path);
    }

    Ok(res)
}

fn read_dir_xml(path: impl AsRef<Path>) -> HashMap<String, PathBuf> {
    let mut res = HashMap::new();

    if let Ok(iter) = std::fs::read_dir(path) {
        for entry in iter.filter_map(|x| x.ok()) {
            match (entry.path().file_stem(), entry.path().extension()) {
                (Some(name), Some(ext)) => {
                    if matches!(ext.to_str(), Some("xml") | Some("svg") | Some("mustache")) {
                        if let Some(name) = name.to_str() {
                            res.insert(
                                name.to_string(),
                                entry.path().to_path_buf()
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    res
}
