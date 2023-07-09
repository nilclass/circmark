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
    fn open(&mut self, label: &str, position: Position, size: Size, rotate: bool);
    fn wire(&mut self, a: Position, b: Position);
    fn junction(&mut self, position: Position);
}

impl Draw for circuit::Element<'_> {
    fn draw<D: Drawer>(&self, size: Size, ctx: Context, drawer: &mut D) {
        match self {
            circuit::Element::R(label) => drawer.resistor(label, ctx.position, size, ctx.rotate),
            circuit::Element::C(label) => drawer.capacitor(label, ctx.position, size, ctx.rotate),
            circuit::Element::L(label) => drawer.inductor(label, ctx.position, size, ctx.rotate),
            circuit::Element::V(label) => drawer.voltage_source(label, ctx.position, size, ctx.rotate),
            circuit::Element::Open => drawer.open("", ctx.position, size, ctx.rotate),
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
                left.draw(left_size, ctx.translate(-left_size.0 / 2, 0), drawer);
                right.draw(right_size, ctx.translate(right_size.0 / 2, 0), drawer);
            }
            circuit::SubCircuitGroup::Parallel(top, bottom) => {
                let top_size = top.layout_size();
                let bottom_size = bottom.layout_size();
                let height_requested = top_size.1 + bottom_size.1;
                let width = top_size.0.max(bottom_size.0);
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
