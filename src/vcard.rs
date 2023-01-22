use chrono::prelude::NaiveDate;
use std::{error, fmt};

#[derive(Debug, PartialEq, Eq)]
pub struct VCard {
    pub name: String,
    pub bday: Option<NaiveDate>,
}

#[derive(Debug, PartialEq)]
pub enum VCardError {
    UnexpectedFieldError(String),
    MissingEndError,
    NoNameError,
    InvalidNameError,
    InvalidBDayError(String),
}

impl fmt::Display for VCardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::UnexpectedFieldError(field) => write!(f, "Unexpected {}", field),
            Self::MissingEndError => write!(f, "Missing END:VCARD"),
            Self::NoNameError => write!(f, "No name at end of vcard"),
            Self::InvalidNameError => write!(f, "error while parsing name"),
            Self::InvalidBDayError(msg) => write!(f, "error while parsing bday: {}", msg),
        }
    }
}

impl error::Error for VCardError {}

#[derive(PartialEq)]
enum ParseState {
    In,
    Out,
}

impl ParseState {
    fn is_in(&self, msg: VCardError) -> Result<(), VCardError> {
        return if *self != ParseState::In {
            Err(msg)
        } else {
            Ok(())
        };
    }

    fn is_out(&self, msg: VCardError) -> Result<(), VCardError> {
        return if *self != ParseState::Out {
            Err(msg)
        } else {
            Ok(())
        };
    }
}

pub fn parse_vcards(contents: String) -> Result<Vec<VCard>, VCardError> {
    let mut result: Vec<VCard> = Vec::new();
    let mut parse_state = ParseState::Out;

    let mut name: Option<String> = None;
    let mut bday: Option<NaiveDate> = None;

    for line in contents.lines() {
        match line {
            "BEGIN:VCARD" => {
                parse_state.is_out(VCardError::UnexpectedFieldError(String::from(
                    "BEGIN:VCARD",
                )))?;
                parse_state = ParseState::In;
            }
            "END:VCARD" => {
                parse_state.is_in(VCardError::UnexpectedFieldError(String::from("END:VCARD")))?;
                parse_state = ParseState::Out;
                let vcard = match name {
                    Some(name) => VCard { name, bday },
                    None => return Err(VCardError::NoNameError),
                };
                result.push(vcard);
                name = None;
                bday = None;
            }
            line => {
                parse_state.is_in(VCardError::UnexpectedFieldError(String::from("contents")))?;
                if line.starts_with("FN:") {
                    name = Some(String::from(&line[3..]));
                } else if line.starts_with("FN;CHARSET=UTF-8;ENCODING=QUOTED-PRINTABLE:") {
                    let decoded_name = decode_quoted_printable(&line[43..])?;
                    name = Some(decoded_name);
                } else if line.starts_with("BDAY:") {
                    let res = NaiveDate::parse_from_str(&line[5..], "%Y-%m-%d");

                    bday = match res {
                        Ok(nd) => Some(nd),
                        Err(pe) => return Err(VCardError::InvalidBDayError(pe.to_string())),
                    };
                }
            }
        }
    }

    match parse_state {
        ParseState::In => Err(VCardError::MissingEndError),
        ParseState::Out => Ok(result),
    }
}

fn decode_quoted_printable(encoded: &str) -> Result<String, VCardError> {
    let mut bytes = Vec::new();
    for s in encoded.split("=").skip(1) {
        let b = match u8::from_str_radix(s, 16) {
            Ok(b) => b,
            Err(_) => return Err(VCardError::InvalidNameError),
        };
        bytes.push(b);
    }

    match String::from_utf8(bytes) {
        Ok(s) => Ok(s),
        Err(_) => Err(VCardError::InvalidNameError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vcards_ok() {
        let input = "\
BEGIN:VCARD
VERSION:2.1
N:Test;Allice;;;
FN:Allice Test
TEL;CELL:+01234567890
END:VCARD
BEGIN:VCARD
VERSION:2.1
N:Test;Bob;;;
FN:Bob Test
BDAY:1980-05-07
END:VCARD
BEGIN:VCARD
VERSION:2.1
N:Täst;;;;
FN;CHARSET=UTF-8;ENCODING=QUOTED-PRINTABLE:=54=C3=A4=73=74
TEL;CELL:+01234567890
END:VCARD";

        let result = parse_vcards(input.to_string()).unwrap();
        let expected = vec![
            VCard {
                name: String::from("Allice Test"),
                bday: None,
            },
            VCard {
                name: String::from("Bob Test"),
                bday: NaiveDate::from_ymd_opt(1980, 5, 7),
            },
            VCard {
                name: String::from("Täst"),
                bday: None,
            },
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_vcards_no_begin_vcard_contents() {
        let input = "\
VERSION:2.1
N:Test;Allice;;;
FN:Allice Test
TEL;CELL:+01234567890
END:VCARD
BEGIN:VCARD
VERSION:2.1
N:Test;Bob;;;
FN:Bob Test
BDAY:1980-05-07
END:VCARD";

        let result = parse_vcards(input.to_string()).unwrap_err();

        assert_eq!(
            VCardError::UnexpectedFieldError(String::from("contents")),
            result
        );
    }

    #[test]
    fn parse_vcards_no_begin_vcard_end_vcard() {
        let input = "\
END:VCARD
BEGIN:VCARD
VERSION:2.1
N:Test;Bob;;;
FN:Bob Test
BDAY:1980-05-07
END:VCARD";

        let result = parse_vcards(input.to_string()).unwrap_err();

        assert_eq!(
            VCardError::UnexpectedFieldError(String::from("END:VCARD")),
            result
        );
    }

    #[test]
    fn parse_vcards_double_begin_vcard() {
        let input = "\
BEGIN:VCARD
BEGIN:VCARD
VERSION:2.1
N:Test;Bob;;;
FN:Bob Test
BDAY:1980-05-07
END:VCARD";

        let result = parse_vcards(input.to_string()).unwrap_err();

        assert_eq!(
            VCardError::UnexpectedFieldError(String::from("BEGIN:VCARD")),
            result
        );
    }

    #[test]
    fn parse_vcards_missing_end_vcard() {
        let input = "\
BEGIN:VCARD
VERSION:2.1
N:Test;Bob;;;
FN:Bob Test
BDAY:1980-05-07";

        let result = parse_vcards(input.to_string()).unwrap_err();

        assert_eq!(VCardError::MissingEndError, result);
    }

    #[test]
    fn parse_vcards_no_name() {
        let input = "\
BEGIN:VCARD
VERSION:2.1
N:Test;Allice;;;
FN:Allice Test
TEL;CELL:+01234567890
END:VCARD
BEGIN:VCARD
VERSION:2.1
N:Test;Bob;;;
BDAY:1980-05-07
END:VCARD";

        let result = parse_vcards(input.to_string()).unwrap_err();

        assert_eq!(VCardError::NoNameError, result);
    }

    #[test]
    fn parse_vcards_invalid_bday() {
        let input = "\
BEGIN:VCARD
VERSION:2.1
N:Test;Allice;;;
FN:Allice Test
TEL;CELL:+01234567890
END:VCARD
BEGIN:VCARD
VERSION:2.1
N:Test;Bob;;;
FN:Bob Test
BDAY:1980-asdf-07
END:VCARD";

        let result = parse_vcards(input.to_string()).unwrap_err();

        assert_eq!(
            VCardError::InvalidBDayError(String::from("input contains invalid characters")),
            result
        );
    }
}
