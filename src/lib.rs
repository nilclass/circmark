use nom::{
    IResult,
    multi::many1,
    branch::alt,
    combinator::map,
    sequence::{preceded, delimited},
    bytes::complete::tag,
    character::complete::alphanumeric1,
};

pub mod prelude {
    pub use super::{CircmarkChain, ChainNode, Element};
}

#[derive(PartialEq, Debug)]
pub struct CircmarkChain<'a>(pub Vec<ChainNode<'a>>);

impl CircmarkChain<'_> {
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
    Sub(CircmarkChain<'a>),
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

pub fn circmark_chain(input: &str) -> IResult<&str, CircmarkChain> {
    map(many1(chain_node), |nodes| CircmarkChain(nodes))(input)
}

pub fn chain_node(input: &str) -> IResult<&str, ChainNode> {
    alt((
        map(preceded(tag("-"), element), ChainNode::Series),
        map(preceded(tag("|"), element), ChainNode::Shunt),
    ))(input)
}

pub fn element(input: &str) -> IResult<&str, Element<'_>> {
    alt((
        map(preceded(tag("R"), alphanumeric1), Element::R),
        map(preceded(tag("C"), alphanumeric1), Element::C),
        map(preceded(tag("V"), alphanumeric1), Element::V),
        map(preceded(tag("L"), alphanumeric1), Element::L),
        map(tag("O"), |_| Element::Open),
        map(delimited(tag("("), circmark_chain, tag(")")), Element::Sub),
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
            circ_mark_chain("-R1-C1").unwrap().1,
            CircMarkChain(vec![
                ChainNode::Series(Element::R("1")),
                ChainNode::Series(Element::C("1")),
            ])
        );
    }

    #[test]
    fn test_shunt_chain() {
        assert_eq!(
            circ_mark_chain("|V1|RL").unwrap().1,
            CircMarkChain(vec![
                ChainNode::Shunt(Element::V("1")),
                ChainNode::Shunt(Element::R("L")),
            ])
        );
    }

    #[test]
    fn test_mixed_chain() {
        assert_eq!(
            circ_mark_chain("|V1|RL").unwrap().1,
            CircMarkChain(vec![
                ChainNode::Shunt(Element::V("1")),
                ChainNode::Shunt(Element::R("L")),
            ])
        );
    }

    #[test]
    fn test_nested_shunt_series() {
        assert_eq!(
            circ_mark_chain("|V1|(-C1-L1)").unwrap().1,
            CircMarkChain(vec![
                ChainNode::Shunt(Element::V("1")),
                ChainNode::Shunt(Element::Sub(CircMarkChain(vec![
                    ChainNode::Series(Element::C("1")),
                    ChainNode::Series(Element::L("1")),
                ]))),
            ])
        );
    }
}
