pub trait RegExp {
    fn is_match(&self, input: &str) -> bool;
    fn matches(&self, input: &str) -> Matches<Self>;
    fn find_match(&self, input: &str) -> Option<(usize, usize)>;
}

pub struct Matches<'input_lifetime, R: RegExp> {
    pos: usize,
    inp: &'input_lifetime str,
    reg: R,
}

impl<'t, R: RegExp> Iterator for Matches<'t, R> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.reg.find_match(self[self.pos..])
            .map(|x| {
                self.pos += x.1;
                x
            })
    }
}
