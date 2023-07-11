use nom::{
    IResult,
    multi::many1,
    branch::alt,
    combinator::map,
    sequence::{preceded, delimited, separated_pair},
    bytes::complete::tag,
    character::complete::alphanumeric1,
};

/// A twoport is an arrangement of series and shunt elements in a signal path.
///
/// For example a voltage divider would be a twoport with three links:
/// - a shunt voltage source
/// - series resistance R1
/// - shunt resistance R2
#[derive(PartialEq, Debug)]
pub struct Twoport<'a> {
    pub links: Vec<TwoportLink<'a>>,
}

#[derive(PartialEq, Debug)]
pub enum TwoportLink<'a> {
    Series(SubCircuit<'a>),
    Shunt(SubCircuit<'a>),
}

/// A sub-circuit consists of either an element, or any series/parallel arrangement of elements.
///
/// Sub-circuits have two legs, just like an element.
#[derive(PartialEq, Debug)]
pub enum SubCircuit<'a> {
    /// Single element, e.g. `R1`
    Element(Element<'a>),
    /// Multiple elements, e.g. `(R1||R2)`
    Group(Box<SubCircuitGroup<'a>>),
}

/// Represents an arrangement of a group of sub-circuits
#[derive(PartialEq, Debug)]
pub enum SubCircuitGroup<'a> {
    /// A single subcircuit
    Single(SubCircuit<'a>),
    /// Two sub-circuits in series
    Series(SubCircuit<'a>, SubCircuit<'a>),
    /// Two sub-circuits in parallel
    Parallel(SubCircuit<'a>, SubCircuit<'a>),
}

/// A single circuit element
#[derive(PartialEq, Debug)]
pub enum Element<'a> {
    /// Resistance
    R(&'a str),
    /// Capacitance
    C(&'a str),
    /// Voltage source
    V(&'a str),
    /// Inductance
    L(&'a str),
    /// Open circuit
    Open,
}

/// A circmark document.
///
/// Currently either two-ended circuit (parallel/series arrangement), or a twoport network.
#[derive(PartialEq, Debug)]
pub enum Document<'a> {
    Circuit(SubCircuit<'a>),
    Twoport(Twoport<'a>),
}

impl Element<'_> {
    pub fn label(&self) -> String {
        match self {
            Element::R(id) => format!("R{id}"),
            Element::C(id) => format!("C{id}"),
            Element::V(id) => format!("V{id}"),
            Element::L(id) => format!("L{id}"),
            Element::Open => format!(""),
        }
    }
}

impl<'a> Into<SubCircuit<'a>> for SubCircuitGroup<'a> {
    fn into(self) -> SubCircuit<'a> {
        match self {
            SubCircuitGroup::Single(circuit) => circuit,
            _ => SubCircuit::Group(Box::new(self))
        }
    }
}

pub fn document(input: &str) -> IResult<&str, Document<'_>> {
    alt((
        map(preceded(tag("@twoport:"), twoport), Document::Twoport),
        map(sub_circuit, Document::Circuit),
    ))(input)
}

pub fn twoport(input: &str) -> IResult<&str, Twoport<'_>> {
    map(many1(twoport_link), |links| Twoport { links })(input)
}

pub fn twoport_link(input: &str) -> IResult<&str, TwoportLink<'_>> {
    alt((
        map(preceded(tag("-"), sub_circuit), TwoportLink::Series),
        map(preceded(tag("|"), sub_circuit), TwoportLink::Shunt),
    ))(input)
}

pub fn element(input: &str) -> IResult<&str, Element<'_>> {
    alt((
        map(preceded(tag("R"), alphanumeric1), Element::R),
        map(preceded(tag("C"), alphanumeric1), Element::C),
        map(preceded(tag("V"), alphanumeric1), Element::V),
        map(preceded(tag("L"), alphanumeric1), Element::L),
        map(tag("O"), |_| Element::Open),
    ))(input)
}

