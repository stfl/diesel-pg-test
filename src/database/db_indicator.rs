use super::super::params::Indicator;
use super::schema::*;
use super::*;

use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;

// table! {
//     use diesel::sql_types::*;
//     use super::DbIndiFuncMapping;
//     indicator_default_func (indicator_id) {
//         indicator_id -> Int4,
//         func -> DbIndiFuncMapping,
//     }
// }

// joinable!(indicator_default_func -> indicators (indicator_id));

#[derive(Queryable, Associations, Identifiable, Debug, Clone)]
#[primary_key(indicator_id)]
#[table_name = "indicators"]
#[belongs_to(DbIndicator, foreign_key = "parent_id")]
// #[belongs_to(DbIndicator, foreign_key = "child_id")]  FIXME
pub struct DbIndicator {
    pub indicator_id: i32,
    pub parent_id: Option<i32>,
    pub child_id: Option<i32>,
    pub name: String,
    pub shift: i16,
    pub func: DbIndiFunc,
}

#[derive(Insertable)]
#[table_name = "indicators"]
pub struct NewDbIndicator {
    pub parent_id: Option<i32>,
    pub child_id: Option<i32>,
    pub indicator_name: String,
    pub shift: i16,
    pub func: DbIndiFunc,
}

#[derive(Queryable, Insertable, Identifiable, Associations, Debug, Clone)]
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

#[derive(DbEnum, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum DbIndiFunc {
    Confirm,
    Confirm2,
    Confirm3,
    Baseline,
    Volume,
    Continue,
    Exit,
}

// #[derive(Queryable, Insertable, Identifiable, Associations, Debug)]
// #[primary_key(indicator_id)]
// #[belongs_to(DbIndicator, foreign_key = "indicator_id")]
// #[table_name = "indicator_default_func"]
// pub struct DbIndicatorDefaultFunc {
//     pub indicator_id: i32,
//     func: DbIndiFunc,
// }

// should not be implemented like this
// impl<'a> From<&'a Indicator> for NewDbIndicator<'a> {
//     fn from(indi: &'a Indicator) -> Self {
//         NewDbIndicator {
//             parent_id: None,
//             child_id: None,
//             indicator_name: &indi.name,
//             shift: indi.shift as i16,
//             func: DbIndiFunc::Confirm, // the default of Confirm is set here which is an abstraction
//         }
//     }
// }

impl<'a> From<(DbIndiFunc, &'a Indicator)> for NewDbIndicator {
    fn from((func, indi): (DbIndiFunc, &'a Indicator)) -> Self {
        NewDbIndicator {
            parent_id: None,
            child_id: None,
            indicator_name: indi.name.clone(),
            shift: indi.shift as i16,
            func, //: func.to_owned(),
        }
    }
}

impl From<(DbIndiFunc, Indicator)> for NewDbIndicator {
    fn from((func, indi): (DbIndiFunc, Indicator)) -> Self {
        NewDbIndicator {
            parent_id: None,
            child_id: None,
            indicator_name: indi.name,
            shift: indi.shift as i16,
            func,
        }
    }
}

// impl From<DbIndicator> for NewDbIndicator {
//     fn from(indi: &DbIndicator) -> Self {
//         NewDbIndicator {
//             parent_id: None,
//             child_id: None,
//             indicator_name: indi.name.to_owned(),
//             shift: indi.shift as i16,
//             func: indi.func,
//         }
//     }
// }

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
    pub fn store_child(
        self: Self,
        conn: &PgConnection,
        indi: &Indicator,
    ) -> QueryResult<DbIndicator> {
        let child = store_indicator(conn, indi, Some(self.indicator_id), self.func)?;
        self.set_child(conn, &child)
    }

    pub fn new_child_no_ref(self: &Self, conn: &PgConnection) -> QueryResult<DbIndicator> {
        use crate::database::schema::indicators::dsl::*;
        diesel::insert_into(indicators)
            .values(NewDbIndicator {
                parent_id: Some(self.id().to_owned()),
                child_id: None,
                indicator_name: self.name.to_owned(),
                shift: self.shift as i16,
                func: self.func,
            })
            .get_result(conn)
    }

