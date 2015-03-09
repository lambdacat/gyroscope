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

use std::collections::BitSet;
use std::iter;
use std::mem;

use channel;

/// Module where implementations of nodes live.
pub mod nodes;

/// A synth node.
pub trait Node {
    /// Prepare a new set of outputs from the last set of inputs provided.
    fn run(&mut self);

    fn num_inputs(&self) -> usize;

    /// Get input channels.
    fn get_input<'x>(&'x mut self, idx: usize) -> Option<&'x mut channel::In>;

    fn num_outputs(&self) -> usize;

    /// Get output channels.
    fn get_output<'x>(&'x self, idx: usize) -> Option<&'x channel::Out>;
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

/// A complete audio pipeline. This is a directed multi-graph, and is required to be acyclic.
pub struct Graph<'x> {
    // The nodes of the graph.
    nodes: Vec<NodeWrapper<'x>>,

    // A list of NodeIDs, in dependency order.
    order: Vec<NodeID>,

    // If `dirty` is true, then `order` is inconsistent and needs to be recomputed.
    dirty: bool,
}

impl<'x> Graph<'x> {

    /// Create a `Graph` without any `Node`s.
    pub fn new() -> Graph<'x> {
        Graph {
            nodes: vec![],
            order: vec![],
            dirty: false,
        }
    }

    /// Add a node to the `Graph` and return an id to refer to it by.
    pub fn add_node<N: Node + 'x>(&mut self, n: N) -> NodeID {
        let inputs = iter::repeat(None)
            .take(n.num_inputs())
            .collect() ;

        let id = self.nodes.len();

        self.nodes.push( NodeWrapper {
            node:   box n,
            inputs: inputs,
        });

        // The order is no longer valid, since we've added a new node.
        self.dirty = true;

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
            Some(nw) => if nw.node.num_outputs() <= o_chan {
                return Err(NoSuchOutput(o_node, o_chan))
            },
            None     => return Err(NoSuchNode(o_node)),
        }

        let nw = match self.nodes.get_mut(i_node) {
            Some(nw) => nw,
            None     => return Err(NoSuchNode(i_node)),
        };

        let old = match nw.inputs.get_mut(i_chan) {
            Some(field) => mem::replace(field, Some((o_node, o_chan))),
            None        => return Err(NoSuchInput(i_node, i_chan)),
        };

        match old {
            Some(..) => Err(InputAlreadyPatched(i_node, i_chan)),
            None     => Ok(()),
        }
    }

    /// This performs a topological sort of the nodes in the graph to determine the order in which
    /// the nodes will do their processing (to ensure that each `Node` has had its inputs computed
    /// before it runs).
    pub fn compute_order(&mut self) -> Result<()> {
        let mut marked   = BitSet::new();
        let mut on_stack = BitSet::new();

        self.order.clear();

        for id in 0..self.nodes.len() {
            if !marked.contains(&id) {
                try!(self.topo_sort_visit(&mut marked, &mut on_stack, id));
            }
        }

        self.dirty = false;

        Ok(())
    }

    fn topo_sort_visit(&mut self, marked: &mut BitSet, on_stack: &mut BitSet, id: NodeID)
        -> Result<()> {

        assert!(id < self.nodes.len());

        if !on_stack.insert(id) {
            return Err(Error::CycleDetected);
        } else if !marked.contains(&id) {
            // This is... annoying.
            let inputs = self.nodes[id].inputs.clone();

            for input in inputs {
                match input {
                    Some((next_id, _)) => try!(self.topo_sort_visit(marked, on_stack, next_id)),
                    None               => return Err(Error::IncompleteGraph(id)),
                }
            }

            marked.insert(id);
            self.order.push(id);
        }

        on_stack.remove(&id);
        Ok(())
    }
}

/// Shorthand for the standard `Result` type with a `graph::Error` as the error type.
pub type Result<T> = ::std::result::Result<T, Error>;

/// An error manipulating a `Graph`.
#[derive(Debug)]
pub enum Error {
    /// The given `NodeID` doesn't refer to any node in this `Graph`.
    NoSuchNode(NodeID),

    /// The given `InputID` doesn't refer to any input channel in the corresponding `Node`.
    NoSuchInput(NodeID, InputID),

    /// The given input already has an output plugged into it.
    InputAlreadyPatched(NodeID, InputID),

    /// The given `OutputID` doesn't refer to any output channel in the corresponding `Node`.
    NoSuchOutput(NodeID, OutputID),

    /// There are nodes which have inputs without any output's patched into them.
    IncompleteGraph(NodeID),

    /// There is a cycle in the graph, this is not allowed.
    CycleDetected,
}
