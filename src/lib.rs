extern crate memrange;
extern crate theban_db;
extern crate theban_db_server;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;
#[macro_use] extern crate quick_error;

mod iter;
mod db_impl;
mod server_impl;
mod error;

use std::borrow::Cow;
use std::io::Cursor;
use rustc_serialize::{Encodable,Decodable};
use msgpack::{Encoder,Decoder};

use memrange::Range;
use theban_db::BitmapSlice;

pub use error::DBResult;
pub use error::DBInterfaceError;

pub type BitmapTableIter<'db> =     Box<Iterator<Item=(Range, Range, BitmapSlice<'db>)> + 'db>;
pub type ObjectTableIter<'db,T> =   Box<Iterator<Item=(Range, Range, DBResult<T>)> + 'db>;
pub type ObjectTableRawIter<'db> =  Box<Iterator<Item=(Range, Range, Cow<'db,Vec<u8>>)> +'db>;

pub type ObjectTableIterOk<'db,T> =   Box<Iterator<Item=(Range, Range, T)> + 'db>;

pub trait DBInterface {
    fn saveas(&mut self, path: &String) -> DBResult<()>;

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8> ) -> DBResult<()>;
    fn bin_put_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> )-> DBResult<()>;

    fn bin_get<'db>(&'db mut self, tbl: &String, rng: Range) -> DBResult<BitmapTableIter<'db>>;
    fn bin_get_many<'db>(&'db mut self, tbl: &String, ranges: Vec<Range>) -> DBResult<BitmapTableIter<'db>>;

    fn bin_del(&mut self, tbl: &String, rng: Range)-> DBResult<()>;
    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>)-> DBResult<()>;

    fn obj_put<T: Encodable>(&mut self, tbl: &String, rng: Range, obj: &T )-> DBResult<()>;
    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, args: Vec<(Range, &T)> )-> DBResult<()>;
    
    fn obj_get<'db, T: Decodable +'db>(&'db mut self, tbl: &String, rng: Range) -> DBResult<ObjectTableIter<T>>;
    fn obj_get_many<'db, T: Decodable + 'db>(&'db mut self, tbl: &String, ranges: Vec<Range> ) -> DBResult<ObjectTableIter<T>>;
    
    fn obj_put_raw(&mut self, tbl: &String, range: Range, obj: Vec<u8> )-> DBResult<()>;
    fn obj_put_raw_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)> )-> DBResult<()>;

    fn obj_get_raw<'db>(&'db mut self, tbl: &String, range: Range ) -> DBResult<ObjectTableRawIter<'db>>;
    fn obj_get_raw_many<'db>(&'db mut self, tbl: &String, args: Vec<Range> ) -> DBResult<ObjectTableRawIter<'db>>;
    
    fn obj_del(&mut self, tbl: &String, rng: Range )-> DBResult<()>;
    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> )-> DBResult<()>;

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range )-> DBResult<()>;
    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> )-> DBResult<()>;

    //version that panics on error
    fn obj_get_ok<'db, T: Decodable + 'db>(&'db mut self, tbl: &String, rng: Range) -> ObjectTableIterOk<'db,T>{
        Box::new(self.obj_get(tbl.into(), rng).unwrap().map(|(q,r,t)| (q,r,t.unwrap())))
    }
}

fn decode_obj<'db,T: Decodable>(rng: Range, data: &Vec<u8>) -> DBResult<T>{
    error::from_obj_decoding(rng, T::decode(&mut Decoder::new(Cursor::new(data))) )
}

fn encode_obj<'db,T: Encodable>(rng: Range, data: &T) -> DBResult<Vec<u8>>{
    let mut buf = vec![];
    try!(error::from_obj_encoding( rng, data.encode(&mut Encoder::new(&mut buf)) ));
    return Ok(buf);
}

#[cfg(test)]
mod tests{
    use theban_db::DB;
    use memrange::{Range,range};
    fn test_dbi<DBI: ::DBInterface>(dbi: &mut DBI){
        dbi.obj_put::<u64>(&"tbl".into(),range(123,124), &1 ).ok();
        dbi.obj_put::<u64>(&"tbl".into(),range(222,235), &2 ).ok();
        dbi.obj_put::<u64>(&"tbl".into(),range(125,224), &3 ).ok();
        dbi.obj_put::<u64>(&"tbl".into(),range(  1,  2), &4 ).ok();
        let result1 = dbi.obj_get_ok::<u64>(&"tbl".into(),range(124,221)).map(|(_,r,_)| r).collect::<Vec<Range>>();
        assert_eq!(result1, vec![range(123,124),range(125,224)] );
        let result2 = dbi.obj_get_ok::<u64>(&"tbl".into(),range(124,221)).collect::<Vec<(Range,Range,u64)>>();
        assert_eq!(result2, vec![(range(124,221), range(123,124), 1), (range(124,221), range(125,224), 3)] );
    }

    #[test]
    fn interface_db_test() {
        let mut db = DB::new();
        test_dbi(&mut db)
    }
}
