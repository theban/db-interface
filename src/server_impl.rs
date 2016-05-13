extern crate rustc_serialize;
extern crate unix_socket;

extern crate theban_db_server;
//extern crate theban_db;

use std::borrow::Cow;
use rustc_serialize::{Encodable,Decodable};
use self::unix_socket::{UnixStream};

struct DBClient{
    sock: UnixStream,
}

use memrange::Range;
use self::theban_db_server::instruction::{DBInstruction, DBAnswer, WriteAccess};
use theban_db::{Bitmap, BitmapSlice};
use error::{DBResult, DBInterfaceError};

use ::DBInterface;
use ::BitmapTableIter;
use ::ObjectTableRawIter;
use ::ObjectTableIter;
use ::decode_obj;

impl DBClient {
    pub fn send(&mut self,instr: DBInstruction) -> DBResult<()>{
        try!(theban_db_server::encoding::encode_instruction(instr, &mut self.sock));
        return Ok(())
    }

    pub fn read(&mut self) -> DBResult<DBAnswer>{
        let ans = try!(theban_db_server::encoding::decode_answer(&mut self.sock));
        return Ok(ans)
    }
}

fn to_bitmap_table_iter<'db>(args: (Range,Vec<(Range,Bitmap)>)) -> BitmapTableIter<'db>{
    let (rng,bmps) = args;
    Box::new(bmps.into_iter().map(move |(k,bmp)| (rng,k.clone(),BitmapSlice::new_from_owned(bmp) )))
}

fn to_object_table_iter<'db,T: Decodable>(args: (Range,Vec<(Range,Vec<u8>)>)) -> ObjectTableIter<'db,T>{
    let (qrng,objs) = args;
    Box::new(objs.into_iter().map(move |(rng,data)| (qrng,rng,decode_obj(rng,&data)) ))
}

fn to_object_table_raw_iter<'db>(args: (Range,Vec<(Range,Vec<u8>)>)) -> ObjectTableRawIter<'db>{
    let (qrng,objs) = args;
    Box::new(objs.into_iter().map(move |(rng,data)| (qrng,rng,Cow::Owned(data)) ))
}

impl DBInterface for DBClient  {
    fn saveas(&mut self, path: &String) -> DBResult<()>{
        return self.send(DBInstruction::Save(path.clone()))
    }

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8> ) ->DBResult<()>{
        return self.bin_put_many(tbl, vec![(rng,data)])
    }
    fn bin_put_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> ) ->DBResult<()>{
        let writes = setters.into_iter().map(|(rng,data)| WriteAccess::new(rng, data)).collect();
        return self.send(DBInstruction::BPut(tbl.clone(), writes ));
    }

    fn bin_get<'db>(&'db mut self, tbl: &String, rng: Range) -> DBResult<BitmapTableIter<'db>>{
        return self.bin_get_many(tbl, vec![rng])
    }

    fn bin_get_many<'db>(&'db mut self, tbl: &String, ranges: Vec<Range>) -> DBResult<BitmapTableIter<'db>>{
        try!(self.send(DBInstruction::BGet(tbl.clone(), ranges)));
        let answer = try!(self.read());
        if let DBAnswer::Bitmap(content) = answer {
            let iter: BitmapTableIter<'db> = Box::new(content.into_iter().flat_map(to_bitmap_table_iter));
            return Ok(iter)
        } else {
            return Err(DBInterfaceError::UnexpectedResponse(format!("Expected bitmap, got {:?}", &answer)))
        }
    }

    fn bin_del(&mut self, tbl: &String, rng: Range)-> DBResult<()>{
        self.bin_del_many(tbl, vec![rng])
    }

    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>) -> DBResult<()>{
        self.send(DBInstruction::BDel(tbl.clone(), ranges))
    }

    fn obj_put<T: Encodable>(&mut self,tbl: &String, rng: Range, obj: &T )-> DBResult<()>{
        self.obj_put_many(tbl, vec![(rng, obj)])
    }
    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, args: Vec<(Range, &T)> )-> DBResult<()>{
        self.obj_put_raw_many(tbl, args.iter().map(|&(r,o)| (r,::encode_obj(r,o).unwrap()) ).collect() )
    }
    
    fn obj_get<'db, T: Decodable +'db>(&'db mut self, tbl: &String, rng: Range) -> DBResult<ObjectTableIter<T>>{
        self.obj_get_many(tbl, vec![rng])
    }

    fn obj_get_many<'db, T: Decodable + 'db>(&'db mut self, tbl: &String, ranges: Vec<Range> ) -> DBResult<ObjectTableIter<T>>{
        try!(self.send(DBInstruction::OGet(tbl.clone(), ranges)));
        let answer = try!(self.read());
        if let DBAnswer::Tags(content) = answer {
            let iter: ObjectTableIter<'db,T> = Box::new(content.into_iter().flat_map(to_object_table_iter));
            return Ok(iter)
        } else {
            return Err(DBInterfaceError::UnexpectedResponse(format!("Expected object, got {:?}", &answer)))
        }
    }
    
    fn obj_put_raw(&mut self, tbl: &String, range: Range, obj: Vec<u8> )-> DBResult<()>{
        self.obj_put_raw_many(tbl, vec![(range, obj)])
    }

    fn obj_put_raw_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)> )-> DBResult<()>{
        let write_accesses = args.into_iter().map(|(rng,data)| WriteAccess::new(rng,data)).collect();
        self.send(DBInstruction::OPut(tbl.clone(), write_accesses))
    }

    fn obj_get_raw<'db>(&'db mut self, tbl: &String, range: Range ) -> DBResult<ObjectTableRawIter<'db>>{
        return self.obj_get_raw_many(tbl, vec![range])
    }

    fn obj_get_raw_many<'db>(&'db mut self, tbl: &String, ranges: Vec<Range> ) -> DBResult<ObjectTableRawIter<'db>>{
        try!(self.send(DBInstruction::OGet(tbl.clone(), ranges)));
        let answer = try!(self.read());
        if let DBAnswer::Tags(content) = answer {
            let iter: ObjectTableRawIter<'db> = Box::new( content.into_iter().flat_map(to_object_table_raw_iter) );
            return Ok(iter)
        } else {
            return Err(DBInterfaceError::UnexpectedResponse(format!("Expected object, got {:?}", &answer)))
        }
    }
    
    fn obj_del(&mut self, tbl: &String, rng: Range )-> DBResult<()>{
        self.obj_del_many(tbl, vec![rng])
    }

    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> )-> DBResult<()>{
        self.send(DBInstruction::ODel(tbl.clone(), ranges))
    }

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range )-> DBResult<()>{
        self.obj_del_intersecting_many(tbl, vec![rng])
    }

    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> )-> DBResult<()>{
        self.send( DBInstruction::ODelAll( tbl.clone(), ranges ) )
    }
}
