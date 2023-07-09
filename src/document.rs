use nom::{
    IResult,
    multi::many1,
    combinator::{map, map_res},
    sequence::{preceded, terminated, delimited},
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0, multispace1},
};

use crate::twoport;

#[derive(Debug, PartialEq)]
pub struct Document<'a> {
    pub sections: Vec<Section<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Section<'a> {
    Twoport(twoport::Chain<'a>),
}

#[derive(Debug, PartialEq)]
enum SectionKind {
    Twoport,
}

pub fn document(input: &str) -> IResult<&str, Document> {
    map(many1(delimited(multispace0, section, multispace0)), |sections| Document { sections })(input)
}

fn section(input: &str) -> IResult<&str, Section> {
    let (input, kind) = terminated(section_kind, multispace1)(input)?;
    match kind {
        SectionKind::Twoport => map(twoport::chain, |chain| Section::Twoport(chain))(input),
    }
}

fn section_kind(input: &str) -> IResult<&str, SectionKind> {
    map_res(
        preceded(tag("@"), alphanumeric1),
        |kind| {
            match kind {
                "twoport" => Ok(SectionKind::Twoport),
                other => Err(format!("Unknown section kind: {other:?}")),
            }
        }
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_kind() {
        assert_eq!(section_kind("@twoport"), Ok(("", SectionKind::Twoport)));
        assert!(section_kind("@something").is_err());
    }

    #[test]
    fn test_section() {
        assert_eq!(section("@twoport\n|V1-R1|O"), Ok(("", Section::Twoport(twoport::Chain(vec![
            twoport::ChainNode::Shunt(twoport::Element::V("1")),
            twoport::ChainNode::Series(twoport::Element::R("1")),
            twoport::ChainNode::Shunt(twoport::Element::Open),
        ])))));
    }

    #[test]
    fn test_document() {
        assert_eq!(document("@twoport |V1\n@twoport |V2"), Ok(("", Document {
            sections: vec![
                Section::Twoport(twoport::Chain(vec![twoport::ChainNode::Shunt(twoport::Element::V("1"))])),
                Section::Twoport(twoport::Chain(vec![twoport::ChainNode::Shunt(twoport::Element::V("2"))])),
            ],
        })));
    }
}
