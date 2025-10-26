/// hello world
#[derive(Debug)]
struct Foo {
    _greet: String,
    _num: Option<i32>,
}
fn main() {
    // this is a comment
    loop {
        let s: &str = r#"the world"#;
        let foo = Foo {
            _greet: s.to_string(),
            _num: Some(123),
        };
        println!("Hello, {foo:?}!");
        break;
    }
    println!("The end.");
}
