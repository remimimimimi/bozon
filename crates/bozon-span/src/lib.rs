use std::ops;

/// Main span of parser
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Span {
    start: usize,
    /// XXX: Hope that this'll be enough for everything :)
    len: u16,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);

        Self {
            start,
            len: (end - start)
                .try_into()
                .expect("oops, span out of range of u16 number :^("),
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.start + usize::from(self.len)
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl From<ops::Range<usize>> for Span {
    fn from(r: ops::Range<usize>) -> Self {
        Span::new(r.start, r.end)
    }
}

impl ops::Add for Span {
    type Output = Span;
    fn add(self, other: Self) -> Self::Output {
        let start = self.start().min(other.start());
        let len = if self.end() > other.end() {
            self.end()
        } else {
            other.end()
        } - start;

        Self {
            start,
            len: len
                .try_into()
                .expect("oops, span out of range of u16 number :^("),
        }
    }
}

impl chumsky::span::Span for Span {
    type Context = ();
    type Offset = usize;

    fn new((): Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        range.into()
    }

    fn context(&self) -> Self::Context {
        ()
    }

    fn start(&self) -> Self::Offset {
        Span::start(self)
    }

    fn end(&self) -> Self::Offset {
        Span::end(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_add() {
        let span1 = Span::new(10, 10000);
        let span2 = Span::new(100, 1000);
        assert_eq!(span1 + span2, Span::new(10, 10000));
    }
}