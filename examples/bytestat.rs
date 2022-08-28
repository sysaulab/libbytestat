//! # Bytestat
//!
//! Bytestat measure randomness of data. 
//! Data is readed from stdin.
//! The final score is between 0 and 100. 
//! Good quality random data should score 100 when rounded.
//! If the sample size if too small to be significant, the "~"" symbol is added as a prefix.
//! Example: ~68% is a bad score, but there is not enough data for the method to be precise.
//! 

use std::{io::Read};
use libbytestat::Bytestat;

fn main() {

  let mut stats = Bytestat::new();
  let mut counter:u128 = 0;
  let percent = 256 * 4096;

  for x in std::io::stdin().bytes() {
    match x {
        Ok(data) => {
          stats.analyze(data);
          counter += 1;
        },
        Err(err) => {
          eprintln!("{:?}", err);
        }
    }
  }

  println!("\nRAW SCORES AS STRING");
  println!("{}", stats.get_scores_string("\n"));

  println!("\nFINAL SCORE");
  println!("{} samples", counter );
  println!("{}{:.0}%", if counter < (percent * 100) {"~"} else {""}, stats.get_score() );

}