pub mod db_indicator;
pub mod db_indicator_set;
pub mod schema;

use super::params::{Indicator, IndicatorSet};
use db_indicator::*;
use db_indicator_set::*;
use diesel::prelude::*;
