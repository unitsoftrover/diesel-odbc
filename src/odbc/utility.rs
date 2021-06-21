use std::borrow::Cow;

pub struct Utility{    
}

impl Utility{

    pub fn utf8_bytes_to_gbk(utf8_bytes: &[u8])->String
    {
        let (cow, _encoding, _had_errors) = encoding_rs::GB18030.decode(utf8_bytes);
        let _msg = String::new();
        match cow {
            Cow::Borrowed(val) => {
                val.to_string()
            }
            Cow::Owned(val) => {
                val
            },
        }
    }

}