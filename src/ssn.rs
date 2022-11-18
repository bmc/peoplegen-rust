use std::iter::Iterator;

/**
 * This struct can be used to generate Social Security numbers, as an iterator.
 * If you use `SsnGenerator::new()` to create the object, it will prepopulate
 * the fields with appropriate values to generate guaranteed-fake SSNs. These
 * defaults will generate a maximum of 99,980,001 fake Social Security numbers,
 * before the iterator runs out. If you then reset the iterator, it will start
 * again. You can also use the `SsnGenerator::new_auto_reset()`
 * constructor to have the generator auto-reset (and, therefore, cycle)
 * infinitely.
 */
pub struct SsnGenerator {
    prefixes: Vec<u32>,
    prefix_index: usize,
    mid_min: u32,
    mid_cur: u32,
    mid_max: u32,
    last_min: u32,
    last_cur: u32,
    last_max: u32,
    auto_reset: bool,
}

impl SsnGenerator {
    /**
     * Create a new `SsnGenerator` with default values, including a set of
     * Social Security prefixes (the first group of three numbers) that are
     * guaranteed to be fake, per the approach outlined at
     * <https://stackoverflow.com/a/2313726/53495>
     *
     * This generator will _not_ auto-reset when it gets to the end of the
     * sequence. Use `SsnGenerator::new_auto_reset()` to create a generator
     * that will cycle back to the beginning automatically. You can manually
     * reset the generator yourself by calling the `reset()` function.
     */
    pub fn new() -> Self {
        let mut prefixes: Vec<u32> = (900..=999).collect();
        prefixes.push(666);
        Self {
            prefixes,
            prefix_index: 0,
            mid_min: 1,
            mid_cur: 0,
            mid_max: 99,
            last_min: 1,
            last_cur: 0,
            last_max: 9999,
            auto_reset: false,
        }
    }

    /**
     * Create a new `SsnGenerator` with default values, including a set of
     * Social Security prefixes (the first group of three numbers) that are
     * guaranteed to be fake, per the approach outlined at
     * <https://stackoverflow.com/a/2313726/53495>
     *
     * This generator will auto-reset when it gets to the end of the
     * sequence. Use `SsnGenerator::new())` to create a generator will not
     * cycle back to the beginning automatically.
     */
    pub fn new_auto_reset() -> Self {
        let mut s = SsnGenerator::new();
        s.auto_reset = true;
        s
    }

    /**
     * Create a new `SsnGenerator`, with complete control over the fields.
     * This constructor is intended primarily for testing.
     */
    pub fn new_for_tests(prefixes: Vec<u32>,
                         mid_min: u32,
                         mid_max: u32,
                         last_min: u32,
                         last_max: u32,
                         auto_reset: bool) -> Self {
        assert!(mid_min > 0);
        assert!(mid_max <= 99);
        assert!(last_min > 0);
        assert!(last_max <= 9999);

        Self {
            prefixes,
            prefix_index: 0,
            mid_min,
            mid_cur: 0,
            mid_max,
            last_min,
            last_cur: 0,
            last_max,
            auto_reset
        }
    }

    /**
     * Reset the generator to its initial values.
     */
    pub fn reset(&mut self) {
        self.mid_cur = 0;
        self.last_cur = 0;
        self.prefix_index = 0;
    }

    pub fn total(&self) -> u64 {
        let total_firsts: u64 = self.prefixes.len() as u64;
        let total_lasts: u64 = (self.last_max - self.last_min + 1) as u64;
        let total_mids: u64 = (self.mid_max - self.mid_min + 1) as u64;

        total_firsts * total_lasts * total_mids
    }
}

impl Iterator for SsnGenerator {
    type Item = String;

