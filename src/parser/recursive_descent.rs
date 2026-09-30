use std::{assert_matches, iter::Peekable, slice::Iter};

use crate::{
    error::{LanguageError, ParseError},
    expression::{
        AssignmentExpression, BinaryExpression, Expression, GroupingExpression,
        LiteralExpression, LogicalExpression, UnaryExpression,
        VariableExpression,
    },
    statement::{
        BlockStatement, ExpressionStatement, IfStatement, PrintStatement,
        Statement, VariableDeclarationStatement, WhileStatement,
    },
    token::Token,
};

// grammar
// program   -> declaration* EOF
// declaration -> variableDeclaration | statement
// statement -> exprStmt | printStmt | blockStmt | ifStmt | whileStmt | forStmt
// exprStmt  -> expression ";"
// printStmt -> "print" expression ";"
// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
// blockStmt -> "{" declaration* "}"
// ifStmt -> "if" "(" expression ")" statement ( "else" statement )?
// whileStmt -> "while" "(" expression ")" statement
// forStmt -> "for" "(" varDecl | exprStmt | ";" )
//              expression? ";" expression? ")" statement
pub(super) fn parse(
    tokens: &Vec<Token>,
) -> Result<Vec<Statement>, LanguageError> {
    let mut tokens = tokens.iter().peekable();
    let mut statements = Vec::new();
    let mut errors = Vec::new();
    while tokens.peek().is_some() {
        match declaration(&mut tokens) {
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

// declaration -> variableDeclaration | statement
fn declaration(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    match tokens.peek() {
        Some(Token::Var) => variable_declaration_statement(tokens),
        _ => statement(tokens),
    }
}

// statement -> exprStmt | printStmt | blockStmt | ifStmt
//
// I wanted to extract out the ";" logic into this
// but that went sideways due to introduction of block, if, etc.
// also our grammar specifies ";" logic into those definitions
fn statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    match tokens.peek() {
        // NOTE: can not be reached because previous loop
        // ensures that it breaks on a none
        None => unreachable!(),
        Some(Token::Print) => print_statement(tokens),
        Some(Token::LeftBrace) => block_statement(tokens),
        Some(Token::If) => if_statement(tokens),
        Some(Token::While) => while_statement(tokens),
        Some(Token::For) => for_statement(tokens),
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

// blockStmt -> "{" declaration* "}"
fn block_statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    // have to be { otherwise function should not be called
    assert_matches!(tokens.next(), Some(Token::LeftBrace));

    let mut statements = Vec::new();
    while let Some(token) = tokens.peek() {
        match token {
            Token::RightBrace => {
                assert_matches!(tokens.next(), Some(Token::RightBrace));
                return Ok(BlockStatement::new(statements));
            }
            _ => {
                statements.push(declaration(tokens)?);
            }
        }
    }
    let statements_string = statements
        .into_iter()
        .map(|statement| statement.to_string())
        .reduce(|acc, statement| acc + "; " + &statement)
        .unwrap_or_default();
    Err(ParseError::UnterminatedBlock(format!(
        "{{ {}",
        statements_string
    )))
}

// ifStmt -> if "(" expression ")" statement ( "else" statement )?
fn if_statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    // have to be if otherwise function should not be called
    assert_matches!(tokens.next(), Some(Token::If));

    if !matches!(tokens.next(), Some(Token::LeftParen)) {
        return Err(ParseError::MalformedControlStatement(
            "no left parenthesis after if keyword".to_owned(),
        ));
    }
    let condition = expression(tokens)?;
    if !matches!(tokens.next(), Some(Token::RightParen)) {
        return Err(ParseError::MalformedControlStatement(
            "no right parenthesis after if keyword".to_owned(),
        ));
    }
    let then_branch = statement(tokens)?;
    let mut else_branch = None;
    if matches!(tokens.peek(), Some(Token::Else)) {
        _ = tokens.next();
        else_branch = Some(statement(tokens)?);
    }
    Ok(IfStatement::new(condition, then_branch, else_branch))
}

// whileStmt -> "while" "(" expression ")" statement
fn while_statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    // have to be while otherwise function should not be called
    assert_matches!(tokens.next(), Some(Token::While));

    if !matches!(tokens.next(), Some(Token::LeftParen)) {
        return Err(ParseError::MalformedControlStatement(
            "no left parenthesis after while keyword".to_owned(),
        ));
    }
    let condition = expression(tokens)?;
    if !matches!(tokens.next(), Some(Token::RightParen)) {
        return Err(ParseError::MalformedControlStatement(
            "no right parenthesis after while keyword".to_owned(),
        ));
    }
    let body = statement(tokens)?;
    Ok(WhileStatement::new(condition, body))
}

// forStmt -> "for" "(" varDecl | exprStmt | ";" )
//              expression? ";" expression? ")" statement
fn for_statement(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Statement, ParseError> {
    // have to be for otherwise function should not be called
    assert_matches!(tokens.next(), Some(Token::For));

    if !matches!(tokens.next(), Some(Token::LeftParen)) {
        return Err(ParseError::MalformedControlStatement(
            "no left parenthesis after for keyword".to_owned(),
        ));
    }

    let mut initializer = None;
    match tokens.peek() {
        Some(Token::Semicolon) => {
            assert_matches!(tokens.next(), Some(Token::Semicolon));
        }
        Some(Token::Var) => {
            initializer = Some(variable_declaration_statement(tokens)?);
        }
        _ => {
            initializer = Some(expression_statement(tokens)?);
        }
    }

    let mut condition = None;
    if !matches!(tokens.peek(), Some(Token::Semicolon)) {
        condition = Some(expression(tokens)?);
    }
    if !matches!(tokens.next(), Some(Token::Semicolon)) {
        return Err(ParseError::MalformedControlStatement(
            "no ; after condition in for loop".to_owned(),
        ));
    }
    let mut increment = None;
    if !matches!(tokens.peek(), Some(Token::RightParen)) {
        increment = Some(expression(tokens)?);
    }
    if !matches!(tokens.next(), Some(Token::RightParen)) {
        return Err(ParseError::MalformedControlStatement(
            "no right parenthesis after for keyword".to_owned(),
        ));
    }
    let mut body = statement(tokens)?;

    // converting into a while statement
    // {
    //   initializer
    //   while (condition) {
    //     body
    //     increment
    //   }
    // }
    if let Some(increment) = increment {
        body =
            BlockStatement::new(vec![body, ExpressionStatement::new(increment)])
    }
    // WARN: this means if we do not use the break keyword, this loop will
    // go till infinity? That is a problem, a BIG problem
    let condition = condition.unwrap_or(incorrect_token_error_wrap(
        LiteralExpression::new(Token::True),
    )?);
    body = WhileStatement::new(condition, body);

    if let Some(initializer) = initializer {
        body = BlockStatement::new(vec![initializer, body]);
    }

    Ok(body)
}

// grammar
// expression -> assignment
// assignment -> IDENTIFIER "=" assignment | logic_or
// logic_or   -> logic_and ( "or" logic_and )*
// logic_and  -> equality ( "and" equality )*
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

// assignment -> IDENTIFIER "=" assignment | logic_or
fn assignment(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let expr = logic_or(tokens)?;
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

// logic_or -> logic_and ( "or" logic_and )*
fn logic_or(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expr = logic_and(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Or => {
                let token = tokens.next();
                assert_matches!(token, Some(Token::Or));
                let left_expression = expr;
                let token = token.unwrap();
                let right_expression = logic_and(tokens)?;
                expr = incorrect_token_error_wrap(LogicalExpression::new(
                    left_expression,
                    token.to_owned(),
                    right_expression,
                ))?;
            }
            _ => {
                break;
            }
        }
    }
    Ok(expr)
}

// logic_and -> equality ( "and" equality )*
fn logic_and(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expr = equality(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::And => {
                let token = tokens.next();
                assert_matches!(token, Some(Token::And));
                let left_expression = expr;
                let token = token.unwrap();
                let right_expression = equality(tokens)?;
                expr = incorrect_token_error_wrap(LogicalExpression::new(
                    left_expression,
                    token.to_owned(),
                    right_expression,
                ))?;
            }
            _ => {
                break;
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
