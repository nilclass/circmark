use nom::{
    IResult,
    multi::many1,
    branch::alt,
    combinator::map,
    sequence::{preceded, delimited},
    bytes::complete::tag,
    character::complete::alphanumeric1,
};


/// At "toplevel", there are series and shunt elements, prefixed by `-` and `|` respectively.
/// Elements can consist of either:
/// - Simple lumped elements (Rn, Cn, Vn, Ln, ...)
/// - Series or parallel combinations of those elements
/// - Open circuits
///
/// Example:
///   |O-(L1||C1)|O
/// describes a twoport network that is open on both sides, with a parallel LC tank circuit in the signal path.
///
/// twoport: (series | shunt)*
/// series: "-" subcircuit
/// shunt: "|" subcircuit
/// subcircuit: element | "(" subcircuit-series ")"
/// subcircuit-series = subcircuit-parallel "+" subcircuit-parallel | subcircuit-parallel
/// subcircuit-parallel: subcircuit "||" subcircuit | subcircuit

#[derive(PartialEq, Debug)]
pub struct Chain<'a>(pub Vec<ChainNode<'a>>);

impl Chain<'_> {
    pub fn nodes(&self) -> &Vec<ChainNode> {
        &self.0
    }
}

#[derive(PartialEq, Debug)]
pub enum ChainNode<'a> {
    Series(Element<'a>),
    Shunt(Element<'a>),
}

#[derive(PartialEq, Debug)]
pub enum Element<'a> {
    R(&'a str),
    C(&'a str),
    V(&'a str),
    L(&'a str),
    Open,
    Sub(Chain<'a>),
}

impl std::fmt::Display for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::R(id) => write!(f, "R{id}"),
            Element::C(id) => write!(f, "C{id}"),
            Element::V(id) => write!(f, "V{id}"),
            Element::L(id) => write!(f, "L{id}"),
            Element::Open => write!(f, ""),
            _ => panic!("Display not implemented for sub-chain"),
        }
    }
}

pub fn chain(input: &str) -> IResult<&str, Chain> {
    map(many1(chain_node), |nodes| Chain(nodes))(input)
}

pub fn chain_node(input: &str) -> IResult<&str, ChainNode> {
    alt((series_chain_node, shunt_chain_node))(input)
}

fn series_chain_node(input: &str) -> IResult<&str, ChainNode> {
    map(preceded(tag("-"), element), ChainNode::Series)(input)
}

fn shunt_chain_node(input: &str) -> IResult<&str, ChainNode> {
    map(preceded(tag("|"), element), ChainNode::Shunt)(input)
}

fn sub_chain(input: &str) -> IResult<&str, Chain> {
    map(many1(sub_chain_node), |nodes| Chain(nodes))(input)
}

fn sub_chain_node(input: &str) -> IResult<&str, ChainNode> {
    if nom::combinator::peek(shunt_chain_node)(input).is_ok() {
        return Err(nom::Err::Failure(nom::error::Error::new("shunt within sub-chains is not supported", nom::error::ErrorKind::Fail)))
    }
    series_chain_node(input)
}

pub fn element(input: &str) -> IResult<&str, Element<'_>> {
    alt((
        map(preceded(tag("R"), alphanumeric1), Element::R),
        map(preceded(tag("C"), alphanumeric1), Element::C),
        map(preceded(tag("V"), alphanumeric1), Element::V),
        map(preceded(tag("L"), alphanumeric1), Element::L),
        map(tag("O"), |_| Element::Open),
        map(delimited(tag("("), sub_chain, tag(")")), Element::Sub),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_elements() {
        assert_eq!(element("R1").unwrap().1, Element::R("1"));
        assert_eq!(element("RL").unwrap().1, Element::R("L"));
        assert_eq!(element("Rs").unwrap().1, Element::R("s"));
        assert_eq!(element("R123something789").unwrap().1, Element::R("123something789")); 
        assert_eq!(element("L7").unwrap().1, Element::L("7"));
        assert_eq!(element("C9").unwrap().1, Element::C("9"));
        assert_eq!(element("V0").unwrap().1, Element::V("0"));
    }

    #[test]
    fn test_series_chain() {
        assert_eq!(
            chain("-R1-C1").unwrap().1,
            Chain(vec![
                ChainNode::Series(Element::R("1")),
                ChainNode::Series(Element::C("1")),
            ])
        );
    }

    #[test]
    fn test_shunt_chain() {
        assert_eq!(
            chain("|V1|RL").unwrap().1,
            Chain(vec![
                ChainNode::Shunt(Element::V("1")),
                ChainNode::Shunt(Element::R("L")),
            ])
        );
    }

    #[test]
    fn test_mixed_chain() {
        assert_eq!(
            chain("|V1|RL").unwrap().1,
            Chain(vec![
                ChainNode::Shunt(Element::V("1")),
                ChainNode::Shunt(Element::R("L")),
            ])
        );
    }

    #[test]
    fn test_nested_shunt_series() {
        assert_eq!(
            chain("|V1|(-C1-L1)").unwrap().1,
            Chain(vec![
                ChainNode::Shunt(Element::V("1")),
                ChainNode::Shunt(Element::Sub(Chain(vec![
                    ChainNode::Series(Element::C("1")),
                    ChainNode::Series(Element::L("1")),
                ]))),
            ])
        );
    }

    #[test]
    fn test_nested_shunt_shunt_not_allowed() {
        let nom::Err::Failure(err) = chain("|V1|(-C1|L1)").unwrap_err() else { panic!("Expected failure") };
        assert_eq!("shunt within sub-chains is not supported", err.input);
    }
}
