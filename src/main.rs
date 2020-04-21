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

use database::indicator::*;
use database::indicator_set::*;
use database::*;
use glob::glob;
// use params;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::io::{self, Write};

extern crate rand;
extern crate rand_distr;
use rand::prelude::*;
use std::collections::HashMap;
use std::ops::Add;
use indicator_inputs_explicit::NewIndicatorInputsExplicit;
use result::RunResult;
use result_set::ResultSet;

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
    //     .load::<Indicator>(&connection)
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

    // HACK comment in to load indicators from file
    let indis = load_all_indicators_from_file(&connection).unwrap();

    let res = generate_test_results(&connection).unwrap();
    // println!("{:?}", res);
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

// pub fn create_indicator<'a>(conn: &PgConnection, name: &'a str) -> db_indicator::Indicator {
//     use schema::indicators;

//     let new_indi = db_indicator::NewIndicator {
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

pub fn update_indicator(conn: &PgConnection, indi: &indicator::Indicator) {
    use schema::indicators::dsl::*;

    let ii = diesel::update(indicators.find(indi.indicator_id))
        .set(parent_id.eq(indi.indicator_id - 1))
        .get_result::<indicator::Indicator>(conn)
        .expect(&format!("Unable to update indicator {}", indi.indicator_id));
    println!("updated indicator {:?}", ii);
}

