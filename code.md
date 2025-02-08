# Folder Structure

```
.
├── .git
├── .gitignore
├── .gitmodules
├── .ruff_cache
├── Cargo.toml
├── README.md
├── book
├── code.md
├── pest_book.md
├── shell.nix
├── src
│   ├── aggregator.rs
│   ├── languages
│   │   ├── go.pest
│   │   ├── go.rs
│   │   ├── js.pest
│   │   ├── js.rs
│   │   ├── mod.rs
│   │   ├── python.pest
│   │   ├── python.rs
│   │   ├── rust.pest
│   │   ├── rust.rs
│   │   ├── ts.pest
│   │   └── ts.rs
│   ├── lib.rs
│   └── main.rs
└── tests
    ├── mod.rs
    ├── python_tests.rs
    └── rust_tests.rs
```

## File: `Cargo.toml`
*(Relative Path: `Cargo.toml`)*

```toml
[package]
name = "todo_extractor"
version = "0.1.0"
edition = "2021"

[dependencies]
env_logger = "0.11.6"
log = "0.4.25"
once_cell = "1.20.2"
pest = { version = "2.7.15", features = ["pretty-print"] }
pest_derive = { version = "2.7.15", features = ["grammar-extras"] }

```

---
## File: `shell.nix`
*(Relative Path: `shell.nix`)*

```nix
{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell rec {
  name = "pre-commit-todo";

  buildInputs = with pkgs; [
    rustc # Rust compiler
    cargo # Rust package manager
    rustfmt # Rust code formatter
    clippy # Rust linter
    gcc # Required for crates needing C compilers
    pkg-config # Helps locate libraries like OpenSSL
    openssl # OpenSSL library for crates like openssl-sys
    git
    git-crypt
    stdenv.cc.cc.lib
    stdenv.cc.cc # jupyter lab needs
    pre-commit
  ];

  # Set the source path for Rust tooling (e.g., rust-analyzer)
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  pre-commit = pkgs.pre-commit;

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

    export RUST_BACKTRACE=1
    export CARGO_HOME=$HOME/.cargo
    export PATH=$CARGO_HOME/bin:$PATH
  '';
}

```

---
## File: `pest_book.md`
*(Relative Path: `pest_book.md`)*

```markdown
# Folder Structure

```
book
├── .git
├── .gitignore
├── LICENSE-APACHE
├── LICENSE-MIT
├── book.toml
├── examples
│   ├── calculator
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── grammar.pest
│   │       └── main.rs
│   ├── csv-tool
│   │   ├── Cargo.toml
│   │   ├── numbers.csv
│   │   └── src
│   │       ├── csv.pest
│   │       └── main.rs
│   ├── ini-parser
│   │   ├── Cargo.toml
│   │   ├── config.ini
│   │   └── src
│   │       ├── ini.pest
│   │       └── main.rs
│   ├── jlang-parser
│   │   ├── Cargo.toml
│   │   ├── example.ijs
│   │   └── src
│   │       ├── j.pest
│   │       └── main.rs
│   ├── json-parser
│   │   ├── Cargo.toml
│   │   ├── data.json
│   │   └── src
│   │       ├── json.pest
│   │       └── main.rs
│   └── pest-calculator
│       ├── Cargo.toml
│       └── src
│           ├── calculator.pest
│           └── main.rs
├── highlight-pest.js
└── src
    ├── SUMMARY.md
    ├── examples
    │   ├── awk.md
    │   ├── calculator.md
    │   ├── csv.md
    │   ├── ini.md
    │   ├── jlang.md
    │   ├── json.md
    │   └── rust
    │       ├── literals.md
    │       ├── setup.md
    │       └── syntax.md
    ├── grammars
    │   ├── built-ins.md
    │   ├── comments.md
    │   ├── grammars.md
    │   ├── peg.md
    │   └── syntax.md
    ├── intro.md
    ├── parser_api.md
    └── precedence.md
```

## File: `highlight-pest.js`
*(Relative Path: `highlight-pest.js`)*

```javascript
// Syntax highlighting for pest PEGs.

// mdBook exposes a minified version of highlight.js, so the language
// definition objects below have abbreviated property names:
//     "b"  => begin
//     "c"  => contains
//     "cN" => className
//     "e"  => end

hljs.registerLanguage("pest", function(hljs) {

    // Basic syntax.
    var comment = {cN: "comment", b: "//", e: /$/};
    var ident = {cN: "title", b: /[_a-zA-Z][_a-z0-9A-Z]*/};
    var special = {b: /COMMENT|WHITESPACE/, cN: "keyword"};

    // Escape sequences within a string or character literal.
    var escape = {b: /\\./};

    // Per highlight.js style, only built-in rules should be highlighted inside
    // a definition.
    var rule = {
        b: /[@_$!]?\{/, e: "}",
        k: {built_in: "ANY SOI EOI PUSH POP PEEK " +
                      "ASCII_ALPHANUMERIC ASCII_DIGIT ASCII_HEX_DIGIT " +
                      "ASCII_NONZERO_DIGIT NEWLINE"},
        c: [comment,
            {cN: "string", b: '"', e: '"', c: [escape]},
            {cN: "string", b: "'", e: "'", c: [escape]}]
    };

    return {
        c: [special, rule, ident, comment]
    };

});

// This file is inserted after the default highlight.js invocation, which tags
// unknown-language blocks with CSS classes but doesn't highlight them.
Array.from(document.querySelectorAll("code.language-pest"))
    .forEach(hljs.highlightBlock);

```

---
## File: `book.toml`
*(Relative Path: `book.toml`)*

```toml
[book]
title = "A thoughtful introduction to the pest parser"
description = "An introduction to the pest parser by implementing a Rust grammar subset"
author = "Dragoș Tiselice"
language = "en"
multilingual = false

[output.html]
git-repository-url = "https://github.com/pest-parser/book/tree/master/"
edit-url-template = "https://github.com/pest-parser/book/edit/master/{path}"
additional-js = ["highlight-pest.js"]

[output.html.playground]
runnable = false

[output.html.print]
enable = false

```

---
## File: `examples/calculator/Cargo.toml`
*(Relative Path: `examples/calculator/Cargo.toml`)*

```toml
[package]
name = "calculator"
version = "0.1.1"
authors = ["wirelyre <wirelyre@gmail.com>"]
edition = "2021"
rust-version = "1.76"

[dependencies]
lazy_static = "1.4"
pest = "2.7"
pest_derive = "2.7"

```

---
## File: `examples/calculator/src/grammar.pest`
*(Relative Path: `examples/calculator/src/grammar.pest`)*

```pest
num = @{ int ~ ("." ~ ASCII_DIGIT*)? ~ (^"e" ~ int)? }
    int = { ("+" | "-")? ~ ASCII_DIGIT+ }

operation = _{ add | subtract | multiply | divide | power }
    add      = { "+" }
    subtract = { "-" }
    multiply = { "*" }
    divide   = { "/" }
    power    = { "^" }

expr = { term ~ (operation ~ term)* }
term = _{ num | "(" ~ expr ~ ")" }

calculation = _{ SOI ~ expr ~ EOI }

WHITESPACE = _{ " " | "\t" }

```

---
## File: `examples/calculator/src/main.rs`
*(Relative Path: `examples/calculator/src/main.rs`)*

```rust
use lazy_static::lazy_static;
use pest_derive::Parser;
use pest::Parser;


use pest::iterators::Pairs;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use std::io::BufRead;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct Calculator;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            .op(Op::infix(power, Right))
    };
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::num => primary.as_str().parse::<f64>().unwrap(),
            Rule::expr => eval(primary.into_inner()),
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide => lhs / rhs,
            Rule::power => lhs.powf(rhs),
            _ => unreachable!(),
        })
        .parse(expression)
}

fn main() {
    let stdin = std::io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap().trim().to_string();
        let parse_result = Calculator::parse(Rule::calculation, &line);
        match parse_result {
            Ok(mut calc) => println!(
                " = {}",
                eval(
                    // inner of expr
                    calc.next().unwrap().into_inner()
                )
            ),
            Err(_) => println!(" Syntax error"),
        }
    }
}

```

---
## File: `examples/json-parser/Cargo.toml`
*(Relative Path: `examples/json-parser/Cargo.toml`)*

```toml
[package]
name = "json-parser"
version = "0.1.1"
authors = ["wirelyre <wirelyre@gmail.com>"]
edition = "2021"
rust-version = "1.56"

[dependencies]
pest = "2.6"
pest_derive = "2.6"

```

---
## File: `examples/json-parser/data.json`
*(Relative Path: `examples/json-parser/data.json`)*

```json
{
    "nesting": { "inner object": {} },
    "an array": [1.5, true, null, 1e-6],
    "string with escaped double quotes" : "\"quick brown foxes\""
}

```

---
## File: `examples/json-parser/src/json.pest`
*(Relative Path: `examples/json-parser/src/json.pest`)*

```pest
json = _{ SOI ~ (object | array) ~ EOI }

value = _{ object | array | string | number | boolean | null }

object = {
    "{" ~ "}" |
    "{" ~ pair ~ ("," ~ pair)* ~ "}"
}
pair = { string ~ ":" ~ value }

array = {
    "[" ~ "]" |
    "[" ~ value ~ ("," ~ value)* ~ "]"
}

string = ${ "\"" ~ inner ~ "\"" }
inner = @{ char* }
char = {
    !("\"" | "\\") ~ ANY
    | "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t")
    | "\\" ~ ("u" ~ ASCII_HEX_DIGIT{4})
}

number = @{
    "-"?
    ~ ("0" | ASCII_NONZERO_DIGIT ~ ASCII_DIGIT*)
    ~ ("." ~ ASCII_DIGIT*)?
    ~ (^"e" ~ ("+" | "-")? ~ ASCII_DIGIT+)?
}

boolean = { "true" | "false" }

null = { "null" }

WHITESPACE = _{ " " | "\t" | "\r" | "\n" }

```

---
## File: `examples/json-parser/src/main.rs`
*(Relative Path: `examples/json-parser/src/main.rs`)*

```rust
use pest::error::Error;
use pest::Parser;
use pest_derive::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "json.pest"]
struct JSONParser;

enum JSONValue<'a> {
    Object(Vec<(&'a str, JSONValue<'a>)>),
    Array(Vec<JSONValue<'a>>),
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

fn serialize_jsonvalue(val: &JSONValue) -> String {
    use JSONValue::*;

    match val {
        Object(o) => {
            let contents: Vec<_> = o
                .iter()
                .map(|(name, value)| format!("\"{}\":{}", name, serialize_jsonvalue(value)))
                .collect();
            format!("{{{}}}", contents.join(","))
        }
        Array(a) => {
            let contents: Vec<_> = a.iter().map(serialize_jsonvalue).collect();
            format!("[{}]", contents.join(","))
        }
        String(s) => format!("\"{}\"", s),
        Number(n) => format!("{}", n),
        Boolean(b) => format!("{}", b),
        Null => format!("null"),
    }
}

fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    use pest::iterators::Pair;

    let json = JSONParser::parse(Rule::json, file)?.next().unwrap();

    fn parse_value(pair: Pair<Rule>) -> JSONValue {
        match pair.as_rule() {
            Rule::object => JSONValue::Object(
                pair.into_inner()
                    .map(|pair| {
                        let mut inner_rules = pair.into_inner();
                        let name = inner_rules
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap()
                            .as_str();
                        let value = parse_value(inner_rules.next().unwrap());
                        (name, value)
                    })
                    .collect(),
            ),
            Rule::array => JSONValue::Array(pair.into_inner().map(parse_value).collect()),
            Rule::string => JSONValue::String(pair.into_inner().next().unwrap().as_str()),
            Rule::number => JSONValue::Number(pair.as_str().parse().unwrap()),
            Rule::boolean => JSONValue::Boolean(pair.as_str().parse().unwrap()),
            Rule::null => JSONValue::Null,
            Rule::json
            | Rule::EOI
            | Rule::pair
            | Rule::value
            | Rule::inner
            | Rule::char
            | Rule::WHITESPACE => unreachable!(),
        }
    }

    Ok(parse_value(json))
}

fn main() {
    let unparsed_file = fs::read_to_string("data.json").expect("cannot read file");

    let json: JSONValue = parse_json_file(&unparsed_file).expect("unsuccessful parse");

    println!("{}", serialize_jsonvalue(&json));
}

```

---
## File: `examples/pest-calculator/Cargo.toml`
*(Relative Path: `examples/pest-calculator/Cargo.toml`)*

```toml
[package]
name = "pest-calculator"
version = "0.1.1"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
lazy_static = "1.4.0"
pest = "2.6"
pest_derive = "2.6"

```

---
## File: `examples/pest-calculator/src/calculator.pest`
*(Relative Path: `examples/pest-calculator/src/calculator.pest`)*

```pest
// No whitespace allowed between digits
integer = @{ ASCII_DIGIT+ }

unary_minus = { "-" }
primary = _{ integer | "(" ~ expr ~ ")" }
atom = _{ unary_minus? ~ primary }

bin_op = _{ add | subtract | multiply | divide | modulo }
	add = { "+" }
	subtract = { "-" }
	multiply = { "*" }
	divide = { "/" }
	modulo = { "%" }

expr = { atom ~ (bin_op ~ atom)* }

// We can't have SOI and EOI on expr directly, because it is used recursively (e.g. with parentheses)
equation = _{ SOI ~ expr ~ EOI }

WHITESPACE = _{ " " }
```

---
## File: `examples/pest-calculator/src/main.rs`
*(Relative Path: `examples/pest-calculator/src/main.rs`)*

```rust
use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use std::io::{self, BufRead};

#[derive(pest_derive::Parser)]
#[grammar = "calculator.pest"]
pub struct CalculatorParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left))
            .op(Op::prefix(unary_minus))
    };
}

#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    UnaryMinus(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer => Expr::Integer(primary.as_str().parse::<i32>().unwrap()),
            Rule::expr => parse_expr(primary.into_inner()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                Rule::modulo => Op::Modulo,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => Expr::UnaryMinus(Box::new(rhs)),
            _ => unreachable!(),
        })
        .parse(pairs)
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

