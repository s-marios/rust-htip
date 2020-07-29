use crate::parsers::*;
use crate::subkeys::*;
use crate::*;
use std::cmp::Ordering;

const TTC_OUI: &[u8; 3] = b"\xe0\x27\x1a";

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct ParserKey {
    tlv_type: u8,
    prefix: Vec<u8>,
}

impl ParserKey {
    pub fn new(tlv_type: u8, prefix: Vec<u8>) -> Self {
        ParserKey { tlv_type, prefix }
    }

    fn cmp_contents(key_val: &[u8], tlv_val: &[u8]) -> Ordering {
        let diff = key_val
            .iter()
            .zip(tlv_val)
            .map(|(a, b)| a.cmp(b))
            .find(|&cmp_res| cmp_res != Ordering::Equal);
        let key_is_shorter = key_val.len() <= tlv_val.len();

        match (diff, key_is_shorter) {
            (Some(cmp_res), _) => cmp_res,
            (None, true) => Ordering::Equal,
            (None, false) => Ordering::Greater,
        }
    }
}

impl LexOrder<TLV<'_>> for ParserKey {
    fn lex_cmp(&self, other: &TLV<'_>) -> Ordering {
        match self.tlv_type.cmp(&other.tlv_type().into()) {
            Ordering::Equal => ParserKey::cmp_contents(&self.prefix, other.value()),
            other => other,
        }
    }
}

pub struct Dispatcher<'a> {
    parsers: Storage<ParserKey, TLV<'a>, Box<dyn Parser>>,
}

