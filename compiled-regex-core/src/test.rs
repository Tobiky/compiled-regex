#[allow(non_camel_case_types)] #[allow(non_snake_case)] mod __m__ca978112ca1bbdcafac231b39a23dc4d {

pub struct __ca978112ca1bbdcafac231b39a23dc4d();

impl crate::types::RegExp for __ca978112ca1bbdcafac231b39a23dc4d {
    const MIN_LEN: usize = 1;

    #[inline(always)]
    fn find_match_at(input: &str, offset: usize) -> Option<(usize, usize)> {

        if input.chars().nth(offset) == Some('a') {
            Some((offset, offset))
        } else {
            None
        }
    }

    #[inline(always)]
    fn find_match(input: &str) -> Option<(usize, usize)> {

        let mut i = 0;
        for c in input.chars() {
            if c == 'a' {
                return Some((i, i))
            }
            i += 1;
        }
        None
    }

    #[inline(always)]
    fn is_match_at(input: &str, offset: usize) -> bool {
        input.chars().nth(offset) == Some('a')
    }

    #[inline(always)]
    fn is_match(input: &str) -> bool {
        input.contains('a')
    }
}
}
type MyRegex = __m__ca978112ca1bbdcafac231b39a23dc4d::__ca978112ca1bbdcafac231b39a23dc4d;
