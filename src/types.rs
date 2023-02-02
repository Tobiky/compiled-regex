pub trait RegExp {
    fn is_match(&self, input: &str) -> bool;
    fn matches(&self, input: &str) -> Vec<(usize, usize)>;
}

pub struct Matches<'input_lifetime> {
    pos: usize,
    inp: &'input_lifetime str,
}
