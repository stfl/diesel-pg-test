use crate::database::db_indicator_set::DbIndicatorSet;
use crate::database::schema::results;

#[derive(Queryable, Associations, Identifiable, Insertable, Debug)]
#[primary_key(run_id, indicator_set_id)]
#[belongs_to(DbIndicatorSet, foreign_key = "indicator_set_id")]
#[table_name = "results"]
pub struct RunResult {
    pub run_id: i64,
    pub indicator_set_id: i64,
    pub result: f64,
    pub profit: f64, // Money? --> float is probably faster to calculate
    pub trades: i32,
}
