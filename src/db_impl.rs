extern crate memrange;
extern crate theban_db;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

use ::DBInterface;
use memrange::Range;
use iter::ManyBitmapSlicesIter;
use theban_db::{Bitmap,Object,DB};
use rustc_serialize::Encodable;
use msgpack::Encoder;

impl DBInterface for DB {
    //fn saveas(&self, path: &String) -> Result<(), DBError>{
    //    return self.save_to_file(path);
    //}

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8>) {
        self.insert_bitmap( tbl, rng, Bitmap::new( 1, data ) )
    }

    fn bin_put_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)>) {
        for (rng,data) in args {
            self.bin_put(tbl, rng, data);
        }
    }

    fn bin_get<'a>(&'a self, tbl: &String, rng: Range ) -> Option<ManyBitmapSlicesIter<'a>> {
        if !self.has_table(tbl) { return None; }
        return Some(ManyBitmapSlicesIter::new(self, tbl.clone(), vec![rng]));
    }

    fn bin_get_many<'a>(&'a self, tbl: &String, ranges: Vec<Range>) -> Option<ManyBitmapSlicesIter<'a>> {
        if !self.has_table(tbl) { return None; }
        return Some(ManyBitmapSlicesIter::new(self, tbl.clone(), ranges));
    }

    fn bin_del(&mut self, tbl: &String, rng: Range){
        self.delete_bitmap(tbl, 1, rng)
    }

    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>){
        for rng in ranges {
            self.bin_del(tbl, rng);
        }
    }

    fn obj_put<T: Encodable>(&mut self, tbl: &String, rng: Range, obj: &T ){
        let mut buf = vec![];
        obj.encode(&mut Encoder::new(&mut buf)).unwrap();
        self.insert_object(tbl, rng, Object::new(buf));
    }

    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, setters: Vec<(Range,&T)> ){
        for (rng,data) in setters {
            self.obj_put(tbl, rng, data);
        }
    }

    fn obj_put_raw_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> ){
        for (rng,data) in setters {
            self.obj_put_raw(tbl, rng, data);
        }
    }

    fn obj_put_raw(&mut self, tbl: &String, rng: Range, obj: Vec<u8> ){
        self.insert_object(tbl, rng, Object::new(obj));
    }

    fn obj_del(&mut self, tbl: &String, rng: Range ){
        self.delete_object(tbl, rng);
    }

    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> ){
        for rng in ranges {
            self.obj_del(tbl, rng);
        }
    }

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range ){
        self.delete_intersecting_objects(tbl, rng);
    }

    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> ){
        for rng in ranges {
            self.obj_del_intersecting(tbl, rng);
        }
    }
}
