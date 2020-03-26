use super::super::params::Indicator;
use super::schema::*;
use super::*;

use bigdecimal::BigDecimal;
use diesel::prelude::*;

#[derive(Queryable, Associations, Identifiable, Debug)]
#[primary_key(indicator_id)]
#[table_name = "indicators"]
#[belongs_to(DbIndicator, foreign_key = "parent_id")]
pub struct DbIndicator {
    pub indicator_id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub shift: i16,
}

#[derive(Insertable)]
#[table_name = "indicators"]
pub struct NewDbIndicator<'a> {
    pub parent_id: Option<i32>,
    pub indicator: &'a str,
    pub shift: i16,
}

impl<'a> From<&'a Indicator> for NewDbIndicator<'a> {
    fn from(indi: &'a Indicator) -> Self {
        NewDbIndicator {
            parent_id: None,
            indicator: &indi.name,
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
                    // let vv = v.clone();
                    let mut input_vec = Vec::<BigDecimal>::new();
                    if let Some(inp) = &v.input {
                        input_vec.push(inp.to_owned());
                    }
                    if let (Some(sta), Some(sto), Some(ste)) =
                        (v.start.to_owned(), v.stop.to_owned(), v.step.to_owned())
                    {
                        input_vec.extend(vec![sta, sto, ste]);
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
        store_indicator(conn, indi, Some(self.indicator_id))
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
#[belongs_to(DbIndicator, foreign_key = "indicator_id")]
#[table_name = "indicator_inputs"]
pub struct DbIndicatorInput {
    pub indicator_id: i32, // 1:m
    pub index: i16,
    pub input: Option<BigDecimal>,
    pub start: Option<BigDecimal>,
    pub stop: Option<BigDecimal>,
    pub step: Option<BigDecimal>,
}

pub fn store_indicator(
    conn: &PgConnection,
    indi: &Indicator,
    parent: Option<i32>,
) -> Result<DbIndicator, diesel::result::Error> {
    use schema::indicator_inputs::dsl::*;
    use schema::indicators::dsl::*;

    let mut new_db_indi = NewDbIndicator::from(indi);
    new_db_indi.parent_id = parent;

    if parent == None {
        // TODO check if an indicator with this name already in the database
    }

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
                indicator_id: new_indi.indicator_id,
                index: i as i16,
                input: Some(v[0].to_owned()),
                start: None,
                stop: None,
                step: None,
            },
            3 => DbIndicatorInput {
                indicator_id: new_indi.indicator_id,
                index: i as i16,
                input: None,
                start: Some(v[0].to_owned()),
                stop: Some(v[1].to_owned()),
                step: Some(v[2].to_owned()),
            },
            4 => DbIndicatorInput {
                indicator_id: new_indi.indicator_id,
                index: i as i16,
                input: Some(v[0].to_owned()),
                start: Some(v[1].to_owned()),
                stop: Some(v[2].to_owned()),
                step: Some(v[3].to_owned()),
            },
            _ => panic!("wrong number values on input"),
        })
        .collect();

    let res: Vec<DbIndicatorInput> = diesel::insert_into(indicator_inputs)
        .values(&indi_inputs)
        .get_results(conn)?;
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

pub fn find_db_indicator(
    conn: &PgConnection,
    indi: Indicator,
) -> Result<Option<(DbIndicator, Vec<DbIndicatorInput>)>, diesel::result::Error> {
    // TODO this requires a join and then checking if all lines for the inputs match

    // SELECT .. from
    // WHERE indicators.name == indi.name AND indicators.shift == indi.shift
    // JOIN indicator_inputs
    // ON indicators.id == indicator_inpus.indicator_id
    // ORDER BY indicators.id and indicator_inputs.index
    unimplemented!();
}
