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

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::ops::Deref;
        use Instruction::*;

        match self {
            Sc => write!(f, "sc"),
            Inc => write!(f, "inc"),
            Dec => write!(f, "dec"),
            Repeat(g, times) if matches!(g.deref(), Group(_)) => write!(f, "[{}] {}", g, times),
            Repeat(i, times) => write!(f, "{i} {times}"),
            // non-repeated group doesn't need brackets
            Group(g) => {
                if !g.is_empty() {
                    write!(f, "{}", g[0])?;
                }

                for i in g.iter().skip(1) {
                    write!(f, ", {i}")?;
                }

                Ok(())
            }
        }
    }
}

pub fn parse_rounds(source: &str) -> Result<Vec<Instruction>, (usize, usize)> {
    let mut ts = lex::tokenize(source);

    let res = parse::parse(&mut ts);

    if ts.is_empty() {
        res
    } else {
        Err(ts.current_loc())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_display() {
        let sources = [
            "sc 4, inc, [sc, inc] 2",
            "sc, inc, sc 2\n[inc, sc] 3",
        ];

        for s in sources {
            let rounds = parse_rounds(s).unwrap();
            let rounds = rounds.iter().map(|x| format!("\n{x}"));

            let s2 = rounds.collect::<String>();
            assert_eq!(&s2[1..], s);
        }
    }
}