fn main() -> io::Result<()> {
    for line in io::stdin().lock().lines() {
        match CalculatorParser::parse(Rule::equation, &line?) {
            Ok(mut pairs) => {
                println!(
                    "Parsed: {:#?}",
                    // inner of expr
                    parse_expr(pairs.next().unwrap().into_inner())
                );
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
    Ok(())
}

```

---
## File: `examples/ini-parser/Cargo.toml`
*(Relative Path: `examples/ini-parser/Cargo.toml`)*

```toml
[package]
name = "ini-parser"
version = "0.1.1"
authors = ["wirelyre <wirelyre@gmail.com>"]
edition = "2021"
rust-version = "1.56"

[dependencies]
pest = "2.6"
pest_derive = "2.6"
```

---
## File: `examples/ini-parser/config.ini`
*(Relative Path: `examples/ini-parser/config.ini`)*

```ini
username = noha
password = plain_text
salt = NaCl

[server_1]
interface=eth0
ip=127.0.0.1
document_root=/var/www/example.org

[empty_section]

[second_server]
document_root=/var/www/example.com
ip=
interface=eth1

```

---
## File: `examples/ini-parser/src/main.rs`
*(Relative Path: `examples/ini-parser/src/main.rs`)*

```rust
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fs;

#[derive(Parser)]
#[grammar = "ini.pest"]
pub struct INIParser;

fn main() {
    let unparsed_file = fs::read_to_string("config.ini").expect("cannot read file");

    let file = INIParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `file` rule; never fails

    let mut properties: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
    let mut current_section_name = "";

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::section => {
                let mut inner_rules = line.into_inner(); // { name }
                current_section_name = inner_rules.next().unwrap().as_str();
            }
            Rule::property => {
                let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                let name: &str = inner_rules.next().unwrap().as_str();
                let value: &str = inner_rules.next().unwrap().as_str();

                // Insert an empty inner hash map if the outer hash map hasn't
                // seen this section name before.
                let section = properties.entry(current_section_name).or_default();
                section.insert(name, value);
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    println!("{:#?}", properties);
}

```

---
## File: `examples/ini-parser/src/ini.pest`
*(Relative Path: `examples/ini-parser/src/ini.pest`)*

```pest
file = {
    SOI ~
    ((section | property)? ~ NEWLINE)* ~
    EOI
}

char = { ASCII_ALPHANUMERIC | "." | "_" | "/" }

section = { "[" ~ name ~ "]" }
property = { name ~ "=" ~ value }
    name = @{ char+ }
    value = @{ char* }

WHITESPACE = _{ " " }

```

---
## File: `examples/csv-tool/numbers.csv`
*(Relative Path: `examples/csv-tool/numbers.csv`)*

```csv
65279,1179403647,1463895090
3.1415927,2.7182817,1.618034
-40,-273.15
13,42
65537

```

---
## File: `examples/csv-tool/Cargo.toml`
*(Relative Path: `examples/csv-tool/Cargo.toml`)*

```toml
[package]
name = "csv-tool"
version = "0.1.1"
authors = ["wirelyre <wirelyre@gmail.com>"]
edition = "2021"
rust-version = "1.56"

[dependencies]
pest = "2.6"
pest_derive = "2.6"

```

---
## File: `examples/csv-tool/src/csv.pest`
*(Relative Path: `examples/csv-tool/src/csv.pest`)*

```pest
field = { (ASCII_DIGIT | "." | "-")+ }
record = { field ~ ("," ~ field)* }
file = { SOI ~ (record ~ ("\r\n" | "\n"))* ~ EOI }

```

---
## File: `examples/csv-tool/src/main.rs`
*(Relative Path: `examples/csv-tool/src/main.rs`)*

```rust
use pest::Parser;
use pest_derive::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "csv.pest"]
pub struct CSVParser;

fn main() {
    let unparsed_file = fs::read_to_string("numbers.csv").expect("cannot read file");

    let file = CSVParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `file` rule; never fails

    let mut field_sum: f64 = 0.0;
    let mut record_count: u64 = 0;

    for record in file.into_inner() {
        match record.as_rule() {
            Rule::record => {
                record_count += 1;

                for field in record.into_inner() {
                    field_sum += field.as_str().parse::<f64>().unwrap();
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    println!("Sum of fields: {}", field_sum);
    println!("Number of records: {}", record_count);
}

```

---
## File: `examples/jlang-parser/example.ijs`
*(Relative Path: `examples/jlang-parser/example.ijs`)*

```jslang
_2.5 ^ 3
*: 4.8
title =: 'Spinning at the Boundary'
*: _1 2 _3 4
1 2 3 + 10 20 30
1 + 10 20 30
1 2 3 + 10
2 | 0 1 2 3 4 5 6 7
another =: 'It''s Escaped'
3 | 0 1 2 3 4 5 6 7
(2+1)*(2+2)
3 * 2 + 1
1 + 3 % 4
x =: 100
x - 1
y =: x - 1
y

```

---
## File: `examples/jlang-parser/Cargo.toml`
*(Relative Path: `examples/jlang-parser/Cargo.toml`)*

```toml
[package]
name = "jlang-parser"
version = "0.1.1"
authors = ["Matt Quinn <matt@mattjquinn.com>"]
edition = "2021"
rust-version = "1.56"

[dependencies]
pest = "2.6"
pest_derive = "2.6"

```

---
## File: `examples/jlang-parser/src/j.pest`
*(Relative Path: `examples/jlang-parser/src/j.pest`)*

```pest
program = _{ SOI ~ "\n"* ~ (stmt ~ "\n"+) * ~ stmt? ~ EOI }

stmt = _{ expr }

expr = {
      assgmtExpr
    | monadicExpr
    | dyadicExpr
    | string
    | terms
}

monadicExpr = { verb ~ expr }

dyadicExpr = { (monadicExpr | terms) ~ verb ~ expr }

assgmtExpr = { ident ~ "=:" ~ expr }

terms = { term+ }

term = _{ decimal | integer | ident | "(" ~ expr ~ ")" }

verb = {
    ">:" | "*:" | "-"  | "%" | "#" | ">."
  | "+"  | "*"  | "<"  | "=" | "^" | "|"
  | ">"  | "$"
}

integer = @{ "_"? ~ ASCII_DIGIT+ }

decimal = @{ "_"? ~ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT* }

ident = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

string = @{ "'" ~ ( "''" | (!"'" ~ ANY) )* ~ "'" }

WHITESPACE = _{ " " | "\t" }

COMMENT = _{ "NB." ~ (!"\n" ~ ANY)* }

```

---
## File: `examples/jlang-parser/src/main.rs`
*(Relative Path: `examples/jlang-parser/src/main.rs`)*

```rust
use self::AstNode::*;
use pest::error::Error;
use pest::Parser;
use pest_derive::Parser;
use std::ffi::CString;

#[derive(Parser)]
#[grammar = "j.pest"]
pub struct JParser;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MonadicVerb {
    Increment,
    Square,
    Negate,
    Reciprocal,
    Tally,
    Ceiling,
    ShapeOf,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DyadicVerb {
    Plus,
    Times,
    LessThan,
    LargerThan,
    Equal,
    Minus,
    Divide,
    Power,
    Residue,
    Copy,
    LargerOf,
    LargerOrEqual,
    Shape,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    Print(Box<AstNode>),
    Integer(i32),
    DoublePrecisionFloat(f64),
    MonadicOp {
        verb: MonadicVerb,
        expr: Box<AstNode>,
    },
    DyadicOp {
        verb: DyadicVerb,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    Terms(Vec<AstNode>),
    IsGlobal {
        ident: String,
        expr: Box<AstNode>,
    },
    Ident(String),
    Str(CString),
}

pub fn parse(source: &str) -> Result<Vec<AstNode>, Error<Rule>> {
    let mut ast = vec![];

    let pairs = JParser::parse(Rule::program, source)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::expr => {
                ast.push(Print(Box::new(build_ast_from_expr(pair))));
            }
            _ => {}
        }
    }

    Ok(ast)
}

fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
        Rule::monadicExpr => {
            let mut pair = pair.into_inner();
            let verb = pair.next().unwrap();
            let expr = pair.next().unwrap();
            let expr = build_ast_from_expr(expr);
            parse_monadic_verb(verb, expr)
        }
        Rule::dyadicExpr => {
            let mut pair = pair.into_inner();
            let lhspair = pair.next().unwrap();
            let lhs = build_ast_from_expr(lhspair);
            let verb = pair.next().unwrap();
            let rhspair = pair.next().unwrap();
            let rhs = build_ast_from_expr(rhspair);
            parse_dyadic_verb(verb, lhs, rhs)
        }
        Rule::terms => {
            let terms: Vec<AstNode> = pair.into_inner().map(build_ast_from_term).collect();
            // If there's just a single term, return it without
            // wrapping it in a Terms node.
            match terms.len() {
                1 => terms.get(0).unwrap().clone(),
                _ => Terms(terms),
            }
        }
        Rule::assgmtExpr => {
            let mut pair = pair.into_inner();
            let ident = pair.next().unwrap();
            let expr = pair.next().unwrap();
            let expr = build_ast_from_expr(expr);
            AstNode::IsGlobal {
                ident: String::from(ident.as_str()),
                expr: Box::new(expr),
            }
        }
        Rule::string => {
            let str = &pair.as_str();
            // Strip leading and ending quotes.
            let str = &str[1..str.len() - 1];
            // Escaped string quotes become single quotes here.
            let str = str.replace("''", "'");
            AstNode::Str(CString::new(&str[..]).unwrap())
        }
        unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
    }
}

fn parse_dyadic_verb(pair: pest::iterators::Pair<Rule>, lhs: AstNode, rhs: AstNode) -> AstNode {
    AstNode::DyadicOp {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        verb: match pair.as_str() {
            "+" => DyadicVerb::Plus,
            "*" => DyadicVerb::Times,
            "-" => DyadicVerb::Minus,
            "<" => DyadicVerb::LessThan,
            "=" => DyadicVerb::Equal,
            ">" => DyadicVerb::LargerThan,
            "%" => DyadicVerb::Divide,
            "^" => DyadicVerb::Power,
            "|" => DyadicVerb::Residue,
            "#" => DyadicVerb::Copy,
            ">." => DyadicVerb::LargerOf,
            ">:" => DyadicVerb::LargerOrEqual,
            "$" => DyadicVerb::Shape,
            _ => panic!("Unexpected dyadic verb: {}", pair.as_str()),
        },
    }
}

fn parse_monadic_verb(pair: pest::iterators::Pair<Rule>, expr: AstNode) -> AstNode {
    AstNode::MonadicOp {
        verb: match pair.as_str() {
            ">:" => MonadicVerb::Increment,
            "*:" => MonadicVerb::Square,
            "-" => MonadicVerb::Negate,
            "%" => MonadicVerb::Reciprocal,
            "#" => MonadicVerb::Tally,
            ">." => MonadicVerb::Ceiling,
            "$" => MonadicVerb::ShapeOf,
            _ => panic!("Unsupported monadic verb: {}", pair.as_str()),
        },
        expr: Box::new(expr),
    }
}

fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::integer => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "_" => (-1, &istr[1..]),
                _ => (1, &istr[..]),
            };
            let integer: i32 = istr.parse().unwrap();
            AstNode::Integer(sign * integer)
        }
        Rule::decimal => {
            let dstr = pair.as_str();
            let (sign, dstr) = match &dstr[..1] {
                "_" => (-1.0, &dstr[1..]),
                _ => (1.0, &dstr[..]),
            };
            let mut flt: f64 = dstr.parse().unwrap();
            if flt != 0.0 {
                // Avoid negative zeroes; only multiply sign by nonzeroes.
                flt *= sign;
            }
            AstNode::DoublePrecisionFloat(flt)
        }
        Rule::expr => build_ast_from_expr(pair),
        Rule::ident => AstNode::Ident(String::from(pair.as_str())),
        unknown_term => panic!("Unexpected term: {:?}", unknown_term),
    }
}

fn main() {
    let unparsed_file = std::fs::read_to_string("example.ijs").expect("cannot read ijs file");
    let astnode = parse(&unparsed_file).expect("unsuccessful parse");
    println!("{:?}", &astnode);
}

```

---
## File: `src/intro.md`
*(Relative Path: `src/intro.md`)*

```markdown
# Introduction

*Speed or simplicity? Why not __both__?*

`pest` is a library for writing plain-text parsers in Rust.

Parsers that use `pest` are **easy to design and maintain** due to the use of
[Parsing Expression Grammars], or *PEGs*. And, because of Rust's zero-cost
abstractions, `pest` parsers can be **very fast**.

## Sample

Here is the complete grammar for a simple calculator [developed in a later chapter](examples/calculator.html):

```pest
num = @{ int ~ ("." ~ ASCII_DIGIT*)? ~ (^"e" ~ int)? }
int = { ("+" | "-")? ~ ASCII_DIGIT+ }

operation = _{ add | subtract | multiply | divide | power }
    add      = { "+" }
    subtract = { "-" }
    multiply = { "*" }
    divide   = { "/" }
    power    = { "^" }

expr = { term ~ (operation ~ term)* }
term = _{ num | "(" ~ expr ~ ")" }

calculation = _{ SOI ~ expr ~ EOI }

WHITESPACE = _{ " " | "\t" }
```

And here is the function that uses that parser to calculate answers:

```rust
lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            .op(Op::infix(power, Right))
    };
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::num => primary.as_str().parse::<f64>().unwrap(),
            Rule::expr => eval(primary.into_inner()),
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide => lhs / rhs,
            Rule::power => lhs.powf(rhs),
            _ => unreachable!(),
        })
        .parse(expression)
}
```

## About this book

This book provides an overview of `pest` as well as several example parsers.
For more details of `pest`'s API, check [the documentation].

Note that `pest` uses some advanced features of the Rust language. For an
introduction to Rust, consult the [official Rust book].

[Parsing Expression Grammars]: grammars/peg.html
[the documentation]: https://docs.rs/pest/
[official Rust book]: https://doc.rust-lang.org/stable/book/second-edition/

```

---
## File: `src/SUMMARY.md`
*(Relative Path: `src/SUMMARY.md`)*

```markdown
# Summary

- [Introduction](intro.md)
    - [Example: CSV](examples/csv.md)
- [Parser API](parser_api.md)
    - [Example: INI](examples/ini.md)
- [Grammars](grammars/grammars.md)
    - [Parsing expression grammars](grammars/peg.md)
    - [Syntax of pest parsers](grammars/syntax.md)
    - [Comments](grammars/comments.md)
    - [Built-in rules](grammars/built-ins.md)
    - [Example: JSON](examples/json.md)
    - [Example: The J language](examples/jlang.md)
- [Operator precedence](precedence.md)
    - [Example: Calculator](examples/calculator.md)
- [Final project: Awk clone (WIP)](examples/awk.md)

```

---
## File: `src/precedence.md`
*(Relative Path: `src/precedence.md`)*

```markdown
# Operator precedence

There are several methods for dealing with operator precedence in `pest`:

1. directly in the PEG grammar;
2. using a `PrecClimber` (*deprecated*);
3. using a `PrattParser`.

Given `PrattParser` is the most general available method that supports
unary prefix and suffix operators, we provide more details on its usage.

## Pratt Parser
The following pest grammar defines a calculator which can be used for Pratt parsing.

```pest
WHITESPACE   =  _{ " " | "\t" | NEWLINE }
  
program      =   { SOI ~ expr ~ EOI }
  expr       =   { prefix? ~ primary ~ postfix? ~ (infix ~ prefix? ~ primary ~ postfix? )* }
    infix    =  _{ add | sub | mul | div | pow }
      add    =   { "+" } // Addition
      sub    =   { "-" } // Subtraction
      mul    =   { "*" } // Multiplication
      div    =   { "/" } // Division
      pow    =   { "^" } // Exponentiation
    prefix   =  _{ neg }
      neg    =   { "-" } // Negation
    postfix  =  _{ fac }
      fac    =   { "!" } // Factorial
    primary  =  _{ int | "(" ~ expr ~ ")" }
      int    =  @{ (ASCII_NONZERO_DIGIT ~ ASCII_DIGIT+ | ASCII_DIGIT) }
```

Below is a `PrattParser` that is able to parse an expr in the above grammar. The order of precedence corresponds to the order in which `op` is called. Thus, `mul` will have higher precedence than `add`. Operators can also be chained with `|` to give them equal precedence.

```rust
let pratt =
    PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::infix(Rule::pow, Assoc::Right))
        .op(Op::postfix(Rule::fac))
        .op(Op::prefix(Rule::neg));
```

To parse an expression, call the `map_primary`, `map_prefix`, `map_postfix`, `map_infix` and parse methods as follows:

```rust
fn parse_expr(pairs: Pairs<Rule>, pratt: &PrattParser<Rule>) -> i32 {
    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::int  => primary.as_str().parse().unwrap(),
            Rule::expr => parse_expr(primary.into_inner(), pratt), // from "(" ~ expr ~ ")"
            _          => unreachable!(),
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg  => -rhs,
            _          => unreachable!(),
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::fac  => (1..lhs+1).product(),
            _          => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add  => lhs + rhs,
            Rule::sub  => lhs - rhs,
            Rule::mul  => lhs * rhs,
            Rule::div  => lhs / rhs,
            Rule::pow  => (1..rhs+1).map(|_| lhs).product(),
            _          => unreachable!(),
        })
        .parse(pairs)
}
```

Note that `map_prefix`, `map_postfix` and `map_infix` only need to be specified if the grammar contains the corresponding operators.

```

---
## File: `src/parser_api.md`
*(Relative Path: `src/parser_api.md`)*

```markdown
# Parser API

`pest` provides several ways of accessing the results of a successful parse.
The examples below use the following grammar:

```pest
number = { ASCII_DIGIT+ }                // one or more decimal digits
enclosed = { "(.." ~ number ~ "..)" }    // for instance, "(..6472..)"
sum = { number ~ " + " ~ number }        // for instance, "1362 + 12"
```

## Tokens

`pest` represents successful parses using *tokens*. Whenever a rule matches,
two tokens are produced: one at the *start* of the text that the rule matched,
and one at the *end*. For example, the rule `number` applied to the string
`"3130 abc"` would match and produce this pair of tokens:

```
"3130 abc"
 |   ^ end(number)
 ^ start(number)
```

Note that the rule doesn't match the entire input text. It only matches as much
text as possible, then stops if successful.

A token is like a cursor in the input string. It has a character position in
the string, as well as a reference to the rule that created it.

### Nested rules

If a named rule contains another named rule, tokens will be produced for *both*
rules. For instance, the rule `enclosed` applied to the string `"(..6472..)"`
would match and produce these four tokens:

```
"(..6472..)"
 |  |   |  ^ end(enclosed)
 |  |   ^ end(number)
 |  ^ start(number)
 ^ start(enclosed)
```

Sometimes, tokens might not occur at distinct character positions. For example,
when parsing the rule `sum`, the inner `number` rules share some start and end
positions:

```
"1773 + 1362"
 |   |  |   ^ end(sum)
 |   |  |   ^ end(number)
 |   |  ^ start(number)
 |   ^ end(number)
 ^ start(number)
 ^ start(sum)
```

In fact, for a rule that matches empty input, the start and end tokens will be
at the same position!

### Interface

Tokens are exposed as the [`Token`] enum, which has `Start` and `End` variants.
You can get an iterator of `Token`s by calling `tokens` on a parse result:

```rust
let parse_result = Parser::parse(Rule::sum, "1773 + 1362").unwrap();
let tokens = parse_result.tokens();

for token in tokens {
    println!("{:?}", token);
}
```

After a successful parse, tokens will occur as nested pairs of matching `Start`
and `End`. Both kinds of tokens have two fields:

- `rule`, which explains which rule generated them; and
- `pos`, which indicates their positions.

A start token's position is the first character that the rule matched. An end
token's position is the first character that the rule did not match &mdash;
that is, an end token refers to a position *after* the match. If a rule matched
the entire input string, the end token points to an imaginary position *after*
the string.

## Pairs

Tokens are not the most convenient interface, however. Usually you will want to
explore the parse tree by considering matching pairs of tokens. For this
purpose, `pest` provides the [`Pair`] type.

A `Pair` represents a matching pair of tokens, or, equivalently, the spanned
text that a named rule successfully matched. It is commonly used in several
ways:

- Determining which rule produced the `Pair`
- Using the `Pair` as a raw `&str`
- Inspecting the inner named sub-rules that produced the `Pair`

```rust
let pair = Parser::parse(Rule::enclosed, "(..6472..) and more text")
    .unwrap().next().unwrap();

assert_eq!(pair.as_rule(), Rule::enclosed);
assert_eq!(pair.as_str(), "(..6472..)");

let inner_rules = pair.into_inner();
println!("{}", inner_rules); // --> [number(3, 7)]
```

In general, a `Pair` might have any number of inner rules: zero, one, or more.
For maximum flexibility, `Pair::into_inner()` returns `Pairs`, which is an
iterator over each pair.

This means that you can use `for` loops on parse results, as well as iterator
methods such as `map`, `filter`, and `collect`.

```rust
let pairs = Parser::parse(Rule::sum, "1773 + 1362")
    .unwrap().next().unwrap()
    .into_inner();

let numbers = pairs
    .clone()
    .map(|pair| str::parse(pair.as_str()).unwrap())
    .collect::<Vec<i32>>();
assert_eq!(vec![1773, 1362], numbers);

for (found, expected) in pairs.zip(vec!["1773", "1362"]) {
    assert_eq!(Rule::number, found.as_rule());
    assert_eq!(expected, found.as_str());
}
```

`Pairs` iterators are also commonly used via the `next` method directly. If a
rule consists of a known number of sub-rules (for instance, the rule `sum` has
exactly two sub-rules), the sub-matches can be extracted with `next` and
`unwrap`:

```rust
let parse_result = Parser::parse(Rule::sum, "1773 + 1362")
    .unwrap().next().unwrap();
let mut inner_rules = parse_result.into_inner();

let match1 = inner_rules.next().unwrap();
let match2 = inner_rules.next().unwrap();

assert_eq!(match1.as_str(), "1773");
assert_eq!(match2.as_str(), "1362");
```

Sometimes rules will not have a known number of sub-rules, such as when a
sub-rule is repeated with an asterisk `*`:

```pest
list = { number* }
```

In cases like these it is not possible to call `.next().unwrap()`, because the
number of sub-rules depends on the input string &mdash; it cannot be known at
compile time.

## The `parse` method

A `pest`-derived [`Parser`] has a single method `parse` which returns a
`Result< Pairs, Error >`. To access the underlying parse tree, it is necessary
to `match` on or `unwrap` the result:

```rust
// check whether parse was successful
match Parser::parse(Rule::enclosed, "(..6472..)") {
    Ok(mut pairs) => {
        let enclosed = pairs.next().unwrap();
        // ...
    }
    Err(error) => {
        // ...
    }
}
```

Our examples so far have included the calls
`Parser::parse(...).unwrap().next().unwrap()`. The first `unwrap` turns the
result into a `Pairs`. If parsing had failed, the program would panic! We only
use `unwrap` in these examples because we already know that they will parse
successfully.

In the example above, in order to get to the `enclosed` rule inside of the
`Pairs`, we use the iterator interface. The `next()` call returns an
`Option<Pair>`, which we finally `unwrap` to get the `Pair` for the `enclosed`
rule.

### Using `Pair` and `Pairs` with a grammar

While the `Result` from `Parser::parse(...)` might very well be an error on
invalid input, `Pair` and `Pairs` often have more subtle behavior. For
instance, with this grammar:

```pest
number = { ASCII_DIGIT+ }
sum = { number ~ " + " ~ number }
```

this function will *never* panic:

```rust
fn process(pair: Pair<Rule>) -> f64 {
    match pair.as_rule() {
        Rule::number => str::parse(pair.as_str()).unwrap(),
        Rule::sum => {
            let mut pairs = pair.into_inner();

            let num1 = pairs.next().unwrap();
            let num2 = pairs.next().unwrap();

            process(num1) + process(num2)
        }
    }
}
```

`str::parse(...).unwrap()` is safe because the `number` rule only ever matches
digits, which `str::parse(...)` can handle. And `pairs.next().unwrap()` is safe
to call twice because a `sum` match *always* has two sub-matches, which is
guaranteed by the grammar.

Since these sorts of guarantees are awkward to express with Rust types, `pest`
only provides a few high-level types to represent parse trees. Nevertheless,
you *should* rely on the meaning of your grammar for properties such as
"contains *n* sub-rules", "is safe to `parse` to `f32`", and "never fails to
match". Idiomatic `pest` code uses `unwrap` and `unreachable!`.

## Spans and positions

Occasionally, you will want to refer to a matching rule in the context of the
raw source text, rather than the interior text alone. For example, you might
want to print the entire line that contained the match. For this you can use
[`Span`] and [`Position`].

A `Span` is returned from `Pair::as_span`. Spans have a start position and an
end position (which correspond to the start and end tokens of the rule that
made the pair).

Spans can be decomposed into their start and end `Position`s, which provide
useful methods for examining the string around that position. For example,
`Position::line_col()` finds out the line and column number of a position.

Essentially, a `Position` is a `Token` without a rule. In fact, you can use
pattern matching to turn a `Token` into its component `Rule` and `Position`.

[`Token`]: https://docs.rs/pest/2.0/pest/enum.Token.html
[`Pair`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pair.html
[`Parser`]: https://docs.rs/pest/2.0/pest/trait.Parser.html
[`Span`]: https://docs.rs/pest/2.0/pest/struct.Span.html
[`Position`]: https://docs.rs/pest/2.0/pest/struct.Position.html

```

---
## File: `src/grammars/grammars.md`
*(Relative Path: `src/grammars/grammars.md`)*

```markdown
# Grammars

Like many parsing tools, `pest` operates using a *formal grammar* that is
distinct from your Rust code. The format that `pest` uses is called a *parsing
expression grammar*, or *PEG*. When building a project, `pest` automatically
compiles the PEG, located in a separate file, into a plain Rust function that
you can call.

## How to activate `pest`

Most projects will have at least two files that use `pest`: the parser (say,
`src/parser/mod.rs`) and the grammar (`src/parser/grammar.pest`). Assuming that
they are in the same directory:

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to project `src`
struct MyParser;
```

Whenever you compile this file, `pest` will automatically use the grammar file
to generate items like this:

```rust
pub enum Rules { /* ... */ }

impl Parser for MyParser {
    pub fn parse(Rules, &str) -> pest::Pairs { /* ... */ }
}
```

You will never see `enum Rules` or `impl Parser` as plain text! The code only
exists during compilation. However, you can use `Rules` just like any other
enum, and you can use `parse(...)` through the [`Pairs`] interface described in
the [Parser API chapter](../parser_api.html).

## Inline grammar

If you don't want to have a separate grammar file, you can use the `grammar_inline`:

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar_inline = r#"
// your grammar here
a = { "a" }
"#]
struct MyParser;
```

## Load multiple grammars

If you have multiple grammars, you can load them all at once:

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/base.pest"]
#[grammar = "parser/grammar.pest"]
struct MyParser;
```

Then `pest` will generate a `Rules` enum that contains all the rules from both.
This is useful if you have a base grammar that you want to extend in multiple grammars.

## Warning about PEGs!

Parsing expression grammars look quite similar to other parsing tools you might
be used to, like regular expressions, BNF grammars, and others (Yacc/Bison,
LALR, CFG). However, PEGs behave subtly differently: PEGs are [eager],
[non-backtracking], [ordered], and [unambiguous].

Don't be scared if you don't recognize any of the above names! You're already a
step ahead of people who do &mdash; when you use `pest`'s PEGs, you won't be
tripped up by comparisons to other tools.

If you have used other parsing tools before, be sure to read the next section
carefully. We'll mention some common mistakes regarding PEGs.

[`Pairs`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pairs.html
[`include_str!`]: https://doc.rust-lang.org/std/macro.include_str.html
[eager]: peg.html#eagerness
[non-backtracking]: peg.html#non-backtracking
[ordered]: peg.html#ordered-choice
[unambiguous]: peg.html#unambiguous

```

---
## File: `src/grammars/comments.md`
*(Relative Path: `src/grammars/comments.md`)*

```markdown
## Comments

### Non-doc comments

Comments follow the general Rust style of line (`//`) and block (`/* ... */`) comment forms.
Non-doc comments are interpreted as a form of whitespace.

```pest
/* 
  Block comment
 */
another_rule = {        // line comment
    ...                 // whitespace goes anywhere
}
```

### Doc comments

Line doc comments begin with exactly three slashes `///`
 and `//!` is used to document the entire grammar file.

```pest
//! A parser for JSON file.

json = { ... }

/// Matches object, e.g.: `{ "foo": "bar" }`
object = { ... }
```

Then will get

```rust
/// A parser for JSON file.
enum Rule {
    json,
    /// Matches object, e.g.: `{ "foo": "bar" }`
    object,
}
```

```

---
## File: `src/grammars/peg.md`
*(Relative Path: `src/grammars/peg.md`)*

```markdown
# Parsing expression grammar

Parsing expression grammars (PEGs) are simply a strict representation of the
simple imperative code that you would write if you were writing a parser by
hand.

```pest
number = {            // To recognize a number...
    ASCII_DIGIT+      //   take as many ASCII digits as possible (at least one).
}
expression = {        // To recognize an expression...
    number            //   first try to take a number...
    | "true"          //   or, if that fails, the string "true".
}
```

In fact, `pest` produces code that is quite similar to the pseudo-code in the
comments above.

## Eagerness

When a [repetition] PEG expression is run on an input string,

```pest
ASCII_DIGIT+      // one or more characters from '0' to '9'
```

it runs that expression as many times as it can (matching "eagerly", or
"greedily"). It either succeeds, consuming whatever it matched and passing the
remaining input on to the next step in the parser,

```
"42 boxes"
 ^ Running ASCII_DIGIT+

"42 boxes"
   ^ Successfully took one or more digits!

" boxes"
 ^ Remaining unparsed input.
```

or fails, consuming nothing.

```
"galumphing"
 ^ Running ASCII_DIGIT+
   Failed to take one or more digits!

"galumphing"
 ^ Remaining unparsed input (everything).
```

If an expression fails to match, the failure propagates upwards, eventually
leading to a failed parse, unless the failure is "caught" somewhere in the
grammar. The *choice operator* is one way to "catch" such failures.

[repetition]: syntax.html#repetition

## Ordered choice

The [choice operator], written as a vertical line `|`, is *ordered*. The PEG
expression `first | second` means "try `first`; but if it fails, try `second`
instead".

In many cases, the ordering does not matter. For instance, `"true" | "false"`
will match either the string `"true"` or the string `"false"` (and fail if
neither occurs).

However, sometimes the ordering *does* matter. Consider the PEG expression `"a"
| "ab"`. You might expect it to match either the string `"a"` or the string
`"ab"`. But it will not &mdash; the expression means "try `"a"`; but if it
fails, try `"ab"` instead". If you are matching on the string `"abc"`, "try
`"a"`" will *not* fail; it will instead match `"a"` successfully, leaving
`"bc"` unparsed!

In general, when writing a parser with choices, put the longest or most
specific choice first, and the shortest or most general choice last.

[choice operator]: syntax.html#ordered-choice

## Non-backtracking

During parsing, a PEG expression either succeeds or fails. If it succeeds, the
next step is performed as usual. But if it fails, the whole expression fails.
The engine will not back up and try again.

Consider this grammar, matching on the string `"frumious"`:

```pest
word = {     // to recognize a word...
    ANY*     //   take any character, zero or more times...
    ~ ANY    //   followed by any character
}
```

You might expect this rule to parse any input string that contains at least one
character (equivalent to `ANY+`). But it will not. Instead, the first `ANY*`
will eagerly eat the entire string &mdash; it will *succeed*. Then, the next
`ANY` will have nothing left, so it will fail.

```
"frumious"
 ^ (word)

"frumious"
         ^ (ANY*) Success! Continue to `ANY` with remaining input "".

""
 ^ (ANY) Failure! Expected one character, but found end of string.
```

In a system with backtracking (like regular expressions), you would back up one
step, "un-eating" a character, and then try again. But PEGs do not do this. In
the rule `first ~ second`, once `first` parses successfully, it has consumed
some characters that will never come back. `second` can only run on the input
that `first` did not consume.

## Unambiguous

These rules form an elegant and simple system. Every PEG rule is run on the
remainder of the input string, consuming as much input as necessary. Once a
rule is done, the rest of the input is passed on to the rest of the parser.

For instance, the expression `ASCII_DIGIT+`, "one or more digits", will always
match the largest sequence of consecutive digits possible. There is no danger
of accidentally having a later rule back up and steal some digits in an
unintuitive and nonlocal way.

This contrasts with other parsing tools, such as regular expressions and CFGs,
where the results of a rule often depend on code some distance away. Indeed,
the famous "shift/reduce conflict" in LR parsers is not a problem in PEGs.

# Don't panic

This all might be a bit counterintuitive at first. But as you can see, the
basic logic is very easy and straightforward. You can trivially step through
the execution of any PEG expression.

- Try this.
- If it succeeds, try the next thing.
- Otherwise, try the other thing.

```
(this ~ next_thing) | (other_thing)
```

These rules together make PEGs very pleasant tools for writing a parser.

```

---
## File: `src/grammars/syntax.md`
*(Relative Path: `src/grammars/syntax.md`)*

```markdown
# Syntax of pest grammars

`pest` grammars are lists of rules. Rules are defined like this:

```pest
//! Grammar doc
my_rule = { ... }

/// Rule doc
another_rule = {        // comments are preceded by two slashes
    ...                 // whitespace goes anywhere
}
```

Since rule names are translated into Rust enum variants, they are not allowed
to be Rust keywords.

The left curly bracket `{` defining a rule can be preceded by [symbols that
affect its operation]:

```pest
silent_rule = _{ ... }
atomic_rule = @{ ... }
```

[symbols that affect its operation]: #silent-and-atomic-rules

## Expressions

Grammar rules are built from *expressions* (hence "parsing expression
grammar"). These expressions are a terse, formal description of how to parse an
input string.

Expressions are composable: they can be built out of other expressions and
nested inside of each other to produce arbitrarily complex rules (although you
should break very complicated expressions into multiple rules to make them
easier to manage).

PEG expressions are suitable for both high-level meaning, like "a function
signature, followed by a function body", and low-level meaning, like "a
semicolon, followed by a line feed". The combining form "followed by",
the [sequence operator], is the same in either case.

[sequence operator]: #sequence

### Terminals

The most basic rule is a **literal string** in double quotes: `"text"`.

A string can be **case-insensitive** (for ASCII characters only) if preceded by
a caret: `^"text"`.

A single **character in a range** is written as two single-quoted characters,
separated by two dots: `'0'..'9'`.

You can match **any single character** at all with the special rule `ANY`. This
is equivalent to `'\u{00}'..'\u{10FFFF}'`, any single Unicode character.

```
"a literal string"
^"ASCII case-insensitive string"
'a'..'z'
ANY
```

Finally, you can **refer to other rules** by writing their names directly, and
even **use rules recursively**:

```pest
my_rule = { "slithy " ~ other_rule }
other_rule = { "toves" }
recursive_rule = { "mimsy " ~ recursive_rule }
```

### Sequence

The sequence operator is written as a tilde `~`.

```
first ~ and_then

("abc") ~ (^"def") ~ ('g'..'z')        // matches "abcDEFr"
```

When matching a sequence expression, `first` is attempted. If `first` matches
successfully, `and_then` is attempted next. However, if `first` fails, the
entire expression fails.

A list of expressions can be chained together with sequences, which indicates
that *all* of the components must occur, in the specified order.

### Ordered choice

The choice operator is written as a vertical line `|`.

```
first | or_else

("abc") | (^"def") | ('g'..'z')        // matches "DEF"
```

When matching a choice expression, `first` is attempted. If `first` matches
successfully, the entire expression *succeeds immediately*. However, if `first`
fails, `or_else` is attempted next.

Note that `first` and `or_else` are always attempted at the same position, even
if `first` matched some input before it failed. When encountering a parse
failure, the engine will try the next ordered choice as though no input had
been matched. Failed parses never consume any input.

```pest
start = { "Beware " ~ creature }
creature = {
    ("the " ~ "Jabberwock")
    | ("the " ~ "Jubjub bird")
}
```

```
"Beware the Jubjub bird"
 ^ (start) Parses via the second choice of `creature`,
           even though the first choice matched "the " successfully.
```

It is somewhat tempting to borrow terminology and think of this operation as
"alternation" or simply "OR", but this is misleading. The word "choice" is used
specifically because [the operation is *not* merely logical "OR"].

[the operation is *not* merely logical "OR"]: peg.html#ordered-choice

### Repetition

There are two repetition operators: the asterisk `*` and plus sign `+`. They
are placed after an expression. The asterisk `*` indicates that the preceding
expression can occur **zero or more** times. The plus sign `+` indicates that
the preceding expression can occur **one or more** times (it must occur at
least once).

The question mark operator `?` is similar, except it indicates that the
expression is **optional** &mdash; it can occur zero or one times.

```
("zero" ~ "or" ~ "more")*
 ("one" | "or" | "more")+
           (^"optional")?
```

Note that `expr*` and `expr?` will always succeed, because they are allowed to
match zero times. For example, `"a"* ~ "b"?` will succeed even on an empty
input string.

Other **numbers of repetitions** can be indicated using curly brackets:

```
expr{n}           // exactly n repetitions
expr{m, n}        // between m and n repetitions, inclusive

expr{, n}         // at most n repetitions
expr{m, }         // at least m repetitions
```

Thus `expr*` is equivalent to `expr{0, }`; `expr+` is equivalent to `expr{1,
}`; and `expr?` is equivalent to `expr{0, 1}`.

### Predicates

Preceding an expression with an ampersand `&` or exclamation mark `!` turns it
into a *predicate* that never consumes any input. You might know these
operators as "lookahead" or "non-progressing".

The **positive predicate**, written as an ampersand `&`, attempts to match its
inner expression. If the inner expression succeeds, parsing continues, but at
the *same position* as the predicate &mdash; `&foo ~ bar` is thus a kind of
"AND" statement: "the input string must match `foo` AND `bar`". If the inner
expression fails, the whole expression fails too.

The **negative predicate**, written as an exclamation mark `!`, attempts to
match its inner expression. If the inner expression *fails*, the predicate
*succeeds* and parsing continues at the same position as the predicate. If the
inner expression *succeeds*, the predicate *fails* &mdash; `!foo ~ bar` is thus
a kind of "NOT" statement: "the input string must match `bar` but NOT `foo`".

This leads to the common idiom meaning "any character but":

```pest
not_space_or_tab = {
    !(                // if the following text is not
        " "           //     a space
        | "\t"        //     or a tab
    )
    ~ ANY             // then consume one character
}

triple_quoted_string = {
    "'''"
    ~ triple_quoted_character*
    ~ "'''"
}
triple_quoted_character = {
    !"'''"        // if the following text is not three apostrophes
    ~ ANY         // then consume one character
}
```

## Operator precedence and grouping (WIP)

The repetition operators asterisk `*`, plus sign `+`, and question mark `?`
apply to the immediately preceding expression.

```
"One " ~ "or " ~ "more. "+
"One " ~ "or " ~ ("more. "+)
    are equivalent and match
"One or more. more. more. more. "
```

Larger expressions can be repeated by surrounding them with parentheses.

```
("One " ~ "or " ~ "more. ")+
    matches
"One or more. One or more. "
```

Repetition operators have the highest precedence, followed by predicate
operators, the sequence operator, and finally ordered choice.

```pest
my_rule = {
    "a"* ~ "b"?
    | &"b"+ ~ "a"
}

// equivalent to

my_rule = {
      ( ("a"*) ~ ("b"?) )
    | ( (&("b"+)) ~ "a" )
}
```

## Start and end of input

The rules `SOI` and `EOI` match the *start* and *end* of the input string,
respectively. Neither consumes any text. They only indicate whether the parser
is currently at one edge of the input.

For example, to ensure that a rule matches the entire input, where any syntax
error results in a failed parse (rather than a successful but incomplete
parse):

```pest
main = {
    SOI
    ~ (...)
    ~ EOI
}
```

## Implicit whitespace

Many languages and text formats allow arbitrary whitespace and comments between
logical tokens. For instance, Rust considers `4+5` equivalent to `4 + 5` and `4
/* comment */ + 5`.

The **optional rules `WHITESPACE` and `COMMENT`** implement this behaviour. If
either (or both) are defined, they will be implicitly inserted at every
[sequence] and between every [repetition] (except in [atomic rules]).

```pest
expression = { "4" ~ "+" ~ "5" }
WHITESPACE = _{ " " }
COMMENT = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" }
```

```
"4+5"
"4 + 5"
"4  +     5"
"4 /* comment */ + 5"
```

As you can see, `WHITESPACE` and `COMMENT` are run repeatedly, so they need
only match a single whitespace character or a single comment. The grammar above
is equivalent to:

```pest
expression = {
    "4"   ~ (ws | com)*
    ~ "+" ~ (ws | com)*
    ~ "5"
}
ws = _{ " " }
com = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" }
```

Note that Implicit whitespace is *not* inserted at the beginning or end of rules
&mdash; for instance, `expression` does *not* match `" 4+5 "`. If you want to
include Implicit whitespace at the beginning and end of a rule, you will need to
sandwich it between two empty rules (often `SOI` and `EOI` [as above]):

```pest
WHITESPACE = _{ " " }
expression = { "4" ~ "+" ~ "5" }
main = { SOI ~ expression ~ EOI }
```

```
"4+5"
"  4 + 5   "
```

(Be sure to mark the `WHITESPACE` and `COMMENT` rules as [silent] unless you
want to see them included inside other rules!)

If you want the `WHITESPACE` and `COMMENT` inner rules included,
note that you need to mark the rule with an explicit `$`:

```
COMMENT = ${ SingleLineComment }
SingleLineComment = { "//" ~ (!"\n" ~ ANY)* }
```

[sequence]: #sequence
[repetition]: #repetition
[atomic rules]: #atomic
[as above]: #start-and-end-of-input
[silent]: #silent-and-atomic-rules

## Silent and atomic rules

### Silent

**Silent** rules are just like normal rules &mdash; when run, they function the
same way &mdash; except they do not produce [pairs] or [tokens]. If a rule is
silent, it will never appear in a parse result.

To make a silent rule, precede the left curly bracket `{` with a low line
(underscore) `_`.

```pest
silent = _{ ... }
```

Rules called from a silent rule are not treated as silent unless they are
declared to be silent. These rules may produce [pairs] or [tokens] and can appear
in a parse result.

[pairs]: ../parser_api.html#pairs
[tokens]: ../parser_api.html#tokens

### Atomic

Pest has two kinds of atomic rules: **atomic** and **compound atomic**. To
make one, write the sigil before the left curly bracket `{`.

```pest
/// Atomic rule start with `@`
atomic = @{ ... }

/// Compound Atomic start with `$`
compound_atomic = ${ ... }
```

Both kinds of atomic rule prevent [implicit whitespace]:

1. Inside an atomic rule, the tilde `~` means "immediately followed by".
2. [Repetition operators] (asterisk `*` and plus sign `+`) have no implicit separation.

In addition, all other rules called from an atomic rule are also treated as atomic.

The difference between the two is how they produce tokens for inner rules:

- **atomic** - In an Atomic rule, interior matching rules are [silent].
- **compound atomic** - By contrast, compound atomic rules produce inner tokens as normal.

Atomic rules are useful when the text you are parsing ignores whitespace except
in a few cases, such as literal strings. In this instance, you can write
`WHITESPACE` or `COMMENT` rules, then make your string-matching rule be atomic.

[implicit whitespace]: #implicit-whitespace
[repetition operators]: #repetition
[silent]: #silent-and-atomic-rules

### Non-atomic

Sometimes, you'll want to cancel the effects of atomic parsing. For instance,
you might want to have string interpolation with an expression inside, where
the inside expression can still have whitespace like normal.

```python
#!/bin/env python3
print(f"The answer is {2 + 4}.")
```

This is where you use a **non-atomic** rule. Write an exclamation mark `!` in
front of the defining curly bracket. The rule will run as non-atomic, whether
it is called from an atomic rule or not.

```pest
fstring = @{ "\"" ~ ... }
expr = !{ ... }
```

### Tags
Sometimes, you may want to attach a label to a part of a rule. This is useful for distinguishing
among different types of tokens (of the same expression) or for the ease of extracting information
from parse trees (without creating additional rules).
To do this, you can use the `#` symbol to bind a name to a part of a rule:

```pest
rule = { #tag = ... }
```

You can then access tags in your parse tree by using the `as_node_tag` method on `Pair`
or you can use the helper methods `find_first_tagged` or `find_tagged` on `Pairs`:

```rust
let pairs = ExampleParser::parse(Rule::example_rule, example_input).unwrap();
for pair in pairs.clone() {
    if let Some(tag) = pair.as_node_tag() {
        // ...
    }
}
let first = pairs.find_first_tagged("tag");
let all_tagged = pairs.find_tagged("tag");
```

Note that the tagged part of the rule cannot be [silent](#silent)
or the tag will be ignored, as would be the case in the following example.

```pest
rule = { #tag = expression }
expression = _{ ... }
```

This also includes [built-in](/src/grammars/built-ins.md) rules, which are always silent.
So the following example will also not create a tag.

```pest
rule = { #tag = ASCII }
```

[!WARNING]
You need to enable "grammar-extras" feature to use this functionality:

```toml
# ...
pest_derive = { version = "2.7", features = ["grammar-extras"] }
```

## The stack (WIP)

`pest` maintains a stack that can be manipulated directly from the grammar. An
expression can be matched and pushed onto the stack with the keyword `PUSH`,
then later matched exactly with the keywords `PEEK` and `POP`.

Using the stack allows *the exact same text* to be matched multiple times,
rather than *the same pattern*.

For example,

```pest
same_text = {
    PUSH( "a" | "b" | "c" )
    ~ POP
}
same_pattern = {
    ("a" | "b" | "c")
    ~ ("a" | "b" | "c")
}
```

In this case, `same_pattern` will match `"ab"`, while `same_text` will not.

One practical use is in parsing Rust ["raw string literals"], which look like
this:

```rust
const raw_str: &str = r###"
    Some number of number signs # followed by a quotation mark ".

    Quotation marks can be used anywhere inside: """"""""",
    as long as one is not followed by a matching number of number signs,
    which ends the string: "###;
```

When parsing a raw string, we have to keep track of how many number signs `#`
occurred before the quotation mark. We can do this using the stack:

```pest
raw_string = {
    "r" ~ PUSH("#"*) ~ "\""    // push the number signs onto the stack
    ~ raw_string_interior
    ~ "\"" ~ POP               // match a quotation mark and the number signs
}
raw_string_interior = {
    (
        !("\"" ~ PEEK)    // unless the next character is a quotation mark
                          // followed by the correct amount of number signs,
        ~ ANY             // consume one character
    )*
}
```

["raw string literals"]: https://doc.rust-lang.org/book/second-edition/appendix-02-operators.html#non-operator-symbols

### Indentation-Sensitive Languages

In conjunction with some extra helpers, the stack can be used to allow parsing indentation-sensitive languages, such as Python.

The general idea is that you store the leading whitespace on the stack with `PUSH` and then use `PEEK_ALL` to match *all* of the whitespace on subsequent lines.

When exiting an indented block, use `DROP` to remove the stack entry without needing to match it.

An example grammar demonstrating this concept is given here:

```pest
Grammar = { SOI ~ NEWLINE* ~ BlockContent* ~ NEWLINE* ~ EOI }

NewBlock = _{
    // The first line in the block
    PEEK_ALL ~ PUSH("  "+ | "\t"+) ~ BlockContent ~
    // Subsequent lines in the block
    (PEEK_ALL ~ BlockContent)* ~
    // Remove the last layer of indentation from the stack when exiting the block
    DROP
}

BlockName = { ASCII_ALPHA+ }

BlockContent = {
    BlockName ~ (NEWLINE | EOI) ~ NewBlock*
}
```

This matches texts such as the following, whilst preserving indentation structure:

```
Hello
  This
    Is
    An
  Indentation
    Sensitive
      Language
Demonstration
```

# Cheat sheet

| Syntax           | Meaning                           | Syntax                  | Meaning              |
|:----------------:|:---------------------------------:|:-----------------------:|:--------------------:|
| `foo =  { ... }` | [regular rule]                    | `baz = @{ ... }`        | [atomic]             |
| `bar = _{ ... }` | [silent]                          | `qux = ${ ... }`        | [compound-atomic]    |
| `#tag = ...`     | [tags]                            | `plugh = !{ ... }`      | [non-atomic]         |
| `"abc"`          | [exact string]                    | `^"abc"`                | [case insensitive]   |
| `'a'..'z'`       | [character range]                 | `ANY`                   | [any character]      |
| `foo ~ bar`      | [sequence]                        | <code>baz \| qux</code> | [ordered choice]     |
| `foo*`           | [zero or more]                    | `bar+`                  | [one or more]        |
| `baz?`           | [optional]                        | `qux{n}`                | [exactly *n*]        |
| `qux{m, n}`      | [between *m* and *n* (inclusive)] |                         |                      |
| `&foo`           | [positive predicate]              | `!bar`                  | [negative predicate] |
| `PUSH(baz)`      | [match and push]                  |                         |                      |
| `POP`            | [match and pop]                   | `PEEK`                  | [match without pop]  |
| `DROP`           | [pop without matching]            | `PEEK_ALL`              | [match entire stack] |

[regular rule]: #syntax-of-pest-parsers
[silent]: #silent-and-atomic-rules
[atomic]: #atomic
[compound-atomic]: #atomic
[non-atomic]: #non-atomic
[tags]: #tags
[exact string]: #terminals
[case insensitive]: #terminals
[character range]: #terminals
[any character]: #terminals
[sequence]: #sequence
[ordered choice]: #ordered-choice
[zero or more]: #repetition
[one or more]: #repetition
[optional]: #repetition
[exactly *n*]: #repetition
[between *m* and *n* (inclusive)]: #repetition
[positive predicate]: #predicates
[negative predicate]: #predicates
[match and push]: #the-stack-wip
[match and pop]: #the-stack-wip
[match without pop]: #the-stack-wip
[pop without matching]: #indentation-sensitive-languages
[match entire stack]: #indentation-sensitive-languages

```

---
## File: `src/grammars/built-ins.md`
*(Relative Path: `src/grammars/built-ins.md`)*

```markdown
# Built-in rules

Besides `ANY`, matching any single Unicode character, `pest` provides several
rules to make parsing text more convenient.

## ASCII rules

Among the printable ASCII characters, it is often useful to match alphabetic
characters and numbers. For **numbers**, `pest` provides digits in common
radixes (bases):

| Built-in rule         | Equivalent                                    |
|:---------------------:|:---------------------------------------------:|
| `ASCII_DIGIT`         | `'0'..'9'`                                    |
| `ASCII_NONZERO_DIGIT` | `'1'..'9'`                                    |
| `ASCII_BIN_DIGIT`     | `'0'..'1'`                                    |
| `ASCII_OCT_DIGIT`     | `'0'..'7'`                                    |
| `ASCII_HEX_DIGIT`     | <code>'0'..'9' \| 'a'..'f' \| 'A'..'F'</code> |

For **alphabetic** characters, distinguishing between uppercase and lowercase:

| Built-in rule       | Equivalent                        |
|:-------------------:|:---------------------------------:|
| `ASCII_ALPHA_LOWER` | `'a'..'z'`                        |
| `ASCII_ALPHA_UPPER` | `'A'..'Z'`                        |
| `ASCII_ALPHA`       | <code>'a'..'z' \| 'A'..'Z'</code> |

And for **miscellaneous** use:

| Built-in rule        | Meaning              | Equivalent                              |
|:--------------------:|:--------------------:|:---------------------------------------:|
| `ASCII`              | any ascii character  | <code>'\u{00}'..'\u{7F}'</code>         |
| `ASCII_ALPHANUMERIC` | any digit or letter  | <code>ASCII_DIGIT \| ASCII_ALPHA</code> |
| `NEWLINE`            | any line feed format | <code>"\n" \| "\r\n" \| "\r"</code>     |

## Unicode rules

To make it easier to correctly parse arbitrary Unicode text, `pest` includes a
large number of rules corresponding to Unicode character properties. These
rules are divided into **general category** and **binary property** rules.

Unicode characters are partitioned into categories based on their general
purpose. Every character belongs to a single category, in the same way that
every ASCII character is a control character, a digit, a letter, a symbol, or a
space.

In addition, every Unicode character has a list of binary properties (true or
false) that it does or does not satisfy. Characters can belong to any number of
these properties, depending on their meaning.

For example, the character "A", "Latin capital letter A", is in the general
category "Uppercase Letter" because its general purpose is being a letter. It
has the binary property "Uppercase" but not "Emoji". By contrast, the character
"&#x1F170;", "negative squared Latin capital letter A", is in the general
category "Other Symbol" because it does not generally occur as a letter in
text. It has both the binary properties "Uppercase" and "Emoji".

For more details, consult Chapter 4 of [The Unicode Standard].

[The Unicode Standard]: https://www.unicode.org/versions/latest/

### General categories

Formally, categories are non-overlapping: each Unicode character belongs to
exactly one category, and no category contains another. However, since certain
groups of categories are often useful together, `pest` exposes the hierarchy of
categories below. For example, the rule `CASED_LETTER` is not technically a
Unicode general category; it instead matches characters that are
`UPPERCASE_LETTER` or `LOWERCASE_LETTER`, which *are* general categories.

- `LETTER`
  - `CASED_LETTER`
    - `UPPERCASE_LETTER`
    - `LOWERCASE_LETTER`
  - `TITLECASE_LETTER`
  - `MODIFIER_LETTER`
  - `OTHER_LETTER`
- `MARK`
  - `NONSPACING_MARK`
  - `SPACING_MARK`
  - `ENCLOSING_MARK`
- `NUMBER`
  - `DECIMAL_NUMBER`
  - `LETTER_NUMBER`
  - `OTHER_NUMBER`
- `PUNCTUATION`
  - `CONNECTOR_PUNCTUATION`
  - `DASH_PUNCTUATION`
  - `OPEN_PUNCTUATION`
  - `CLOSE_PUNCTUATION`
  - `INITIAL_PUNCTUATION`
  - `FINAL_PUNCTUATION`
  - `OTHER_PUNCTUATION`
- `SYMBOL`
  - `MATH_SYMBOL`
  - `CURRENCY_SYMBOL`
  - `MODIFIER_SYMBOL`
  - `OTHER_SYMBOL`
- `SEPARATOR`
  - `SPACE_SEPARATOR`
  - `LINE_SEPARATOR`
  - `PARAGRAPH_SEPARATOR`
- `OTHER`
  - `CONTROL`
  - `FORMAT`
  - `SURROGATE`
  - `PRIVATE_USE`
  - `UNASSIGNED`

### Binary properties

Many of these properties are used to define Unicode text algorithms, such as
[the bidirectional algorithm] and [the text segmentation algorithm]. Such
properties are not likely to be useful for most parsers.

However, the properties `XID_START` and `XID_CONTINUE` are particularly notable
because they are defined "to assist in the standard treatment of identifiers",
"such as programming language variables". See [Technical Report 31] for more
details.

[the bidirectional algorithm]: https://www.unicode.org/reports/tr9/
[the text segmentation algorithm]: https://www.unicode.org/reports/tr29/
[Technical Report 31]: https://www.unicode.org/reports/tr31/

- `ALPHABETIC`
- `BIDI_CONTROL`
- `BIDI_MIRRORED`
- `CASE_IGNORABLE`
- `CASED`
- `CHANGES_WHEN_CASEFOLDED`
- `CHANGES_WHEN_CASEMAPPED`
- `CHANGES_WHEN_LOWERCASED`
- `CHANGES_WHEN_TITLECASED`
- `CHANGES_WHEN_UPPERCASED`
- `DASH`
- `DEFAULT_IGNORABLE_CODE_POINT`
- `DEPRECATED`
- `DIACRITIC`
- `EMOJI`
- `EMOJI_COMPONENT`
- `EMOJI_MODIFIER`
- `EMOJI_MODIFIER_BASE`
- `EMOJI_PRESENTATION`
- `EXTENDED_PICTOGRAPHIC`
- `EXTENDER`
- `GRAPHEME_BASE`
- `GRAPHEME_EXTEND`
- `GRAPHEME_LINK`
- `HEX_DIGIT`
- `HYPHEN`
- `IDS_BINARY_OPERATOR`
- `IDS_TRINARY_OPERATOR`
- `ID_CONTINUE`
- `ID_START`
- `IDEOGRAPHIC`
- `JOIN_CONTROL`
- `LOGICAL_ORDER_EXCEPTION`
- `LOWERCASE`
- `MATH`
- `NONCHARACTER_CODE_POINT`
- `OTHER_ALPHABETIC`
- `OTHER_DEFAULT_IGNORABLE_CODE_POINT`
- `OTHER_GRAPHEME_EXTEND`
- `OTHER_ID_CONTINUE`
- `OTHER_ID_START`
- `OTHER_LOWERCASE`
- `OTHER_MATH`
- `OTHER_UPPERCASE`
- `PATTERN_SYNTAX`
- `PATTERN_WHITE_SPACE`
- `PREPENDED_CONCATENATION_MARK`
- `QUOTATION_MARK`
- `RADICAL`
- `REGIONAL_INDICATOR`
- `SENTENCE_TERMINAL`
- `SOFT_DOTTED`
- `TERMINAL_PUNCTUATION`
- `UNIFIED_IDEOGRAPH`
- `UPPERCASE`
- `VARIATION_SELECTOR`
- `WHITE_SPACE`
- `XID_CONTINUE`
- `XID_START`


### Script properties

The [Unicode script property](https://unicode.org/standard/supported.html)
has included built-in rules for matching characters in particular languages.

**For example:**

We want match a string that contains any CJK (regexp: `\p{CJK}`) characters such as `你好世界` or `こんにちは世界` or `안녕하세요 세계`.

- `HAN`: representing Chinese characters, including Simplified Chinese, Traditional Chinese, Japanese kanji, and Korean hanja.
- `HIRAGANA`: representing the Japanese hiragana syllabary.
- `KATAKANA`: representing the Japanese katakana syllabary.
- `HANGUL`: representing Korean alphabetical characters.
- `BOPOMOFO`: representing Chinese phonetic symbols.

So we define a rule named `CJK` like this:

```pest
CJK = { HAN | HIRAGANA | KATAKANA | HANGUL | BOPOMOFO }
```

**All available rules:**

- `ADLAM`
- `AHOM`
- `ANATOLIAN_HIEROGLYPHS`
- `ARABIC`
- `ARMENIAN`
- `AVESTAN`
- `BALINESE`
- `BAMUM`
- `BASSA_VAH`
- `BATAK`
- `BENGALI`
- `BHAIKSUKI`
- `BOPOMOFO`
- `BRAHMI`
- `BRAILLE`
- `BUGINESE`
- `BUHID`
- `CANADIAN_ABORIGINAL`
- `CARIAN`
- `CAUCASIAN_ALBANIAN`
- `CHAKMA`
- `CHAM`
- `CHEROKEE`
- `CHORASMIAN`
- `COMMON`
- `COPTIC`
- `CUNEIFORM`
- `CYPRIOT`
- `CYPRO_MINOAN`
- `CYRILLIC`
- `DESERET`
- `DEVANAGARI`
- `DIVES_AKURU`
- `DOGRA`
- `DUPLOYAN`
- `EGYPTIAN_HIEROGLYPHS`
- `ELBASAN`
- `ELYMAIC`
- `ETHIOPIC`
- `GEORGIAN`
- `GLAGOLITIC`
- `GOTHIC`
- `GRANTHA`
- `GREEK`
- `GUJARATI`
- `GUNJALA_GONDI`
- `GURMUKHI`
- `HAN`
- `HANGUL`
- `HANIFI_ROHINGYA`
- `HANUNOO`
- `HATRAN`
- `HEBREW`
- `HIRAGANA`
- `IMPERIAL_ARAMAIC`
- `INHERITED`
- `INSCRIPTIONAL_PAHLAVI`
- `INSCRIPTIONAL_PARTHIAN`
- `JAVANESE`
- `KAITHI`
- `KANNADA`
- `KATAKANA`
- `KAWI`
- `KAYAH_LI`
- `KHAROSHTHI`
- `KHITAN_SMALL_SCRIPT`
- `KHMER`
- `KHOJKI`
- `KHUDAWADI`
- `LAO`
- `LATIN`
- `LEPCHA`
- `LIMBU`
- `LINEAR_A`
- `LINEAR_B`
- `LISU`
- `LYCIAN`
- `LYDIAN`
- `MAHAJANI`
- `MAKASAR`
- `MALAYALAM`
- `MANDAIC`
- `MANICHAEAN`
- `MARCHEN`
- `MASARAM_GONDI`
- `MEDEFAIDRIN`
- `MEETEI_MAYEK`
- `MENDE_KIKAKUI`
- `MEROITIC_CURSIVE`
- `MEROITIC_HIEROGLYPHS`
- `MIAO`
- `MODI`
- `MONGOLIAN`
- `MRO`
- `MULTANI`
- `MYANMAR`
- `NABATAEAN`
- `NAG_MUNDARI`
- `NANDINAGARI`
- `NEW_TAI_LUE`
- `NEWA`
- `NKO`
- `NUSHU`
- `NYIAKENG_PUACHUE_HMONG`
- `OGHAM`
- `OL_CHIKI`
- `OLD_HUNGARIAN`
- `OLD_ITALIC`
- `OLD_NORTH_ARABIAN`
- `OLD_PERMIC`
- `OLD_PERSIAN`
- `OLD_SOGDIAN`
- `OLD_SOUTH_ARABIAN`
- `OLD_TURKIC`
- `OLD_UYGHUR`
- `ORIYA`
- `OSAGE`
- `OSMANYA`
- `PAHAWH_HMONG`
- `PALMYRENE`
- `PAU_CIN_HAU`
- `PHAGS_PA`
- `PHOENICIAN`
- `PSALTER_PAHLAVI`
- `REJANG`
- `RUNIC`
- `SAMARITAN`
- `SAURASHTRA`
- `SHARADA`
- `SHAVIAN`
- `SIDDHAM`
- `SIGNWRITING`
- `SINHALA`
- `SOGDIAN`
- `SORA_SOMPENG`
- `SOYOMBO`
- `SUNDANESE`
- `SYLOTI_NAGRI`
- `SYRIAC`
- `TAGALOG`
- `TAGBANWA`
- `TAI_LE`
- `TAI_THAM`
- `TAI_VIET`
- `TAKRI`
- `TAMIL`
- `TANGSA`
- `TANGUT`
- `TELUGU`
- `THAANA`
- `THAI`
- `TIBETAN`
- `TIFINAGH`
- `TIRHUTA`
- `TOTO`
- `UGARITIC`
- `VAI`
- `VITHKUQI`
- `WANCHO`
- `WARANG_CITI`
- `YEZIDI`
- `YI`
- `ZANABAZAR_SQUARE`

```

---
## File: `src/examples/jlang.md`
*(Relative Path: `src/examples/jlang.md`)*

```markdown
# Example: The J language

The J language is an array programming language influenced by APL.
In J, operations on individual numbers (`2 * 3`) can just as easily 
be applied to entire lists of numbers (`2 * 3 4 5`, returning `6 8 10`).

Operators in J are referred to as *verbs*.
Verbs are either *monadic* (taking a single argument, such as `*: 3`, "3 squared")
or *dyadic* (taking two arguments, one on either side, such as `5 - 4`, "5 minus 4").

Here's an example of a J program:

```j
'A string'

*: 1 2 3 4

matrix =: 2 3 $ 5 + 2 3 4 5 6 7
10 * matrix

1 + 10 20 30
1 2 3 + 10

residues =: 2 | 0 1 2 3 4 5 6 7
residues
```

Using J's [interpreter] to run the above program
yields the following on standard out:

```
A string

1 4 9 16

 70  80  90
100 110 120

11 21 31
11 12 13

0 1 0 1 0 1 0 1
```

In this section we'll write a grammar for a subset of J. We'll then walk 
through a parser that builds an AST by iterating over the rules that 
`pest` gives us. You can find the full source code
[within this book's repository].

## The grammar

We'll build up a grammar section by section, starting with
the program rule:

```pest
program = _{ SOI ~ "\n"* ~ (stmt ~ "\n"+) * ~ stmt? ~ EOI }
```

Each J program contains statements delimited by one or more newlines.
Notice the leading underscore, which tells `pest` to [silence] the `program`
rule &mdash; we don't want `program` to appear as a token in the parse stream,
we want the underlying statements instead.

A statement is simply an expression, and since there's only one such 
possibility, we also [silence] this `stmt` rule as well, and thus our 
parser will receive an iterator of underlying `expr`s:

```pest
stmt = _{ expr }
```

An expression can be an assignment to a variable identifier, a monadic
expression, a dyadic expression, a single string, or an array of terms:

```pest
expr = {
      assgmtExpr
    | monadicExpr
    | dyadicExpr
    | string
    | terms
}
```

A monadic expression consists of a verb with its sole operand on the right;
a dyadic expression has operands on either side of the verb.
Assignment expressions associate identifiers with expressions.

In J, there is no operator precedence &mdash; evaluation is right-associative
(proceeding from right to left), with parenthesized expressions evaluated
first.

```pest
monadicExpr = { verb ~ expr }

dyadicExpr = { (monadicExpr | terms) ~ verb ~ expr }

assgmtExpr = { ident ~ "=:" ~ expr }
```

A list of terms should contain at least one decimal, integer, 
identifier, or parenthesized expression; we care only about those 
underlying values, so we make the `term` rule [silent] with a leading 
underscore:

```pest
terms = { term+ }

term = _{ decimal | integer | ident | "(" ~ expr ~ ")" }
```

A few of J's verbs are defined in this grammar;
J's [full vocabulary] is much more extensive.

```pest
verb = {
    ">:" | "*:" | "-"  | "%" | "#" | ">."
  | "+"  | "*"  | "<"  | "=" | "^" | "|"
  | ">"  | "$"
}
```

Now we can get into lexing rules. Numbers in J are represented as 
usual, with the exception that negatives are represented using a 
leading `_` underscore (because `-` is a verb that performs negation 
as a monad and subtraction as a dyad).  Identifiers in J must start 
with a letter, but can contain numbers thereafter. Strings are 
surrounded by single quotes; quotes themselves can be embedded by 
escaping them with an additional quote.

Notice how we use `pest`'s `@` modifier to make each of these rules [atomic],
meaning [implicit whitespace] is forbidden, and
that interior rules (i.e., `ASCII_ALPHA` in `ident`) become [silent] &mdash;
when our parser receives any of these tokens, they will be terminal:

```pest
integer = @{ "_"? ~ ASCII_DIGIT+ }

decimal = @{ "_"? ~ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT* }

ident = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

string = @{ "'" ~ ( "''" | (!"'" ~ ANY) )* ~ "'" }
```

Whitespace in J consists solely of spaces and tabs. Newlines are
significant because they delimit statements, so they are excluded
from this rule:

```pest
WHITESPACE = _{ " " | "\t" }
```

Finally, we must handle comments. Comments in J start with `NB.` and 
continue to the end of the line on which they are found. Critically, we must 
not consume the newline at the end of the comment line; this is needed 
to separate any statement that might precede the comment from the statement 
on the succeeding line.

```pest
COMMENT = _{ "NB." ~ (!"\n" ~ ANY)* }
```

## Parsing and AST generation

This section will walk through a parser that uses the grammar above.
Library includes and self-explanatory code are omitted here; you can find 
the parser in its entirety [within this book's repository].

First we'll enumerate the verbs defined in our grammar, distinguishing between 
monadic and dyadic verbs. These enumerations will be be used as labels 
in our AST:

```rust
pub enum MonadicVerb {
    Increment,
    Square,
    Negate,
    Reciprocal,
    Tally,
    Ceiling,
    ShapeOf,
}

pub enum DyadicVerb {
    Plus,
    Times,
    LessThan,
    LargerThan,
    Equal,
    Minus,
    Divide,
    Power,
    Residue,
    Copy,
    LargerOf,
    LargerOrEqual,
    Shape,
}
```

Then we'll enumerate the various kinds of AST nodes:

```rust
pub enum AstNode {
    Print(Box<AstNode>),
    Integer(i32),
    DoublePrecisionFloat(f64),
    MonadicOp {
        verb: MonadicVerb,
        expr: Box<AstNode>,
    },
    DyadicOp {
        verb: DyadicVerb,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    Terms(Vec<AstNode>),
    IsGlobal {
        ident: String,
        expr: Box<AstNode>,
    },
    Ident(String),
    Str(CString),
}
```

To parse top-level statements in a J program, we have the following 
`parse` function that accepts a J program in string form and passes it 
to `pest` for parsing. We get back a sequence of [`Pair`]s. As specified
in the grammar, a statement can only consist of an expression, so the `match` 
below parses each of those top-level expressions and wraps them in a `Print` 
AST node in keeping with the J interpreter's REPL behavior:

```rust
pub fn parse(source: &str) -> Result<Vec<AstNode>, Error<Rule>> {
    let mut ast = vec![];

    let pairs = JParser::parse(Rule::program, source)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::expr => {
                ast.push(Print(Box::new(build_ast_from_expr(pair))));
            }
            _ => {}
        }
    }

    Ok(ast)
}
```

AST nodes are built from expressions by walking the [`Pair`] iterator in
lockstep with the expectations set out in our grammar file. Common behaviors 
are abstracted out into separate functions, such as `parse_monadic_verb`
and `parse_dyadic_verb`, and [`Pair`]s representing expressions themselves are
passed in recursive calls to `build_ast_from_expr`:

```rust
fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
        Rule::monadicExpr => {
            let mut pair = pair.into_inner();
            let verb = pair.next().unwrap();
            let expr = pair.next().unwrap();
            let expr = build_ast_from_expr(expr);
            parse_monadic_verb(verb, expr)
        }
        // ... other cases elided here ...
    }
}
```

Dyadic verbs are mapped from their string representations to AST nodes in 
a straightforward way:

```rust
fn parse_dyadic_verb(pair: pest::iterators::Pair<Rule>, lhs: AstNode, rhs: AstNode) -> AstNode {
    AstNode::DyadicOp {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        verb: match pair.as_str() {
            "+" => DyadicVerb::Plus,
            "*" => DyadicVerb::Times,
            "-" => DyadicVerb::Minus,
            "<" => DyadicVerb::LessThan,
            "=" => DyadicVerb::Equal,
            ">" => DyadicVerb::LargerThan,
            "%" => DyadicVerb::Divide,
            "^" => DyadicVerb::Power,
            "|" => DyadicVerb::Residue,
            "#" => DyadicVerb::Copy,
            ">." => DyadicVerb::LargerOf,
            ">:" => DyadicVerb::LargerOrEqual,
            "$" => DyadicVerb::Shape,
            _ => panic!("Unexpected dyadic verb: {}", pair.as_str()),
        },
    }
}
```

As are monadic verbs:

```rust
fn parse_monadic_verb(pair: pest::iterators::Pair<Rule>, expr: AstNode) -> AstNode {
    AstNode::MonadicOp {
        verb: match pair.as_str() {
            ">:" => MonadicVerb::Increment,
            "*:" => MonadicVerb::Square,
            "-" => MonadicVerb::Negate,
            "%" => MonadicVerb::Reciprocal,
            "#" => MonadicVerb::Tally,
            ">." => MonadicVerb::Ceiling,
            "$" => MonadicVerb::ShapeOf,
            _ => panic!("Unsupported monadic verb: {}", pair.as_str()),
        },
        expr: Box::new(expr),
    }
}
```

Finally, we define a function to process terms such as numbers and strings. 
Numbers require some manuevering to handle J's leading underscores 
representing negation, but other than that the process is typical:

```rust
fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::integer => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "_" => (-1, &istr[1..]),
                _ => (1, &istr[..]),
            };
            let integer: i32 = istr.parse().unwrap();
            AstNode::Integer(sign * integer)
        }
        Rule::decimal => {
            let dstr = pair.as_str();
            let (sign, dstr) = match &dstr[..1] {
                "_" => (-1.0, &dstr[1..]),
                _ => (1.0, &dstr[..]),
            };
            let mut flt: f64 = dstr.parse().unwrap();
            if flt != 0.0 {
                // Avoid negative zeroes; only multiply sign by nonzeroes.
                flt *= sign;
            }
            AstNode::DoublePrecisionFloat(flt)
        }
        Rule::expr => build_ast_from_expr(pair),
        Rule::ident => AstNode::Ident(String::from(pair.as_str())),
        unknown_term => panic!("Unexpected term: {:?}", unknown_term),
    }
}
```

## Running the Parser

We can now define a `main` function to pass J programs to our 
`pest`-enabled parser:

```rust
fn main() {
    let unparsed_file = std::fs::read_to_string("example.ijs")
      .expect("cannot read ijs file");
    let astnode = parse(&unparsed_file).expect("unsuccessful parse");
    println!("{:?}", &astnode);
}
```

Using this code in `example.ijs`:

```j
_2.5 ^ 3
*: 4.8
title =: 'Spinning at the Boundary'
*: _1 2 _3 4
1 2 3 + 10 20 30
1 + 10 20 30
1 2 3 + 10
2 | 0 1 2 3 4 5 6 7
another =: 'It''s Escaped'
3 | 0 1 2 3 4 5 6 7
(2+1)*(2+2)
3 * 2 + 1
1 + 3 % 4
x =: 100
x - 1
y =: x - 1
y
```

We'll get the following abstract syntax tree on stdout when we run 
the parser:

```shell
$ cargo run
  [ ... ]
[Print(DyadicOp { verb: Power, lhs: DoublePrecisionFloat(-2.5),
    rhs: Integer(3) }),
Print(MonadicOp { verb: Square, expr: DoublePrecisionFloat(4.8) }),
Print(IsGlobal { ident: "title", expr: Str("Spinning at the Boundary") }),
Print(MonadicOp { verb: Square, expr: Terms([Integer(-1), Integer(2),
    Integer(-3), Integer(4)]) }),
Print(DyadicOp { verb: Plus, lhs: Terms([Integer(1), Integer(2), Integer(3)]),
    rhs: Terms([Integer(10), Integer(20), Integer(30)]) }),
Print(DyadicOp { verb: Plus, lhs: Integer(1), rhs: Terms([Integer(10),
    Integer(20), Integer(30)]) }),
Print(DyadicOp { verb: Plus, lhs: Terms([Integer(1), Integer(2), Integer(3)]),
    rhs: Integer(10) }),
Print(DyadicOp { verb: Residue, lhs: Integer(2),
    rhs: Terms([Integer(0), Integer(1), Integer(2), Integer(3), Integer(4),
    Integer(5), Integer(6), Integer(7)]) }),
Print(IsGlobal { ident: "another", expr: Str("It\'s Escaped") }),
Print(DyadicOp { verb: Residue, lhs: Integer(3), rhs: Terms([Integer(0),
    Integer(1), Integer(2), Integer(3), Integer(4), Integer(5),
    Integer(6), Integer(7)]) }),
Print(DyadicOp { verb: Times, lhs: DyadicOp { verb: Plus, lhs: Integer(2),
    rhs: Integer(1) }, rhs: DyadicOp { verb: Plus, lhs: Integer(2),
        rhs: Integer(2) } }),
Print(DyadicOp { verb: Times, lhs: Integer(3), rhs: DyadicOp { verb: Plus,
    lhs: Integer(2), rhs: Integer(1) } }),
Print(DyadicOp { verb: Plus, lhs: Integer(1), rhs: DyadicOp { verb: Divide,
    lhs: Integer(3), rhs: Integer(4) } }),
Print(IsGlobal { ident: "x", expr: Integer(100) }),
Print(DyadicOp { verb: Minus, lhs: Ident("x"), rhs: Integer(1) }),
Print(IsGlobal { ident: "y", expr: DyadicOp { verb: Minus, lhs: Ident("x"),
    rhs: Integer(1) } }),
Print(Ident("y"))]
```

[J language]: https://jsoftware.com/
[interpreter]: https://jsoftware.com/
[full vocabulary]: https://code.jsoftware.com/wiki/NuVoc
[implicit whitespace]: ../grammars/syntax.md#implicit-whitespace
[atomic]: ../grammars/syntax.md#atomic
[silence]: ../grammars/syntax.md#silent-and-atomic-rules
[silent]: ../grammars/syntax.md#silent-and-atomic-rules
[`Pair`]: ../parser_api.md#pairs
[within this book's repository]: https://github.com/pest-parser/book/tree/master/examples/jlang-parser

```

---
## File: `src/examples/ini.md`
*(Relative Path: `src/examples/ini.md`)*

```markdown
# Example: INI

INI (short for *initialization*) files are simple configuration files. Since
there is no standard for the format, we'll write a program that is able to
parse this example file:

```ini
username = noha
password = plain_text
salt = NaCl

[server_1]
interface=eth0
ip=127.0.0.1
document_root=/var/www/example.org

[empty_section]

[second_server]
document_root=/var/www/example.com
ip=
interface=eth1
```

Each line contains a **key and value** separated by an equals sign; or contains
a **section name** surrounded by square brackets; or else is **blank** and has
no meaning.

Whenever a section name appears, the following keys and values belong to that
section, until the next section name. The key&ndash;value pairs at the
beginning of the file belong to an implicit "empty" section.

## Writing the grammar

Start by [initializing a new project] using Cargo, adding the dependencies
`pest = "2.6"` and `pest_derive = "2.6"`. Make a new file, `src/ini.pest`, to
hold the grammar.

The text of interest in our file &mdash; `username`, `/var/www/example.org`,
*etc.* &mdash; consists of only a few characters. Let's make a rule to
recognize a single character in that set. The built-in rule
`ASCII_ALPHANUMERIC` is a shortcut to represent any uppercase or lowercase
ASCII letter, or any digit.

```pest
char = { ASCII_ALPHANUMERIC | "." | "_" | "/" }
```

Section names and property keys *must not* be empty, but property values *may*
be empty (as in the line `ip=` above). That is, the former consist of one or
more characters, `char+`; and the latter consist of zero or more characters,
`char*`. We separate the meaning into two rules:

```pest
name = { char+ }
value = { char* }
```

Now it's easy to express the two kinds of input lines.

```pest
section = { "[" ~ name ~ "]" }
property = { name ~ "=" ~ value }
```

Finally, we need a rule to represent an entire input file. The expression
`(section | property)?` matches `section`, `property`, or else nothing. Using
the built-in rule `NEWLINE` to match line endings:

```pest
file = {
    SOI ~
    ((section | property)? ~ NEWLINE)* ~
    EOI
}
```

To compile the parser into Rust, we need to add the following to `src/main.rs`:

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ini.pest"]
pub struct INIParser;
```

## Program initialization

Now we can read the file and parse it with `pest`:

```rust
use std::collections::HashMap;
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("config.ini").expect("cannot read file");

    let file = INIParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next().unwrap(); // get and unwrap the `file` rule; never fails

    // ...
}
```

We'll express the properties list using nested [`HashMap`]s. The outer hash map
will have section names as keys and section contents (inner hash maps) as
values. Each inner hash map will have property keys and property values. For
example, to access the `document_root` of `server_1`, we could write
`properties["server_1"]["document_root"]`. The implicit "empty" section will be
represented by a regular section with an empty string `""` for the name, so
that `properties[""]["salt"]` is valid.

```rust
fn main() {
    // ...

    let mut properties: HashMap<&str, HashMap<&str, &str>> = HashMap::new();

    // ...
}
```

Note that the hash map keys and values are all `&str`, borrowed strings. `pest`
parsers do not copy the input they parse; they borrow it. All methods for
inspecting a parse result return strings which are borrowed from the original
parsed string.

## The main loop

Now we interpret the parse result. We loop through each line of the file, which
is either a section name or a key&ndash;value property pair. If we encounter a
section name, we update a variable. If we encounter a property pair, we obtain
a reference to the hash map for the current section, then insert the pair into
that hash map.

```rust
    // ...

    let mut current_section_name = "";

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::section => {
                let mut inner_rules = line.into_inner(); // { name }
                current_section_name = inner_rules.next().unwrap().as_str();
            }
            Rule::property => {
                let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                let name: &str = inner_rules.next().unwrap().as_str();
                let value: &str = inner_rules.next().unwrap().as_str();

                // Insert an empty inner hash map if the outer hash map hasn't
                // seen this section name before.
                let section = properties.entry(current_section_name).or_default();
                section.insert(name, value);
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    // ...
```

For output, let's simply dump the hash map using [the pretty-printed `Debug`
format].

```rust
fn main() {
    // ...

    println!("{:#?}", properties);
}
```

## Whitespace

If you copy the example INI file at the top of this chapter into a file
`config.ini` and run the program, it will not parse. We have forgotten about
the optional spaces around equals signs!

Handling whitespace can be inconvenient for large grammars. Explicitly writing
a `whitespace` rule and manually inserting it makes a grammar difficult to read
and modify. `pest` provides a solution using [the special rule `WHITESPACE`].
If defined, it will be implicitly run, as many times as possible, at every
tilde `~` and between every repetition (for example, `*` and `+`). For our INI
parser, only spaces are legal whitespace.

```pest
WHITESPACE = _{ " " }
```

We mark the `WHITESPACE` rule [*silent*] with a leading low line (underscore)
`_{ ... }`. This way, even if it matches, it won't show up inside other rules.
If it weren't silent, parsing would be much more complicated, since every call
to `Pairs::next(...)` could potentially return `Rule::WHITESPACE` instead of
the desired next regular rule.

But wait! Spaces shouldn't be allowed in section names, keys, or values!
Currently, whitespace is automatically inserted between characters in `name = {
char+ }`. Rules that *are* whitespace-sensitive need to be marked [*atomic*]
with a leading at sign `@{ ... }`. In atomic rules, automatic whitespace
handling is disabled, and interior rules are silent.

```pest
name = @{ char+ }
value = @{ char* }
```

## Done

Try it out! Make sure that the file `config.ini` exists, then run the program!
You should see something like this:

```shell
$ cargo run
  [ ... ]
{
    "": {
        "password": "plain_text",
        "username": "noha",
        "salt": "NaCl"
    },
    "second_server": {
        "ip": "",
        "document_root": "/var/www/example.com",
        "interface": "eth1"
    },
    "server_1": {
        "interface": "eth0",
        "document_root": "/var/www/example.org",
        "ip": "127.0.0.1"
    }
}
```

[initializing a new project]: csv.md#setup
[`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
[the pretty-printed `Debug` format]: https://doc.rust-lang.org/std/fmt/index.html#sign0
[the special rule `WHITESPACE`]: ../grammars/syntax.md#implicit-whitespace
[*silent*]: ../grammars/syntax.md#silent-and-atomic-rules
[*atomic*]: ../grammars/syntax.md#atomic

```

---
## File: `src/examples/json.md`
*(Relative Path: `src/examples/json.md`)*

```markdown
# Example: JSON

[JSON] is a popular format for data serialization that is derived from the
syntax of JavaScript. JSON documents are tree-like and potentially recursive
&mdash; two data types, *objects* and *arrays*, can contain other values,
including other objects and arrays.

Here is an example JSON document:

```json
{
    "nesting": { "inner object": {} },
    "an array": [1.5, true, null, 1e-6],
    "string with escaped double quotes" : "\"quick brown foxes\""
}
```

Let's write a program that **parses** the JSON to an Rust object, known as an
*abstract syntax tree*, then **serializes** the AST back to JSON.

## Setup

We'll start by defining the AST in Rust. Each JSON data type is represented by
an enum variant.

```rust
enum JSONValue<'a> {
    Object(Vec<(&'a str, JSONValue<'a>)>),
    Array(Vec<JSONValue<'a>>),
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}
```

To avoid copying when deserializing strings, `JSONValue` borrows strings from
the original unparsed JSON. In order for this to work, we cannot interpret
string escape sequences: the input string `"\n"` will be represented by
`JSONValue::String("\\n")`, a Rust string with two characters, even though it
represents a JSON string with just one character.

Let's move on to the serializer. For the sake of clarity, it uses allocated
`String`s instead of providing an implementation of [`std::fmt::Display`],
which would be more idiomatic.

```rust
fn serialize_jsonvalue(val: &JSONValue) -> String {
    use JSONValue::*;

    match val {
        Object(o) => {
            let contents: Vec<_> = o
                .iter()
                .map(|(name, value)|
                     format!("\"{}\":{}", name, serialize_jsonvalue(value)))
                .collect();
            format!("{{{}}}", contents.join(","))
        }
        Array(a) => {
            let contents: Vec<_> = a.iter().map(serialize_jsonvalue).collect();
            format!("[{}]", contents.join(","))
        }
        String(s) => format!("\"{}\"", s),
        Number(n) => format!("{}", n),
        Boolean(b) => format!("{}", b),
        Null => format!("null"),
    }
}
```

Note that the function invokes itself recursively in the `Object` and `Array`
cases. This pattern appears throughout the parser. The AST creation function
iterates recursively through the parse result, and the grammar has rules which
include themselves.

## Writing the grammar

Let's begin with whitespace. JSON whitespace can appear anywhere, except inside
strings (where it must be parsed separately) and between digits in numbers
(where it is not allowed). This makes it a good fit for `pest`'s [implicit
whitespace]. In `src/json.pest`:

```pest
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
```

[The JSON specification] includes diagrams for parsing JSON strings. We can
write the grammar directly from that page. Let's write `object` as a sequence
of `pair`s separated by commas `,`.

```pest
object = {
    "{" ~ "}" |
    "{" ~ pair ~ ("," ~ pair)* ~ "}"
}
pair = { string ~ ":" ~ value }

array = {
    "[" ~ "]" |
    "[" ~ value ~ ("," ~ value)* ~ "]"
}
```

The `object` and `array` rules show how to parse a potentially empty list with
separators. There are two cases: one for an empty list, and one for a list with
at least one element. This is necessary because a trailing comma in an array,
such as in `[0, 1,]`, is illegal in JSON.

Now we can write `value`, which represents any single data type. We'll mimic
our AST by writing `boolean` and `null` as separate rules.

```pest
value = _{ object | array | string | number | boolean | null }

boolean = { "true" | "false" }

null = { "null" }
```

Let's separate the logic for strings into three parts. `char` is a rule
matching any logical character in the string, including any backslash escape
sequence. `inner` represents the contents of the string, without the
surrounding double quotes. `string` matches the inner contents of the string,
including the surrounding double quotes.

The `char` rule uses [the idiom `!(...) ~ ANY`], which matches any character
except the ones given in parentheses. In this case, any character is legal
inside a string, except for double quote `"` and backslash <code>\\</code>,
which require separate parsing logic.

```pest
string = ${ "\"" ~ inner ~ "\"" }
inner = @{ char* }
char = {
    !("\"" | "\\") ~ ANY
    | "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t")
    | "\\" ~ ("u" ~ ASCII_HEX_DIGIT{4})
}
```

Because `string` is marked [compound atomic], `string` [token pairs] will also
contain a single `inner` pair. Because `inner` is marked [atomic], no `char`
pairs will appear inside `inner`. Since these rules are atomic, no whitespace
is permitted between separate tokens.

Numbers have four logical parts: an optional sign, an integer part, an optional
fractional part, and an optional exponent. We'll mark `number` atomic so that
whitespace cannot appear between its parts.

```pest
number = @{
    "-"?
    ~ ("0" | ASCII_NONZERO_DIGIT ~ ASCII_DIGIT*)
    ~ ("." ~ ASCII_DIGIT*)?
    ~ (^"e" ~ ("+" | "-")? ~ ASCII_DIGIT+)?
}
```

We need a final rule to represent an entire JSON file. The only legal contents
of a JSON file is a single object or array. We'll mark this rule [silent], so
that a parsed JSON file contains only two token pairs: the parsed value itself,
and [the `EOI` rule].

```pest
json = _{ SOI ~ (object | array) ~ EOI }
```

## AST generation

Let's compile the grammar into Rust.

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "json.pest"]
struct JSONParser;
```

We'll write a function that handles both parsing and AST generation. Users of
the function can call it on an input string, then use the result returned as
either a `JSONValue` or a parse error.

```rust
use pest::error::Error;

fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    let json = JSONParser::parse(Rule::json, file)?.next().unwrap();

    // ...
}
```

Now we need to handle `Pair`s recursively, depending on the rule. We know that
`json` is either an `object` or an `array`, but these values might contain an
`object` or an `array` themselves! The most logical way to handle this is to
write an auxiliary recursive function that parses a `Pair` into a `JSONValue`
directly.

```rust
fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    // ...

    use pest::iterators::Pair;

    fn parse_value(pair: Pair<Rule>) -> JSONValue {
        match pair.as_rule() {
            Rule::object => JSONValue::Object(
                pair.into_inner()
                    .map(|pair| {
                        let mut inner_rules = pair.into_inner();
                        let name = inner_rules
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap()
                            .as_str();
                        let value = parse_value(inner_rules.next().unwrap());
                        (name, value)
                    })
                    .collect(),
            ),
            Rule::array => JSONValue::Array(pair.into_inner().map(parse_value).collect()),
            Rule::string => JSONValue::String(pair.into_inner().next().unwrap().as_str()),
            Rule::number => JSONValue::Number(pair.as_str().parse().unwrap()),
            Rule::boolean => JSONValue::Boolean(pair.as_str().parse().unwrap()),
            Rule::null => JSONValue::Null,
            Rule::json
            | Rule::EOI
            | Rule::pair
            | Rule::value
            | Rule::inner
            | Rule::char
            | Rule::WHITESPACE => unreachable!(),
        }
    }

    // ...
}
```

The `object` and `array` cases deserve special attention. The contents of an
`array` token pair is just a sequence of `value`s. Since we're working with a
Rust iterator, we can simply map each value to its parsed AST node recursively,
then collect them into a `Vec`. For `object`s, the process is similar, except
the iterator is over `pair`s, from which we need to extract names and values
separately.

The `number` and `boolean` cases use Rust's `str::parse` method to convert the
parsed string to the appropriate Rust type. Every legal JSON number can be
parsed directly into a Rust floating point number!

We run `parse_value` on the parse result to finish the conversion.

```rust
fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    // ...

    Ok(parse_value(json))
}
```

## Finishing

Our `main` function is now very simple. First, we read the JSON data from a
file named `data.json`. Next, we parse the file contents into a JSON AST.
Finally, we serialize the AST back into a string and print it.

```rust
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("data.json").expect("cannot read file");

    let json: JSONValue = parse_json_file(&unparsed_file).expect("unsuccessful parse");

    println!("{}", serialize_jsonvalue(&json));
}
```

Try it out! Copy the example document at the top of this chapter into
`data.json`, then run the program! You should see something like this:

```shell
$ cargo run
  [ ... ]
{"nesting":{"inner object":{}},"an array":[1.5,true,null,0.000001],"string with escaped double quotes":"\"quick brown foxes\""}
```

[JSON]: https://json.org/
[`std::fmt::Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[implicit whitespace]: ../grammars/syntax.md#implicit-whitespace
[The JSON specification]: https://json.org/
[the idiom `!(...) ~ ANY`]: ../grammars/syntax.md#predicates
[compound atomic]: ../grammars/syntax.md#atomic
[token pairs]: ../parser_api.md#pairs
[atomic]: ../grammars/syntax.md#atomic
[silent]: ../grammars/syntax.md#silent-and-atomic-rules
[the `EOI` rule]: ../grammars/syntax.md#start-and-end-of-input

```

---
## File: `src/examples/awk.md`
*(Relative Path: `src/examples/awk.md`)*

```markdown
# Final project: Awk clone (WIP)

This chapter will walk through the creation of a simple variant of [Awk] (only
loosely following the POSIX specification). It will probably have several
sections. It will provide an example of a full project based on `pest` with a
manageable grammar, a straightforward AST, and a fairly simple interpreter.

This Awk clone will support regex patterns, string and numeric variables, most
of the POSIX operators, and some functions. It will not support user-defined
functions in the interest of avoiding variable scoping.

[Awk]: http://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html

```

---
## File: `src/examples/calculator.md`
*(Relative Path: `src/examples/calculator.md`)*

```markdown
# Example: Calculator

This example focuses on the practical aspect of using a Pratt parser to parse expressions using `pest`.
To illustrate this, we build a parser for simple equations, and construct an abstract syntax tree.

## Precedence and associativity
In a simple equation multiplication and division are evaluated first, which means they have a higher precedence.
e.g. `1 + 2 * 3` is evaluated as `1 + (2 * 3)`, if the precedence was equal it would be `(1 + 2) * 3`.
For our system we have the following operands:
- highest precedence: multiplication & division
- lowest precedence: addition & subtraction

In the expression `1 + 2 - 3`, no operator is inherently more important than the other.
Addition, subtraction, multiplication and division are evaluated from left to right,
e.g. `1 - 2 + 3` is evaluated as `(1 - 2) + 3`. We call this property left associativity. 
Operators can also be right associative. For example, we usually evaluate the statement `x = y = 1` by first 
assigning `y = 1` and `x = 1` (or `x = y`) afterwards.

Associativity only matters if two operators have the same precedence, as is the case with addition and subtraction for 
example. This means that if we have an expression with only additions and subtractions, we can just evaluate it from 
left to right. `1 + 2 - 3` is equal to `(1 + 2) - 3`. And `1 - 2 + 3` is equal to `(1 - 2) + 3`.

To go from a flat list of operands separated by operators, it suffices to define a precedence and associativity for each 
operator. With these definitions an algorithm such as Pratt parsing is able to construct a corresponding 
expression tree.

If you are curious to know more about how Pratt parsing is implemented, Aleksey Kladov has a
[great tutorial](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) on implementing it
from scratch using Rust.

## Calculator example
We want our calculator to be able to parse simple equations that consist of integers and simple binary operators.
Additionally, we want to support parenthesis and unary minus.
For example:
```
1 + 2 * 3
-(2 + 5) * 16
```

## Grammar
We start by defining our atoms, bits of self-contained syntax that cannot be split up into smaller parts.
For our calculator we start with just simple integers:
```pest
// No whitespace allowed between digits
integer = @{ ASCII_DIGIT+ }

atom = _{ integer }
```

Next, our binary operators:
```pest
bin_op = _{ add | subtract | multiply | divide }
	add = { "+" }
	subtract = { "-" }
	multiply = { "*" }
	divide = { "/" }
```

These two rules will be the input to the
[`PrattParser`](https://docs.rs/pest/latest/pest/pratt_parser/struct.PrattParser.html). 
It expects to receive atoms separated by operators, like so: `atom, bin_op, atom, bin_op, atom, ...`.

Corresponding to this format, we define our rule for expressions:
```pest
expr = { atom ~ (bin_op ~ atom)* }
```

And finally, we define our `WHITESPACE` and equation rule:
```pest
WHITESPACE = _{ " " }

// We can't have SOI and EOI on expr directly, because it is used
// recursively (e.g. with parentheses)
equation = _{ SOI ~ expr ~ EOI }
```

This defines the grammar which generates the required input for the Pratt parser.

## Abstract Syntax Tree
We want to convert our input into an abstract syntax tree.
For this we define the following types:

```rust
#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}
```

Note the `Box<Expr>` required because Rust 
[does not allow unboxed recursive types](https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes). 

There is no separate atom type, any atom is also a valid expression.

## Pratt parser
The precedence of operations is defined in the Pratt parser.

An easy approach is to define the PrattParser as global using [`lazy_static`](https://docs.rs/lazy_static/1.4.0/lazy_static/).

Adhering to standard rules of arithmetic, 
we will define addition and subtraction to have lower priority than multiplication and division, 
and make all operators left associative.

```rust
lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
    };
}
```

We are almost there, the only thing that's left is to use our Pratt parser.
For this the `map_primary`, `map_infix`, and `parse` functions are used, the first two take functions and the third one takes an iterator over pairs.
`map_primary` is executed for every primary (atom), and `map_infix` is executed for every binop with its new left-hand
and right-hand side according to the precedence rules defined earlier.
In this example, we create an AST in the Pratt parser.

```rust
pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer => Expr::Integer(primary.as_str().parse::<i32>().unwrap()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .parse(pairs)

}
```

Here's an example of how to use the parser.

```rust
fn main() -> io::Result<()> {
    for line in io::stdin().lock().lines() {
        match CalculatorParser::parse(Rule::equation, &line?) {
            Ok(mut pairs) => {
                println!(
                    "Parsed: {:#?}",
                    parse_expr(
                        // inner of expr
                        pairs.next().unwrap().into_inner()
                    )
                );
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
    Ok(())
}

```

With this, we can parse the following simple equation:
```
> 1 * 2 + 3 / 4
Parsed: BinOp {
    lhs: BinOp {
        lhs: Integer( 1 ),
        op: Multiply,
        rhs: Integer( 2 ),
    },
    op: Add,
    rhs: BinOp {
        lhs: Integer( 3 ),
        op: Divide,
        rhs: Integer( 4 ),
    },
}
```

## Unary minus and parenthesis
So far, our calculator can parse fairly complicated expressions, but it will fail if it encounters explicit parentheses 
or a unary minus sign. Let's fix that.

### Parentheses
Consider the expression `(1 + 2) * 3`. Clearly removing the parentheses would give a different result, so we must 
support parsing such expressions. Luckily, this can be a simple addition to our `atom` rule:

```diff
- atom = _{ integer }
+ atom = _{ integer | "(" ~ expr ~ ")" }
```

Earlier we said that atoms should be simple token sequences that cannot be split up further, but now an atom can contain
arbitrary expressions! The reason we are okay with this is that the parentheses mark clear boundaries for the 
expression, it will not make ambiguous what operators belong to the inner expression and which to the outer one.

### Unary minus
We can currently only parse positive integers, eg `16` or `2342`. But we also want to do calculations with negative integers.
To do this, we introduce the unary minus, so we can make `-4` and `-(8 + 15)`.
We need the following change to grammar:
```pest
+ unary_minus = { "-" }
+ primary = _{ integer | "(" ~ expr ~ ")" }
- atom = _{ integer | "(" ~ expr ~ ")" }
+ atom = _{ unary_minus? ~ primary }
```

For these last changes we've omitted the small changes to the AST and parsing logic (using `map_prefix`).

You can find all these details in 
the repository: [github.com/pest-parser/book/tree/master/examples/pest-calculator](https://github.com/pest-parser/book/tree/master/examples/pest-calculator).

```

---
## File: `src/examples/csv.md`
*(Relative Path: `src/examples/csv.md`)*

```markdown
# Example: CSV

Comma-Separated Values is a very simple text format. CSV files consist of a
list of *records*, each on a separate line. Each record is a list of *fields*
separated by commas.

For example, here is a CSV file with numeric fields:

```
65279,1179403647,1463895090
3.1415927,2.7182817,1.618034
-40,-273.15
13,42
65537
```

Let's write a program that computes the **sum of these fields** and counts the
**number of records**.

## Setup

Start by initializing a new project using [Cargo]:

```shell
$ cargo init --bin csv-tool
     Created binary (application) project
$ cd csv-tool
```

Add the `pest` and `pest_derive` crates to the dependencies section in `Cargo.toml`:

```toml
[dependencies]
pest = "2.6"
pest_derive = "2.6"
```

## Writing the parser

`pest` works by compiling a description of a file format, called a *grammar*,
into Rust code. Let's write a grammar for a CSV file that contains numbers.
Create a new file named `src/csv.pest` with a single line:

```pest
field = { (ASCII_DIGIT | "." | "-")+ }
```

This is a description of every number field: each character is either an ASCII
digit `0` through `9`, a full stop `.`, or a hyphen&ndash;minus `-`. The plus
sign `+` indicates that the pattern can occur one or more times.

Rust needs to know to compile this file using `pest`:

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "csv.pest"]
pub struct CSVParser;
```

If you run `cargo doc`, you will see that `pest` has created the function
`CSVParser::parse` and an enum called `Rule` with a single variant
`Rule::field`.

Let's test it out! Rewrite `main`:

```rust
fn main() {
    let successful_parse = CSVParser::parse(Rule::field, "-273.15");
    println!("{:?}", successful_parse);

    let unsuccessful_parse = CSVParser::parse(Rule::field, "this is not a number");
    println!("{:?}", unsuccessful_parse);
}
```

```shell
$ cargo run
  [ ... ]
Ok([Pair { rule: field, span: Span { str: "-273.15", start: 0, end: 7 }, inner: [] }])
Err(Error { variant: ParsingError { positives: [field], negatives: [] }, location: Pos(0), path: None, line: "this is not a number", continued_line: None, start: (1, 1), end: None })
```

Yikes! That's a complicated type! But you can see that the successful parse was
`Ok`, while the failed parse was `Err`. We'll get into the details later.

For now, let's complete the grammar:

```pest
field = { (ASCII_DIGIT | "." | "-")+ }
record = { field ~ ("," ~ field)* }
file = { SOI ~ (record ~ ("\r\n" | "\n"))* ~ EOI }
```

The tilde `~` means "and then", so that `"abc" ~ "def"` matches `abc` followed
by `def`. (For this grammar, `"abc" ~ "def"` is exactly the same as `"abcdef"`,
although this is not true in general; see [a later chapter about
`WHITESPACE`].)

In addition to literal strings (`"\r\n"`) and built-ins (`ASCII_DIGIT`), rules
can contain other rules. For instance, a `record` is a `field`, and optionally
a comma `,` and then another `field` repeated as many times as necessary. The
asterisk `*` is just like the plus sign `+`, except the pattern is optional: it
can occur any number of times at all (zero or more).

There are two more rules that we haven't defined: `SOI` and `EOI` are two
special rules that match, respectively, the *start of input* and the *end of
input*. Without `EOI`, the `file` rule would gladly parse an invalid file! It
would just stop as soon as it found the first invalid character and report a
successful parse, possibly consisting of nothing at all!

## The main program loop

Now we're ready to finish the program. We will use [`File`] to read the CSV
file into memory. We'll also be messy and use [`expect`] everywhere.

```rust
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("numbers.csv").expect("cannot read file");

    // ...
}
```

Next we invoke the parser on the file. Don't worry about the specific types for
now. Just know that we're producing a [`pest::iterators::Pair`] that represents
the `file` rule in our grammar.

```rust
fn main() {
    // ...

    let file = CSVParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next().unwrap(); // get and unwrap the `file` rule; never fails

    // ...
}
```

Finally, we iterate over the `record`s and `field`s, while keeping track of the
count and sum, then print those numbers out.

```rust
fn main() {
    // ...

    let mut field_sum: f64 = 0.0;
    let mut record_count: u64 = 0;

    for record in file.into_inner() {
        match record.as_rule() {
            Rule::record => {
                record_count += 1;

                for field in record.into_inner() {
                    field_sum += field.as_str().parse::<f64>().unwrap();
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    println!("Sum of fields: {}", field_sum);
    println!("Number of records: {}", record_count);
}
```

If `p` is a parse result (a [`Pair`]) for a rule in the grammar, then
`p.into_inner()` returns an [iterator] over the named sub-rules of that rule.
For instance, since the `file` rule in our grammar has `record` as a sub-rule,
`file.into_inner()` returns an iterator over each parsed `record`. Similarly,
since a `record` contains `field` sub-rules, `record.into_inner()` returns an
iterator over each parsed `field`.

## Done

Try it out! Copy the sample CSV at the top of this chapter into a file called
`numbers.csv`, then run the program! You should see something like this:

```shell
$ cargo run
  [ ... ]
Sum of fields: 2643429302.327908
Number of records: 5
```

[Cargo]: https://doc.rust-lang.org/cargo/
[a later chapter about `WHITESPACE`]: ../grammars/syntax.html
[`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
[`expect`]: https://doc.rust-lang.org/std/option/enum.Option.html#method.expect
[`pest::iterators::Pair`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pair.html
[`Pair`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pair.html
[iterator]: https://doc.rust-lang.org/std/iter/index.html

```

---
## File: `src/examples/rust/literals.md`
*(Relative Path: `src/examples/rust/literals.md`)*

```markdown
# Literals

A good place to start when writing out the grammar of a language are the
literals. For our small Rust subset, the literals that we are going to define
are booleans, integers, floating point numbers, strings, characters, types, and
identifiers.

## Booleans

Defining booleans is probably the easiest step. We need a rule with two
variants, `true` and `false`:

```
bool = { "true" | "false" }
```

This, however, will only generate a token for the `bool` rule without telling us
which variant it is, forcing us to dig through the input in order to see whether
it is `true` or `false`. In order to parse this only once and get the necessary
information right away, we can make `true` and `false` separate rules:

```
true  = { "true" }
false = { "false" }
bool  = { true | false }
```

Unfortunately, running `cargo check` will print the following error:

```
grammar error

 --> rust.pest:1:1
  |
1 | true  = { "true" }
  | ^--^
  |
  = true is a rust keyword

grammar error

 --> rust.pest:2:1
  |
2 | false = { "false" }
  | ^---^
  |
  = false is a rust keyword
```

This is because every one of the rules you define will populate an `enum` named
`Rule`. Thus, if any rules conflict with Rust's naming scheme, it will error
out with an ambiguous message which is why *pest* tries its best to catch any
possible error before it reaches the compiler.

A simple (but less elegant) solution here would be to suffix these rules with
`_lit`:

```
true_lit  = { "true" }
false_lit = { "false" }
bool      = { true_lit | false_lit }
```

This seems to work fine, but before we head on to integers, let's first write a
couple of tests. *pest* comes with a handy macro for asserting parse results
named [parses_to!][1].

```rust
#[test]
fn true_lit() {
    parses_to! {
        parser: RustParser,     // our parser struct
        input: "true",          // the input we're testing
        rule: Rule::bool,       // the rule that should be run
        tokens: [
            bool(0, 4, [        // name_of_rule(start_pos, end_pos, [children])
                true_lit(0, 4)  // name_of_rule(start_pos, end_pos): no children
            ])
        ]
    };
}

#[test]
fn false_lit() {
    parses_to! {
        parser: RustParser,
        input: "false",
        rule: Rule::bool,
        tokens: [
            bool(0, 5, [
                false_lit(0, 5)
            ])
        ]
    };
}
```

[1]: https://docs.rs/pest/1.0/pest/macro.parses_to.html

## Integers

Although not as trivial as the booleans, integers should be quite
straightforward. In our implementation, we will only implement decimal integers
which start with a digit, then continue with any mixture of digits and
underscores:

```
int = { '0'..'9' ~ ('0'..'9' | "_")* }
```

In the example above, the range defining a digit (`'0'..'9'`) is repeated and
can be turned into a rule. Since we do not want it to generate tokens or be
reported in errors, we will make it silent (`_`).

```
digit = _{ '0'..'9' }
int   =  { digit ~ (digit | "_")* }
```

Testing a few cases like `"0"`, `"01"`, `"0___"`, `"1_000_000"` should suffice.

## Floating point numbers

Here is where it starts to become a little bit tricky. Floating points come in
two different shapes:

* integer literal followed by a `'.'`, followed by another optional integer
  literal, followed by an optional exponent
* integer literal, followed by a an exponent

By abstracting the definition of the exponent, the grammar will look like this:

```
float = {
    int ~ "." ~ int? ~ exp? |
    int ~ exp
}
```

The exponent part is a case insensitive `'e'`, followed by an optional sign
(`'+'`/`'-'`), followed by an integer. To match a string insensitively, you can
use the `^` prefix operator. Again, we would like to keep track of the signs in
order not to have to parse again, so we make the signs separate rules:

```
plus  = { "+" }
minus = { "-" }
exp   = { ^"e" ~ (plus | minus)? ~ int }
```

Testing floating point numbers should take into consideration their nested
integer and exponent tokens:

```rust
#[test]
fn zero_point() {
    parses_to! {
        parser: RustParser,
        input: "0.",
        rule: Rule::float,
        tokens: [
            float(0, 2, [
                int(0, 1)
            ])
        ]
    };
}

#[test]
fn one_exponent() {
    parses_to! {
        parser: RustParser,
        input: "1e10",
        rule: Rule::float,
        tokens: [
            float(0, 4, [
                int(0, 1),
                exp(1, 4, [
                    int(2, 4)
                ])
            ])
        ]
    };
}
```

More interesting test cases could be `"0.e0"`, `"0.0e+0"`, `"0.0"`,
`"0__.0__e-0__"`.

## Strings

Strings can get a little bit tricky since you have to make sure that you include
string escapes in your grammar. This is needed since you have no other way of
knowing exactly where the string ending quote will be and also because it makes
escaping easier later on.

Let's start by focusing on the high level definition. A string is a repetition
of raw string parts (containing no escapes) and actual escapes, all enclosed
within a pair of quotes:

```
string = { "\"" ~ (raw_string | escape)* ~ "\"" }
```

Raw strings can basically be any character apart from `'\'`, since that means
we're about to start an escape clause, and `'"'`, since that means we're about
to end the string. In order to match anything but these two characters, we look
ahead and fail the rule if we match these two characters. For this, we're going
to use a negative lookahead (`!`). After we made sure that we're matching the
correct character, we use the predefined rule `any` to actually force the parser
to skip this character, since the lookahead is non-destructive:

```
raw_string = { (!("\\" | "\"") ~ any)+ }
```

Rust string literals can be:

* predefined: `'\n'`, `'\r'`, `'\t'`, `'\\'`, `'\0'`,
* bytes: `'\x$$'`, where `$$` are two hexadecimal digits
* unicode: `\u{$}` - `\u{$$$$$$}`, where `$`s are from 1 up to 6 hexadecimal
  digits

A good place to start is to define the hex digit:

```
hex = _{ '0'..'9' | 'a'..'f' | 'A'..'F' }
```

To define a rule that can have from 1 up to 6 hex digits, pest offers a convenient
syntax `{m, n}`. Limits are inclusive. Note that `{n}`, `{n, }`, and `{, n}` syntaxes
exist too. Please see [non-terminals expressions][2] for more details.

```
unicode_hex = { hex{1, 6} }
```

[2]: https://docs.rs/pest_derive/1.0/pest_derive/#expressions

We now have everything we need to define escapes:

```
predefined = { "n" | "r" | "t" | "\\" | "0" | "\"" | "'" }
byte       = { "x" ~ hex{2} }
unicode    = { "u" ~ "{" ~ unicode_hex ~ "}" }
escape     = { "\\" ~ (predefined | byte | unicode) }
```

For the sake of compactness, we can write a single test that encompasses
everything interesting:

```rust
#[test]
fn string_with_all_escape_types() {
    parses_to! {
        parser: RustParser,
        input: r#""a\nb\x0Fc\u{a}d\u{AbAbAb}e""#,
        rule: Rule::string,
        tokens: [
            string(0, 28, [
                raw_string(1, 2),
                escape(2, 4, [
                    predefined(3, 4)
                ]),
                raw_string(4, 5),
                escape(5, 9, [
                    byte(6, 9)
                ]),
                raw_string(9, 10),
                escape(10, 15, [
                    unicode(11, 15, [
                        unicode_hex(13, 14)
                    ])
                ]),
                raw_string(15, 16),
                escape(16, 26, [
                    unicode(17, 26, [
                        unicode_hex(19, 25)
                    ])
                ]),
                raw_string(26, 27)
            ])
        ]
    };
}
```

## Characters

Characters are very similar to strings, with the obvious exception that may only
store one character:

```
chr = { "'" ~ (escape | any) ~ "'" }
```

Tests should cover at least the usual and the escape cases, e.g. `"'a'"`,
`"'\''"`.

## Types

Types should only be the few primitives defined here:

```
i32_ty  = { "i32" }
f32_ty  = { "f32" }
char_ty = { "char" }
str_ty  = { "str" }

ty = { i32_ty | f32_ty | char_ty | str_ty }
```

Writing one test for each of the four cases should suffice.

## Identifiers

Full-blown Rust identifiers can be a bit complex, so we will only focus on ASCII
variants:

* an identifier is made up of alphanumeric characters and underscores
* the first character cannot be a digit
* underscores need to be followed by at least another character

This can be implemented by having a choice clause between two cases:

```
ident_char = _{ 'a'..'z' | 'A'..'Z' | '0'..'9' | "_" }
ident      =  {
    ('a'..'z' | 'A'..'Z') ~ ident_char* |
    "_" ~ ident_char+
}
```

Interesting test cases could be `"aBc0"`, `"_0AbC"`.

```

---
## File: `src/examples/rust/syntax.md`
*(Relative Path: `src/examples/rust/syntax.md`)*

```markdown
# Syntax

Now that we have literals defined, the next step is to compose them into the
syntax of the language. This syntax will only focus on expressions, statements,
and functions as a subset of Rust. These in turn will not be complete
definitions.

## Expressions

We will define expressions as a combination of unary and infix operations, and
method calls. The operators that we will use for this subset are:

```
op_unary_minus =  { "-" }
op_unary_not   =  { "!" }
op_unary       = _{
    op_unary_minus |
    op_unary_not
}

op_plus          =  { "+" }
op_minus         =  { "-" }
op_times         =  { "*" }
op_divide        =  { "/" }
op_and           =  { "&&" }
op_or            =  { "||" }
op_greater       =  { ">" }
op_greater_equal =  { ">=" }
op_lower         =  { "<" }
op_lower_equal   =  { "<=" }
op_equal         =  { "==" }
op_infix         = _{
    op_plus |
    op_minus |
    op_times |
    op_divide |
    op_and |
    op_or |
    op_greater |
    op_greater_equal |
    op_lower |
    op_lower_equal |
    op_equal
}

paren_open  = { "(" }
paren_close = { ")" }
```

We also defined parentheses rules since they will come in handy in a bit.
Because PEGs do not support left-recursion, we will have to make sure to have
a layer of indirection when defining infix expressions, while unaries and method
calls will be defined with the use of repetitions.

The easiest way to start would be to define expressions with the highest
priorities. These expressions will be the only ones that unaries can be formed
with and methods can be called on. They are the literals defined in the previous
chapter plus expressions nested in parentheses:

```
value = {
    float | // float comes before int since they overlap
    int |
    chr |
    string |
    ident |
    paren_open ~ expr ~ paren_close
}
```

With that out of the way, a next step would be to define what a call should look
like:

```
dot   =  { "." }
comma =  { "," }
args  = _{ expr ~ (comma ~ expr)* }
call  =  { ident ~ paren_open ~ args? ~ paren_close }
```

Now we can include unaries and method calls in one single term rule that will
be used in infix expressions:

```
term = { op_unary* ~ value ~ (dot ~ call)* }
expr = { term ~ (op_infix ~ term)* }
```

Extensive testing would be handy here, especially more complex cases that
combine expression types, but also separate tests for individual behavior.

## Statements

```

---
## File: `src/examples/rust/setup.md`
*(Relative Path: `src/examples/rust/setup.md`)*

```markdown
# Setup

Before getting into the more theoretical parts of grammars and APIs, let's first
make sure we're all set up.

## Rust and Cargo

The easiest way to install Rust and Cargo together is to follow the instructions
on [rustup.rs](https://rustup.rs). Once that is out of the way, make sure you
add *pest* to your `Cargo.toml`:

```toml
pest = "2.6"
pest_derive = { version = "2.6", features = ["grammar-extras"] }
```

*pest_derive* is the part of the parser that analyzes, verifies, optimizes, and
generates the code that then makes use of the APIs found in the *pest* crate.
This is separate because the actual procedural macro that derives the parser for
you is linked at compile time.

The "grammar-extras" feature mainly adds advanced functionality, but it also
changes the parser behaviour in backwards-incompatible ways (hence it will not
become the default in 2.x) by fixing several problems. New projects are
encouraged to enable this feature as it helps prevent unexpected behaviour.

## The `.pest` grammar file

The actual grammar gets saved in separate `.pest` files, relative to Cargo's
`src` directory. They are then used in order to derive an implementation of the
[Parser][1] trait.

Due to the fact that procedural macro do not offer an API to tell the compiler
which files are relevant to compilation, it is necessary to provide a small hint
in the form of a debug-only `const` in order to make sure that your grammar gets
recompiled after every change.

So, you should add the following code to the Rust file where you want the parser
to be.

[1]: https://docs.rs/pest/1.0/pest/trait.Parser.html

```rust
// Don't forget to request use of the pest and pest_derive crate
use pest::Parser;
use pest_derive::Parser

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("path/to/rust.pest"); // relative to this file

#[derive(Parser)]
#[grammar = "path/to/rust.pest"] // relative to src
struct RustParser;
```

```

---
# Unrecognized Files

The following files have extensions not recognized by the script, so their contents were *not* included:

- `LICENSE-APACHE`
- `.gitignore`
- `LICENSE-MIT`

```

---
## File: `README.md`
*(Relative Path: `README.md`)*

```markdown
# TODO Extractor

A **multi-language TODO comment extractor** for source code files. This tool parses Python, Rust, JavaScript, TypeScript, and Go files to extract TODO comments, including single-line and multi-line TODOs.

## Features
- 📌 **Supports multiple programming languages**: Python, Rust, JavaScript, TypeScript, and Go.
- 📝 **Extracts TODO comments** from both single-line (`// TODO:`) and block (`/* TODO: */`) comments.
- 🔄 **Handles multi-line TODOs** by merging indented or docstring-style comments.
- 🚀 **Fast and efficient** using the [Pest](https://pest.rs/) parser.
- 📍 **Provides line numbers** for each TODO found.

---

## 📦 Installation

Clone the repository and build the project using Cargo:

```sh
# Clone the repository
git clone https://github.com/your-repo/todo-extractor.git
cd todo-extractor

# Build the project
cargo build --release

# Run the executable
./target/release/todo-extractor path/to/your/file.rs
```

---

## 🚀 Usage

### Command Line
Run the extractor by providing a path to a source file:

```sh
./todo-extractor path/to/source_file.rs
```

### Example Output
```sh
Found 2 TODOs:
3 - Refactor this function
10 - Handle edge cases properly
```

### **Example: Python File (`test.py`)**
```python
# TODO: Implement feature X
def function():
    """
    TODO: Improve performance
    by reducing loops
    """
    pass
```
#### **Extracted TODOs:**
```sh
Found 2 TODOs:
1 - Implement feature X
3 - Improve performance by reducing loops
```

---

## 🔍 How It Works
### **1. Detects File Type**
The parser checks the file extension (`.py`, `.rs`, `.js`, `.ts`, `.go`) and selects the appropriate parser.

### **2. Extracts Comment Lines**
Using **Pest grammars**, the extractor retrieves comment lines while ignoring code and string literals.

### **3. Identifies TODO Markers**
The extractor searches for `TODO:` inside comment lines and captures the message.

### **4. Handles Multi-line TODOs**
If a TODO is followed by indented lines, they are merged into a single entry.

---

## 🛠️ Supported Languages
| Language | Single-line Syntax | Block Syntax |
|----------|-------------------|--------------|
| Python   | `# TODO: ...` | `""" TODO: ... """` |
| Rust     | `// TODO: ...` | `/* TODO: ... */` |
| JavaScript | `// TODO: ...` | `/* TODO: ... */` |
| TypeScript | `// TODO: ...` | `/* TODO: ... */` |
| Go       | `// TODO: ...` | `/* TODO: ... */` |

---

## 🧪 Running Tests
Run tests to validate the TODO extraction:
```sh
cargo test
```

Example test case for Rust files:
```rust
#[test]
fn test_rust_single_line() {
    let src = "// TODO: Refactor this code";
    let todos = extract_todos(Path::new("example.rs"), src);
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].message, "Refactor this code");
}
```

---

## 🤝 Contributing
Want to add support for another language? Contributions are welcome! Feel free to open an issue or submit a pull request.

```

---
## File: `tests/rust_tests.rs`
*(Relative Path: `tests/rust_tests.rs`)*

```rust
#[cfg(test)]
mod rust_tests {
    use env_logger;
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::aggregator::extract_todos;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_default_env()
                .filter_level(LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok(); 
        });
    }

    #[test]
    fn test_rust_single_line() {
        init_logger();
        let src = r#"
// normal comment
// TODO: single line
fn main() {
   let s = "TODO: not a comment in string";
}
"#;
        let todos = extract_todos(Path::new("example.rs"), src);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 3);
        assert_eq!(todos[0].message, "single line");
    }

    #[test]
    fn test_rust_block_doc() {
        init_logger();
        let src = r#"
/// TODO: fix this doc
/// second line
fn foo() {}

/*
   stuff
   TODO: block
   more lines
*/
"#;
        let todos = extract_todos(Path::new("lib.rs"), src);
        assert_eq!(todos.len(), 2);

        // doc comment
        assert_eq!(todos[0].line_number, 2);
        assert!(todos[0].message.contains("fix this doc"));
        assert!(todos[0].message.contains("second line"));

        // block
        assert!(todos[1].message.contains("block"));
        assert!(todos[1].message.contains("more lines"));
    }
}

```

---
## File: `tests/python_tests.rs`
*(Relative Path: `tests/python_tests.rs`)*

```rust
#[cfg(test)]
mod python_tests {
    use env_logger;
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::aggregator::extract_todos;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_default_env()
                .filter_level(LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok(); 
        });
    }

    #[test]
    fn test_python_single_line() {
        init_logger();

        let src = r#"
# TODO: do something
x = "TODO: not a comment"
"#;
        // We'll pass an artificial .py path
        let todos = extract_todos(Path::new("test.py"), src);
        println!("{:?}", todos);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 2); // line is 1-based
        assert_eq!(todos[0].message, "do something");
    }

    #[test]
    fn test_python_docstring() {
        init_logger();
        let src = r#"
def f():
    """
    normal doc line
    TODO: fix f
      some more text
    """
"#;
        let todos = extract_todos(Path::new("test.py"), src);
        assert_eq!(todos.len(), 1);
        let item = &todos[0];
        // The docstring starts presumably on line 3 or 4
        assert_eq!(item.line_number, 4);
        // The text merges "fix f" + "some more text" due to aggregator's logic
        assert!(item.message.contains("fix f"));
        assert!(item.message.contains("some more text"));
    }
}

```

---
## File: `tests/mod.rs`
*(Relative Path: `tests/mod.rs`)*

```rust
mod rust_tests;
mod python_tests;
```

---
## File: `src/lib.rs`
*(Relative Path: `src/lib.rs`)*

```rust
pub mod aggregator;
pub mod languages;

pub use aggregator::extract_todos;

```

---
## File: `src/main.rs`
*(Relative Path: `src/main.rs`)*

```rust
use std::env;
use std::path::Path;
use log::{info, warn, error, LevelFilter};

fn main() {
    // Initialize the logger based on RUST_LOG or default to Debug.
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        warn!("Usage: {} <path/to/file>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);
    let content = std::fs::read_to_string(path).expect("cannot read file");

    // Use the library's extractor
    let todos = todo_extractor::extract_todos(path, &content);

    if todos.is_empty() {
        info!("No TODOs found.");
    } else {
        info!("Found {} TODOs:", todos.len());
        for todo in todos {
            info!("{} - {}", todo.line_number, todo.message);
        }
    }
}

```

---
## File: `src/aggregator.rs`
*(Relative Path: `src/aggregator.rs`)*

```rust
use std::path::Path;
use log::debug;

use crate::languages::{
    python::parse_python_comments,
    rust::parse_rust_comments,
    go::parse_go_comments,
    js::parse_js_comments,
    ts::parse_ts_comments,
};

/// Represents a single found TODO item.
#[derive(Debug, PartialEq)]
pub struct TodoItem {
    pub line_number: usize,
    pub message: String,
}

/// Detects file extension and chooses the parser to gather raw comment lines,
/// then extracts multi-line TODOs from those comments.
pub fn extract_todos(path: &Path, file_content: &str) -> Vec<TodoItem> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    debug!("extract_todos: extension = '{}'", extension);

    // Call the relevant language parser to extract comment lines
    let comment_lines = match extension.as_str() {
        "py" => parse_python_comments(file_content),
        "rs" => parse_rust_comments(file_content),
        "go" => parse_go_comments(file_content),
        "js" => parse_js_comments(file_content),
        "ts" => parse_ts_comments(file_content),
        _ => {
            debug!("No recognized extension for file {:?}; returning empty list.", path);
            vec![]
        }
    };

    debug!(
        "extract_todos: found {} comment lines from parser",
        comment_lines.len()
    );

    // Next, find any TODOs among these comment lines
    let todos = collect_todos_from_comment_lines(&comment_lines);
    debug!("extract_todos: found {} TODO items total", todos.len());
    todos
}

/// A single comment line with (line_number, entire_comment_text).
#[derive(Debug)]
pub struct CommentLine {
    pub line_number: usize,
    pub text: String,
}

/// Merge multi-line TODO lines and produce `TodoItem` for each distinct `TODO:`.
pub fn collect_todos_from_comment_lines(lines: &[CommentLine]) -> Vec<TodoItem> {
    let mut result = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        let text = &lines[idx].text;
        let line_num = lines[idx].line_number;

        debug!(
            "collect_todos: checking line {} => '{}'",
            line_num, text
        );

        if let Some(pos) = text.find("TODO:") {
            debug!(" -> Found TODO at line {} pos {}", line_num, pos);
            let after_todo = &text[pos + 5..]; // skip "TODO:"
            let mut collected = after_todo.trim_start().to_string();

            idx += 1;
            while idx < lines.len() {
                let cont_text = &lines[idx].text;
                if cont_text.starts_with(' ') || cont_text.starts_with('\t') {
                    debug!(
                        "   continuing multiline TODO at line {} => '{}'",
                        lines[idx].line_number, cont_text
                    );
                    collected.push(' ');
                    collected.push_str(cont_text.trim());
                    idx += 1;
                } else {
                    break;
                }
            }

            let final_msg = collected.trim_end().to_string();
            debug!(
                " -> Final merged TODO from line {} => '{}'",
                line_num, final_msg
            );

            result.push(TodoItem {
                line_number: line_num,
                message: final_msg,
            });
        } else {
            idx += 1;
        }
    }

    result
}

```

---
## File: `src/languages/ts.pest`
*(Relative Path: `src/languages/ts.pest`)*

```pest
comment_ts = { line_comment | block_comment }

line_comment = @{
    "//" ~ (!NEWLINE ~ ANY)*
}

block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'" |
    "`" ~ (!"`" ~ ANY)* ~ "`"
}

ts_file = {
    (comment_ts | str_literal | ANY)*
}

```

---
## File: `src/languages/ts.rs`
*(Relative Path: `src/languages/ts.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/ts.pest"]
pub struct TsParser;

pub fn parse_ts_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = TsParser::parse(Rule::ts_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            if pair.as_rule() == Rule::comment_ts {
                handle_ts_comment(pair, &mut comments);
            }
        }
    }
    comments
}

fn handle_ts_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            let stripped = text.trim_start_matches("//").trim();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            let trimmed = block_text.trim_start_matches("/*").trim_end_matches("*/");
            let lines = trimmed.split('\n');
            let mut current_line = start_line;
            for l in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: l.trim().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
## File: `src/languages/rust.rs`
*(Relative Path: `src/languages/rust.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;
use log::{debug, info, error};

#[derive(Parser)]
#[grammar = "languages/rust.pest"]
pub struct RustParser;

pub fn parse_rust_comments(file_content: &str) -> Vec<CommentLine> {
    info!("Starting to parse Rust comments. File content length: {}", file_content.len());

    let parse_result = RustParser::parse(Rule::rust_file, file_content);
    let mut comments = Vec::new();

    match parse_result {
        Ok(pairs) => {
            debug!("Parsing successful. Number of top-level pairs: {}", pairs.clone().count());

            for pair in pairs {
                debug!("Found pair: {:?}", pair.as_rule());

                if pair.as_rule() == Rule::comment_rust {
                    debug!("Extracting Rust comment: {:?}", pair.as_str());
                    handle_rust_comment(pair, &mut comments);
                }
            }
        }
        Err(e) => {
            error!("Parsing error: {:?}", e);
        }
    }

    info!("Total extracted Rust comments: {}", comments.len());
    comments
}

fn handle_rust_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            debug!("Line comment found at line {}: '{}'", line, text);
            // Remove leading `//`
            let stripped = text.trim_start_matches('/').trim_start_matches('/').trim_start();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            debug!("Block comment found starting at line {}: length {}", start_line, block_text.len());
            // remove `/*` and `*/`
            let trimmed = block_text
                .trim_start_matches("/*")
                .trim_end_matches("*/");

            // split by lines
            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
            for line_text in lines {
                debug!("Block comment line {}: '{}'", current_line, line_text);
                out.push(CommentLine {
                    line_number: current_line,
                    text: line_text.trim_end().to_string(),
                });
                current_line += 1;
            }
        }
        Rule::doc_comment_line => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            debug!("Doc comment line found at line {}: '{}'", line, text);
            // remove leading `///` or `//!`
            let stripped = if text.starts_with("///") {
                &text[3..]
            } else {
                &text[3..] // `//!`
            };
            out.push(CommentLine {
                line_number: line,
                text: stripped.trim_end().to_string(),
            });
        }
        _ => {}
    }
}

