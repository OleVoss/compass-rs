#[derive(Debug, Default, PartialEq, Eq)]
pub struct RouteData {
    // TODO: address struct instead of tuple
    pub entries: Vec<(u16, u16)>,
}

impl RouteData {
    pub fn add(&mut self, addr: (u16, u16)) {
        self.entries.push(addr);
    }
}

#[derive(Debug, Default)]
pub struct RouteEntry {
    from: (u16, u16),
    to: (u16, u16),
}