    pub fn new_child(self: Self, conn: &PgConnection) -> QueryResult<DbIndicator> {
        use crate::database::schema::indicators::dsl::*;
        let child = self.new_child_no_ref(conn)?;
        let _ = self.set_child(conn, &child)?;
        Ok(child)
    }

    pub fn set_child(
        self: Self,
        conn: &PgConnection,
        indi: &DbIndicator,
    ) -> QueryResult<DbIndicator> {
        unimplemented!()
    }

    pub fn set_parent() -> QueryResult<DbIndicator> {
        unimplemented!()
    }

    pub fn get_parent(
        self: Self,
        conn: &PgConnection,
        indi: &DbIndicator,
    ) -> QueryResult<Option<DbIndicator>> {
        match indi.parent_id {
            Some(p) => DbIndicator::try_load(conn, p).map(|i| Some(i)),
            None => Ok(None),
        }
    }

    pub fn try_load(conn: &PgConnection, indi_id: i32) -> QueryResult<DbIndicator> {
        use schema::indicators::dsl::*;
        indicators.find(indi_id).first::<DbIndicator>(conn)
    }
}

// TODO implment a trait ToDb which params::Indicator implements
pub fn store_indicator(
    conn: &PgConnection,
    indi: &Indicator,
    parent: Option<i32>,
    indi_func: DbIndiFunc,
) -> Result<DbIndicator, diesel::result::Error> {
    use schema::indicator_inputs::dsl::*;
    use schema::indicators::dsl::*;

    let mut new_db_indi = NewDbIndicator::from((indi_func, indi));
    new_db_indi.parent_id = parent;

    if parent == None {
        // TODO check if an indicator with this name is already in the database
    }

    let new_indi: DbIndicator = diesel::insert_into(indicators)
        .values(new_db_indi)
        .get_result(conn)?;

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

    let indi_inputs: Vec<DbIndicatorInput> = diesel::insert_into(indicator_inputs)
        .values(&indi_inputs)
        .get_results(conn)?;
    // TODO info!()
    println!(
        "inserted indicator: {:?}\nwith inputs: {:?}",
        new_indi, indi_inputs
    );

    Ok(new_indi)
}

// trait ToDb {
//     fn to_db(conn: &PgConnection) -> Result<(), Error>;
// }

// pub fn store_indicators_with_default_func(conn: &PgConnection, indis: &Vec<(DbIndiFunc, Indicator)>) -> QueryResult<Vec<DbIndicator>> {
//     let mut db_indis : Vec<DbIndicator> = vec![];
//     for (f, i) in indis {
//         let db_indi = store_indicator(conn, &i, None)?;
//         let _ = store_indicator_default_func(conn, f, &db_indi);
//         db_indis.push(db_indi);
//     }
//     Ok(db_indis)
// }

// pub fn store_indicator_default_func(conn: &PgConnection, indi_func: &DbIndiFunc, indi: &DbIndicator) -> QueryResult<DbIndicatorDefaultFunc> {
//     use self::indicator_default_func::dsl::*;
//     diesel::insert_into(indicator_default_func)
//         .values(DbIndicatorDefaultFunc {
//             indicator_id: indi.indicator_id,
//             func: indi_func.to_owned(),
//         })
//         .get_result(conn)
// }

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
) -> QueryResult<(DbIndicator, Vec<DbIndicatorInput>)> {
    use schema::indicator_inputs::dsl::*;

    let indi = DbIndicator::try_load(conn, indi_id)?;

    let indi_inputs =
        DbIndicatorInput::belonging_to(&indi).get_results::<DbIndicatorInput>(conn)?;

    // let indi_inputs = indicator_inputs
    //     .filter(indicator_id.eq(indi_id))
    //     .load::<DbIndicatorInput>(conn)?;

    Ok((indi, indi_inputs))
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
