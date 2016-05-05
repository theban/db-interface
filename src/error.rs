use theban_db::DBError;
use ::memrange::Range;
use ::msgpack::decode::Error as DecodeError;
use ::msgpack::encode::Error as EncodeError;

quick_error! {
    #[derive(Debug)]
    pub enum DBInterfaceError {
        DB(err: DBError){
            description("Error in DB")
            display("Error in DB: {}", err)
            from()
            cause(err)
        }
        TableNotFound(err: String){
            description("No such table")
            display("No such table: {}", err)
        }

        ObjectDeserializationError(rng: Range, err: DecodeError){
            description("Failed To deserialize object")
            display("iled To deserialize object at {:?}: {}", rng, err)
            cause(err)
        }
        ObjectSerializationError(rng: Range, err: EncodeError){
            description("Failed To serialize object")
            display("iled To serialize object at {:?}: {}", rng, err)
            cause(err)
        }
    }
}


pub type DBResult<T> = Result<T, DBInterfaceError>;

pub fn from_obj_decoding<T>(rng: Range, des_res: Result<T,DecodeError>) -> DBResult<T>{
    return des_res.map_err(|e| DBInterfaceError::ObjectDeserializationError(rng, e))
}

pub fn from_obj_encoding<T>(rng: Range, des_res: Result<T,EncodeError>) -> DBResult<T>{
    return des_res.map_err(|e| DBInterfaceError::ObjectSerializationError(rng, e))
}
