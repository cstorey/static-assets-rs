#[derive(Debug, Clone)]
pub struct Asset<'a> {
    pub name: &'a str,
    pub content: &'a [u8],
    pub content_type: &'a str,
    pub digest: &'a [u8],
}

pub struct Map<'a> {
    pub members: &'a [Asset<'a>],
}

pub struct MapIter<'a>(::std::slice::Iter<'a, Asset<'a>>);

impl<'a> Map<'a> {
    pub fn get(&self, s: &str) -> Option<&'a Asset<'a>> {
        self.members
            .binary_search_by_key(&s, |a| a.name)
            .ok()
            .map(|idx| &self.members[idx])
    }

    pub fn iter(&self) -> MapIter<'a> {
        MapIter(self.members.iter())
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = Asset<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().cloned()
    }
}
