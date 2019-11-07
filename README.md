[![Build Status](https://travis-ci.org/potassco/clingo_derive.svg?branch=master)](https://travis-ci.org/potassco/clingo_derive)

# clingo_derive

This crate provides the derive macro for the [`clingo::ToSymbol`](https://docs.rs/clingo/0.6.0/clingo/trait.ToSymbol.html) trait.
Datatypes that implement this trait can be added to a [`clingo::FactBase`](https://docs.rs/clingo/0.6.0/clingo/struct.FactBase.html)

In your `Cargo.toml` add:

    [dependencies]
    clingo = { version = "0.6", features = ["derive"] }

In your source write:

    use clingo::ToSymbol;
    use clingo::ClingoError;
    use clingo::FactBase;

    #[derive(ToSymbol)]
    struct Point {
        x: i32,
        y: i32,
    }

    let p = Point{ x:4, y:2 };
    let fb = FactBase::new();
    fb.insert(p);

The macro performs a conversion to snake case. This means the corresponing fact for `MyPoint{x:3,y:2}` is `my_point(3,2)`.
