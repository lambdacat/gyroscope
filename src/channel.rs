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

/// A channel which reads `f32`s.
pub trait In {
    /// Get a mutable reference to some buffer owned by the channel, which has room for `size`
    /// samples.
    fn input<'x>(&'x mut self, count: usize) -> &'x mut [f32];
}

/// A channel which writes `f32`s.
pub trait Out {
    /// Return number of `f32` samples that the next call to `to_f32` will produce.
    fn num_samples(&self) -> usize;

    /// Populate `dst` with data from the channel, return the number of `f32`s written, if `dst` is
    /// big enough this will be the same number as is returned by a call to `self.num_samples()`
    /// beforehand.
    fn output(&self, dst: &mut [f32]) -> usize;
}
