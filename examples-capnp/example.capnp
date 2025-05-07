@0x91d22e8672e0d86d;

struct Basic {
    a @0 :UInt32;
    b @1 :Bool;
}

struct Nested {
    a @0 :UInt64;
    b @1 :Basic;
    c @2 :Text;
}

struct Complex {
    a @0 :Data;
    b @1 :Text;
    c :group {
        d @2 :UInt64;
        e @3 :Bool;
    }
    f @4 :List(Text);
    g @5 :List(UInt16);
}
# TODO: lists, enums, unions
