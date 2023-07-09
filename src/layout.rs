use crate::circuit;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Size(pub i32, pub i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Position(pub i32, pub i32);

impl Position {
    pub fn zero() -> Self {
        Self(0, 0)
    }
}

pub const ELEMENT_SIZE: Size = Size(200, 60);

pub trait Layout {
    fn layout_size(&self) -> Size;
}

impl Layout for circuit::Element<'_> {
    fn layout_size(&self) -> Size {
        ELEMENT_SIZE
    }
}

impl Layout for circuit::SubCircuitGroup<'_> {
    fn layout_size(&self) -> Size {
        use circuit::SubCircuitGroup::*;
        match self {
            Single(circuit) => circuit.layout_size(),
            Series(left, right) => {
                let (left_size, right_size) = (left.layout_size(), right.layout_size());
                Size(left_size.0 + right_size.0, left_size.1.max(right_size.1))
            }
            Parallel(left, right) => {
                let (left_size, right_size) = (left.layout_size(), right.layout_size());
                Size(left_size.0.max(right_size.0), left_size.1 + right_size.1)
            }
        }
    }
}

impl Layout for circuit::SubCircuit<'_> {
    fn layout_size(&self) -> Size {
        match self {
            circuit::SubCircuit::Element(element) => element.layout_size(),
            circuit::SubCircuit::Group(group) => group.layout_size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element() {
        assert_eq!(circuit::Element::R("1").layout_size(), ELEMENT_SIZE);
    }

    #[test]
    fn test_group_series() {
        assert_eq!(circuit::SubCircuitGroup::Series(
            circuit::SubCircuit::Element(circuit::Element::R("1")),
            circuit::SubCircuit::Element(circuit::Element::R("2")),
        ).layout_size(), Size(ELEMENT_SIZE.0 * 2, ELEMENT_SIZE.1));
    }

    #[test]
    fn test_group_parallel() {
        assert_eq!(circuit::SubCircuitGroup::Parallel(
            circuit::SubCircuit::Element(circuit::Element::R("1")),
            circuit::SubCircuit::Element(circuit::Element::R("2")),
        ).layout_size(), Size(ELEMENT_SIZE.0, ELEMENT_SIZE.1 * 2));
    }

    #[test]
    fn test_group_parallel_with_series_element() {
        assert_eq!(circuit::SubCircuitGroup::Parallel(
            circuit::SubCircuit::Element(circuit::Element::R("1")),
            circuit::SubCircuit::Group(Box::new(circuit::SubCircuitGroup::Series(
                circuit::SubCircuit::Element(circuit::Element::R("2")),
                circuit::SubCircuit::Element(circuit::Element::R("3")),
            )))
        ).layout_size(), Size(ELEMENT_SIZE.0 * 2, ELEMENT_SIZE.1 * 2));
    }
}
