// https://stackoverflow.com/questions/38821671/how-can-slices-be-split-using-another-slice-as-a-delimiter

#[derive(Debug)]
pub struct SplitSubsequence<'a, 'b, T: 'a + 'b> {
    slice: &'a [T],
    needle: &'b [T],
    ended: bool,
}

impl<'a, 'b, T: 'a + 'b + PartialEq> Iterator for SplitSubsequence<'a, 'b, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            None
        } else if self.slice.is_empty() {
            self.ended = true;
            Some(self.slice)
        } else if let Some(p) = self
            .slice
            .windows(self.needle.len())
            .position(|w| w == self.needle)
        {
            let item = &self.slice[..p];
            self.slice = &self.slice[p + self.needle.len()..];
            Some(item)
        } else {
            let item = self.slice;
            self.slice = &self.slice[self.slice.len() - 1..];
            Some(item)
        }
    }
}

pub fn split_subsequence<'a, 'b, T>(slice: &'a [T], needle: &'b [T]) -> SplitSubsequence<'a, 'b, T>
where
    T: 'a + 'b + PartialEq,
{
    SplitSubsequence {
        slice: slice,
        needle: needle,
        ended: false,
    }
}
