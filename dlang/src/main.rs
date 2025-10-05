use std::env;
use dlang::parser::Parser;

fn print_ast_for(input: &str) {
    println!("--- Input ---\n{}\n--- AST ---", input);
    let mut parser = Parser::new(input);
    match parser.parse_program() {
        Ok(ast) => println!("{:#?}", ast),
        Err(e) => println!("Parse error: {}", e),
    }
    println!("--------------\n");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // read file (first arg)
        let path = &args[1];
        match std::fs::read_to_string(path) {
            Ok(src) => print_ast_for(&src),
            Err(e) => eprintln!("Failed to read {}: {}", path, e),
        }
        return;
    }

    // default demo snippets
    let samples = vec![
        "var x := 42",
        r#"if x < 10 then print "small" else print "big" end"#,
        "var f := func(x)=>x+1",
        "arr[1] := {x:=2,y:=3}.y",
        "for i in [1,2,3] loop print i end",
        // range not yet implemented in parser as an operator; keep arrays demo
        r#"
        var i := 0
        while i < 3 loop //while cicle
            var j := 0
            while j < 2 loop
                print i, j
                j := j + 1
            end
            i := i + 1
        end
        "#,
        r#"
        var outer := func(x) is
            var inner := func(y) => y + 1
            return inner(x)
        end
        print outer(10)
        "#,
        r#"
        var f := func(x, y) => x * y
        print f(3, 4)
        "#,
        "@ # $", // error case
    ];

    for s in samples { print_ast_for(s); }
}
