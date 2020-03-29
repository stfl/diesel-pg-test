// use serde_repr::{Serialize_repr, Deserialize_repr};
use super::super::params::Indicator;
use super::schema::{indicator_sets, indicators};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;

use super::super::params::IndicatorSet;
use super::load_indicator;

// Custom declaration of indicator_sets to allow derive(DbEnum) for IndiFunc
table! {
    use diesel::sql_types::*;
    use super::IndiFuncMapping;
    set_indicators (set_id, indicator_id) {
        set_id -> Int8,
        indicator_id -> Int4,
        func -> IndiFuncMapping,
    }
}

joinable!(set_indicators -> indicators (indicator_id));
joinable!(set_indicators -> indicator_sets (set_id));

#[derive(DbEnum, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum IndiFunc {
    Confirm,
    Confirm2,
    Confirm3,
    Baseline,
    Volume,
    Continue,
    Exit,
}

#[derive(Queryable, Insertable, Identifiable, Associations, Debug)]
#[primary_key(set_id, indicator_id)]
#[belongs_to(Indicator)]
#[table_name = "set_indicators"]
pub struct DbSetIndicator {
    set_id: i64,
    indicator_id: i32, // 1:m
    func: IndiFunc,
}

#[derive(Queryable, Insertable, Identifiable, Debug)]
#[primary_key(set_id)]
#[table_name = "indicator_sets"]
pub struct DbIndicatorSet {
    set_id: i64,
}

// #[derive(Insertable, Debug)]
// #[table_name = "indicator_sets"]
// pub struct DbNewIndicatorSet;

pub fn load_indicator_set(
    conn: &PgConnection,
    indi_set_id: i64,
) -> Result<IndicatorSet, diesel::result::Error> {
    use self::set_indicators::dsl::*;
    use IndiFunc::*;

    let db_indi_set = set_indicators
        .filter(set_id.eq(indi_set_id))
        .load::<DbSetIndicator>(conn)? // load the indicator set from DB
        .iter()
        .map(|set| (load_indicator(conn, set.indicator_id).unwrap(), set.func)) // load all indicators specified in the Set
        // FIXME database errors or if the indicator is not found are ignored
        .collect::<Vec<(Indicator, IndiFunc)>>(); // store the Indicator struct together with it's function for the set

    let mut indi_set: IndicatorSet = Default::default();
    for indi in db_indi_set {
        // match IndiFunc
        match indi.1 {
            Confirm => indi_set.confirm = Some(indi.0), // and assign the Indicator struct
            Confirm2 => indi_set.confirm2 = Some(indi.0),
            Confirm3 => indi_set.confirm3 = Some(indi.0),
            Baseline => indi_set.baseline = Some(indi.0),
            Volume => indi_set.volume = Some(indi.0),
            Continue => indi_set.cont = Some(indi.0),
            Exit => indi_set.exit = Some(indi.0),
        }
    }
    Ok(indi_set)
}

pub fn find_db_indicator_set(
    conn: &PgConnection,
    indi_set: IndicatorSet,
) -> Result<Option<Vec<DbSetIndicator>>, diesel::result::Error> {
    // TODO
    // for each func in indi_set
    // find_db_indicator()
    // SELECT indicator_id, func from indicator_sets
    // WHERE func ==
    unimplemented!();
}

pub fn store_plain_indicator_set(
    conn: &PgConnection,
    indi_set: IndicatorSet,
) -> QueryResult<Vec<DbSetIndicator>> {
    let db_indi_set: store_new_db_indicator_set(conn)?;
    // TODO Optional
    // TODO
    // for each func in indi_set
    // find_db_indicator()
    // insert_into
    // if find or insert fails.. -> delete db_indi_set
    unimplemented!();
}

pub fn store_set_indicators(
    conn: &PgConnection,
    indi_set: Vec<DbSetIndicator>,
) -> QueryResult<Vec<DbSetIndicator>> {
    let db_indi_set: store_new_db_indicator_set(conn)?;
    // TODO Optional
    // TODO
    // if find or insert fails.. -> delete db_indi_set
    unimplemented!();
}

// creates a new indicator_set in the DB (a new row) which gets a ne unique id that can be used for set_indicators
pub fn store_new_db_indicator_set(conn: &PgConnection) -> QueryResult<DbIndicatorSet> {
    use super::schema::indicator_sets::dsl::*;
    diesel::insert_into(indicator_sets)
        .default_values()
        .get_result(conn)
}
