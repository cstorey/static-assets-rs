pub struct Asset<'a> {
    pub name: &'a str,
    pub content: &'a [u8],
    pub content_type: &'a str,
}

pub struct Map<'a> {
    pub members: &'a [Asset<'a>],
}

impl<'a> Map<'a> {
    pub fn get(&self, s: &str) -> Option<&'a Asset<'a>> {
        self.members
            .binary_search_by_key(&s, |a| a.name)
            .ok()
            .map(|idx| &self.members[idx])
    }
}
