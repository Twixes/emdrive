use std::{
    convert::{From, TryFrom},
    fmt::Debug,
    mem, str,
};

use crate::construct::components::{DataInstance, DataInstanceRaw, DataType, DataTypeRaw};

// Important note: all data stored on disk by Emdrive is big-endian. Use `from_be_bytes` and `to_be_bytes` methods.

// Read-only blob that is being decoded.
pub type ReadBlob<'b> = &'b [u8];
// Read-write blob used for encoding.
pub type WriteBlob = Vec<u8>;
/// Length of variable-length value.
pub type VarLen = u16;
/// Page index.
pub type PageIndex = u32;
/// A count that pertains to a single row (e.g. rows inside a leaf).
pub type LocalCount = u16;
/// A count that pertains to possibly more than a single row (e.g. rows in all leaf children of a node).
pub type GlobalCount = u64;

/// Trait for reading data from blobs.
pub trait Encodable: Sized {
    /// Extract value from blob in an optimized way, returning the rest of the blob for futher processing.
    fn try_decode<'b>(blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String>;

    /// How many bytes are needed to encode this value.
    fn encoded_size(&self) -> usize;

    /// Encode and write this value to blob at specified position.
    fn encode(self, blob: &mut WriteBlob, position: usize) -> usize;

    /// Like `encode`, but writing to the end of the blob.
    fn encode_back(self, blob: &mut WriteBlob, position: usize) -> usize {
        let encoded_size = self.encoded_size();
        self.encode(blob, blob.len() - encoded_size - position)
    }
}

pub trait EncodableWithAssumption<'b>: Sized {
    type Assumption;

    /// Like `Encodable::try_decode`, but with `assumption` which allows for contextful decoding.
    fn try_decode_assume(
        blob: ReadBlob<'b>,
        assumption: Self::Assumption,
    ) -> Result<(Self, ReadBlob<'b>), String>;
}

macro_rules! encodable_number_impl {
    ($($t:ty)*) => ($(
        impl Encodable for $t {
            fn try_decode<'b>(blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String> {
                Ok((
                    Self::from_be_bytes(
                        unsafe {
                            *(blob[..mem::size_of::<Self>()].as_ptr() as *const [u8; mem::size_of::<Self>()])
                        }
                    ),
                    &blob[mem::size_of::<Self>()..]
                ))
            }

            fn encode(self, blob: &mut WriteBlob, position: usize) -> usize {
                let advanced_position = position + self.encoded_size();
                blob.splice(position..advanced_position, self.to_be_bytes());
                advanced_position
            }

            fn encoded_size(&self) -> usize {
                mem::size_of::<Self>()
            }
        }
    )*)
}

encodable_number_impl! { isize i8 i16 i32 i64 i128 usize u16 u32 u64 u128 }

// u8 is a special case, as it can be used in blobs with zero transformation
impl Encodable for u8 {
    fn try_decode<'b>(blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String> {
        Ok((blob[0], &blob[1..]))
    }

    fn encode(self, blob: &mut WriteBlob, position: usize) -> usize {
        blob[position] = self;
        position + 1
    }

    fn encoded_size(&self) -> usize {
        1
    }
}

impl Encodable for bool {
    fn try_decode<'b>(blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String> {
        Ok((blob[0] != 0, &blob[1..]))
    }

    fn encode(self, blob: &mut WriteBlob, position: usize) -> usize {
        blob[position] = self as u8;
        position + 1
    }

    fn encoded_size(&self) -> usize {
        1
    }
}

impl Encodable for String {
    fn try_decode<'b>(blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String> {
        let (char_count, rest) = VarLen::try_decode(blob)?;
        let char_count_idx = usize::from(char_count);
        match str::from_utf8(&rest[..char_count_idx]) {
            Ok(ok) => Ok((ok.to_string(), &rest[char_count_idx..])),
            Err(err) => Err(err.to_string()),
        }
    }

    fn encode(self, blob: &mut WriteBlob, position: usize) -> usize {
        let char_count = self.len();
        let position = VarLen::try_from(char_count).unwrap().encode(blob, position);
        let advanced_position = position + char_count;
        blob.splice(position..advanced_position, self.as_bytes().to_owned());
        advanced_position
    }

    fn encoded_size(&self) -> usize {
        mem::size_of::<VarLen>() + self.len()
    }
}

impl Encodable for DataInstanceRaw {
    fn try_decode<'b>(_blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String> {
        panic!("`try_decode` would be too ambiguous for `DataInstanceRaw` - `try_decode_assume` should be used instead")
    }

    fn encode(self, blob: &mut WriteBlob, position: usize) -> usize {
        match self {
            Self::UInt8(value) => value.encode(blob, position),
            Self::UInt16(value) => value.encode(blob, position),
            Self::UInt32(value) => value.encode(blob, position),
            Self::UInt64(value) => value.encode(blob, position),
            Self::UInt128(value) => value.encode(blob, position),
            Self::Bool(value) => value.encode(blob, position),
            Self::Timestamp(value) => value.encode(blob, position),
            Self::Uuid(value) => value.encode(blob, position),
            Self::String(value) => value.encode(blob, position),
        }
    }

    fn encoded_size(&self) -> usize {
        match self {
            Self::UInt8(value) => value.encoded_size(),
            Self::UInt16(value) => value.encoded_size(),
            Self::UInt32(value) => value.encoded_size(),
            Self::UInt64(value) => value.encoded_size(),
            Self::UInt128(value) => value.encoded_size(),
            Self::Bool(value) => value.encoded_size(),
            Self::Timestamp(value) => value.encoded_size(),
            Self::Uuid(value) => value.encoded_size(),
            Self::String(value) => value.encoded_size(),
        }
    }
}

