use crate::{layout::{Size, Position, Layout}, circuit};

pub mod svg;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Context {
    position: Position,
    rotate: bool,
}

impl Context {
    fn translate(self, x: i32, y: i32) -> Self {
        Self {
            position: if self.rotate {
                Position(self.position.0 + y, self.position.1 + x)
            } else {
                Position(self.position.0 + x, self.position.1 + y)
            },
            ..self
        }
    }

    pub fn rotate(self) -> Self {
        Self {
            rotate: !self.rotate,
            ..self
        }
    }
}

pub trait Draw {
    fn draw<D: Drawer>(&self, size: Size, ctx: Context, drawer: &mut D);
}

pub trait Drawer {
    fn resistor(&mut self, label: &str, position: Position, size: Size, rotate: bool);
    fn capacitor(&mut self, label: &str, position: Position, size: Size, rotate: bool);
    fn inductor(&mut self, label: &str, position: Position, size: Size, rotate: bool);
    fn voltage_source(&mut self, label: &str, position: Position, size: Size, rotate: bool);
    fn current_source(&mut self, label: &str, position: Position, size: Size, rotate: bool);
    fn open(&mut self, label: &str, position: Position, size: Size, rotate: bool);
    fn wire(&mut self, a: Position, b: Position);
    fn junction(&mut self, position: Position);
}

impl Draw for circuit::Element<'_> {
    fn draw<D: Drawer>(&self, size: Size, ctx: Context, drawer: &mut D) {
        match self {
            circuit::Element::R(_) => drawer.resistor(&self.label(), ctx.position, size, ctx.rotate),
            circuit::Element::C(_) => drawer.capacitor(&self.label(), ctx.position, size, ctx.rotate),
            circuit::Element::L(_) => drawer.inductor(&self.label(), ctx.position, size, ctx.rotate),
            circuit::Element::V(_) => drawer.voltage_source(&self.label(), ctx.position, size, ctx.rotate),
            circuit::Element::Z(_) => drawer.resistor(&self.label(), ctx.position, size, ctx.rotate),
            circuit::Element::I(_) => drawer.current_source(&self.label(), ctx.position, size, ctx.rotate),
            circuit::Element::Open => drawer.open(&self.label(), ctx.position, size, ctx.rotate),
        }
    }
}

impl Draw for circuit::SubCircuitGroup<'_> {
    fn draw<D: Drawer>(&self, size: Size, ctx: Context, drawer: &mut D) {
        match self {
            circuit::SubCircuitGroup::Single(circuit) => circuit.draw(size, ctx, drawer),
            circuit::SubCircuitGroup::Series(left, right) => {
                let left_size = left.layout_size();
                let right_size = right.layout_size();
                let width_requested = left_size.0 + right_size.0;
                let height = left_size.1.max(right_size.1);
                let left_size = Size(size.0 * left_size.0 / width_requested, height);
                let right_size = Size(size.0 * right_size.0 / width_requested, height);
                left.draw(left_size, ctx.translate(-size.0 / 2 + left_size.0 / 2, 0), drawer);
                right.draw(right_size, ctx.translate(size.0 / 2 - right_size.0 / 2, 0), drawer);
            }
            circuit::SubCircuitGroup::Parallel(top, bottom) => {
                let end_wire_length = 20;
                let top_size = top.layout_size();
                let bottom_size = bottom.layout_size();
                let height_requested = top_size.1 + bottom_size.1;
                let width = top_size.0.max(bottom_size.0) - 2 * end_wire_length;
                let top_size = Size(width, size.1 * top_size.1 / height_requested);
                let bottom_size = Size(width, size.1 * bottom_size.1 / height_requested);
                top.draw(top_size, ctx.translate(0, -top_size.1 / 2), drawer);
                bottom.draw(bottom_size, ctx.translate(0, bottom_size.1 / 2), drawer);
                drawer.wire(
                    ctx.translate(-width / 2, -top_size.1 / 2).position,
                    ctx.translate(-width / 2, bottom_size.1 / 2).position,
                );
                drawer.wire(
                    ctx.translate(width / 2, -top_size.1 / 2).position,
                    ctx.translate(width / 2, bottom_size.1 / 2).position,
                );
                drawer.junction(ctx.translate(-width / 2, 0).position);
                drawer.junction(ctx.translate(width / 2, 0).position);
                drawer.wire(
                    ctx.translate(-width / 2 - end_wire_length, 0).position,
                    ctx.translate(-width / 2, 0).position,
                );
                drawer.wire(
                    ctx.translate(width / 2 + end_wire_length, 0).position,
                    ctx.translate(width / 2, 0).position,
                );
            }
        }
    }
}

impl Draw for circuit::SubCircuit<'_> {
    fn draw<D: Drawer>(&self, size: Size, ctx: Context, drawer: &mut D) {
        match self {
            circuit::SubCircuit::Element(element) => element.draw(size, ctx, drawer),
            circuit::SubCircuit::Group(group) => group.draw(size, ctx, drawer),
        }
    }
}

impl Draw for circuit::Document<'_> {
    fn draw<D: Drawer>(&self, size: Size, ctx: Context, drawer: &mut D) {
        match self {
            circuit::Document::Circuit(circuit) => circuit.draw(size, ctx, drawer),
            circuit::Document::Twoport(twoport) => twoport.draw(size, ctx, drawer),
        }
    }
}

impl Draw for circuit::Twoport<'_> {
    fn draw<D: Drawer>(&self, size: Size, ctx: Context, drawer: &mut D) {
        let top_line = -size.1 / 2;
        let bottom_line = size.1 / 2;
        let mut offset = -size.0 / 2;
        let mut links = self.links.iter().enumerate().peekable();
        while let Some((i, link)) = links.next() {
            let requested_size = link.layout_size();

            offset += requested_size.0/2;
                
            match link {
                circuit::TwoportLink::Series(circuit) => {
                    circuit.draw(requested_size, ctx.translate(offset, -size.1/2), drawer);
                    drawer.wire(Position(offset - requested_size.0 / 2, bottom_line), Position(offset + requested_size.0 / 2, bottom_line));
                },
                circuit::TwoportLink::Shunt(circuit) => {
                    let left_exists = i != 0;
                    let right_exists = links.peek().is_some();

                    if left_exists {
                        // top wire to the left
                        drawer.wire(Position(offset - requested_size.0/2, top_line), Position(offset, top_line));
                        // bottom wire to the left
                        drawer.wire(Position(offset - requested_size.0/2, bottom_line), Position(offset, bottom_line));
                    }
                    if right_exists {
                        // top wire to the right
                        drawer.wire(Position(offset, top_line), Position(offset + requested_size.0/2, top_line));
                        // bottom wire to the right
                        drawer.wire(Position(offset, bottom_line), Position(offset + requested_size.0/2, bottom_line));
                    }
                    if left_exists && right_exists {
                        drawer.junction(Position(offset, top_line));
                        drawer.junction(Position(offset, bottom_line));
                    }

                    circuit.draw(Size(size.1, requested_size.0), ctx.translate(offset, 0).rotate(), drawer);
                },
            }
            offset += requested_size.0/2;
        }
    }
}
