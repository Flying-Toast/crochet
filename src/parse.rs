use crate::lex::{TokenKind, TokenStream};
use crate::Instruction;

/// Possibly adds a repetition number to the passed instruction.
fn maybe_parse_count(ts: &mut TokenStream<'_>, inst: Instruction) -> Instruction {
    match ts.peek_kind() {
        Some(TokenKind::Number(n)) => {
            ts.next();
            Instruction::Repeat(inst.into(), n)
        }
        _ => inst,
    }
}

/// Parses as many comma-separated instructions into a group as possible.
/// Returns the group when it can't parse another instruction into the group.
/// If the group only has one instruction, do not wrap it in a Group.
/// Errors if it cannot parse at least one instruction.
fn parse_group(ts: &mut TokenStream<'_>) -> Result<Instruction, (usize, usize)> {
    let mut insts = Vec::new();

    loop {
        insts.push(parse_inst(ts)?);

        match ts.peek_kind() {
            Some(TokenKind::Comma) => ts.next(),
            _ => {
                if insts.len() == 1 {
                    return Ok(insts.pop().unwrap());
                } else {
                    return Ok(Instruction::Group(insts));
                }
            }
        };
    }
}

/// Errors if `ts` is empty
fn parse_inst(ts: &mut TokenStream<'_>) -> Result<Instruction, (usize, usize)> {
    let next = match ts.next() {
        Some(x) => x,
        None => return Err(ts.current_loc()),
    };

    match next.kind() {
        TokenKind::Sc => Ok(maybe_parse_count(ts, Instruction::Sc)),
        TokenKind::Inc => Ok(maybe_parse_count(ts, Instruction::Inc)),
        TokenKind::Dec => Ok(maybe_parse_count(ts, Instruction::Dec)),
        TokenKind::LBracket => {
            let group = parse_group(ts)?;

            match ts.next() {
                Some(t) if t.kind() == TokenKind::RBracket => Ok(maybe_parse_count(ts, group)),
                Some(unexpected) => Err(unexpected.source_loc()),
                None => Err(ts.current_loc()),
            }
        }
        _ => Err(next.source_loc()),
    }
}

/// Parses a list of rounds.
pub fn parse(ts: &mut TokenStream<'_>) -> Result<Vec<Instruction>, (usize, usize)> {
    while let Some(TokenKind::Newline) = ts.peek_kind() {
        ts.next();
    }

    let mut rounds = Vec::new();

    while ts.peek().is_some() {
        rounds.push(parse_group(ts)?);

        while let Some(TokenKind::Newline) = ts.peek_kind() {
            ts.next();
        }
    }

    Ok(rounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group() {
        let mut ts = crate::lex::tokenize("[sc, inc, dec]");
        let ast = Instruction::Group(vec![Instruction::Sc, Instruction::Inc, Instruction::Dec]);
        assert_eq!(parse_inst(&mut ts), Ok(ast));
    }

    #[test]
    fn test_repeated_group() {
        use Instruction::*;

        let mut ts = crate::lex::tokenize("[inc 2, sc] 3");
        let ast = Repeat(Group(vec![Repeat(Inc.into(), 2), Sc]).into(), 3);
        assert_eq!(parse_inst(&mut ts), Ok(ast));
    }

    #[test]
    fn test_simple_rounds() {
        use Instruction::*;

        let mut ts = crate::lex::tokenize("sc\nsc 2, inc");
        let rounds = vec![Sc, Group(vec![Repeat(Sc.into(), 2), Inc])];
        assert_eq!(parse(&mut ts), Ok(rounds));
    }

    #[test]
    fn test_empty_line_round() {
        use Instruction::*;

        let mut ts = crate::lex::tokenize("\n\n\nsc 2\ninc\n\nsc 123");
        let rounds = vec![Repeat(Sc.into(), 2), Inc, Repeat(Sc.into(), 123)];
        assert_eq!(parse(&mut ts), Ok(rounds));
    }

    #[test]
    fn test_unexpected_token() {
        let mut ts = crate::lex::tokenize("\nsc 2, ]");
        assert_eq!(parse(&mut ts), Err((2, 7)));
    }
}
