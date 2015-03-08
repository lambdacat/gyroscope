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

use std::iter;
use std::mem;

use channel;

/// A synth node.
pub trait Node {
    /// Prepare a new set of outputs from the last set of inputs provided.
    fn run(&mut self);

    /// Get input channels.
    fn inputs<'x>(&'x mut self) -> &[&'x mut channel::In];

    /// Get output channels.
    fn outputs<'x>(&'x self) -> &[&'x channel::Out];
}

/// A descriptor used to refer to a node within a particular graph.
pub type NodeID = usize;

/// A descriptor used to refer to an output of a particular node.
pub type OutputID = usize;

/// A descriptor used to refer to an input of a particular node.
pub type InputID  = usize;

// A NodeWrapper holds onto a node and the mapping from the outputs of other nodes to its inputs.
struct NodeWrapper<'x> {
    // The underlying node.
    node: Box<Node + 'x>,

    // A vector indexed by InputID. If an element is None it means that no input has been assigned
    // yet. A Graph can't be evaluated before all of its Nodes have had their inputs fully assigned.
    //
    // Unused outputs are fine, but unused inputs are not.
    inputs: Vec<Option<(NodeID, OutputID)>>,
}

/// A complete audio pipeline.
pub struct Graph<'x> {
    nodes: Vec<NodeWrapper<'x>>,
}

impl<'x> Graph<'x> {
    /// Add a node to the `Graph` and return an id to refer to it by.
    pub fn add_node<N: Node + 'x>(&mut self, mut n: N) -> NodeID {
        let inputs = iter::repeat(None)
            .take(n.inputs().len())
            .collect() ;

        let id = self.nodes.len();

        self.nodes.push( NodeWrapper {
            node:   box n,
            inputs: inputs,
        });

        id
    }

    /// Patch the output of channel `o_node` of node `o_node` into the input channel `i_chan` of
    /// `i_node`.
    ///
    /// # Errors
    ///
    /// `Error::NoSuchNode` indicates that either `o_node` or `i_node` refers to a node which
    ///  doesn't exist.
    ///
    /// `Error::NoSuchInput` indicates that `i_node` exists, but `i_chan` is not a valid input
    ///  channel of that node.
    ///
    /// `Error::InputAlreadyPatched` is returned if patch has already been called with `i_node`
    ///  and `i_chan` as the input arguments. If this error is returned, the patch will still go
    ///  through, replacing the old assignment.
    ///
    /// `Error::NoSuchOutput` indicates that `o_node` exists, but `o_chan` is not a valid output
    ///  channel of that node.
    pub fn patch(&mut self, o_node: NodeID, o_chan: OutputID, i_node: NodeID, i_chan: InputID)
        -> Result<()> {

        use self::Error::*;

        match self.nodes.get(o_node) {
            Some(nw) => if nw.node.outputs().len() <= o_node { return Err(NoSuchOutput) },
            None     => return Err(NoSuchNode),
        }

        let nw = match self.nodes.get_mut(i_node) {
            Some(nw) => nw,
            None     => return Err(NoSuchNode),
        };

        let old = match nw.inputs.get_mut(i_chan) {
            Some(field) => mem::replace(field, Some((o_node, o_chan))),
            None        => return Err(NoSuchInput),
        };

        match old {
            Some(..) => Err(InputAlreadyPatched),
            None     => Ok(()),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// An error manipulating a `Graph`.
#[derive(Debug)]
pub enum Error {
    /// The given `NodeID` doesn't refer to any node in this `Graph`.
    NoSuchNode,


    /// The given `InputID` doesn't refer to any input channel in the corresponding `Node`.
    NoSuchInput,

    /// The given input already has an output plugged into it.
    InputAlreadyPatched,

    /// The given `OutputID` doesn't refer to any output channel in the corresponding `Node`.
    NoSuchOutput,
}
