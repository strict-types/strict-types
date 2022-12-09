use amplify::num::u5;

/// Information about numeric type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NumInfo {
    /// Class of the number
    pub ty: NumTy,
    /// Size of the number, in bytes
    pub size: NumSize,
}

impl NumInfo {
    pub fn from_code(id: u8) -> Self {
        NumInfo {
            ty: NumTy::from_code(id),
            size: NumSize::from_code(id),
        }
    }

    pub fn into_code(self) -> u8 { self.ty.into_code() | self.size.into_code() }

    pub fn size(self) -> u16 { self.size.size() }
}

/// The way how the size is computed and encoded in the type id
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NumSize {
    /// Lowest 5 bits contain type size in bytes
    Bytes(u5),
    /// Lowest 5 bits contain a factor defining the size according to the
    /// equation `16 * (2 + factor)`
    Factored(u5),
}

impl NumSize {
    pub fn from_code(id: u8) -> Self {
        let code = id & 0x1F;
        match id & 0x20 / 0x20 {
            0 => NumSize::Bytes(code.try_into().expect("bit masked")),
            1 => NumSize::Factored(code.try_into().expect("bit masked")),
            _ => unreachable!(),
        }
    }

    pub fn into_code(self) -> u8 {
        match self {
            NumSize::Bytes(bytes) => bytes.as_u8(),
            NumSize::Factored(factor) => factor.as_u8() | 0x20,
        }
    }

    pub fn size(self) -> u16 {
        match self {
            NumSize::Bytes(bytes) => bytes.as_u8() as u16,
            NumSize::Factored(factor) => 2 * (factor.as_u8() as u16 + 1),
        }
    }
}

/// Class of the number type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum NumTy {
    Unsigned = 0x00,
    Signed = 0x40,
    NonZero = 0x80,
    Float = 0xC0,
}

impl NumTy {
    pub fn from_code(id: u8) -> Self {
        match id & 0xC0 {
            x if x == NumTy::Unsigned as u8 => NumTy::Unsigned,
            x if x == NumTy::Signed as u8 => NumTy::Signed,
            x if x == NumTy::NonZero as u8 => NumTy::NonZero,
            x if x == NumTy::Float as u8 => NumTy::Float,
            _ => unreachable!(),
        }
    }

    pub fn into_code(self) -> u8 { self as u8 }
}
