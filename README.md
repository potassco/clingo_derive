# clingo_derive

This crate provides the clingo ToSymbol derive macro.

In your `Cargo.toml` add

    [dependencies]
    clingo-derive = "*" 

In your source write:

    use clingo_derive::*;

    #[derive(ToSymbol)]
    struct Point {
        x: i32,
        y: i32,
    }
