extern crate memrange;
extern crate theban_db;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;
#[macro_use] extern crate quick_error;

mod iter;
mod db_impl;
mod error;

use std::io::Cursor;
use rustc_serialize::{Encodable,Decodable};
use msgpack::{Encoder,Decoder};

use memrange::Range;
use theban_db::BitmapSlice;

pub use error::DBResult;
pub use error::DBInterfaceError;

pub type BitmapTableIter<'db> =     Box<Iterator<Item=(Range, Range, BitmapSlice<'db>)> + 'db>;
pub type ObjectTableIter<'db,T> =   Box<Iterator<Item=(Range, Range, DBResult<T>)> + 'db>;
pub type ObjectTableRawIter<'db> =  Box<Iterator<Item=(Range, Range, &'db Vec<u8>)> +'db>;

pub trait DBInterface {
    fn saveas(&self, path: &String) -> DBResult<()>;

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8> );
    fn bin_put_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> );

    fn bin_get<'db>(&'db self, tbl: &String, rng: Range) -> DBResult<BitmapTableIter<'db>>;
    fn bin_get_many<'db>(&'db self, tbl: &String, ranges: Vec<Range>) -> DBResult<BitmapTableIter<'db>>;

    fn bin_del(&mut self, tbl: &String, rng: Range);
    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>);

    fn obj_put<T: Encodable>(&mut self,tbl: &String, rng: Range, obj: &T );
    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, args: Vec<(Range, &T)> );
    
    fn obj_get<'db, T: Decodable +'db>(&'db self, tbl: &String, rng: Range) -> DBResult<ObjectTableIter<T>>;
    fn obj_get_many<'db, T: Decodable + 'db>(&'db self, tbl: &String, ranges: Vec<Range> ) -> DBResult<ObjectTableIter<T>>;
    
    fn obj_put_raw(&mut self, tbl: &String, range: Range, obj: Vec<u8> );
    fn obj_put_raw_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)> );

    fn obj_get_raw<'db>(&'db self, tbl: &String, range: Range ) -> DBResult<ObjectTableRawIter<'db>>;
    fn obj_get_raw_many<'db>(&'db self, tbl: &String, args: Vec<Range> ) -> DBResult<ObjectTableRawIter<'db>>;
    
    fn obj_del(&mut self, tbl: &String, rng: Range );
    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> );

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range );
    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> );
}


fn decode_obj<'db,T: Decodable>(rng: Range, data: &Vec<u8>) -> DBResult<T>{
    error::from_obj_decoding(rng, T::decode(&mut Decoder::new(Cursor::new(data))) )
}

fn encode_obj<'db,T: Encodable>(rng: Range, data: &T) -> DBResult<Vec<u8>>{
    let mut buf = vec![];
    try!(error::from_obj_encoding( rng, data.encode(&mut Encoder::new(&mut buf)) ));
    return Ok(buf);
}
