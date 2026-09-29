use std::{assert_matches, iter::Peekable, slice::Iter};

use crate::{
    error::{LanguageError, ParseError},
    expression::{
        AssignmentExpression, BinaryExpression, Expression, GroupingExpression,
        LiteralExpression, UnaryExpression, VariableExpression,
    },
    statement::{
        ExpressionStatement, PrintStatement, Statement,
        VariableDeclarationStatement,
    },
    token::Token,
};

// grammar
// program   → statement* EOF
// statement -> variableDeclaration | exprStmt | printStmt
// exprStmt  → expression ";"
// printStmt → "print" expression ";"
// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
pub(super) fn parse(
    tokens: &Vec<Token>,
) -> Result<Vec<Statement>, LanguageError> {
    let mut tokens = tokens.iter().peekable();
    let mut statements = Vec::new();
    let mut errors = Vec::new();
    while tokens.peek().is_some() {
        match statement(&mut tokens) {
            Ok(s) => statements.push(s),
            Err(err) => {
                errors.push(err);
                // NOTE:
                // we might need to have \n as end operator too alongside
                // ';' otherwise the following will never be caught
                // 1 + 2;
                // 1 + 3
                // 1 + 4
                // can't parse after 1 + 3 and gives only that
                //
                // consume tokens till a clear end
                while let Some(token) = tokens.peek() {
                    match token {
                        Token::Class
                        | Token::Fun
                        | Token::Var
                        | Token::For
                        | Token::If
                        | Token::While
                        | Token::Print
                        | Token::Return => {
                            break;
                        }
                        Token::Semicolon => {
                            _ = tokens.next();
                            break;
                        }
                        _ => {
                            _ = tokens.next();
                        }
                    }
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(statements)
    } else {
        Err(LanguageError::Parse(errors))
    }
}

// NOTE: it might be better to extract out the ";" logic into this
// but I don't know what other statements are there so leaving it
fn statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    match tokens.peek() {
        // NOTE: can not be reached because previous loop
        // ensures that it breaks on a none
        None => unreachable!(),
        Some(Token::Print) => print_statement(tokens),
        Some(Token::Var) => variable_declaration_statement(tokens),
        _ => expression_statement(tokens),
    }
}

// exprStmt  → expression ";"
fn expression_statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    let expression = expression(tokens)?;
    // last token has to be ;
    if !matches!(tokens.peek(), Some(Token::Semicolon)) {
        return Err(ParseError::UnterminatedStatement(expression.to_string()));
    }
    // consume the ;
    _ = tokens.next();
    Ok(ExpressionStatement::new(expression))
}

// printStmt → "print" expression ";"
fn print_statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    // have to be print otherwise the function should not be called
    assert_matches!(tokens.next(), Some(Token::Print));
    let expression = expression(tokens)?;
    // last token has to be ;
    if !matches!(tokens.peek(), Some(Token::Semicolon)) {
        return Err(ParseError::UnterminatedStatement(expression.to_string()));
    }
    // consume the ;
    _ = tokens.next();
    Ok(PrintStatement::new(expression))
}

// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
fn variable_declaration_statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    let mut statement_string = String::new();
    // have to be var otherwise the function should not be called
    assert_matches!(tokens.next(), Some(Token::Var));
    statement_string.push_str("var ");

    let variable_token = match tokens.next() {
        Some(token @ Token::Identifier(i)) => {
            statement_string.push_str(i);
            token
        }
        t => return Err(ParseError::IncompleteVariableDeclaration(t.cloned())),
    };

    let mut expr = None;
    if matches!(tokens.peek(), Some(Token::Equal)) {
        _ = tokens.next();
        let expr_inner = expression(tokens)?;
        statement_string.push_str(&expr_inner.to_string());
        expr = Some(expr_inner);
    }

    if !matches!(tokens.peek(), Some(Token::Semicolon)) {
        return Err(ParseError::UnterminatedStatement(statement_string));
    }
    _ = tokens.next();

    incorrect_token_error_wrap(VariableDeclarationStatement::new(
        variable_token.clone(),
        expr,
    ))
}

// grammar
// expression -> assignment
// assignment -> IDENTIFIER "=" assignment | equality
// equality   -> comparison ( ( "!=" | "==" ) comparison )*
// comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )*
// term       -> factor ( ( "-" | "+" ) factor )*
// factor     -> unary ( ( "/" | "*" ) unary )*
// unary      -> ( "!" | "-" ) unary | primary
// primary    -> NUMBER | STRING | "true" | "false" | "nil"
//              | "(" expression ")" | IDENTIFIER
pub(super) fn parse_expression(
    tokens: &Vec<Token>,
) -> Result<Expression, LanguageError> {
    let mut tokens = tokens.iter().peekable();
    let mut errors = Vec::new();
    // add a dummy expression for rust checks
    // I can put in anything and it would work
    let mut expr: Expression = LiteralExpression::new(Token::Nil)
        .expect("nil is a literal expression");
    while let Some(_) = tokens.peek() {
        match expression(&mut tokens) {
            Ok(ex) => expr = ex,
            Err(err) => {
                errors.push(err);
                // consume tokens till a clear end
                while let Some(token) = tokens.peek() {
                    match token {
                        Token::Class
                        | Token::Fun
                        | Token::Var
                        | Token::For
                        | Token::If
                        | Token::While
                        | Token::Print
                        | Token::Return => {
                            break;
                        }
                        _ => {
                            _ = tokens.next();
                        }
                    }
                }
                continue;
            }
        }
    }
    if errors.is_empty() {
        Ok(expr)
    } else {
        Err(LanguageError::Parse(errors))
    }
}

