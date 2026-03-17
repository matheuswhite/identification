use crate::identification::first_order::sundaresan::Sundaresan;

#[allow(unused)]
pub struct Krishnaswamy(pub Sundaresan);

impl AsRef<Sundaresan> for Krishnaswamy {
    fn as_ref(&self) -> &Sundaresan {
        &self.0
    }
}

impl AsMut<Sundaresan> for Krishnaswamy {
    fn as_mut(&mut self) -> &mut Sundaresan {
        &mut self.0
    }
}
