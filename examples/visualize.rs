use circmark_parse::prelude::*;

fn main() {
    let input = std::fs::read_to_string(std::env::args().nth(1).expect("filename")).expect("read from file");
    let (rest, document) = circmark_parse::document::document(&input).expect("parse circmark document");
    if !rest.trim().is_empty() {
        eprintln!("WARNING: trailing content: {rest:?}");
    }

    let Section::Twoport(chain) = &document.sections[0] else { panic!("Expected twoport chain") };

    println!("{chain:#?}");

    let size = circuit_size(&chain).expect("circuit size");

    println!("Size: {size:?}");

    draw(&chain, &mut PrintDrawer);

    let mut svg_drawer = SvgDrawer::new();

    draw(&chain, &mut svg_drawer);

    svg::save("visualized.svg", &svg_drawer.finalize()).expect("save SVG");
}

struct PrintDrawer;

impl Drawer for PrintDrawer {
    fn series(&mut self, x: usize, element: &Element) {
        println!("Series element at {}: {}", x, element);
    }

    fn shunt(&mut self, x: usize, elements: &[&Element]) {
        for (i, element) in elements.iter().enumerate() {
            println!("Shunt element at {},{}: {}", x, i, element)
        }
    }
}

use svg::node::element::{Path, Rectangle, Group, Text, Circle, path::Data};

struct SvgDrawer {
    document: Option<svg::Document>,
    width: usize,
    height: usize,
}

impl SvgDrawer {
    const CELL_SIZE: i32 = 200;
    const MARGIN: i32 = 60;

    fn new() -> Self {
        Self {
            document: Some(svg::Document::new()),
            width: 0,
            height: 0,
        }
    }

    fn finalize(mut self) -> svg::Document {
        let w = self.width * Self::CELL_SIZE as usize + 2 * Self::MARGIN as usize;
        let h = self.height * Self::CELL_SIZE as usize + 2 * Self::MARGIN as usize;
        self.document.take().expect("document")
            .add(self.make_bottom_line())
            .set("viewBox", format!("0 0 {} {}", w, h))
    }

    fn make_bottom_line(&self) -> Path {
        let w = self.width as i32 * Self::CELL_SIZE;
        let y = self.height as i32 * Self::CELL_SIZE;
        Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new()
                 .move_to((Self::MARGIN, Self::MARGIN + y))
                 .line_to((Self::MARGIN + w, Self::MARGIN + y)))
    }

    fn make_resistor(&self) -> Group {
        let element_width = 70;
        let element_height = 20;
        let line1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((-Self::CELL_SIZE / 2, 0)).line_to((-element_width/2, 0)));
        let rect = Rectangle::new()
            .set("x", -element_width/2)
            .set("y", -element_height/2)
            .set("width", element_width)
            .set("height", element_height)
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", 2);
        let line2 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((element_width/2, 0)).line_to((Self::CELL_SIZE/2, 0)));
        Group::new()
            .add(line1)
            .add(rect)
            .add(line2)
    }

    fn make_voltage_source(&self) -> Group {
        let element_width = 10;
        let element_height = 40;
        let line1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((-Self::CELL_SIZE / 2, 0)).line_to((-element_width/2, 0)));
        let plate1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "4")
            .set("d", Data::new().move_to((-element_width / 2, -element_height/2)).line_to((-element_width/2, element_height/2)));
        let plate2 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "4")
            .set("d", Data::new().move_to((element_width / 2, -element_height/4)).line_to((element_width/2, element_height/4)));
        let line2 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((element_width/2, 0)).line_to((Self::CELL_SIZE/2, 0)));
        Group::new()
            .add(line1)
            .add(plate1)
            .add(plate2)
            .add(line2)
    }

    fn make_capacitor(&self) -> Group {
        let element_width = 10;
        let element_height = 30;
        let plate_width = 5;
        let line1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((-Self::CELL_SIZE / 2, 0)).line_to((-element_width/2, 0)));
        let plate1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", plate_width)
            .set("d", Data::new().move_to((-element_width/2, -element_height/2)).line_to((-element_width/2, element_height/2)));
        let plate2 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", plate_width)
            .set("d", Data::new().move_to((element_width/2, -element_height/2)).line_to((element_width/2, element_height/2)));
        let line2 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((element_width / 2, 0)).line_to((Self::CELL_SIZE/2, 0)));
        Group::new()
            .add(line1)
            .add(plate1)
            .add(plate2)
            .add(line2)
    }

    fn make_inductor(&self) -> Group {
        let element_width = 80;
        let radius = 10;
        let path = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", 2)
            .set("d", Data::new()
                 .move_to((-Self::CELL_SIZE/2, 0))
                 .line_to((-element_width/2, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 2, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 4, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 6, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 8, 0))
                 .line_to((Self::CELL_SIZE/2, 0))
            );
        Group::new()
            .add(path)
    }

    fn make_open(&self) -> Group {
        let circle1 = Circle::new()
            .set("cx", -Self::CELL_SIZE / 2)
            .set("cy", 0)
            .set("r", 5)
            .set("stroke-width", 2)
            .set("stroke", "black")
            .set("fill", "white");
        let circle2 = Circle::new()
            .set("cx", Self::CELL_SIZE / 2)
            .set("cy", 0)
            .set("r", 5)
            .set("stroke-width", 2)
            .set("stroke", "black")
            .set("fill", "white");
        Group::new()
            .add(circle1)
            .add(circle2)
    }

    fn add_series_element(&mut self, x: usize, element: &Element, group: Group) {
        let document = self.document.take().unwrap();
        let tx = Self::MARGIN + x as i32 * Self::CELL_SIZE + Self::CELL_SIZE / 2;
        let ty = Self::MARGIN;
        let label = Text::new()
            .add(svg::node::Text::new(format!("{element}")))
            .set("x", 0)
            .set("y", -Self::MARGIN / 2)
            .set("text-anchor", "middle")
            .set("font-size", "20pt");
        let group = group.add(label);
        self.document = Some(document.add(group.set("transform", format!("translate({},{})", tx, ty))));
        self.width += 1;
    }

    fn add_shunt_element(&mut self, x: usize, y: usize, element: &Element, group: Group) {
        let document = self.document.take().unwrap();
        let tx = Self::MARGIN + x as i32 * Self::CELL_SIZE;
        let ty = Self::MARGIN + y as i32 * Self::CELL_SIZE + Self::CELL_SIZE / 2;
        let label = Text::new()
            .add(svg::node::Text::new(format!("{element}")))
            .set("x", tx + 40)
            .set("y", ty + 10)
            .set("text-anchor", "middle")
            .set("font-size", "20pt");
        self.document = Some(document.add(group.set("transform", format!("translate({},{}) rotate(90)", tx, ty))).add(label));
        self.height = self.height.max(y + 1);
    }
}

