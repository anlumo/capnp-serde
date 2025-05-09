use capnp_serde::{CapnpSerdeBuilder, CapnpSerdeReader};

mod schemas {
    pub mod example_capnp {
        include!(concat!(env!("OUT_DIR"), "/example_capnp.rs"));
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let mut message =
        capnp::message::TypedBuilder::<schemas::example_capnp::basic::Owned>::new_default();
    let mut root = message.init_root();
    root.set_a(42);
    root.set_b(true);
    let root_reader = root.into_reader();
    println!("Original message:\n{:?}\n", root_reader);

    let serde_reader = CapnpSerdeReader::from(root_reader);

    println!(
        "JSON:\n{}\n",
        serde_json::to_string(&serde_reader).expect("Failed to serialize to JSON")
    );
    println!(
        "YAML:\n{}",
        serde_yml::to_string(&serde_reader).expect("Failed to serialize to YAML")
    );
    println!(
        "MessagePack:\n{:x?}\n",
        rmp_serde::to_vec(&serde_reader).expect("Failed to serialize to MessagePack")
    );

    let json = serde_json::to_vec(&serde_reader).expect("Failed to serialize to JSON");
    let back_message: CapnpSerdeBuilder<schemas::example_capnp::basic::Owned> =
        serde_json::from_slice(&json).expect("Failed to deserialize from JSON");

    println!(
        "Deserialized message:\n{:?}\n",
        capnp::message::TypedBuilder::from(back_message)
            .get_root_as_reader()
            .unwrap()
    );
}
