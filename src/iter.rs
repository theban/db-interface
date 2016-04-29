extern crate theban_db;

use ::memrange::Range;
use theban_db::BitmapSliceIter;

use theban_db::DB;
use theban_db::BitmapSlice;

//pub struct ManyBitmapSliceIter<'a> {
//    database: &'a DB,
//    table: String,
//    current_range_iter: Box<Iterator<Item=Range>>,
//    current_query_iter: Option<BitmapSliceIter<'a>>,
//}
//
//impl<'a> ManyBitmapSliceIter<'a> {
//    pub fn new(db: &DB, tbl: String, rngs: Vec<Range>) -> ManyBitmapSliceIter {
//        return ManyBitmapSliceIter{ database: db,table: tbl, current_range_iter: Box::new(rngs.into_iter()), current_query_iter: None}
//    }
//
//    fn ensure_current_query_iter(&mut self) {
//        if self.current_query_iter.is_none() {
//            let next_range = self.current_range_iter.next();
//            self.current_query_iter = next_range.and_then(|rng| self.database.query_bitmap( &self.table, rng))
//         }
//    }
//}
//
//impl<'a> Iterator for ManyBitmapSliceIter<'a> {
//
//    type Item = (Range,Range,BitmapSlice<'a>);
//
//    fn next(&mut self) -> Option<(Range,Range,BitmapSlice<'a>)> {
//
//        // makes sure that current_query_iter is Some if we have remaining ranges (even if the
//        // current query_iter is used up
//        self.ensure_current_query_iter();
//
//
//        if let Some(ref mut query_iter) = self.current_query_iter.take() {
//
//            //query_iter for the current range has an element => return it
//            if let Some((rng, bitmap)) =  query_iter.next() {
//                return Some( ( query_iter.get_range(), rng, bitmap ) )
//            }
//
//            //the query_iter for the current range has no more elements => recurse to get query_iter for the next range
//            self.current_query_iter = None;
//            return self.next();
//        }
//        //even after enusre_current_query_iter(), there is no query_iter => all ranges have been
//        //used up, iteration finished.
//        return None;
//    }
//}


pub trait Queryable<'db, Iter: Iterator<Item=(Range,Self)> >: Sized {
    fn get_next_iter_for(db: &'db DB, tbl: &String, rng: Range) -> Option<Iter>;
} 

pub struct ManyRangeIter<'db, Iter: Iterator<Item=(Range, InnerItem)>, InnerItem: Queryable<'db, Iter>> {
    database: &'db DB,
    table: String,
    current_range: Option<Range>,
    current_range_iter: Box<Iterator<Item=Range>>,
    current_query_iter: Option<Iter>,
}

impl<'db, Iter: Iterator<Item=(Range,InnerItem)>, InnerItem: Queryable<'db, Iter> > ManyRangeIter<'db, Iter, InnerItem> {
    pub fn new(db: &'db DB, tbl: String, rngs: Vec<Range>) -> Self {
        return ManyRangeIter{ 
            database: db,
            table: tbl, 
            current_range: None, 
            current_range_iter: Box::new(rngs.into_iter()), 
            current_query_iter: None
        }
    }

    fn ensure_current_query_iter(&mut self) {
        if self.current_query_iter.is_none() {
            self.current_range = self.current_range_iter.next();
            self.current_query_iter = self.current_range.and_then(|rng| InnerItem::get_next_iter_for(self.database, &self.table, rng) )
         }
    }
}

impl<'db, Iter: Iterator<Item=(Range,InnerItem)>, InnerItem: Queryable<'db, Iter> > Iterator for ManyRangeIter<'db, Iter, InnerItem> {

    type Item = (Range,Range,InnerItem);

    fn next(&mut self) -> Option<(Range,Range,InnerItem)> {

        // makes sure that current_query_iter is Some if we have remaining ranges (even if the
        // current query_iter is used up
        self.ensure_current_query_iter();


        if let Some(ref mut query_iter) = self.current_query_iter.take() {

            //query_iter for the current range has an element => return it
            if let Some((rng, data)) =  query_iter.next() {
                return Some( ( self.current_range.unwrap(), rng, data ) )
            }

            //the query_iter for the current range has no more elements => recurse to get query_iter for the next range
            self.current_query_iter = None;
            return self.next();
        }
        //even after enusre_current_query_iter(), there is no query_iter => all ranges have been
        //used up, iteration finished.
        return None;
    }
}

impl<'db> Queryable<'db, BitmapSliceIter<'db>> for BitmapSlice<'db> {
    fn get_next_iter_for(db: &'db DB, tbl: &String, rng: Range) -> Option<BitmapSliceIter<'db>>{
        return db.query_bitmap(tbl, rng);
    }
}

pub type ManyBitmapSlicesIter<'db> = ManyRangeIter<'db, BitmapSliceIter<'db>, BitmapSlice<'db>>;

