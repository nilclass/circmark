use nom::{
    IResult,
    multi::many1,
    branch::alt,
    combinator::map,
    sequence::{preceded, delimited, separated_pair},
    bytes::complete::tag,
    character::complete::alphanumeric1,
};

#[derive(PartialEq, Debug)]
pub struct Twoport<'a> {
    links: Vec<TwoportLink<'a>>,
}

#[derive(PartialEq, Debug)]
pub enum TwoportLink<'a> {
    Series(SubCircuit<'a>),
    Shunt(SubCircuit<'a>),
}

#[derive(PartialEq, Debug)]
pub enum SubCircuit<'a> {
    Element(Element<'a>),
    Group(Box<SubCircuitGroup<'a>>),
}

#[derive(PartialEq, Debug)]
pub enum SubCircuitGroup<'a> {
    Single(SubCircuit<'a>),
    Series(SubCircuit<'a>, SubCircuit<'a>),
    Parallel(SubCircuit<'a>, SubCircuit<'a>),
}

#[derive(PartialEq, Debug)]
pub enum Element<'a> {
    R(&'a str),
    C(&'a str),
    V(&'a str),
    L(&'a str),
    Open,
}

impl<'a> Into<SubCircuit<'a>> for SubCircuitGroup<'a> {
    fn into(self) -> SubCircuit<'a> {
        match self {
            SubCircuitGroup::Single(circuit) => circuit,
            _ => SubCircuit::Group(Box::new(self))
        }
    }
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
        map(element, SubCircuit::Element),
        map(delimited(tag("("), sub_circuit_series, tag(")")), |group| SubCircuit::Group(Box::new(group))),
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