impl<'b> EncodableWithAssumption<'b> for DataInstanceRaw {
    type Assumption = DataTypeRaw;

    fn try_decode_assume(
        blob: ReadBlob<'b>,
        assumption: Self::Assumption,
    ) -> Result<(Self, ReadBlob<'b>), String> {
        match assumption {
            DataTypeRaw::UInt8 => {
                let (value, rest) = u8::try_decode(blob)?;
                Ok((DataInstanceRaw::UInt8(value), rest))
            }
            DataTypeRaw::UInt16 => {
                let (value, rest) = u16::try_decode(blob)?;
                Ok((DataInstanceRaw::UInt16(value), rest))
            }
            DataTypeRaw::UInt32 => {
                let (value, rest) = u32::try_decode(blob)?;
                Ok((DataInstanceRaw::UInt32(value), rest))
            }
            DataTypeRaw::UInt64 => {
                let (value, rest) = u64::try_decode(blob)?;
                Ok((DataInstanceRaw::UInt64(value), rest))
            }
            DataTypeRaw::UInt128 => {
                let (value, rest) = u128::try_decode(blob)?;
                Ok((DataInstanceRaw::UInt128(value), rest))
            }
            DataTypeRaw::Bool => {
                let (value, rest) = bool::try_decode(blob)?;
                Ok((DataInstanceRaw::Bool(value), rest))
            }
            DataTypeRaw::Timestamp => {
                let (value, rest) = i64::try_decode(blob)?;
                Ok((DataInstanceRaw::Timestamp(value), rest))
            }
            DataTypeRaw::Uuid => {
                let (value, rest) = u128::try_decode(blob)?;
                Ok((DataInstanceRaw::Uuid(value), rest))
            }
            DataTypeRaw::String => {
                let (value, rest) = String::try_decode(blob)?;
                Ok((DataInstanceRaw::String(value), rest))
            }
        }
    }
}

impl Encodable for DataInstance {
    fn try_decode<'b>(_blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String> {
        panic!("`try_decode` would be too ambiguous for `DataInstance` – use `try_decode_assume` instead")
    }

    fn encode(self, blob: &mut WriteBlob, position: usize) -> usize {
        match self {
            Self::Null => true.encode(blob, position), // true signifies NULL
            Self::Nullable(value) => {
                let position = false.encode(blob, position); // false signifies non-NULL
                value.encode(blob, position)
            }
            Self::Direct(value) => value.encode(blob, position),
        }
    }

    fn encoded_size(&self) -> usize {
        match self {
            Self::Null => mem::size_of::<Self>(),
            Self::Nullable(value) => mem::size_of::<Self>() + value.encoded_size(),
            Self::Direct(value) => value.encoded_size(),
        }
    }
}

impl<'b> EncodableWithAssumption<'b> for DataInstance {
    type Assumption = &'b DataType;

    fn try_decode_assume(
        blob: ReadBlob<'b>,
        assumption: Self::Assumption,
    ) -> Result<(Self, ReadBlob<'b>), String> {
        if assumption.is_nullable {
            let (null_marker, rest) = bool::try_decode(blob)?;
            if null_marker {
                let (value, rest) = DataInstanceRaw::try_decode_assume(rest, assumption.raw_type)?;
                Ok((DataInstance::Nullable(value), rest))
            } else {
                Ok((DataInstance::Null, rest))
            }
        } else {
            let (value, rest) = DataInstanceRaw::try_decode_assume(blob, assumption.raw_type)?;
            Ok((DataInstance::Direct(value), rest))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Row(pub Vec<DataInstance>);

impl Encodable for Row {
    fn try_decode<'b>(_blob: ReadBlob<'b>) -> Result<(Self, ReadBlob<'b>), String> {
        panic!("`try_decode` would be too ambiguous for `Row` - `try_decode_assume` should be used instead")
    }

    fn encode(self, blob: &mut WriteBlob, mut position: usize) -> usize {
        for value in self.0 {
            position = value.encode(blob, position)
        }
        position
    }

    fn encoded_size(&self) -> usize {
        self.0.iter().map(|value| value.encoded_size()).sum()
    }
}

impl<'b> EncodableWithAssumption<'b> for Row {
    type Assumption = Vec<&'b DataType>;

    fn try_decode_assume(
        mut blob: ReadBlob<'b>,
        data_types: Self::Assumption,
    ) -> Result<(Self, ReadBlob<'b>), String> {
        let mut values: Vec<DataInstance> = Vec::with_capacity(data_types.len());
        for data_type in data_types {
            let decode_result = DataInstance::try_decode_assume(blob, data_type)?;
            values.push(decode_result.0);
            blob = decode_result.1;
        }
        Ok((Self(values), blob))
    }
}
