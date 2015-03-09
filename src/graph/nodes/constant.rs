// This file is part of Gyroscope, a program and library for electronic music production.
// Copyright (C) 2015, Sam Payson <scpayson at gmail dot com>
//
// This program is free software: you can redistribute it and/or modify it under the terms of the
// GNU Affero General Public License as published by the Free Software Foundation, either version 3
// of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with this program.
// If not, see <http://www.gnu.org/licenses/>.

use std::cmp;

use channel;
use graph;

pub struct Constant {
    output: Out,
}

impl graph::Node for Constant {
    /// Running a `Constant` is a no-op.
    fn run(&mut self) {}

    /// a `constant` has no inputs.
    fn num_inputs(&self) -> usize { 0 }

    /// this will always return `none`.
    fn get_input<'x>(&'x mut self, _idx: usize) -> Option<&'x mut channel::In> { None }

    /// a `constant` has no inputs.
    fn num_outputs(&self) -> usize { 1 }

    /// this will always return `none`.
    fn get_output<'x>(&'x self, idx: usize) -> Option<&'x channel::Out> {
        match idx {
            0 => Some(&self.output as &channel::Out),
            _ => None,
        }
    }

}

struct Out {
    count: usize,
    val:   f32,
}

impl channel::Out for Out {
    fn num_samples(&self) -> usize { self.count }

    fn output(&self, dst: &mut [f32]) -> usize {
        let upper = cmp::min(self.count, dst.len());

        for fptr in dst[..upper].iter_mut() {
            *fptr = self.val;
        }

        upper
    }
}
