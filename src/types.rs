pub trait RegExp
where Self: Sized
{
    /// Returns true if the search string contains the RegEx pattern, otherwise false.
    ///
    /// # Examples
    ///
    /// ```rust
    /// regex!(MyRegex = "abc");
    ///
    /// fn main() {
    ///     let search = "aabcabccbabc";
    ///     for match in MyRegex::matches(search) {
    ///         println!("{}", search[match.0..match.1]);
    ///     }
    /// }
    /// ```
    fn is_match(&self, input: &str) -> bool;

    /// Finds all non-intersecting matches within the search string, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// regex!(MyRegex = "abc");
    ///
    /// fn main() {
    ///     let search = "aabcabccbabc";
    ///     for match in MyRegex::matches(search) {
    ///         println!("{}", search[match.0..match.1]);
    ///     }
    /// }
    /// ```
    fn matches(&self, input: &str) -> Matches<Self>;

    /// Finds the first match, if any, and returns it as the start and ending
    /// positions of the match in the search string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// regex!(MyRegex = "aabc");
    ///
    /// fn main() {
    ///     let match = MyRegex::matches("qeaabce");
    ///     assert_eq!(Some((2, 5)), match);
    ///
    ///     let match = MyRegex::matches("abra");
    ///     assert_eq!(None, match);
    /// }
    /// ```
    fn find_match(&self, input: &str) -> Option<(usize, usize)>;

    /// Find the match, if any, for the RegExp starting exactly at the given
    /// offset in the input string. The match is returned as the start and
    /// end positions of the match in the search string.
    ///
    /// This function is intended for internal use only.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn matches_regexp_from_start<R: RegExp>(regexp: &R, input: &str) -> bool {
    ///     regexp.find_match_at(input, 0)
    /// }
    /// ```
    fn find_match_at(&self, input: &str, offset: usize) -> Option<(usize, usize)>;

    /// The length of the minimum string that would be valid to the RegExp.
    ///
    /// This constant is intended for internal use only.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn fits_my_regexp<R: RegExp>(input: &str) -> bool {
    ///     input.len() >= R::MIN_LEN
    /// }
    /// ```
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
