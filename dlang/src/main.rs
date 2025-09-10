use std::env;
use dlang::lexer::Lexer;
use dlang::token::Token;

fn print_tokens_for(input: &str) {
    println!("--- Input ---\n{}\n--- Tokens ---", input);
    let mut lexer = Lexer::new(input);
    loop {
        let tok = lexer.next_token();
        println!("{:?}", tok);
        if tok == Token::EOF { break; }
    }
    println!("--------------\n");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // read file (first arg)
        let path = &args[1];
        match std::fs::read_to_string(path) {
            Ok(src) => print_tokens_for(&src),
            Err(e) => eprintln!("Failed to read {}: {}", path, e),
        }
        return;
    }

    // default demo snippets
    let samples = vec![
        "var x := 42;",
        r#"if x < 10 then print "small" else print "big" end"#,
        "var f := func(x)=>x+1",
        "arr[1] := {x:=2,y:=3}.y",
        "for i in [1,2,3] loop print i end",
        "for i in 1..3 loop print i end",
        "@ # $", // error case
    ];

    for s in samples {
        print_tokens_for(s);
    }
}
