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

impl Instruction {
    /// How many stitches this instruction consumes.
    ///
    /// Example:
    /// ```
    /// # use crochet::Instruction;
    ///
    /// assert_eq!(Instruction::Inc.input_count(), 1);
    /// assert_eq!(Instruction::Dec.input_count(), 2);
    /// ```
    pub fn input_count(&self) -> u32 {
        use Instruction::*;

        match self {
            Sc => 1,
            Inc => 1,
            Dec => 2,
            Group(insts) => insts.iter().map(Self::input_count).sum(),
            Repeat(inst, times) => inst.input_count() * times,
        }
    }

    /// How many stitches this instruction creates.
    ///
    /// Example:
    /// ```
    /// # use crochet::Instruction;
    ///
    /// assert_eq!(Instruction::Sc.output_count(), 1);
    /// assert_eq!(Instruction::Inc.output_count(), 2);
    /// ```
    pub fn output_count(&self) -> u32 {
        use Instruction::*;

        match self {
            Sc => 1,
            Inc => 2,
            Dec => 1,
            Group(insts) => insts.iter().map(Self::output_count).sum(),
            Repeat(inst, times) => inst.output_count() * times,
        }
    }
}

pub fn parse_rounds(source: &str) -> Result<Vec<Instruction>, (usize, usize)> {
    let mut ts = lex::tokenize(source);

    parse::parse(&mut ts)
}
