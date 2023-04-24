mod lex;
mod parse;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Sc,
    Inc,
    Dec,
    Group(Vec<Instruction>),
    Repeat(Box<Instruction>, u32),
}

pub fn parse_rounds(source: &str) -> Result<Vec<Instruction>, (usize, usize)> {
    let mut ts = lex::tokenize(source);

    parse::parse(&mut ts)
}
