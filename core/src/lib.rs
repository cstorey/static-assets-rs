pub struct Asset<'a> {
    pub content: &'a [u8],
}

pub struct Map<'a> {
    pub members: &'a [(&'a str, Asset<'a>)],
}

impl<'a> Map<'a> {
    pub fn get(&self, s: &str) -> Option<&'a Asset<'a>> {
        self.members
            .binary_search_by_key(&s, |(k, _)| *k)
            .ok()
            .map(|idx| &self.members[idx].1)
    }
}
