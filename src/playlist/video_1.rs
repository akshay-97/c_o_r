pub struct StrSplit<'a, D>{
    remainder: Option<&'a str>,
    delimiter: D,
}

impl <'a, D> StrSplit<'a, D>{
    pub fn new(haystack: &'a str, delimiter: D) -> Self{
        Self {
            remainder : Some(haystack),
            delimiter
        }
    }
}

trait Delimiter {
    fn find_next(&self, s : &str) -> Option<(usize, usize)>;
}
impl <'a, D> Iterator for StrSplit<'a, D>
where
    D: Delimiter
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item>{
        let remainder = self.remainder.as_mut()?;
        if let Some((delim_start, delim_end)) = self.delimiter.find_next(remainder){
            let until_delim = &remainder[..delim_start];
            *remainder = &remainder[(delim_end)..];
            Some(until_delim)
        }
        else{
            self.remainder.take()
        }
    }
}


impl Delimiter for &str {
    fn find_next(&self, s : &str)->Option<(usize,usize)> {
        s.find(self).map(|start| {
            (start, start + self.len())
        })
    }
}

impl Delimiter for char{
    fn find_next(&self, s : &str) -> Option<(usize, usize)> {
        None
    }
}


