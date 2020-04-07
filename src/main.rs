#![allow(dead_code)]
#![allow(unused)]

#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate glob;

extern crate chrono;
extern crate uuid;

pub mod database;
pub mod params;

use database::db_indicator::*;
use database::db_indicator_set::*;
use database::*;
use glob::glob;
use params::*;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

extern crate rand;
use rand::prelude::*;
use std::collections::HashMap;

fn main() {
    use database::schema::indicators::dsl::*;

    let connection = establish_connection();

    // let num_del = diesel::delete(indicators.filter(name.like("test1")))
    //     .execute(&connection)
    //     .expect("delete failed");
    // println!("del {}", num_del);

    // let indi = create_indicator(&connection, "test1");
    // println!("{:?}", indi);

    // let results = indicators
    //     // .filter(parent_ranged_id.eq(Some))
    //     .limit(5)
    //     .load::<DbIndicator>(&connection)
    //     .expect("Error loading indicators");

    // println!("Displaying {} posts", results.len());
    // for indi in results {
    //     println!("{}", indi.name);
    //     println!("-----------\n");
    // }

    // update_indicator(&connection, &indi);
    // let _ = store_indicator(
    //     &connection,
    //     &Indicator {
    //         name: "test123".into(),
    //         shift: 0,
    //         inputs: vec_vec_to_bigdecimal(vec![vec![0.6], vec![20.0, 40.0, 2.0], vec![40.0]]),
    //     },
    //     None,
    //     DbIndiFunc::Confirm,
    // );
    // let ind = load_indicator(&connection, indi.indicator_id);
    // println!("{:?}", ind);

    // let indis = load_all_indicators_from_file(&connection).unwrap();
    // println!("{:?}", indis);

    // use database::db_indicator::DbIndiFunc::*;
    // let pop = generate_population(
    //     &connection,
    //     200,
    //     vec![Confirm, Confirm2, Baseline, Volume, Exit],
    // )
    // .unwrap();
    // println!("{:?}", pop);

    generate_test_results(&connection).unwrap();
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

// pub fn create_indicator<'a>(conn: &PgConnection, name: &'a str) -> db_indicator::DbIndicator {
//     use schema::indicators;

//     let new_indi = db_indicator::NewDbIndicator {
//         parent_id: None,
//         child_id: None,
//         indicator_name: name,
//         shift: 0i16,
//         func:  DbIndiFunc::Exit,
//     };

//     diesel::insert_into(indicators::table)
//         .values(&new_indi)
//         .get_result(conn)
//         .expect("Error saving new indicator")
// }

pub fn update_indicator(conn: &PgConnection, indi: &db_indicator::DbIndicator) {
    use schema::indicators::dsl::*;

    let ii = diesel::update(indicators.find(indi.indicator_id))
        .set(parent_id.eq(indi.indicator_id - 1))
        .get_result::<db_indicator::DbIndicator>(conn)
        .expect(&format!("Unable to update indicator {}", indi.indicator_id));
    println!("updated indicator {:?}", ii);
}

// TODO this is a really ugly implementation
pub fn load_all_indicators_from_file<'a>(conn: &PgConnection) -> QueryResult<Vec<DbIndicator>> {
    use database::schema::indicators;
    use DbIndiFunc::*;

    let mut indis: Vec<DbIndicator> = vec![];
    for entry in glob("config/indicator/*/*").unwrap().filter_map(Result::ok) {
        println!("loading-indicator-file: {:?}", entry);
        let func = match entry
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
        {
            "confirm" => Confirm,
            "baseline" => Baseline,
            "exit" => Exit,
            "volume" => Volume,
            "continue" => Continue,
            e => panic!("unknown func {:?}", e),
        };
        let indi: Indicator = serde_any::from_file(entry).unwrap();
        indis.push(store_indicator(conn, &indi, None, func)?);

        // indis.push((func, &indi).into());
        match func {
            Confirm => {
                indis.push(store_indicator(conn, &indi, None, Confirm2)?);
                indis.push(store_indicator(conn, &indi, None, Confirm3)?);
                indis.push(store_indicator(conn, &indi, None, Exit)?);
                indis.push(store_indicator(conn, &indi, None, Continue)?);
            }
            Baseline => {
                indis.push(store_indicator(conn, &indi, None, Confirm)?);
                indis.push(store_indicator(conn, &indi, None, Confirm2)?);
                indis.push(store_indicator(conn, &indi, None, Confirm3)?);
                indis.push(store_indicator(conn, &indi, None, Exit)?);
                indis.push(store_indicator(conn, &indi, None, Continue)?);
            }
            _ => (),
        }
    }

    // diesel::insert_into(indicators::table)
    //     .values(&indis)
    //     .get_results(conn)
    Ok(indis)
}

