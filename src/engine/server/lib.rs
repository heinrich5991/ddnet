mod asdf;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn Foo();
    }
}
fn Foo() {
    println!("foobar");
}
