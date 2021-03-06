use std::fs;
use std::path::PathBuf;

use super::Token::*;
use super::*;

#[test]
fn decimal_literals() {
    for i in 0..11 {
        assert_eq!(tokenize(&i.to_string()).unwrap(), vec![IntLiteral(i)]);
    }
}

#[test]
fn decimal_literals_line_breaks() {
    assert_eq!(
        tokenize("1\n2\n3\n").unwrap(),
        vec![IntLiteral(1), IntLiteral(2), IntLiteral(3)]
    );
}

#[test]
fn hex_literals() {
    assert_eq!(tokenize("0x1").unwrap(), vec![IntLiteral(1)]);
    assert_eq!(tokenize("0xa").unwrap(), vec![IntLiteral(10)]);
    assert_eq!(tokenize("0xB").unwrap(), vec![IntLiteral(11)]);
    assert_eq!(tokenize("0xABC").unwrap(), vec![IntLiteral(2748)]);
}

#[test]
fn oct_literals() {
    assert_eq!(tokenize("00").unwrap(), vec![IntLiteral(0)]);
    assert_eq!(tokenize("01").unwrap(), vec![IntLiteral(1)]);
    assert_eq!(tokenize("07").unwrap(), vec![IntLiteral(7)]);
    assert_eq!(tokenize("071").unwrap(), vec![IntLiteral(57)]);
    assert_eq!(tokenize("0777").unwrap(), vec![IntLiteral(511)]);
}

#[test]
fn tab_separated_ints() {
    assert_eq!(
        tokenize("\t1\t2\t3\t").unwrap(),
        vec![IntLiteral(1), IntLiteral(2), IntLiteral(3)]
    );
}

#[test]
fn basic_keywords() {
    assert_eq!(tokenize("int").unwrap(), vec![IntKw]);
    assert_eq!(tokenize("return").unwrap(), vec![ReturnKw]);
}

#[test]
fn unary_operators() {
    assert_eq!(tokenize("-").unwrap(), vec![Minus]);
    assert_eq!(tokenize("~").unwrap(), vec![Tilde]);
    assert_eq!(tokenize("!").unwrap(), vec![Bang]);
}

#[test]
fn binary_operators() {
    assert_eq!(tokenize("+").unwrap(), vec![Plus]);
    assert_eq!(tokenize("/").unwrap(), vec![Slash]);
    assert_eq!(tokenize("*").unwrap(), vec![Asterisk]);
    assert_eq!(tokenize("&&").unwrap(), vec![DoubleAmpersand]);
    assert_eq!(tokenize("||").unwrap(), vec![DoubleBar]);
    assert_eq!(tokenize("==").unwrap(), vec![DoubleEqual]);
    assert_eq!(tokenize("!=").unwrap(), vec![BangEqual]);
    assert_eq!(tokenize("<").unwrap(), vec![LessThan]);
    assert_eq!(tokenize("<=").unwrap(), vec![LessThanEqual]);
    assert_eq!(tokenize(">").unwrap(), vec![GreaterThan]);
    assert_eq!(tokenize(">=").unwrap(), vec![GreaterThanEqual]);
}

#[test]
fn return_statement() {
    assert_eq!(
        tokenize("return 0;").unwrap(),
        vec![ReturnKw, IntLiteral(0), Semicolon]
    );
}

#[test]
fn empty_function_one_line() {
    assert_eq!(
        tokenize("int foo() {}").unwrap(),
        vec![
            IntKw,
            Identifier(String::from("foo")),
            OpenParen,
            CloseParen,
            OpenBrace,
            CloseBrace
        ]
    );
}

#[test]
fn empty_function() {
    assert_eq!(
        tokenize("int foo() {\n}").unwrap(),
        vec![
            IntKw,
            Identifier(String::from("foo")),
            OpenParen,
            CloseParen,
            OpenBrace,
            CloseBrace
        ]
    );
}

