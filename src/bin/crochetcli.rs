use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} path/to/pattern.crochet", args[0]);
        return ExitCode::FAILURE;
    }

    let source = match std::fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Can't read `{}`: {e}", args[1]);
            return ExitCode::FAILURE;
        }
    };

    let rounds = match crochet::parse_rounds(&source) {
        Ok(r) => r,
        Err((lineno, col)) => {
            eprintln!("Parse error at {lineno}:{col}");

            let line = source.split("\n").nth(lineno - 1).unwrap();
            let prefix = format!("{lineno} ");

            let mut lpad = String::with_capacity(prefix.len() + 1);
            for _ in 0..prefix.len() {
                lpad.push(' ');
            }
            lpad.push('|');

            eprintln!("{lpad}");
            eprintln!("{prefix}| {line}");

            eprint!("{lpad} ");
            for _ in 1..col {
                eprint!(" ");
            }
            eprintln!("^");

            return ExitCode::FAILURE;
        }
    };

    let lints = crochet::lint_rounds(&rounds);

    if lints.is_empty() {
        println!("{}", crochet::pretty_format(&rounds));

        ExitCode::SUCCESS
    } else {
        for l in lints {
            println!("Lint: {l}");
        }

        ExitCode::FAILURE
    }
}