```

---
## File: `src/languages/go.rs`
*(Relative Path: `src/languages/go.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/go.pest"]
pub struct GoParser;

pub fn parse_go_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = GoParser::parse(Rule::go_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            if pair.as_rule() == Rule::comment_go {
                handle_go_comment(pair, &mut comments);
            }
        }
    }
    comments
}

fn handle_go_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            // remove `//`
            let stripped = text.trim_start_matches('/').trim_start_matches('/').trim();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            // remove `/*`...`*/`
            let trimmed = block_text
                .trim_start_matches("/*")
                .trim_end_matches("*/");

            let lines = trimmed.split('\n');
            let mut current_line = start_line;
            for l in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: l.trim().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
## File: `src/languages/js.pest`
*(Relative Path: `src/languages/js.pest`)*

```pest
comment_js = { line_comment | block_comment }

// single-line
line_comment = @{
    "//" ~ (!NEWLINE ~ ANY)*
}

// multi-line
block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

// skip strings
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'" |
    "`" ~ (!"`" ~ ANY)* ~ "`" // backticks in JS
}

js_file = {
    (comment_js | str_literal | ANY)*
}

```

---
## File: `src/languages/js.rs`
*(Relative Path: `src/languages/js.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/js.pest"]
pub struct JsParser;

pub fn parse_js_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = JsParser::parse(Rule::js_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            if pair.as_rule() == Rule::comment_js {
                handle_js_comment(pair, &mut comments);
            }
        }
    }
    comments
}

