
#[derive(Debug)]
#[derive(Clone)]
pub struct Stream{
    pub id: String
}

impl Stream {
    pub fn new(id :String) -> Self {
        Stream{id}
    }
}