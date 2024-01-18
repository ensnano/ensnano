/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#[derive(Clone, Debug)]
pub struct ClicCounter {
    value: u32,
}

impl ClicCounter {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn set(&mut self, value: u32) {
        self.value = value
    }

    pub fn next(&mut self) -> u32 {
        let ret = self.value;
        self.value += 1;
        return ret;
    }

    pub fn count(&self) -> u32 {
        return self.value;
    }

    pub fn last(&self) -> Option<u32> {
        if self.value > 0 {
            return Some(self.value - 1);
        }
        return None;
    }
}
