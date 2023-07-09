use svg::node::element::{Path, Rectangle, Group, Text, Circle, path::Data};
use crate::layout::{self, Size, Position};

pub struct SvgDrawer {
    document: Option<svg::Document>,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl SvgDrawer {
    pub fn new() -> Self {
        Self {
            document: Some(svg::Document::new()),
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
        self.document.unwrap()
            .set("viewBox", format!("0 0 {} {}", w, h))
            .set("transform", format!("translate({},{}", w/2, h/2))
    }
}

impl SvgDrawer {
    fn add<N: svg::Node>(&mut self, node: N) {
        self.document = Some(self.document.take().unwrap().add(node));
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
                .add(line2),
            position,
            rotate
        ));
    }

    fn capacitor(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        todo!()
    }

    fn inductor(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        todo!()
    }

    fn voltage_source(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
        todo!()
    }

    fn open(&mut self, label: &str, position: layout::Position, size: layout::Size, rotate: bool) {
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
        self.add(self.transform(Group::new().add(circle1).add(circle2), position, rotate))
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
        svg::save("test-output/draw_single_resistor.svg", &drawer.finalize());
    }

    #[test]
    fn test_draw_single_resistor_rotated() {
        let mut drawer = SvgDrawer::new();
        let element = circuit::Element::R("1");
        element.draw(element.layout_size(), Context::default().rotate(), &mut drawer);
        svg::save("test-output/draw_single_resistor_rotated.svg", &drawer.finalize());
    }

    #[test]
    fn test_draw_two_series_resistors() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1+R2)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_two_series_resistors.svg", &drawer.finalize());
    }

    #[test]
    fn test_draw_two_series_resistors_rotated() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1+R2)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default().rotate(), &mut drawer);
        svg::save("test-output/draw_two_series_resistors_rotated.svg", &drawer.finalize());
    }

    #[test]
    fn test_draw_two_parallel_resistors() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1||R2)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_two_parallel_resistors.svg", &drawer.finalize());
    }

    #[test]
    fn test_draw_parallel_series_combi() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1||(R2+R3))").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_parallel_series_combi.svg", &drawer.finalize());
    }

    #[test]
    fn test_draw_parallel_series_combi2() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("(R1+R2||R3)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_parallel_series_combi2.svg", &drawer.finalize());
    }

    #[test]
    fn test_draw_parallel_series_combi3() {
        let mut drawer = SvgDrawer::new();
        let circuit = circuit::sub_circuit("((R1+R2||R3)+R4)").unwrap().1;
        circuit.draw(circuit.layout_size(), Context::default(), &mut drawer);
        svg::save("test-output/draw_parallel_series_combi3.svg", &drawer.finalize());
    }
}
