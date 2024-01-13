use crate::token::{Span, Token};

pub struct Context<'t, T> {
    tokens: &'t [Token<T>],
}

impl<'t, T> Context<'t, T> {
    pub fn get(&self, index: usize) -> Option<&Token<T>> {
        self.tokens.get(index)
    }

    pub fn get_required(
        &self,
        pty: &str,
        index: usize,
        optional: bool,
    ) -> Result<&Token<T>, ParseResult<T>> {
        self.get(index).ok_or_else(|| {
            if optional {
                ParseResult::None
            } else {
                ParseResult::Err(ParseError::new(
                    pty.to_string(),
                    self.span_last(),
                    "Unexpected end of input",
                ))
            }
        })
    }

    pub fn span_last(&self) -> Span {
        self.get(self.tokens.len() - 1)
            .map(|t| t.span().clone())
            .unwrap_or_default()
    }

    pub const fn new(tokens: &'t [Token<T>]) -> Self {
        Self { tokens }
    }
}

pub struct ParseError {
    expected: String,
    span: Span,
    message: &'static str,
}

impl ParseError {
    pub fn span(&self) -> &Span {
        &self.span
    }

    pub const fn from(expected: String, span: Span) -> Self {
        Self::new(expected, span, "Syntax error")
    }

    pub const fn new(expected: String, span: Span, message: &'static str) -> Self {
        Self {
            expected,
            span,
            message,
        }
    }
}

pub struct Parse<'t, T> {
    type_parsed: &'t str,
    data: ParseResult<T>,
    start_offset: usize,
    end_offset: usize,
}

impl<'t, T> Parse<'t, T> {
    pub fn size(&self) -> usize {
        self.end_offset - self.start_offset
    }

    pub fn type_parsed(&self) -> &'t str {
        self.type_parsed
    }

    pub fn data(&self) -> &ParseResult<T> {
        &self.data
    }

    pub fn start_offset(&self) -> usize {
        self.start_offset
    }

    pub fn end_offset(&self) -> usize {
        self.end_offset
    }

    pub const fn new(
        type_parsed: &'t str,
        data: ParseResult<T>,
        start_offset: usize,
        end_offset: usize,
    ) -> Self {
        Self {
            type_parsed,
            data,
            start_offset,
            end_offset,
        }
    }
}

pub enum ParseData<T> {
    Nested(Vec<ParseData<T>>),
    TokenList(Vec<Token<T>>),
    Token(Token<T>),
}

pub enum ParseResult<T> {
    Ok(ParseData<T>),
    Err(ParseError),
    None,
}

pub trait Parser<T> {
    fn parse(&self, ctx: &Context<T>, offset: usize) -> Parse<T>;
}

pub struct OfType<T> {
    pty: String,
    optional: bool,
    ty: T,
}

impl<T> OfType<T> {
    pub fn from(pty: &str, optional: bool, ty: T) -> Self {
        Self::new(pty.to_string(), optional, ty)
    }

    pub const fn new(pty: String, optional: bool, ty: T) -> Self {
        Self { pty, optional, ty }
    }
}

impl<T> Parser<T> for OfType<T>
where
    T: PartialEq + Clone,
{
    fn parse(&self, ctx: &Context<T>, offset: usize) -> Parse<T> {
        let token = match ctx.get_required(&self.pty, offset, self.optional) {
            Ok(t) => t,
            Err(e) => return Parse::new(&self.pty, e, offset, offset),
        };

        if &self.ty == token.ty() {
            return Parse::new(
                &self.pty,
                ParseResult::Ok(ParseData::Token(token.clone())),
                offset,
                offset,
            );
        }
        Parse::new(
            &self.pty,
            ParseResult::Err(ParseError::from(self.pty.to_string(), token.span().clone())),
            offset,
            offset,
        )
    }
}

pub struct Predicate<T> {
    pty: String,
    optional: bool,
    predicate: fn(&T) -> bool,
}

impl<T> Predicate<T> {
    pub fn from(pty: &str, optional: bool, predicate: fn(&T) -> bool) -> Self {
        Self::new(pty.to_string(), optional ,predicate)
    }

    pub const fn new(pty: String, optional: bool, predicate: fn(&T) -> bool) -> Self {
        Self {
            pty,
            optional,
            predicate,
        }
    }
}

impl<T> Parser<T> for Predicate<T>
where T: Clone {
    fn parse(&self, ctx: &Context<T>, offset: usize) -> Parse<T> {
        let token = match ctx.get_required(&self.pty, offset, self.optional) {
            Ok(t) => t,
            Err(e) => return Parse::new(&self.pty, e, offset, offset),
        };

        if (self.predicate)(token.ty()) {
            return Parse::new(
                &self.pty,
                ParseResult::Ok(ParseData::Token(token.clone())),
                offset,
                offset,
            );
        }
        Parse::new(
            &self.pty,
            ParseResult::Err(ParseError::from(self.pty.to_string(), token.span().clone())),
            offset,
            offset + 1,
        )
    }
}

pub struct Sequence<T> {
    pty: String,
    optional: bool,
    inner: Vec<Box<dyn Parser<T>>>,
}

