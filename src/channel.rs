// This file is part of Gyroscope.
//
// Gyroscope is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Gyroscope is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Gyroscope.  If not,
// see <http://www.gnu.org/licenses/>.

/// A channel which reads `f32`s.
pub trait In {
    /// Read from this slice of `f32`s.
    fn from_f32(&mut self, src: &[f32]);
}

/// A channel which writes `f32`s.
pub trait Out {
    /// Return number of `f32` samples that the next call to `to_f32` will produce.
    fn num_samples(&self) -> usize;

    /// Populate `dst` with data from the channel, return the number of `f32`s written, if `dst` is
    /// big enough this will be the same number as is returned by a call to `self.num_samples()`
    /// beforehand.
    fn to_f32(&self, dst: &mut [f32]) -> usize;
}
