use std::error;
use std::fmt;

use theban_db::DBError;
use ::memrange::Range;
use ::msgpack::decode::Error as DecodeError;
use ::msgpack::encode::Error as EncodeError;

#[derive(Debug)]
pub enum DBInterfaceError<'a> {
    DBError(DBError<'a>),
    TableNotFound(String),
    ObjectDeserializationError(Range, DecodeError),
    ObjectSerializationError(Range, EncodeError)
}

pub type DBResult<'a,T> = Result<T, DBInterfaceError<'a>>;

pub fn from_obj_decoding<'db,T>(rng: Range, des_res: Result<T,DecodeError>) -> DBResult<'db, T>{
    return des_res.map_err(|e| DBInterfaceError::ObjectDeserializationError(rng, e))
}

pub fn from_obj_encoding<'db,T>(rng: Range, des_res: Result<T,EncodeError>) -> DBResult<'db, T>{
    return des_res.map_err(|e| DBInterfaceError::ObjectSerializationError(rng, e))
}

impl<'a> From<DBError<'a>> for DBInterfaceError<'a> {
    fn from(err: DBError<'a>) -> DBInterfaceError<'a> {
        DBInterfaceError::DBError(err)
    }
}

impl<'a> fmt::Display for DBInterfaceError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DBInterfaceError::DBError(ref err) => write!(f, "Error in DB: {}", err),
            DBInterfaceError::TableNotFound(ref err) => write!(f, "Table Not Found: {}", err),
            DBInterfaceError::ObjectDeserializationError(ref rng, ref err) => write!(f, "Failed To Deserialize Object: {} (at {:?})", err, rng),
            DBInterfaceError::ObjectSerializationError(ref rng, ref err) => write!(f, "Failed To Serialize Object: {} (at {:?})", err, rng),
        }
    }
}

impl<'a> error::Error for DBInterfaceError<'a> {
    fn description(&self) -> &str {
        match *self {
            DBInterfaceError::DBError(ref err) => err.description(),
            DBInterfaceError::TableNotFound(_) => "No such table in db",
            DBInterfaceError::ObjectDeserializationError(_,_) => "Failed to deserialize obj",
            DBInterfaceError::ObjectSerializationError(_,_) => "Failed to serialize obj",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DBInterfaceError::TableNotFound(_) => None,
            DBInterfaceError::DBError(ref err) => Some(err), 
            DBInterfaceError::ObjectDeserializationError(_, ref err) => Some(err), 
            DBInterfaceError::ObjectSerializationError(_, ref err) => Some(err), 
        }
    }
}
