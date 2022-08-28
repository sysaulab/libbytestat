//! # Bytestat
//!
//! Bytestat is a crate to measure randomness of data. 
//! Data is processed one byte at a time, sequentially.
//! The distribution and interval of each byte is measured. 
//! Five metrics are used to measure different aspects of the set. 
//! The final score is between 0 and 100 as f64. 
//! Good quality random data should score 100 when rounded up.
pub struct Bytestat {
    counter:u128,
    dist:[u128;256],
    interval:[u128;256*256],
    last:[u128;256],
    score_counter:u128,
    score_non_zero:f64,
    score_unique:f64,
    score_amplitude:f64,
    score_interval_continuity:f64,
    score_interval_amplitude:f64,
    score:f64,
}

impl Bytestat {
  /// Create new Bytestat object.
  ///
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// ```
  pub fn new() -> Bytestat {
    Bytestat {
      counter:0,
      dist:[0;256],
      interval:[0;256*256],
      last:[0;256],
      score_counter:0,
      score_non_zero:0.0,
      score_unique:0.0,
      score_amplitude:0.0,
      score_interval_continuity:0.0,
      score_interval_amplitude:0.0,
      score:0.0,
      }
  }

  /// Analyze one byte, bytes must be analysed in sequence.
  /// If bytes are not analyzed in sequence, the final score will not be valid.
  /// Repeat as needed.
  ///
  /// # Arguments
  ///
  /// * `value` - A byte to be analyzed, u8
  /// 
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// 
  /// for x in 0..limit {
  ///   let my_byte = get_random_byte();
  ///   stats.analyze( my_byte );
  /// }
  /// ```
  pub fn analyze(&mut self, value:u8) {
      self.counter += 1;
      self.dist[value as usize] += 1;
      self.interval[ ((self.counter - self.last[value as usize]) as u16) as usize ] += 1;
      self.last[value as usize] = self.counter;
    }

  fn update_scores(&mut self) {
    if self.score_counter == self.counter {
      return
    }

    //1 of 5
    let mut dist_not_zero = 0;
    for x in self.dist {
      if x > 0 {
        dist_not_zero += 1;
      }
    }
    self.score_non_zero = dist_not_zero as f64 / 256 as f64;

    //2 of 5
    let mut dist_unique = 0;
    let mut dist_unique_map:std::collections::HashMap<u128, i32> = std::collections::HashMap::new();
    for x in 0..256 {
      match dist_unique_map.get(&self.dist[x]) {
        Some(value) => dist_unique_map.insert(self.dist[x], 1+value),
        None => dist_unique_map.insert(self.dist[x], 1)
      };
    }
    dist_unique_map.values().for_each(|x| {
      if *x == 1 {
        dist_unique += 1;
      }
    });
    self.score_unique = dist_unique as f64 / 256 as f64;

    //3 of 5
    let mut dist_amp_min:u128 = std::u128::MAX;
    let mut dist_amp_max:u128 = std::u128::MIN;
    for x in self.dist {
      if x < dist_amp_min {
        dist_amp_min = x;
      }
      if x > dist_amp_max {
        dist_amp_max = x;
      }
    }
    let dist_amp_variation = dist_amp_max - dist_amp_min;
    self.score_amplitude = (dist_amp_max - dist_amp_variation) as f64 / dist_amp_max as f64;

    //4 of 5
    let mut interval_min = std::u16::MAX;
    let mut interval_max = std::u16::MIN;

    for x in 1..self.interval.len() {
      if self.interval[x] > self.counter / 4096 {
        if (x as u16) < interval_min {
          interval_min = x as u16;
        }
        if (x as u16) > interval_max {
          interval_max = x as u16;
        }
      }
    }

    let mut populated = 1;
    for x in 1..interval_max {
      if self.interval[x as usize] > self.counter / 4096 {
        populated += 1;
      }
    }
    self.score_interval_continuity = (if populated < 512 { populated } else { 512 }) as f64 / 512 as f64;

    //5 of 5
    if interval_max > 512 {
      interval_max = 512;
    }
    self.score_interval_amplitude = interval_max as f64 / 512 as f64;

    //FINAL SCORE
    self.score = self.score_non_zero * 20f64;
    self.score += self.score_unique * 20f64;
    self.score += self.score_amplitude * 20f64;
    self.score += self.score_interval_continuity * 20f64;
    self.score += self.score_interval_amplitude * 20f64;


    self.score_counter = self.counter;
  }

