use crate::Instruction;

#[derive(Debug, PartialEq, Eq)]
pub enum Lint {
    MismatchedStitchCount {
        /// How many stitches the first round produces
        a_out: u32,
        /// One-based round index
        a_idx: usize,
        /// How many stitches the second round consumes
        b_in: u32,
        /// One-based round index
        b_idx: usize,
    },
    NonzeroFirstRoundInput {
        /// How many stitches the first round actually consumed, when it was exspected to consume 0.
        actual_consumed: u32,
    },
}

fn pluralstitch(n: u32) -> &'static str {
    if n == 1 {
        "stitch"
    } else {
        "stitches"
    }
}

impl std::fmt::Display for Lint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MismatchedStitchCount {
                a_out,
                a_idx,
                b_in,
                b_idx,
            } => {
                let aplural = pluralstitch(*a_out);
                let bplural = pluralstitch(*b_in);

                write!(
                    f,
                    "round {a_idx} produces {a_out} \
                        {aplural} but round {b_idx} \
                        consumes {b_in} {bplural}",
                )
            }
            Self::NonzeroFirstRoundInput { actual_consumed } => {
                let plural = pluralstitch(*actual_consumed);
                write!(
                    f,
                    "round 1 consumes {actual_consumed} {plural} but the first round shouldn't consume any stitches"
                )
            }
        }
    }
}

fn lint_nonzero_first_round_input(rounds: &[Instruction]) -> Option<Lint> {
    let cnt = rounds.get(0)?.input_count();

    if cnt != 0 {
        Some(Lint::NonzeroFirstRoundInput {
            actual_consumed: cnt,
        })
    } else {
        None
    }
}

fn lint_mismatched_stitch_count(rounds: &[Instruction]) -> Vec<Lint> {
    if rounds.len() < 2 {
        return Vec::new();
    }

    let mut ret = Vec::new();

    'outer: for i in 0..rounds.len() - 1 {
        let a_out = rounds[i].output_count();
        if a_out == 0 && rounds[i].input_count() == 0 {
            // skip rounds that have 0 in and 0 out (e.g. a round of just comments)
            continue;
        }

        let mut b_offset = 1;
        // skip 0in0out rounds until we find a suitable b
        let b_in = loop {
            match rounds.get(i + b_offset) {
                Some(possible_b) => {
                    let incount = possible_b.input_count();
                    if incount == 0 && possible_b.output_count() == 0 {
                        b_offset += 1;
                        continue;
                    } else {
                        break incount;
                    }
                }
                // we reached the end of the `rounds` array without finding a suitable 'b' round -
                // all rounds after the a round have 0in+0out, so a is the last real round of the pattern
                None => break 'outer,
            }
        };

        if a_out != b_in {
            ret.push(Lint::MismatchedStitchCount {
                a_out,
                b_in,
                a_idx: i + 1,
                b_idx: i + 2,
            })
        }
    }

    ret
}

pub fn lint_rounds(rounds: &[Instruction]) -> Vec<Lint> {
    let mut lints = lint_mismatched_stitch_count(rounds);

    if let Some(l) = lint_nonzero_first_round_input(rounds) {
        lints.push(l);
    }

    lints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_rounds;

    fn assert_produces_lint(src: &str, lint: &Lint) {
        let rounds = parse_rounds(src).unwrap();
        let lints = lint_rounds(&rounds);

        dbg!(&lints, &lint);

        assert!(lints.contains(lint));
    }

    #[test]
    fn test_lint_nonzero_first_round_input() {
        assert_produces_lint("sc 3", &Lint::NonzeroFirstRoundInput { actual_consumed: 3 });
    }

    #[test]
    fn test_lint_mismatched_stitch_counts() {
        assert_produces_lint(
            "sc 3\n[inc, sc] 2",
            &Lint::MismatchedStitchCount {
                a_out: 3,
                b_in: 4,
                a_idx: 1,
                b_idx: 2,
            },
        );
    }

    #[test]
    fn test_lint_display() {
        let s = format!(
            "{}",
            Lint::MismatchedStitchCount {
                a_out: 1,
                b_in: 3,
                a_idx: 1,
                b_idx: 2,
            }
        );
        assert_eq!(
            &s,
            "round 1 produces 1 stitch but round 2 consumes 3 stitches"
        );

        let s = format!("{}", Lint::NonzeroFirstRoundInput { actual_consumed: 4 });
        assert_eq!(
            &s,
            "round 1 consumes 4 stitches but the first round shouldn't consume any stitches"
        );
    }

    fn no_lints(src: &str) {
        let rounds = parse_rounds(src).unwrap();
        let lints = lint_rounds(&rounds);
        assert!(lints.is_empty());
    }

    #[test]
    fn test_no_false_positives_in_lints() {
        no_lints(
            "
            ch 3
            sc, inc, sc
            [inc, sc] 2
            ",
        );

        no_lints(
            "
            sc 6 in mr
            [sc, inc, sc]2
            [inc, sc] 4
            ",
        );

        no_lints(
            "
            ch 12
            % comment %
            sc 12
            ",
        );

        no_lints(
            "
            sc 10 in mr
            skip 2, sc, skip 2, sc 5
            "
            );
    }
}
