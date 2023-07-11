use std::io::Read;
use circmark_parse::{
    circuit,
    draw::{self, Draw},
    layout::Layout,
};

fn main() {
    let mut input;
    if let Some(arg) = std::env::args().nth(1) {
        input = arg;
    } else {
        input = String::new();
        std::io::stdin().read_to_string(&mut input).unwrap();
    }
    let (rest, document) = circuit::document(&input).expect("parse");
    if rest.len() > 0 {
        eprintln!("WARNING: trailing input {rest:?}");
    }
    let mut svg_drawer = draw::svg::SvgDrawer::new();
    document.draw(document.layout_size(), draw::Context::default(), &mut svg_drawer);
    svg::write(std::io::stdout(), &svg_drawer.finalize()).expect("write");
}