impl Dispatcher<'_> {
    fn add_parser(&mut self, tlv_type: TlvType, key: Vec<u8>, parser: Box<dyn Parser>) {
        let key = ParserKey::new(tlv_type.into(), key);

        if self.parsers.insert(key, parser).is_some() {
            //Just panic here, we probably did a bad registration
            panic!("overwriting a parser!");
        }
    }

    fn add_htip_parser(&mut self, key: Vec<u8>, parser: Box<dyn Parser>) {
        let mut prefix = TTC_OUI.to_vec();
        prefix.extend(key);
        self.add_parser(TlvType::Custom, prefix, parser);
    }

    fn empty() -> Self {
        Dispatcher {
            parsers: Storage::new(),
        }
    }

    pub fn parse_tlv<'a, 's>(
        &mut self,
        tlv: &'a TLV<'s>,
    ) -> (ParserKey, Result<ParseData, ParsingError<'s>>) {
        //get key
        let key = self.parsers.key_of(tlv).unwrap();
        //skipping data related to the key
        let skip = key.prefix.len();
        let parser = self.parsers.get_mut(&key).unwrap();
        //setup context(take skip into account)
        let mut context = Context::new(&tlv.value[skip..]);
        (key.clone(), parser.parse(&mut context))
    }

    pub fn new() -> Self {
        let mut instance = Dispatcher::empty();
        instance.add_parser(TlvType::from(1u8), b"".to_vec(), Box::new(TypedData::new()));
        instance.add_parser(TlvType::from(2u8), b"".to_vec(), Box::new(TypedData::new()));
        instance.add_parser(
            TlvType::from(3u8),
            b"".to_vec(),
            Box::new(Number::new(NumberSize::Two)),
        );
        //this is "whatever stated in the first byte (maximum length 255)"
        instance.add_htip_parser(b"\x01\x01".to_vec(), Box::new(SizedText::new(255)));
        //this should be "exact length 6"
        instance.add_htip_parser(b"\x01\x02".to_vec(), Box::new(SizedText::exact(6)));
        //this is "whatever stated in the first byte (maximum length 31)"
        instance.add_htip_parser(b"\x01\x03".to_vec(), Box::new(SizedText::new(31)));
        //subtype1 info4
        instance.add_htip_parser(b"\x01\x04".to_vec(), Box::new(SizedText::new(31)));
        //subtype1 info20
        instance.add_htip_parser(b"\x01\x20".to_vec(), Box::new(Percentage::new()));
        //subtype1 info21
        instance.add_htip_parser(b"\x01\x21".to_vec(), Box::new(Percentage::new()));
        //subtype1 info22
        instance.add_htip_parser(b"\x01\x22".to_vec(), Box::new(Percentage::new()));
        //subtype1 info23
        instance.add_htip_parser(
            b"\x01\x23".to_vec(),
            Box::new(SizedNumber::new(NumberSize::Six)),
        );
        //subtype1 info24
        instance.add_htip_parser(
            b"\x01\x24".to_vec(),
            Box::new(SizedNumber::new(NumberSize::One)),
        );
        //subtype1 info25
        instance.add_htip_parser(
            b"\x01\x25".to_vec(),
            Box::new(SizedNumber::new(NumberSize::One)),
        );
        //subtype1 info26
        instance.add_htip_parser(
            b"\x01\x26".to_vec(),
            Box::new(SizedNumber::new(NumberSize::One)),
        );
        //subtype1 info27
        instance.add_htip_parser(
            b"\x01\x27".to_vec(),
            Box::new(SizedNumber::new(NumberSize::One)),
        );
        //subtype1 info50
        instance.add_htip_parser(b"\x01\x50".to_vec(), Box::new(SizedText::new(63)));
        //subtype1 info51
        instance.add_htip_parser(b"\x01\x51".to_vec(), Box::new(Percentage::new()));
        //subtype1 info52
        instance.add_htip_parser(b"\x01\x52".to_vec(), Box::new(Percentage::new()));
        //subtype1 info53
        instance.add_htip_parser(b"\x01\x53".to_vec(), Box::new(Percentage::new()));
        //subtype1 info54
        instance.add_htip_parser(b"\x01\x54".to_vec(), Box::new(Percentage::new()));
        //subtype1 info80
        instance.add_htip_parser(
            b"\x01\x80".to_vec(),
            Box::new(SizedNumber::new(NumberSize::Two)),
        );
        //TODO: use a composite parser for this in the future
        //subtype1 info255
        //TODO: use composite parser for this
        //subtype 2
        instance.add_htip_parser(b"\x03".to_vec(), Box::new(Mac::new()));

        instance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_test() {
        let mut dsp = Dispatcher::empty();
        dsp.add_htip_parser(b"\x01\x01".to_vec(), Box::new(SizedText::new(255)));
    }

    #[test]
    fn finds_key() {
        //type 127, length 16
        let frame = b"\xfe\x0f\xe0\x27\x1a\x01\x01\x09123456789\
            \xfe\x0c\xe0\x27\x1a\x01\x02\x06OUIOUI\
            \x02\x0a0123456789";
        let dsp = Dispatcher::new();
        //collect our two tlvs, and do stuff with them
        let tlvs = parse_frame(frame)
            .into_iter()
            .collect::<Result<Vec<TLV>, _>>()
            .unwrap();
        assert_eq!(tlvs.len(), 3);
        let key0 = dsp.parsers.key_of(&tlvs[0]).unwrap();
        assert_eq!(key0.tlv_type, 127);
        assert_eq!(key0.prefix, b"\xe0\x27\x1a\x01\x01");

        let key1 = dsp.parsers.key_of(&tlvs[1]).unwrap();
        assert_eq!(key1.tlv_type, 127);
        assert_eq!(key1.prefix, b"\xe0\x27\x1a\x01\x02");

        let key2 = dsp.parsers.key_of(&tlvs[2]).unwrap();
        assert_eq!(key2.tlv_type, 1);
        assert_eq!(key2.prefix, b"");
    }

    #[test]
    fn find_key_is_none() {
        //unknown oui
        let frame = b"\xfe\x0f\xAA\xBB\x1a\x01\x01\x09123456789";
        let dsp = Dispatcher::new();
        let tlvs = parse_frame(frame)
            .into_iter()
            .collect::<Result<Vec<TLV>, _>>()
            .unwrap();
        assert_eq!(tlvs.len(), 1);
        assert_eq!(None, dsp.parsers.key_of(&tlvs[0]));
    }

    #[test]
    #[should_panic]
    fn adding_key_twice_panics() {
        let mut dsp = Dispatcher::new();
        dsp.add_htip_parser(b"\x01\x01".to_vec(), Box::new(SizedText::new(255)));
    }

    #[test]
    fn one_tlv_parse_succeeds() {
        let frame = b"\xfe\x0f\xe0\x27\x1a\x01\x01\x09123456789";
        let mut dsp = Dispatcher::new();
        //collect our two tlvs, and do stuff with them
        let tlvs = parse_frame(frame)
            .into_iter()
            .collect::<Result<Vec<TLV>, _>>()
            .unwrap();
        assert_eq!(tlvs.len(), 1);
        assert_eq!(
            "123456789",
            dsp.parse_tlv(&tlvs[0]).1.unwrap().into_string().unwrap()
        );
    }

    #[test]
    fn simple_tlv_parse_succeeds() {
        let frame = b"\xfe\x0f\xe0\x27\x1a\x01\x01\x09123456789\
            \xfe\x0c\xe0\x27\x1a\x01\x02\x06OUIOUI";
        let mut dsp = Dispatcher::new();
        //collect our two tlvs, and do stuff with them
        let tlvs = parse_frame(frame)
            .into_iter()
            .collect::<Result<Vec<TLV>, _>>()
            .unwrap();
        assert_eq!(
            "123456789",
            dsp.parse_tlv(&tlvs[0]).1.unwrap().into_string().unwrap()
        );
        assert_eq!(
            "OUIOUI",
            dsp.parse_tlv(&tlvs[1]).1.unwrap().into_string().unwrap()
        );
    }
}
