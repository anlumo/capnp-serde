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

enum Foo {
    a @0;
    b @1;
    c @2;
    d @3;
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
    h @6 :List(Basic);
    i @7 :Foo;
    j @8 :List(Foo);
    shouldbenull @9 :Basic;
    default @10 :UInt64 = 12;
}

struct Unions {
    named :union {
        a @0 :UInt32;
        b @1 :Void;
        c @2 :Basic;
    }
    union {
        d @3 :UInt32;
        e @4 :Void;
        f @5 :Basic;
    }
}