fn generate_population(
    conn: &PgConnection,
    size: usize,
    functions: Vec<DbIndiFunc>,
) -> QueryResult<Vec<DbIndicatorSet>> {
    use database::schema::indicator_sets;
    use database::schema::indicators::dsl::*;
    use rand::seq::SliceRandom;

    let mut fmap = HashMap::<DbIndiFunc, Vec<DbIndicator>>::new();
    for f in &functions {
        // TODO load all at once and group locally instead of loadin individually
        // FIXME load filter parent_id.is_null() or child_id.is_null()
        fmap.insert(*f, indicators.filter(func.eq(f)).load::<DbIndicator>(conn)?);
    }

    let mut rng = rand::thread_rng();
    let mut set_indis = Vec::<DbSetIndicator>::new();
    let mut indi_sets = Vec::<DbIndicatorSet>::new();
    for i in 0..size {
        let indi_set = store_new_db_indicator_set(conn)?;
        for f in &functions {
            set_indis.push(DbSetIndicator {
                indicator_set_id: indi_set.indicator_set_id,
                indicator_id: fmap.get(&f).unwrap().choose(&mut rng).unwrap().indicator_id,
            })
        }
        indi_sets.push(indi_set);
    }

    // TODO check if the indicator_set already exists

    // store all SetIndicators at once
    let _ = store_set_indicators(conn, set_indis)?;
    Ok(indi_sets)
}

// fn generate_population(
//     conn: &PgConnection,
//     size: usize,
//     functions: Vec<DbIndiFunc>,
// ) -> QueryResult<Vec<DbIndicatorSet>> {
//     use database::schema::indicators::dsl::*;
//     use database::schema::indicator_sets;
//     use rand::seq::SliceRandom;

//     // let fmap = HashMap::<DbIndiFunc, DbIndicator>;
//     for f in functions {
//         let indis = indicators
//             .filter(func.eq(f))
//             .load::<DbIndicator>(conn)?;

//         // TODO I need this to be "mit zuruecklegen"
//         // let selection: Vec<&DbIndicator> = indis.choose_multiple(&mut rand::thread_rng(), size)
//                                                 // .cloned().collect();
//     }

//     unimplemented!()
// }

