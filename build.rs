use std::io::Result;

use prost_build;

fn main() -> Result<()> {
    let steamproto_package_name = "steamproto";
    let steamproto_path = "src/protobufs/steam";
    let steamproto_includes: [&str; 0] = [];
    let steamproto_files = [
        "steammessages_twofactor.steamclient.proto",
        "steammessages_auth.steamclient.proto",
        "steammessages_unified_base.steamclient.proto",
        "steammessages_base.proto",
        "enums.proto",
    ];

    let serializable_messages: [&str; 0] = [];
    let serializable_enums: [&str; 0] = [];

    let mut config = prost_build::Config::new();
    config.protoc_arg(format!("--proto_path={}", steamproto_path));
    config.default_package_filename(steamproto_package_name);

    for path in serializable_messages {
        config.type_attribute(path, "#[derive(serde::Serialize, serde::Deserialize)]");
        config.type_attribute(path, "#[serde(default)]");
    }

    for path in serializable_enums {
        config.type_attribute(path, "#[derive(serde::Serialize, serde::Deserialize)]");
        config.type_attribute(path, "#[serde(rename_all = \"lowercase\")]");
    }

    config.compile_protos(&steamproto_files, &steamproto_includes)

    // let config_toml = include_str!("prost_build_config.toml");
    // let config: BuildConfig = toml::de::from_str(config_toml)
    //     .expect("prost_build_config.toml should be a valid BuildConfig");

    // Builder::from(config).compile_protos()
}

/*
[[messages]]
paths = []
attrs = [
  "derive(serde::Serialize, serde::Deserialize, validator::Validate)",
  "serde(default)"
]

[[enums]]
paths = []
attrs = [
  "derive(serde::Serialize, serde::Deserialize, validator::Validate)",
  "serde(rename_all = \"lowercase\")"
]
*/

// struct Builder {
//     config: prost_build::Config,
//     /// protobuf include dirs
//     includes: Vec<String>,
//     /// protobuf files
//     files: Vec<String>,
// }

// impl Builder {
//     fn compile_protos(&mut self) -> Result<()> {
//         self.config.compile_protos(&self.files, &self.includes)
//     }
// }

// #[derive(Deserialize, Serialize, Debug, Default)]
// #[serde(default)]
// pub struct BuildOption {
//     /// a list of paths you want to add the attribute
//     pub paths: Vec<String>,
//     /// description of the option
//     pub description: String,
//     /// extra attributes to put on generated data structure, for example: `derive(Serialize, Deserialize)`
//     /// it will be converted to `#[derive(Serialize, Deserialize)]`
//     pub attrs: Vec<String>,
// }

// #[derive(Deserialize, Serialize, Debug, Default)]
// #[serde(default)]
// pub struct BuildConfig {
//     pub default_package_filename: Option<String>,
//     /// base path for protobuf files
//     pub base_path: Option<String>,
//     /// protobuf include dirs
//     pub includes: Vec<String>,
//     /// protobuf files
//     pub files: Vec<String>,
//     /// build options for messages
//     pub messages: Vec<BuildOption>,
//     /// build options for enums
//     pub enums: Vec<BuildOption>,
//     /// build options for fields
//     pub fields: Vec<BuildOption>,
//     /// build options for bytes
//     pub bytes: Vec<String>,
//     /// build options for BTreeMap
//     pub btree_maps: Vec<String>,
// }

// impl From<BuildConfig> for Builder {
//     fn from(config: BuildConfig) -> Self {
//         let mut c = prost_build::Config::new();
//         c.btree_map(config.btree_maps);
//         c.bytes(config.bytes);

//         for opt in config.messages {
//             for p in opt.paths {
//                 c.type_attribute(p, to_attr(&opt.attrs));
//             }
//         }

//         for opt in config.enums {
//             for p in opt.paths {
//                 c.type_attribute(p, to_attr(&opt.attrs));
//             }
//         }

//         for opt in config.fields {
//             for p in opt.paths {
//                 c.field_attribute(p, to_attr(&opt.attrs));
//             }
//         }

//         if let Some(default_package_filename) = config.default_package_filename {
//             c.default_package_filename(default_package_filename);
//         }

//         if let Some(base_path) = config.base_path {
//             c.protoc_arg(format!("--proto_path={}", base_path));
//         }

//         Self {
//             config: c,
//             includes: config.includes,
//             files: config.files,
//         }
//     }
// }

// fn to_attr(attrs: &[String]) -> String {
//     attrs
//         .iter()
//         .map(|s| format!("#[{}]", s))
//         .collect::<Vec<_>>()
//         .join("\n")
// }