pub fn sub_circuit(input: &str) -> IResult<&str, SubCircuit<'_>> {
    alt((
        map(delimited(tag("("), sub_circuit_series, tag(")")), |group| group.into()),
        map(element, SubCircuit::Element)
    ))(input)
}

pub fn sub_circuit_series(input: &str) -> IResult<&str, SubCircuitGroup<'_>> {
    alt((
        map(separated_pair(sub_circuit_parallel, tag("+"), sub_circuit_parallel), |(left, right)| SubCircuitGroup::Series(left.into(), right.into())),
        sub_circuit_parallel
    ))(input)
}

pub fn sub_circuit_parallel(input: &str) -> IResult<&str, SubCircuitGroup<'_>> {
    alt((
        map(separated_pair(sub_circuit, tag("||"), sub_circuit), |(left, right)| SubCircuitGroup::Parallel(left, right)),
        map(sub_circuit, SubCircuitGroup::Single),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element() {
        assert_eq!(element("R1").unwrap().1, Element::R("1"));
        assert_eq!(element("C2").unwrap().1, Element::C("2"));
        assert_eq!(element("V3").unwrap().1, Element::V("3"));
        assert_eq!(element("L4").unwrap().1, Element::L("4"));
        assert_eq!(element("O").unwrap().1, Element::Open);
        assert_eq!(element("Req").unwrap().1, Element::R("eq"));
    }

    #[test]
    fn test_sub_circuit() {
        assert_eq!(sub_circuit("R1").unwrap().1, SubCircuit::Element(Element::R("1")));
        assert_eq!(sub_circuit("(R1+R2)").unwrap().1, SubCircuit::Group(Box::new(
            SubCircuitGroup::Series(
                SubCircuit::Element(Element::R("1")),
                SubCircuit::Element(Element::R("2"))
            )
        )));
        assert_eq!(sub_circuit("(R1+R2||R3)").unwrap().1, SubCircuit::Group(Box::new(
            SubCircuitGroup::Series(
                SubCircuit::Element(Element::R("1")),
                SubCircuit::Group(Box::new(
                    SubCircuitGroup::Parallel(
                        SubCircuit::Element(Element::R("2")),
                        SubCircuit::Element(Element::R("3")),
                    )
                ))
            )
        )));
        assert_eq!(sub_circuit("(R1+(R2||R3))").unwrap().1, SubCircuit::Group(Box::new(
            SubCircuitGroup::Series(
                SubCircuit::Element(Element::R("1")),
                SubCircuit::Group(Box::new(
                    SubCircuitGroup::Parallel(
                        SubCircuit::Element(Element::R("2")),
                        SubCircuit::Element(Element::R("3")),
                    )
                ))
            )
        )));
        assert_eq!(sub_circuit("((R1+R2)||R3)").unwrap().1, SubCircuit::Group(Box::new(
            SubCircuitGroup::Parallel(
                SubCircuit::Group(Box::new(
                    SubCircuitGroup::Series(
                        SubCircuit::Element(Element::R("1")),
                        SubCircuit::Element(Element::R("2")),
                    )
                )),
                SubCircuit::Element(Element::R("3")),
            )
        )));
    }

    #[test]
    fn test_twoport() {
        assert_eq!(twoport("|O-((L1+R1)||C1)|O").unwrap().1, Twoport {
            links: vec![
                TwoportLink::Shunt(SubCircuit::Element(Element::Open)),
                TwoportLink::Series(SubCircuit::Group(Box::new(SubCircuitGroup::Parallel(
                    SubCircuit::Group(Box::new(SubCircuitGroup::Series(
                        SubCircuit::Element(Element::L("1")),
                        SubCircuit::Element(Element::R("1")),
                    ))),
                    SubCircuit::Element(Element::C("1"))
                )))),
                TwoportLink::Shunt(SubCircuit::Element(Element::Open)),
            ],
        });
    }
}
