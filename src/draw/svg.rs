use svg::node::element::{Path, Rectangle, Group, Text, Circle, path::Data};
use crate::layout::{self, Size, Position};

pub struct SvgDrawer {
    root: Option<Group>,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl SvgDrawer {
    pub fn new() -> Self {
        Self {
            root: Some(Group::new()),
            min_x: 0,
            max_x: 0,
            min_y: 0,
            max_y: 0,
        }
    }

    pub fn finalize(self) -> svg::Document {
        let margin = 30;
        let w = self.max_x - self.min_x + 2 * margin;
        let h = self.max_y - self.min_y + 2 * margin;
        let document = svg::Document::new();
        document
            .add(
                self.root.unwrap()
                    .set("transform", format!("translate({},{})", w/2, h/2))
            )
            .set("viewBox", format!("0 0 {} {}", w, h))
            .set("width", w)
            .set("height", h)
            .set("style", "background: white")
    }
}

impl SvgDrawer {
    fn add<N: svg::Node>(&mut self, node: N) {
        self.root = Some(self.root.take().unwrap().add(node));
    }

    fn transform(&self, group: Group, position: layout::Position, rotate: bool) -> Group {
        group.set("transform", format!("translate({},{}) rotate({})", position.0, position.1, if rotate { 90 } else { 0 }))
    }

    fn grow_viewbox(&mut self, position: Position, size: Size, rotate: bool) {
        let size = if rotate { Size(size.1, size.0) } else { size };
        let min_x = position.0 - size.0 / 2;
        let max_x = position.0 + size.0 / 2;
        let min_y = position.1 - size.1 / 2;
        let max_y = position.1 + size.1 / 2;
        self.min_x = self.min_x.min(min_x);
        self.max_x = self.max_x.max(max_x);
        self.min_y = self.min_y.min(min_y);
        self.max_y = self.max_y.max(max_y);
    }

    fn label(&self, label: &str, rotate: bool, xoff: i32, yoff: i32) -> Text {
        let (lx, ly, ltrans) = if rotate {
            (xoff, 5, "rotate(-90)")
        } else {
            (0, yoff, "")
        };
        Text::new()
            .add(svg::node::Text::new(label))
            .set("x", lx)
            .set("y", ly)
            .set("text-anchor", "middle")
            .set("transform", ltrans)
    }        
}

impl super::Drawer for SvgDrawer {
    fn resistor(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        self.grow_viewbox(position, size, rotate);
        let element_width = 70;
        let element_height = 20;
        let line1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((-size.0 / 2, 0)).line_to((-element_width/2, 0)));
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
            .set("d", Data::new().move_to((element_width/2, 0)).line_to((size.0/2, 0)));
        self.add(self.transform(
            Group::new()
                .add(line1)
                .add(rect)
                .add(line2)
                .add(self.label(label, rotate, 0, 4)),
            position,
            rotate
        ));
    }

