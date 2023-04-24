use crate::lex::{TokenKind, TokenStream};

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Sc,
    Inc,
    Dec,
    Group(Vec<Instruction>),
    Repeat(Box<Instruction>, u32),
}

/// Possibly adds a repetition number to the passed instruction.
fn maybe_parse_count(ts: &mut TokenStream<'_>, inst: Instruction) -> Instruction {
    match ts.peek().map(|x| x.kind()) {
        Some(TokenKind::Number(n)) => {
            ts.next();
            Instruction::Repeat(inst.into(), n)
        }
        _ => inst,
    }
}

/// Parses as many comma-separated instructions into a group as possible.
/// Returns the group when it can't parse another instruction into the group.
/// Errors if it cannot parse at least one instruction.
fn parse_group(ts: &mut TokenStream<'_>) -> Result<Instruction, (usize, usize)> {
    let mut insts = Vec::new();

    loop {
        insts.push(parse_inst(ts)?);

        let peeked = ts.peek();

        match peeked.map(|x| x.kind()) {
            Some(TokenKind::Comma) => ts.next(),
            _ => return Ok(Instruction::Group(insts)),
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
}
