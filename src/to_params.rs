type Buf = crate::Packet;

#[fortuples::auto_impl]
pub trait ToParams {
    fn encode(&self, buf: &mut Buf);
}

impl ToParams for u8 {
    fn encode(&self, buf: &mut Buf) {
        buf.push(*self);
    }
}

impl ToParams for u16 {
    fn encode(&self, buf: &mut Buf) {
        buf.extend(self.to_be_bytes());
    }
}

impl<T: ToParams + ?Sized> ToParams for &T {
    fn encode(&self, buf: &mut Buf) {
        T::encode(*self, buf);
    }
}