#[test]
fn function_return_0() {
    assert_eq!(
        tokenize("int foo() {\n\treturn 0;\n}").unwrap(),
        vec![
            IntKw,
            Identifier(String::from("foo")),
            OpenParen,
            CloseParen,
            OpenBrace,
            ReturnKw,
            IntLiteral(0),
            Semicolon,
            CloseBrace
        ]
    )
}

#[test]
fn syntax_error_with_invalid_identifier() {
    assert_eq!(
        *tokenize("int $foo() {}")
            .err()
            .unwrap()
            .downcast::<SyntaxError>()
            .unwrap(),
        SyntaxError::InvalidIdentifier(String::from("$foo"))
    );
}

macro_rules! file_tests {
    ($
        (
            $test_dir:literal: (
                $($name:ident: ($file:literal, $expected:expr),)+
            ),
        )+
    ) => {
        $(
            $(
                #[test]
                fn $name() {
                    let mut path = PathBuf::from($test_dir);
                    path.push($file);
                    let contents = fs::read_to_string(path).unwrap();
                    assert_eq!(
                        tokenize(&contents).unwrap(),
                        $expected
                    );
                }
            )+
        )+
    }
}

file_tests! {
    "tests/testfiles/valid": (
        file_abundant_spaces: ("abundant_spaces.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_add: ("add.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                Plus,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_and_false: ("and_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                DoubleAmpersand,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_and_true: ("and_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                DoubleAmpersand,
                Minus,
                IntLiteral(1),
                Semicolon,
                CloseBrace
            ]
        ),
        file_associativity_div: ("associativity_div.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(6),
                Slash,
                IntLiteral(3),
                Slash,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_associativity: ("associativity.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                Minus,
                IntLiteral(2),
                Minus,
                IntLiteral(3),
                Semicolon,
                CloseBrace
            ]
        ),
        file_bitwise: ("bitwise.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Bang,
                IntLiteral(12),
                Semicolon,
                CloseBrace
            ]
        ),
        file_bitwise_zero: ("bitwise_zero.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Tilde,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_comp_precedence_2: ("comp_precedence_2.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                DoubleEqual,
                IntLiteral(2),
                DoubleBar,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_comp_precedence: ("comp_precedence.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                DoubleEqual,
                IntLiteral(2),
                GreaterThan,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_div_neg: ("div_neg.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                OpenParen,
                Minus,
                IntLiteral(12),
                CloseParen,
                Slash,
                IntLiteral(5),
                Semicolon,
                CloseBrace
            ]
        ),
        file_div: ("div.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(4),
                Slash,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_eq_false: ("eq_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                DoubleEqual,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_eq_true: ("eq_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                DoubleEqual,
                IntLiteral(1),
                Semicolon,
                CloseBrace
            ]
        ),
        file_ge_false: ("ge_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                GreaterThanEqual,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_ge_true: ("ge_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                GreaterThanEqual,
                IntLiteral(1),
                Semicolon,
                CloseBrace
            ]
        ),
        file_gt_false: ("gt_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                GreaterThan,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_gt_true: ("gt_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                GreaterThan,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_le_false: ("le_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                LessThanEqual,
                Minus,
                IntLiteral(1),
                Semicolon,
                CloseBrace
            ]
        ),
        file_le_true: ("le_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                LessThanEqual,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_logical_precedence_2: ("logical_precedence_2.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                OpenParen,
                IntLiteral(1),
                DoubleBar,
                IntLiteral(0),
                CloseParen,
                DoubleAmpersand,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_logical_precedence: ("logical_precedence.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                DoubleBar,
                IntLiteral(0),
                DoubleAmpersand,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_lt_false: ("lt_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                LessThan,
                IntLiteral(1),
                Semicolon,
                CloseBrace
            ]
        ),
        file_lt_true: ("lt_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                LessThan,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_multi_digit: ("multi_digit.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(100),
                Semicolon,
                CloseBrace
            ]
        ),
        file_many_newlines: ("many_newlines.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_minimal_whitespace: ("minimal_whitespace.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_mult: ("mult.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                Asterisk,
                IntLiteral(3),
                Semicolon,
                CloseBrace
            ]
        ),
        file_ne_false: ("ne_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                BangEqual,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_ne_true: ("ne_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Minus,
                IntLiteral(1),
                BangEqual,
                Minus,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_neg: ("neg.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Minus,
                IntLiteral(5),
                Semicolon,
                CloseBrace
            ]
        ),
        file_nested_ops: ("nested_ops.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Bang,
                Minus,
                IntLiteral(3),
                Semicolon,
                CloseBrace
            ]
        ),
        file_nested_ops_2: ("nested_ops_2.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Minus,
                Tilde,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_not_0: ("not_0.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Bang,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_not_5: ("not_5.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Bang,
                IntLiteral(5),
                Semicolon,
                CloseBrace
            ]
        ),
        file_or_false: ("or_false.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                DoubleBar,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_or_true: ("or_true.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                DoubleBar,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_parens: ("parens.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                Asterisk,
                OpenParen,
                IntLiteral(3),
                Plus,
                IntLiteral(4),
                CloseParen,
                Semicolon,
                CloseBrace
            ]
        ),
        file_precedence: ("precedence.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                Plus,
                IntLiteral(3),
                Asterisk,
                IntLiteral(4),
                Semicolon,
                CloseBrace
            ]
        ),
        file_return_0: ("return_0.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_return_2: ("return_2.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        // TODO: skip_on_failure tests
        file_sub_neg: ("sub_neg.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                Minus,
                Minus,
                IntLiteral(1),
                Semicolon,
                CloseBrace
            ]
        ),
        file_sub: ("sub.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                Minus,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        ),
        file_unop_add: ("unop_add.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Tilde,
                IntLiteral(2),
                Plus,
                IntLiteral(3),
                Semicolon,
                CloseBrace
            ]
        ),
        file_unop_parens: ("unop_parens.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Tilde,
                OpenParen,
                IntLiteral(1),
                Plus,
                IntLiteral(1),
                CloseParen,
                Semicolon,
                CloseBrace
            ]
        ),
    ),
    "tests/testfiles/invalid": (
        file_missing_paren: ("missing_paren.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        ),
        file_missing_return_val: ("missing_return_val.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Semicolon,
                CloseBrace
            ]
        ),
        file_missing_closing_brace: ("missing_closing_brace.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
            ]
        ),
        file_missing_semicolon: ("missing_semicolon.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                CloseBrace,
            ]
        ),
        file_missing_return_space: ("missing_return_space.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                Identifier(String::from("return0")),
                Semicolon,
                CloseBrace,
            ]
        ),
        file_wrong_return_case: ("wrong_return_case.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                Identifier(String::from("RETURN")),
                IntLiteral(0),
                Semicolon,
                CloseBrace,
            ]
        ),
        file_missing_const: ("missing_const.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Bang,
                Semicolon,
                CloseBrace,
            ]
        ),
        file_missing_semicolon_2: ("missing_semicolon_2.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Bang,
                IntLiteral(5),
                CloseBrace,
            ]
        ),
        file_nested_missing_const: ("nested_missing_const.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Bang,
                Tilde,
                Semicolon,
                CloseBrace,
            ]
        ),
        file_wrong_unary_order: ("wrong_unary_order.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(4),
                Minus,
                Semicolon,
                CloseBrace,
            ]
        ),
        file_malformed_paren: ("malformed_paren.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                OpenParen,
                Minus,
                IntLiteral(3),
                CloseParen,
                Semicolon,
                CloseBrace,
            ]
        ),
        file_missing_first_op: ("missing_first_op.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Slash,
                IntLiteral(3),
                Semicolon,
                CloseBrace,
            ]
        ),
        file_missing_second_op: ("missing_second_op.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(1),
                Plus,
                Semicolon,
                CloseBrace,
            ]
        ),
        file_no_semicolon: ("no_semicolon.c",
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                Asterisk,
                IntLiteral(2),
                CloseBrace,
            ]
        ),
    ),
}