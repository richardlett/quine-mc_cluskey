#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Bool {
    True,
    False,
    Term(u8),
    And(Vec<Bool>),
    Or(Vec<Bool>),
    Not(Box<Bool>),
}

impl Bool {
    fn terms(&self) -> u32 {
        use self::Bool::*;
        match *self {
            Term(u) => 1 << u,
            Or(ref a) |
            And(ref a) => a.iter().fold(0, |state, item| { state | item.terms() }),
            Not(ref a) => a.terms(),
            True | False => 0,
        }
    }

    fn eval(&self, terms: u32) -> bool {
        use self::Bool::*;
        match *self {
            True => true,
            False => false,
            Term(i) => (terms & (1 << i)) != 0,
            And(ref a) => a.iter().all(|item| item.eval(terms)),
            Or(ref a) => a.iter().any(|item| item.eval(terms)),
            Not(ref a) => !a.eval(terms),
        }
    }

    pub fn minterms(&self) -> Vec<Term> {
        let terms = self.terms();
        let nterms = terms.count_ones();
        assert!((0..nterms).all(|i| (terms & (1 << i)) != 0), "non-continuous naming scheme");
        (0..(1 << nterms)).filter(|&i| self.eval(i)).map(Term::new).collect()
    }
}

#[derive(Debug)]
pub struct Essentials {
    pub minterms: Vec<Term>,
    pub essentials: Vec<Term>,
}

#[derive(Clone, Eq, Ord)]
pub struct Term {
    dontcare: u32,
    term: u32,
}

impl std::cmp::PartialOrd for Term {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match self.dontcare.partial_cmp(&rhs.dontcare) {
            Some(Equal) => {},
            other => return other,
        }
        let l = self.term & !self.dontcare;
        let r = rhs.term & !rhs.dontcare;
        l.partial_cmp(&r)
    }
}

impl std::fmt::Debug for Term {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in (0..32).rev() {
            if (self.dontcare & (1 << i)) != 0 {
                try!(write!(fmt, "-"));
            } else if (self.term & (1 << i)) != 0 {
                try!(write!(fmt, "1"));
            } else {
                try!(write!(fmt, "0"));
            }
        }
        Ok(())
    }
}

impl std::cmp::PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        (self.dontcare == other.dontcare) && ((self.term & !self.dontcare) == (other.term & !other.dontcare))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TermFromStrError {
    Only32TermsSupported,
    UnsupportedCharacter(char),
}

impl std::str::FromStr for Term {
    type Err = TermFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 32 {
            return Err(TermFromStrError::Only32TermsSupported);
        }
        let mut term = Term::new(0);
        for (i, c) in s.chars().rev().enumerate() {
            match c {
                '-' => term.dontcare |= 1 << i,
                '1' => term.term |= 1 << i,
                '0' => {},
                c => return Err(TermFromStrError::UnsupportedCharacter(c)),
            }
        }
        Ok(term)
    }
}

impl Term {
    pub fn new(i: u32) -> Self {
        Term {
            dontcare: 0,
            term: i,
        }
    }

    pub fn with_dontcare(term: u32, dontcare: u32) -> Self {
        Term {
            dontcare: dontcare,
            term: term,
        }
    }

    pub fn combine(&self, other: &Term) -> Option<Term> {
        let dc = self.dontcare ^ other.dontcare;
        let term = self.term ^ other.term;
        let dc_mask = self.dontcare | other.dontcare;
        match (dc.count_ones(), (!dc_mask & term).count_ones()) {
            (0, 1) |
            (1, 0) => Some(Term {
                dontcare: dc_mask | term,
                term: self.term,
            }),
            _ => None,
        }
    }
}

pub fn essential_minterms(mut minterms: Vec<Term>) -> Essentials {
    minterms.sort();
    let minterms = minterms;
    let mut terms = minterms.clone();
    let mut essentials: Vec<Term> = Vec::new();
    while !terms.is_empty() {
        println!("{:#?}", essentials);
        println!("{:#?}", terms);
        let old = std::mem::replace(&mut terms, Vec::new());
        let mut combined_terms = std::collections::BTreeSet::new();
        for (i, term) in old.iter().enumerate() {
            for (other_i, other) in old[i..].iter().enumerate() {
                if let Some(new_term) = term.combine(other) {
                    println!("combined {} and {}", i, other_i + i);
                    terms.push(new_term);
                    combined_terms.insert(other_i + i);
                    combined_terms.insert(i);
                }
            }
            if !combined_terms.contains(&i) {
                println!("{} is essential", i);
                essentials.push(term.clone());
            }
        }
        terms.dedup();
    }
    Essentials {
        minterms: minterms,
        essentials: essentials,
    }
}
