pub struct IndexedCharIterator<I: Iterator<Item = char>> {
    internal_iter: I,
    index: usize,
    line: usize,
    column: usize,
}

impl<I> IndexedCharIterator<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(internal_iter: I) -> Self {
        Self {
            internal_iter,
            index: 0,
            line: 1,
            column: 1,
        }
    }
}

pub struct IndexedChar {
    pub value: char,
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl<I> Iterator for IndexedCharIterator<I>
where
    I: Iterator<Item = char>,
{
    type Item = IndexedChar;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.internal_iter.next()?;
        let indexed_char = IndexedChar {
            value,
            index: self.index,
            line: self.line,
            column: self.column,
        };
        self.index += 1;
        if value == '\n' {
            self.column = 1;
            self.line += 1;
        } else {
            self.column += 1;
        };
        return Some(indexed_char);
    }
}
