use std::io::Result;
fn main() -> Result<()> {
    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_arg("--proto_path=src/protobufs/steam");
    prost_build.default_package_filename("steamproto");
    prost_build.compile_protos(
        &[
            "steammessages_auth.steamclient.proto",
            "steammessages_unified_base.steamclient.proto",
            "steammessages_base.proto",
            "enums.proto",
        ],
        &["src/"],
    )
}
