use std::marker::PhantomData;

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
    fn is_match(input: &str) -> bool;


    /// Find if there is a match for the RegExp starting exactly at the given
    /// offset in the input string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// regex!(MyRegex = "a\w");
    ///
    /// fn matches_regexp_from_start(input: &str) -> bool {
    ///     MyRegex::find_match_at(input, 0)
    /// }
    /// ```
    fn is_match_at(input: &str, offset: usize) -> bool;

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
    fn matches(input: &str) -> Matches<Self> {
        Matches { pos: 0, inp: input, reg: PhantomData }
    }

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
    fn find_match(input: &str) -> Option<(usize, usize)>;

    /// Find the match, if any, for the RegExp starting exactly at the given
    /// offset in the input string. The match is returned as the start and
    /// end positions of the match in the search string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// regex!(MyRegex = "aabc");
    ///
    /// fn matches_regexp_from_start(input: &str) -> bool {
    ///     MyRegex::find_match_at(input, 0)
    /// }
    /// ```
    fn find_match_at(input: &str, offset: usize) -> Option<(usize, usize)>;

    /// The length of the minimum string that would be valid to the RegExp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// regex!(MyRegex = "aabc");
    ///
    /// fn fits_my_regexp(input: &str) -> bool {
    ///     input.len() >= MyRegex::MIN_LEN
    /// }
    /// ```
    const MIN_LEN: usize;
}

pub struct Matches<'input_lifetime, R: RegExp> {
    pos: usize,
    inp: &'input_lifetime str,
    reg: PhantomData<R>,
}

impl<'t, R: RegExp> Iterator for Matches<'t, R> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Boundry checking as well as length optimization
        // Cannot match a (sub)string that has a length shorter
        // than the minimum constructable length from the regex
        while self.pos + R::MIN_LEN < self.inp.len() {
            // Find the next matching location that starts exactly at position
            let location_match = R::find_match_at(self.inp, self.pos);

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

#[derive(Debug)]
pub enum CompileError {
    UnexpectedToken(usize, usize)
}

