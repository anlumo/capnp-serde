use capnp_serde::CapnpSerdeBuilder;

use serde::de::Deserialize;

mod schemas {
    pub mod example_capnp {
        include!(concat!(env!("OUT_DIR"), "/example_capnp.rs"));
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let json = serde_json::json!({
        "a": [1,2,3,4,5],
        "b": "hello world!",
        "c": {
            "d": 14,
            "e": true
        },
        "f": ["entry 0","entry 1","entry 2","entry 3","entry 4"],
        "g": [0,5,10,15,20,25,30,35,40,45],
        "h": [
            { "a": 0, "b": true },
            { "a": 1, "b": false },
            { "a": 2, "b": true }
        ],
        "i":"a",
        "j": ["c","b","a"],
    });

    let back_message =
        CapnpSerdeBuilder::<schemas::example_capnp::complex::Owned>::deserialize(json)
            .expect("Failed to deserialize from serde_json::Value");

    println!(
        "Deserialized message:\n{:?}\n",
        capnp::message::TypedBuilder::from(back_message)
            .get_root_as_reader()
            .unwrap()
    );
}
