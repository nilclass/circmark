use nom::{
    IResult,
    multi::many1,
    branch::alt,
    combinator::map,
    sequence::{preceded, delimited, separated_pair},
    bytes::complete::tag,
    character::complete::alphanumeric1,
    error::{context, ContextError, ParseError, VerboseError},
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
    /// Impedance
    Z(&'a str),
    /// Current source
    I(&'a str),
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
            Element::Z(id) => format!("Z{id}"),
            Element::I(id) => format!("I{id}"),
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

pub fn document<'a>(input: &'a str) -> IResult<&'a str, Document<'a>, VerboseError<&str>> {
    match input.chars().nth(0) {
        Some('|' | '-') => map(twoport, Document::Twoport)(input),
        _ => map(sub_circuit, Document::Circuit)(input),
    }
}

pub fn twoport<'a, E: ParseError<&'a str> + ContextError<&'a str>>(input: &'a str) -> IResult<&'a str, Twoport<'a>, E> {
    context("twoport", map(many1(twoport_link), |links| Twoport { links }))(input)
}

pub fn twoport_link<'a, E: ParseError<&'a str> + ContextError<&'a str>>(input: &'a str) -> IResult<&'a str, TwoportLink<'a>, E> {
    alt((
        map(preceded(tag("-"), sub_circuit), TwoportLink::Series),
        map(preceded(tag("|"), sub_circuit), TwoportLink::Shunt),
    ))(input)
}

pub fn element<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Element<'a>, E> {
    alt((
        map(preceded(tag("R"), alphanumeric1), Element::R),
        map(preceded(tag("C"), alphanumeric1), Element::C),
        map(preceded(tag("V"), alphanumeric1), Element::V),
        map(preceded(tag("L"), alphanumeric1), Element::L),
        map(preceded(tag("Z"), alphanumeric1), Element::Z),
        map(preceded(tag("I"), alphanumeric1), Element::I),
        map(tag("O"), |_| Element::Open),
    ))(input)
}

pub fn sub_circuit<'a, E: ParseError<&'a str> + ContextError<&'a str>>(input: &'a str) -> IResult<&'a str, SubCircuit<'a>, E> {
    alt((
        context("sub_circuit-group", map(delimited(tag("("), sub_circuit_series, tag(")")), |group| group.into())),
        context("sub_circuit-element", map(element, SubCircuit::Element)),
    ))(input)
}

pub fn sub_circuit_series<'a, E: ParseError<&'a str> + ContextError<&'a str>>(input: &'a str) -> IResult<&'a str, SubCircuitGroup<'a>, E> {
    alt((
        map(separated_pair(sub_circuit_parallel, tag("+"), sub_circuit_series), |(left, right)| SubCircuitGroup::Series(left.into(), right.into())),
        sub_circuit_parallel
    ))(input)
}

pub fn sub_circuit_parallel<'a, E: ParseError<&'a str> + ContextError<&'a str>>(input: &'a str) -> IResult<&'a str, SubCircuitGroup<'a>, E> {
    alt((
        map(separated_pair(sub_circuit, tag("||"), sub_circuit_parallel), |(left, right)| SubCircuitGroup::Parallel(left, right.into())),
        map(sub_circuit, SubCircuitGroup::Single),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    type E = VerboseError<&'static str>;

    fn try_parse<'a, F: Fn(&'a str) -> IResult<&'a str, T, E>, T>(f: F, input: &'a str) -> IResult<&'a str, T, String> {
        f(input).map_err(|e| {
            match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => {
                    let msg = nom::error::convert_error(input, e);
                    eprintln!("{msg}");
                    nom::Err::Error(msg)
                },
                _ => nom::Err::Error(format!("Other err: {e:?}")),
            }
        })
    }

    #[test]
    fn test_element() {
        assert_eq!(element::<E>("R1").unwrap().1, Element::R("1"));
        assert_eq!(element::<E>("C2").unwrap().1, Element::C("2"));
        assert_eq!(element::<E>("V3").unwrap().1, Element::V("3"));
        assert_eq!(element::<E>("L4").unwrap().1, Element::L("4"));
        assert_eq!(element::<E>("Zth1").unwrap().1, Element::Z("th1"));
        assert_eq!(element::<E>("Ino").unwrap().1, Element::I("no"));
        assert_eq!(element::<E>("O").unwrap().1, Element::Open);
        assert_eq!(element::<E>("Req").unwrap().1, Element::R("eq"));
    }

    #[test]
    fn test_sub_circuit() {
        assert_eq!(sub_circuit::<E>("R1").unwrap().1, SubCircuit::Element(Element::R("1")));
        assert_eq!(sub_circuit::<E>("(R1+R2)").unwrap().1, SubCircuit::Group(Box::new(
            SubCircuitGroup::Series(
                SubCircuit::Element(Element::R("1")),
                SubCircuit::Element(Element::R("2"))
            )
        )));
        assert_eq!(sub_circuit::<E>("(R1+R2||R3)").unwrap().1, SubCircuit::Group(Box::new(
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
        assert_eq!(sub_circuit::<E>("(R1+(R2||R3))").unwrap().1, SubCircuit::Group(Box::new(
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
        assert_eq!(sub_circuit::<E>("((R1+R2)||R3)").unwrap().1, SubCircuit::Group(Box::new(
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
    fn test_multi_series() {
        assert_eq!(try_parse(sub_circuit, "(R1+R2+R3)").unwrap().1, SubCircuit::Group(Box::new(
            SubCircuitGroup::Series(
                SubCircuit::Element(Element::R("1")),
                SubCircuit::Group(Box::new(
                    SubCircuitGroup::Series(
                        SubCircuit::Element(Element::R("2")),
                        SubCircuit::Element(Element::R("3")),
                    ),
                )),
            )
        )));
    }

    #[test]
    fn test_multi_parallel() {
        assert_eq!(try_parse(sub_circuit, "(R1||R2||R3)").unwrap().1, SubCircuit::Group(Box::new(
            SubCircuitGroup::Parallel(
                SubCircuit::Element(Element::R("1")),
                SubCircuit::Group(Box::new(
                    SubCircuitGroup::Parallel(
                        SubCircuit::Element(Element::R("2")),
                        SubCircuit::Element(Element::R("3")),
                    ),
                )),
            )
        )));
    }

    #[test]
    fn test_twoport() {
        assert_eq!(twoport::<E>("|O-((L1+R1)||C1)|O").unwrap().1, Twoport {
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
