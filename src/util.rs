use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

macro_rules! custom_encodable {
    ($struct_name:ident, $($field:ident),*) => {
        impl <S: Encoder<E>, E> Encodable<S, E> for $struct_name {
            fn encode(&self, encoder: &mut S) -> Result<(), E> {
                encoder.emit_struct(stringify!($struct_name), 0, |encoder| {
                    $(
                        try!(encoder.emit_struct_field(stringify!($field), 0, |encoder|self.$field.encode(encoder)));
                    )*
                    Ok(())
                })
            }
        }
        
        impl<S: Decoder<E>, E> Decodable<S, E> for $struct_name {
            fn decode(decoder: &mut S) -> Result<$struct_name, E> {
                decoder.read_struct("root", 0, |decoder| {
                    let d = $struct_name {
                        $(
                            $field: try!(decoder.read_struct_field(stringify!($field), 0, |decoder| Decodable::decode(decoder))),
                        )*
                    };
                    Ok(d)
                })
            }
        }
    }
}

macro_rules! encode_field {
    ($field:ident) => {
        try!(encoder.emit_struct_field(stringify!($field), 0, |encoder|self.$field.encode(encoder)))
    }
}

macro_rules! encode_fields {
    ($($field:ident),+) => {
        $(
            encode_field!($field);
        )+
    }
}

macro_rules! decode_fields {
    ($($field:ident),+) => {
        $(
            $field: decode_field!($field),
        )+
    }
}

macro_rules! decode_field {
    ($field:ident) => {
        try!(decoder.read_struct_field(stringify!($field), 0, |decoder| Decodable::decode(decoder)))
    }
}
