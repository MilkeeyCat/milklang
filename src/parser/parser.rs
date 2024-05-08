use super::{
    expr::{BinOp, ExprBinary, ExprLit, ExprUnary, IntLitRepr, UnOp},
    type_::Type,
    Expr, Stmt, StmtVarDecl,
};
use crate::{
    lexer::{Lexer, Token},
    symtable::SymbolTable,
};

pub struct Parser {
    lexer: Lexer,
    symtable: SymbolTable,
    cur_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        Self {
            cur_token: lexer.next_token().unwrap(),
            peek_token: lexer.next_token().unwrap(),
            symtable: SymbolTable::new(),
            lexer,
        }
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token().unwrap();
    }

    fn cur_token_is(&self, token: Token) -> bool {
        self.cur_token == token
    }

    fn peek_token_is(&self, token: Token) -> bool {
        self.peek_token == token
    }

    fn expect_peek(&mut self, token: Token) {
        if self.peek_token_is(token.clone()) {
            self.next_token();
        } else {
            panic!("Expected: {:?}, got: {:?}", token, self.peek_token);
        }
    }

    pub fn into_parts(mut self) -> (Vec<Stmt>, SymbolTable) {
        (self.parse(), self.symtable)
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        while !&self.cur_token_is(Token::Eof) {
            stmts.push(self.stmt());
            self.next_token();
        }

        stmts
    }

    fn expr(&mut self, precedence: u8) -> Expr {
        let mut left = match &self.cur_token {
            Token::Ident(_) => self.ident(),
            Token::Integer(_) => self.int_lit(),
            Token::Minus | Token::Bang => self.unary_expr(),
            Token::LParen => self.grouped_bin_expr(),
            token => panic!("Failed to parse prefix token {:?}", token),
        };

        while !self.peek_token_is(Token::Semicolon) && precedence < self.peek_token.precedence() {
            self.next_token();

            left = match &self.cur_token {
                Token::Plus | Token::Minus | Token::Asterisk | Token::Slash => self.bin_expr(left),
                token => panic!("Failed to parse infix token {:?}", token),
            }
        }

        left
    }

    fn stmt(&mut self) -> Stmt {
        match &self.cur_token {
            Token::U8 | Token::I8 => {
                let type_ = self.parse_type();

                self.var_decl(type_)
            }
            token => {
                let expr = Stmt::Expr(self.expr(token.precedence()));
                self.expect_peek(Token::Semicolon);

                expr
            }
        }
    }

    fn parse_type(&mut self) -> Type {
        let token = self.cur_token.clone();
        self.next_token();

        match token {
            Token::U8 => Type::U8,
            Token::I8 => Type::I8,
            token => panic!("Couldn't parse type: {:?}", token),
        }
    }

    fn var_decl(&mut self, type_: Type) -> Stmt {
        let name;

        if let Token::Ident(ident) = &self.cur_token {
            name = ident.to_string();
        } else {
            panic!("Expected: ident, got: {:?}", self.cur_token);
        }

        self.next_token();

        Stmt::VarDecl(StmtVarDecl::new(type_, name, None))
    }

    fn ident(&mut self) -> Expr {
        if let Token::Ident(ident) = &self.cur_token {
            Expr::Ident(ident.to_owned())
        } else {
            panic!("Expected ident, got: {:?}", self.cur_token);
        }
    }

    fn int_lit(&mut self) -> Expr {
        if let Token::Integer(num_str) = &self.cur_token {
            Expr::Lit(ExprLit::Int(IntLitRepr::new(num_str.to_owned())))
        } else {
            panic!("Expected integer literal, got: {:?}", self.cur_token);
        }
    }

    fn bin_expr(&mut self, left: Expr) -> Expr {
        let token = self.cur_token.clone();
        self.next_token();

        Expr::Binary(ExprBinary::new(
            BinOp::from(&token),
            Box::new(left),
            Box::new(self.expr(token.precedence())),
        ))
    }

    fn unary_expr(&mut self) -> Expr {
        let expr = Expr::Unary(ExprUnary::new(
            UnOp::from(&self.cur_token),
            Box::new(self.expr(self.cur_token.precedence())),
        ));
        self.next_token();

        expr
    }

    fn grouped_bin_expr(&mut self) -> Expr {
        self.next_token();

        let expr = self.expr(Token::LParen.precedence());
        self.expect_peek(Token::RParen);

        expr
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    use crate::lexer::Lexer;

    #[test]
    fn parse_arithmetic_expression() {
        let input = "1 * 2 + 3 / (4 + 1);";
        let mut parser = Parser::new(Lexer::new(input.to_string()));

        //assert_eq!(
        //    parser.parse_expression(Precedence::default()),
        //    Expr::Binary(ExprBinary::new(
        //        BinOp::Add,
        //        Box::new(Expr::Binary(ExprBinary::new(
        //            BinOp::Mul,
        //            Box::new(Expr::Lit(ExprLit::Int(IntLitRepr::from_string(
        //                "1".to_string()
        //            )))),
        //            Box::new(Expr::Lit(ExprLit::Int(IntLitRepr::from_string(
        //                "2".to_string()
        //            ))))
        //        ))),
        //        Box::new(Expr::Binary(ExprBinary::new(
        //            BinOp::Div,
        //            Box::new(Expr::Lit(ExprLit::Int(IntLitRepr::from_string(
        //                "3".to_string()
        //            )))),
        //            Box::new(Expr::Binary(ExprBinary::new(
        //                BinOp::Add,
        //                Box::new(Expr::Lit(ExprLit::Int(IntLitRepr::from_string(
        //                    "4".to_string()
        //                )))),
        //                Box::new(Expr::Lit(ExprLit::Int(IntLitRepr::from_string(
        //                    "1".to_string()
        //                )))),
        //            )))
        //        )))
        //    ))
        //);
    }
}
