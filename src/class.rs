pub trait RegExpClass {
    fn matches(&self, input: &str) -> Vec<(usize, usize)>;

}