impl Drawer for SvgDrawer {
    fn series(&mut self, x: usize, element: &Element) {
        match element {
            Element::R(_) => self.add_series_element(x, element, self.make_resistor()),
            Element::C(_) => self.add_series_element(x, element, self.make_capacitor()),
            Element::L(_) => self.add_series_element(x, element, self.make_inductor()),
            Element::V(_) => self.add_series_element(x, element, self.make_voltage_source()),
            Element::Open => self.add_series_element(x, element, self.make_open()),
            _ => {}
        }
    }

    fn shunt(&mut self, x: usize, elements: &[&Element]) {
        for (y, element) in elements.into_iter().enumerate() {
            match element {
                Element::R(_) => self.add_shunt_element(x, y, element, self.make_resistor()),
                Element::C(_) => self.add_shunt_element(x, y, element, self.make_capacitor()),
                Element::L(_) => self.add_shunt_element(x, y, element, self.make_inductor()),
                Element::V(_) => self.add_shunt_element(x, y, element, self.make_voltage_source()),
                Element::Open => self.add_shunt_element(x, y, element, self.make_open()),
                _ => {}
            }
        }
    }
}

trait Drawer {
    fn series(&mut self, x: usize, element: &Element);
    fn shunt(&mut self, x: usize, elements: &[&Element]);
}

fn draw<D: Drawer>(chain: &Chain, drawer: &mut D) {
    let mut x = 0;
    let mut last_shunt_at = None;
    for node in chain.nodes() {
        match node {
            ChainNode::Series(element) => {
                drawer.series(x, element);
                x += 1;
            }
            ChainNode::Shunt(element) => {
                if let Some(last_shunt_at) = last_shunt_at {
                    if last_shunt_at == x {
                        x += 1;
                    }
                }
                match element {
                    Element::Sub(sub_chain) => {
                        let elements: Vec<_> = sub_chain.nodes().iter().map(|node| {
                            if let ChainNode::Series(element) = node {
                                element
                            } else {
                                unreachable!("there are no nested shunt elements");
                            }
                        }).collect();
                        drawer.shunt(x, &elements);
                    }
                    element => {
                        drawer.shunt(x, &[element]);
                    }
                }
                last_shunt_at = Some(x);
            }
        }
    }
}

fn circuit_size(chain: &Chain) -> Result<(usize, usize), &'static str> {
    let (mut w, mut h) = (0, 1);

    for node in chain.nodes() {
        match node {
            ChainNode::Series(element) => {
                match element {
                    Element::Sub(_) => return Err("Sub-element not allowed in Series position"),
                    _ => w += 1,
                }
            },
            ChainNode::Shunt(element) => {
                if let Element::Sub(sub_chain) = element {
                    h = h.max(sub_chain_length(&sub_chain)?);
                }
            },
        }
    }
    
    Ok((w, h))
}

fn sub_chain_length(chain: &Chain) -> Result<usize, &'static str> {
    let mut l = 0;
    for node in chain.nodes() {
        match node {
            ChainNode::Shunt(_) => return Err("Nested shunt element not allowed"),
            ChainNode::Series(e) => {
                match e {
                    Element::Sub(_) => return Err("Nested sub-elements are not allowed"),
                    _ => l += 1,
                }
            }
        }
    }

    Ok(l)
}
