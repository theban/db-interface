extern crate memrange;
extern crate theban_db;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

mod iter;
mod db_impl;

use memrange::Range;
use rustc_serialize::{Encodable,Decodable};
use theban_db::BitmapSlice;

pub type BitmapTableIter<'db> =     Box<Iterator<Item=(Range, Range, BitmapSlice<'db>)> + 'db>;
pub type ObjectTableIter<'db,T> =   Box<Iterator<Item=(Range, Range, T)> + 'db>;
pub type ObjectTableRawIter<'db> =  Box<Iterator<Item=(Range, Range, &'db Vec<u8>)> +'db>;

pub trait DBInterface {
    //fn saveas(&self, path: &String) -> Result<()>;

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8> );
    fn bin_put_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> );

    fn bin_get<'db>(&'db self, tbl: &String, rng: Range) -> Option<BitmapTableIter<'db>>;
    fn bin_get_many<'db>(&'db self, tbl: &String, ranges: Vec<Range>) -> Option<BitmapTableIter<'db>>;

    fn bin_del(&mut self, tbl: &String, rng: Range);
    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>);

    fn obj_put<T: Encodable>(&mut self,tbl: &String, rng: Range, obj: &T );
    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, args: Vec<(Range, &T)> );
    
    fn obj_get<'db, T: Decodable +'db>(&'db self, tbl: &String, rng: Range) -> Option<ObjectTableIter< T>>;
    fn obj_get_many<'db, T: Decodable + 'db>(&'db self, tbl: &String, ranges: Vec<Range> ) -> Option<ObjectTableIter<T>>;
    
    fn obj_put_raw(&mut self, tbl: &String, range: Range, obj: Vec<u8> );
    fn obj_put_raw_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)> );

    fn obj_get_raw<'db>(&'db self, tbl: &String, range: Range ) -> Option<ObjectTableRawIter<'db>>;
    fn obj_get_raw_many<'db>(&'db self, tbl: &String, args: Vec<Range> ) -> Option<ObjectTableRawIter<'db>>;
    
    fn obj_del(&mut self, tbl: &String, rng: Range );
    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> );

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range );
    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> );
}

