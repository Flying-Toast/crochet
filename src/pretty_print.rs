use crate::Instruction;
use std::fmt::Write;

/// Formats rounds into a format suitible for publishing.
///
/// ```rust
/// # use crochet::pretty_format;
/// use crochet::parse_rounds;
///
/// let expected = "Round 1: sc 6 in mr (6)
/// Round 2: inc 6 (12)
/// Round 3: [inc, sc] 6 (18)";
///
/// let src = "
///     sc6in mr
///     inc 6
///     [inc,sc] 6
/// ";
///
/// assert_eq!(pretty_format(&parse_rounds(src).unwrap()), expected);
/// ```
pub fn pretty_format(rounds: &[Instruction]) -> String {
    let mut ret = String::new();

    for (i, round) in rounds.iter().enumerate() {
        write!(ret, "Round {}: {round} ({})\n", i + 1, round.output_count()).expect("writing to a string shouldn't fail... right?");
    }

    // remove trailing newline
    ret.pop();

    ret
}