impl<T> Sequence<T> {
    pub fn from(pty: &str, optional: bool, sequence: Vec<Box<dyn Parser<T>>>) -> Self {
        Self::new(pty.to_string(), optional, sequence)
    }

    pub const fn new(pty: String, optional: bool, inner: Vec<Box<dyn Parser<T>>>) -> Self {
        Self {
            pty,
            optional,
            inner,
        }
    }
}

impl<T> Parser<T> for Sequence<T> {
    fn parse(&self, ctx: &Context<T>, offset: usize) -> Parse<T> {
        let mut offs = offset;
        let mut expr = vec![];

        for item in &self.inner {
            let parse = item.parse(ctx, offs);
            let size = parse.size();

            match parse.data {
                ParseResult::Ok(d) => {
                    offs += size;
                    expr.push(d);
                }
                ParseResult::Err(e) => {
                    if self.optional {
                        return Parse::new(&self.pty, ParseResult::None, offset, offs);
                    }
                    return Parse::new(&self.pty, ParseResult::Err(e), offset, offs);
                }
                _ => (),
            }
        }
        Parse::new(
            &self.pty,
            ParseResult::Ok(ParseData::Nested(expr)),
            offset,
            offs,
        )
    }
}

pub struct Repeatable<T> {
    pty: String,
    optional: bool,
    inner: Box<dyn Parser<T>>,
}

impl<T> Repeatable<T> {
    pub fn from(pty: &str, optional: bool, repeatable: Box<dyn Parser<T>>) -> Self {
        Self::new(pty.to_string(), optional, repeatable)
    }

    pub const fn new(pty: String, optional: bool, inner: Box<dyn Parser<T>>) -> Self {
        Self {
            pty,
            optional,
            inner,
        }
    }
}

impl<T> Parser<T> for Repeatable<T> {
    fn parse(&self, ctx: &Context<T>, offset: usize) -> Parse<T> {
        let mut expr = vec![];
        let mut err = None;

        let mut offs = offset;

        loop {
            let parse = self.inner.parse(ctx, offs);

            match parse.data {
                ParseResult::Ok(data) => {
                    match &data {
                        ParseData::Nested(l) => offs += l.len(),
                        ParseData::TokenList(l) => offs += l.len(),
                        ParseData::Token(_) => offs += 1,
                    }
                    expr.push(data);
                }
                ParseResult::Err(e) => {
                    err = Some(e);
                    break;
                }
                ParseResult::None => {
                    break;
                }
            }
        }

        let data = if expr.is_empty() {
            if self.optional {
                ParseResult::None
            } else if let Some(err) = err {
                ParseResult::Err(err)
            } else {
                ParseResult::Err(ParseError::from(
                    self.pty.clone(),
                    ctx.tokens[offs].span().clone(),
                ))
            }
        } else {
            ParseResult::Ok(ParseData::Nested(expr))
        };
        Parse::new(&self.pty, data, offset, offs)
    }
}

pub struct Not<T> {
    pty: String,
    optional: bool,
    inner: Box<dyn Parser<T>>,
}

impl<T> Not<T> {
    pub const fn new(pty: String, optional: bool, inner: Box<dyn Parser<T>>) -> Self {
        Self {
            pty,
            optional,
            inner,
        }
    }
}

impl<T> Parser<T> for Not<T> {
    fn parse(&self, ctx: &Context<T>, offset: usize) -> Parse<T> {
        fn get_data_span<T>(data: &ParseData<T>) -> Span {
            match data {
                ParseData::Token(s) => s.span().clone(),
                ParseData::TokenList(l) => l.first().unwrap().span().clone(),
                ParseData::Nested(l) => get_data_span(l.first().unwrap()),
            }
        }

        let parse = self.inner.parse(ctx, offset);

        return Parse::new(
            &self.pty,
            match parse.data {
                ParseResult::Ok(data) => {
                    if self.optional {
                        ParseResult::None
                    } else {
                        let span = get_data_span(&data);
                        ParseResult::Err(ParseError::from(self.pty.clone(), span))
                    }
                }
                ParseResult::Err(_) | ParseResult::None => ParseResult::None,
            },
            parse.start_offset,
            parse.end_offset,
        );
    }
}

pub struct Choice<T> {
    pty: String,
    optional: bool,
    inner: Vec<Box<dyn Parser<T>>>,
}

impl<T> Choice<T> {
    pub fn from(pty: &str, optional: bool, inner: Vec<Box<dyn Parser<T>>>) -> Self {
        Self::new(pty.to_string(), optional, inner)
    }

    pub const fn new(pty: String, optional: bool, inner: Vec<Box<dyn Parser<T>>>) -> Self {
        Self {
            pty,
            optional,
            inner,
        }
    }
}

impl<T> Parser<T> for Choice<T> {
    fn parse(&self, ctx: &Context<T>, offset: usize) -> Parse<T> {
        for choice in &self.inner {
            let parse = choice.parse(ctx, offset);

            if let ParseResult::Ok(_) = parse.data {
                return parse;
            }
        }

        return Parse::new(
            &self.pty,
            ParseResult::Err(ParseError::from(self.pty.clone(), ctx.span_last())),
            offset,
            offset,
        );
    }
}
