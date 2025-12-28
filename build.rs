use std::io::Result;

use prost_build::Config;

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

    let serializable_messages: [&str; 2] = [".CTwoFactor_Time_Request", ".CTwoFactor_Time_Response"];
    let serializable_enums: [&str; 0] = [];

    let mut config = Config::new();
    config.protoc_arg(format!("--proto_path={}", steamproto_path));
    config.default_package_filename(steamproto_package_name);

    for path in serializable_messages {
        config.type_attribute(
            path,
            "#[derive(::steam_encode_derive::Encode, ::steam_encode_derive::Decode)]",
        );
        config.type_attribute(path, "#[encode(proto)]");
        config.type_attribute(path, "#[decode(proto)]");
    }

    // for path in serializable_messages {
    //     config.type_attribute(path, "#[derive(serde::Serialize, serde::Deserialize)]");
    //     config.type_attribute(path, "#[serde(default)]");
    // }

    for path in serializable_enums {
        config.type_attribute(path, "#[derive(serde::Serialize, serde::Deserialize)]");
        config.type_attribute(path, "#[serde(rename_all = \"lowercase\")]");
    }

    config.compile_protos(&steamproto_files, &steamproto_includes)
}
