pub struct Map<'a> {
    pub members: &'a [(&'a str, ())],
}

impl<'a> Map<'a> {
    pub fn get(&self, s: &str) -> Option<&'a ()> {
        self.members
            .binary_search_by_key(&s, |(k, _)| *k)
            .ok()
            .map(|idx| &self.members[idx].1)
    }
}
