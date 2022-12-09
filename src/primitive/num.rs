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
    pub fn unsigned(bytes: u16) -> Self {
        NumInfo {
            ty: NumTy::Unsigned,
            size: NumSize::from_bytes(bytes),
        }
    }

    pub fn signed(bytes: u16) -> Self {
        NumInfo {
            ty: NumTy::Signed,
            size: NumSize::from_bytes(bytes),
        }
    }

    pub fn non_zero(bytes: u16) -> Self {
        NumInfo {
            ty: NumTy::NonZero,
            size: NumSize::from_bytes(bytes),
        }
    }

    pub fn float(bytes: u16) -> Self {
        NumInfo {
            ty: NumTy::Float,
            size: NumSize::from_bytes(bytes),
        }
    }

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
    pub fn from_bytes(bytes: u16) -> Self {
        if bytes < 0x20 {
            NumSize::Bytes(u5::try_from(bytes as u8).expect("< 0x20"))
        } else if bytes % 16 != 0 {
            panic!(
                "for more than 256 bits it is required to have the number of bits proportional to \
                 128"
            )
        } else {
            NumSize::Factored(
                u5::try_from((bytes / 16 - 2) as u8).expect("number of bytes too high"),
            )
        }
    }

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