  /// Generate the score based on distribution of unique bytes being present in the set.
  /// 
  /// (unique byte present in set) / (maximum number of possible unique bytes, 256)
  ///
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// 
  /// for x in 0..limit {
  ///   let my_byte = get_random_byte();
  ///   stats.analyze( my_byte );
  /// }
  /// 
  /// stats.get_score_non_zero()
  /// ```
  pub fn get_score_non_zero(&mut self) -> f64 {
    self.update_scores();
    self.score_non_zero
  }

  /// Generate the score based on the uniqueness of the bytes distribution in the set.
  /// The score is between 0.0 and 1.0. Any score lower than 0.99 should be considered problematic.
  /// 
  /// (unique byte count in set) / (maximum number of possible unique bytes, 256)
  ///
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// 
  /// for x in 0..limit {
  ///   let my_byte = get_random_byte();
  ///   stats.analyze( my_byte );
  /// }
  /// 
  /// stats.get_score_unique()
  /// ```
  pub fn get_score_unique(&mut self) -> f64 {
    self.update_scores();
    self.score_unique
  }

  /// Generate the score based on the amplitude of the bytes distribution in the set.
  /// The score is between 0.0 and 1.0. 
  /// Any score lower than 0.99 should be considered problematic.
  /// 
  /// ((bytes count max) - (bytes count min)) / (bytes count max)
  ///
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// 
  /// for x in 0..limit {
  ///   let my_byte = get_random_byte();
  ///   stats.analyze( my_byte );
  /// }
  /// 
  /// stats.get_score_amplitude()
  /// ```
  pub fn get_score_amplitude(&mut self) -> f64 {
    self.update_scores();
    self.score_amplitude
  }

  /// Generate the sub score based on the amplitude of the continuity of significant interval measurements.
  /// The score is between 0 and 1. 
  /// Any score lower than 0.99 should be considered problematic.
  /// 
  /// ( 1 ... interval_largest [] ) / (interval_largest)
  ///
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// 
  /// for x in 0..limit {
  ///   let my_byte = get_random_byte();
  ///   stats.analyze( my_byte );
  /// }
  /// 
  /// stats.get_score_amplitude()
  /// ```
  pub fn get_score_interval_continuity(&mut self) -> f64 {
    self.update_scores();
    self.score_interval_continuity
  }

  /// Generate the score based on the amplitude of significant interval measurements relative to twice the range of byte.
  /// The score is between 0.0 and 1.0. Any score lower than 1.0 should be considered problematic.
  /// 
  /// ( interval_largest ) / 512
  ///
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// 
  /// for x in 0..limit {
  ///   let my_byte = get_random_byte();
  ///   stats.analyze( my_byte );
  /// }
  /// 
  /// stats.get_score_interval_amplitude()
  /// ```
  pub fn get_score_interval_amplitude(&mut self) -> f64 {
    self.update_scores();
    self.score_interval_amplitude
  }

  /// Generate the final score based on the 5 individual tests. 
  /// Score between 0 and 100. 99 or lower is very problematic.
  ///
  /// # Examples
  ///
  /// ```
  /// use bytestat::Bytestat;
  /// let stats = Bytestat::new();
  /// 
  /// for x in 0..limit {
  ///   let my_byte = get_random_byte();
  ///   stats.analyze( my_byte );
  /// }
  /// 
  /// stats.get_score()
  /// ```
  pub fn get_score(&mut self) -> f64 {
    self.update_scores();
    self.score
  }

  pub fn get_scores_array(&mut self) -> [f64;6] {
    [
      self.get_score_non_zero(),
      self.get_score_unique(),
      self.get_score_amplitude(),
      self.get_score_interval_continuity(),
      self.get_score_interval_amplitude(),
      self.get_score()
    ]
  }

  pub fn get_scores_string(&mut self, seperator:&str) -> String {
    let mut answer = String::from("");

    answer.push_str( self.get_score_non_zero().to_string().as_str() );
    answer.push_str( seperator );

    answer.push_str( self.get_score_unique().to_string().as_str() );
    answer.push_str( seperator );

    answer.push_str( self.get_score_amplitude().to_string().as_str() );
    answer.push_str( seperator );

    answer.push_str( self.get_score_interval_continuity().to_string().as_str() );
    answer.push_str( seperator );

    answer.push_str( self.get_score_interval_amplitude().to_string().as_str() );
    answer.push_str( seperator );

    answer.push_str( self.get_score().to_string().as_str() );

    answer
  }
}