fn generate_test_results(conn: &PgConnection) -> QueryResult<Vec<result::RunResult>> {
    use bigdecimal::BigDecimal;
    use chrono::{NaiveDate, NaiveDateTime, Utc};
    use database::db_indicator::DbIndiFunc::*;
    use database::run::*;
    use database::run_session::*;
    use database::schema::indicators;
    use database::schema::run_sessions;
    use database::schema::runs;
    use database::schema::set_indicators;
    use database::schema::indicator_inputs;
    use database::symbols::*;
    use diesel::insert_into;
    use rand::seq::SliceRandom;

    // create symbols
    let symbol_set = store_default_forex_symbols(conn)?;

    // create run_session
    let run_session = insert_into(run_sessions::table)
        .values(NewRunSession {
            start_date: NaiveDate::from_ymd(2017, 12, 10), // TODO investigate the use of now{} http://docs.diesel.rs/diesel/dsl/struct.now.html
            end_date: NaiveDate::from_ymd(2019, 12, 10),
            expert_version: None,
            symbol_set_id: symbol_set.id().to_owned(),
        })
        .get_result::<RunSession>(conn)?;

    // get an indicator_set
    let mut pop = generate_population(conn, 5, vec![Confirm, Confirm2, Baseline, Volume, Exit])?;

    // TODO only runs on the first
    let indi_set_ranged = pop.pop().unwrap();

    // create run
    let run = insert_into(runs::table)
        .values(NewRun {
            session_id: run_session.id().to_owned(),
            run_date: Utc::now().naive_utc(),
            indicator_set_id: indi_set_ranged.id().to_owned(),
        })
        .get_result::<Run>(conn)?;

    let set_indis =
        DbSetIndicator::belonging_to(&indi_set_ranged).get_results::<DbSetIndicator>(conn)?;

    // let mut indi_set_map = HashMap::<DbIndiFunc, (DbIndicator, Vec<DbIndicatorInput>)>::new();
    let mut indi_set_inputs = Vec::<(DbIndicator, Vec<DbIndicatorInput>)>::new();
    for set_indi in set_indis {
        indi_set_inputs.push(load_indicator(conn, set_indi.indicator_id)?);
    }

    // spread out the ranged inputs get a Vec of singles
    let mut inputs_spread = HashMap::<DbIndiFunc, HashMap<i16, Vec<BigDecimal>>>::new();
    for (indi, inputs) in &indi_set_inputs {
        let mut input_map: HashMap<i16, Vec<BigDecimal>> = inputs
            .iter()
            .filter(|i| i.start.is_none() && i.input.is_some()) // take all inputs that are not ranged
            .map(|i| (i.index, vec![i.input.clone().unwrap()])) // create a hashmap out of it with the key per index
            .collect();

        let ranged_inputs = inputs.iter().filter(|i| i.start.is_some());

        for i in ranged_inputs {
            let mut input_vec = Vec::<BigDecimal>::new();
            step_by(
                i.start.clone().unwrap(),
                i.stop.clone().unwrap(),
                i.step.clone().unwrap(),
                |j| {
                    input_vec.push(j.to_owned());
                },
            );

            assert!(!input_map.contains_key(&i.index));
            input_map.insert(i.index, input_vec);
        }

        inputs_spread.insert(indi.func, input_map);
    }

    // create the new indi_sets from the spread-out inputs
    let mut rng = rand::thread_rng();
    for _ in (0..1) {
        let new_indis_for_set = indi_set_inputs
            .iter()
            .map(|(indi, _)| indi.new_child_no_ref(conn).unwrap())
            .collect::<Vec<DbIndicator>>();

        let new_indi_set = store_new_indicator_set(conn, &new_indis_for_set)?;

        // rand chose for every index according to func
        let mut new_inputs = Vec::<DbIndicatorInput>::new();
        for new_indi in new_indis_for_set {
            // println!("new_indi {:?}", indicators::)
            new_inputs.extend(inputs_spread.get(&new_indi.func).unwrap().iter().map(
                |(idx, inputs)| {
                    DbIndicatorInput {
                        indicator_id: new_indi.id().to_owned(),
                        index: idx.to_owned(),
                        input: Some(inputs.choose(&mut rng).unwrap().to_owned()), // if choose return None we have to deal with the error
                        start: None,
                        stop: None,
                        step: None,
                    }
                },
            ));
        }
        // println!(
        //     "{:#?}\n\n{:#?}",
        //     DbSetIndicator::belonging_to(&new_indi_set).get_results::<DbSetIndicator>(conn)?,
        //     new_inputs
        // );
        insert_into(indicator_inputs::table).values(new_inputs).execute(conn)?;
    }

    // create results for (some) of the indicators sets within this indicator_set

    let dummy = Vec::<result::RunResult>::new();
    Ok(dummy)
}

use std::ops::Add;
fn step_by<T, F>(start: T, end_inclusive: T, step: T, mut body: F)
where
    T: Add<Output = T> + PartialOrd + Clone,
    F: FnMut(&T),
{
    let mut i: T = start;
    while i <= end_inclusive {
        body(&i);
        i = i + step.clone();
    }
}
