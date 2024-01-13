use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct Span {
    ln: usize,
    cs: usize,
    ce: usize,
}

impl Span {
    pub fn ln(&self) -> usize {
        self.ln
    }

    pub fn cs(&self) -> usize {
        self.cs
    }

    pub fn ce(&self) -> usize {
        self.ce
    }

    pub const fn new(ln: usize, cs: usize, ce: usize) -> Self {
        Self { ln, cs, ce }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(1, 1, 1)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}-{}", self.ln, self.cs, self.ce)
    }
}

pub struct Token<T> {
    ty: T,
    span: Span,
}

impl<T> Token<T> {
    pub const fn span_size(&self) -> usize {
        self.span.ce - self.span.cs + 1
    }

    pub const fn ty(&self) -> &T {
        &self.ty
    }

    pub const fn span(&self) -> &Span {
        &self.span
    }

    pub const fn new(ty: T, span: Span) -> Self {
        Self { ty, span }
    }
}

impl<T> Clone for Token<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.ty.clone(), self.span.clone())
    }
}
