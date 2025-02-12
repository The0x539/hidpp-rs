use tinystr::TinyAsciiStr;

pub(crate) type Buf = crate::Packet;

#[fortuples::auto_impl]
pub trait Encode {
    fn encode(&self, buf: &mut Buf);
}

impl Encode for u8 {
    fn encode(&self, buf: &mut Buf) {
        buf.push(*self);
    }
}

impl Encode for u16 {
    fn encode(&self, buf: &mut Buf) {
        buf.extend(self.to_be_bytes());
    }
}

impl<T: Encode + ?Sized> Encode for &T {
    fn encode(&self, buf: &mut Buf) {
        T::encode(*self, buf);
    }
}

pub trait Decode: Sized {
    fn decode(buf: &mut &[u8]) -> Self;
}

impl Decode for u8 {
    fn decode(buf: &mut &[u8]) -> Self {
        let b = buf[0];
        *buf = &buf[1..];
        b
    }
}

impl Decode for u16 {
    fn decode(buf: &mut &[u8]) -> Self {
        let msb = u8::decode(buf);
        let lsb = u8::decode(buf);
        u16::from_be_bytes([msb, lsb])
    }
}

impl Decode for bool {
    fn decode(buf: &mut &[u8]) -> Self {
        u8::decode(buf) != 0
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    fn decode(buf: &mut &[u8]) -> Self {
        std::array::from_fn(|_| T::decode(buf))
    }
}

impl<const N: usize> Decode for TinyAsciiStr<N> {
    fn decode(buf: &mut &[u8]) -> Self {
        let (head, tail) = buf.split_at(N);
        *buf = tail;
        Self::try_from_raw(head.try_into().unwrap()).unwrap()
    }
}

fortuples::fortuples! {
    impl Decode for #Tuple
    where
        #(#Member: Decode),*
    {
        fn decode(_buf: &mut &[u8]) -> Self {
            (#(#Member::decode(_buf),)*)
        }
    }
}

macro_rules! encode_to_primitive {
    ($struct:ty as $int:ty) => {
        impl $crate::encode::Encode for $struct {
            fn encode(&self, buf: &mut $crate::encode::Buf) {
                <$int>::from(*self).encode(buf);
            }
        }
    };
}

macro_rules! decode_from_primitive {
    ($struct:ty as $int:ty) => {
        impl $crate::encode::Decode for $struct {
            fn decode(buf: &mut &[u8]) -> Self {
                <$int>::decode(buf).into()
            }
        }
    };
}

macro_rules! derive_decode {
    ($struct:ty, $($field:ident),* $(,)?) => {
        impl $crate::encode::Decode for $struct {
            fn decode(buf: &mut &[u8]) -> Self {
                Self {
                    $($field: Decode::decode(buf),)*
                }
            }
        }
    }
}
