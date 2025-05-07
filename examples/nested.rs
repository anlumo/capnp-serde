use capnp_serde::converter::{CapnpSerdeBuilder, CapnpSerdeReader};

mod schemas {
    pub mod example_capnp {
        include!(concat!(env!("OUT_DIR"), "/example_capnp.rs"));
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let mut message =
        capnp::message::TypedBuilder::<schemas::example_capnp::nested::Owned>::new_default();
    let mut root = message.init_root();
    root.set_a(42);
    let mut basic = root.reborrow().init_b();
    basic.set_a(44);
    basic.set_b(false);
    root.set_c("hello");
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
    let back_message: CapnpSerdeBuilder<schemas::example_capnp::nested::Owned> =
        serde_json::from_slice(&json).expect("Failed to deserialize from JSON");

    println!(
        "Deserialized message:\n{:?}\n",
        back_message.into_inner().get_root().unwrap().into_reader()
    );
}
