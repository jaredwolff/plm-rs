use serde::Deserialize;

// Used to parse an Eagle .sch file.
// Has all the necessary hooks to get the proper
// Information from the files!

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Eagle {
    pub drawing: Drawing,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Drawing {
    pub schematic: Schematic,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Schematic {
    pub parts: Parts,
    pub attributes: Attributes,
    pub libraries: Libraries,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Libraries {
    pub library: Vec<Library>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Library {
    pub name: String,
    pub devicesets: DeviceSets,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DeviceSets {
    pub deviceset: Vec<DeviceSet>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DeviceSet {
    pub name: String,
    pub devices: Devices,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Devices {
    pub device: Vec<Device>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Device {
    pub name: String,
    pub technologies: Technologies,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Technologies {
    pub technology: Vec<Technology>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Technology {
    pub name: String,
    pub attribute: Option<Vec<Attribute>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Attributes {
    pub attribute: Vec<Attribute>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Parts {
    pub part: Vec<Part>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Part {
    pub name: String,
    pub deviceset: String,
    pub technology: Option<String>,
    pub device: String,
    pub variant: Option<Variant>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Variant {
    pub name: String,
    pub populate: String,
}