    fn capacitor(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        self.grow_viewbox(position, size, rotate);
        let element_width = 10;
        let element_height = 30;
        let plate_width = 5;
        let line1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((-size.0 / 2, 0)).line_to((-element_width/2, 0)));
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
            .set("d", Data::new().move_to((element_width / 2, 0)).line_to((size.0/2, 0)));
        self.add(self.transform(
            Group::new()
                .add(line1)
                .add(plate1)
                .add(plate2)
                .add(line2)
                .add(self.label(label, rotate, 30, 30)),
            position,
            rotate
        ));
    }

    fn inductor(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        self.grow_viewbox(position, size, rotate);
        let element_width = 80;
        let radius = 10;
        let path = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", 2)
            .set("d", Data::new()
                 .move_to((-size.0/2, 0))
                 .line_to((-element_width/2, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 2, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 4, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 6, 0))
                 .elliptical_arc_to((radius, radius, 0, 0, 1, -element_width/2 + radius * 8, 0))
                 .line_to((size.0/2, 0))
            );
        self.add(self.transform(Group::new().add(path).add(self.label(label, rotate, -25, -20)), position, rotate));
    }

    fn voltage_source(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        self.grow_viewbox(position, size, rotate);
        let element_width = 10;
        let element_height = 40;
        let line1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((-size.0 / 2, 0)).line_to((-element_width/2, 0)));
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
            .set("d", Data::new().move_to((element_width/2, 0)).line_to((size.0/2, 0)));
        self.add(self.transform(
            Group::new()
                .add(line1)
                .add(plate1)
                .add(plate2)
                .add(line2)
                .add(self.label(label, rotate, 30, 30)),
            position,
            rotate
        ))
    }

    fn current_source(&mut self, label: &str, position: Position, size: Size, rotate: bool) {
        let radius = 15;
        let offset = 10;
        self.grow_viewbox(position, size, rotate);
        let circle1 = Circle::new()
            .set("cx", -offset)
            .set("cy", 0)
            .set("r", radius)
            .set("stroke-width", 2)
            .set("stroke", "black")
            .set("fill", "none");
        let circle2 = Circle::new()
            .set("cx", offset)
            .set("cy", 0)
            .set("r", radius)
            .set("stroke-width", 2)
            .set("stroke", "black")
            .set("fill", "none");
        let line1 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((-size.0/2, 0)).line_to((-(offset + radius), 0)));
        let line2 = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((offset + radius, 0)).line_to((size.0 / 2, 0)));
        self.add(self.transform(
            Group::new()
                .add(line1)
                .add(circle1)
                .add(circle2)
                .add(line2)
                .add(self.label(label, rotate, 30, 30)),
            position,
            rotate
        ))
    }

    fn open(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        self.grow_viewbox(position, size, rotate);
        let circle1 = Circle::new()
            .set("cx", -size.0 / 2)
            .set("cy", 0)
            .set("r", 5)
            .set("stroke-width", 2)
            .set("stroke", "black")
            .set("fill", "white");
        let circle2 = Circle::new()
            .set("cx", size.0 / 2)
            .set("cy", 0)
            .set("r", 5)
            .set("stroke-width", 2)
            .set("stroke", "black")
            .set("fill", "white");
        self.add(self.transform(Group::new().add(circle1).add(circle2).add(self.label(label, rotate, 30, 30)), position, rotate))
    }

    fn wire(&mut self, a: layout::Position, b: layout::Position) {
        let line = Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("stroke-width", "2")
            .set("d", Data::new().move_to((a.0, a.1)).line_to((b.0, b.1)));
        self.add(line);
    }

    fn junction(&mut self, position: layout::Position) {
        let circle = Circle::new()
            .set("cx", position.0)
            .set("cy", position.1)
            .set("r", 3)
            .set("fill", "black");
        self.add(circle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{circuit, draw::{Draw, Context}, layout::Layout};

    #[test]
    fn test_draw_single_resistor() {
        let mut drawer = SvgDrawer::new();
        let element = circuit::Element::R("1");
        element.draw(element.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_single_resistor.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_single_capacitor() {
        let mut drawer = SvgDrawer::new();
        let element = circuit::Element::C("1");
        element.draw(element.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_single_capacitor.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_single_inductor() {
        let mut drawer = SvgDrawer::new();
        let element = circuit::Element::L("1");
        element.draw(element.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_single_inductor.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_single_resistor_rotated() {
        let mut drawer = SvgDrawer::new();
        let element = circuit::Element::R("1");
        element.draw(element.layout_size(), Context::default().rotate(), &mut drawer);
        svg::save("test-output/draw_single_resistor_rotated.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_single_capacitor_rotated() {
        let mut drawer = SvgDrawer::new();
        let element = circuit::Element::C("1");
        element.draw(element.layout_size(), Context::default().rotate(), &mut drawer);
        svg::save("test-output/draw_single_capacitor_rotated.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_single_inductor_rotated() {
        let mut drawer = SvgDrawer::new();
        let element = circuit::Element::L("1");
        element.draw(element.layout_size(), Context::default().rotate(), &mut drawer);
        svg::save("test-output/draw_single_inductor_rotated.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_two_series_resistors() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1+R2)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_two_series_resistors.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_two_series_resistors_rotated() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1+R2)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default().rotate(), &mut drawer);
        svg::save("test-output/draw_two_series_resistors_rotated.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_two_parallel_resistors() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1||R2)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_two_parallel_resistors.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_parallel_series_combi() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1||(R2+R3))").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_parallel_series_combi.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_parallel_series_combi2() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1+R2||R3)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_parallel_series_combi2.svg", &drawer.finalize()).unwrap();
    }

    #[test]
    fn test_draw_parallel_series_combi3() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("((R1+R2||R3)+R4)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_parallel_series_combi3.svg", &drawer.finalize()).unwrap();
    }
}
