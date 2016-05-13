use theban_db::DBError;
use theban_db_server::error::NetworkEncodingError;
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

        NetworkEncodingError(err: NetworkEncodingError){
            description("Failed to encode network command")
            display("Failed to encode network command {}", err)
            from()
            cause(err)
        }

        ObjectDeserialization(rng: Range, err: DecodeError){
            description("Failed to deserialize object")
            display("Failed to deserialize object at {:?}: {}", rng, err)
            cause(err)
        }
        ObjectSerialization(rng: Range, err: EncodeError){
            description("Failed To serialize object")
            display("iled To serialize object at {:?}: {}", rng, err)
            cause(err)
        }
        UnexpectedResponse(err: String){
            description("Got unexpected Response from Server")
            display("Got unexpected Response from Server: {}", err)
        }
    }
}


pub type DBResult<T> = Result<T, DBInterfaceError>;

pub fn from_obj_decoding<T>(rng: Range, des_res: Result<T,DecodeError>) -> DBResult<T>{
    return des_res.map_err(|e| DBInterfaceError::ObjectDeserialization(rng, e))
}

pub fn from_obj_encoding<T>(rng: Range, des_res: Result<T,EncodeError>) -> DBResult<T>{
    return des_res.map_err(|e| DBInterfaceError::ObjectSerialization(rng, e))
}