    /**
     * Get the next Social Security number in the sequence. If auto-reset
     * is enabled, and there are no more numbers in the sequence, the generator
     * will start at the beginning again. Otherwise, it will return `None`.
     */
    fn next(&mut self) -> Option<Self::Item> {
        if (self.prefix_index == (self.prefixes.len() - 1)) &&
           (self.mid_cur == self.mid_max) &&
           (self.last_cur == self.last_max) {

            if self.auto_reset {
                self.reset()
            }
            else {
                return None;
            }
        }

        if (self.prefix_index == 0) && (self.mid_cur == 0) && (self.last_cur == 0) {
            self.mid_cur += 1;
            self.last_cur += 1;
        }
        else if (self.mid_cur == self.mid_max) &&
                (self.last_cur == self.last_max) {
            self.prefix_index += 1;
            self.mid_cur = self.mid_min;
            self.last_cur = self.last_min;
        }
        else if self.last_cur == self.last_max {
            self.mid_cur += 1;
            self.last_cur = self.last_min;
        }
        else {
            self.last_cur += 1;
        }

        let first = self.prefixes[self.prefix_index];
        let ssn = format!("{:03}-{:02}-{:04}", first, self.mid_cur, self.last_cur);
        Some(ssn)
    }
}

#[cfg(test)]
mod tests {
    use crate::ssn::SsnGenerator;

    #[test]
    fn gen_several() {
        let mut ssns = SsnGenerator::new();
        let ssn1: Option<String> = ssns.next();
        let ssn2: Option<String> = ssns.next();
        assert_ne!(ssn1, None);
        assert_eq!(ssn1.unwrap(), "900-01-0001");
        assert_ne!(ssn2, None);
        assert_eq!(ssn2.unwrap(), "900-01-0002");
        assert_eq!(ssns.total(), 99_980_001);
    }

    #[test]
    fn gen_two_prefixes() {
        let prefixes: Vec<u32> = (900..=901).collect();
        let mut ssns = SsnGenerator::new_for_tests(
            prefixes, 1, 2, 1, 2, false
        );

        assert_eq!(ssns.total(), 8);

        let ssn1 = ssns.next();
        let ssn2 = ssns.next();
        let ssn3 = ssns.next();
        let ssn4 = ssns.next();
        let ssn5 = ssns.next();
        let ssn6 = ssns.next();
        let ssn7 = ssns.next();
        let ssn8 = ssns.next();
        let ssn9 = ssns.next();

        assert_eq!(ssn1, Some(String::from("900-01-0001")));
        assert_eq!(ssn2, Some(String::from("900-01-0002")));
        assert_eq!(ssn3, Some(String::from("900-02-0001")));
        assert_eq!(ssn4, Some(String::from("900-02-0002")));
        assert_eq!(ssn5, Some(String::from("901-01-0001")));
        assert_eq!(ssn6, Some(String::from("901-01-0002")));
        assert_eq!(ssn7, Some(String::from("901-02-0001")));
        assert_eq!(ssn8, Some(String::from("901-02-0002")));
        assert_eq!(ssn9, None);

        ssns.reset();
        let ssn10 = ssns.next();
        assert_eq!(ssn10, Some(String::from("900-01-0001")));
    }

    #[test]
    fn test_reset() {
        let mut prefixes: Vec<u32> = Vec::new();
        prefixes.push(900);
        let mut ssns = SsnGenerator::new_for_tests(
            prefixes, 1, 2, 1, 2, false
        );
        let ssn1 = ssns.next();
        let ssn2 = ssns.next();
        let ssn3 = ssns.next();
        let ssn4 = ssns.next();

        assert_eq!(ssn1, Some(String::from("900-01-0001")));
        assert_eq!(ssn2, Some(String::from("900-01-0002")));
        assert_eq!(ssn3, Some(String::from("900-02-0001")));
        assert_eq!(ssn4, Some(String::from("900-02-0002")));

        let ssn5 = ssns.next();
        assert_eq!(ssn5, None);

        ssns.reset();
        let ssn6 = ssns.next();
        assert_eq!(ssn6, Some(String::from("900-01-0001")));
    }

    #[test]
    fn test_auto_reset() {
        let mut prefixes: Vec<u32> = Vec::new();
        prefixes.push(900);
        let mut ssns = SsnGenerator::new_for_tests(
            prefixes, 1, 2, 1, 2, true
        );
        let ssn1 = ssns.next();
        let ssn2 = ssns.next();
        let ssn3 = ssns.next();
        let ssn4 = ssns.next();

        assert_eq!(ssn1, Some(String::from("900-01-0001")));
        assert_eq!(ssn2, Some(String::from("900-01-0002")));
        assert_eq!(ssn3, Some(String::from("900-02-0001")));
        assert_eq!(ssn4, Some(String::from("900-02-0002")));

        let ssn5 = ssns.next();
        assert_eq!(ssn5, Some(String::from("900-01-0001")));
    }
}
