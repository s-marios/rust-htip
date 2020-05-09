use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq, Eq)]
///These are the errors that a basic parser may produce.
///The slice represents the original data that caused
///the error.
pub enum HtipError<'a> {
    ///Not enough data to parse
    TooShort,
    ///The actual length is different from what is expected
    UnexpectedLength(usize),
    ///A sequence of bytes is different from what it was expected
    NotEqual(&'a [u8]),
    ///An invalid percentage, outside the range of [0-100]
    InvalidPercentage(u8),
    ///The input data does not represent a valid MAC address
    InvalidMac(&'a [u8]),
    ///The text length is not utf8
    InvalidText(std::str::Utf8Error),
}

#[derive(Debug)]
///An enum holding the various possible types of HTIP data.
pub enum HtipData {
    ///Represents a number of up to 4 bytes, as well as percentages.
    U32(u32),
    ///Rare type, currently used for a 6byte update interval
    U64(u64),
    ///Represents textual data
    Text(String),
    ///Represents various binary data
    Binary(Vec<u8>),
}

#[derive(Debug)]
pub struct InvalidConversion(HtipData);

impl TryFrom<HtipData> for u32 {
    type Error = InvalidConversion;

    fn try_from(data: HtipData) -> Result<Self, Self::Error> {
        match data {
            HtipData::U32(value) => Ok(value),
            _ => Err(InvalidConversion(data)),
        }
    }
}

impl TryFrom<HtipData> for u64 {
    type Error = InvalidConversion;

    fn try_from(data: HtipData) -> Result<Self, Self::Error> {
        match data {
            HtipData::U64(value) => Ok(value),
            _ => Err(InvalidConversion(data)),
        }
    }
}

impl TryFrom<HtipData> for String {
    type Error = InvalidConversion;

    fn try_from(data: HtipData) -> Result<Self, Self::Error> {
        match data {
            HtipData::Text(value) => Ok(value),
            _ => Err(InvalidConversion(data)),
        }
    }
}

impl TryFrom<HtipData> for Vec<u8> {
    type Error = InvalidConversion;

    fn try_from(data: HtipData) -> Result<Self, Self::Error> {
        match data {
            HtipData::Binary(value) => Ok(value.to_vec()),
            _ => Err(InvalidConversion(data)),
        }
    }
}

///I am not sure about this API, but I'll try it out for now
impl HtipData {
    pub fn into_u32(self) -> Option<u32> {
        self.try_into().ok()
    }

    pub fn into_u64(self) -> Option<u64> {
        self.try_into().ok()
    }

    pub fn into_string(self) -> Option<String> {
        self.try_into().ok()
    }

    pub fn into_bytes(self) -> Option<Vec<u8>> {
        self.try_into().ok()
    }
}

pub trait Parser {
    fn parse<'a>(&mut self, input: &'a [u8]) -> Result<&'a [u8], HtipError<'a>>;
    fn data(&self) -> HtipData;
}

///use with the fixed-size number parser
#[derive(Clone, Copy)]
pub enum NumberSize {
    One = 1,
    Two,
    Three,
    Four,
}

///A parser for numbers of fixed size, up to four bytes
pub struct SizedNumber {
    size: NumberSize,
    value: u32,
}

impl SizedNumber {
    pub fn new(size: NumberSize) -> Self {
        SizedNumber { size, value: 0 }
    }

    fn check_length(
        expected: usize,
        actual: usize,
        remaining: usize,
    ) -> Result<(), HtipError<'static>> {
        match (actual == expected, remaining >= expected) {
            (true, true) => Ok(()),
            (true, false) => Err(HtipError::TooShort),
            (false, _) => Err(HtipError::UnexpectedLength(actual)),
        }
    }
}

impl Parser for SizedNumber {
    fn parse<'a>(&mut self, input: &'a [u8]) -> Result<&'a [u8], HtipError<'a>> {
        if input.is_empty() {
            return Err(HtipError::TooShort);
        }

        //normal processing
        //consume the length
        let actual = input[0] as usize;
        let input = &input[1..];
        //check actual, expected and remaining buffer lengths
        SizedNumber::check_length(self.size as usize, actual, input.len())?;
        //we have the size we expect, try to parse
        //this into a number
        self.value = (0..actual).fold(0u32, |mut acc, index| {
            acc <<= 8;
            acc += input[index] as u32;
            acc
        });
        //consume the bytes we used so far
        Ok(&input[actual..])
    }

    fn data(&self) -> HtipData {
        HtipData::U32(self.value)
    }
}

///A fake parser used in testing
pub struct Dummy(pub u32);

impl Parser for Dummy {
    fn parse<'a>(&mut self, input: &'a [u8]) -> Result<&'a [u8], HtipError<'a>> {
        Ok(&input[(self.0 as usize)..])
    }

    fn data(&self) -> HtipData {
        HtipData::U32(self.0)
    }
}

pub struct FixedSequence {
    key: Vec<u8>,
    //add other things if you think you need them
}

impl FixedSequence {
    pub fn new(key: Vec<u8>) -> Self {
        FixedSequence { key }
    }
}

impl Parser for FixedSequence {
    fn parse<'a>(&mut self, input: &'a [u8]) -> Result<&'a [u8], HtipError<'a>> {
        if input.is_empty() || input.len() < self.key.len(){
            return Err(HtipError::TooShort);
        }

        if self.key != input {
            let mut diff_p = 0;
            for (i,j) in self.key.iter().enumerate() {
                if j != &input[i] {
                    diff_p += i;
                    return Err(HtipError::NotEqual(&input[..diff_p+1]));
                }
            }
            
            return Ok(&input[self.key.len()..]);
        }

        // return empty Vec slice
        return Ok(&input[input.len()..]);
    }

    //return an HtipData::Binary
    fn data(&self) -> HtipData {
        HtipData::Binary(self.key.clone())
    }
}

pub struct Text {
    //add your implementation here
}

impl Text {
    pub fn new(max_size: u8) -> Self {
        unimplemented!()
    }
}

impl Parser for Text {
    fn parse<'a>(&mut self, input: &'a [u8]) -> Result<&'a [u8], HtipError<'a>> {
        unimplemented!()
    }

    //you return a String in the HtipData, copy/clone that
    fn data(&self) -> HtipData {
        unimplemented!()
    }
}

//maybe reuse FixedSequence and/or SizedNumber?
pub struct Percentage {
    value: u8,
}

impl Percentage {
    pub fn new() -> Self {
        Percentage {
            value: 0u8
        }
    }
}

impl Parser for Percentage {
    fn parse<'a>(&mut self, input: &'a [u8]) -> Result<&'a [u8], HtipError<'a>> {
        
        if input.is_empty() || input.len() < 2{
            return Err(HtipError::TooShort);
        }

        let size = input[0] as usize;

        if size != 1 {
            return Err(HtipError::UnexpectedLength(usize::from(size)));
        }

        let val = input[size];

        if  val > 100 {
            return Err(HtipError::InvalidPercentage(val));
        } else {
            self.value = input[size];
            Ok(&input[size+1..])
        }
    }

    fn data(&self) -> HtipData {
        HtipData::U32(self.value.clone().into())
    }
}