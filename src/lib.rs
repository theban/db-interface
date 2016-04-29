extern crate memrange;
extern crate theban_db;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

mod iter;
mod db_impl;

use memrange::Range;
use iter::ManyBitmapSlicesIter;
//use theban_db::BitmapSliceIter;
use rustc_serialize::{Encodable,Decodable};

pub trait DBInterface {
    //fn saveas(&self, path: &String) -> Result<()>;

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8> );
    fn bin_put_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> );

    fn bin_get<'a>(&'a self, tbl: &String, rng: Range) -> Option<ManyBitmapSlicesIter<'a>>;
    fn bin_get_many<'a>(&'a self, tbl: &String, ranges: Vec<Range>) -> Option<ManyBitmapSlicesIter<'a>>;

    fn bin_del(&mut self, tbl: &String, rng: Range);
    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>);

    fn obj_put<T: Encodable>(&mut self,tbl: &String, rng: Range, obj: &T );
    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, args: Vec<(Range, &T)> );
    
    //fn obj_get<T: RustcDecodable>(tbl: &String, rng: Range -> Iter<Range, Cow<T>>;
    //fn obj_get_many<T: RustcDecodable>(tbl: &String, ranges: Vec<Range> ) -> Iter<Range, Cow<T>>;
    
    fn obj_put_raw(&mut self, tbl: &String, range: Range, obj: Vec<u8> );
    fn obj_put_raw_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)> );

    //fn obj_get_raw<'a>(&'a self tbl: &String, args: OneOrMany<&Range> ) -> Option<ManyObjectIter<'a>>;
    //fn obj_get_raw_many(tbl: &String, args: OneOrMany<&Range> ) -> Iter<Range, Cow<T>>;
    
    fn obj_del(&mut self, tbl: &String, rng: Range );
    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> );

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range );
    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> );
}

