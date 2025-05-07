use capnp_serde::{CapnpSerdeBuilder, CapnpSerdeReader};

mod schemas {
    pub mod example_capnp {
        include!(concat!(env!("OUT_DIR"), "/example_capnp.rs"));
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let mut message =
        capnp::message::TypedBuilder::<schemas::example_capnp::complex::Owned>::new_default();
    let mut root = message.init_root();
    root.set_a(&[1, 2, 3, 4, 5]);
    root.set_b("hello world!");
    let mut group_c = root.reborrow().init_c();
    group_c.set_d(14);
    group_c.set_e(true);
    let mut list_f = root.reborrow().init_f(5);
    for i in 0..5 {
        list_f.set(i, format!("entry {i}"));
    }
    let mut list_g = root.reborrow().init_g(10);
    for i in 0..10 {
        list_g.set(i as u32, i * 5);
    }

    let root_reader = root.into_reader();
    println!("Original message:\n{:?}\n", root_reader);

    let serde_reader = CapnpSerdeReader::new(root_reader);

    println!(
        "JSON:\n{}\n",
        serde_json::to_string(&serde_reader).expect("Failed to serialize to JSON")
    );
    // YAML not possible because it doesn't support binary data

    let mut cbor = Vec::new();
    ciborium::into_writer(&serde_reader, &mut cbor).expect("Failed to serialize to CBOR");

    println!("CBOR:\n{cbor:?}\n");

    let back_message: CapnpSerdeBuilder<schemas::example_capnp::complex::Owned> =
        ciborium::from_reader(cbor.as_slice()).expect("Failed to deserialize from CBOR");

    println!(
        "Deserialized message:\n{:?}\n",
        back_message.into_inner().get_root().unwrap().into_reader()
    );
}