// expression -> assignment
fn expression(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    assignment(tokens)
}

// assignment -> IDENTIFIER "=" assignment | equality
fn assignment(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let expr = equality(tokens)?;
    if matches!(tokens.peek(), Some(Token::Equal)) {
        let token = tokens.next();
        assert_matches!(token, Some(Token::Equal));

        let value = assignment(tokens)?;
        match expr {
            Expression::Variable(v) => {
                return Ok(AssignmentExpression::from(v.variable, value));
            }
            _ => {
                return Err(ParseError::InvalidAssignment(expr));
            }
        }
    }
    Ok(expr)
}

// equality -> comparison ( ( "!=" | "==" ) comparison )*
fn equality(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expression = comparison(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::BangEqual | Token::EqualEqual => {
                let left_expression = expression;
                let operator = tokens
                    .next()
                    .expect("no token after peek check - equality");
                let right_expression = comparison(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
fn comparison(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expression = term(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual => {
                let left_expression = expression;
                let operator = tokens
                    .next()
                    .expect("no token after peek check - comparison");
                let right_expression = term(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// term → factor ( ( "-" | "+" ) factor )*
fn term(tokens: &mut Peekable<Iter<Token>>) -> Result<Expression, ParseError> {
    let mut expression = factor(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Minus | Token::Plus => {
                let left_expression = expression;
                let operator =
                    tokens.next().expect("no token after peek check - term");
                let right_expression = factor(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// factor → unary ( ( "/" | "*" ) unary )*
fn factor(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expression = unary(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Slash | Token::Star => {
                let left_expression = expression;
                let operator =
                    tokens.next().expect("no token after peek check - factor");
                let right_expression = unary(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// unary → ( "!" | "-" ) unary | primary
fn unary(tokens: &mut Peekable<Iter<Token>>) -> Result<Expression, ParseError> {
    match tokens.peek() {
        Some(Token::Bang) | Some(Token::Minus) => {
            let token =
                tokens.next().expect("no token after peek check - unary");
            incorrect_token_error_wrap(UnaryExpression::new(
                token.to_owned(),
                unary(tokens)?,
            ))
        }
        _ => primary(tokens),
    }
}

// primary → NUMBER | STRING | "true" | "false" | "nil"
//          | "(" expression ")" | IDENTIFIER
fn primary(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let Some(token) = tokens.next() else {
        return Err(ParseError::IncompletePrimaryExpression);
    };
    match token {
        Token::False
        | Token::True
        | Token::Nil
        | Token::Number(_)
        | Token::String(_) => {
            incorrect_token_error_wrap(LiteralExpression::new(token.to_owned()))
        }
        Token::LeftParen => {
            let expression = expression(tokens)?;
            // should end with right parenthesis
            if !matches!(tokens.next(), Some(Token::RightParen)) {
                return Err(ParseError::IncompletePrimaryExpression);
            }
            Ok(GroupingExpression::new(expression))
        }
        Token::Identifier(_) => incorrect_token_error_wrap(
            VariableExpression::new(token.to_owned()),
        ),
        // FIX: cases found till now
        // 1 + ;
        _ => Err(ParseError::Unknown(token.to_owned())),
    }
}

fn incorrect_token_error_wrap<T>(
    expression: Result<T, LanguageError>,
) -> Result<T, ParseError> {
    expression.map_err(|e| match e {
        LanguageError::IncorrectTokenConversion {
            base_token,
            converted_to,
        } => ParseError::WrongTokenForExpression {
            expression_type: converted_to,
            token: base_token,
        },
        _ => {
            unreachable!("found not handled error during expression wrap: {e}")
        }
    })
}

// for chapter 6
// solution to challenge 1
// I thought of three solutions
// 1. changing primary to
// primary → ... | "nil" | "(" expression ")" | "," expression
// but this allows expressions like ", 1 + 2"
// 2. changing equality to
// equality → comparison ( ( "!=" | "==" | "," ) comparison )*
// but this causes problems with cases like "x, y == z"
// not to mention we need to add , as a binary operator which might
// cause problems with evaluation later on - just an intuition
// 3. changing expression to
// expression -> equality ( "," equality )
// but in order to capture this we need to change the return values here
// I don't want to disturb the current type system so. otherwise,
// we have to return a vector everywhere instead of just expression

// solution to challenge 2
// ternary operator is right associative. ex,
// check1 ? true1 : check2 ? true2 : false2 boils down to
// check1 ? true1 : (check2 ? true2 : false2)
// as for precedence, it comes between equality and comparison

// solution to challenge 3
// adding error productions for binary operator is easy, just add them
// at the equality step accepting the operators
// problem is that with this typing, we can't form statements with them
// that is why i didn't add error production for + at unary
// for such cases i think having a very verbose parse error helps
