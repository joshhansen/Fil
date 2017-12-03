trait AudioDecoder {
    fn decode_audio(&mut self, bytes: &Vec<u8>) -> Option<&Vec<u8>>;
}

// impl <T:AudioDecoder> Decoder for T {
//     fn decode<F:Fn(Option<&Vec<u8>>)>(&mut self, callback: F) {
//
//     }
// }

struct AdHocAudioDecoder {

}



impl AudioDecoder for AdHocAudioDecoder {
    fn decode_audio(&mut self, bytes: &Vec<u8>) -> Option<&Vec<u8>> {
        None
    }
}

pub fn decode<F:Fn(Option<&Vec<u8>>)>(callback: F) {

}
