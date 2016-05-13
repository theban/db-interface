extern crate memrange;
extern crate theban_db;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

use ::DBInterface;
use ::BitmapTableIter;
use ::ObjectTableRawIter;
use ::ObjectTableIter;
use error::DBResult;
use error::DBInterfaceError;

use memrange::Range;
use iter::ManyBitmapSlicesIter;
use iter::ManyObjectsDataIter;
use iter::ManyObjectsDecodedIter;
use theban_db::{Bitmap,Object,DB};
use rustc_serialize::{Encodable,Decodable};

impl DBInterface for DB {
    fn saveas(&mut self, path: &String) -> DBResult<()>{
        try!(self.save_to_file(path));
        return Ok(())
    }

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8>) -> DBResult<()>{
        self.insert_bitmap( tbl, rng, Bitmap::new( 1, data ) );
        return Ok(())
    }

    fn bin_put_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)>) -> DBResult<()>{
        for (rng,data) in args {
            try!(self.bin_put(tbl, rng, data));
        }
        return Ok(())
    }

    fn bin_get<'db>(&'db mut self, tbl: &String, rng: Range ) -> DBResult<BitmapTableIter<'db>> {
        if !self.has_table(tbl) { return Err(DBInterfaceError::TableNotFound(tbl.clone())); }
        return Ok(Box::new(ManyBitmapSlicesIter::new( self, tbl.clone(), vec![rng])));
    }

    fn bin_get_many<'db>(&'db mut self, tbl: &String, ranges: Vec<Range>) -> DBResult<BitmapTableIter<'db>> {
        if !self.has_table(tbl) { return Err(DBInterfaceError::TableNotFound(tbl.clone())); }
        return Ok(Box::new(ManyBitmapSlicesIter::new(self, tbl.clone(), ranges)));
    }


    fn bin_del(&mut self, tbl: &String, rng: Range)-> DBResult<()>{
        self.delete_bitmap(tbl, 1, rng);
        return Ok(())
    }

    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>)-> DBResult<()>{
        for rng in ranges {
            try!(self.bin_del(tbl, rng));
        }
        return Ok(());
    }

    fn obj_put<T: Encodable>(&mut self, tbl: &String, rng: Range, obj: &T )-> DBResult<()>{
        self.insert_object(tbl, rng, Object::new(::encode_obj(rng, obj).unwrap()));
        return Ok(())
    }

    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, setters: Vec<(Range,&T)> )-> DBResult<()>{
        for (rng,data) in setters {
            try!(self.obj_put(tbl, rng, data));
        }
        return Ok(())
    }

    fn obj_put_raw_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> )-> DBResult<()>{
        for (rng,data) in setters {
            try!(self.obj_put_raw(tbl, rng, data));
        }
        return Ok(())
    }

    fn obj_put_raw(&mut self, tbl: &String, rng: Range, obj: Vec<u8> )-> DBResult<()>{
        self.insert_object(tbl, rng, Object::new(obj));
        return Ok(())
    }

    fn obj_del(&mut self, tbl: &String, rng: Range )-> DBResult<()>{
        self.delete_object(tbl, rng);
        return Ok(())
    }

    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> )-> DBResult<()>{
        for rng in ranges {
            try!(self.obj_del(tbl, rng));
        }
        return Ok(())
    }

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range )-> DBResult<()>{
        self.delete_intersecting_objects(tbl, rng);
        return Ok(())
    }

    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> )-> DBResult<()>{
        for rng in ranges {
            try!(self.obj_del_intersecting(tbl, rng));
        }
        return Ok(())
    }

    fn obj_get<'db, T: Decodable + 'db>(&'db mut self, tbl: &String, rng: Range) -> DBResult<ObjectTableIter<T>>{
        return self.obj_get_many(tbl, vec![rng])
    }

    fn obj_get_many<'db, T: Decodable + 'db>(&'db mut self, tbl: &String, ranges: Vec<Range> ) -> DBResult<ObjectTableIter<T>>{
        if !self.has_table(tbl) { return Err(DBInterfaceError::TableNotFound(tbl.clone())); }
        return Ok(Box::new(ManyObjectsDecodedIter::<'db,T>::new( self, tbl.clone(), ranges)));
    }

    fn obj_get_raw<'db>(&'db mut self, tbl: &String, rng: Range ) -> DBResult<ObjectTableRawIter<'db>> {
        return self.obj_get_raw_many(tbl, vec![rng])
    }

    fn obj_get_raw_many<'db>(&'db mut self, tbl: &String, ranges: Vec<Range>) -> DBResult<ObjectTableRawIter<'db>> {
        if !self.has_table(tbl) { return Err(DBInterfaceError::TableNotFound(tbl.clone())); }
        return Ok(Box::new(ManyObjectsDataIter::new( self, tbl.clone(), ranges)));
    }
}
