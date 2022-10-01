use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IdGenerator {
    next_id: Id,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl IdGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gen(&mut self) -> Id {
        let id = self.next_id;
        self.next_id.0 += 1;
        id
    }
}

impl Id {
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self { next_id: Id(0) }
    }
}
