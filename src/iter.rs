extern crate theban_db;
extern crate theban_interval_tree;

use ::memrange::Range;
use theban_db::BitmapSliceIter;
use self::theban_interval_tree::RangePairIter;

use std::borrow::Cow;
use std::marker::PhantomData;

use theban_db::{DB, Object, BitmapSlice};

use rustc_serialize::Decodable;
use ::DBResult;

pub trait Queryable<'db >: Sized {
    fn get_next_iter_for(db: &'db DB, tbl: &String, rng: Range) -> Option<Self>;
} 


//Todo: Can this be replaced with std::iter::FlatMap?
//pub struct ManyRangeIter<'db, Iter: Iterator<Item=(Range, InnerItem)> + Queryable<'db>, InnerItem> {
pub struct ManyRangeIter<'db, Iter: Iterator<Item=(Range, InnerItem)> + Queryable<'db>, InnerItem> {
    database: &'db DB,
    table: String,
    current_range: Option<Range>,
    current_range_iter: Box<Iterator<Item=Range>>,
    current_query_iter: Option<Iter>,
}

impl<'db, Iter: Iterator<Item=(Range,InnerItem)> + Queryable<'db>, InnerItem > ManyRangeIter<'db, Iter, InnerItem> {
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
            self.current_query_iter = self.current_range.and_then(|rng| Iter::get_next_iter_for(self.database, &self.table, rng) )
         }
    }
}

impl<'db, Iter: Iterator<Item=(Range,InnerItem)> + Queryable<'db>, InnerItem > Iterator for ManyRangeIter<'db, Iter, InnerItem> {

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

impl<'db> Queryable<'db> for BitmapSliceIter<'db> {
    fn get_next_iter_for(db: &'db DB, tbl: &String, rng: Range) -> Option<BitmapSliceIter<'db>>{
        return db.query_bitmap(tbl, rng);
    }
}

pub type ManyBitmapSlicesIter<'db> = ManyRangeIter<'db, BitmapSliceIter<'db>, BitmapSlice<'db>>;


impl<'db> Queryable<'db> for RangePairIter<'db, Object> {
    fn get_next_iter_for(db: &'db DB, tbl: &String, rng: Range) -> Option<RangePairIter<'db, Object>>{
        return db.query_object(tbl, rng);
    }
}

pub type ManyObjectsIter<'db> = ManyRangeIter<'db, RangePairIter<'db, Object>, &'db Object>;

pub struct ManyObjectsDataIter<'db> {
    orig_iter: ManyRangeIter<'db, RangePairIter<'db, Object>, &'db Object>,
}

impl<'db> Iterator for ManyObjectsDataIter<'db> {
    type Item = (Range, Range, Cow<'db,Vec<u8>>);
    fn next(&mut self) -> Option< (Range, Range, Cow<'db,Vec<u8>>) > {
        self.orig_iter.next().map(|(q,r, obj)| (q,r, Cow::Borrowed(&obj.data)))
    }
}

impl<'db> ManyObjectsDataIter<'db> {
    pub fn new(db: &'db DB, tbl: String, rngs: Vec<Range>) -> Self {
        return ManyObjectsDataIter::<'db>{orig_iter: ManyObjectsIter::new(db, tbl, rngs)}
    }
}

pub struct ManyObjectsDecodedIter<'db,T> {
    orig_iter: ManyRangeIter<'db, RangePairIter<'db, Object>, &'db Object>,
    converted_object_type: PhantomData<T>,
}

impl<'db,T: Decodable> Iterator for ManyObjectsDecodedIter<'db,T> {
    type Item = (Range, Range, DBResult<T>);
    fn next(&mut self) -> Option<(Range, Range, DBResult<T>)> {
        self.orig_iter.next().map(|(q,r, obj)| (q,r, ::decode_obj(r, &obj.data) ) )
    }
}

impl<'db, T> ManyObjectsDecodedIter<'db, T> {
    pub fn new(db: &'db DB, tbl: String, rngs: Vec<Range>) -> Self {
        return ManyObjectsDecodedIter::<'db,T>{orig_iter: ManyObjectsIter::new(db, tbl, rngs), converted_object_type: PhantomData}
    }
}
