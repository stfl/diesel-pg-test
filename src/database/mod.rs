pub mod db_indicator;
pub mod schema;

use super::params::Indicator;
use diesel::prelude::*;
use db_indicator::*;

pub fn store_indicator(
    conn: &PgConnection,
    indi: &Indicator,
    parent: Option<i32>,
) -> Result<DbIndicator, diesel::result::Error> {
    use schema::indicator_inputs::dsl::*;
    use schema::indicators::dsl::*;

    let mut new_db_indi = NewDbIndicator::from(indi);
    new_db_indi.parent_id = parent;

    let new_indi: DbIndicator = diesel::insert_into(indicators)
        .values(new_db_indi)
        .get_result(conn)
        .expect("Error saving new indicator");

    let indi_inputs: Vec<DbIndicatorInput> = indi
        .inputs
        .iter()
        .enumerate()
        .map(|(i, v)| match v.len() {
            1 => DbIndicatorInput {
                indicator_id: new_indi.id,
                index: i as i16,
                input: Some(v[0] as f32),
                start: None,
                stop: None,
                step: None,
            },
            3 => DbIndicatorInput {
                indicator_id: new_indi.id,
                index: i as i16,
                input: None,
                start: Some(v[0] as f32),
                stop: Some(v[1] as f32),
                step: Some(v[2] as f32),
            },
            4 => DbIndicatorInput {
                indicator_id: new_indi.id,
                index: i as i16,
                input: Some(v[0] as f32),
                start: Some(v[1] as f32),
                stop: Some(v[2] as f32),
                step: Some(v[3] as f32),
            },
            _ => panic!("wrong number values on input"),
        })
        .collect();

    let res: Vec<DbIndicatorInput> = diesel::insert_into(indicator_inputs)
        .values(&indi_inputs)
        .get_results(conn)
        .expect("Error insert new indicator");
    println!("inserted {:?}\n{:?}", new_indi, res);

    Ok(new_indi)
}

pub fn load_db_indicator(
    conn: &PgConnection,
    indi_id: i32,
) -> Result<DbIndicator, diesel::result::Error> {
    use schema::indicators::dsl::*;
    indicators.find(indi_id).first::<DbIndicator>(conn)
}

pub fn load_indicator(
    conn: &PgConnection,
    indi_id: i32,
) -> Result<Indicator, diesel::result::Error> {
    use schema::indicator_inputs::dsl::*;

    let indi = DbIndicator::try_load(conn, indi_id)?;

    let indi_inputs = indicator_inputs
        .filter(indicator_id.eq(indi_id))
        .load::<DbIndicatorInput>(conn)?;

    Ok((indi, indi_inputs).into())
}
