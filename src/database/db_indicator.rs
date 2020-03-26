use super::super::params::Indicator;
use super::schema::*;
use super::*;

use diesel::prelude::*;

#[derive(Queryable, Associations, Identifiable, Debug)]
#[table_name = "indicators"]
#[belongs_to(Indicator, foreign_key = "parent_id")]
pub struct DbIndicator {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub shift: i16,
}

#[derive(Insertable)]
#[table_name = "indicators"]
pub struct NewDbIndicator<'a> {
    pub parent_id: Option<i32>,
    pub name: &'a str,
    pub shift: i16,
}

impl<'a> From<&'a Indicator> for NewDbIndicator<'a> {
    fn from(indi: &'a Indicator) -> Self {
        NewDbIndicator {
            parent_id: None,
            name: &indi.name,
            shift: indi.shift as i16,
        }
    }
}

impl From<(DbIndicator, Vec<DbIndicatorInput>)> for Indicator {
    fn from((indi, mut indi_inputs): (DbIndicator, Vec<DbIndicatorInput>)) -> Self {
        indi_inputs.sort_by_key(|v| v.index);
        Indicator {
            name: indi.name,
            shift: indi.shift as u8,
            inputs: indi_inputs
                .iter()
                .map(|v| {
                    let mut input_vec = Vec::<f32>::new();
                    if let Some(inp) = v.input {
                        input_vec.push(inp);
                    }
                    if let (Some(sta), Some(sto), Some(ste)) = (v.start, v.stop, v.step) {
                        input_vec.extend([sta, sto, ste].iter().copied());
                    }
                    // TODO should not panic!
                    assert!(input_vec.len() > 0 && input_vec.len() <= 4);
                    input_vec
                })
                .collect(),
        }
    }
}

impl DbIndicator {
    pub fn child(
        self: Self,
        conn: &PgConnection,
        indi: &Indicator,
    ) -> Result<DbIndicator, diesel::result::Error> {
        store_indicator(conn, indi, Some(self.id))
    }

    pub fn parent(
        self: Self,
        conn: &PgConnection,
        indi: &DbIndicator,
    ) -> Result<Option<DbIndicator>, diesel::result::Error> {
        match indi.parent_id {
            Some(p) => DbIndicator::try_load(conn, p).map(|i| Some(i)),
            None => Ok(None),
        }
    }

    pub fn try_load(
        conn: &PgConnection,
        indi_id: i32,
    ) -> Result<DbIndicator, diesel::result::Error> {
        use schema::indicators::dsl::*;
        indicators.find(indi_id).first::<DbIndicator>(conn)
    }
}

#[derive(Queryable, Insertable, Identifiable, Associations, Debug)]
#[primary_key(indicator_id, index)]
#[belongs_to(Indicator)]
#[table_name = "indicator_inputs"]
pub struct DbIndicatorInput {
    pub indicator_id: i32, // 1:m
    pub index: i16,
    pub input: Option<f32>,
    pub start: Option<f32>,
    pub stop: Option<f32>,
    pub step: Option<f32>,
}

