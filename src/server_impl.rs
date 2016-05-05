extern crate theban_db_server;

struct DBClient{
    sock: UnixSocket,
}

impl DBClient {
    send(instr DBInstruction) -> DBResult<()>{
        try!(self.sock.write(try!(serialize(instr))));
        return Ok(())
    }

    read() -> DBresult<DBAnswer>{
        unimplemented!()
    }
}

impl DBInterface for DBClient  {
    fn saveas(&mut self, path: &String) -> DBResult<()>{
        self.send(DBInstruction::Save(path.clone()));
    }

    fn bin_put(&mut self, tbl: &String, rng: Range, data: Vec<u8> ) {
        self.bin_put_many(tbl, vec![(Range,Vec<u8>)])
    }
    fn bin_put_many(&mut self, tbl: &String, setters: Vec<(Range,Vec<u8>)> ){
        self.send(DBInstruction::BPut(tbl.clone(), 1, setters.map(|(rng,data)| WriteAcces::new(rng, data)])))
    }

    fn bin_get<'db>(&'db self, tbl: &String, rng: Range) -> DBResult<BitmapTableIter<'db>>{
        return self.bin_get_mayn(tbl, vec![rng])
    }

    fn bin_get_many<'db>(&'db self, tbl: &String, ranges: Vec<Range>) -> DBResult<BitmapTableIter<'db>>{
        try!(self.send(DBInstruction::BGet(tbl.clone(), ranges)));
        let answer = try!(self.read())
        if let Bitmap(content) = answer {
            return Box::new(content.iter.flat_map(|range,bmps| bmps.map(|(k,v)| (range,k,&v)) ))
        } else {
            return Err(DBinterfaceError::UnexpectedResponseError())
        }
    }

    fn bin_del(&mut self, tbl: &String, rng: Range){
        self.bin_del_many(tbl, vec![rng])
    }

    fn bin_del_many(&mut self, tbl: &String, ranges: Vec<Range>){
        self.send(DBInstruction::BDel(tbl.clone, 1, ranges))
    }

    fn obj_put<T: Encodable>(&mut self,tbl: &String, rng: Range, obj: &T ){
        self.obj_put_many(tbl, vec![(rng, T)])
    }
    fn obj_put_many<T: Encodable>(&mut self, tbl: &String, args: Vec<(Range, &T)> ){
        self.obj_put_raw_many(tbl, args.map(|r,o| (r,::decode_obj(r,o).unwrap())))
    }
    
    fn obj_get<'db, T: Decodable +'db>(&'db self, tbl: &String, rng: Range) -> DBResult<ObjectTableIter<T>>{
        self.obj_get_many(tbl, vec![rng])
    }

    fn obj_get_many<'db, T: Decodable + 'db>(&'db self, tbl: &String, ranges: Vec<Range> ) -> DBResult<ObjectTableIter<T>>;
    
    fn obj_put_raw(&mut self, tbl: &String, range: Range, obj: Vec<u8> ){
        self.obj_put_raw_many(tbl, vec![range, obj])
    }

    fn obj_put_raw_many(&mut self, tbl: &String, args: Vec<(Range,Vec<u8>)> ){
        self.send(DBinstruction::OPut(tbl.clone, args.map(|rng,data| WriteAccess::new(rng,data))))
    }

    fn obj_get_raw<'db>(&'db self, tbl: &String, range: Range ) -> DBResult<ObjectTableRawIter<'db>>{
        return self.obj_get_raw_many(tbl, vec![range])
    }

    fn obj_get_raw_many<'db>(&'db self, tbl: &String, args: Vec<Range> ) -> DBResult<ObjectTableRawIter<'db>>;
    
    fn obj_del(&mut self, tbl: &String, rng: Range ){
        self.obj_del_many(tbl, vec![rng])
    }

    fn obj_del_many(&mut self, tbl: &String, ranges: Vec<Range> ){
        self.send(DBInstruction::ODel(tbl, ranges))
    }

    fn obj_del_intersecting(&mut self, tbl: &String, rng: Range ){
        self.obj_del_intersecting_many(tbl, vec![rng])
    }

    fn obj_del_intersecting_many(&mut self, tbl: &String, ranges: Vec<Range> ){
        self.send( DBInstruction::ODelAll( tbl.cone(), ranges ) )
    }
}
