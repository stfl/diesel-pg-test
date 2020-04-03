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

use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use glob::glob;
use database::db_indicator::*;
use database::db_indicator_set::*;
use database::*;
use params::*;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

fn main() {
    use database::schema::indicators::dsl::*;

    let connection = establish_connection();

    // let num_del = diesel::delete(indicators.filter(name.like("test1")))
    //     .execute(&connection)
    //     .expect("delete failed");
    // println!("del {}", num_del);

    let indi = create_indicator(&connection, "test1");
    println!("{:?}", indi);

    let results = indicators
        // .filter(parent_ranged_id.eq(Some))
        .limit(5)
        .load::<DbIndicator>(&connection)
        .expect("Error loading indicators");

    println!("Displaying {} posts", results.len());
    for indi in results {
        println!("{}", indi.name);
        println!("-----------\n");
    }

    update_indicator(&connection, &indi);
    let _ = store_indicator(
        &connection,
        &Indicator {
            name: "test123".into(),
            shift: 0,
            inputs: vec_vec_to_bigdecimal(vec![vec![0.6], vec![20.0, 40.0, 2.0], vec![40.0]]),
        },
        None,
    );
    let ind = load_indicator(&connection, indi.indicator_id);
    println!("{:?}", ind);

    let indis = load_all_indicator_files().unwrap();
    let db_indis = store_indicators_with_default_func(&connection, &indis).unwrap();
    println!("{:?}", db_indis);

    // let indi_set = store_new_db_indicator_set(&connection);
    // println!("indi set with new id: {:?}", indi_set);
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_indicator<'a>(conn: &PgConnection, name: &'a str) -> db_indicator::DbIndicator {
    use schema::indicators;

    let new_indi = db_indicator::NewDbIndicator {
        parent_id: None,
        indicator: name,
        shift: 0i16,
    };

    diesel::insert_into(indicators::table)
        .values(&new_indi)
        .get_result(conn)
        .expect("Error saving new indicator")
}

pub fn update_indicator(conn: &PgConnection, indi: &db_indicator::DbIndicator) {
    use schema::indicators::dsl::*;

    let ii = diesel::update(indicators.find(indi.indicator_id))
        .set(parent_id.eq(indi.indicator_id - 1))
        .get_result::<db_indicator::DbIndicator>(conn)
        .expect(&format!("Unable to update indicator {}", indi.indicator_id));
    println!("updated indicator {:?}", ii);
}

pub fn load_all_indicator_files() -> std::io::Result<Vec<(DbIndiFunc, Indicator)>> {
    use DbIndiFunc::*;
    let mut indis: Vec<(DbIndiFunc, Indicator)> = vec![];
    for entry in glob("config/indicator/*/*").unwrap().filter_map(Result::ok) {
        // entry.ancestors().next().unwrap().next().unwrap().file_name()
        // TODO debug!()
        println!("loading indicator file: {:?}", entry);
        indis.push((match entry.parent().unwrap().file_name().unwrap().to_str().unwrap() {
            "confirm" => Confirm,
            "baseline" => Baseline,
            "exit" => Exit,
            "volume" => Volume,
            "continue" => Continue,
            e => panic!("unknown func {:?}", e),
        }, serde_any::from_file(entry).unwrap()))
    }
    Ok(indis)
}
