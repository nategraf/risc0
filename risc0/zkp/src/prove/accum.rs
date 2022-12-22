// Copyright 2022 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{collections::BTreeMap, sync::Mutex};

use anyhow::Result;

use crate::{
    adapter::CircuitStepHandler,
    field::{Elem, ExtElem, Field},
};

/// Tracks grand product accumulations for PLOOKUP.
pub struct Accum<E: Elem> {
    /// Total number of cycles in this run.
    cycles: usize,

    // PLOOKUP relies on two grand product accumulation checks;
    // one for the memory permutation and a second for a lookup table.
    // We have two `kinds`: memory and bytes.
    // TODO check with Jeremy if this is accurate. (what are "kinds?")
    kinds: BTreeMap<String, Vec<E>>,
}

impl<E: Elem> Accum<E> {
    // Constructs a new Accum
    pub fn new(cycles: usize) -> Self {
        Accum {
            cycles,
            kinds: BTreeMap::new(),
        }
    }

    // Generates "pre-" side of grand product accumulation
    // TODO check with Jeremy if this is accurate. (what is "prefix?")
    pub fn calc_prefix_products(&mut self) {
        for (_kind, elems) in self.kinds.iter_mut() {
            let mut tot = E::ONE;
            for elem in elems.iter_mut() {
                tot = tot * *elem;
                *elem = tot;
            }
        }
    }

    fn get_ptr(&mut self, kind: String) -> *mut E {
        self.kinds
            .entry(kind)
            .or_insert_with(|| vec![E::ONE; self.cycles])
            .as_mut_ptr()
    }
}

pub struct Handler<'a, F: Field> {
    // TODO what is p?
    p: &'a Mutex<Accum<F::ExtElem>>,
    cycles: usize,
    kinds: BTreeMap<String, *mut F::ExtElem>,
}

impl<'a, F: Field> Handler<'a, F> {
    pub fn new(p: &'a Mutex<Accum<F::ExtElem>>) -> Self {
        let cycles = p.lock().unwrap().cycles;
        Handler {
            p,
            kinds: BTreeMap::new(),
            cycles,
        }
    }

    fn get_ptr(&mut self, kind: &str) -> *mut F::ExtElem {
        if let Some(entry) = self.kinds.get_mut(kind) {
            *entry
        } else {
            let mut p = self.p.lock().unwrap();
            let ptr = p.get_ptr(kind.to_string());
            self.kinds.insert(kind.to_string(), ptr);
            ptr
        }
    }
}

impl<'a, F: Field> CircuitStepHandler<F::Elem> for Handler<'a, F> {
    fn call(
        &mut self,
        cycle: usize,
        // TODO what is name?
        name: &str,
        // TODO what is extra?
        extra: &str,
        args: &[F::Elem],
        outs: &mut [F::Elem],
    ) -> Result<()> {
        assert!(cycle < self.cycles);
        match name {
            "plonkWriteAccum" => {
                assert_eq!(args.len(), F::ExtElem::EXT_SIZE);
                let elem = F::ExtElem::from_subelems(args.iter().copied());
                let ptr = self.get_ptr(extra);
                // Already checked that our cycle number is in range, so this offset is in the
                // buffer.
                unsafe { ptr.add(cycle).write(elem) };
            }
            "plonkReadAccum" => {
                assert_eq!(outs.len(), F::ExtElem::EXT_SIZE);
                let ptr = self.get_ptr(extra);
                // Already checked that our cycle number is in range, so this offset is in the
                // buffer.
                let elem = unsafe { ptr.add(cycle).read() };
                outs.clone_from_slice(elem.subelems());
            }
            _ => panic!("Unknown accum operation {name}"),
        }
        Ok(())
    }

    fn sort(&mut self, _name: &str) {
        unimplemented!()
    }

    fn calc_prefix_products(&mut self) {
        unimplemented!()
    }
}