// TODO this is a really ugly implementation
pub fn load_all_indicators_from_file<'a>(conn: &PgConnection) -> QueryResult<Vec<Indicator>> {
    use database::schema::indicators;
    use IndiFunc::*;

    let mut indis: Vec<Indicator> = vec![];
    for entry in glob("config/indicator/*/*").unwrap().filter_map(Result::ok) {
        // println!("loading-indicator-file: {:?}", entry);
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
        let indi: params::Indicator = serde_any::from_file(entry).unwrap();
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
    functions: Vec<IndiFunc>,
) -> QueryResult<Vec<IndicatorSet>> {
    use database::schema::indicator_sets;
    use database::schema::indicators::dsl::*;
    use rand::seq::SliceRandom;

    let mut fmap = HashMap::<IndiFunc, Vec<Indicator>>::new();
    for f in &functions {
        // TODO load all at once and group locally instead of loadin individually
        // FIXME load filter parent_id.is_null() or child_id.is_null()
        fmap.insert(*f, indicators.filter(func.eq(f)).load::<Indicator>(conn)?);
    }

    let mut rng = rand::thread_rng();
    let mut set_indis = Vec::<SetIndicator>::new();
    let mut indi_sets = Vec::<IndicatorSet>::new();
    for i in 0..size {
        let indi_set = store_new_db_indicator_set(conn)?;
        for f in &functions {
            set_indis.push(SetIndicator {
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
// ) -> QueryResult<Vec<IndicatorSet>> {
//     use database::schema::indicators::dsl::*;
//     use database::schema::indicator_sets;
//     use rand::seq::SliceRandom;

//     // let fmap = HashMap::<DbIndiFunc, Indicator>;
//     for f in functions {
//         let indis = indicators
//             .filter(func.eq(f))
//             .load::<Indicator>(conn)?;

//         // TODO I need this to be "mit zuruecklegen"
//         // let selection: Vec<&Indicator> = indis.choose_multiple(&mut rand::thread_rng(), size)
//                                                 // .cloned().collect();
//     }

//     unimplemented!()
// }

fn generate_test_results(conn: &PgConnection) -> QueryResult<()> {
    use bigdecimal::BigDecimal;
    use chrono::{NaiveDate, NaiveDateTime, Utc};
    use database::indicator::IndiFunc::*;
    use database::result::*;
    use database::run::*;
    use database::run_session::*;
    use database::indicator_inputs_explicit::*;
    use database::schema::*;
    use database::symbols::*;
    use diesel::insert_into;
    // use rand::seq::SliceRandom;
    use rand::prelude::*;
    use rand_distr::{Distribution, Normal};

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
    let mut pop = generate_population(conn, 30, vec![Confirm, Confirm2, Baseline, Volume, Exit])?;
    // let indi_set_ranged = pop.pop().unwrap();

    for indi_set_ranged in pop {
        // create run
        let run = insert_into(runs::table)
            .values(NewRun {
                session_id: run_session.id().to_owned(),
                run_date: Utc::now().naive_utc(),
                indicator_set_id: indi_set_ranged.id().to_owned(),
            })
            .get_result::<Run>(conn)?;

        let set_indis =
            SetIndicator::belonging_to(&indi_set_ranged).get_results::<SetIndicator>(conn)?;

        // let mut indi_set_map = HashMap::<DbIndiFunc, (Indicator, Vec<IndicatorInput>)>::new();
        let mut indi_set_with_inputs = Vec::<(Indicator, Vec<IndicatorInput>)>::new();
        for set_indi in set_indis {
            let indi_with_inputs = load_indicator(conn, set_indi.indicator_id)?;
            println!("{:?}", params::Indicator::from(indi_with_inputs.clone()));
            indi_set_with_inputs.push(indi_with_inputs);
        }

        // TODO remove
        // let mut new_indi_sets = Vec::<IndicatorSet>::new(); // init with_capactity to elimitate growing contantly
                                                              // spread out the ranged inputs get a Vec of singles
        let mut inputs_spread = HashMap::<IndiFunc, HashMap<i16, Vec<BigDecimal>>>::new();
        for (indi, inputs) in &indi_set_with_inputs {
            let mut input_map: HashMap<i16, Vec<BigDecimal>> = inputs
                .iter()
                .filter(|i| i.start.is_none() && i.input.is_some()) // take all inputs that are not ranged
                .map(|i| (i.index, vec![i.input.clone().unwrap()])) // create a hashmap out of it with the key per index
                .collect();

            // TODO if we want all inputs -> don't filter here
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

        let mut rng = rand::thread_rng();
        let num_indi_sets = 500;
        let normal = Normal::new(0.5, 0.2).unwrap(); // mean 0.5 sig**2 0.3
                                                      // create results for (some) of the indicators sets within this indicator_set
        let mut res = Vec::<NewRunResult>::with_capacity(num_indi_sets);
        for n in (0..num_indi_sets) {
            let s = normal.sample(&mut rng);
            res.push(
                NewRunResult {
                    run_id: run.id().to_owned(),
                    result: s,
                    profit: rng.gen_range(0.0, 200000.0),
                    trades: rng.gen_range(0, 2000),
                    expected_payoff: 0.,
                    profit_factor: 0.,
                    recovery_factor: 0.,
                    sharpe_ratio: 0.,
                    custom_result: s,
                    equity_drawdown: 0.,
                });
        }

        let mut db_results = insert_into(results::table).values(&res).get_results::<RunResult>(conn)?;
        // println!("res: {}", db_results.len());

        // create the new indi_sets from the spread-out inputs
        for n in (0..num_indi_sets) {
            // print!("\r{}/{}", n+1, num_indi_sets);
            // io::stdout().flush().unwrap();

            let mut new_inputs = indi_set_with_inputs.iter().map(
                |(indi, _)| {
                let inputs = inputs_spread.get(&indi.func).unwrap();
                NewIndicatorInputsExplicit {
                    indicator_id: indi.id().to_owned(),
                    input0: inputs.get(&0).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input1: inputs.get(&1).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input2: inputs.get(&2).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input3: inputs.get(&3).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input4: inputs.get(&4).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input5: inputs.get(&5).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input6: inputs.get(&6).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input7: inputs.get(&7).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input8: inputs.get(&8).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input9: inputs.get(&9).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input10: inputs.get(&10).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input11: inputs.get(&11).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input12: inputs.get(&12).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input13: inputs.get(&13).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                    input14: inputs.get(&14).map(|v| v.choose(&mut rng).unwrap().to_owned()),
                }}
            ).collect::<Vec<NewIndicatorInputsExplicit>>();

            let db_inputs = insert_into(indicator_inputs_explicit::table)
                .values(new_inputs).get_results::<IndicatorInputsExplicit>(conn)?;

            let next_res = db_results.pop().unwrap();
            let result_sets = db_inputs.iter()
                    .map(|i|
                         ResultSet {
                             result_id: next_res.id().to_owned(),
                             inputs_id: i.id().to_owned()
                         })
                    .collect::<Vec<ResultSet>>();
            let cnt = insert_into(result_sets::table).values(&result_sets).execute(conn)?;
            // println!("result-set: {:?}", result_sets);
        }

        // println!("inputs-explicit: {:?}", db_inputs);

        // let result_sets = db_results.iter().zip(db_inputs.iter()).map(|(r, i)| ResultSet {
        //     result_id: r.id().to_owned(), inputs_id: i.id().to_owned()}).collect::<Vec<ResultSet>>();
        // let cnt = insert_into(result_sets::table).values(&result_sets).execute(conn)?;
        // println!("inserted: {}\n{:?}", cnt, result_sets);


        // let mut new_inputs = Vec::<IndicatorInput>::new();
        // for n in (0..num_indi_sets) {
        //     print!("\r{}/{}", n+1, num_indi_sets);
        //     io::stdout().flush().unwrap();

        //     let new_indis_for_set = indi_set_with_inputs
        //         .iter()
        //         .map(|(indi, _)| indi.new_child_no_ref(conn).unwrap())
        //         .collect::<Vec<Indicator>>();

        //     let new_indi_set = store_new_indicator_set(conn, &new_indis_for_set)?;
        //     // these two transactions new_child() and store_new_indicator_set take a long time.
        //     // this might be helpfull to aggregate to do the transaction all at once

        //     // rand chose for every index according to func
        //     for new_indi in new_indis_for_set {
        //         // println!("new_indi {:?}", indicators::)
        //         new_inputs.extend(inputs_spread.get(&new_indi.func).unwrap().iter().map(
        //             |(idx, inputs)| {
        //                 IndicatorInput {
        //                     indicator_id: new_indi.id().to_owned(),
        //                     index: idx.to_owned(),
        //                     input: Some(inputs.choose(&mut rng).unwrap().to_owned()), // if choose return None we have to deal with the error
        //                     start: None,
        //                     stop: None,
        //                     step: None,
        //                 }
        //             },
        //         ));
        //     }
        //     // println!(
        //     //     "{:#?}\n\n{:#?}",
        //     //     SetIndicator::belonging_to(&new_indi_set).get_results::<SetIndicator>(conn)?,
        //     //     new_inputs
        //     // );
        //     new_indi_sets.push(new_indi_set);
        // }

        // insert_into(indicator_inputs::table)
        //     .values(&new_inputs)
        //     .execute(conn)?;
        // println!("inserted {} inputs", new_inputs.len());

        // let res = new_indi_sets
        //     .iter()
        //     .map(|i| NewRunResult {
        //         run_id: run.id().to_owned(),
        //         result: normal.sample(&mut rng),
        //         profit: rng.gen_range(0.0, 200000.0),
        //         trades: rng.gen_range(0, 2000),
        //         expected_payoff: 0.,
        //         profit_factor: 0.,
        //         recovery_factor: 0.,
        //         sharpe_ratio: 0.,
        //         custom_result: 0.,
        //         equity_drawdown: 0.,
        //     })
        //     .collect::<Vec<NewRunResult>>();

        // let res_len = insert_into(results::table).values(&res).execute(conn)?;
        // println!("inserted {} results", res_len);
    }

    Ok(())
}

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
