use capnp_serde::{CapnpSerdeBuilder, CapnpSerdeReader};

mod schemas {
    pub mod example_capnp {
        include!(concat!(env!("OUT_DIR"), "/example_capnp.rs"));
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let mut message = capnp::message::Builder::new_default();
    let mut root = message.init_root::<schemas::example_capnp::unions::Builder>();
    let mut named = root.reborrow().init_named();
    named.set_a(42);
    root.set_d(84);
    let root_reader = root.into_reader();
    println!("Original message:\n{:?}\n", root_reader);

    let serde_reader = CapnpSerdeReader::new(root_reader);

    println!(
        "JSON:\n{}\n",
        serde_json::to_string(&serde_reader).expect("Failed to serialize to JSON")
    );
    println!(
        "YAML:\n{}\n",
        serde_yml::to_string(&serde_reader).expect("Failed to serialize to YAML")
    );

    let json = serde_json::to_vec(&serde_reader).expect("Failed to serialize to JSON");
    let back_message: CapnpSerdeBuilder<schemas::example_capnp::unions::Owned> =
        serde_json::from_slice(&json).expect("Failed to deserialize from JSON");

    println!(
        "Deserialized message:\n{:?}\n",
        back_message.into_inner().get_root().unwrap().into_reader()
    );
}
