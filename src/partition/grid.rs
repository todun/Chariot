// OpenAOE: An open source reimplementation of Age of Empires (1997)
// Copyright (c) 2016 Kevin Fuller
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::collections::HashMap;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CellKey {
    row: i32,
    col: i32,
}

impl CellKey {
    pub fn new(row: i32, col: i32) -> CellKey {
        CellKey {
            row: row,
            col: col,
        }
    }
}

struct Cell {
    entities: Vec<u32>,
}

impl Cell {
    fn new() -> Cell {
        Cell { entities: Vec::new() }
    }

    fn add(&mut self, entity_id: u32) {
        self.entities.push(entity_id);
    }

    fn remove(&mut self, entity_id: u32) {
        if let Some(index) = self.entities.iter().position(|id| *id == entity_id) {
            self.entities.swap_remove(index);
        }
    }

    fn entities<'a>(&'a self) -> &Vec<u32> {
        &self.entities
    }
}

pub struct GridPartition {
    cell_width: i32,
    cell_height: i32,
    entities: HashMap<u32, CellKey>,
    cells: HashMap<CellKey, Cell>,
}

/// Infinite grid spatial partition
impl GridPartition {
    pub fn new(cell_width: i32, cell_height: i32) -> GridPartition {
        GridPartition {
            cell_width: cell_width,
            cell_height: cell_height,
            entities: HashMap::new(),
            cells: HashMap::new(),
        }
    }

    /// Tells the grid where an entity is so that it can be queried later
    pub fn update_entity(&mut self, entity_id: u32, position: (i32, i32)) {
        let cell_key = self.cell_key(position);
        if !self.entities.contains_key(&entity_id) {
            self.entities.insert(entity_id, cell_key);
        } else {
            let old_cell_key = *self.entities.get(&entity_id).unwrap();
            self.remove_from_cell(old_cell_key, entity_id);
        }
        self.add_to_cell(cell_key, entity_id);
    }

    /// Returns the entity IDs that lie in the cells overlapped by the given bounds
    /// Note: the returned entity IDs can lie outside of the bounds
    pub fn query(&mut self, start_position: (i32, i32), end_position: (i32, i32)) -> Vec<u32> {
        let (start_row, start_col) = self.row_col(start_position);
        let (end_row, end_col) = self.row_col(end_position);

        let mut entities = Vec::new();
        for row in start_row..(end_row + 1) {
            for col in start_col..(end_col + 1) {
                entities.extend(self.cell_mut(CellKey::new(row, col)).entities().iter());
            }
        }
        entities
    }

    fn add_to_cell(&mut self, cell_key: CellKey, entity_id: u32) {
        self.cell_mut(cell_key).add(entity_id);
    }

    fn remove_from_cell(&mut self, cell_key: CellKey, entity_id: u32) {
        self.cell_mut(cell_key).remove(entity_id);
    }

    fn cell_mut<'a>(&'a mut self, cell_key: CellKey) -> &'a mut Cell {
        if !self.cells.contains_key(&cell_key) {
            self.cells.insert(cell_key, Cell::new());
        }
        self.cells.get_mut(&cell_key).unwrap()
    }

    fn cell_key(&self, position: (i32, i32)) -> CellKey {
        let row_col = self.row_col(position);
        CellKey::new(row_col.0, row_col.1)
    }

    fn row_col(&self, position: (i32, i32)) -> (i32, i32) {
        (position.1 / self.cell_height, position.0 / self.cell_width)
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, CellKey, GridPartition};

    #[test]
    fn test_cell_add_remove() {
        let mut cell = Cell::new();
        cell.remove(5); // shouldn't panic

        cell.add(4);
        assert_eq!(&vec![4], cell.entities());

        cell.add(5);
        cell.add(6);
        assert_eq!(&vec![4, 5, 6], cell.entities());

        cell.remove(4);
        assert_eq!(&vec![6, 5], cell.entities());
    }

    #[test]
    fn test_cell_key_from_position() {
        let grid = GridPartition::new(10, 10);
        assert_eq!(CellKey::new(0, 0), grid.cell_key((0, 0)));
        assert_eq!(CellKey::new(0, 0), grid.cell_key((5, 5)));
        assert_eq!(CellKey::new(0, 1), grid.cell_key((10, 5)));
        assert_eq!(CellKey::new(1, 0), grid.cell_key((5, 10)));
    }

    #[test]
    fn test_grid_update_entity() {
        let mut grid = GridPartition::new(10, 10);
        grid.update_entity(1, (5, 5));
        grid.update_entity(2, (15, 5));

        assert_eq!(&vec![1], grid.cell_mut(CellKey::new(0, 0)).entities());
        assert_eq!(&vec![2], grid.cell_mut(CellKey::new(0, 1)).entities());

        grid.update_entity(1, (25, 15));
        assert!(grid.cell_mut(CellKey::new(0, 0)).entities().is_empty());
        assert_eq!(&vec![2], grid.cell_mut(CellKey::new(0, 1)).entities());
        assert_eq!(&vec![1], grid.cell_mut(CellKey::new(1, 2)).entities());
    }

    #[test]
    fn test_grid_query() {
        let mut grid = GridPartition::new(10, 10);
        grid.update_entity(1, (5, 5));
        grid.update_entity(2, (6, 5));
        grid.update_entity(3, (15, 5));
        grid.update_entity(4, (5, 15));

        assert_eq!(vec![1, 2], grid.query((1, 1), (9, 9)));
        assert_eq!(vec![1, 2, 3, 4], grid.query((0, 0), (20, 20)));
        assert_eq!(vec![1, 2, 3, 4], grid.query((9, 0), (20, 10)));
        assert_eq!(vec![3], grid.query((10, 0), (20, 10)));
    }
}
