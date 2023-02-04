pub trait RegExp
where Self: Sized
{
    fn is_match(&self, input: &str) -> bool;
    fn matches(&self, input: &str) -> Matches<Self>;
    fn find_match(&self, input: &str) -> Option<(usize, usize)>;
    fn find_match_at(&self, input: &str, offset: usize) -> Option<(usize, usize)>;

    const MIN_LEN: usize;
}

pub struct Matches<'input_lifetime, R: RegExp> {
    pos: usize,
    inp: &'input_lifetime str,
    reg: R,
}

impl<'t, R: RegExp> Iterator for Matches<'t, R> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Boundry checking as well as length optimization
        // Cannot match a (sub)string that has a length shorter
        // than the minimum constructable length from the regex
        while self.pos + R::MIN_LEN < self.inp.len() {
            // Find the next matching location that starts exactly at position
            let location_match = self.reg.find_match_at(self.inp, self.pos);

            // Found a matching location, return it
            if let Some(range) = location_match {
                // Since a match was found the position needs to be updated to the
                // ending position of the match since matches are not overlapping.
                self.pos = range.1;
                return Some(range)
            } else {
                // Increment along the search string since there was not match starting
                // at this position
                self.pos += 1;
            }
        }
        None
    }
}
