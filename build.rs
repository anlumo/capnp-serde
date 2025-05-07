fn main() {
    if std::env::var("CARGO_FEATURE_EXAMPLES").is_err() {
        return;
    }

    capnpc::CompilerCommand::new()
        .file("examples-capnp/example.capnp")
        .src_prefix("examples-capnp")
        .default_parent_module(vec!["schemas".into()])
        .run()
        .expect("failed to run capnpc");
    println!("cargo:rerun-if-changed=examples-capnp/example.capnp");
}