fn handle_js_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            let stripped = text.trim_start_matches("//").trim();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            let trimmed = block_text.trim_start_matches("/*").trim_end_matches("*/");
            let lines = trimmed.split('\n');
            let mut current_line = start_line;
            for l in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: l.trim().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
## File: `src/languages/python.rs`
*(Relative Path: `src/languages/python.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;
use log::{debug, info, error};

// Load grammar at compile time
#[derive(Parser)]
#[grammar = "languages/python.pest"]
pub struct PythonParser;

/// Parse the entire Python file content, returning lines of comment text (and line numbers).
pub fn parse_python_comments(file_content: &str) -> Vec<CommentLine> {
    info!("Starting to parse Python comments. Input length: {}", file_content.len());

    let parse_result = PythonParser::parse(Rule::python_file, file_content);
    let mut comments = Vec::new();

    match parse_result {
        Ok(pairs) => {
            debug!("Parsing successful. Top-level pairs count: {}", pairs.clone().count());
            for pair in pairs {
                debug!("Found pair: {:?}", pair.as_rule());
                match pair.as_rule() {
                    Rule::comment_python => {
                        handle_comment_token(&pair, &mut comments);
                    }
                    _ => {
                        // ignore everything else
                    }
                }
            }
        }
        Err(e) => {
            error!("Parsing error: {}", e);
        }
    }
    info!("Returning {} comment lines", comments.len());

    comments
}

/// If it's a single-line `# ...`, store exactly that line.
/// If it's a docstring_comment, store each line inside that triple-quoted block.
fn handle_comment_token(pair: &Pair<Rule>, out: &mut Vec<CommentLine>) {
    let rule = pair.as_rule();
    debug!("Handling comment token: rule = {:?}", rule);

    match rule {
        Rule::line_comment => {
            let span = pair.as_span();
            let start_pos = span.start_pos().line_col().0; // line number (1-based)
            let text = span.as_str();

            debug!("Line comment found at line {}: '{}'", start_pos, text);

            // remove leading '#'
            let stripped = text.trim_start_matches('#').trim_end().to_string();

            out.push(CommentLine {
                line_number: start_pos,
                text: stripped,
            });
        }
        Rule::docstring_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();

            debug!("Docstring comment found starting at line {}: length {}", start_line, block_text.len());

            // remove the surrounding """..."""
            let trimmed = block_text
                .trim_start_matches("\"\"\"")
                .trim_end_matches("\"\"\"");

            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
            for line_text in lines {
                debug!("Docstring line {}: '{}'", current_line, line_text);
                out.push(CommentLine {
                    line_number: current_line,
                    text: line_text.trim_end().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
## File: `src/languages/python.pest`
*(Relative Path: `src/languages/python.pest`)*

```pest
// We assume the "grammar-extras" feature is on, so we can define COMMENT or WHITESPACE if we want.
//
// For Python, we'll capture top-level "comment" tokens as well as "docstring_comment".

comment_python = { line_comment | docstring_comment }

// Single-line # comment
line_comment = @{
    "#" ~ (!NEWLINE ~ ANY)*
}

// We'll consider triple quotes as docstring_comment for simplicity.
docstring_comment = @{
    "\"\"\"" ~ (!"\"\"\"" ~ ANY)* ~ "\"\"\""
}

// For normal string literals (to skip them), we define a rough rule to ensure we don't treat them as comments:
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'"
}

// The main rule: a Python file can have any mixture of comments, string literals, or code (ANY).
// We only want to capture "comment_python" tokens as recognized comment pairs in the parse tree.
python_file = {
    (comment_python | str_literal | ANY)*
}

```

---
## File: `src/languages/rust.pest`
*(Relative Path: `src/languages/rust.pest`)*

```pest
// Minimal Rust grammar focusing on comment tokens & ignoring string literals.

comment_rust = { line_comment | block_comment | doc_comment_line }

// Single-line: `//` until newline
line_comment = @{ "//" ~ (!NEWLINE ~ ANY)* }

// Block comment: `/* ... */` (this can be multi-line)
block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

// Doc comment lines start with `///` or `//!` (we unify them)
doc_comment_line = @{
    "///" ~ (!NEWLINE ~ ANY)* |
    "//!" ~ (!NEWLINE ~ ANY)*
}

// For ignoring string literals roughly:
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'"
}

rust_file = {
    (comment_rust | str_literal | ANY)*
}

```

---
## File: `src/languages/mod.rs`
*(Relative Path: `src/languages/mod.rs`)*

```rust
pub mod ts;
pub mod rust;
pub mod python;
pub mod js;
pub mod go;

pub use ts::parse_ts_comments;
pub use rust::parse_rust_comments;
pub use python::parse_python_comments;
pub use js::parse_js_comments;
pub use go::parse_go_comments;

```

---
## File: `src/languages/go.pest`
*(Relative Path: `src/languages/go.pest`)*

```pest
comment_go = { line_comment | block_comment }

// single-line
line_comment = @{
    "//" ~ (!NEWLINE ~ ANY)*
}

// multi-line
block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

// skip string literals
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'"
}

go_file = {
    (comment_go | str_literal | ANY)*
}

```

---
# Unrecognized Files

The following files were not recognized by extension and were skipped:

- `.gitmodules`
- `.gitignore`
