#[derive(Debug)]
pub struct Cursor<T: Clone> {
    pub stream: Vec<T>,
    index: usize,
}

impl<T: Clone> Iterator for Cursor<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.stream.get(self.index).map(|token| {
            self.index += 1;
            token.clone()
        })
    }
}

impl<T: Clone> Cursor<T> {
    pub fn new(stream: Vec<T>) -> Self {
        Cursor { stream, index: 0 }
    }

    // pub fn next_ref(&mut self) -> Option<&Token> {
    //     self.stream.get(self.index).map(|token| {
    //         self.index += 1;
    //         token
    //     })
    // }

    // pub fn index(&self) -> usize {
    //     self.index
    // }

    // pub fn append(&mut self, new_stream: Vec<Token>) {
    //     if new_stream.is_empty() {
    //         return;
    //     }
    //     let index = self.index;
    //     let stream = std::mem::take(&mut self.stream);
    //     *self = Cursor::new(vec![stream, new_stream].concat());
    //     self.index = index;
    // }

    // pub fn look_ahead(&self, n: usize) -> Option<&Token> {
    //     self.stream[self.index..].get(n).map(|token| token)
    // }
}
