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
    // // let indis = store_all_indicators(&connection, &indis).unwrap();
    // println!("{:?}", indis);
    use database::db_indicator::DbIndiFunc::*;
    let pop = generate_population(&connection, 200, vec![Confirm, Confirm2, Baseline, Volume, Exit]).unwrap();
    println!("{:?}", pop);

    // let indi_set = store_new_db_indicator_set(&connection);
    // println!("indi set with new id: {:?}", indi_set);
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
        println!("loading indicator file: {:?}", entry);
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
    use database::schema::indicators::dsl::*;
    use database::schema::indicator_sets;
    use rand::seq::SliceRandom;

    let mut fmap = HashMap::<DbIndiFunc, Vec<DbIndicator>>::new();
    for f in &functions {
        // TODO load all at once and group locally instead of loadin individually
        fmap.insert(*f, indicators
            .filter(func.eq(f))
            .load::<DbIndicator>(conn)?);
    }

    let mut rng = rand::thread_rng();
    let mut set_indis = Vec::<DbSetIndicator>::new();
    let mut indi_sets = Vec::<DbIndicatorSet>::new();
    for i in 0..size {
        let indi_set = store_new_db_indicator_set(conn)?;
        for f in &functions {
            set_indis.push(DbSetIndicator{
                indicator_set_id: indi_set.indicator_set_id,
                indicator_id: fmap.get(&f).unwrap().choose(&mut rng).unwrap().indicator_id
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
