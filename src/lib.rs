mod lex;
mod lint;
mod parse;
mod pretty_print;

pub use lint::{lint_rounds, Lint};
pub use pretty_print::pretty_format;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Ch,
    Sc,
    Fpsc,
    Bpsc,
    Blsc,
    Inc,
    Flinc,
    Blinc,
    Dec,
    /// Do the given instruction into a magic ring
    IntoMagicRing(Box<Instruction>),
    Group(Vec<Instruction>),
    Repeat(Box<Instruction>, u32),
    Comment(String),
}

impl Instruction {
    /// How many stitches this instruction consumes.
    ///
    /// Example:
    /// ```
    /// # use crochet::Instruction;
    /// assert_eq!(Instruction::Inc.input_count(), 1);
    /// assert_eq!(Instruction::Dec.input_count(), 2);
    /// ```
    pub fn input_count(&self) -> u32 {
        use Instruction::*;

        match self {
            Ch => 0,
            Sc | Fpsc | Bpsc | Blsc => 1,
            Inc | Flinc | Blinc => 1,
            Dec => 2,
            IntoMagicRing(_) => 0,
            Group(insts) => insts.iter().map(Self::input_count).sum(),
            Repeat(inst, times) => inst.input_count() * times,
            Comment(_) => 0,
        }
    }

    /// How many stitches this instruction creates.
    ///
    /// Example:
    /// ```
    /// # use crochet::Instruction;
    /// assert_eq!(Instruction::Sc.output_count(), 1);
    /// assert_eq!(Instruction::Inc.output_count(), 2);
    /// ```
    pub fn output_count(&self) -> u32 {
        use Instruction::*;

        match self {
            Ch => 1,
            Sc | Fpsc | Bpsc | Blsc => 1,
            Inc | Flinc | Blinc => 2,
            Dec => 1,
            IntoMagicRing(i) => i.output_count(),
            Group(insts) => insts.iter().map(Self::output_count).sum(),
            Repeat(inst, times) => inst.output_count() * times,
            Comment(_) => 0,
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::ops::Deref;
        use Instruction::*;

        match self {
            Ch => write!(f, "ch"),
            Sc => write!(f, "sc"),
            Fpsc => write!(f, "fpsc"),
            Bpsc => write!(f, "bpsc"),
            Blsc => write!(f, "blsc"),
            Inc => write!(f, "inc"),
            Flinc => write!(f, "flinc"),
            Blinc => write!(f, "blinc"),
            Dec => write!(f, "dec"),
            // group has "in mr" suffix, needs brackets
            IntoMagicRing(g) if matches!(g.deref(), Group(_)) => write!(f, "[{g}] in mr"),
            IntoMagicRing(i) => write!(f, "{i} in mr"),
            // group has repeat suffix, needs brackets
            Repeat(g, times) if matches!(g.deref(), Group(_)) => write!(f, "[{g}] {times}"),
            Repeat(i, times) => write!(f, "{i} {times}"),
            // non-suffixed group doesn't need brackets
            Group(g) => {
                if !g.is_empty() {
                    write!(f, "{}", g[0])?;
                }

                for i in g.iter().skip(1) {
                    write!(f, ", {i}")?;
                }

                Ok(())
            }
            Comment(s) => write!(f, "% {s} %"),
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

    /// assert source->deserialize->serialize is the equal to `displayed`
    fn assert_derser(source: &str, displayed: &str) {
        assert_eq!(
            parse_rounds(source)
                .unwrap()
                .iter()
                .map(ToString::to_string)
                .collect::<String>(),
            displayed
        );
    }

    #[test]
    fn test_instruction_display() {
        // these sources have an identical Display as their original source
        let sources = [
            "sc 4 in mr, inc, [sc, % hi im a comment %, inc] 2",
            "% hi again %, sc, inc, sc 2\n[inc, sc] 3",
            "[sc, inc 2] in mr",
        ];

        for s in sources {
            let rounds = parse_rounds(s).unwrap();
            let rounds = rounds.iter().map(|x| format!("\n{x}"));

            let s2 = rounds.collect::<String>();
            assert_eq!(&s2[1..], s);
        }

        assert_derser("sc 1", "sc 1");
        assert_derser("[ch 1] 1", "[ch 1] 1");
        assert_derser("[sc 3 in mr]", "sc 3 in mr");
        assert_derser("[sc 6] in mr", "[sc 6] in mr");
    }

    #[test]
    fn test_unexpected_at_end_of_input() {
        assert_eq!(crate::parse_rounds("sc 3, % foobar"), Err((1, 7)));
        assert_eq!(crate::parse_rounds("% foobar"), Err((1, 1)));
    }
}
