fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} path/to/pattern.crochet", args[0]);
        return;
    }

    let source = match std::fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Can't read `{}`: {e}", args[1]);
            return;
        }
    };

    let rounds = match crochet::parse_rounds(&source) {
        Ok(r) => r,
        Err((line, col)) => {
            eprintln!("Parse error at {line}:{col}");
            return;
        }
    };

    let lints = crochet::lint_rounds(&rounds);

    if lints.is_empty() {
        println!("{}", crochet::pretty_format(&rounds));
    } else {
        for l in lints {
            println!("Lint: {l}");
        }
    }
}
