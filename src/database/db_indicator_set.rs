// use serde_repr::{Serialize_repr, Deserialize_repr};
use super::super::params::Indicator;
use super::schema::indicators;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;

use super::super::params::IndicatorSet;
use super::load_indicator;

// Custom declaration of indicator_sets to allow derive(DbEnum) for IndiFunc
table! {
    use diesel::sql_types::*;
    use super::IndiFuncMapping;
    indicator_sets (set_id, indicator_id) {
        set_id -> Int8,
        indicator_id -> Int4,
        func -> IndiFuncMapping,
    }
}

joinable!(indicator_sets -> indicators (indicator_id));

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
#[table_name = "indicator_sets"]
pub struct DbIndicatorSet {
    set_id: i64,
    indicator_id: i32, // 1:m
    func: IndiFunc,
}

pub fn load_indicator_set(
    conn: &PgConnection,
    indi_set_id: i64,
) -> Result<IndicatorSet, diesel::result::Error> {
    use self::indicator_sets::dsl::*;
    use IndiFunc::*;

    let db_indi_set = indicator_sets
        .filter(set_id.eq(indi_set_id))
        .load::<DbIndicatorSet>(conn)? // load the indicator set from DB
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
) -> Result<Option<Vec<DbIndicatorSet>>, diesel::result::Error> {
    // TODO
    // for each func in indi_set
    // find_db_indicator()
    // SELECT indicator_id, func from indicator_sets
    // WHERE func ==
    unimplemented!();
}

pub fn store_indicator_set(
    conn: &PgConnection,
    indi_set: IndicatorSet,
) -> Result<Option<Vec<DbIndicatorSet>>, diesel::result::Error> {
    // TODO
    // for each func in indi_set
    // find_db_indicator()
    // vec![]
    // insert
    unimplemented!();
}
