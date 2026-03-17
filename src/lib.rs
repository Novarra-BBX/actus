// Copyright (C) 2019-2025 Alpha-Delta Network Inc.
// This file is part of the actus library.

// The actus library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The actus library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the actus library. If not, see <https://www.gnu.org/licenses/>.

//! ACTUS DEX Integration — S4.
//!
//! Tokenized ACTUS positions and DEX order book for Alpha-Delta protocol.

pub mod order_book;
pub mod tokenized;

pub use order_book::OrderBook;
pub use tokenized::{TokenizedPosition, TokenizedPositionRegistry};
